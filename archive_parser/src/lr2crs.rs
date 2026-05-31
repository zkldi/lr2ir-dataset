use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result};
use quick_xml::events::Event;
use quick_xml::Reader;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lr2crsEntry {
	pub title: String,
	pub keys: String,
	pub hash: String,
	pub course_type: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CourseMatchInput {
	pub course_id: i64,
	pub title: String,
	pub keys: String,
	pub stage_md5s: Vec<String>,
}

/// Read and parse `{pages_dir}/all-courses/course.lr2crs` (Shift_JIS XML).
pub fn read_lr2crs_file(path: &Path) -> Result<Vec<Lr2crsEntry>> {
	let bytes = std::fs::read(path).with_context(|| format!("open lr2crs: {path:?}"))?;
	let (text, _, _) = encoding_rs::SHIFT_JIS.decode(&bytes);
	parse_lr2crs(&text)
}

pub fn parse_lr2crs(text: &str) -> Result<Vec<Lr2crsEntry>> {
	let mut reader = Reader::from_str(text);
	reader.config_mut().trim_text(true);

	let mut entries = Vec::new();
	let mut buf = Vec::new();

	let mut current: Option<CourseBuilder> = None;
	let mut field: Option<&str> = None;

	loop {
		match reader.read_event_into(&mut buf) {
			Ok(Event::Start(e)) => match e.name().as_ref() {
				b"course" => {
					current = Some(CourseBuilder::default());
					field = None;
				}
				b"title" if current.is_some() => field = Some("title"),
				b"line" if current.is_some() => field = Some("line"),
				b"hash" if current.is_some() => field = Some("hash"),
				b"type" if current.is_some() => field = Some("type"),
				_ => {}
			},
			Ok(Event::Text(e)) => {
				if let Some(course) = current.as_mut() {
					let value = e.unescape().context("unescape lr2crs text")?.into_owned();
					match field {
						Some("title") => course.title = value,
						Some("line") => course.line = value,
						Some("hash") => course.hash = value,
						Some("type") => course.course_type = value,
						_ => {}
					}
				}
			}
			Ok(Event::End(e)) => {
				if e.name().as_ref() == b"course" {
					if let Some(course) = current.take() {
						entries.push(course.into_entry());
					}
					field = None;
				} else if matches!(e.name().as_ref(), b"title" | b"line" | b"hash" | b"type") {
					field = None;
				}
			}
			Ok(Event::Eof) => break,
			Ok(_) => {}
			Err(e) => return Err(e.into()),
		}
		buf.clear();
	}

	Ok(entries)
}

fn line_to_keys(line: &str) -> String {
	if line.is_empty() || line.ends_with("KEYS") {
		line.to_string()
	} else {
		format!("{line}KEYS")
	}
}

#[derive(Default)]
struct CourseBuilder {
	title: String,
	line: String,
	hash: String,
	course_type: String,
}

impl CourseBuilder {
	fn into_entry(self) -> Lr2crsEntry {
		Lr2crsEntry {
			title: self.title,
			keys: line_to_keys(&self.line),
			hash: self.hash,
			course_type: self.course_type.parse().unwrap_or(0),
		}
	}
}

pub fn hash_len_for_stage_count(stage_count: usize) -> usize {
	match stage_count {
		1 => 64,
		2 => 96,
		3 => 128,
		4 => 160,
		5 => 192,
		6 => 200,
		_ => 0,
	}
}

const HASH_PREFIX_LEN: usize = 32;

/// Index for fast lr2crs lookup during course hash import.
pub struct MatchIndex {
	by_stage_body: HashMap<String, Vec<usize>>,
	by_title_keys: HashMap<(String, String), Vec<usize>>,
}

impl MatchIndex {
	pub fn new(entries: &[Lr2crsEntry]) -> Self {
		let mut by_stage_body: HashMap<String, Vec<usize>> = HashMap::new();
		let mut by_title_keys: HashMap<(String, String), Vec<usize>> = HashMap::new();
		for (i, e) in entries.iter().enumerate() {
			if e.hash.len() > HASH_PREFIX_LEN {
				by_stage_body
					.entry(e.hash[HASH_PREFIX_LEN..].to_string())
					.or_default()
					.push(i);
			}
			by_title_keys
				.entry((e.title.clone(), e.keys.clone()))
				.or_default()
				.push(i);
		}
		Self {
			by_stage_body,
			by_title_keys,
		}
	}
}

fn resolve_hits<'a>(
	course: &CourseMatchInput,
	entries: &'a [Lr2crsEntry],
	mut hit_idxs: Vec<usize>,
) -> Option<&'a Lr2crsEntry> {
	if hit_idxs.is_empty() {
		return None;
	}
	if hit_idxs.len() == 1 {
		return Some(&entries[hit_idxs[0]]);
	}

	hit_idxs.retain(|&i| entries[i].title == course.title && entries[i].keys == course.keys);
	if hit_idxs.len() == 1 {
		return Some(&entries[hit_idxs[0]]);
	}

	let exp_len = hash_len_for_stage_count(course.stage_md5s.len());
	if exp_len > 0 {
		hit_idxs.retain(|&i| entries[i].hash.len() == exp_len);
		if hit_idxs.len() == 1 {
			return Some(&entries[hit_idxs[0]]);
		}
	}
	None
}

/// Match a DB course to an lr2crs entry using stage MD5 concat, then disambiguation.
pub fn match_entry_with_index<'a>(
	course: &CourseMatchInput,
	entries: &'a [Lr2crsEntry],
	index: &MatchIndex,
) -> Option<&'a Lr2crsEntry> {
	let concat: String = course.stage_md5s.concat();
	if !concat.is_empty() {
		let mut hit_idxs = index
			.by_stage_body
			.get(&concat)
			.cloned()
			.unwrap_or_default();
		if hit_idxs.is_empty() {
			hit_idxs = entries
				.iter()
				.enumerate()
				.filter(|(_, e)| e.hash.contains(&concat))
				.map(|(i, _)| i)
				.collect();
		}
		if let Some(entry) = resolve_hits(course, entries, hit_idxs) {
			return Some(entry);
		}
	}

	let hit_idxs = index
		.by_title_keys
		.get(&(course.title.clone(), course.keys.clone()))?
		.clone();
	resolve_hits(course, entries, hit_idxs)
}

/// Match using a freshly built index (for small test fixtures).
pub fn match_entry<'a>(
	course: &CourseMatchInput,
	entries: &'a [Lr2crsEntry],
) -> Option<&'a Lr2crsEntry> {
	match_entry_with_index(course, entries, &MatchIndex::new(entries))
}

/// Group `(course_id, md5)` rows into ordered stage MD5 lists.
pub fn group_stage_md5s(rows: &[(i64, Option<String>)]) -> HashMap<i64, Vec<String>> {
	let mut map: HashMap<i64, Vec<String>> = HashMap::new();
	for (course_id, md5) in rows {
		if let Some(md5) = md5 {
			map.entry(*course_id).or_default().push(md5.clone());
		}
	}
	map
}

#[cfg(test)]
mod tests {
	use super::*;

	const FIXTURE: &str = r#"<?xml version="1.0" encoding="shift_jis"?>
<courselist>
	<course>
		<title>OverJoy</title>
		<line>7</line>
		<hash>00000000002000000000000000005190fb7a747a5115a4a4739397d17ccb26bbec6d68fd56fbe302edf0ac8923494c24284712605e020dfbd6b8be94afc053fdc46a81cb184f5a804c119930d6eba748</hash>
		<type>2</type>
	</course>
	<course>
		<title>test course</title>
		<line>7</line>
		<hash>abc123</hash>
		<type>0</type>
	</course>
</courselist>"#;

	#[test]
	fn parses_lr2crs_entries() {
		let entries = parse_lr2crs(FIXTURE).expect("parse fixture");
		assert_eq!(entries.len(), 2);
		assert_eq!(entries[0].title, "OverJoy");
		assert_eq!(entries[0].keys, "7KEYS");
		assert_eq!(entries[0].course_type, 2);
		assert!(entries[0].hash.contains("fb7a747a5115a4a4739397d17ccb26bb"));
	}

	#[test]
	fn matches_by_stage_md5_concat() {
		let entries = parse_lr2crs(FIXTURE).expect("parse fixture");
		let course = CourseMatchInput {
			course_id: 1,
			title: "OverJoy".into(),
			keys: "7KEYS".into(),
			stage_md5s: vec![
				"fb7a747a5115a4a4739397d17ccb26bb".into(),
				"ec6d68fd56fbe302edf0ac8923494c24".into(),
				"284712605e020dfbd6b8be94afc053fd".into(),
				"c46a81cb184f5a804c119930d6eba748".into(),
			],
		};
		let matched = match_entry(&course, &entries).expect("match OverJoy");
		assert_eq!(matched.hash, entries[0].hash);
		assert_eq!(matched.course_type, 2);
	}

	#[test]
	fn falls_back_to_unique_title_keys() {
		let entries = parse_lr2crs(FIXTURE).expect("parse fixture");
		let course = CourseMatchInput {
			course_id: 99,
			title: "test course".into(),
			keys: "7KEYS".into(),
			stage_md5s: vec![],
		};
		let matched = match_entry(&course, &entries).expect("match by title");
		assert_eq!(matched.hash, "abc123");
	}
}
