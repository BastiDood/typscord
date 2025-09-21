use anyhow::{Context as _, Result};
use discordyst_worker::Worker;
use std::env;

pub fn main(application_id: &str, interaction_token: &str) -> Result<()> {
	let application_id = application_id.parse().context("application_id must be a valid u64")?;
	let Some(application_id) = discordyst_worker::try_cast_application_id(application_id) else {
		anyhow::bail!("application_id must be a valid id");
	};

	let discord_bot_token =
		env::var("DISCORD_BOT_TOKEN").context("DISCORD_BOT_TOKEN must be set")?;
	let worker = Worker::new(discord_bot_token, application_id, interaction_token);
	todo!()
}
