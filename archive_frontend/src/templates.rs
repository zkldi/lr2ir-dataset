use askama::Template;

use crate::models::{
	BbsItem, ChartListItem, ChartMeta, ChartTableMembership, CourseListItem, CourseMetaDisplay,
	CourseRankingEntry, CourseStageItem, PbEntry, PlayItem, PlayerBrief, PlayerDetail,
	PlayerListItem, PlayerTableChartItem, PlayerTableLevelSummary, PlayerTableListItem, RivalItem,
	TableItem, TableLevelChartItem, TableLevelItem,
};

#[derive(Template)]
#[template(path = "index.html.jinja")]
pub struct IndexTemplate {
	pub charts: String,
	pub scores: String,
	pub players: String,
	pub ghosts: String,
	pub bbs: String,
}

#[derive(Template)]
#[template(path = "error.html.jinja")]
pub struct ErrorTemplate {
	pub status: u16,
	pub title: String,
	pub message: String,
}

#[derive(Template)]
#[template(path = "charts.html.jinja")]
pub struct ChartsTemplate {
	pub charts: Vec<ChartListItem>,
	pub page: u32,
	pub total_pages: u32,
	pub total: i64,
	pub q: String,
}

#[derive(Template)]
#[template(path = "chart.html.jinja")]
pub struct ChartTemplate {
	pub meta: ChartMeta,
	pub rows: Vec<PbEntry>,
	pub page: u32,
	pub total_pages: u32,
	pub total_rows: i64,
	pub table_memberships: Vec<ChartTableMembership>,
	/// Active input filter, e.g. `"BM"` or `"KB"`. Empty string means no filter.
	pub input_filter: String,
	/// Ready-made query-string fragment to append to pagination links, e.g. `"&input=BM"` or `""`.
	pub input_qs: String,
}

#[derive(Template)]
#[template(path = "players.html.jinja")]
pub struct PlayersTemplate {
	pub players: Vec<PlayerListItem>,
	pub page: u32,
	pub total_pages: u32,
	pub total: i64,
	pub q: String,
	pub row_start: usize,
}

#[derive(Template)]
#[template(path = "player.html.jinja")]
pub struct PlayerTemplate {
	pub profile: PlayerDetail,
	pub tables: Vec<PlayerTableListItem>,
	pub most_plays: Vec<PlayItem>,
	pub recent_plays: Vec<PlayItem>,
	pub rivals: Vec<RivalItem>,
	pub bbs: Vec<BbsItem>,
}

#[derive(Template)]
#[template(path = "player_table.html.jinja")]
pub struct PlayerTableTemplate {
	pub player: PlayerBrief,
	pub table: TableItem,
	pub levels: Vec<PlayerTableLevelSummary>,
}

#[derive(Template)]
#[template(path = "player_table_level.html.jinja")]
pub struct PlayerTableLevelTemplate {
	pub player: PlayerBrief,
	pub table: TableItem,
	pub level: String,
	pub charts: Vec<PlayerTableChartItem>,
}

#[derive(Template)]
#[template(path = "tables.html.jinja")]
pub struct TablesTemplate {
	pub tables: Vec<TableItem>,
	pub rendered_at: Option<String>,
}

#[derive(Template)]
#[template(path = "table.html.jinja")]
pub struct TableTemplate {
	pub table: TableItem,
	pub levels: Vec<TableLevelItem>,
}

#[derive(Template)]
#[template(path = "table_level.html.jinja")]
pub struct TableLevelTemplate {
	pub table: TableItem,
	pub level: String,
	pub charts: Vec<TableLevelChartItem>,
}

#[derive(Template)]
#[template(path = "courses.html.jinja")]
pub struct CoursesTemplate {
	pub courses: Vec<CourseListItem>,
	pub page: u32,
	pub total_pages: u32,
	pub total: i64,
	pub q: String,
}

#[derive(Template)]
#[template(path = "bbs.html.jinja")]
pub struct BbsTemplate {
	pub messages: Vec<BbsItem>,
	pub page: u32,
	pub total_pages: u32,
	pub total: i64,
	pub q: String,
	pub row_start: usize,
}

#[derive(Template)]
#[template(path = "course.html.jinja")]
pub struct CourseTemplate {
	pub meta: CourseMetaDisplay,
	pub stages: Vec<CourseStageItem>,
	pub rows: Vec<CourseRankingEntry>,
	pub page: u32,
	pub total_pages: u32,
	pub total_rows: i64,
}
