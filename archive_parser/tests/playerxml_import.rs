//! Integration test: bulk-import getplayerxml scores into pb.
//!
//!   PLAYERXML_DB=~/Documents/lr2ir-backup/raw-data/getplayerxml/getplayerxml.db \
//!   ARCHIVE_DB=sqlite:///tmp/playerxml-test.sqlite \
//!       cargo test -p lr2ir_archive_parser --test playerxml_import -- --ignored --nocapture
//!
//! Or run the self-contained fixture test (no env vars required):
//!       cargo test -p lr2ir_archive_parser --test playerxml_import -- --nocapture

use std::path::Path;

use sqlx::{sqlite::SqliteConnectOptions, Row, SqlitePool};

use lr2ir_archive_parser::db;

const MD5: &str = "98f2a9a05b16db52f1260e3b6812023f";
const EXISTING_PLAYER: i64 = 189_461;
const NEW_PLAYER: i64 = 120_831;

async fn open_archive_db(db_url: &str) -> SqlitePool {
	let opts = db_url
		.parse::<SqliteConnectOptions>()
		.expect("parse SQLite URL")
		.create_if_missing(true);
	SqlitePool::connect_with(opts)
		.await
		.expect("open archive db")
}

async fn seed_existing_pb_row(pool: &SqlitePool) {
	sqlx::query(
		r#"INSERT INTO pb
		   (md5, rank, player_id, player_name, dan, clear_type, letter_rank,
		    score, score_max, combo, combo_max,
		    bad_poor, pgreat, great, good, bad, poor,
		    option_1, option_2, option_3, option_4,
		    input, client, note, is_cheated)
		   VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)"#,
	)
	.bind(MD5)
	.bind(1_i64)
	.bind(EXISTING_PLAYER)
	.bind("html-player")
	.bind("dan-from-html")
	.bind("FULLCOMBO")
	.bind("AAA")
	.bind(3215_i64)
	.bind(3220_i64)
	.bind(1610_i64)
	.bind(1610_i64)
	.bind(0_i64)
	.bind(1606_i64)
	.bind(3_i64)
	.bind(0_i64)
	.bind(0_i64)
	.bind(1_i64)
	.bind("op1")
	.bind("op2")
	.bind(None::<String>)
	.bind(None::<String>)
	.bind("KB")
	.bind("LR2")
	.bind("note from html")
	.bind(0_i64)
	.execute(pool)
	.await
	.expect("seed existing pb row");
}

async fn create_fixture_playerxml_db(path: &Path) {
	let opts = SqliteConnectOptions::new()
		.filename(path)
		.create_if_missing(true);
	let pool = SqlitePool::connect_with(opts)
		.await
		.expect("open fixture getplayerxml.db");

	sqlx::query(
		r#"CREATE TABLE scores (
		    playerid INTEGER,
		    hash TEXT,
		    clear INTEGER,
		    notes INTEGER,
		    combo INTEGER,
		    pg INTEGER,
		    gr INTEGER,
		    gd INTEGER,
		    bd INTEGER,
		    pr INTEGER,
		    minbp INTEGER,
		    exscore INTEGER,
		    lastupdate INTEGER,
		    UNIQUE(playerid, hash)
		)"#,
	)
	.execute(&pool)
	.await
	.expect("create scores table");

	sqlx::query(
		r#"CREATE TABLE players (
		    playerid INTEGER PRIMARY KEY,
		    name TEXT,
		    scores INTEGER
		)"#,
	)
	.execute(&pool)
	.await
	.expect("create players table");

	sqlx::query("INSERT INTO players (playerid, name, scores) VALUES (?, ?, ?)")
		.bind(EXISTING_PLAYER)
		.bind("qkqj")
		.bind(1_i64)
		.execute(&pool)
		.await
		.expect("insert existing player");

	sqlx::query("INSERT INTO players (playerid, name, scores) VALUES (?, ?, ?)")
		.bind(NEW_PLAYER)
		.bind("qwea")
		.bind(1_i64)
		.execute(&pool)
		.await
		.expect("insert new player");

	// Row that should conflict with the pre-seeded HTML pb entry.
	sqlx::query(
		r#"INSERT INTO scores
		   (playerid, hash, clear, notes, combo, pg, gr, gd, bd, pr, minbp, exscore, lastupdate)
		   VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
	)
	.bind(EXISTING_PLAYER)
	.bind(MD5)
	.bind(5_i64)
	.bind(1610_i64)
	.bind(1610_i64)
	.bind(1606_i64)
	.bind(3_i64)
	.bind(0_i64)
	.bind(0_i64)
	.bind(1_i64)
	.bind(0_i64)
	.bind(3215_i64)
	.bind(1_779_703_646_i64)
	.execute(&pool)
	.await
	.expect("insert conflicting score");

	// Row that should be imported.
	sqlx::query(
		r#"INSERT INTO scores
		   (playerid, hash, clear, notes, combo, pg, gr, gd, bd, pr, minbp, exscore, lastupdate)
		   VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
	)
	.bind(NEW_PLAYER)
	.bind(MD5)
	.bind(5_i64)
	.bind(1610_i64)
	.bind(1610_i64)
	.bind(1592_i64)
	.bind(18_i64)
	.bind(0_i64)
	.bind(0_i64)
	.bind(1_i64)
	.bind(1_i64)
	.bind(3202_i64)
	.bind(1_779_703_646_i64)
	.execute(&pool)
	.await
	.expect("insert new score");

	pool.close().await;
}

#[tokio::test]
async fn playerxml_import_fixture() {
	let temp = tempfile::tempdir().expect("tempdir");
	let playerxml_path = temp.path().join("getplayerxml.db");
	let archive_path = temp.path().join("archive.sqlite");

	create_fixture_playerxml_db(&playerxml_path).await;

	let archive_url = format!("sqlite://{}", archive_path.display());
	let pool = open_archive_db(&archive_url).await;
	db::run_migrations(&pool).await.expect("run migrations");
	seed_existing_pb_row(&pool).await;

	let (rows_read, rows_inserted, md5s_ranked) =
		db::import_playerxml_scores(&pool, &playerxml_path)
			.await
			.expect("import playerxml scores");
	assert_eq!(rows_read, 2);
	assert_eq!(rows_inserted, 1);
	assert_eq!(md5s_ranked, 1);

	let existing = sqlx::query(
		r#"SELECT rank, player_name, dan, clear_type, letter_rank, score, score_max,
		          combo, combo_max, bad_poor, pgreat, great, good, bad, poor,
		          option_1, option_2, option_3, option_4, input, client, note, is_cheated
		   FROM pb WHERE md5 = ? AND player_id = ?"#,
	)
	.bind(MD5)
	.bind(EXISTING_PLAYER)
	.fetch_one(&pool)
	.await
	.expect("fetch existing pb row");

	assert_eq!(existing.get::<i64, _>("rank"), 1);
	assert_eq!(existing.get::<String, _>("player_name"), "html-player");
	assert_eq!(existing.get::<String, _>("dan"), "dan-from-html");
	assert_eq!(existing.get::<String, _>("clear_type"), "FULLCOMBO");
	assert_eq!(existing.get::<String, _>("letter_rank"), "AAA");
	assert_eq!(existing.get::<i64, _>("score"), 3215);
	assert_eq!(existing.get::<i64, _>("score_max"), 3220);
	assert_eq!(existing.get::<String, _>("option_1"), "op1");
	assert_eq!(existing.get::<String, _>("note"), "note from html");

	let imported = sqlx::query(
		r#"SELECT rank, player_name, dan, clear_type, letter_rank, score, score_max,
		          combo, combo_max, bad_poor, pgreat, great, good, bad, poor,
		          option_1, option_2, option_3, option_4, input, client, note, is_cheated
		   FROM pb WHERE md5 = ? AND player_id = ?"#,
	)
	.bind(MD5)
	.bind(NEW_PLAYER)
	.fetch_one(&pool)
	.await
	.expect("fetch imported pb row");

	assert_eq!(imported.get::<i64, _>("rank"), 2);
	assert_eq!(imported.get::<String, _>("player_name"), "qwea");
	assert_eq!(imported.get::<String, _>("dan"), "");
	assert_eq!(imported.get::<String, _>("clear_type"), "FULLCOMBO");
	assert_eq!(imported.get::<String, _>("letter_rank"), "");
	assert_eq!(imported.get::<i64, _>("score"), 3202);
	assert_eq!(imported.get::<i64, _>("score_max"), 3220);
	assert_eq!(imported.get::<i64, _>("combo"), 1610);
	assert_eq!(imported.get::<i64, _>("combo_max"), 1610);
	assert_eq!(imported.get::<i64, _>("bad_poor"), 1);
	assert_eq!(imported.get::<i64, _>("pgreat"), 1592);
	assert_eq!(imported.get::<i64, _>("great"), 18);
	assert_eq!(imported.get::<String, _>("option_1"), "");
	assert_eq!(imported.get::<Option<String>, _>("option_3"), None);
	assert_eq!(imported.get::<String, _>("input"), "");
	assert_eq!(imported.get::<String, _>("client"), "");
	assert_eq!(imported.get::<String, _>("note"), "");
	assert_eq!(imported.get::<i64, _>("is_cheated"), 0);

	let zero_ranks: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM pb WHERE rank = 0")
		.fetch_one(&pool)
		.await
		.expect("count zero ranks");
	assert_eq!(zero_ranks, 0);
}

#[tokio::test]
async fn playerxml_rank_ties() {
	let temp = tempfile::tempdir().expect("tempdir");
	let playerxml_path = temp.path().join("getplayerxml.db");
	let archive_path = temp.path().join("archive.sqlite");
	let md5 = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

	let opts = SqliteConnectOptions::new()
		.filename(&playerxml_path)
		.create_if_missing(true);
	let px_pool = SqlitePool::connect_with(opts)
		.await
		.expect("open fixture getplayerxml.db");

	sqlx::query(
		r#"CREATE TABLE scores (
		    playerid INTEGER, hash TEXT, clear INTEGER, notes INTEGER, combo INTEGER,
		    pg INTEGER, gr INTEGER, gd INTEGER, bd INTEGER, pr INTEGER,
		    minbp INTEGER, exscore INTEGER, lastupdate INTEGER,
		    UNIQUE(playerid, hash)
		)"#,
	)
	.execute(&px_pool)
	.await
	.expect("create scores");
	sqlx::query("CREATE TABLE players (playerid INTEGER PRIMARY KEY, name TEXT, scores INTEGER)")
		.execute(&px_pool)
		.await
		.expect("create players");

	for (pid, name) in [(1_i64, "a"), (2, "b"), (3, "c")] {
		sqlx::query("INSERT INTO players (playerid, name, scores) VALUES (?, ?, 1)")
			.bind(pid)
			.bind(name)
			.execute(&px_pool)
			.await
			.expect("insert player");
	}

	// Player 1 leads; players 2 and 3 tie for second.
	for (pid, score) in [(1_i64, 110_i64), (2, 100), (3, 100)] {
		sqlx::query(
			r#"INSERT INTO scores
			   (playerid, hash, clear, notes, combo, pg, gr, gd, bd, pr, minbp, exscore, lastupdate)
			   VALUES (?, ?, 5, 50, 50, 50, 0, 0, 0, 0, 0, ?, 0)"#,
		)
		.bind(pid)
		.bind(md5)
		.bind(score)
		.execute(&px_pool)
		.await
		.expect("insert score");
	}
	px_pool.close().await;

	let archive_url = format!("sqlite://{}", archive_path.display());
	let pool = open_archive_db(&archive_url).await;
	db::run_migrations(&pool).await.expect("run migrations");

	let (_, rows_inserted, md5s_ranked) = db::import_playerxml_scores(&pool, &playerxml_path)
		.await
		.expect("import");
	assert_eq!(rows_inserted, 3);
	assert_eq!(md5s_ranked, 1);

	let ranks: Vec<i64> =
		sqlx::query_scalar("SELECT rank FROM pb WHERE md5 = ? ORDER BY player_id")
			.bind(md5)
			.fetch_all(&pool)
			.await
			.expect("fetch ranks");
	assert_eq!(ranks, vec![1, 2, 2]);
}

#[tokio::test]
#[ignore = "requires local getplayerxml.db — set PLAYERXML_DB and ARCHIVE_DB"]
async fn playerxml_import_real_db_smoke() {
	let playerxml_db = std::env::var("PLAYERXML_DB").expect("set PLAYERXML_DB");
	let archive_db = std::env::var("ARCHIVE_DB").expect("set ARCHIVE_DB");

	let pool = open_archive_db(&archive_db).await;
	db::run_migrations(&pool).await.expect("run migrations");

	let (rows_read, rows_inserted, md5s_ranked) =
		db::import_playerxml_scores(&pool, Path::new(&playerxml_db))
			.await
			.expect("import real getplayerxml.db");

	println!(
		"read {rows_read} scores, inserted {rows_inserted} new pb rows, re-ranked {md5s_ranked} charts"
	);
	assert!(rows_read > 0);
	assert!(rows_inserted <= rows_read);
}
