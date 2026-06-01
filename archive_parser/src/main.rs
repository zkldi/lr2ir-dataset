use std::path::{Component, Path, PathBuf};

use anyhow::{Context, Result};
use clap::Parser as ClapParser;
use rayon::prelude::*;
use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use walkdir::WalkDir;

use lr2ir_archive_parser::{
	cheaters, db, jst_naive_to_rfc3339,
	lr2crs::{self, CourseMatchInput},
	parsers::{self, course::CourseMeta, searchcgi::ChartMeta, user::UserPageData, RankingRow},
	read_html_gz,
};

// ── CLI ───────────────────────────────────────────────────────────────────────

#[derive(ClapParser, Debug)]
#[command(
	name = "archive_parser",
	about = "Parse scraped LR2IR HTML pages into a SQLite database"
)]
struct Args {
	/// Root directory containing scraped pages:
	///   searchcgi/{md5:32}/{n}.html[.gz]  →  chart ranking pages
	///   courses/course_{id}/{n}.html[.gz] →  course ranking pages
	///   users/{id}.html[.gz]              →  user profile pages
	#[arg(short, long)]
	pages_dir: PathBuf,

	/// SQLite database URL — MUST begin with `sqlite://`, e.g.:
	///   sqlite:///absolute/path/to/out.sqlite
	///   sqlite://./relative/out.sqlite
	#[arg(long, env = "DATABASE_URL")]
	db: String,
}

// ── Parsed result types (sent over the channel) ───────────────────────────────

#[expect(clippy::large_enum_variant)]
enum FileResult {
	SearchCgi {
		md5: String,
		meta: Option<ChartMeta>,
		rows: Vec<RankingRow>,
	},
	Course {
		course_id: i64,
		meta: Option<CourseMeta>,
		rows: Vec<RankingRow>,
	},
	User(UserPageData),
}

// ── Entry point ───────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<()> {
	let args = Args::parse();

	if !args.db.starts_with("sqlite://") {
		panic!(
			"--db / DATABASE_URL must begin with sqlite://, got: {:?}",
			args.db
		);
	}

	let opts = args
		.db
		.parse::<SqliteConnectOptions>()
		.with_context(|| format!("parse SQLite URL: {}", args.db))?
		.create_if_missing(true);

	let pool = SqlitePool::connect_with(opts)
		.await
		.with_context(|| format!("open SQLite: {}", args.db))?;

	db::run_migrations(&pool).await?;

	let cheaters = cheaters::cheater_set();
	eprintln!("cheater list: {} ids", cheaters.len());

	// ── Producer: rayon parallel file walk + parse ────────────────────────────
	// Channel buffer sized to let parsers run ahead of the writer.
	let (result_tx, mut result_rx) = tokio::sync::mpsc::channel::<FileResult>(8_192);

	let pages_dir = args.pages_dir.clone();
	let walk_handle = tokio::task::spawn_blocking(move || {
		WalkDir::new(&pages_dir)
			.into_iter()
			.filter_map(|e| e.ok())
			.filter(|e| e.file_type().is_file() && is_html_page_file(e.path()))
			// par_bridge hands each entry to rayon's thread pool.
			.par_bridge()
			.for_each_with(result_tx, |tx, entry| {
				let path = entry.path();
				let rel = match path.strip_prefix(&pages_dir) {
					Ok(r) => r.to_owned(),
					Err(_) => return,
				};
				let Some(page_type) = detect_page_type(&rel) else {
					return;
				};

				match parse_file(path, page_type) {
					Ok(Some(result)) => {
						// blocking_send applies back-pressure when the writer
						// falls behind — prevents unbounded memory growth.
						let _ = tx.blocking_send(result);
					}
					Ok(None) => {}
					Err(e) => eprintln!("WARN [{rel:?}]: {e:#}"),
				}
			});
	});

	// ── Consumer: single async writer with batched transactions ───────────────
	let mut tx = pool.begin().await?;
	let mut file_count: u64 = 0;
	let mut pb_count: u64 = 0;
	let mut error_count: u64 = 0;

	while let Some(result) = result_rx.recv().await {
		let write_result = async {
			match &result {
				FileResult::SearchCgi { md5, meta, rows } => {
					if let Some(m) = meta {
						db::upsert_chart(&mut *tx, m).await?;
					}
					for row in rows {
						db::upsert_pb(&mut *tx, md5, row, &cheaters).await?;
					}
					pb_count += rows.len() as u64;
				}
				FileResult::Course {
					course_id,
					meta,
					rows,
				} => {
					if let Some(m) = meta {
						db::upsert_course(&mut *tx, m).await?;
						for s in &m.stages {
							db::upsert_course_stage(&mut *tx, *course_id, s).await?;
						}
					}
					for row in rows {
						db::upsert_course_ranking(&mut *tx, *course_id, row, &cheaters).await?;
					}
					pb_count += rows.len() as u64;
				}
				FileResult::User(data) => {
					let pid = data.profile.player_id;
					db::upsert_user(&mut *tx, &data.profile, &cheaters).await?;
					for r in &data.rivals {
						db::upsert_user_rival(&mut *tx, pid, r).await?;
					}
					for e in &data.most_plays {
						db::upsert_user_most_play(&mut *tx, pid, e).await?;
					}
					for e in &data.recent_plays {
						db::upsert_user_recent_play(&mut *tx, pid, e).await?;
					}
					for e in &data.recent_courses {
						db::upsert_user_recent_course(&mut *tx, pid, e).await?;
					}
					for e in &data.bbs {
						db::upsert_user_bbs(&mut *tx, pid, e).await?;
					}
				}
			}
			anyhow::Ok(())
		}
		.await;

		if let Err(e) = write_result {
			eprintln!("WARN [write]: {e:#}");
			error_count += 1;
		}

		file_count += 1;
		if file_count.is_multiple_of(10_000) {
			tx.commit().await?;
			tx = pool.begin().await?;
			eprintln!(
				"progress: {file_count} files, {pb_count} ranking rows, {error_count} errors"
			);
		}
	}

	tx.commit().await?;
	walk_handle.await.context("walk task panicked")?;
	eprintln!("done: {file_count} files, {pb_count} ranking rows, {error_count} errors");

	// ── Ghosts: copy from ghosts/ghosts.db if present ────────────────────────
	import_ghosts(&pool, &args.pages_dir).await?;

	// ── Course hashes: from all-courses/course.lr2crs if present ─────────────
	import_course_hashes(&pool, &args.pages_dir).await?;

	// ── BBS: copy from bbs/bbs.db if present ─────────────────────────────────
	import_bbs(&pool, &args.pages_dir).await?;

	// ── Player scores: fill gaps from getplayerxml/getplayerxml.db ───────────
	import_playerxml(&pool, &args.pages_dir).await?;

	db::refresh_site_stats(&pool).await?;
	eprintln!("site stats cached");

	Ok(())
}

// ── Ghost importer ────────────────────────────────────────────────────────────

async fn import_ghosts(pool: &SqlitePool, pages_dir: &Path) -> Result<()> {
	let ghost_db_path = pages_dir.join("ghosts/ghosts.db");
	if !ghost_db_path.exists() {
		return Ok(());
	}

	eprintln!("importing ghosts from {ghost_db_path:?}");

	let ghost_opts = SqliteConnectOptions::new()
		.filename(&ghost_db_path)
		.read_only(true);
	let ghost_pool = SqlitePool::connect_with(ghost_opts)
		.await
		.with_context(|| format!("open ghosts.db: {ghost_db_path:?}"))?;

	let rows: Vec<(String, i64, String, Vec<u8>)> =
		sqlx::query_as("SELECT md5, player_id, player_name, ghost FROM ghost")
			.fetch_all(&ghost_pool)
			.await
			.context("read ghosts")?;

	let total = rows.len();
	let mut tx = pool.begin().await?;
	for (i, (md5, player_id, player_name, ghost)) in rows.iter().enumerate() {
		db::upsert_ghost(&mut *tx, md5, *player_id, player_name, ghost).await?;
		if (i + 1).is_multiple_of(10_000) {
			tx.commit().await?;
			tx = pool.begin().await?;
			eprintln!("ghosts progress: {}/{total}", i + 1);
		}
	}
	tx.commit().await?;
	eprintln!("done: imported {total} ghosts");

	Ok(())
}

// ── Course hash importer ──────────────────────────────────────────────────────

async fn import_course_hashes(pool: &SqlitePool, pages_dir: &Path) -> Result<()> {
	let lr2crs_path = pages_dir.join("all-courses/course.lr2crs");
	if !lr2crs_path.exists() {
		return Ok(());
	}

	eprintln!("importing course hashes from {lr2crs_path:?}");

	let entries = lr2crs::read_lr2crs_file(&lr2crs_path)?;
	eprintln!("parsed {} lr2crs entries", entries.len());
	let index = lr2crs::MatchIndex::new(&entries);

	let courses = db::list_courses_for_hash_match(pool).await?;
	let stage_rows = db::list_course_stage_md5s(pool).await?;
	let stage_md5s = lr2crs::group_stage_md5s(&stage_rows);

	let total = courses.len();
	let mut matched = 0u64;
	let mut skipped = 0u64;
	let mut tx = pool.begin().await?;
	for (i, (course_id, title, keys)) in courses.iter().enumerate() {
		let input = CourseMatchInput {
			course_id: *course_id,
			title: title.clone(),
			keys: keys.clone(),
			stage_md5s: stage_md5s.get(course_id).cloned().unwrap_or_default(),
		};
		let Some(entry) = lr2crs::match_entry_with_index(&input, &entries, &index) else {
			eprintln!("WARN [course course_id={course_id}]: no lr2crs match for {title:?}");
			skipped += 1;
			continue;
		};
		db::update_course_hash(&mut *tx, *course_id, &entry.hash, entry.course_type).await?;
		matched += 1;
		if (i + 1).is_multiple_of(10_000) {
			tx.commit().await?;
			tx = pool.begin().await?;
			eprintln!("course hash progress: {}/{total}", i + 1);
		}
	}
	tx.commit().await?;
	eprintln!("done: updated {matched} course hashes ({skipped} unmatched)");

	Ok(())
}

// ── BBS importer ──────────────────────────────────────────────────────────────

async fn import_bbs(pool: &SqlitePool, pages_dir: &Path) -> Result<()> {
	let bbs_db_path = pages_dir.join("bbs/bbs.db");
	if !bbs_db_path.exists() {
		return Ok(());
	}

	eprintln!("importing bbs from {bbs_db_path:?}");

	let bbs_opts = SqliteConnectOptions::new()
		.filename(&bbs_db_path)
		.read_only(true);
	let bbs_pool = SqlitePool::connect_with(bbs_opts)
		.await
		.with_context(|| format!("open bbs.db: {bbs_db_path:?}"))?;

	let rows: Vec<(i64, i64, Option<String>, String)> =
		sqlx::query_as("SELECT msgid, playerid, message, time FROM bbs")
			.fetch_all(&bbs_pool)
			.await
			.context("read bbs")?;

	let total = rows.len();
	let mut skipped = 0u64;
	let mut tx = pool.begin().await?;
	for (i, (msgid, playerid, message, time)) in rows.iter().enumerate() {
		let Some(time_rfc3339) = jst_naive_to_rfc3339(time) else {
			eprintln!("WARN [bbs msgid={msgid}]: unparseable time: {time:?}");
			skipped += 1;
			continue;
		};
		db::upsert_bbs(
			&mut *tx,
			*msgid,
			*playerid,
			message.as_deref().unwrap_or(""),
			&time_rfc3339,
		)
		.await?;
		if (i + 1).is_multiple_of(10_000) {
			tx.commit().await?;
			tx = pool.begin().await?;
			eprintln!("bbs progress: {}/{total}", i + 1);
		}
	}
	tx.commit().await?;
	eprintln!(
		"done: imported {} bbs messages ({} skipped)",
		total - skipped as usize,
		skipped
	);

	Ok(())
}

// ── getplayerxml score importer ───────────────────────────────────────────────

async fn import_playerxml(pool: &SqlitePool, pages_dir: &Path) -> Result<()> {
	let playerxml_db_path = pages_dir.join("getplayerxml/getplayerxml.db");
	if !playerxml_db_path.exists() {
		return Ok(());
	}

	eprintln!("importing player scores from {playerxml_db_path:?}");

	let (rows_read, rows_inserted, md5s_ranked) =
		db::import_playerxml_scores(pool, &playerxml_db_path).await?;
	eprintln!(
		"done: read {rows_read} getplayerxml scores, inserted {rows_inserted} new pb rows, re-ranked {md5s_ranked} charts"
	);

	Ok(())
}

// ── Page-type detection ───────────────────────────────────────────────────────

enum PageType {
	SearchCgi {
		md5: String,
		page: u32,
	},
	Course {
		course_id: i64,
		#[allow(dead_code)]
		page: u32,
	},
	User(i64),
}

fn is_html_page_file(path: &Path) -> bool {
	path.file_name()
		.and_then(|name| name.to_str())
		.is_some_and(|name| name.ends_with(".html.gz") || name.ends_with(".html"))
}

fn strip_html_page_suffix(name: &str) -> Option<&str> {
	name.strip_suffix(".html.gz")
		.or_else(|| name.strip_suffix(".html"))
}

fn detect_page_type(rel: &Path) -> Option<PageType> {
	let parts: Vec<_> = rel.components().collect();

	match parts.as_slice() {
		// users/{id}.html[.gz]
		[Component::Normal(a), Component::Normal(b)] => {
			let a = a.to_str()?;
			let b = b.to_str()?;
			if a == "users" {
				let id: i64 = strip_html_page_suffix(b)?.parse().ok()?;
				Some(PageType::User(id))
			} else {
				None
			}
		}
		// searchcgi/{md5}/{n}.html[.gz] or courses/{id}/{n}.html[.gz]
		[Component::Normal(a), Component::Normal(b), Component::Normal(c)] => {
			let a = a.to_str()?;
			let b = b.to_str()?;
			let c = c.to_str()?;
			let page: u32 = strip_html_page_suffix(c)?.parse().ok()?;
			if a == "searchcgi" && b.len() == 32 && b.bytes().all(|c| c.is_ascii_hexdigit()) {
				Some(PageType::SearchCgi {
					md5: b.to_string(),
					page,
				})
			} else if a == "courses" && b.bytes().all(|c| c.is_ascii_digit()) {
				let course_id: i64 = b.parse().ok()?;
				Some(PageType::Course { course_id, page })
			} else {
				None
			}
		}
		_ => None,
	}
}

// ── Per-file parser (runs inside rayon) ──────────────────────────────────────

fn parse_file(path: &Path, page_type: PageType) -> Result<Option<FileResult>> {
	let html = read_html_gz(path)?;
	Ok(Some(match page_type {
		PageType::SearchCgi { md5, page } => {
			let (meta, rows) = parsers::searchcgi::parse_page(&md5, page, &html);
			FileResult::SearchCgi { md5, meta, rows }
		}
		PageType::Course { course_id, .. } => {
			let (meta, rows) = parsers::course::parse_page(course_id, &html);
			FileResult::Course {
				course_id,
				meta,
				rows,
			}
		}
		PageType::User(player_id) => match parsers::user::parse_user(player_id, &html) {
			Some(data) => FileResult::User(data),
			None => return Ok(None),
		},
	}))
}
