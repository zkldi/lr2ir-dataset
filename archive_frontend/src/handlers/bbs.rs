use axum::extract::rejection::QueryRejection;
use axum::extract::{Query, State};
use axum::response::IntoResponse;

use crate::models::{BbsItem, BbsListRow};
use crate::query::{total_pages, ListQuery, PER_PAGE};
use crate::response::{bad_request, render};
use crate::state::AppState;
use crate::templates::BbsTemplate;
use crate::util::display_player_name;

pub async fn bbs_list(
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

	let total: i64 = sqlx::query_scalar(
		r#"SELECT COUNT(*)
		   FROM bbs b
		   LEFT JOIN user u ON u.player_id = b.playerid
		   WHERE b.message LIKE ?1 OR COALESCE(u.name, '') LIKE ?1"#,
	)
	.bind(&pattern)
	.fetch_one(state.db.as_ref())
	.await
	.unwrap_or(0);

	let db_rows: Vec<BbsListRow> = sqlx::query_as(
		r#"SELECT b.msgid, b.playerid, b.message, b.time, u.name
		   FROM bbs b
		   LEFT JOIN user u ON u.player_id = b.playerid
		   WHERE b.message LIKE ?1 OR COALESCE(u.name, '') LIKE ?1
		   ORDER BY b.time DESC
		   LIMIT ?2 OFFSET ?3"#,
	)
	.bind(&pattern)
	.bind(PER_PAGE)
	.bind(offset)
	.fetch_all(state.db.as_ref())
	.await
	.unwrap_or_default();

	let messages = db_rows
		.into_iter()
		.map(|r| BbsItem {
			player_id: r.playerid,
			commenter_name: display_player_name(r.name.as_deref().unwrap_or_default()),
			message: r.message.unwrap_or_default(),
			posted_at: r.time.unwrap_or_default(),
		})
		.collect();

	render(BbsTemplate {
		messages,
		row_start: ((page - 1) as usize) * PER_PAGE as usize + 1,
		page,
		total_pages: total_pages(total),
		total,
		q,
	})
}
