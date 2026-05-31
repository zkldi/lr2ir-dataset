use crate::models::ChartDetailRow;
use crate::models::PbDbRow;

#[derive(serde::Serialize)]
pub struct ChartLeaderboardJson {
	pub chart: ChartDetailRow,
	pub leaderboard: Vec<PbDbRow>,
	pub page: u32,
	pub total_pages: u32,
	pub total_rows: i64,
}
