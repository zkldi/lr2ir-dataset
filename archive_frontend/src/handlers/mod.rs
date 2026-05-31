mod bbs;
mod charts;
mod courses;
mod ghosts;
mod player_tables;
mod players;
mod tables;

pub use bbs::bbs_list;
pub use charts::{chart_detail, chart_detail_json, charts_list};
pub use courses::{course_detail, courses_list};
pub use ghosts::ghost_download;
pub use player_tables::{fetch_player_table_list, player_table_level, player_table_summary};
pub use players::{player_detail, players_list};
pub use tables::{handle_not_found, table_detail, table_level, tables_list};

use axum::extract::State;
use axum::response::IntoResponse;

use crate::response::render;
use crate::state::AppState;
use crate::templates::IndexTemplate;
use crate::util::fmt_num;

pub async fn index(State(state): State<AppState>) -> impl IntoResponse {
	render(IndexTemplate {
		charts: fmt_num(state.stats.charts),
		scores: fmt_num(state.stats.scores),
		players: fmt_num(state.stats.players),
		ghosts: fmt_num(state.stats.ghosts),
		bbs: fmt_num(state.stats.bbs),
	})
}
