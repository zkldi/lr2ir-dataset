pub fn fmt_num(n: i64) -> String {
	let s = n.abs().to_string();
	let mut out = String::with_capacity(s.len() + s.len() / 3);
	for (i, ch) in s.chars().enumerate() {
		if i > 0 && (s.len() - i).is_multiple_of(3) {
			out.push(',');
		}
		out.push(ch);
	}
	if n < 0 {
		format!("-{}", out)
	} else {
		out
	}
}

pub fn score_pct(score: i64, score_max: i64) -> String {
	if score_max == 0 {
		"–".to_string()
	} else {
		format!("{:.2}%", score as f64 * 100.0 / score_max as f64)
	}
}

/// Player names that are empty or whitespace-only render as `_` so links stay usable.
pub fn display_player_name(name: &str) -> String {
	if name.trim().is_empty() {
		"_".to_string()
	} else {
		name.to_string()
	}
}

pub fn display_player_name_opt(name: Option<&str>, player_id: i64) -> String {
	match name {
		Some(n) if !n.trim().is_empty() => n.to_string(),
		Some(_) => "_".to_string(),
		None => format!("#{}", player_id),
	}
}

// Grade names in ascending order, matching IIDX_LIKE_GBOUNDARIES from Tachi.
const BMS_GRADES: [&str; 9] = ["F", "E", "D", "C", "B", "A", "AA", "AAA", "MAX"];

fn bms_grade_boundaries(score_max: i64) -> [i64; 9] {
	[
		0,
		score_max * 2 / 9,
		score_max * 3 / 9,
		score_max * 4 / 9,
		score_max * 5 / 9,
		score_max * 6 / 9,
		score_max * 7 / 9,
		score_max * 8 / 9,
		score_max,
	]
}

pub fn bms_grade_delta(score: i64, score_max: i64) -> String {
	if score_max == 0 {
		return String::new();
	}
	let bounds = bms_grade_boundaries(score_max);

	// Find the highest grade whose lower bound is <= score.
	let grade_idx = bounds.iter().rposition(|&b| score >= b).unwrap_or(0);

	let lower_delta = score - bounds[grade_idx];

	let wrap = |g: &str| -> String {
		if g.ends_with('-') {
			format!("({})", g)
		} else {
			g.to_string()
		}
	};

	if grade_idx + 1 < BMS_GRADES.len() {
		let upper_delta = bounds[grade_idx + 1] - score; // always positive
		let lower_str = format!("{}+{}", wrap(BMS_GRADES[grade_idx]), lower_delta);
		let upper_str = format!("{}-{}", wrap(BMS_GRADES[grade_idx + 1]), upper_delta);
		// Show whichever boundary the score is closer to.
		if upper_delta <= lower_delta {
			upper_str
		} else {
			lower_str
		}
	} else {
		// Score is at MAX.
		format!("{}+{}", wrap(BMS_GRADES[grade_idx]), lower_delta)
	}
}

pub fn clear_pct(cleared: i64, total: i64) -> String {
	if total == 0 {
		"–".to_string()
	} else {
		format!("{:.2}%", cleared as f64 * 100.0 / total as f64)
	}
}

/// CSS class suffix for `.rank-clear-{suffix}` (see `_base.html.jinja`).
pub fn clear_css_class(clear_type: &str) -> String {
	let s = clear_type.trim_start_matches('★').trim();
	if s.eq_ignore_ascii_case("FULL COMBO") || s.eq_ignore_ascii_case("FULLCOMBO") {
		return "FULLCOMBO".to_string();
	}
	if s.eq_ignore_ascii_case("HARD CLEAR") || s == "HARD" {
		return "HARD".to_string();
	}
	if s.eq_ignore_ascii_case("NORMAL CLEAR") || s == "NORMAL" || s.eq_ignore_ascii_case("CLEAR") {
		return "NORMAL".to_string();
	}
	if s.eq_ignore_ascii_case("EASY CLEAR") || s == "EASY" {
		return "EASY".to_string();
	}
	if s.eq_ignore_ascii_case("FAILED") {
		return "FAILED".to_string();
	}
	s.replace(' ', "").to_uppercase()
}

/// Increment one of the five clear-type buckets from a LR2IR clear-type string.
pub fn add_clear_to_breakdown(
	fc: &mut i64,
	hard: &mut i64,
	normal: &mut i64,
	easy: &mut i64,
	failed: &mut i64,
	clear_type: &str,
) {
	match clear_css_class(clear_type).as_str() {
		"FULLCOMBO" => *fc += 1,
		"HARD" => *hard += 1,
		"NORMAL" => *normal += 1,
		"EASY" => *easy += 1,
		"FAILED" => *failed += 1,
		_ => {}
	}
}

/// Returns true when `s` looks like a 32-char lowercase/uppercase hex MD5.
pub fn is_md5(s: &str) -> bool {
	s.len() == 32 && s.chars().all(|c| c.is_ascii_hexdigit())
}

/// Build a safe FTS5 MATCH expression from a user-supplied search string.
/// Each whitespace-separated token becomes a quoted prefix term ("tok"*)
/// so that "Air GOD" matches "Air -GOD-".
/// Returns None when the query is blank (caller skips FTS entirely).
pub fn fts_query(q: &str) -> Option<String> {
	let terms: Vec<String> = q
		.split_whitespace()
		.map(|w| format!("\"{}\"*", w.replace('"', "\"\"")))
		.collect();
	if terms.is_empty() {
		None
	} else {
		Some(terms.join(" "))
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_clear_css_class() {
		assert_eq!(clear_css_class("FULL COMBO"), "FULLCOMBO");
		assert_eq!(clear_css_class("★FULLCOMBO"), "FULLCOMBO");
		assert_eq!(clear_css_class("HARD CLEAR"), "HARD");
		assert_eq!(clear_css_class("EASY CLEAR"), "EASY");
	}

	#[test]
	fn test_fts_query_empty() {
		assert!(fts_query("").is_none());
		assert!(fts_query("   ").is_none());
	}

	#[test]
	fn test_fts_query_single() {
		assert_eq!(fts_query("air").unwrap(), r#""air"*"#);
	}

	#[test]
	fn test_fts_query_multi() {
		assert_eq!(fts_query("air god").unwrap(), r#""air"* "god"*"#);
	}

	#[test]
	fn test_fts_query_escapes_quotes() {
		assert_eq!(fts_query(r#"say "hi""#).unwrap(), r#""say"* """hi"""*"#);
	}

	#[test]
	fn test_add_clear_to_breakdown() {
		let mut fc = 0;
		let mut hard = 0;
		let mut normal = 0;
		let mut easy = 0;
		let mut failed = 0;
		add_clear_to_breakdown(
			&mut fc,
			&mut hard,
			&mut normal,
			&mut easy,
			&mut failed,
			"FULL COMBO",
		);
		add_clear_to_breakdown(
			&mut fc,
			&mut hard,
			&mut normal,
			&mut easy,
			&mut failed,
			"HARD CLEAR",
		);
		add_clear_to_breakdown(
			&mut fc,
			&mut hard,
			&mut normal,
			&mut easy,
			&mut failed,
			"FAILED",
		);
		assert_eq!((fc, hard, normal, easy, failed), (1, 1, 0, 0, 1));
	}

	#[test]
	fn test_display_player_name() {
		assert_eq!(display_player_name("Alice"), "Alice");
		assert_eq!(display_player_name(""), "_");
		assert_eq!(display_player_name("   "), "_");
		assert_eq!(display_player_name_opt(Some("Bob"), 1), "Bob");
		assert_eq!(display_player_name_opt(Some("  "), 42), "_");
		assert_eq!(display_player_name_opt(None, 42), "#42");
	}

	#[test]
	fn test_bms_grade_delta() {
		// score_max = 900 (450 notes * 2) — convenient round numbers
		let max = 900i64;
		// Boundaries: F=0, E=200, D=300, C=400, B=500, A=600, AA=700, AAA=800, MAX-=850, MAX=900

		// Exactly at AAA boundary → closer to AAA (delta 0 up), show "AAA+0"
		assert_eq!(bms_grade_delta(800, max), "AAA+0");

		// 1 below AAA → closer to AAA (1 away up vs 49 above AA) → "(MAX-)-50" no wait:
		// score=799: lower=AA, delta_lower=99, upper=AAA, delta_upper=1 → upper wins
		assert_eq!(bms_grade_delta(799, max), "AAA-1");

		// Exactly at MAX → "MAX+0"
		assert_eq!(bms_grade_delta(900, max), "MAX+0");

		// score=850: equidistant from AAA (800) and MAX (900) → show "MAX-50"
		assert_eq!(bms_grade_delta(850, max), "MAX-50");

		// score=0 → F+0
		assert_eq!(bms_grade_delta(0, max), "F+0");

		// score_max=0 → empty
		assert_eq!(bms_grade_delta(0, 0), "");
	}
}
