use anyhow::Result;
use clap::Parser;
use tracing_subscriber::EnvFilter;

use lr2ir_archive_frontend::{app, cli};

#[tokio::main]
async fn main() -> Result<()> {
	dotenvy::dotenv().ok();

	tracing_subscriber::fmt()
		.with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
		.init();

	let args = cli::Args::parse();

	match args.command {
		cli::Cmd::Serve(a) => app::serve(a).await?,
	}

	Ok(())
}
