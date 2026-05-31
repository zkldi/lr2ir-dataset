use anyhow::{bail, Context, Result};
use clap::Parser;
use regex::Regex;
use serde_json::Value;
use sqlx::SqlitePool;
use tracing::{error, info, warn};
use url::Url;

#[derive(Parser)]
struct Args {
	/// Path to the output SQLite database
	#[arg(long, default_value = "data/tableinfo.db")]
	output: String,
}

struct TableDef {
	name: &'static str,
	symbol: &'static str,
	url: &'static str,
}

static TABLES: &[TableDef] = &[
	TableDef {
		name: "Insane",
		symbol: "★",
		url: "https://darksabun.club/table/archive/insane1/",
	},
	TableDef {
		name: "Normal",
		symbol: "☆",
		url: "https://darksabun.club/table/archive/normal1/",
	},
	TableDef {
		name: "Stella",
		symbol: "st",
		url: "https://stellabms.xyz/st/table.html",
	},
	TableDef {
		name: "Satellite",
		symbol: "sl",
		url: "https://stellabms.xyz/sl/table.html",
	},
	TableDef {
		name: "Insane2",
		symbol: "▼",
		url: "https://rattoto10.github.io/second_table/insane_header.json",
	},
	TableDef {
		name: "Normal2",
		symbol: "▽",
		url: "https://rattoto10.github.io/second_table/header.json",
	},
	TableDef {
		name: "Overjoy",
		symbol: "★★",
		url: "http://rattoto10.jounin.jp/table_overjoy.html",
	},
	TableDef {
		name: "DP Insane",
		symbol: "★",
		url: "http://dpbmsdelta.web.fc2.com/table/insane.html",
	},
	TableDef {
		name: "DP Normal",
		symbol: "δ",
		url: "http://dpbmsdelta.web.fc2.com/table/dpdelta.html",
	},
	TableDef {
		name: "DP Satellite",
		symbol: "sl",
		url: "https://stellabms.xyz/dp/table.html",
	},
	TableDef {
		name: "Scratch 3rd",
		symbol: "h◎",
		url: "http://minddnim.web.fc2.com/sara/3rd_hard/bms_sara_3rd_hard.html",
	},
	TableDef {
		name: "LN",
		symbol: "◆",
		url: "http://flowermaster.web.fc2.com/lrnanido/gla/LN.html",
	},
	TableDef {
		name: "Stardust",
		symbol: "ξ",
		url: "https://mqppppp.neocities.org/StardustTable.html",
	},
	TableDef {
		name: "Starlight",
		symbol: "sr",
		url: "https://djkuroakari.github.io/starlighttable.html",
	},
	TableDef {
		name: "LN Overjoy",
		symbol: "◆◆",
		url: "https://notepara.com/glassist/lnoj",
	},
	TableDef {
		name: "Luminous",
		symbol: "ln",
		url: "https://ladymade-star.github.io/luminous/",
	},
	TableDef {
		name: "Gachimjoy",
		symbol: "双",
		url: "http://su565fx.web.fc2.com/Gachimijoy/gachimijoy.html",
	},
	TableDef {
		name: "delayjoy",
		symbol: "dl",
		url: "https://boku.tachi.ac/api/v1/games/bms-7k/custom-tables/delayjoy-fixed",
	},
	TableDef {
		name: "Arm Shougakkou",
		symbol: "Ude",
		url: "https://boku.tachi.ac/api/v1/games/bms-7k/custom-tables/arm-shougakkou-fixed",
	},
	TableDef {
		name: "Exoplanet",
		symbol: "fr",
		url: "https://stellabms.xyz/fr/table.html",
	},
	TableDef {
		name: "DP Library",
		symbol: "☆",
		url: "https://yaruki0.net/DPlibrary/",
	},
	TableDef {
		name: "Dystopia",
		symbol: "dy",
		url: "https://monibms.github.io/Dystopia/dystopia.html",
	},
	TableDef {
		name: "Scramble",
		symbol: "SB",
		url: "https://egret9.github.io/Scramble/",
	},
	TableDef {
		name: "Supernova",
		symbol: "sn",
		url: "https://stellabms.xyz/sn/table.html",
	},
	TableDef {
		name: "Solar",
		symbol: "so",
		url: "https://stellabms.xyz/so/table.html",
	},
	TableDef {
		name: "Code Stream (st)",
		symbol: "重発狂",
		url: "https://air-afother.github.io/chordstream-table-split/chordstreamST.html",
	},
	TableDef {
		name: "Code Stream (sl)",
		symbol: "乱打",
		url: "https://air-afother.github.io/chordstream-table-split/chordstream.html",
	},
];

/// Fetches text from a URL. Returns (final_url, body_text).
async fn fetch_text(client: &reqwest::Client, url: &str) -> Result<(Url, String)> {
	let resp = client
		.get(url)
		.send()
		.await
		.with_context(|| format!("GET {url}"))?
		.error_for_status()
		.with_context(|| format!("HTTP error for {url}"))?;
	let final_url = resp.url().clone();
	let text = resp
		.text()
		.await
		.with_context(|| format!("reading body of {url}"))?;
	Ok((final_url, text))
}

/// Returns true if the text looks like a JSON object (the header format).
fn looks_like_json(text: &str) -> bool {
	text.trim_start().starts_with('{') || text.trim_start().starts_with('[')
}

/// Extracts the bmstable header URL from an HTML page using a regex.
///
/// Matches `<meta name="bmstable" content="...">` in either attribute order,
/// with or without quotes around attribute values.
fn extract_bmstable_url(html: &str, page_url: &Url) -> Result<Url> {
	// Two patterns to handle either attribute order:
	//   <meta name="bmstable" content="header.json">
	//   <meta content="header.json" name="bmstable">
	let patterns = [
		r#"(?i)<meta[^>]+name=["']?bmstable["']?[^>]+content=["']([^"'\s>]+)["']?"#,
		r#"(?i)<meta[^>]+content=["']([^"'\s>]+)["']?[^>]+name=["']?bmstable["']?"#,
	];

	for pattern in &patterns {
		let re = Regex::new(pattern).unwrap();
		if let Some(cap) = re.captures(html) {
			let content = cap.get(1).unwrap().as_str();
			return Ok(page_url.join(content)?);
		}
	}

	let preview: String = html.chars().take(500).collect();
	Err(anyhow::anyhow!(
		"no <meta name=\"bmstable\"> found in HTML from {page_url}\nResponse preview:\n{preview}"
	))
}

struct FetchedTable {
	symbol: String,
	data_url: Url,
}

/// Resolves a table URL to its header JSON, returning the parsed symbol and data_url.
async fn resolve_header(client: &reqwest::Client, start_url: &str) -> Result<FetchedTable> {
	let (final_url, text) = fetch_text(client, start_url).await?;

	let header_json: Value = if looks_like_json(&text) {
		serde_json::from_str(&text).context("parsing header JSON")?
	} else {
		// HTML page — find the bmstable meta tag pointing to the header JSON
		let header_url = extract_bmstable_url(&text, &final_url)?;
		let (_, header_text) = fetch_text(client, header_url.as_str()).await?;
		serde_json::from_str(&header_text).context("parsing header JSON from meta URL")?
	};

	let symbol = header_json
		.get("symbol")
		.and_then(Value::as_str)
		.context("header JSON missing 'symbol'")?
		.to_owned();

	let data_url_str = header_json
		.get("data_url")
		.and_then(Value::as_str)
		.context("header JSON missing 'data_url'")?;

	// data_url is relative to the final resolved URL
	let resolved_final = if looks_like_json(&text) {
		// If the initial response was already JSON, its URL is the header URL
		final_url.clone()
	} else {
		// header was fetched from the meta URL; reconstruct
		extract_bmstable_url(&text, &final_url)?
	};

	let data_url = resolved_final.join(data_url_str)?;

	Ok(FetchedTable { symbol, data_url })
}

/// Fetches chart entries from a data URL. Returns Vec of (md5, level).
async fn fetch_charts(client: &reqwest::Client, data_url: &Url) -> Result<Vec<(String, String)>> {
	let (_, text) = fetch_text(client, data_url.as_str()).await?;
	let arr: Vec<Value> = serde_json::from_str(&text).context("parsing charts JSON")?;

	let mut out = Vec::new();
	for entry in arr {
		let md5 = match entry.get("md5").and_then(Value::as_str) {
			Some(s) if !s.is_empty() && s != "null" => s.to_owned(),
			_ => continue,
		};
		let level = entry
			.get("level")
			.map(|v| match v {
				Value::String(s) => s.clone(),
				other => other.to_string(),
			})
			.unwrap_or_default();
		out.push((md5, level));
	}
	Ok(out)
}

#[tokio::main]
async fn main() -> Result<()> {
	tracing_subscriber::fmt()
		.with_env_filter(
			tracing_subscriber::EnvFilter::try_from_default_env()
				.unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
		)
		.init();

	let args = Args::parse();

	let db_url = format!("sqlite://{}?mode=rwc", args.output);
	let pool = SqlitePool::connect(&db_url)
		.await
		.with_context(|| format!("opening SQLite at {}", args.output))?;

	sqlx::query("DROP TABLE IF EXISTS table_level")
		.execute(&pool)
		.await?;
	sqlx::query("DROP TABLE IF EXISTS table_main")
		.execute(&pool)
		.await?;
	sqlx::query("DROP TABLE IF EXISTS meta")
		.execute(&pool)
		.await?;

	sqlx::query(
		"CREATE TABLE table_main (
			table_id INTEGER PRIMARY KEY,
			name	 TEXT NOT NULL,
			symbol   TEXT NOT NULL
		)",
	)
	.execute(&pool)
	.await?;

	sqlx::query(
		"CREATE TABLE table_level (
			rowid	INTEGER PRIMARY KEY,
			table_id INTEGER NOT NULL,
			md5	  TEXT NOT NULL,
			level	TEXT NOT NULL,
			symbol   TEXT NOT NULL
		)",
	)
	.execute(&pool)
	.await?;

	sqlx::query("CREATE TABLE meta (rendered_at TEXT NOT NULL)")
		.execute(&pool)
		.await?;

	let client = reqwest::Client::builder()
		.user_agent("lr2ir_archive_tablegen/0.1")
		.redirect(reqwest::redirect::Policy::limited(20))
		.build()?;

	let mut success = 0usize;
	let mut failures = 0usize;

	for table in TABLES {
		info!("Fetching table: {} ({})", table.name, table.url);

		let header = match resolve_header(&client, table.url).await {
			Ok(h) => h,
			Err(e) => {
				error!("Failed to load header for {}: {:#}", table.name, e);
				failures += 1;
				continue;
			}
		};

		if header.symbol != table.symbol {
			warn!(
				"Symbol mismatch for {}: expected {:?}, got {:?}",
				table.name, table.symbol, header.symbol
			);
		}

		let charts = match fetch_charts(&client, &header.data_url).await {
			Ok(c) => c,
			Err(e) => {
				error!("Failed to fetch charts for {}: {:#}", table.name, e);
				failures += 1;
				continue;
			}
		};

		let result = sqlx::query("INSERT INTO table_main (name, symbol) VALUES (?, ?)")
			.bind(table.name)
			.bind(table.symbol)
			.execute(&pool)
			.await?;

		let table_id = result.last_insert_rowid();

		for (md5, level) in &charts {
			sqlx::query(
				"INSERT INTO table_level (table_id, md5, level, symbol) VALUES (?, ?, ?, ?)",
			)
			.bind(table_id)
			.bind(md5)
			.bind(level)
			.bind(table.symbol)
			.execute(&pool)
			.await?;
		}

		info!(
			"  {} → {} charts inserted (symbol: {})",
			table.name,
			charts.len(),
			table.symbol
		);
		success += 1;
	}

	sqlx::query("INSERT INTO meta (rendered_at) VALUES (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))")
		.execute(&pool)
		.await?;

	info!("Done. {} table(s) succeeded, {} failed.", success, failures);

	if failures > 0 {
		bail!("{} table(s) failed to load", failures);
	}

	Ok(())
}
