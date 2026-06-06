use sqlx::FromRow;

#[derive(FromRow)]
pub struct CourseListRow {
	pub course_id: i64,
	pub title: Option<String>,
	pub category: Option<String>,
	pub creator_name: Option<String>,
	pub keys: Option<String>,
	pub play_count: Option<i64>,
	pub play_people: Option<i64>,
	pub clear_people: Option<i64>,
}

#[derive(FromRow)]
pub struct CourseDetailRow {
	pub title: Option<String>,
	pub category: Option<String>,
	pub creator_id: Option<i64>,
	pub creator_name: Option<String>,
	pub keys: Option<String>,
	pub play_count: Option<i64>,
	pub play_people: Option<i64>,
	pub clear_count: Option<i64>,
	pub clear_people: Option<i64>,
	pub fc_count: Option<i64>,
	pub hard_count: Option<i64>,
	pub normal_count: Option<i64>,
	pub easy_count: Option<i64>,
	pub failed_count: Option<i64>,
	pub hash: Option<String>,
}

#[derive(FromRow)]
pub struct CourseStageRow {
	pub stage: i64,
	pub label: Option<String>,
	pub md5: Option<String>,
}

#[derive(FromRow)]
pub struct CourseRankingRow {
	pub rank: i64,
	pub player_id: i64,
	pub player_name: Option<String>,
	pub dan: Option<String>,
	pub clear_type: Option<String>,
	pub letter_rank: Option<String>,
	pub score: Option<i64>,
	pub score_max: Option<i64>,
	pub combo: Option<i64>,
	pub combo_max: Option<i64>,
	pub bad_poor: Option<i64>,
	pub pgreat: Option<i64>,
	pub great: Option<i64>,
	pub good: Option<i64>,
	pub bad: Option<i64>,
	pub poor: Option<i64>,
	pub option_1: Option<String>,
	pub option_2: Option<String>,
	pub option_3: Option<String>,
	pub option_4: Option<String>,
	pub input: Option<String>,
	pub client: Option<String>,
	pub note: Option<String>,
	pub is_cheated: Option<i64>,
}

#[derive(FromRow)]
pub struct ChartListRow {
	pub md5: String,
	pub title: String,
	pub artist: String,
	pub genre: String,
	pub keys: String,
	pub level: String,
	pub play_count: i64,
	pub play_people: i64,
	pub clear_people: i64,
}

#[derive(FromRow, serde::Serialize)]
pub struct ChartDetailRow {
	pub md5: String,
	pub bmsid: Option<i64>,
	pub title: String,
	pub genre: String,
	pub artist: String,
	pub bpm_min: String,
	pub bpm_max: String,
	pub level: String,
	pub keys: String,
	pub judge_rank: String,
	pub play_count: i64,
	pub play_people: i64,
	pub clear_count: i64,
	pub clear_people: i64,
	pub fc_count: i64,
	pub hard_count: i64,
	pub normal_count: i64,
	pub easy_count: i64,
	pub failed_count: i64,
	pub last_updated_by: Option<String>,
	pub last_updated_at: Option<String>,
	pub body_url: Option<String>,
	pub diff_url: Option<String>,
	pub comment: Option<String>,
	pub tag_1: Option<String>,
	pub tag_2: Option<String>,
	pub tag_3: Option<String>,
	pub tag_4: Option<String>,
	pub tag_5: Option<String>,
	pub tag_6: Option<String>,
	pub tag_7: Option<String>,
	pub tag_8: Option<String>,
	pub tag_9: Option<String>,
	pub tag_10: Option<String>,
	pub suspended: i64,
}

#[derive(FromRow, serde::Serialize)]
pub struct PbDbRow {
	pub rank: i64,
	pub player_id: i64,
	pub player_name: String,
	pub dan: String,
	pub clear_type: String,
	pub letter_rank: String,
	pub score: i64,
	pub score_max: i64,
	pub combo: i64,
	pub combo_max: i64,
	pub bad_poor: i64,
	pub pgreat: i64,
	pub great: i64,
	pub good: i64,
	pub bad: i64,
	pub poor: i64,
	pub option_1: String,
	pub option_2: String,
	pub option_3: String,
	pub option_4: String,
	pub input: String,
	pub client: String,
	pub note: String,
	pub is_cheated: i64,
	pub has_ghost: bool,
}

#[derive(FromRow)]
pub struct UserListRow {
	pub player_id: i64,
	pub name: Option<String>,
	pub dan: Option<String>,
	pub play_count: Option<i64>,
	pub fc_count: Option<i64>,
	pub privacy_level: Option<String>,
	pub is_cheater: Option<i64>,
}

#[derive(FromRow)]
pub struct UserDetailRow {
	pub player_id: i64,
	pub name: Option<String>,
	pub dan: Option<String>,
	pub bio: Option<String>,
	pub privacy_level: Option<String>,
	pub songs_played: Option<i64>,
	pub play_count: Option<i64>,
	pub fc_count: Option<i64>,
	pub perfect_fc_count: Option<i64>,
	pub hard_count: Option<i64>,
	pub normal_count: Option<i64>,
	pub easy_count: Option<i64>,
	pub failed_count: Option<i64>,
	pub is_cheater: Option<i64>,
}

#[derive(FromRow)]
pub struct UserPlayRow {
	pub title: Option<String>,
	pub clear_type: Option<String>,
	pub play_count: Option<i64>,
	pub rank_pos: Option<i64>,
	pub rank_total: Option<i64>,
	pub md5: Option<String>,
}

#[derive(FromRow)]
pub struct UserBbsRow {
	pub commenter_id: Option<i64>,
	pub commenter_name: Option<String>,
	pub message: Option<String>,
	pub posted_at: Option<String>,
}

#[derive(FromRow)]
pub struct BbsListRow {
	pub msgid: i64,
	pub playerid: i64,
	pub message: Option<String>,
	pub time: Option<String>,
	pub name: Option<String>,
}

#[derive(FromRow)]
pub struct UserRivalRow {
	pub rival_id: Option<i64>,
	pub rival_name: Option<String>,
}

pub struct ChartListItem {
	pub md5: String,
	pub title: String,
	pub artist: String,
	pub genre: String,
	pub keys: String,
	pub level: String,
	pub play_count: i64,
	pub play_people: i64,
	pub clear_pct: String,
}

pub struct ChartMeta {
	pub md5: String,
	pub bmsid: Option<i64>,
	pub title: String,
	pub genre: String,
	pub artist: String,
	pub bpm_min: String,
	pub bpm_max: String,
	pub level: String,
	pub is_dp: bool,
	pub keys: String,
	pub judge_rank: String,
	pub play_count: i64,
	pub play_people: i64,
	pub clear_count: i64,
	pub clear_people: i64,
	pub clear_pct: String,
	pub fc_count: i64,
	pub hard_count: i64,
	pub normal_count: i64,
	pub easy_count: i64,
	pub failed_count: i64,
	pub last_updated_by: String,
	pub last_updated_at: String,
	pub body_url: Option<String>,
	pub diff_url: Option<String>,
	pub comment: Option<String>,
	pub tags: Vec<String>,
	pub suspended: bool,
}

pub struct PbEntry {
	pub rank: i64,
	pub player_id: i64,
	pub player_name: String,
	pub dan: String,
	pub clear_type: String,
	pub clear_class: String,
	pub letter_rank: String,
	pub score_display: String,
	pub grade_delta: String,
	pub combo_display: String,
	pub bad_poor: i64,
	pub pgreat: i64,
	pub great: i64,
	pub good: i64,
	pub bad: i64,
	pub poor: i64,
	pub option_1: String,
	pub option_2: String,
	pub option_3: String,
	pub option_4: String,
	pub input: String,
	pub client: String,
	pub note: String,
	pub has_ghost: bool,
	pub is_cheated: bool,
}

pub struct PlayerListItem {
	pub player_id: i64,
	pub name: String,
	pub dan: String,
	pub play_count: i64,
	pub fc_count: i64,
	/// Empty string = public; `"playcount"` or `"full"` otherwise.
	pub privacy_level: String,
	pub is_cheater: bool,
}

pub struct PlayerDetail {
	pub player_id: i64,
	pub name: String,
	pub dan: String,
	pub bio: String,
	/// Empty string = public; `"playcount"` or `"full"` otherwise.
	pub privacy_level: String,
	pub songs_played: i64,
	pub play_count: i64,
	pub fc_count: i64,
	pub perfect_fc_count: i64,
	pub hard_count: i64,
	pub normal_count: i64,
	pub easy_count: i64,
	pub failed_count: i64,
	pub total_clears: i64,
	pub is_cheater: bool,
}

pub struct PlayItem {
	pub title: String,
	pub clear_type: String,
	pub clear_class: String,
	pub play_count: i64,
	pub rank_pos: i64,
	pub rank_total: i64,
	pub md5: Option<String>,
}

pub struct BbsItem {
	pub player_id: i64,
	pub commenter_name: String,
	pub message: String,
	pub posted_at: String,
}

pub struct RivalItem {
	pub rival_id: i64,
	pub rival_name: String,
}

pub struct CourseListItem {
	pub course_id: i64,
	pub title: String,
	pub category: String,
	pub creator_name: String,
	pub keys: String,
	pub play_count: i64,
	pub play_people: i64,
	pub clear_pct: String,
}

pub struct CourseMetaDisplay {
	pub course_id: i64,
	pub title: String,
	pub category: String,
	pub creator_id: Option<i64>,
	pub creator_name: String,
	pub keys: String,
	pub play_count: i64,
	pub play_people: i64,
	pub clear_count: i64,
	pub clear_people: i64,
	pub fc_count: i64,
	pub hard_count: i64,
	pub normal_count: i64,
	pub easy_count: i64,
	pub failed_count: i64,
	pub total_clears: i64,
	pub clear_pct: String,
	pub is_dp: bool,
	pub hash: String,
}

pub struct CourseStageItem {
	pub stage: i64,
	pub label: String,
	pub md5: Option<String>,
}

pub struct CourseRankingEntry {
	pub rank: i64,
	pub player_id: i64,
	pub player_name: String,
	pub dan: String,
	pub clear_type: String,
	pub clear_class: String,
	pub letter_rank: String,
	pub grade_delta: String,
	pub score_display: String,
	pub combo_display: String,
	pub bad_poor: i64,
	pub pgreat: i64,
	pub great: i64,
	pub good: i64,
	pub bad: i64,
	pub poor: i64,
	pub option_1: String,
	pub option_2: String,
	pub option_3: String,
	pub option_4: String,
	pub input: String,
	pub client: String,
	pub note: String,
	pub is_cheated: bool,
}

#[derive(FromRow)]
pub struct TableMainRow {
	pub table_id: i64,
	pub name: String,
	pub symbol: String,
	pub chart_count: i64,
}

#[derive(FromRow)]
pub struct TableLevelSummaryRow {
	pub level: String,
	pub count: i64,
}

#[derive(FromRow)]
pub struct TableLevelMd5Row {
	pub md5: String,
}

#[derive(FromRow)]
pub struct TableLevelEntryRow {
	pub level: String,
	pub md5: String,
}

#[derive(FromRow)]
pub struct TableLevelTableRow {
	pub table_id: i64,
	pub md5: String,
}

#[derive(FromRow)]
pub struct PlayerPbRow {
	pub md5: String,
	pub clear_type: Option<String>,
	pub score: Option<i64>,
	pub score_max: Option<i64>,
	pub rank: Option<i64>,
	pub letter_rank: Option<String>,
}

#[derive(FromRow)]
pub struct UserBriefRow {
	pub player_id: i64,
	pub name: Option<String>,
	pub dan: Option<String>,
}

#[derive(FromRow)]
pub struct ChartTableRow {
	pub table_id: i64,
	pub name: String,
	pub symbol: String,
	pub level: String,
}

pub struct TableItem {
	pub id: i64,
	pub name: String,
	pub symbol: String,
	pub chart_count: i64,
}

pub struct TableLevelItem {
	pub level: String,
	pub count: i64,
}

pub struct TableLevelChartItem {
	pub md5: String,
	pub title: String,
	pub artist: String,
	pub keys: String,
	pub play_count: i64,
	pub clear_pct: String,
	pub in_archive: bool,
}

pub struct ChartTableMembership {
	pub table_id: i64, // rowid of table_main row, used for links
	pub name: String,
	pub symbol: String,
	pub level: String,
}

pub struct PlayerBrief {
	pub player_id: i64,
	pub name: String,
	pub dan: String,
}

#[derive(Copy, Clone)]
pub struct ClearBreakdown {
	pub fc_count: i64,
	pub hard_count: i64,
	pub normal_count: i64,
	pub easy_count: i64,
	pub failed_count: i64,
	pub total: i64,
}

impl ClearBreakdown {
	pub fn scored(&self) -> i64 {
		self.fc_count + self.hard_count + self.normal_count + self.easy_count + self.failed_count
	}
}

pub struct PlayerTableListItem {
	pub table_id: i64,
	pub name: String,
	pub symbol: String,
	pub clears: ClearBreakdown,
}

pub struct PlayerTableLevelSummary {
	pub level: String,
	pub clears: ClearBreakdown,
}

pub struct PlayerTableChartItem {
	pub md5: String,
	pub title: String,
	pub artist: String,
	pub keys: String,
	pub play_count: i64,
	pub clear_pct: String,
	pub in_archive: bool,
	pub has_score: bool,
	pub clear_type: String,
	pub clear_class: String,
	pub score_display: String,
	pub grade_delta: String,
	pub letter_rank: String,
	pub rank: i64,
}
