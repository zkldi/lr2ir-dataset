use anyhow::{Context, Result};
use sqlx::{Executor, Sqlite, SqlitePool};
use std::collections::HashSet;
use std::path::Path;

use crate::cheaters;
use crate::parsers::{
	course::{CourseMeta, CourseStage},
	searchcgi::ChartMeta,
	user::{UserBbsEntry, UserCourseEntry, UserPlayEntry, UserProfile, UserRival},
	RankingRow,
};

/// Apply all pending migrations from `parser/migrations/`.
pub async fn run_migrations(pool: &SqlitePool) -> Result<()> {
	sqlx::migrate!("./migrations")
		.run(pool)
		.await
		.context("run migrations")
}

// ── Upserts ───────────────────────────────────────────────────────────────────
// Each function accepts any sqlx executor (pool, &mut transaction, …).

pub async fn upsert_chart<'e, E>(exec: E, m: &ChartMeta) -> Result<()>
where
	E: Executor<'e, Database = Sqlite>,
{
	sqlx::query(
		r#"INSERT OR REPLACE INTO chart
		   (md5, bmsid, suspended, title, genre, artist,
			bpm_min, bpm_max, level, keys, judge_rank,
			play_count, play_people, clear_count, clear_people,
			fc_count, hard_count, normal_count, easy_count, failed_count,
			last_updated_by, last_updated_at,
			body_url, diff_url, comment,
			tag_1, tag_2, tag_3, tag_4, tag_5,
			tag_6, tag_7, tag_8, tag_9, tag_10)
		   VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)"#,
	)
	.bind(&m.md5)
	.bind(m.bmsid)
	.bind(m.suspended)
	.bind(&m.title)
	.bind(&m.genre)
	.bind(&m.artist)
	.bind(&m.bpm_min)
	.bind(&m.bpm_max)
	.bind(&m.level)
	.bind(&m.keys)
	.bind(&m.judge_rank)
	.bind(m.play_count)
	.bind(m.play_people)
	.bind(m.clear_count)
	.bind(m.clear_people)
	.bind(m.fc_count)
	.bind(m.hard_count)
	.bind(m.normal_count)
	.bind(m.easy_count)
	.bind(m.failed_count)
	.bind(&m.last_updated_by)
	.bind(&m.last_updated_at)
	.bind(&m.body_url)
	.bind(&m.diff_url)
	.bind(&m.comment)
	.bind(m.tags.first().map(String::as_str))
	.bind(m.tags.get(1).map(String::as_str))
	.bind(m.tags.get(2).map(String::as_str))
	.bind(m.tags.get(3).map(String::as_str))
	.bind(m.tags.get(4).map(String::as_str))
	.bind(m.tags.get(5).map(String::as_str))
	.bind(m.tags.get(6).map(String::as_str))
	.bind(m.tags.get(7).map(String::as_str))
	.bind(m.tags.get(8).map(String::as_str))
	.bind(m.tags.get(9).map(String::as_str))
	.execute(exec)
	.await
	.context("upsert chart")?;
	Ok(())
}

pub async fn upsert_pb<'e, E>(
	exec: E,
	md5: &str,
	row: &RankingRow,
	cheaters: &HashSet<i64>,
) -> Result<()>
where
	E: Executor<'e, Database = Sqlite>,
{
	sqlx::query(
		r#"INSERT OR REPLACE INTO pb
		   (md5, rank, player_id, player_name, dan, clear_type, letter_rank,
			score, score_max, combo, combo_max,
			bad_poor, pgreat, great, good, bad, poor,
			option_1, option_2, option_3, option_4,
			input, client, note, is_cheated)
		   VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)"#,
	)
	.bind(md5)
	.bind(row.rank)
	.bind(row.player_id)
	.bind(&row.player_name)
	.bind(&row.dan)
	.bind(&row.clear_type)
	.bind(&row.letter_rank)
	.bind(row.score)
	.bind(row.score_max)
	.bind(row.combo)
	.bind(row.combo_max)
	.bind(row.bad_poor)
	.bind(row.pgreat)
	.bind(row.great)
	.bind(row.good)
	.bind(row.bad)
	.bind(row.poor)
	.bind(&row.option_1)
	.bind(&row.option_2)
	.bind(&row.option_3)
	.bind(&row.option_4)
	.bind(&row.input)
	.bind(&row.client)
	.bind(&row.note)
	.bind(cheaters::is_cheater_id(row.player_id, cheaters))
	.execute(exec)
	.await
	.context("upsert pb")?;
	Ok(())
}

pub async fn upsert_user<'e, E>(exec: E, p: &UserProfile, cheaters: &HashSet<i64>) -> Result<()>
where
	E: Executor<'e, Database = Sqlite>,
{
	sqlx::query(
		r#"INSERT OR REPLACE INTO user
		   (player_id, name, dan, bio, privacy_level, songs_played, play_count,
			fc_count, perfect_fc_count, hard_count, normal_count, easy_count, failed_count,
			is_cheater)
		   VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?)"#,
	)
	.bind(p.player_id)
	.bind(&p.name)
	.bind(&p.dan)
	.bind(&p.bio)
	.bind(&p.privacy_level)
	.bind(p.songs_played)
	.bind(p.play_count)
	.bind(p.fc_count)
	.bind(p.perfect_fc_count)
	.bind(p.hard_count)
	.bind(p.normal_count)
	.bind(p.easy_count)
	.bind(p.failed_count)
	.bind(cheaters::is_cheater_id(p.player_id, cheaters))
	.execute(exec)
	.await
	.context("upsert user")?;
	Ok(())
}

pub async fn upsert_course<'e, E>(exec: E, m: &CourseMeta) -> Result<()>
where
	E: Executor<'e, Database = Sqlite>,
{
	sqlx::query(
		r#"INSERT OR REPLACE INTO course
		   (course_id, title, category, creator_id, creator_name, keys,
			play_count, play_people, clear_count, clear_people,
			fc_count, hard_count, normal_count, easy_count, failed_count)
		   VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)"#,
	)
	.bind(m.course_id)
	.bind(&m.title)
	.bind(&m.category)
	.bind(m.creator_id)
	.bind(&m.creator_name)
	.bind(&m.keys)
	.bind(m.play_count)
	.bind(m.play_people)
	.bind(m.clear_count)
	.bind(m.clear_people)
	.bind(m.fc_count)
	.bind(m.hard_count)
	.bind(m.normal_count)
	.bind(m.easy_count)
	.bind(m.failed_count)
	.execute(exec)
	.await
	.context("upsert course")?;
	Ok(())
}

pub async fn update_course_hash<'e, E>(
	exec: E,
	course_id: i64,
	hash: &str,
	course_type: i64,
) -> Result<()>
where
	E: Executor<'e, Database = Sqlite>,
{
	sqlx::query("UPDATE course SET hash = ?, course_type = ? WHERE course_id = ?")
		.bind(hash)
		.bind(course_type)
		.bind(course_id)
		.execute(exec)
		.await
		.context("update course hash")?;
	Ok(())
}

pub async fn list_courses_for_hash_match(pool: &SqlitePool) -> Result<Vec<(i64, String, String)>> {
	let rows: Vec<(i64, String, String)> =
		sqlx::query_as("SELECT course_id, title, keys FROM course ORDER BY course_id")
			.fetch_all(pool)
			.await
			.context("list courses for hash match")?;
	Ok(rows)
}

pub async fn list_course_stage_md5s(pool: &SqlitePool) -> Result<Vec<(i64, Option<String>)>> {
	let rows: Vec<(i64, Option<String>)> = sqlx::query_as(
		r#"SELECT cs.course_id, c.md5
		   FROM course_stage cs
		   LEFT JOIN chart c ON c.bmsid = cs.bmsid
		   ORDER BY cs.course_id, cs.stage"#,
	)
	.fetch_all(pool)
	.await
	.context("list course stage md5s")?;
	Ok(rows)
}

pub async fn upsert_course_stage<'e, E>(exec: E, course_id: i64, s: &CourseStage) -> Result<()>
where
	E: Executor<'e, Database = Sqlite>,
{
	sqlx::query(
		"INSERT OR REPLACE INTO course_stage (course_id, stage, bmsid, label) VALUES (?,?,?,?)",
	)
	.bind(course_id)
	.bind(s.stage)
	.bind(s.bmsid)
	.bind(&s.label)
	.execute(exec)
	.await
	.context("upsert course_stage")?;
	Ok(())
}

pub async fn upsert_course_ranking<'e, E>(
	exec: E,
	course_id: i64,
	row: &RankingRow,
	cheaters: &HashSet<i64>,
) -> Result<()>
where
	E: Executor<'e, Database = Sqlite>,
{
	sqlx::query(
		r#"INSERT OR REPLACE INTO course_ranking
		   (course_id, rank, player_id, player_name, dan, clear_type, letter_rank,
			score, score_max, combo, combo_max,
			bad_poor, pgreat, great, good, bad, poor,
			option_1, option_2, option_3, option_4,
			input, client, note, is_cheated)
		   VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)"#,
	)
	.bind(course_id)
	.bind(row.rank)
	.bind(row.player_id)
	.bind(&row.player_name)
	.bind(&row.dan)
	.bind(&row.clear_type)
	.bind(&row.letter_rank)
	.bind(row.score)
	.bind(row.score_max)
	.bind(row.combo)
	.bind(row.combo_max)
	.bind(row.bad_poor)
	.bind(row.pgreat)
	.bind(row.great)
	.bind(row.good)
	.bind(row.bad)
	.bind(row.poor)
	.bind(&row.option_1)
	.bind(&row.option_2)
	.bind(&row.option_3)
	.bind(&row.option_4)
	.bind(&row.input)
	.bind(&row.client)
	.bind(&row.note)
	.bind(cheaters::is_cheater_id(row.player_id, cheaters))
	.execute(exec)
	.await
	.context("upsert course_ranking")?;
	Ok(())
}

pub async fn upsert_user_rival<'e, E>(exec: E, player_id: i64, r: &UserRival) -> Result<()>
where
	E: Executor<'e, Database = Sqlite>,
{
	sqlx::query(
		"INSERT OR REPLACE INTO user_rival (player_id, rival_id, rival_name) VALUES (?,?,?)",
	)
	.bind(player_id)
	.bind(r.rival_id)
	.bind(&r.rival_name)
	.execute(exec)
	.await
	.context("upsert user_rival")?;
	Ok(())
}

pub async fn upsert_user_most_play<'e, E>(exec: E, player_id: i64, e: &UserPlayEntry) -> Result<()>
where
	E: Executor<'e, Database = Sqlite>,
{
	sqlx::query(
		r#"INSERT OR REPLACE INTO user_most_plays
		   (player_id, pos, bmsid, title, clear_type, play_count, rank_pos, rank_total)
		   VALUES (?,?,?,?,?,?,?,?)"#,
	)
	.bind(player_id)
	.bind(e.pos)
	.bind(e.bmsid)
	.bind(&e.title)
	.bind(&e.clear_type)
	.bind(e.play_count)
	.bind(e.rank_pos)
	.bind(e.rank_total)
	.execute(exec)
	.await
	.context("upsert user_most_plays")?;
	Ok(())
}

pub async fn upsert_user_recent_play<'e, E>(
	exec: E,
	player_id: i64,
	e: &UserPlayEntry,
) -> Result<()>
where
	E: Executor<'e, Database = Sqlite>,
{
	sqlx::query(
		r#"INSERT OR REPLACE INTO user_recent_plays
		   (player_id, pos, bmsid, title, clear_type, play_count, rank_pos, rank_total)
		   VALUES (?,?,?,?,?,?,?,?)"#,
	)
	.bind(player_id)
	.bind(e.pos)
	.bind(e.bmsid)
	.bind(&e.title)
	.bind(&e.clear_type)
	.bind(e.play_count)
	.bind(e.rank_pos)
	.bind(e.rank_total)
	.execute(exec)
	.await
	.context("upsert user_recent_plays")?;
	Ok(())
}

pub async fn upsert_user_recent_course<'e, E>(
	exec: E,
	player_id: i64,
	e: &UserCourseEntry,
) -> Result<()>
where
	E: Executor<'e, Database = Sqlite>,
{
	sqlx::query(
		r#"INSERT OR REPLACE INTO user_recent_courses
		   (player_id, pos, course_id, title, clear_type, play_count, rank_pos, rank_total)
		   VALUES (?,?,?,?,?,?,?,?)"#,
	)
	.bind(player_id)
	.bind(e.pos)
	.bind(e.course_id)
	.bind(&e.title)
	.bind(&e.clear_type)
	.bind(e.play_count)
	.bind(e.rank_pos)
	.bind(e.rank_total)
	.execute(exec)
	.await
	.context("upsert user_recent_courses")?;
	Ok(())
}

pub async fn upsert_user_bbs<'e, E>(exec: E, player_id: i64, e: &UserBbsEntry) -> Result<()>
where
	E: Executor<'e, Database = Sqlite>,
{
	sqlx::query(
		r#"INSERT OR REPLACE INTO user_bbs
		   (player_id, pos, commenter_id, commenter_name, message, posted_at)
		   VALUES (?,?,?,?,?,?)"#,
	)
	.bind(player_id)
	.bind(e.pos)
	.bind(e.commenter_id)
	.bind(&e.commenter_name)
	.bind(&e.message)
	.bind(&e.posted_at)
	.execute(exec)
	.await
	.context("upsert user_bbs")?;
	Ok(())
}

pub async fn upsert_bbs<'e, E>(
	exec: E,
	msgid: i64,
	playerid: i64,
	message: &str,
	time: &str,
) -> Result<()>
where
	E: Executor<'e, Database = Sqlite>,
{
	sqlx::query("INSERT OR REPLACE INTO bbs (msgid, playerid, message, time) VALUES (?,?,?,?)")
		.bind(msgid)
		.bind(playerid)
		.bind(message)
		.bind(time)
		.execute(exec)
		.await
		.context("upsert bbs")?;
	Ok(())
}

pub async fn upsert_ghost<'e, E>(
	exec: E,
	md5: &str,
	player_id: i64,
	player_name: &str,
	ghost: &[u8],
) -> Result<()>
where
	E: Executor<'e, Database = Sqlite>,
{
	sqlx::query(
		"INSERT OR IGNORE INTO ghost (md5, player_id, player_name, ghost) VALUES (?,?,?,?)",
	)
	.bind(md5)
	.bind(player_id)
	.bind(player_name)
	.bind(ghost)
	.execute(exec)
	.await
	.context("upsert ghost")?;
	Ok(())
}

/// SQL `CASE` mapping getplayerxml `clear` integers to LR2IR `clear_type` strings.
pub fn playerxml_clear_type_case_sql() -> &'static str {
	r#"CASE s.clear
    WHEN 0 THEN 'NO PLAY'
    WHEN 1 THEN 'FAILED'
    WHEN 2 THEN 'EASY CLEAR'
    WHEN 3 THEN 'NORMAL CLEAR'
    WHEN 4 THEN 'HARD CLEAR'
    WHEN 5 THEN 'FULLCOMBO'
    ELSE 'FAILED'
  END"#
}

/// Comma-separated cheater IDs for embedding in SQL `IN (...)` clauses.
pub fn cheater_in_list_sql() -> String {
	cheaters::CHEATER_IDS
		.iter()
		.map(|id| id.to_string())
		.collect::<Vec<_>>()
		.join(", ")
}

/// Calculate ranking data. We can't just use the score data for this as we're merging
/// datasets.
pub async fn refresh_pb_ranks_for_md5s(
	conn: &mut sqlx::SqliteConnection,
	md5s: &[String],
) -> Result<u64> {
	const BATCH_LOG: usize = 1_000;

	for (i, md5) in md5s.iter().enumerate() {
		sqlx::query(
			r#"UPDATE pb
			   SET rank = (
			     SELECT rnk FROM (
			       SELECT player_id,
			              RANK() OVER (PARTITION BY md5 ORDER BY score DESC) AS rnk
			       FROM pb
			       WHERE md5 = ?
			     ) ranked
			     WHERE ranked.player_id = pb.player_id
			   )
			   WHERE md5 = ?"#,
		)
		.bind(md5)
		.bind(md5)
		.execute(&mut *conn)
		.await
		.with_context(|| format!("refresh pb ranks for md5={md5}"))?;

		if (i + 1).is_multiple_of(BATCH_LOG) {
			eprintln!("pb rank progress: {}/{}", i + 1, md5s.len());
		}
	}

	Ok(md5s.len() as u64)
}

/// Bulk-import scores from `getplayerxml.db` into `pb`, skipping rows that already exist.
///
/// Returns `(rows_read, rows_inserted, md5s_ranked)`.
pub async fn import_playerxml_scores(
	pool: &SqlitePool,
	playerxml_db_path: &Path,
) -> Result<(u64, u64, u64)> {
	let abs_path = std::fs::canonicalize(playerxml_db_path)
		.with_context(|| format!("resolve path: {playerxml_db_path:?}"))?;
	let abs_path = abs_path.to_string_lossy();

	let insert_sql = format!(
		r#"INSERT OR IGNORE INTO pb (
  md5, rank, player_id, player_name, dan, clear_type, letter_rank,
  score, score_max, combo, combo_max,
  bad_poor, pgreat, great, good, bad, poor,
  option_1, option_2, option_3, option_4,
  input, client, note, is_cheated
)
SELECT
  s.hash,
  -1,
  s.playerid,
  COALESCE(p.name, ''),
  '',
  {clear_case},
  '',
  s.exscore,
  s.notes * 2,
  s.combo,
  s.notes,
  s.minbp,
  s.pg, s.gr, s.gd, s.bd, s.pr,
  '', '', NULL, NULL,
  '', '', '',
  CASE WHEN s.playerid IN ({cheater_list}) THEN 1 ELSE 0 END
FROM px.scores s
JOIN px.players p ON p.playerid = s.playerid
WHERE length(s.hash) = 32
  AND s.hash GLOB '[0-9a-fA-F][0-9a-fA-F]*'"#,
		clear_case = playerxml_clear_type_case_sql(),
		cheater_list = cheater_in_list_sql(),
	);

	// ATTACH is per-connection; hold one connection for the whole import.
	let mut conn = pool.acquire().await.context("acquire sqlite connection")?;

	sqlx::query("ATTACH DATABASE ? AS px")
		.bind(abs_path.as_ref())
		.execute(&mut *conn)
		.await
		.context("attach getplayerxml.db")?;

	let rows_read: i64 = sqlx::query_scalar(
		r#"SELECT COUNT(*) FROM px.scores s
		   JOIN px.players p ON p.playerid = s.playerid
		   WHERE length(s.hash) = 32
		     AND s.hash GLOB '[0-9a-fA-F][0-9a-fA-F]*'"#,
	)
	.fetch_one(&mut *conn)
	.await
	.context("count getplayerxml scores")?;

	let all_md5s: Vec<String> = sqlx::query_scalar(
		r#"SELECT DISTINCT s.hash
		   FROM px.scores s
		   JOIN px.players p ON p.playerid = s.playerid
		   WHERE length(s.hash) = 32
		     AND s.hash GLOB '[0-9a-fA-F][0-9a-fA-F]*'"#,
	)
	.fetch_all(&mut *conn)
	.await
	.context("list getplayerxml md5s")?;

	let rows_inserted = sqlx::query(&insert_sql)
		.execute(&mut *conn)
		.await
		.context("import getplayerxml scores")?
		.rows_affected();

	let md5s_ranked = if all_md5s.is_empty() {
		0
	} else {
		refresh_pb_ranks_for_md5s(&mut conn, &all_md5s).await?
	};

	sqlx::query("DETACH DATABASE px")
		.execute(&mut *conn)
		.await
		.context("detach getplayerxml.db")?;

	Ok((rows_read as u64, rows_inserted, md5s_ranked))
}

/// Recompute homepage counters and store them in `_site_stats`.
pub async fn refresh_site_stats(pool: &SqlitePool) -> Result<()> {
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

	let (charts, scores, players, ghosts, bbs) = tokio::join!(
		sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM chart").fetch_one(pool),
		sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM pb").fetch_one(pool),
		sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM user").fetch_one(pool),
		sqlx::query_scalar::<_, i64>("SELECT COUNT(DISTINCT md5 || '|' || player_id) FROM ghost")
			.fetch_one(pool),
		sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM bbs").fetch_one(pool),
	);

	sqlx::query("DELETE FROM _site_stats")
		.execute(pool)
		.await
		.context("clear _site_stats")?;
	sqlx::query(
		"INSERT INTO _site_stats (charts, scores, players, ghosts, bbs) VALUES (?, ?, ?, ?, ?)",
	)
	.bind(charts.context("count charts")?)
	.bind(scores.context("count scores")?)
	.bind(players.context("count players")?)
	.bind(ghosts.context("count ghosts")?)
	.bind(bbs.context("count bbs")?)
	.execute(pool)
	.await
	.context("store site stats")?;
	Ok(())
}
