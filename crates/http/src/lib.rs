#![no_std]

extern crate alloc;

use alloc::{boxed::Box, string::String};
use tracing::{info, instrument};
use twilight_http::{Client, client::InteractionClient};
use twilight_model::{
	channel::message::Embed,
	channel::message::MessageFlags,
	http::attachment::Attachment,
	id::{Id, marker::ApplicationMarker},
};

pub type ApplicationId = Id<ApplicationMarker>;

pub struct Http {
	http: Client,
}

type TwilightHttpError<T> = Result<T, twilight_http::Error>;

impl Http {
	pub fn new(bot_token: String) -> Self {
		Self { http: Client::new(bot_token) }
	}

	pub fn interaction<'token>(
		&'token self,
		application_id: ApplicationId,
		interaction_token: Box<str>,
	) -> HttpInteraction<'token> {
		HttpInteraction { http: self.http.interaction(application_id), interaction_token }
	}
}

pub struct HttpInteraction<'http> {
	http: InteractionClient<'http>,
	interaction_token: Box<str>,
}

impl HttpInteraction<'_> {
	#[instrument(skip(self), level = "trace")]
	pub async fn update_response_with_embeds(
		&self,
		content: &str,
		embeds: &[Embed],
	) -> TwilightHttpError<()> {
		let message = self
			.http
			.update_response(&self.interaction_token)
			.content(Some(content))
			.embeds(Some(embeds))
			.await?;
		info!(?message, "response updated with embeds");
		Ok(())
	}

	#[instrument(skip(self), level = "trace")]
	pub async fn replace_response_with_attachments(
		&self,
		attachments: &[Attachment],
	) -> TwilightHttpError<()> {
		let message = self
			.http
			.update_response(&self.interaction_token)
			.content(None)
			.embeds(None)
			.attachments(attachments)
			.await?;
		info!(?message, "response replaced with attachments");
		Ok(())
	}

	#[instrument(skip(self), level = "trace")]
	pub async fn create_ephemeral_followup_with_embeds(
		&self,
		content: &str,
		embeds: &[Embed],
	) -> TwilightHttpError<()> {
		let message = self
			.http
			.create_followup(&self.interaction_token)
			.content(content)
			.embeds(embeds)
			.flags(MessageFlags::EPHEMERAL)
			.await?;
		info!(?message, "ephemeral followup created");
		Ok(())
	}
}
