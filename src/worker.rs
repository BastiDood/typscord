use anyhow::{Context as _, Result};
use core::str;
use discordyst_http::Http;
use std::{
	env,
	io::{self, Read},
};
use tracing::info;

pub fn main(application_id: &str, interaction_token: &str) -> Result<()> {
	let application_id = application_id.parse().context("application_id must be a valid u64")?;
	let Some(application_id) = discordyst_http::try_cast_application_id(application_id) else {
		anyhow::bail!("application_id must be a valid id");
	};

	let discord_bot_token =
		env::var("DISCORD_BOT_TOKEN").context("DISCORD_BOT_TOKEN must be set")?;
	let http = Http::new(discord_bot_token, application_id, interaction_token);

	let mut content = String::new();
	let mut stdin = io::stdin();

	let size = stdin.read_to_string(&mut content)?;
	info!(%size, "read content from stdin");

	todo!()
}
