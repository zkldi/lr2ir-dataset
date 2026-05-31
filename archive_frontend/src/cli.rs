use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
	name = "lr2ir_archive_frontend",
	about = "LR2IR read-only archive viewer"
)]
pub struct Args {
	#[command(subcommand)]
	pub command: Cmd,
}

#[derive(Subcommand, Debug)]
pub enum Cmd {
	/// Run the web server.
	Serve(ServeArgs),
}

#[derive(Parser, Debug)]
pub struct ServeArgs {
	/// Address to listen on.
	#[arg(long, env = "BIND", default_value = "0.0.0.0:3000")]
	pub bind: String,

	/// Path to the SQLite dataset file produced by the parser.
	#[arg(long, env = "DATABASE_PATH")]
	pub database: String,

	/// Optional path to the tableinfo.db produced by frontend_tablegen.
	#[arg(long, env = "TABLEINFO_PATH")]
	pub tableinfo: Option<String>,
}
