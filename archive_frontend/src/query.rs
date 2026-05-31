use serde::Deserialize;

#[derive(Deserialize)]
pub struct ListQuery {
	#[serde(default = "one")]
	pub page: u32,
	#[serde(default)]
	pub q: String,
}

#[derive(Deserialize, Debug)]
pub struct ChartQuery {
	#[serde(default = "one")]
	pub page: u32,
	/// Optional input-device filter, e.g. `BM` or `KB`.
	pub input: Option<String>,
}

fn one() -> u32 {
	1
}

pub const PER_PAGE: i64 = 100;

pub fn total_pages(total: i64) -> u32 {
	((total + PER_PAGE - 1) / PER_PAGE).max(1) as u32
}
