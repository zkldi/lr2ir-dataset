use axum::extract::rejection::QueryRejection;
use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;

use crate::handlers::player_tables::fetch_player_table_list;
use crate::models::{
	BbsItem, PlayItem, PlayerDetail, PlayerListItem, RivalItem, UserBbsRow, UserDetailRow,
	UserListRow, UserPlayRow, UserRivalRow,
};
use crate::query::{total_pages, ListQuery, PER_PAGE};
use crate::response::{bad_request, not_found, render};
use crate::state::AppState;
use crate::templates::{PlayerTemplate, PlayersTemplate};
use crate::util::{clear_css_class, display_player_name, display_player_name_opt};

pub async fn players_list(
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

	let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM user WHERE name LIKE ?")
		.bind(&pattern)
		.fetch_one(state.db.as_ref())
		.await
		.unwrap_or(0);

	let db_rows: Vec<UserListRow> = sqlx::query_as(
		r#"SELECT player_id, name, dan, play_count, fc_count, privacy_level, is_cheater
		   FROM user
		   WHERE name LIKE ?1
		   ORDER BY COALESCE(play_count, 0) DESC
		   LIMIT ?2 OFFSET ?3"#,
	)
	.bind(&pattern)
	.bind(PER_PAGE)
	.bind(offset)
	.fetch_all(state.db.as_ref())
	.await
	.unwrap_or_default();

	let players = db_rows
		.into_iter()
		.map(|r| PlayerListItem {
			player_id: r.player_id,
			name: display_player_name_opt(r.name.as_deref(), r.player_id),
			dan: r.dan.unwrap_or_default(),
			play_count: r.play_count.unwrap_or(0),
			fc_count: r.fc_count.unwrap_or(0),
			privacy_level: r.privacy_level.unwrap_or_default(),
			is_cheater: r.is_cheater.unwrap_or(0) != 0,
		})
		.collect();

	render(PlayersTemplate {
		players,
		row_start: ((page - 1) as usize) * PER_PAGE as usize + 1,
		page,
		total_pages: total_pages(total),
		total,
		q,
	})
}

pub async fn player_detail(
	State(state): State<AppState>,
	Path(player_id): Path<i64>,
) -> impl IntoResponse {
	let maybe_user: Option<UserDetailRow> = sqlx::query_as(
		r#"SELECT player_id, name, dan, bio, privacy_level, songs_played, play_count,
				  fc_count, perfect_fc_count, hard_count, normal_count, easy_count, failed_count,
				  is_cheater
		   FROM user WHERE player_id = ?"#,
	)
	.bind(player_id)
	.fetch_optional(state.db.as_ref())
	.await
	.unwrap_or(None);

	let user = match maybe_user {
		Some(u) => u,
		None => return not_found("No player found with that ID."),
	};

	let player_id = user.player_id;

	let (most_plays, recent_plays, rivals, bbs, tables) = tokio::join!(
		sqlx::query_as::<_, UserPlayRow>(
			"SELECT p.title, p.clear_type, p.play_count, p.rank_pos, p.rank_total, c.md5 \
			 FROM user_most_plays p \
			 LEFT JOIN chart c ON c.bmsid = p.bmsid \
			 WHERE p.player_id = ? ORDER BY p.pos ASC"
		)
		.bind(player_id)
		.fetch_all(state.db.as_ref()),
		sqlx::query_as::<_, UserPlayRow>(
			"SELECT p.title, p.clear_type, p.play_count, p.rank_pos, p.rank_total, c.md5 \
			 FROM user_recent_plays p \
			 LEFT JOIN chart c ON c.bmsid = p.bmsid \
			 WHERE p.player_id = ? ORDER BY p.pos ASC"
		)
		.bind(player_id)
		.fetch_all(state.db.as_ref()),
		sqlx::query_as::<_, UserRivalRow>(
			"SELECT rival_id, rival_name FROM user_rival WHERE player_id = ? ORDER BY rival_id"
		)
		.bind(player_id)
		.fetch_all(state.db.as_ref()),
		sqlx::query_as::<_, UserBbsRow>(
			"SELECT commenter_id, commenter_name, message, posted_at \
			 FROM user_bbs WHERE player_id = ? ORDER BY pos ASC"
		)
		.bind(player_id)
		.fetch_all(state.db.as_ref()),
		fetch_player_table_list(&state, player_id),
	);

	let most_plays = most_plays.unwrap_or_default();
	let recent_plays = recent_plays.unwrap_or_default();
	let rivals = rivals.unwrap_or_default();
	let bbs = bbs.unwrap_or_default();

	let hard_count = user.hard_count.unwrap_or(0);
	let normal_count = user.normal_count.unwrap_or(0);
	let easy_count = user.easy_count.unwrap_or(0);
	let failed_count = user.failed_count.unwrap_or(0);
	let fc_count = user.fc_count.unwrap_or(0);
	let perfect_fc_count = user.perfect_fc_count.unwrap_or(0);
	let total_clears = fc_count + hard_count + normal_count + easy_count;

	let profile = PlayerDetail {
		player_id,
		name: display_player_name_opt(user.name.as_deref(), player_id),
		dan: user.dan.unwrap_or_default(),
		bio: user.bio.unwrap_or_default(),
		privacy_level: user.privacy_level.unwrap_or_default(),
		songs_played: user.songs_played.unwrap_or(0),
		play_count: user.play_count.unwrap_or(0),
		fc_count,
		perfect_fc_count,
		hard_count,
		normal_count,
		easy_count,
		failed_count,
		total_clears,
		is_cheater: user.is_cheater.unwrap_or(0) != 0,
	};

	render(PlayerTemplate {
		profile,
		tables,
		most_plays: most_plays
			.into_iter()
			.map(|r| {
				let clear_type = r.clear_type.unwrap_or_default();
				PlayItem {
					title: r.title.unwrap_or_default(),
					clear_class: clear_css_class(&clear_type),
					clear_type,
					play_count: r.play_count.unwrap_or(0),
					rank_pos: r.rank_pos.unwrap_or(0),
					rank_total: r.rank_total.unwrap_or(0),
					md5: r.md5,
				}
			})
			.collect(),
		recent_plays: recent_plays
			.into_iter()
			.map(|r| {
				let clear_type = r.clear_type.unwrap_or_default();
				PlayItem {
					title: r.title.unwrap_or_default(),
					clear_class: clear_css_class(&clear_type),
					clear_type,
					play_count: r.play_count.unwrap_or(0),
					rank_pos: r.rank_pos.unwrap_or(0),
					rank_total: r.rank_total.unwrap_or(0),
					md5: r.md5,
				}
			})
			.collect(),
		rivals: rivals
			.into_iter()
			.map(|r| RivalItem {
				rival_id: r.rival_id.unwrap_or(0),
				rival_name: display_player_name(r.rival_name.as_deref().unwrap_or_default()),
			})
			.collect(),
		bbs: bbs
			.into_iter()
			.map(|b| BbsItem {
				player_id: b.commenter_id.unwrap_or(0),
				commenter_name: display_player_name(
					b.commenter_name.as_deref().unwrap_or_default(),
				),
				message: b.message.unwrap_or_default(),
				posted_at: b.posted_at.unwrap_or_default(),
			})
			.collect(),
	})
}
