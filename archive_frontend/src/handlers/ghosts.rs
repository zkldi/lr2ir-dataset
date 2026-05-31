use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;

use crate::state::AppState;

pub async fn ghost_download(
	State(state): State<AppState>,
	Path((md5, player_id)): Path<(String, i64)>,
) -> impl IntoResponse {
	let ghost_data: Option<Vec<u8>> =
		sqlx::query_scalar("SELECT ghost FROM ghost WHERE md5 = ? AND player_id = ? LIMIT 1")
			.bind(&md5)
			.bind(player_id)
			.fetch_optional(state.db.as_ref())
			.await
			.unwrap_or(None);

	match ghost_data {
		Some(data) => ([("Content-Type", "application/octet-stream")], data).into_response(),
		None => StatusCode::NOT_FOUND.into_response(),
	}
}
