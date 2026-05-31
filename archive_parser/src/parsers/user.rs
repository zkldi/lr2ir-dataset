use std::collections::HashMap;

use scraper::{Element, Html, Selector};

use super::{collect_text, collect_text_with_br};

// ── Public types ──────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct UserProfile {
	pub player_id: i64,
	pub name: String,
	pub dan: String,
	pub bio: String,
	/// `None` = fully public.
	/// `Some("playcount")` = play counts hidden (no histogram, 4-col play tables).
	/// `Some("full")` = entire stats section hidden.
	pub privacy_level: Option<String>,
	pub songs_played: i64,
	pub play_count: i64,
	pub fc_count: i64,
	/// Perfect full combos, shown as `★N` inside the FC cell e.g. `499(★33)`.
	pub perfect_fc_count: Option<i64>,
	pub hard_count: i64,
	pub normal_count: i64,
	pub easy_count: i64,
	pub failed_count: i64,
}

#[derive(Debug)]
pub struct UserRival {
	pub rival_id: i64,
	pub rival_name: String,
}

#[derive(Debug)]
pub struct UserPlayEntry {
	pub pos: i64,
	pub bmsid: Option<i64>,
	pub title: Option<String>,
	pub clear_type: Option<String>,
	pub play_count: Option<i64>,
	pub rank_pos: Option<i64>,
	pub rank_total: Option<i64>,
}

#[derive(Debug)]
pub struct UserCourseEntry {
	pub pos: i64,
	pub course_id: Option<i64>,
	pub title: Option<String>,
	pub clear_type: Option<String>,
	pub play_count: Option<i64>,
	pub rank_pos: Option<i64>,
	pub rank_total: Option<i64>,
}

#[derive(Debug)]
pub struct UserBbsEntry {
	pub pos: i64,
	pub commenter_id: Option<i64>,
	pub commenter_name: Option<String>,
	pub message: Option<String>,
	pub posted_at: Option<String>,
}

/// Everything scraped from one user mypage.
#[derive(Debug)]
pub struct UserPageData {
	pub profile: UserProfile,
	pub rivals: Vec<UserRival>,
	pub most_plays: Vec<UserPlayEntry>,
	pub recent_plays: Vec<UserPlayEntry>,
	pub recent_courses: Vec<UserCourseEntry>,
	pub bbs: Vec<UserBbsEntry>,
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Parse a user mypage HTML document.  Returns `None` if the profile table is absent.
pub fn parse_user(player_id: i64, html: &str) -> Option<UserPageData> {
	let document = Html::parse_document(html);

	let profile_table_sel = Selector::parse(r#"table[border="0"]"#).unwrap();
	let clear_table_sel = Selector::parse(r#"table[border="1"]"#).unwrap();
	let tr_sel = Selector::parse("tr").unwrap();
	let th_sel = Selector::parse("th").unwrap();
	let td_sel = Selector::parse("td").unwrap();
	let a_sel = Selector::parse("a").unwrap();

	let profile_table = document.select(&profile_table_sel).next()?;

	// Build a key → value map from <th>/<td> pairs.
	let mut fields: HashMap<String, String> = HashMap::new();
	let mut rivals = vec![];
	let mut bio = String::new();

	for tr in profile_table.select(&tr_sel) {
		let key = tr
			.select(&th_sel)
			.next()
			.map(|th| collect_text(&th))
			.unwrap_or_default();
		let Some(td) = tr.select(&td_sel).next() else {
			continue;
		};

		match key.trim() {
			"ライバル" => {
				rivals = td
					.select(&a_sel)
					.filter_map(|a| {
						let href = a.value().attr("href")?;
						let rival_id = extract_playerid(href)?;
						Some(UserRival {
							rival_id,
							rival_name: collect_text(&a).trim().to_string(),
						})
					})
					.collect();
			}
			"自己紹介" => {
				bio = collect_text_with_br(&td);
			}
			k if !k.is_empty() => {
				fields.insert(k.to_string(), collect_text(&td));
			}
			_ => {}
		}
	}

	let name = fields.get("プレイヤー名").cloned().unwrap_or_default();
	let dan = fields.get("段位認定").cloned().unwrap_or_default();
	let songs_played = parse_i64_prefix(
		fields
			.get("プレイした曲数")
			.map(String::as_str)
			.unwrap_or(""),
	);
	let play_count = parse_i64_prefix(
		fields
			.get("プレイした回数")
			.map(String::as_str)
			.unwrap_or(""),
	);
	let lr2id = fields
		.get("LR2ID")
		.and_then(|s| s.trim().parse::<i64>().ok())
		.unwrap_or(player_id);

	// Clear histogram: border=1 tables in document order.
	// Index 0 = clear histogram, 1 = most plays, 2 = recent plays, 3 = recent courses.
	let border1_tables: Vec<_> = document.select(&clear_table_sel).collect();

	// Detect privacy level by inspecting the first border=1 table.
	// Public profiles:			 clear histogram (first <th> = "FULLCOMBO") + 5-col play tables.
	// "playcount" privacy:		 no histogram, only 4-col play tables (play count stripped).
	// "full" privacy:			  no border=1 tables at all.
	let first_th_text = border1_tables.first().and_then(|t| {
		let th_sel = Selector::parse("th").unwrap();
		t.select(&th_sel).next().map(|th| collect_text(&th))
	});
	let privacy_level: Option<String> = if border1_tables.is_empty() {
		Some("full".to_string())
	} else if first_th_text.as_deref() != Some("FULLCOMBO") {
		Some("playcount".to_string())
	} else {
		None
	};

	let (fc, pfc, hard, normal, easy, failed) = border1_tables
		.first()
		.map(|t| {
			let rows: Vec<_> = t.select(&tr_sel).collect();
			let cell = |col: usize| -> String {
				rows.get(1)
					.and_then(|tr| tr.select(&td_sel).nth(col))
					.map(|td| collect_text(&td))
					.unwrap_or_default()
			};
			let (fc, pfc) = parse_fc_cell(&cell(0));
			(
				fc,
				pfc,
				parse_i64_prefix(&cell(1)),
				parse_i64_prefix(&cell(2)),
				parse_i64_prefix(&cell(3)),
				parse_i64_prefix(&cell(4)),
			)
		})
		.unwrap_or((0, None, 0, 0, 0, 0));

	// When the clear histogram is absent (playcount privacy), the play tables shift down by one.
	let play_offset: usize = if privacy_level.is_none() { 1 } else { 0 };
	let most_plays = parse_play_table(border1_tables.get(play_offset), &tr_sel, &td_sel, &a_sel);
	let recent_plays = parse_play_table(
		border1_tables.get(play_offset + 1),
		&tr_sel,
		&td_sel,
		&a_sel,
	);
	let recent_courses = parse_course_table(
		border1_tables.get(play_offset + 2),
		&tr_sel,
		&td_sel,
		&a_sel,
	);
	let bbs = parse_bbs(&document, &tr_sel, &td_sel, &a_sel);

	Some(UserPageData {
		profile: UserProfile {
			player_id: lr2id,
			name,
			dan,
			bio,
			privacy_level,
			songs_played,
			play_count,
			fc_count: fc,
			perfect_fc_count: pfc,
			hard_count: hard,
			normal_count: normal,
			easy_count: easy,
			failed_count: failed,
		},
		rivals,
		most_plays,
		recent_plays,
		recent_courses,
		bbs,
	})
}

// ── Internal ──────────────────────────────────────────────────────────────────

/// Parse a song play-list table (most-played or recently-played).
/// Public layout (5 cols):           pos | title | clear | play_count | rank
/// Playcount-hidden layout (4 cols): pos | title | clear | rank
fn parse_play_table(
	table: Option<&scraper::ElementRef>,
	tr_sel: &Selector,
	td_sel: &Selector,
	a_sel: &Selector,
) -> Vec<UserPlayEntry> {
	let Some(table) = table else { return vec![] };
	table
		.select(tr_sel)
		.skip(1) // header row
		.filter_map(|tr| {
			let tds: Vec<_> = tr.select(td_sel).collect();
			if tds.len() < 4 {
				return None;
			}
			let pos: i64 = collect_text(&tds[0]).trim().parse().ok()?;
			let link = tds[1].select(a_sel).next();
			let bmsid = link
				.and_then(|a| a.value().attr("href"))
				.and_then(extract_bmsid);
			let title = link
				.map(|a| collect_text(&a).trim().to_string())
				.filter(|s| !s.is_empty());
			let clear_type =
				Some(collect_text(&tds[2]).trim().to_string()).filter(|s| !s.is_empty());
			let (play_count, rank_col) = if tds.len() >= 5 {
				(
					collect_text(&tds[3]).trim().parse::<i64>().ok(),
					collect_text(&tds[4]),
				)
			} else {
				(None, collect_text(&tds[3]))
			};
			let (rank_pos, rank_total) = parse_ranking(&rank_col);
			Some(UserPlayEntry {
				pos,
				bmsid,
				title,
				clear_type,
				play_count,
				rank_pos,
				rank_total,
			})
		})
		.collect()
}

/// Parse a course play-list table (recently-played courses).
/// Public layout (5 cols):           pos | title | clear | play_count | rank
/// Playcount-hidden layout (4 cols): pos | title | clear | rank
fn parse_course_table(
	table: Option<&scraper::ElementRef>,
	tr_sel: &Selector,
	td_sel: &Selector,
	a_sel: &Selector,
) -> Vec<UserCourseEntry> {
	let Some(table) = table else { return vec![] };
	table
		.select(tr_sel)
		.skip(1)
		.filter_map(|tr| {
			let tds: Vec<_> = tr.select(td_sel).collect();
			if tds.len() < 4 {
				return None;
			}
			let pos: i64 = collect_text(&tds[0]).trim().parse().ok()?;
			let link = tds[1].select(a_sel).next();
			let course_id = link
				.and_then(|a| a.value().attr("href"))
				.and_then(extract_courseid);
			let title = link
				.map(|a| collect_text(&a).trim().to_string())
				.filter(|s| !s.is_empty());
			let clear_type =
				Some(collect_text(&tds[2]).trim().to_string()).filter(|s| !s.is_empty());
			let (play_count, rank_col) = if tds.len() >= 5 {
				(
					collect_text(&tds[3]).trim().parse::<i64>().ok(),
					collect_text(&tds[4]),
				)
			} else {
				(None, collect_text(&tds[3]))
			};
			let (rank_pos, rank_total) = parse_ranking(&rank_col);
			Some(UserCourseEntry {
				pos,
				course_id,
				title,
				clear_type,
				play_count,
				rank_pos,
				rank_total,
			})
		})
		.collect()
}

/// Parse the 一行BBS guestbook section (the table after `<h3>一行BBS</h3>`).
fn parse_bbs(
	document: &Html,
	tr_sel: &Selector,
	td_sel: &Selector,
	a_sel: &Selector,
) -> Vec<UserBbsEntry> {
	let h3_sel = Selector::parse("h3").unwrap();

	let bbs_h3 = document
		.select(&h3_sel)
		.find(|e| collect_text(e).trim() == "一行BBS");
	let Some(h3) = bbs_h3 else { return vec![] };

	// Walk next siblings until we find a <table>.
	let mut table = None;
	let mut sib = h3.next_sibling_element();
	while let Some(elem) = sib {
		if elem.value().name() == "table" {
			table = Some(elem);
			break;
		}
		sib = elem.next_sibling_element();
	}
	let Some(table) = table else { return vec![] };

	table
		.select(tr_sel)
		.enumerate()
		.filter_map(|(pos, tr)| {
			let td = tr.select(td_sel).next()?;
			let commenter_link = td.select(a_sel).next();
			let commenter_id = commenter_link
				.and_then(|a| a.value().attr("href"))
				.and_then(extract_playerid);
			let commenter_name = commenter_link
				.map(|a| collect_text(&a).trim().to_string())
				.filter(|s| !s.is_empty());

			let full_text = collect_text(&td);
			// Format: " NAME:message text [YYYY-MM-DD HH:MM:SS]"
			let (message, posted_at) = if let Some(ts_start) = full_text.rfind(" [") {
				let ts = full_text[ts_start + 2..]
					.trim_end_matches(']')
					.trim()
					.to_string();
				let before = full_text[..ts_start].trim();
				// Strip "NAME:" prefix — find first ':' after the name
				let msg = before
					.find(':')
					.map(|i| before[i + 1..].trim().to_string())
					.filter(|s| !s.is_empty());
				(msg, Some(ts).filter(|s| !s.is_empty()))
			} else {
				(None, None)
			};

			Some(UserBbsEntry {
				pos: pos as i64,
				commenter_id,
				commenter_name,
				message,
				posted_at,
			})
		})
		.collect()
}

// ── Small helpers ─────────────────────────────────────────────────────────────

fn extract_playerid(href: &str) -> Option<i64> {
	href.split("playerid=")
		.nth(1)
		.and_then(|s| s.split('&').next())
		.and_then(|s| s.parse().ok())
}

fn extract_bmsid(href: &str) -> Option<i64> {
	href.split("bmsid=")
		.nth(1)
		.and_then(|s| s.split('&').next())
		.and_then(|s| s.parse().ok())
}

fn extract_courseid(href: &str) -> Option<i64> {
	href.split("courseid=")
		.nth(1)
		.and_then(|s| s.split('&').next())
		.and_then(|s| s.parse().ok())
}

/// Parse "9/9033" into (Some(9), Some(9033)).
fn parse_ranking(s: &str) -> (Option<i64>, Option<i64>) {
	if let Some((a, b)) = s.trim().split_once('/') {
		(a.trim().parse().ok(), b.trim().parse().ok())
	} else {
		(None, None)
	}
}

/// Parse the leading integer from a string that may have a suffix like `499(★33)`.
fn parse_i64_prefix(s: &str) -> i64 {
	s.trim()
		.split(|c: char| !c.is_ascii_digit())
		.next()
		.and_then(|n| n.parse::<i64>().ok())
		.unwrap_or(0)
}

/// Parse `499(★33)` into `(499, Some(33))`.
/// The parenthetical is the perfect-full-combo (PFC) count shown as `★N`.
fn parse_fc_cell(s: &str) -> (i64, Option<i64>) {
	let fc = parse_i64_prefix(s);
	let pfc = s.find('(').and_then(|open| {
		let rest = &s[open + 1..];
		rest.find(')')
			.map(|close| &rest[..close])
			.and_then(|inner| {
				inner
					.split(|c: char| !c.is_ascii_digit())
					.find(|t| !t.is_empty())
					.and_then(|n| n.parse::<i64>().ok())
			})
	});
	(fc, pfc)
}
