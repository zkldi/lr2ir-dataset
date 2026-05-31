//! Integration test: match lr2crs entries against an existing archive DB.
//!
//!   ARCHIVE_DB=./data/lr2ir-archive.db \
//!   LR2CRS=path/to/course.lr2crs \
//!       cargo test -p lr2ir_archive_parser --test course_hash_import -- --ignored --nocapture

use std::path::Path;

use sqlx::sqlite::SqliteConnectOptions;
use sqlx::SqlitePool;

use lr2ir_archive_parser::{
	db,
	lr2crs::{self, CourseMatchInput, MatchIndex},
	read_lr2crs_file,
};

#[tokio::test]
#[ignore = "requires local archive DB with course + course_stage — set ARCHIVE_DB and LR2CRS"]
async fn import_course_hashes_smoke() {
	let db_path = std::env::var("ARCHIVE_DB").expect("set ARCHIVE_DB");
	let lr2crs_path = std::env::var("LR2CRS").expect("set LR2CRS");

	let opts = SqliteConnectOptions::new()
		.filename(&db_path)
		.read_only(true);
	let pool = SqlitePool::connect_with(opts)
		.await
		.expect("open archive db");

	let entries = read_lr2crs_file(Path::new(&lr2crs_path)).expect("parse lr2crs");
	let index = MatchIndex::new(&entries);
	let courses = db::list_courses_for_hash_match(&pool)
		.await
		.expect("list courses");
	let stage_rows = db::list_course_stage_md5s(&pool)
		.await
		.expect("list stage md5s");
	let stage_md5s = lr2crs::group_stage_md5s(&stage_rows);

	let mut matched = 0usize;
	for (course_id, title, keys) in &courses {
		let input = CourseMatchInput {
			course_id: *course_id,
			title: title.clone(),
			keys: keys.clone(),
			stage_md5s: stage_md5s.get(course_id).cloned().unwrap_or_default(),
		};
		if lr2crs::match_entry_with_index(&input, &entries, &index).is_some() {
			matched += 1;
		}
	}

	eprintln!(
		"matched {matched}/{} courses ({:.1}%)",
		courses.len(),
		100.0 * matched as f64 / courses.len() as f64
	);
	assert!(
		matched as f64 / courses.len() as f64 > 0.95,
		"expected >95% match rate, got {matched}/{}",
		courses.len()
	);

	// OverJoy sanity check
	let overjoy = entries
		.iter()
		.find(|e| e.title == "OverJoy")
		.expect("OverJoy");
	assert!(overjoy.hash.contains("fb7a747a5115a4a4739397d17ccb26bb"));
}
