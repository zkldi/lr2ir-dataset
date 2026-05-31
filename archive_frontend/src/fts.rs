use anyhow::{Context, Result};
use sqlx::SqlitePool;

/// Create the FTS5 virtual table if it doesn't exist, and (re)populate it
/// whenever its row count diverges from the `chart` table.
/// Uses a standalone (non-content) table with `md5 UNINDEXED` so we can JOIN
/// back to `chart` on the primary key without relying on fragile rowid mapping.
pub async fn ensure_fts(pool: &SqlitePool) -> Result<()> {
	sqlx::query(
		"CREATE VIRTUAL TABLE IF NOT EXISTS chart_fts \
		 USING fts5(md5 UNINDEXED, title, artist, genre, tokenize='unicode61')",
	)
	.execute(pool)
	.await
	.context("create chart_fts virtual table")?;

	let chart_n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM chart")
		.fetch_one(pool)
		.await
		.unwrap_or(0);
	let fts_n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM chart_fts")
		.fetch_one(pool)
		.await
		.unwrap_or(-1);

	if chart_n != fts_n {
		tracing::info!(
			charts = chart_n,
			fts = fts_n,
			"rebuilding FTS5 (Search) index. This may take a while…"
		);
		sqlx::query("DELETE FROM chart_fts")
			.execute(pool)
			.await
			.ok();
		sqlx::query(
			"INSERT INTO chart_fts(md5, title, artist, genre) \
			 SELECT md5, title, artist, genre FROM chart",
		)
		.execute(pool)
		.await
		.context("populate chart_fts")?;
		tracing::info!(rows = chart_n, "FTS5 (Search) index ready");
	}

	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::models::ChartListRow;
	use crate::util::fts_query;

	async fn make_db() -> SqlitePool {
		let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
		sqlx::query(
			"CREATE TABLE chart (
				md5		  TEXT PRIMARY KEY,
				title		TEXT NOT NULL DEFAULT '',
				artist	   TEXT NOT NULL DEFAULT '',
				genre		TEXT NOT NULL DEFAULT '',
				keys		 TEXT NOT NULL DEFAULT '',
				level		TEXT NOT NULL DEFAULT '',
				play_count   INTEGER NOT NULL DEFAULT 0,
				play_people  INTEGER NOT NULL DEFAULT 0,
				clear_people INTEGER NOT NULL DEFAULT 0
			)",
		)
		.execute(&pool)
		.await
		.unwrap();
		pool
	}

	async fn insert_chart(pool: &SqlitePool, md5: &str, title: &str, artist: &str, genre: &str) {
		sqlx::query("INSERT INTO chart (md5, title, artist, genre) VALUES (?, ?, ?, ?)")
			.bind(md5)
			.bind(title)
			.bind(artist)
			.bind(genre)
			.execute(pool)
			.await
			.unwrap();
	}

	#[tokio::test]
	async fn test_fts_basic_match() {
		let pool = make_db().await;
		insert_chart(
			&pool,
			"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
			"Air -GOD-",
			"dj nagureo",
			"TECHNO",
		)
		.await;
		ensure_fts(&pool).await.unwrap();

		let fts_q = fts_query("air").unwrap();
		let count: i64 =
			sqlx::query_scalar("SELECT COUNT(*) FROM chart_fts WHERE chart_fts MATCH ?")
				.bind(&fts_q)
				.fetch_one(&pool)
				.await
				.unwrap();
		assert_eq!(
			count, 1,
			"single-word 'air' should match 'Air -GOD-' (got {count}, query={fts_q:?})"
		);
	}

	#[tokio::test]
	async fn test_fts_multi_word_across_separator() {
		let pool = make_db().await;
		insert_chart(
			&pool,
			"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
			"Air -GOD-",
			"dj nagureo",
			"TECHNO",
		)
		.await;
		ensure_fts(&pool).await.unwrap();

		let fts_q = fts_query("air god").unwrap();
		let count: i64 =
			sqlx::query_scalar("SELECT COUNT(*) FROM chart_fts WHERE chart_fts MATCH ?")
				.bind(&fts_q)
				.fetch_one(&pool)
				.await
				.unwrap();
		assert_eq!(
			count, 1,
			"multi-word 'air god' should match 'Air -GOD-' (got {count}, query={fts_q:?})"
		);
	}

	#[tokio::test]
	async fn test_fts_prefix_match() {
		let pool = make_db().await;
		insert_chart(
			&pool,
			"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
			"Air -GOD-",
			"dj nagureo",
			"TECHNO",
		)
		.await;
		ensure_fts(&pool).await.unwrap();

		let fts_q = fts_query("air g").unwrap();
		let count: i64 =
			sqlx::query_scalar("SELECT COUNT(*) FROM chart_fts WHERE chart_fts MATCH ?")
				.bind(&fts_q)
				.fetch_one(&pool)
				.await
				.unwrap();
		assert_eq!(
			count, 1,
			"prefix 'air g' should match 'Air -GOD-' (got {count}, query={fts_q:?})"
		);
	}

	#[tokio::test]
	async fn test_fts_no_match() {
		let pool = make_db().await;
		insert_chart(
			&pool,
			"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
			"Air -GOD-",
			"dj nagureo",
			"TECHNO",
		)
		.await;
		ensure_fts(&pool).await.unwrap();

		let fts_q = fts_query("xenon").unwrap();
		let count: i64 =
			sqlx::query_scalar("SELECT COUNT(*) FROM chart_fts WHERE chart_fts MATCH ?")
				.bind(&fts_q)
				.fetch_one(&pool)
				.await
				.unwrap();
		assert_eq!(
			count, 0,
			"query 'xenon' should not match 'Air -GOD-' (got {count})"
		);
	}

	#[tokio::test]
	async fn test_fts_join_returns_chart_row() {
		let pool = make_db().await;
		insert_chart(
			&pool,
			"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
			"Air -GOD-",
			"dj nagureo",
			"TECHNO",
		)
		.await;
		ensure_fts(&pool).await.unwrap();

		let fts_q = fts_query("air god").unwrap();
		let rows: Vec<ChartListRow> = sqlx::query_as(
			r#"SELECT c.md5, c.title, c.artist, c.genre, c.keys, c.level,
					  c.play_count, c.play_people, c.clear_people
			   FROM chart c
			   JOIN (SELECT md5 FROM chart_fts WHERE chart_fts MATCH ?) AS fts
				 USING (md5)
			   ORDER BY c.play_count DESC
			   LIMIT 100 OFFSET 0"#,
		)
		.bind(&fts_q)
		.fetch_all(&pool)
		.await
		.unwrap();

		assert_eq!(
			rows.len(),
			1,
			"JOIN query should return 1 row (got {}, query={fts_q:?})",
			rows.len()
		);
		assert_eq!(rows[0].title, "Air -GOD-");
	}
}
