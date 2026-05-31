use anyhow::{Context, Result};
use sqlx::{FromRow, SqlitePool};

use crate::state::SiteStats;

#[derive(FromRow)]
struct SiteStatsRow {
	charts: i64,
	scores: i64,
	players: i64,
	ghosts: i64,
	bbs: i64,
}

impl From<SiteStatsRow> for SiteStats {
	fn from(row: SiteStatsRow) -> Self {
		SiteStats {
			charts: row.charts,
			scores: row.scores,
			players: row.players,
			ghosts: row.ghosts,
			bbs: row.bbs,
		}
	}
}

async fn ensure_table(pool: &SqlitePool) -> Result<()> {
	sqlx::query(
		"CREATE TABLE IF NOT EXISTS _site_stats (
			charts INTEGER NOT NULL,
			scores INTEGER NOT NULL,
			players INTEGER NOT NULL,
			ghosts INTEGER NOT NULL,
			bbs INTEGER NOT NULL
		) STRICT",
	)
	.execute(pool)
	.await
	.context("create _site_stats table")?;
	Ok(())
}

async fn load_cached(pool: &SqlitePool) -> Result<Option<SiteStats>> {
	let row: Option<SiteStatsRow> =
		sqlx::query_as("SELECT charts, scores, players, ghosts, bbs FROM _site_stats LIMIT 1")
			.fetch_optional(pool)
			.await
			.context("load cached site stats")?;
	Ok(row.map(Into::into))
}

async fn compute(pool: &SqlitePool) -> Result<SiteStats> {
	let (charts, scores, players, ghosts, bbs) = tokio::join!(
		sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM chart").fetch_one(pool),
		sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM pb").fetch_one(pool),
		sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM user").fetch_one(pool),
		sqlx::query_scalar::<_, i64>("SELECT COUNT(DISTINCT md5 || '|' || player_id) FROM ghost")
			.fetch_one(pool),
		sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM bbs").fetch_one(pool),
	);
	Ok(SiteStats {
		charts: charts.context("count charts")?,
		scores: scores.context("count scores")?,
		players: players.context("count players")?,
		ghosts: ghosts.context("count ghosts")?,
		bbs: bbs.context("count bbs")?,
	})
}

async fn store(pool: &SqlitePool, stats: &SiteStats) -> Result<()> {
	sqlx::query("DELETE FROM _site_stats")
		.execute(pool)
		.await
		.context("clear _site_stats")?;
	sqlx::query(
		"INSERT INTO _site_stats (charts, scores, players, ghosts, bbs) VALUES (?, ?, ?, ?, ?)",
	)
	.bind(stats.charts)
	.bind(stats.scores)
	.bind(stats.players)
	.bind(stats.ghosts)
	.bind(stats.bbs)
	.execute(pool)
	.await
	.context("store site stats")?;
	Ok(())
}

/// Load site stats from `_site_stats`, computing and caching them on first use.
pub async fn ensure_site_stats(pool: &SqlitePool) -> Result<SiteStats> {
	ensure_table(pool).await?;
	if let Some(stats) = load_cached(pool).await? {
		tracing::info!(
			charts = stats.charts,
			scores = stats.scores,
			players = stats.players,
			ghosts = stats.ghosts,
			bbs = stats.bbs,
			"loaded site stats from cache"
		);
		return Ok(stats);
	}

	tracing::info!("computing site stats…");
	let stats = compute(pool).await?;
	store(pool, &stats).await?;
	tracing::info!(
		charts = stats.charts,
		scores = stats.scores,
		players = stats.players,
		ghosts = stats.ghosts,
		bbs = stats.bbs,
		"site stats cached"
	);
	Ok(stats)
}
