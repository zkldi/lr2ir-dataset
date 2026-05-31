pub mod course;
pub mod searchcgi;
pub mod user;

use scraper::{ElementRef, Html, Selector};

// ── Public types ──────────────────────────────────────────────────────────────

/// One ranking entry (personal best) shared by both chart and course pages.
#[derive(Debug, Default)]
pub struct RankingRow {
	pub rank: i64,
	pub player_id: i64,
	pub player_name: String,
	pub dan: String,
	pub clear_type: String,
	pub letter_rank: String,
	pub score: i64,
	pub score_max: i64,
	pub combo: i64,
	pub combo_max: i64,
	pub bad_poor: i64,
	pub pgreat: i64,
	pub great: i64,
	pub good: i64,
	pub bad: i64,
	pub poor: i64,
	/// SP charts (7/9/5KEY) have option_1 and option_2.
	/// DP charts (14/10KEY) additionally have option_3 and option_4.
	pub option_1: Option<String>,
	pub option_2: Option<String>,
	pub option_3: Option<String>,
	pub option_4: Option<String>,
	pub input: String,
	pub client: String,
	pub note: String,
}

// ── Shared ranking-row parser ─────────────────────────────────────────────────

/// Extract all ranking entries from a document.
/// Works for both searchcgi chart pages and course ranking pages.
pub fn parse_ranking_rows(document: &Html) -> Vec<RankingRow> {
	let table_sel = Selector::parse("table").unwrap();
	let tr_sel = Selector::parse("tr").unwrap();
	let td_rowspan_sel = Selector::parse(r#"td[rowspan="2"]"#).unwrap();
	let td_gray_sel = Selector::parse("td.gray").unwrap();
	let td_sel = Selector::parse("td").unwrap();
	let a_sel = Selector::parse("a").unwrap();

	// The ranking table is the one that has td[rowspan="2"] rank cells.
	let Some(table) = document
		.select(&table_sel)
		.find(|t| t.select(&td_rowspan_sel).next().is_some())
	else {
		return Vec::new();
	};

	let mut rows: Vec<RankingRow> = Vec::new();
	let mut pending: Option<PartialRow> = None;

	for tr in table.select(&tr_sel) {
		if tr.select(&td_rowspan_sel).next().is_some() {
			if let Some(p) = pending.take() {
				rows.push(p.into_row(String::new()));
			}
			let tds: Vec<_> = tr.select(&td_sel).collect();
			if let Some(p) = parse_main_row(&tds, &a_sel) {
				pending = Some(p);
			}
		} else if let Some(gray_td) = tr.select(&td_gray_sel).next() {
			if let Some(p) = pending.take() {
				rows.push(p.into_row(collect_text_with_br(&gray_td)));
			}
		}
	}
	if let Some(p) = pending {
		rows.push(p.into_row(String::new()));
	}

	rows
}

// ── Internal ──────────────────────────────────────────────────────────────────

struct PartialRow {
	rank: i64,
	player_id: i64,
	player_name: String,
	dan: String,
	clear_type: String,
	letter_rank: String,
	score: i64,
	score_max: i64,
	combo: i64,
	combo_max: i64,
	bad_poor: i64,
	pgreat: i64,
	great: i64,
	good: i64,
	bad: i64,
	poor: i64,
	option_1: Option<String>,
	option_2: Option<String>,
	option_3: Option<String>,
	option_4: Option<String>,
	input: String,
	client: String,
}

impl PartialRow {
	fn into_row(self, note: String) -> RankingRow {
		RankingRow {
			rank: self.rank,
			player_id: self.player_id,
			player_name: self.player_name,
			dan: self.dan,
			clear_type: self.clear_type,
			letter_rank: self.letter_rank,
			score: self.score,
			score_max: self.score_max,
			combo: self.combo,
			combo_max: self.combo_max,
			bad_poor: self.bad_poor,
			pgreat: self.pgreat,
			great: self.great,
			good: self.good,
			bad: self.bad,
			poor: self.poor,
			option_1: self.option_1,
			option_2: self.option_2,
			option_3: self.option_3,
			option_4: self.option_4,
			input: self.input,
			client: self.client,
			note,
		}
	}
}

fn parse_main_row(tds: &[ElementRef], a_sel: &Selector) -> Option<PartialRow> {
	// Layout: rank(0) player(1) dan(2) clear(3) letter(4) score(5) combo(6)
	//		 bp(7) pg(8) gr(9) gd(10) bd(11) pr(12)
	//		 OP…(13..n-2) input(n-2) client(n-1)
	// Minimum with zero OP cols would be 15; in practice always ≥17.
	if tds.len() < 15 {
		return None;
	}

	let rank = collect_text(&tds[0]).parse::<i64>().unwrap_or(0);

	let player_link = tds[1].select(a_sel).next();
	let player_name = player_link.map(|a| collect_text(&a)).unwrap_or_default();
	let player_id = player_link
		.and_then(|a| a.value().attr("href"))
		.and_then(extract_player_id)
		.unwrap_or(0);

	let dan = collect_text(&tds[2]);
	let clear_type = collect_text(&tds[3]);
	let letter_rank = collect_text(&tds[4]);

	let (score, score_max) = parse_score_field(&collect_text(&tds[5]));
	let (combo, combo_max) = parse_slash_pair(&collect_text(&tds[6]));

	let bad_poor = parse_i64(&tds[7]);
	let pgreat = parse_i64(&tds[8]);
	let great = parse_i64(&tds[9]);
	let good = parse_i64(&tds[10]);
	let bad = parse_i64(&tds[11]);
	let poor = parse_i64(&tds[12]);

	// OP columns sit between PR (index 12) and INPUT (second-to-last) / client (last).
	// SP (7/9/5KEY): 17 cols total → 2 OP cols at indices 13, 14
	// DP (14/10KEY): 19 cols total → 4 OP cols at indices 13, 14, 15, 16
	let n = tds.len();
	let op = |i: usize| -> Option<String> {
		// Only present if the index falls within the OP window [13, n-2).
		if i >= 13 && i + 2 < n {
			Some(collect_text(&tds[i]))
		} else {
			None
		}
	};
	let option_1 = op(13);
	let option_2 = op(14);
	let option_3 = op(15);
	let option_4 = op(16);

	let input = if n >= 2 {
		collect_text(&tds[n - 2])
	} else {
		String::new()
	};
	let client = if n >= 1 {
		collect_text(&tds[n - 1])
	} else {
		String::new()
	};

	Some(PartialRow {
		rank,
		player_id,
		player_name,
		dan,
		clear_type,
		letter_rank,
		score,
		score_max,
		combo,
		combo_max,
		bad_poor,
		pgreat,
		great,
		good,
		bad,
		poor,
		option_1,
		option_2,
		option_3,
		option_4,
		input,
		client,
	})
}

// ── Helpers visible to child modules ─────────────────────────────────────────
// (private items in a parent module are accessible from descendant modules)

fn collect_text(el: &ElementRef) -> String {
	el.text().collect::<String>().trim().to_string()
}

/// Like `collect_text` but converts `<br>` elements to `\n`, preserving
/// line breaks that the original page used for formatting.
pub(crate) fn collect_text_with_br(el: &ElementRef) -> String {
	use scraper::node::Node;
	let mut out = String::new();
	for node in el.descendants() {
		match node.value() {
			Node::Text(t) => out.push_str(t),
			Node::Element(e) if e.name() == "br" => out.push('\n'),
			_ => {}
		}
	}
	out.trim().to_string()
}

fn parse_i64(el: &ElementRef) -> i64 {
	collect_text(el).parse::<i64>().unwrap_or(0)
}

fn extract_player_id(href: &str) -> Option<i64> {
	href.split("playerid=")
		.nth(1)
		.and_then(|s| s.split('&').next())
		.and_then(|s| s.parse::<i64>().ok())
}

fn parse_score_field(s: &str) -> (i64, i64) {
	// "6820/6828(99.88%)" → (6820, 6828)
	let s = s.split('(').next().unwrap_or(s);
	parse_slash_pair(s)
}

fn parse_slash_pair(s: &str) -> (i64, i64) {
	let mut it = s.trim().splitn(2, '/');
	let a = it.next().unwrap_or("").trim().parse::<i64>().unwrap_or(0);
	let b = it.next().unwrap_or("").trim().parse::<i64>().unwrap_or(0);
	(a, b)
}
