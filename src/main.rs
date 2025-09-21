mod web;
mod worker;

use anyhow::Result;
use std::{env, io};
use tracing::error;
use tracing_subscriber::{EnvFilter, fmt};

fn main() -> Result<()> {
	fmt().with_writer(io::stderr).with_env_filter(EnvFilter::from_default_env()).init();
	let mode = env::args().nth(1);
	match mode.as_deref() {
		Some("worker") => worker::main()?,
		None => web::main()?,
		Some(mode) => {
			error!(mode, "unknown mode");
			anyhow::bail!("unknown arguments");
		}
	}
	Ok(())
}
