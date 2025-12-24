#![no_std]

extern crate alloc;

use alloc::{boxed::Box, string::String, vec::Vec};
use tracing::{info, instrument};
use twilight_http::{Client, client::InteractionClient};
use twilight_model::{
	channel::message::Embed,
	channel::message::MessageFlags,
	channel::message::component::Component,
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

	#[instrument(skip_all, level = "trace")]
	pub async fn replace_response(
		&self,
		attachments: &[Attachment],
		components: &[Component],
	) -> TwilightHttpError<()> {
		let message = self
			.http
			.update_response(&self.interaction_token)
			.flags(MessageFlags::IS_COMPONENTS_V2)
			.content(None)
			.embeds(None)
			.attachments(attachments)
			.components(Some(components))
			.await?;
		info!(?message, "response replaced");
		Ok(())
	}

	#[instrument(skip_all, level = "trace")]
	pub async fn replace_response_with_preview(
		&self,
		attachments: &[Attachment],
		components: &[Component],
		embeds: &[Embed],
	) -> TwilightHttpError<()> {
		let message = self
			.http
			.update_response(&self.interaction_token)
			.flags(MessageFlags::IS_COMPONENTS_V2)
			.content(None)
			.embeds(Some(embeds))
			.attachments(attachments)
			.components(Some(components))
			.await?;
		info!(?message, "response replaced with preview");
		Ok(())
	}

	#[instrument(skip_all, level = "trace")]
	pub async fn delete_response(&self) -> TwilightHttpError<()> {
		self.http.delete_response(&self.interaction_token).await?;
		info!("response deleted");
		Ok(())
	}

	#[instrument(skip_all, level = "trace")]
	pub async fn create_public_followup(
		&self,
		components: Vec<Component>,
	) -> TwilightHttpError<()> {
		let message = self
			.http
			.create_followup(&self.interaction_token)
			.flags(MessageFlags::IS_COMPONENTS_V2)
			.components(&components)
			.await?;
		info!(?message, "public followup created");
		Ok(())
	}

	#[instrument(skip_all, level = "trace")]
	pub async fn create_ephemeral_followup_with_embeds(
		&self,
		content: &str,
		embeds: &[Embed],
	) -> TwilightHttpError<()> {
		let message = self
			.http
			.create_followup(&self.interaction_token)
			.flags(MessageFlags::EPHEMERAL)
			.content(content)
			.embeds(embeds)
			.await?;
		info!(?message, "ephemeral followup created");
		Ok(())
	}
}
