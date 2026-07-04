mod web;
mod worker;

use anyhow::Result;
use std::env;
use tokio::runtime::Builder;
use tracing::{Instrument as _, error, info_span};

fn main() -> Result<()> {
	if let Err(provider) = rustls::crypto::aws_lc_rs::default_provider().install_default() {
		error!(?provider, "failed to install the crypto provider");
		anyhow::bail!("failed to install the crypto provider");
	}

	let mode = env::args().nth(1);
	Builder::new_multi_thread().enable_io().enable_time().build()?.block_on(async {
		match mode.as_deref() {
			None => {
				let telemetry = typscord_telemetry::init("typscord-web")?;
				let result = web::main().await;
				telemetry.shutdown()?;
				result?;
			}
			Some("worker") => {
				let telemetry = typscord_telemetry::init("typscord-worker")?;
				let result = {
					let span = info_span!("main");
					typscord_telemetry::set_parent_from_env(&span)?;
					worker::render().instrument(span).await
				};
				telemetry.shutdown()?;
				result?;
			}
			Some(mode) => {
				error!(mode, "unknown mode");
				anyhow::bail!("unknown arguments");
			}
		}

		Ok(())
	})
}
