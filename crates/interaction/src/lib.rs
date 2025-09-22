use discordyst_http::{ApplicationId, Http};
use std::{io, path::Path, process::Stdio, sync::Arc};
use tokio::{
	io::{
		AsyncBufRead, AsyncBufReadExt as _, AsyncRead, AsyncReadExt as _, AsyncWriteExt as _,
		BufReader,
	},
	process::Command,
};
use tracing::info;
use twilight_model::{
	application::{
		command::CommandType,
		interaction::{
			Interaction, InteractionData, InteractionType,
			application_command::CommandData,
			modal::{
				ModalInteractionData, ModalInteractionDataActionRow, ModalInteractionDataComponent,
			},
		},
	},
	channel::message::{
		Embed, MessageFlags,
		component::{ActionRow, Component, TextInput, TextInputStyle},
		embed::EmbedField,
	},
	http::{
		attachment::Attachment,
		interaction::{InteractionResponseData, InteractionResponseType},
	},
};

pub use twilight_model::http::interaction::InteractionResponse;

pub struct InteractionHandler {
	exe_path: Box<Path>,
	http: Http,
}

impl InteractionHandler {
	pub fn new(exe_path: Box<Path>, bot_token: String) -> Self {
		Self { exe_path, http: Http::new(bot_token) }
	}

	#[must_use]
	pub fn handle(self: Arc<Self>, interaction: Interaction) -> InteractionResponse {
		match interaction {
			Interaction { kind: InteractionType::Ping, .. } => {
				InteractionResponse { kind: InteractionResponseType::Pong, data: None }
			}
			Interaction {
				kind: InteractionType::ApplicationCommand,
				data: Some(InteractionData::ApplicationCommand(cmd)),
				..
			} => {
				let CommandData { kind, name, .. } = *cmd;
				assert_eq!(kind, CommandType::ChatInput);
				assert_eq!(name, "typst");
				InteractionResponse {
				kind: InteractionResponseType::Modal,
				data: Some(InteractionResponseData {
					custom_id: Some("modal".into()),
					title: Some("Render Typst Code".into()),
					components: Some(vec![
						Component::ActionRow(ActionRow {
							components: vec![
								Component::TextInput(TextInput {
									custom_id: "code".into(),
									label: "Typst Code".into(),
									style: TextInputStyle::Paragraph,
									max_length: Some(4000),
									placeholder: Some("Enter your Typst code here. Third-party packages are not yet supported.".into()),
									required: Some(true),
									value: None,
									min_length: None,
								}),
							],
						}),
					]),
					..Default::default()
				}),
			}
			}
			Interaction {
				kind: InteractionType::ModalSubmit,
				application_id,
				token,
				data:
					Some(InteractionData::ModalSubmit(ModalInteractionData {
						custom_id,
						mut components,
						..
					})),
				..
			} => {
				assert_eq!(custom_id, "modal");

				let action_row = components.pop().expect("modal must have at least one action row");
				assert!(components.is_empty(), "modal must have exactly one action row");

				let ModalInteractionDataActionRow { mut components } = action_row;
				let ModalInteractionDataComponent { custom_id, value, .. } =
					components.pop().expect("modal must have at least one component");
				assert!(components.is_empty(), "modal must have exactly one component");

				assert_eq!(custom_id, "code");

				// Ship the render request to a subprocess
				let value = value.expect("modal text input has required value").into_boxed_str();
				let token = token.into_boxed_str();
				tokio::spawn(self.subprocess(application_id, token, value));

				InteractionResponse {
					kind: InteractionResponseType::DeferredChannelMessageWithSource,
					data: Some(InteractionResponseData {
						content: Some("Rendering your Typst code...".into()),
						flags: Some(MessageFlags::EPHEMERAL),
						..Default::default()
					}),
				}
			}
			_ => unreachable!("unknown interaction"),
		}
	}

	async fn subprocess(
		self: Arc<Self>,
		application_id: ApplicationId,
		token: Box<str>,
		value: Box<str>,
	) {
		// TODO: Handle timeout
		let mut command = Command::new(self.exe_path.as_os_str())
			.arg("worker")
			.stdin(Stdio::piped())
			.stdout(Stdio::piped())
			.spawn()
			.expect("worker process must be spawned");

		command
			.stdin
			.take()
			.expect("stdin must have been piped")
			.write_all(value.as_bytes())
			.await
			.expect("stdin must be writable");

		let stdout = command.stdout.take().expect("stdout must have been piped");
		let mut stdout = BufReader::new(stdout);

		let warning_count = read_usize(&mut stdout).await.expect("stdout must read warning count");
		info!(warnings = warning_count, "read warning count");

		// Shared string buffer whose capacity can be reused by everyone
		let mut buffer = String::new();

		let mut warning_embed_fields = Vec::<EmbedField>::new();
		for _ in 0..warning_count {
			warning_embed_fields.push(EmbedField {
				name: read_line(&mut stdout, &mut buffer)
					.await
					.expect("stdout must read warning title"),
				value: read_line(&mut stdout, &mut buffer)
					.await
					.expect("stdout must read warning hint"),
				inline: false,
			});
		}

		let error_count = read_usize(&mut stdout).await.expect("stdout must read error count");
		info!(errors = error_count, "reading error count");

		let mut error_embed_fields = Vec::<EmbedField>::new();
		for _ in 0..error_count {
			error_embed_fields.push(EmbedField {
				name: read_line(&mut stdout, &mut buffer)
					.await
					.expect("stdout must read error title"),
				value: read_line(&mut stdout, &mut buffer)
					.await
					.expect("stdout must read error hint"),
				inline: false,
			});
		}

		// No need for the shared buffer after this point.
		drop(buffer);

		let mut file = Vec::new();
		stdout.read_to_end(&mut file).await.expect("stdout must be readable up to this point");

		// Should close the pipe after this point
		drop(stdout);

		let status = command.wait().await.expect("worker process must exit");
		info!(?status, "worker process exited");

		// Subprocess has since exited already
		drop(command);

		let http = self.http.interaction(application_id, token);

		http.update_response_with_embeds("Rendering complete.", &{
			let mut embeds = Vec::<Embed>::with_capacity(2);
			if !error_embed_fields.is_empty() {
				embeds.push(Embed {
					author: None,
					color: Some(0xf33f33),
					description: None,
					fields: error_embed_fields,
					footer: None,
					image: None,
					kind: "rich".into(),
					provider: None,
					thumbnail: None,
					timestamp: None,
					title: Some("Compilation Errors".into()),
					url: None,
					video: None,
				});
			}
			if !warning_embed_fields.is_empty() {
				embeds.push(Embed {
					author: None,
					color: Some(0xf7b955),
					description: None,
					fields: warning_embed_fields,
					footer: None,
					image: None,
					kind: "rich".into(),
					provider: None,
					thumbnail: None,
					timestamp: None,
					title: Some("Compilation Warnings".into()),
					url: None,
					video: None,
				});
			}
			embeds
		})
		.await
		.expect("original response must succeed");

		if !file.is_empty() {
			http.create_followup_with_attachments(&[Attachment {
				description: None,
				file,
				filename: "typst.webp".into(),
				id: 0,
			}])
			.await
			.expect("followup must succeed");
		}
	}
}

async fn read_usize<R: AsyncRead + Unpin>(reader: &mut R) -> io::Result<usize> {
	let mut bytes = 0usize.to_be_bytes();
	reader.read_exact(&mut bytes).await?;
	Ok(usize::from_be_bytes(bytes))
}

async fn read_line<R: AsyncBufRead + Unpin>(
	reader: &mut R,
	buffer: &mut String,
) -> io::Result<String> {
	reader.read_line(buffer).await?;
	let line = buffer.trim().to_owned();
	buffer.clear();
	Ok(line)
}
