use anyhow::Result;
use axum::{
	Router,
	extract::{Request, State},
	http::{StatusCode, request::Parts},
	response::Json,
	routing, serve,
};
use bytes::BytesMut;
use core::future::ready;
use core::net::Ipv4Addr;
use discordyst_interaction::CreateInteractionResponse;
use ed25519_dalek::{Signature, VerifyingKey};
use futures_util::TryStreamExt as _;
use std::{env::var, sync::Arc};
use tokio::{net::TcpListener, runtime::Builder};
use tracing::error;

#[derive(Clone)]
struct AppState(Arc<VerifyingKey>);

async fn handle_discord_interaction(
	State(AppState(public_key)): State<AppState>,
	request: Request,
) -> Result<Json<CreateInteractionResponse>, StatusCode> {
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
			ready(Ok(message))
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

	Ok(Json(discordyst_interaction::handle(interaction)))
}

fn main() -> Result<()> {
	let port: u16 = var("PORT")?.parse()?;

	let discord_public_key = var("DISCORD_PUBLIC_KEY")?;
	let mut public_key = [0; 32];
	hex::decode_to_slice(discord_public_key, &mut public_key)?;
	let public_key = VerifyingKey::from_bytes(&public_key)?;

	let app = Router::new()
		.route("/discord/interaction", routing::post(handle_discord_interaction))
		.with_state(AppState(Arc::new(public_key)));

	let runtime = Builder::new_current_thread().enable_io().build()?;
	runtime.block_on(async {
		let listener = TcpListener::bind((Ipv4Addr::UNSPECIFIED, port)).await?;
		serve(listener, app).await?;
		Ok(())
	})
}
