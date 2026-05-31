use scraper::{Html, Selector};

use super::{collect_text, RankingRow};

// ── Public types ──────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct CourseMeta {
	pub course_id: i64,
	pub title: String,
	pub category: String,
	pub creator_id: Option<i64>,
	pub creator_name: String,
	pub keys: String,
	pub play_count: i64,
	pub play_people: i64,
	pub clear_count: i64,
	pub clear_people: i64,
	pub fc_count: i64,
	pub hard_count: i64,
	pub normal_count: i64,
	pub easy_count: i64,
	pub failed_count: i64,
	pub stages: Vec<CourseStage>,
}

#[derive(Debug)]
pub struct CourseStage {
	pub stage: i64,
	pub bmsid: Option<i64>,
	pub label: String,
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Parse one course ranking HTML page.
/// Returns the course metadata (present on every page) and all ranking rows.
pub fn parse_page(course_id: i64, html: &str) -> (Option<CourseMeta>, Vec<RankingRow>) {
	let document = Html::parse_document(html);
	let meta = parse_meta(&document, course_id);
	let rows = super::parse_ranking_rows(&document);
	(meta, rows)
}

// ── Internal ──────────────────────────────────────────────────────────────────

fn parse_meta(document: &Html, course_id: i64) -> Option<CourseMeta> {
	let h1_sel = Selector::parse("h1").unwrap();
	let h2_sel = Selector::parse("h2").unwrap();
	let h4_sel = Selector::parse("h4").unwrap();
	let a_sel = Selector::parse("a").unwrap();
	let tr_sel = Selector::parse("tr").unwrap();
	let th_sel = Selector::parse("th").unwrap();
	let td_sel = Selector::parse("td").unwrap();
	let table_sel = Selector::parse("table[border=\"0\"]").unwrap();

	let title = document
		.select(&h1_sel)
		.next()
		.map(|e| collect_text(&e))
		.unwrap_or_default();
	let category = document
		.select(&h4_sel)
		.next()
		.map(|e| collect_text(&e))
		.unwrap_or_default();

	if title.is_empty() {
		return None;
	}

	// Creator is in <h2>: the first <a> link holds player_id and name.
	let (creator_id, creator_name) = document
		.select(&h2_sel)
		.next()
		.and_then(|h2| h2.select(&a_sel).next())
		.map(|a| {
			let href = a.value().attr("href").unwrap_or("");
			let id = href
				.split("maker=")
				.nth(1)
				.and_then(|s| s.split('&').next())
				.and_then(|s| s.parse::<i64>().ok());
			(id, collect_text(&a))
		})
		.unwrap_or((None, String::new()));

	// The first border=0 table is the info table (stages, keys, tags).
	let tables: Vec<_> = document.select(&table_sel).collect();
	let info_table = tables.first()?;

	let mut stages = vec![];
	let mut keys = String::new();

	for tr in info_table.select(&tr_sel) {
		let key = tr
			.select(&th_sel)
			.next()
			.map(|th| collect_text(&th))
			.unwrap_or_default();
		let Some(td) = tr.select(&td_sel).next() else {
			continue;
		};

		if key.starts_with("STAGE") {
			// "STAGE N" → stage number N
			let stage_num: i64 = key
				.trim_start_matches("STAGE")
				.trim()
				.parse()
				.unwrap_or((stages.len() + 1) as i64);
			let link = td.select(&a_sel).next();
			let bmsid = link.and_then(|a| a.value().attr("href")).and_then(|href| {
				href.split("bmsid=")
					.nth(1)
					.and_then(|s| s.split('&').next())
					.and_then(|s| s.parse::<i64>().ok())
			});
			let label = link.map(|a| collect_text(&a)).unwrap_or_default();
			stages.push(CourseStage {
				stage: stage_num,
				bmsid,
				label,
			});
		} else if key == "鍵盤数" {
			keys = collect_text(&td);
		}
	}

	// The second border=0 table is the overall stats table.
	// Row structure:
	//   header: | | プレイ | クリア | クリアレート |
	//   回数:   | 回数 | play_count | clear_count | … |
	//   人数:   | 人数 | play_people | clear_people | … |
	let stats_table = tables.get(1);
	let (mut play_count, mut play_people, mut clear_count, mut clear_people) =
		(0i64, 0i64, 0i64, 0i64);

	if let Some(st) = stats_table {
		for tr in st.select(&tr_sel) {
			let th_text = tr
				.select(&th_sel)
				.next()
				.map(|t| collect_text(&t))
				.unwrap_or_default();
			let tds: Vec<_> = tr.select(&td_sel).collect();
			match th_text.as_str() {
				"回数" => {
					play_count = tds
						.first()
						.map(|t| collect_text(t).parse().unwrap_or(0))
						.unwrap_or(0);
					clear_count = tds
						.get(1)
						.map(|t| collect_text(t).parse().unwrap_or(0))
						.unwrap_or(0);
				}
				"人数" => {
					play_people = tds
						.first()
						.map(|t| collect_text(t).parse().unwrap_or(0))
						.unwrap_or(0);
					clear_people = tds
						.get(1)
						.map(|t| collect_text(t).parse().unwrap_or(0))
						.unwrap_or(0);
				}
				_ => {}
			}
		}
	}

	// The third border=0 table has the per-clear-type breakdown.
	// Header row: | | FULLCOMBO | HARD | NORMAL | EASY | FAILED |
	// "クリア人数内訳" row: | label | fc | hard | normal | easy | failed |
	let (mut fc, mut hard, mut normal, mut easy, mut failed) = (0i64, 0i64, 0i64, 0i64, 0i64);
	if let Some(bt) = tables.get(2) {
		for tr in bt.select(&tr_sel) {
			let th_text = tr
				.select(&th_sel)
				.next()
				.map(|t| collect_text(&t))
				.unwrap_or_default();
			if th_text == "クリア人数内訳" {
				let tds: Vec<_> = tr.select(&td_sel).collect();
				let cell = |i: usize| {
					tds.get(i)
						.map(|t| collect_text(t).parse::<i64>().unwrap_or(0))
						.unwrap_or(0)
				};
				fc = cell(0);
				hard = cell(1);
				normal = cell(2);
				easy = cell(3);
				failed = cell(4);
			}
		}
	}

	Some(CourseMeta {
		course_id,
		title,
		category,
		creator_id,
		creator_name,
		keys,
		play_count,
		play_people,
		clear_count,
		clear_people,
		fc_count: fc,
		hard_count: hard,
		normal_count: normal,
		easy_count: easy,
		failed_count: failed,
		stages,
	})
}
