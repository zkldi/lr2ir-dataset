use anyhow::{Context, Result};
use axum::{routing::get, Router};
use sqlx::SqlitePool;
use std::sync::Arc;

use crate::cli::ServeArgs;
use crate::fts::ensure_fts;
use crate::handlers::{
	bbs_list, chart_detail, chart_detail_json, charts_list, course_detail, courses_list,
	ghost_download, handle_not_found, index, player_detail, player_table_level,
	player_table_summary, players_list, table_detail, table_level, tables_list,
};
use crate::site_stats::ensure_site_stats;
use crate::state::AppState;

pub async fn serve(args: ServeArgs) -> Result<()> {
	let pool = SqlitePool::connect(&format!("sqlite:{}", args.database))
		.await
		.context("open sqlite database")?;

	ensure_fts(&pool).await?;

	// This condition always returns true because the logic is inverted.
	// The .eq(&0) returns true if the count is 0, so !.eq(&0) is true when count is not 0,
	// i.e., when at least one of the indexes *already exists*, not when missing.
	// If the intention is to log info when the indexes are *missing*, remove the '!':

	let existing_idx_count = sqlx::query_scalar::<_, i64>(
		"SELECT COUNT(1) FROM sqlite_master WHERE type = 'index' AND name IN ('idx_chart_bmsid', 'idx_ghost_lookup', 'idx_pb_player_md5')"
	)
	.fetch_one(&pool)
	.await
	.unwrap_or(0);

	if existing_idx_count == 0 {
		tracing::info!("Making indexes... This may take a while...");
	}

	sqlx::query("CREATE INDEX IF NOT EXISTS idx_chart_bmsid ON chart(bmsid)")
		.execute(&pool)
		.await
		.context("create bmsid index")?;
	sqlx::query("CREATE INDEX IF NOT EXISTS idx_ghost_lookup ON ghost(md5, player_id)")
		.execute(&pool)
		.await
		.context("create ghost index")?;
	sqlx::query("CREATE INDEX IF NOT EXISTS idx_pb_player_md5 ON pb(player_id, md5)")
		.execute(&pool)
		.await
		.context("create pb player index")?;

	let tableinfo = if let Some(path) = &args.tableinfo {
		tracing::info!(%path, "opening tableinfo database");
		let ti = SqlitePool::connect(&format!("sqlite:{}", path))
			.await
			.with_context(|| format!("open tableinfo database at {path}"))?;
		Some(Arc::new(ti))
	} else {
		tracing::info!("no --tableinfo provided; table routes disabled");
		None
	};

	let stats = Arc::new(ensure_site_stats(&pool).await.context("load site stats")?);

	let state = AppState {
		db: Arc::new(pool),
		tableinfo,
		stats,
	};

	let app = Router::new()
		.route("/", get(index))
		.route("/charts", get(charts_list))
		.route("/charts/{md5}", get(chart_detail))
		.route("/api/charts/{md5}", get(chart_detail_json))
		.route("/players", get(players_list))
		.route(
			"/players/{player_id}/tables/{table_id}",
			get(player_table_summary),
		)
		.route(
			"/players/{player_id}/tables/{table_id}/{level}",
			get(player_table_level),
		)
		.route("/players/{player_id}", get(player_detail))
		.route("/tables", get(tables_list))
		.route("/tables/{id}", get(table_detail))
		.route("/tables/{id}/{level}", get(table_level))
		.route("/courses", get(courses_list))
		.route("/courses/{id}", get(course_detail))
		.route("/bbs", get(bbs_list))
		.route("/ghosts/{md5}/{player_id}", get(ghost_download))
		.fallback(handle_not_found)
		.with_state(state);

	let listener = tokio::net::TcpListener::bind(&args.bind)
		.await
		.with_context(|| format!("bind to {}", args.bind))?;

	tracing::info!(bind = %args.bind, "listening");
	axum::serve(listener, app).await.context("axum serve")?;

	Ok(())
}
