use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};

use crate::models::{
	ChartListRow, TableItem, TableLevelChartItem, TableLevelItem, TableLevelMd5Row,
	TableLevelSummaryRow, TableMainRow,
};
use crate::response::{not_found, render};
use crate::state::AppState;
use crate::templates::{TableLevelTemplate, TableTemplate, TablesTemplate};
use crate::util::clear_pct;

pub async fn handle_not_found() -> Response {
	not_found("The page you requested doesn't exist.")
}

fn no_tableinfo() -> Response {
	not_found(
		"Table data is not available — this instance was started without a tableinfo database.",
	)
}

pub async fn tables_list(State(state): State<AppState>) -> impl IntoResponse {
	let Some(ti) = &state.tableinfo else {
		return no_tableinfo();
	};

	let db_rows: Vec<TableMainRow> = sqlx::query_as(
		"SELECT table_id, name, symbol,
				(SELECT COUNT(*) FROM table_level WHERE table_id = table_main.table_id) AS chart_count
		 FROM table_main
		 ORDER BY table_id",
	)
	.fetch_all(ti.as_ref())
	.await
	.unwrap_or_default();

	let tables = db_rows
		.into_iter()
		.map(|r| TableItem {
			id: r.table_id,
			name: r.name,
			symbol: r.symbol,
			chart_count: r.chart_count,
		})
		.collect();

	let rendered_at: Option<String> = sqlx::query_scalar("SELECT rendered_at FROM meta LIMIT 1")
		.fetch_optional(ti.as_ref())
		.await
		.unwrap_or(None);

	render(TablesTemplate {
		tables,
		rendered_at,
	})
}

pub async fn table_detail(State(state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
	let Some(ti) = &state.tableinfo else {
		return no_tableinfo();
	};

	let maybe_table: Option<TableMainRow> = sqlx::query_as(
		"SELECT table_id, name, symbol,
				(SELECT COUNT(*) FROM table_level WHERE table_id = table_main.table_id) AS chart_count
		 FROM table_main WHERE table_id = ?",
	)
	.bind(id)
	.fetch_optional(ti.as_ref())
	.await
	.unwrap_or(None);

	let Some(table_row) = maybe_table else {
		return not_found("No difficulty table found with that ID.");
	};

	let level_rows: Vec<TableLevelSummaryRow> = sqlx::query_as(
		"SELECT level, COUNT(*) AS count
		 FROM table_level WHERE table_id = ?
		 GROUP BY level
		 ORDER BY CAST(level AS INTEGER), level",
	)
	.bind(table_row.table_id)
	.fetch_all(ti.as_ref())
	.await
	.unwrap_or_default();

	render(TableTemplate {
		table: TableItem {
			id: table_row.table_id,
			name: table_row.name,
			symbol: table_row.symbol,
			chart_count: table_row.chart_count,
		},
		levels: level_rows
			.into_iter()
			.map(|r| TableLevelItem {
				level: r.level,
				count: r.count,
			})
			.collect(),
	})
}

pub async fn table_level(
	State(state): State<AppState>,
	Path((id, level)): Path<(i64, String)>,
) -> impl IntoResponse {
	let Some(ti) = &state.tableinfo else {
		return no_tableinfo();
	};

	let maybe_table: Option<TableMainRow> = sqlx::query_as(
		"SELECT table_id, name, symbol,
				(SELECT COUNT(*) FROM table_level WHERE table_id = table_main.table_id) AS chart_count
		 FROM table_main WHERE table_id = ?",
	)
	.bind(id)
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

	let md5s: Vec<String> = md5_rows.into_iter().map(|r| r.md5).collect();

	// Look up chart info from the main db for those md5s.
	let chart_map: std::collections::HashMap<String, ChartListRow> = if md5s.is_empty() {
		Default::default()
	} else {
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

	let mut charts: Vec<TableLevelChartItem> = md5s
		.iter()
		.map(|m| match chart_map.get(m) {
			Some(r) => TableLevelChartItem {
				md5: r.md5.clone(),
				title: r.title.clone(),
				artist: r.artist.clone(),
				keys: r.keys.clone(),
				play_count: r.play_count,
				clear_pct: clear_pct(r.clear_people, r.play_people),
				in_archive: true,
			},
			None => TableLevelChartItem {
				md5: m.clone(),
				title: String::new(),
				artist: String::new(),
				keys: String::new(),
				play_count: 0,
				clear_pct: String::new(),
				in_archive: false,
			},
		})
		.collect();

	charts.sort_by(|a, b| {
		a.in_archive
			.cmp(&b.in_archive)
			.reverse()
			.then_with(|| a.title.to_lowercase().cmp(&b.title.to_lowercase()))
			.then_with(|| a.md5.cmp(&b.md5))
	});

	render(TableLevelTemplate {
		table: TableItem {
			id: table_row.table_id,
			name: table_row.name,
			symbol: table_row.symbol,
			chart_count: table_row.chart_count,
		},
		level,
		charts,
	})
}
