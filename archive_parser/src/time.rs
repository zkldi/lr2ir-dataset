use chrono::{FixedOffset, NaiveDateTime};

const NAIVE_FMT: &str = "%Y-%m-%d %H:%M:%S";

/// JST (+09:00), used for LR2IR timestamps scraped without an explicit timezone.
const JST: FixedOffset = match FixedOffset::east_opt(9 * 3600) {
	Some(offset) => offset,
	None => panic!("JST offset must be valid"),
};

/// Parse a naive LR2IR datetime string (assumed JST) into RFC3339/ISO8601.
pub fn jst_naive_to_rfc3339(input: &str) -> Option<String> {
	let naive = NaiveDateTime::parse_from_str(input.trim(), NAIVE_FMT).ok()?;
	let dt = naive.and_local_timezone(JST).single()?;
	Some(dt.to_rfc3339())
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn converts_naive_jst_to_rfc3339() {
		assert_eq!(
			jst_naive_to_rfc3339("2009-05-11 00:26:41").as_deref(),
			Some("2009-05-11T00:26:41+09:00")
		);
	}
}
