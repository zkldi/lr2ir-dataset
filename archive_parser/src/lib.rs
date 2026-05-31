pub mod cheaters;
pub mod db;
pub mod lr2crs;
pub mod parsers;
pub mod time;

pub use cheaters::{cheater_set, is_cheater_id, CHEATER_IDS};

pub use lr2crs::{
	match_entry, match_entry_with_index, parse_lr2crs, read_lr2crs_file, CourseMatchInput,
	Lr2crsEntry, MatchIndex,
};
pub use time::jst_naive_to_rfc3339;

use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use std::io::Read;
use std::path::Path;

const GZIP_MAGIC: [u8; 2] = [0x1f, 0x8b];

/// Read an HTML file that is either gzip-compressed or plain, decoding Shift_JIS → UTF-8.
pub fn read_html_gz(path: &Path) -> Result<String> {
	let bytes = std::fs::read(path).with_context(|| format!("open {path:?}"))?;
	let decoded = if bytes.starts_with(&GZIP_MAGIC) {
		let mut out = Vec::new();
		GzDecoder::new(bytes.as_slice())
			.read_to_end(&mut out)
			.with_context(|| format!("decompress {path:?}"))?;
		out
	} else {
		bytes
	};
	let (text, _, _) = encoding_rs::SHIFT_JIS.decode(&decoded);
	Ok(text.into_owned())
}
