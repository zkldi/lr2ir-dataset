use axum::extract::rejection::QueryRejection;
use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;

use crate::models::{
	CourseDetailRow, CourseListItem, CourseListRow, CourseMetaDisplay, CourseRankingEntry,
	CourseRankingRow, CourseStageItem, CourseStageRow,
};
use crate::query::{total_pages, ListQuery, PER_PAGE};
use crate::response::{bad_request, not_found, render};
use crate::state::AppState;
use crate::templates::{CourseTemplate, CoursesTemplate};
use crate::util::{bms_grade_delta, clear_pct, display_player_name, score_pct};

pub async fn courses_list(
	State(state): State<AppState>,
	query: Result<Query<ListQuery>, QueryRejection>,
) -> impl IntoResponse {
	let Query(params) = match query {
		Ok(q) => q,
		Err(_) => return bad_request("'page' must be a positive integer."),
	};
	let page = params.page.max(1);
	let offset = (page as i64 - 1) * PER_PAGE;
	let q = params.q.trim().to_string();
	let pattern = format!("%{}%", q);

	let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM course WHERE title LIKE ?")
		.bind(&pattern)
		.fetch_one(state.db.as_ref())
		.await
		.unwrap_or(0);

	let db_rows: Vec<CourseListRow> = sqlx::query_as(
		r#"SELECT course_id, title, category, creator_name, keys,
				  play_count, play_people, clear_people
		   FROM course
		   WHERE title LIKE ?1
		   ORDER BY COALESCE(play_people, 0) DESC
		   LIMIT ?2 OFFSET ?3"#,
	)
	.bind(&pattern)
	.bind(PER_PAGE)
	.bind(offset)
	.fetch_all(state.db.as_ref())
	.await
	.unwrap_or_default();

	let courses = db_rows
		.into_iter()
		.map(|r| CourseListItem {
			course_id: r.course_id,
			title: r.title.unwrap_or_default(),
			category: r.category.unwrap_or_default(),
			creator_name: display_player_name(r.creator_name.as_deref().unwrap_or_default()),
			keys: r.keys.unwrap_or_default(),
			play_count: r.play_count.unwrap_or(0),
			play_people: r.play_people.unwrap_or(0),
			clear_pct: clear_pct(r.clear_people.unwrap_or(0), r.play_people.unwrap_or(0)),
		})
		.collect();

	render(CoursesTemplate {
		courses,
		page,
		total_pages: total_pages(total),
		total,
		q,
	})
}

pub async fn course_detail(
	State(state): State<AppState>,
	Path(course_id): Path<i64>,
	query: Result<Query<ListQuery>, QueryRejection>,
) -> impl IntoResponse {
	let Query(params) = match query {
		Ok(q) => q,
		Err(_) => return bad_request("'page' must be a positive integer."),
	};
	let page = params.page.max(1);
	let offset = (page as i64 - 1) * PER_PAGE;

	let maybe_course: Option<CourseDetailRow> = sqlx::query_as(
		r#"SELECT title, category, creator_id, creator_name, keys,
				  play_count, play_people, clear_count, clear_people,
				  fc_count, hard_count, normal_count, easy_count, failed_count,
				  hash
		   FROM course WHERE course_id = ?"#,
	)
	.bind(course_id)
	.fetch_optional(state.db.as_ref())
	.await
	.unwrap_or(None);

	let course = match maybe_course {
		Some(c) => c,
		None => return not_found("Course not found."),
	};

	let stages: Vec<CourseStageRow> = sqlx::query_as(
		r#"SELECT cs.stage, cs.label, c.md5
		   FROM course_stage cs
		   LEFT JOIN chart c ON c.bmsid = cs.bmsid
		   WHERE cs.course_id = ?
		   ORDER BY cs.stage ASC"#,
	)
	.bind(course_id)
	.fetch_all(state.db.as_ref())
	.await
	.unwrap_or_default();

	let total_rows: i64 =
		sqlx::query_scalar("SELECT COUNT(*) FROM course_ranking WHERE course_id = ?")
			.bind(course_id)
			.fetch_one(state.db.as_ref())
			.await
			.unwrap_or(0);

	let ranking_rows: Vec<CourseRankingRow> = sqlx::query_as(
		r#"SELECT rank, player_id, player_name, dan, clear_type, letter_rank,
				  score, score_max, combo, combo_max, bad_poor,
				  pgreat, great, good, bad, poor,
				  option_1, option_2, option_3, option_4, input, client, note, is_cheated
		   FROM course_ranking
		   WHERE course_id = ?
		   ORDER BY rank ASC
		   LIMIT ? OFFSET ?"#,
	)
	.bind(course_id)
	.bind(PER_PAGE)
	.bind(offset)
	.fetch_all(state.db.as_ref())
	.await
	.unwrap_or_default();

	let fc_count = course.fc_count.unwrap_or(0);
	let hard_count = course.hard_count.unwrap_or(0);
	let normal_count = course.normal_count.unwrap_or(0);
	let easy_count = course.easy_count.unwrap_or(0);
	let failed_count = course.failed_count.unwrap_or(0);
	let total_clears = fc_count + hard_count + normal_count + easy_count;
	let play_people = course.play_people.unwrap_or(0);
	let clear_people = course.clear_people.unwrap_or(0);

	let meta = CourseMetaDisplay {
		course_id,
		title: course.title.unwrap_or_default(),
		category: course.category.unwrap_or_default(),
		creator_id: course.creator_id,
		creator_name: display_player_name(course.creator_name.as_deref().unwrap_or_default()),
		is_dp: {
 			let k = course.keys.as_deref().unwrap_or("");
    		k.contains("14") || k.contains("10")
		},
		keys: course.keys.unwrap_or_default(),
		play_count: course.play_count.unwrap_or(0),
		play_people,
		clear_count: course.clear_count.unwrap_or(0),
		clear_people,
		fc_count,
		hard_count,
		normal_count,
		easy_count,
		failed_count,
		total_clears,
		clear_pct: clear_pct(clear_people, play_people),
		hash: course.hash.unwrap_or_default(),
	};

	let stages = stages
		.into_iter()
		.map(|s| CourseStageItem {
			stage: s.stage,
			label: s.label.unwrap_or_default(),
			md5: s.md5,
		})
		.collect();

	let rows = ranking_rows
		.into_iter()
		.map(|r| {
			let score = r.score.unwrap_or(0);
			let score_max = r.score_max.unwrap_or(0);
			let combo = r.combo.unwrap_or(0);
			let combo_max = r.combo_max.unwrap_or(0);
			let clear_type = r.clear_type.unwrap_or_default();
			CourseRankingEntry {
				rank: r.rank,
				player_id: r.player_id,
				player_name: display_player_name(r.player_name.as_deref().unwrap_or_default()),
				dan: r.dan.unwrap_or_default(),
				clear_class: clear_type.trim_start_matches('★').to_string(),
				clear_type,
				letter_rank: r.letter_rank.unwrap_or_default(),
				grade_delta: bms_grade_delta(score, score_max),
				score_display: format!("{}/{}\n{}", score, score_max, score_pct(score, score_max)),
				combo_display: format!("{}/{}", combo, combo_max),
				bad_poor: r.bad_poor.unwrap_or(0),
				pgreat: r.pgreat.unwrap_or(0),
				great: r.great.unwrap_or(0),
				good: r.good.unwrap_or(0),
				bad: r.bad.unwrap_or(0),
				poor: r.poor.unwrap_or(0),
				option_1: r.option_1.unwrap_or_default(),
				option_2: r.option_2.unwrap_or_default(),
				option_3: r.option_3.unwrap_or_default(),
				option_4: r.option_4.unwrap_or_default(),
				input: r.input.unwrap_or_default(),
				client: r.client.unwrap_or_default(),
				note: r.note.unwrap_or_default(),
				is_cheated: r.is_cheated.unwrap_or(0) != 0,
			}
		})
		.collect();

	render(CourseTemplate {
		meta,
		stages,
		rows,
		page,
		total_pages: total_pages(total_rows),
		total_rows,
	})
}
