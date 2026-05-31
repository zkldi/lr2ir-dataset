use std::collections::HashMap;

use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};

use crate::models::{
	ChartListRow, ClearBreakdown, PlayerBrief, PlayerPbRow, PlayerTableChartItem,
	PlayerTableLevelSummary, PlayerTableListItem, TableItem, TableLevelEntryRow, TableLevelMd5Row,
	TableLevelTableRow, TableMainRow, UserBriefRow,
};
use crate::response::{not_found, render};
use crate::state::AppState;
use crate::templates::{PlayerTableLevelTemplate, PlayerTableTemplate};
use crate::util::{
	add_clear_to_breakdown, bms_grade_delta, clear_css_class, clear_pct, display_player_name_opt,
	score_pct,
};

fn no_tableinfo() -> Response {
	not_found(
		"Table data is not available — this instance was started without a tableinfo database.",
	)
}

async fn lookup_player(state: &AppState, player_id: i64) -> Option<PlayerBrief> {
	let row: Option<UserBriefRow> =
		sqlx::query_as("SELECT player_id, name, dan FROM user WHERE player_id = ?")
			.bind(player_id)
			.fetch_optional(state.db.as_ref())
			.await
			.unwrap_or(None);

	row.map(|u| PlayerBrief {
		player_id: u.player_id,
		name: display_player_name_opt(u.name.as_deref(), u.player_id),
		dan: u.dan.unwrap_or_default(),
	})
}

async fn fetch_player_pb_map(
	state: &AppState,
	player_id: i64,
	md5s: &[String],
) -> HashMap<String, PlayerPbRow> {
	if md5s.is_empty() {
		return HashMap::new();
	}

	let placeholders = md5s.iter().map(|_| "?").collect::<Vec<_>>().join(",");
	let sql = format!(
		"SELECT md5, clear_type, score, score_max, rank, letter_rank \
		 FROM pb WHERE player_id = ? AND md5 IN ({placeholders})"
	);
	let mut q = sqlx::query_as::<_, PlayerPbRow>(&sql).bind(player_id);
	for m in md5s {
		q = q.bind(m);
	}

	q.fetch_all(state.db.as_ref())
		.await
		.unwrap_or_default()
		.into_iter()
		.map(|r| (r.md5.clone(), r))
		.collect()
}

fn pb_to_chart_fields(pb: &PlayerPbRow) -> (String, String, String, String, String, i64) {
	let clear_type = pb.clear_type.clone().unwrap_or_default();
	let score = pb.score.unwrap_or(0);
	let score_max = pb.score_max.unwrap_or(0);
	(
		clear_type.clone(),
		clear_css_class(&clear_type),
		format!("{}/{}\n{}", score, score_max, score_pct(score, score_max)),
		bms_grade_delta(score, score_max),
		pb.letter_rank.clone().unwrap_or_default(),
		pb.rank.unwrap_or(0),
	)
}

fn make_breakdown(
	fc: i64,
	hard: i64,
	normal: i64,
	easy: i64,
	failed: i64,
	total: i64,
) -> ClearBreakdown {
	ClearBreakdown {
		fc_count: fc,
		hard_count: hard,
		normal_count: normal,
		easy_count: easy,
		failed_count: failed,
		total,
	}
}

fn increment_breakdown(breakdown: &mut ClearBreakdown, clear_type: &str) {
	add_clear_to_breakdown(
		&mut breakdown.fc_count,
		&mut breakdown.hard_count,
		&mut breakdown.normal_count,
		&mut breakdown.easy_count,
		&mut breakdown.failed_count,
		clear_type,
	);
}

pub async fn fetch_player_table_list(state: &AppState, player_id: i64) -> Vec<PlayerTableListItem> {
	let Some(ti) = &state.tableinfo else {
		return vec![];
	};

	let player_md5s: Vec<String> = sqlx::query_scalar("SELECT md5 FROM pb WHERE player_id = ?")
		.bind(player_id)
		.fetch_all(state.db.as_ref())
		.await
		.unwrap_or_default();

	if player_md5s.is_empty() {
		return vec![];
	}

	let pb_map = fetch_player_pb_map(state, player_id, &player_md5s).await;

	let level_rows: Vec<TableLevelTableRow> =
		sqlx::query_as("SELECT table_id, md5 FROM table_level")
			.fetch_all(ti.as_ref())
			.await
			.unwrap_or_default();

	let mut breakdowns: HashMap<i64, ClearBreakdown> = HashMap::new();
	for row in level_rows {
		let breakdown = breakdowns
			.entry(row.table_id)
			.or_insert_with(|| make_breakdown(0, 0, 0, 0, 0, 0));
		breakdown.total += 1;
		if let Some(pb) = pb_map.get(&row.md5) {
			increment_breakdown(breakdown, pb.clear_type.as_deref().unwrap_or_default());
		}
	}

	let table_ids: Vec<i64> = breakdowns
		.iter()
		.filter(|(_, b)| b.scored() > 0)
		.map(|(id, _)| *id)
		.collect();

	if table_ids.is_empty() {
		return vec![];
	}

	let placeholders = table_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
	let sql = format!(
		"SELECT table_id, name, symbol,
				(SELECT COUNT(*) FROM table_level WHERE table_id = table_main.table_id) AS chart_count
		 FROM table_main WHERE table_id IN ({placeholders}) ORDER BY table_id"
	);
	let mut q = sqlx::query_as::<_, TableMainRow>(&sql);
	for id in &table_ids {
		q = q.bind(id);
	}
	let table_rows = q.fetch_all(ti.as_ref()).await.unwrap_or_default();

	table_rows
		.into_iter()
		.filter_map(|r| {
			let clears = breakdowns.remove(&r.table_id)?;
			Some(PlayerTableListItem {
				table_id: r.table_id,
				name: r.name,
				symbol: r.symbol,
				clears,
			})
		})
		.collect()
}

pub async fn player_table_summary(
	State(state): State<AppState>,
	Path((player_id, table_id)): Path<(i64, i64)>,
) -> impl IntoResponse {
	let Some(ti) = &state.tableinfo else {
		return no_tableinfo();
	};

	let Some(player) = lookup_player(&state, player_id).await else {
		return not_found("No player found with that ID.");
	};

	let maybe_table: Option<TableMainRow> = sqlx::query_as(
		"SELECT table_id, name, symbol,
				(SELECT COUNT(*) FROM table_level WHERE table_id = table_main.table_id) AS chart_count
		 FROM table_main WHERE table_id = ?",
	)
	.bind(table_id)
	.fetch_optional(ti.as_ref())
	.await
	.unwrap_or(None);

	let Some(table_row) = maybe_table else {
		return not_found("No difficulty table found with that ID.");
	};

	let entry_rows: Vec<TableLevelEntryRow> = sqlx::query_as(
		"SELECT level, md5 FROM table_level WHERE table_id = ? ORDER BY CAST(level AS INTEGER), level, rowid",
	)
	.bind(table_id)
	.fetch_all(ti.as_ref())
	.await
	.unwrap_or_default();

	let md5s: Vec<String> = entry_rows.iter().map(|r| r.md5.clone()).collect();
	let pb_map = fetch_player_pb_map(&state, player.player_id, &md5s).await;

	let mut level_order: Vec<String> = Vec::new();
	let mut level_breakdowns: HashMap<String, ClearBreakdown> = HashMap::new();
	for row in &entry_rows {
		if !level_order.contains(&row.level) {
			level_order.push(row.level.clone());
		}
		let breakdown = level_breakdowns
			.entry(row.level.clone())
			.or_insert_with(|| make_breakdown(0, 0, 0, 0, 0, 0));
		breakdown.total += 1;
		if let Some(pb) = pb_map.get(&row.md5) {
			increment_breakdown(breakdown, pb.clear_type.as_deref().unwrap_or_default());
		}
	}

	let levels = level_order
		.into_iter()
		.map(|level| {
			let clears = level_breakdowns
				.remove(&level)
				.unwrap_or_else(|| make_breakdown(0, 0, 0, 0, 0, 0));
			PlayerTableLevelSummary { level, clears }
		})
		.collect();

	let table = TableItem {
		id: table_row.table_id,
		name: table_row.name,
		symbol: table_row.symbol,
		chart_count: table_row.chart_count,
	};

	render(PlayerTableTemplate {
		player,
		table,
		levels,
	})
}

pub async fn player_table_level(
	State(state): State<AppState>,
	Path((player_id, table_id, level)): Path<(i64, i64, String)>,
) -> impl IntoResponse {
	let Some(ti) = &state.tableinfo else {
		return no_tableinfo();
	};

	let Some(player) = lookup_player(&state, player_id).await else {
		return not_found("No player found with that ID.");
	};

	let maybe_table: Option<TableMainRow> = sqlx::query_as(
		"SELECT table_id, name, symbol,
				(SELECT COUNT(*) FROM table_level WHERE table_id = table_main.table_id) AS chart_count
		 FROM table_main WHERE table_id = ?",
	)
	.bind(table_id)
	.fetch_optional(ti.as_ref())
	.await
	.unwrap_or(None);

	let Some(table_row) = maybe_table else {
		return not_found("No difficulty table found with that ID.");
	};

	let md5_rows: Vec<TableLevelMd5Row> = sqlx::query_as(
		"SELECT md5 FROM table_level WHERE table_id = ? AND level = ? ORDER BY rowid",
	)
	.bind(table_row.table_id)
	.bind(&level)
	.fetch_all(ti.as_ref())
	.await
	.unwrap_or_default();

	if md5_rows.is_empty() {
		return not_found("No level found with that name on this table.");
	}

	let md5s: Vec<String> = md5_rows.into_iter().map(|r| r.md5).collect();
	let pb_map = fetch_player_pb_map(&state, player.player_id, &md5s).await;

	let chart_map: HashMap<String, ChartListRow> = {
		let placeholders = md5s.iter().map(|_| "?").collect::<Vec<_>>().join(",");
		let sql = format!(
			"SELECT md5, title, artist, genre, keys, level, play_count, play_people, clear_people \
			 FROM chart WHERE md5 IN ({placeholders})"
		);
		let mut q = sqlx::query_as::<_, ChartListRow>(&sql);
		for m in &md5s {
			q = q.bind(m);
		}
		q.fetch_all(state.db.as_ref())
			.await
			.unwrap_or_default()
			.into_iter()
			.map(|r| (r.md5.clone(), r))
			.collect()
	};

	let charts = md5s
		.iter()
		.map(|m| {
			if let Some(r) = chart_map.get(m) {
				let (clear_type, clear_class, score_display, grade_delta, letter_rank, rank) =
					if let Some(pb) = pb_map.get(m) {
						let fields = pb_to_chart_fields(pb);
						(fields.0, fields.1, fields.2, fields.3, fields.4, fields.5)
					} else {
						(
							String::new(),
							String::new(),
							String::new(),
							String::new(),
							String::new(),
							0,
						)
					};
				PlayerTableChartItem {
					md5: r.md5.clone(),
					title: r.title.clone(),
					artist: r.artist.clone(),
					keys: r.keys.clone(),
					play_count: r.play_count,
					clear_pct: clear_pct(r.clear_people, r.play_people),
					in_archive: true,
					has_score: pb_map.contains_key(m),
					clear_type,
					clear_class,
					score_display,
					grade_delta,
					letter_rank,
					rank,
				}
			} else {
				PlayerTableChartItem {
					md5: m.clone(),
					title: String::new(),
					artist: String::new(),
					keys: String::new(),
					play_count: 0,
					clear_pct: String::new(),
					in_archive: false,
					has_score: pb_map.contains_key(m),
					clear_type: String::new(),
					clear_class: String::new(),
					score_display: String::new(),
					grade_delta: String::new(),
					letter_rank: String::new(),
					rank: 0,
				}
			}
		})
		.collect();

	let table = TableItem {
		id: table_row.table_id,
		name: table_row.name,
		symbol: table_row.symbol,
		chart_count: table_row.chart_count,
	};

	render(PlayerTableLevelTemplate {
		player,
		table,
		level,
		charts,
	})
}
