use axum::extract::rejection::QueryRejection;
use axum::extract::{Path, Query, State};
use axum::response::{IntoResponse, Json, Redirect};

use crate::json::ChartLeaderboardJson;
use crate::models::{
	ChartDetailRow, ChartListItem, ChartListRow, ChartMeta, ChartTableMembership, ChartTableRow,
	PbDbRow, PbEntry,
};
use crate::query::{total_pages, ChartQuery, ListQuery, PER_PAGE};
use crate::response::{bad_request, not_found, render};
use crate::state::AppState;
use crate::templates::{ChartTemplate, ChartsTemplate};
use crate::util::{bms_grade_delta, clear_pct, display_player_name, fts_query, is_md5, score_pct};

pub async fn charts_list(
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

	let (total, db_rows): (i64, Vec<ChartListRow>) = if is_md5(&q) {
		// ── Exact MD5 lookup ────────────────────────────────────────────────
		let row: Option<ChartListRow> = sqlx::query_as(
			"SELECT md5, title, artist, genre, keys, level, play_count, play_people, clear_people \
			 FROM chart WHERE md5 = ?",
		)
		.bind(&q)
		.fetch_optional(state.db.as_ref())
		.await
		.unwrap_or(None);
		let rows = row.into_iter().collect::<Vec<_>>();
		(rows.len() as i64, rows)
	} else if let Some(fts_q) = fts_query(&q) {
		// ── FTS5 full-text search ────────────────────────────────────────────
		let total: i64 =
			sqlx::query_scalar("SELECT COUNT(*) FROM chart_fts WHERE chart_fts MATCH ?")
				.bind(&fts_q)
				.fetch_one(state.db.as_ref())
				.await
				.unwrap_or(0);

		let rows: Vec<ChartListRow> = sqlx::query_as(
			r#"SELECT c.md5, c.title, c.artist, c.genre, c.keys, c.level,
					  c.play_count, c.play_people, c.clear_people
			   FROM chart c
			   JOIN (SELECT md5 FROM chart_fts WHERE chart_fts MATCH ?) AS fts
				 USING (md5)
			   ORDER BY c.play_count DESC
			   LIMIT ? OFFSET ?"#,
		)
		.bind(&fts_q)
		.bind(PER_PAGE)
		.bind(offset)
		.fetch_all(state.db.as_ref())
		.await
		.unwrap_or_default();

		(total, rows)
	} else {
		// ── No filter – all charts ───────────────────────────────────────────
		let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM chart")
			.fetch_one(state.db.as_ref())
			.await
			.unwrap_or(0);

		let rows: Vec<ChartListRow> = sqlx::query_as(
			r#"SELECT md5, title, artist, genre, keys, level,
					  play_count, play_people, clear_people
			   FROM chart
			   ORDER BY play_count DESC
			   LIMIT ? OFFSET ?"#,
		)
		.bind(PER_PAGE)
		.bind(offset)
		.fetch_all(state.db.as_ref())
		.await
		.unwrap_or_default();

		(total, rows)
	};

	let charts = db_rows
		.into_iter()
		.map(|r| ChartListItem {
			clear_pct: clear_pct(r.clear_people, r.play_people),
			md5: r.md5,
			title: r.title,
			artist: r.artist,
			genre: r.genre,
			keys: r.keys,
			level: r.level,
			play_count: r.play_count,
			play_people: r.play_people,
		})
		.collect();

	render(ChartsTemplate {
		charts,
		page,
		total_pages: total_pages(total),
		total,
		q,
	})
}

/// Shared DB fetch for both the HTML and JSON chart detail handlers.
/// Returns `None` when the MD5 is not found.
/// `input_filter` — when `Some`, restricts rows to entries whose `input` column starts with that
/// prefix (e.g. `"BM"` or `"KB"`).
async fn fetch_chart_detail(
	pool: &sqlx::SqlitePool,
	md5: &str,
	page: u32,
	input_filter: Option<&str>,
) -> Option<(ChartDetailRow, Vec<PbDbRow>, i64)> {
	let offset = (page as i64 - 1) * PER_PAGE;

	let chart_row: ChartDetailRow = sqlx::query_as(
		r#"SELECT md5, bmsid, title, genre, artist, bpm_min, bpm_max, level, keys, judge_rank,
				  play_count, play_people, clear_count, clear_people,
				  fc_count, hard_count, normal_count, easy_count, failed_count,
				  last_updated_by, last_updated_at, body_url, diff_url, comment,
				  tag_1, tag_2, tag_3, tag_4, tag_5,
				  tag_6, tag_7, tag_8, tag_9, tag_10,
				  suspended
		   FROM chart WHERE md5 = ?"#,
	)
	.bind(md5)
	.fetch_optional(pool)
	.await
	.unwrap_or(None)?;

	let total_rows: i64 =
		sqlx::query_scalar("SELECT COUNT(*) FROM pb WHERE md5 = ?1 AND (?2 IS NULL OR input = ?2)")
			.bind(md5)
			.bind(input_filter)
			.fetch_one(pool)
			.await
			.unwrap_or(0);

	let pb_rows: Vec<PbDbRow> = sqlx::query_as(
		r#"SELECT p.rank, p.player_id, p.player_name, p.dan, p.clear_type, p.letter_rank,
				  p.score, p.score_max, p.combo, p.combo_max, p.bad_poor,
				  p.pgreat, p.great, p.good, p.bad, p.poor,
				  p.option_1, p.option_2, p.option_3, p.option_4, p.input, p.client, p.note, p.is_cheated,
				  EXISTS(SELECT 1 FROM ghost g WHERE g.md5 = p.md5 AND g.player_id = p.player_id) AS has_ghost
		   FROM pb p WHERE p.md5 = ?1 AND (?2 IS NULL OR p.input = ?2)
		   ORDER BY p.rank ASC
		   LIMIT ?3 OFFSET ?4"#,
	)
	.bind(md5)
	.bind(input_filter)
	.bind(PER_PAGE)
	.bind(offset)
	.fetch_all(pool)
	.await
	.unwrap_or_default();

	Some((chart_row, pb_rows, total_rows))
}

pub async fn chart_detail(
	State(state): State<AppState>,
	Path(md5): Path<String>,
	query: Result<Query<ChartQuery>, QueryRejection>,
) -> impl IntoResponse {
	let Query(params) = match query {
		Ok(q) => q,
		Err(_) => return bad_request("'page' must be a positive integer."),
	};
	if let Some(ref inp) = params.input {
		if !matches!(inp.as_str(), "BM" | "KB") {
			return bad_request(&format!(
				"Invalid input filter '{inp}'. Allowed values: BM, KB."
			));
		}
	}
	let page = params.page.max(1);
	let input_filter = params.input.as_deref().filter(|s| !s.is_empty());

	let (chart_row, pb_rows, total_rows) =
		match fetch_chart_detail(state.db.as_ref(), &md5, page, input_filter).await {
			Some(d) => d,
			None => return not_found("No chart found with that MD5."),
		};

	let tags: Vec<String> = [
		&chart_row.tag_1,
		&chart_row.tag_2,
		&chart_row.tag_3,
		&chart_row.tag_4,
		&chart_row.tag_5,
		&chart_row.tag_6,
		&chart_row.tag_7,
		&chart_row.tag_8,
		&chart_row.tag_9,
		&chart_row.tag_10,
	]
	.iter()
	.filter_map(|t| t.as_ref().filter(|s| !s.is_empty()).cloned())
	.collect();

	let meta = ChartMeta {
		clear_pct: clear_pct(chart_row.clear_people, chart_row.play_people),
		tags,
		suspended: chart_row.suspended != 0,
		last_updated_by: chart_row.last_updated_by.unwrap_or_default(),
		last_updated_at: chart_row.last_updated_at.unwrap_or_default(),
		body_url: chart_row.body_url,
		diff_url: chart_row.diff_url,
		comment: chart_row.comment,
		md5: chart_row.md5,
		bmsid: chart_row.bmsid,
		title: chart_row.title,
		genre: chart_row.genre,
		artist: chart_row.artist,
		bpm_min: chart_row.bpm_min,
		bpm_max: chart_row.bpm_max,
		level: chart_row.level,
		keys: chart_row.keys,
		judge_rank: chart_row.judge_rank,
		play_count: chart_row.play_count,
		play_people: chart_row.play_people,
		clear_count: chart_row.clear_count,
		clear_people: chart_row.clear_people,
		fc_count: chart_row.fc_count,
		hard_count: chart_row.hard_count,
		normal_count: chart_row.normal_count,
		easy_count: chart_row.easy_count,
		failed_count: chart_row.failed_count,
	};

	let rows = pb_rows
		.into_iter()
		.map(|r| PbEntry {
			score_display: format!(
				"{}/{}\n{}",
				r.score,
				r.score_max,
				score_pct(r.score, r.score_max)
			),
			grade_delta: bms_grade_delta(r.score, r.score_max),
			combo_display: format!("{}/{}", r.combo, r.combo_max),
			rank: r.rank,
			player_id: r.player_id,
			player_name: display_player_name(&r.player_name),
			dan: r.dan,
			clear_class: r.clear_type.trim_start_matches('★').to_string(),
			clear_type: r.clear_type,
			letter_rank: r.letter_rank,
			bad_poor: r.bad_poor,
			pgreat: r.pgreat,
			great: r.great,
			good: r.good,
			bad: r.bad,
			poor: r.poor,
			option_1: r.option_1,
			option_2: r.option_2,
			option_3: r.option_3,
			option_4: r.option_4,
			input: r.input,
			client: r.client,
			note: r.note,
			has_ghost: r.has_ghost,
			is_cheated: r.is_cheated != 0,
		})
		.collect();

	let table_memberships: Vec<ChartTableMembership> = if let Some(ti) = &state.tableinfo {
		sqlx::query_as::<_, ChartTableRow>(
			"SELECT tm.table_id, tm.name, tm.symbol, tl.level
				 FROM table_level tl
				 JOIN table_main tm ON tm.table_id = tl.table_id
				 WHERE tl.md5 = ?
				 ORDER BY tm.name, tl.level",
		)
		.bind(&md5)
		.fetch_all(ti.as_ref())
		.await
		.unwrap_or_default()
		.into_iter()
		.map(|r| ChartTableMembership {
			table_id: r.table_id,
			name: r.name,
			symbol: r.symbol,
			level: r.level,
		})
		.collect()
	} else {
		vec![]
	};

	let input_filter_str = input_filter.unwrap_or("").to_string();
	let input_qs = if input_filter_str.is_empty() {
		String::new()
	} else {
		format!("&input={}", input_filter_str)
	};
	render(ChartTemplate {
		meta,
		rows,
		page,
		total_pages: total_pages(total_rows),
		total_rows,
		table_memberships,
		input_filter: input_filter_str,
		input_qs,
	})
}

pub async fn chart_by_bmsid(
	State(state): State<AppState>,
	Path(bmsid): Path<i64>,
) -> impl IntoResponse {
	let md5: Option<String> = sqlx::query_scalar("SELECT md5 FROM chart WHERE bmsid = ? LIMIT 1")
		.bind(bmsid)
		.fetch_optional(state.db.as_ref())
		.await
		.unwrap_or(None);

	match md5 {
		Some(md5) => Redirect::to(&format!("/charts/{md5}")).into_response(),
		None => not_found(&format!("No chart found with bmsid {bmsid}.")),
	}
}

pub async fn chart_detail_json(
	State(state): State<AppState>,
	Path(md5): Path<String>,
	query: Result<Query<ChartQuery>, QueryRejection>,
) -> impl IntoResponse {
	let Query(params) = match query {
		Ok(q) => q,
		Err(_) => return bad_request("'page' must be a positive integer."),
	};
	if let Some(ref inp) = params.input {
		if !matches!(inp.as_str(), "BM" | "KB") {
			return bad_request(&format!(
				"Invalid input filter '{inp}'. Allowed values: BM, KB."
			));
		}
	}
	let page = params.page.max(1);
	let input_filter = params.input.as_deref().filter(|s| !s.is_empty());

	match fetch_chart_detail(state.db.as_ref(), &md5, page, input_filter).await {
		None => not_found("No chart found with that MD5."),
		Some((chart, mut leaderboard, total_rows)) => {
			for row in &mut leaderboard {
				row.player_name = display_player_name(&row.player_name);
			}
			Json(ChartLeaderboardJson {
				chart,
				leaderboard,
				page,
				total_pages: total_pages(total_rows),
				total_rows,
			})
			.into_response()
		}
	}
}
