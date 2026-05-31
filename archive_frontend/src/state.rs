use sqlx::SqlitePool;
use std::sync::Arc;

#[derive(Clone)]
pub struct SiteStats {
	pub charts: i64,
	pub scores: i64,
	pub players: i64,
	pub ghosts: i64,
	pub bbs: i64,
}

#[derive(Clone)]
pub struct AppState {
	pub db: Arc<SqlitePool>,
	pub tableinfo: Option<Arc<SqlitePool>>,
	pub stats: Arc<SiteStats>,
}
