use anyhow::{Context as _, Result};
use axum::{
	Router,
	extract::{Request, State},
	http::{StatusCode, request::Parts},
	response::Json,
	routing, serve,
};
use bytes::BytesMut;
use core::future;
use core::net::Ipv4Addr;
use ed25519_dalek::{Signature, VerifyingKey};
use futures_util::TryStreamExt as _;
use std::{env, sync::Arc};
use tokio::{net::TcpListener, runtime::Builder};
use tracing::{error, info};
use typscord_interaction::{InteractionHandler, InteractionResponse};

pub fn main() -> Result<()> {
	let port: u16 = env::var("PORT")
		.context("PORT must be set")?
		.parse()
		.context("PORT must be a valid port number")?;

	let discord_bot_token =
		env::var("DISCORD_BOT_TOKEN").context("DISCORD_BOT_TOKEN must be set")?;
	let discord_public_key =
		env::var("DISCORD_PUBLIC_KEY").context("DISCORD_PUBLIC_KEY must be set")?;

	let public_key = {
		let mut bytes = [0; 32];
		hex::decode_to_slice(discord_public_key, &mut bytes)
			.context("DISCORD_PUBLIC_KEY must be valid hex")?;
		VerifyingKey::from_bytes(&bytes)
			.context("DISCORD_PUBLIC_KEY must be valid point under ZIP-215 rules")?
	};

	let exe_path = env::current_exe()?.into_boxed_path();
	info!(exe = %exe_path.display(), "executable path found");

	let app = Router::new()
		.route("/discord/interaction", routing::post(handle_discord_interaction))
		.with_state(KeyState {
			public_key: Arc::new(public_key),
			interaction_handler: Arc::new(InteractionHandler::new(exe_path, discord_bot_token)),
		});

	let runtime = Builder::new_current_thread().enable_io().enable_time().build()?;
	runtime.block_on(async {
		let listener = TcpListener::bind((Ipv4Addr::UNSPECIFIED, port)).await?;
		{
			let address = listener.local_addr()?;
			info!(%address, "listening on local address");
		}
		serve(listener, app).await?;
		Ok(())
	})
}

#[derive(Clone)]
struct KeyState {
	public_key: Arc<VerifyingKey>,
	interaction_handler: Arc<InteractionHandler>,
}

async fn handle_discord_interaction(
	State(KeyState { public_key, interaction_handler }): State<KeyState>,
	request: Request,
) -> Result<Json<InteractionResponse>, StatusCode> {
	let (Parts { headers, .. }, body) = request.into_parts();
	let signature = headers.get("X-Signature-Ed25519");
	let timestamp = headers.get("X-Signature-Timestamp");
	let (signature, timestamp) = signature.zip(timestamp).ok_or(StatusCode::UNAUTHORIZED)?;

	let signature = hex::decode(signature).map_err(|error| {
		error!(?error);
		StatusCode::BAD_REQUEST
	})?;
	let signature = Signature::from_slice(&signature).map_err(|error| {
		error!(?error);
		StatusCode::BAD_REQUEST
	})?;

	let message = BytesMut::from(timestamp.as_bytes());
	let start = message.len();

	let message = body
		.into_data_stream()
		.try_fold(message, |mut message, chunk| {
			message.extend_from_slice(&chunk);
			future::ready(Ok(message))
		})
		.await
		.map_err(|error| {
			error!(?error);
			StatusCode::BAD_REQUEST
		})?;

	public_key.verify_strict(&message, &signature).map_err(|error| {
		error!(?error);
		StatusCode::UNAUTHORIZED
	})?;

	let json = message.get(start..).ok_or(StatusCode::BAD_REQUEST)?;
	let interaction = serde_json::from_slice(json).map_err(|error| {
		error!(?error);
		StatusCode::BAD_REQUEST
	})?;

	Ok(Json(interaction_handler.handle(interaction)))
}
