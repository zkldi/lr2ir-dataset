use scraper::{Element, ElementRef, Html, Selector};

use super::{collect_text, collect_text_with_br, RankingRow};

// ── Public types ──────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct ChartMeta {
	pub md5: String,
	/// Extracted from the `editlogList` href present on every chart page.
	/// NULL only for charts with no edit-log link (very rare/broken pages).
	pub bmsid: Option<i64>,
	pub title: String,
	pub genre: String,
	pub artist: String,
	pub bpm_min: String,
	pub bpm_max: String,
	/// `true` when the page contains "このランキングは停止中です。"
	pub suspended: bool,
	/// Raw level string, e.g. "☆16 / ★20".
	pub level: String,
	pub keys: String,
	pub judge_rank: String,
	// ── Aggregate play / clear counts (first status table) ───────────────────
	pub play_count: i64,
	pub play_people: i64,
	pub clear_count: i64,
	pub clear_people: i64,
	// ── Clear-type breakdown (second status table) ───────────────────────────
	pub fc_count: i64,
	pub hard_count: i64,
	pub normal_count: i64,
	pub easy_count: i64,
	pub failed_count: i64,
	// ── Extra info-table fields ───────────────────────────────────────────────
	pub last_updated_by: Option<String>,
	pub last_updated_at: Option<String>,
	pub body_url: Option<String>,
	pub diff_url: Option<String>,
	/// 備考 free-text notes field.
	pub comment: Option<String>,
	/// Up to 10 tags; empty slots are omitted so the Vec may be shorter.
	pub tags: Vec<String>,
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Parse one searchcgi HTML page.
///
/// Returns `(Some(ChartMeta), ranking_rows)` for page 1, and `(None, ranking_rows)`
/// for subsequent pages.  Returns `(None, [])` for any page that contains the
/// "song not registered" message, so the chart is never written to the DB.
pub fn parse_page(md5: &str, page_num: u32, html: &str) -> (Option<ChartMeta>, Vec<RankingRow>) {
	if html.contains("この曲は登録されていません。") {
		return (None, vec![]);
	}
	let suspended = html.contains("このランキングは停止中です。");
	let document = Html::parse_document(html);
	let rows = super::parse_ranking_rows(&document);
	let meta = if page_num == 1 {
		let mut m = parse_meta(md5, html, &document);
		m.suspended = suspended;
		Some(m)
	} else {
		None
	};
	(meta, rows)
}

// ── Internal ──────────────────────────────────────────────────────────────────

fn parse_meta(md5: &str, raw_html: &str, document: &Html) -> ChartMeta {
	let h1_sel = Selector::parse("h1").unwrap();
	let h2_sel = Selector::parse("h2").unwrap();
	let h4_sel = Selector::parse("h4").unwrap();
	let tr_sel = Selector::parse("tr").unwrap();
	let th_sel = Selector::parse("th").unwrap();
	let td_sel = Selector::parse("td").unwrap();
	let a_sel = Selector::parse("a").unwrap();

	let title = document
		.select(&h1_sel)
		.next()
		.map(|e| collect_text(&e))
		.unwrap_or_default();
	let artist = document
		.select(&h2_sel)
		.next()
		.map(|e| collect_text(&e))
		.unwrap_or_default();
	let genre = document
		.select(&h4_sel)
		.next()
		.map(|e| collect_text(&e))
		.unwrap_or_default();

	// bmsid: prefer the editlogList link (present on every chart page) and fall
	// back to the ">>" pagination link (only present when there are 100+ players).
	let bmsid = document
		.select(&a_sel)
		.find(|a| {
			a.value()
				.attr("href")
				.map(|h| h.contains("mode=editlogList"))
				.unwrap_or(false)
		})
		.or_else(|| document.select(&a_sel).find(|a| collect_text(a) == ">>"))
		.and_then(|a| a.value().attr("href"))
		.and_then(extract_bmsid);

	// First <table>: the info table.  Row 0 has BPM/level/keys/judge_rank.
	// Subsequent rows are keyed by <th> text.
	let table_sel = Selector::parse("table").unwrap();
	let (bpm_min, bpm_max, level, keys, judge_rank, body_url, diff_url, comment, tags) =
		parse_info_table(document, &table_sel, &tr_sel, &th_sel, &td_sel, &a_sel);

	let (last_updated_by, last_updated_at) = parse_last_updated(raw_html);

	let (
		play_count,
		play_people,
		clear_count,
		clear_people,
		fc_count,
		hard_count,
		normal_count,
		easy_count,
		failed_count,
	) = parse_status_tables(document, &tr_sel, &td_sel);

	ChartMeta {
		md5: md5.to_string(),
		bmsid,
		suspended: false, // overwritten by parse_page
		title,
		genre,
		artist,
		bpm_min,
		bpm_max,
		level,
		keys,
		judge_rank,
		play_count,
		play_people,
		clear_count,
		clear_people,
		fc_count,
		hard_count,
		normal_count,
		easy_count,
		failed_count,
		last_updated_by,
		last_updated_at,
		body_url,
		diff_url,
		comment,
		tags,
	}
}

/// Parse the first `<table>` in the document (the info table).
///
/// Row 0: BPM | level | keys | judge_rank  (th/td pairs)
/// Rows 1+: identified by `<th>` text — タグ / 本体URL / 差分URL / 備考
#[allow(clippy::type_complexity)]
fn parse_info_table(
	document: &Html,
	table_sel: &Selector,
	tr_sel: &Selector,
	th_sel: &Selector,
	td_sel: &Selector,
	a_sel: &Selector,
) -> (
	String,
	String, // bpm_min, bpm_max
	String,
	String,         // level, keys
	String,         // judge_rank
	Option<String>, // body_url
	Option<String>, // diff_url
	Option<String>, // comment
	Vec<String>,    // tags (non-empty only, up to 10)
) {
	let Some(table) = document.select(table_sel).next() else {
		return Default::default();
	};

	let mut bpm_min = String::new();
	let mut bpm_max = String::new();
	let mut level = String::new();
	let mut keys = String::new();
	let mut judge_rank = String::new();
	let mut body_url: Option<String> = None;
	let mut diff_url: Option<String> = None;
	let mut comment: Option<String> = None;
	let mut tags: Vec<String> = vec![];

	for (i, tr) in table.select(tr_sel).enumerate() {
		if i == 0 {
			// BPM | level | keys | judge_rank
			let tds: Vec<_> = tr.select(td_sel).collect();
			let bpm = tds.first().map(|t| collect_text(t)).unwrap_or_default();
			level = tds.get(1).map(|t| collect_text(t)).unwrap_or_default();
			keys = tds.get(2).map(|t| collect_text(t)).unwrap_or_default();
			judge_rank = tds.get(3).map(|t| collect_text(t)).unwrap_or_default();
			(bpm_min, bpm_max) = split_bpm(&bpm);
			continue;
		}

		let th_text = tr
			.select(th_sel)
			.next()
			.map(|e| collect_text(&e))
			.unwrap_or_default();
		let first_td: Option<ElementRef> = tr.select(td_sel).next();

		match th_text.trim() {
			"タグ" => {
				tags = tr
					.select(a_sel)
					.map(|a| collect_text(&a))
					.filter(|s| !s.is_empty())
					.take(10)
					.collect();
			}
			"本体URL" => {
				body_url = first_td
					.as_ref()
					.and_then(|td| td.select(a_sel).next())
					.and_then(|a| a.value().attr("href"))
					.map(str::to_string);
			}
			"差分URL" => {
				diff_url = first_td
					.as_ref()
					.and_then(|td| td.select(a_sel).next())
					.and_then(|a| a.value().attr("href"))
					.map(str::to_string);
			}
			"備考" => {
				comment = first_td
					.map(|td| collect_text_with_br(&td))
					.filter(|s| !s.is_empty());
			}
			_ => {}
		}
	}

	(
		bpm_min, bpm_max, level, keys, judge_rank, body_url, diff_url, comment, tags,
	)
}

/// Extract `最終更新者 NAME (YYYY-MM-DD HH:MM:SS)` from the raw (decoded) HTML.
fn parse_last_updated(html: &str) -> (Option<String>, Option<String>) {
	let marker = "最終更新者";
	let Some(pos) = html.find(marker) else {
		return (None, None);
	};
	let after = html[pos + marker.len()..].trim_start();
	// Format: "NAME (YYYY-MM-DD HH:MM:SS)"
	let Some(open) = after.find(" (") else {
		return (None, None);
	};
	let name = after[..open].trim().to_string();
	let rest = &after[open + 2..];
	let Some(close) = rest.find(')') else {
		return (None, None);
	};
	let timestamp = rest[..close].trim().to_string();
	(
		if name.is_empty() { None } else { Some(name) },
		if timestamp.is_empty() {
			None
		} else {
			Some(timestamp)
		},
	)
}

/// Walk siblings after `<a name="status">` to collect all `<table>` elements
/// in the status section, then extract:
///
/// Table 0 — aggregate counts: (play_count, play_people, clear_count, clear_people)
/// Table 1 — clear-type breakdown: (fc, hard, normal, easy, failed)
#[allow(clippy::type_complexity)]
fn parse_status_tables(
	document: &Html,
	tr_sel: &Selector,
	td_sel: &Selector,
) -> (i64, i64, i64, i64, i64, i64, i64, i64, i64) {
	let anchor_sel = Selector::parse(r#"a[name="status"]"#).unwrap();

	let Some(anchor) = document.select(&anchor_sel).next() else {
		return (0, 0, 0, 0, 0, 0, 0, 0, 0);
	};
	let Some(h3) = anchor.parent_element() else {
		return (0, 0, 0, 0, 0, 0, 0, 0, 0);
	};

	let mut tables: Vec<ElementRef> = vec![];
	let mut sibling = h3.next_sibling_element();
	while let Some(elem) = sibling {
		match elem.value().name() {
			"table" => tables.push(elem),
			"h3" => break,
			_ => {}
		}
		sibling = elem.next_sibling_element();
	}

	let td_i64 = |table: &ElementRef, row: usize, col: usize| -> i64 {
		let rows: Vec<_> = table.select(tr_sel).collect();
		rows.get(row)
			.and_then(|tr| tr.select(td_sel).nth(col))
			.map(|td| collect_text(&td).parse::<i64>().unwrap_or(0))
			.unwrap_or(0)
	};

	// Table 0: row 1 = play/clear counts, row 2 = play/clear people
	let (play_count, play_people, clear_count, clear_people) = if let Some(t) = tables.first() {
		(
			td_i64(t, 1, 0),
			td_i64(t, 2, 0),
			td_i64(t, 1, 1),
			td_i64(t, 2, 1),
		)
	} else {
		(0, 0, 0, 0)
	};

	// Table 1: row 1 = fc/hard/normal/easy/failed player counts
	let (fc_count, hard_count, normal_count, easy_count, failed_count) =
		if let Some(t) = tables.get(1) {
			(
				td_i64(t, 1, 0),
				td_i64(t, 1, 1),
				td_i64(t, 1, 2),
				td_i64(t, 1, 3),
				td_i64(t, 1, 4),
			)
		} else {
			(0, 0, 0, 0, 0)
		};

	(
		play_count,
		play_people,
		clear_count,
		clear_people,
		fc_count,
		hard_count,
		normal_count,
		easy_count,
		failed_count,
	)
}

fn split_bpm(s: &str) -> (String, String) {
	if let Some((a, b)) = s.split_once(" - ") {
		(a.trim().to_string(), b.trim().to_string())
	} else {
		let v = s.trim().to_string();
		(v.clone(), v)
	}
}

fn extract_bmsid(href: &str) -> Option<i64> {
	href.split("bmsid=")
		.nth(1)
		.and_then(|s| s.split('&').next())
		.and_then(|s| s.parse::<i64>().ok())
}
