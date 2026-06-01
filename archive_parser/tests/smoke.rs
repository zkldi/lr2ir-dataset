//! Smoke test: parse a single MD5 directory and verify the DB output.
//!
//! Required environment variables:
//!   SMOKE_PAGES_DIR  — path to the md5 directory, e.g. `./pages/751738dea1169c5c39db935adfc9e85f`
//!   SMOKE_DB — SQLite URL that must begin with `sqlite://`, e.g. `sqlite://./smoke.sqlite`
//!
//! Run with:
//!   SMOKE_PAGES_DIR=./pages/751738dea1169c5c39db935adfc9e85f \
//!   SMOKE_DB=sqlite://./smoke.sqlite \
//!       cargo test -p parser -- --ignored --nocapture

use std::path::Path;

use sqlx::{sqlite::SqliteConnectOptions, Row, SqlitePool};

use lr2ir_archive_parser::{cheaters, db, parsers, read_html_gz};

#[tokio::test]
#[ignore = "requires local pages directory — set SMOKE_PAGES_DIR and SMOKE_DB"]
async fn smoke_chart_parse() {
	let pages_dir = std::env::var("SMOKE_PAGES_DIR")
		.expect("set SMOKE_PAGES_DIR to a single md5 directory, e.g. ./pages/751738de…");
	let db_url = std::env::var("SMOKE_DB")
		.expect("set SMOKE_DB to a sqlite:// path, e.g. sqlite://./smoke.sqlite");

	assert!(
		db_url.starts_with("sqlite://"),
		"SMOKE_DB must begin with sqlite://, got: {db_url:?}"
	);

	let pages_path = Path::new(&pages_dir);
	let md5 = pages_path
		.file_name()
		.and_then(|s| s.to_str())
		.expect("SMOKE_PAGES_DIR must end with the md5 hash as the last path component");

	let db_file = db_url.trim_start_matches("sqlite://");
	let _ = std::fs::remove_file(db_file);

	let opts = db_url
		.parse::<SqliteConnectOptions>()
		.expect("parse SQLite URL")
		.create_if_missing(true);
	let pool = SqlitePool::connect_with(opts)
		.await
		.expect("open smoke database");

	db::run_migrations(&pool).await.expect("run migrations");
	let cheaters = cheaters::cheater_set();

	// ── Parse all pages in the directory ─────────────────────────────────────

	let mut total_pb = 0usize;
	for page in 1u32.. {
		let file = pages_path.join(format!("{page}.html.gz"));
		if !file.exists() {
			break;
		}
		let html = read_html_gz(&file).expect("decompress + decode page");
		let (meta, rows) = parsers::searchcgi::parse_page(md5, page, &html);
		if let Some(m) = meta {
			db::upsert_chart(&pool, &m).await.expect("upsert chart");
		}
		for row in &rows {
			db::upsert_pb(&pool, md5, row, &cheaters)
				.await
				.expect("upsert pb row");
		}
		total_pb += rows.len();
		println!("  page {page}: {} ranking rows", rows.len());
	}
	println!("total pb rows inserted: {total_pb}");

	// ── Assertions ────────────────────────────────────────────────────────────

	// 1. Chart row exists with non-empty title, artist, genre.
	let chart = sqlx::query("SELECT title, artist, genre, bmsid FROM chart WHERE md5 = ?")
		.bind(md5)
		.fetch_one(&pool)
		.await
		.expect("chart row must exist after parsing page 1");

	let title: Option<String> = chart.get("title");
	let artist: Option<String> = chart.get("artist");
	let genre: Option<String> = chart.get("genre");

	assert!(
		title.as_deref().is_some_and(|s| !s.is_empty()),
		"title should be non-empty,  got {title:?}"
	);
	assert!(
		artist.as_deref().is_some_and(|s| !s.is_empty()),
		"artist should be non-empty, got {artist:?}"
	);
	assert!(
		genre.as_deref().is_some_and(|s| !s.is_empty()),
		"genre should be non-empty,  got {genre:?}"
	);
	println!("chart: {title:?} / {artist:?} / {genre:?}");

	// 2. At least one pb row.
	let pb_count: i64 = sqlx::query("SELECT COUNT(*) FROM pb WHERE md5 = ?")
		.bind(md5)
		.fetch_one(&pool)
		.await
		.expect("count pb rows")
		.get(0);
	assert!(
		pb_count >= 1,
		"must have at least one pb row, got {pb_count}"
	);
	println!("pb rows: {pb_count}");

	// 3. bmsid non-NULL for charts with multiple pages (≥100 entries).
	if pb_count >= 100 {
		let bmsid: Option<i64> = chart.get("bmsid");
		assert!(
			bmsid.is_some(),
			"bmsid should be non-NULL for chart with ≥100 entries (got {pb_count} rows)"
		);
		println!("bmsid: {}", bmsid.unwrap());
	}

	// 4. Every pb row has a non-NULL player_name and non-zero score_max.
	let bad_rows: i64 = sqlx::query(
		"SELECT COUNT(*) FROM pb WHERE md5 = ? AND (player_name IS NULL OR score_max IS NULL OR score_max = 0)",
	)
	.bind(md5)
	.fetch_one(&pool)
	.await
	.expect("count bad pb rows")
	.get(0);
	assert_eq!(
		bad_rows, 0,
		"{bad_rows} pb rows have NULL player_name or zero/NULL score_max"
	);

	// 5. No duplicate (md5, rank) pairs.
	let dupes: i64 = sqlx::query(
		"SELECT COUNT(*) FROM (SELECT rank FROM pb WHERE md5 = ? GROUP BY rank HAVING COUNT(*) > 1)",
	)
	.bind(md5)
	.fetch_one(&pool)
	.await
	.expect("count duplicate ranks")
	.get(0);
	assert_eq!(dupes, 0, "found {dupes} duplicate (md5, rank) pairs");

	println!("smoke test passed ✓  md5={md5}");
	println!("db written to: {db_file}");
}
