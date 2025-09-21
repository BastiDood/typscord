#![no_std]

extern crate alloc;

use alloc::string::String;
use twilight_http::Client;
use twilight_model::id::{Id, marker::ApplicationMarker};

pub use twilight_model::{channel::message::Embed, http::attachment::Attachment};

pub type ApplicationId = Id<ApplicationMarker>;

pub struct Http<'token> {
	http: Client,
	application_id: ApplicationId,
	interaction_token: &'token str,
}

type TwilightHttpError<T> = Result<T, twilight_http::Error>;

impl<'token> Http<'token> {
	pub fn new(
		bot_token: String,
		application_id: ApplicationId,
		interaction_token: &'token str,
	) -> Self {
		Self { http: Client::new(bot_token), application_id, interaction_token }
	}
}

impl Http<'_> {
	pub async fn update_response_with_embeds(
		&self,
		content: &str,
		embeds: &[Embed],
	) -> TwilightHttpError<()> {
		// TODO: Log the `Message` object.
		self.http
			.interaction(self.application_id)
			.update_response(self.interaction_token)
			.embeds(Some(embeds))
			.content(Some(content))
			.await?;
		Ok(())
	}

	pub async fn create_followup_with_attachments(
		&self,
		attachments: &[Attachment],
	) -> TwilightHttpError<()> {
		// TODO: Log the `Message` object.
		self.http
			.interaction(self.application_id)
			.create_followup(self.interaction_token)
			.attachments(attachments)
			.await?;
		Ok(())
	}
}
