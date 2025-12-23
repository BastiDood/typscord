mod buffer;

use core::time::Duration;
use std::{io, path::Path, process::Stdio, sync::Arc, time::Instant};
use tokio::{
	io::{AsyncBufRead, AsyncReadExt as _, AsyncWriteExt as _, BufReader},
	process::{Child, Command},
};
use tracing::{error, info, instrument, trace};
use twilight_model::{
	application::{
		command::CommandType,
		interaction::{
			Interaction, InteractionData, InteractionType,
			application_command::CommandData,
			modal::{
				ModalInteractionComponent, ModalInteractionData, ModalInteractionLabel,
				ModalInteractionStringSelect, ModalInteractionTextInput,
			},
		},
	},
	channel::message::{
		Embed, EmojiReactionType, MessageFlags,
		component::{
			ActionRow, Button, ButtonStyle, Component, Label, SelectMenu, SelectMenuOption,
			SelectMenuType, TextInput, TextInputStyle,
		},
		embed::{EmbedAuthor, EmbedField, EmbedFooter},
	},
	http::{
		attachment::Attachment,
		interaction::{InteractionResponseData, InteractionResponseType},
	},
};
use typscord_http::{ApplicationId, Http};

pub use twilight_model::http::interaction::InteractionResponse;

pub struct InteractionHandler {
	compilation_timeout: Duration,
	exe_path: Box<Path>,
	http: Http,
}

impl InteractionHandler {
	pub fn new(compilation_timeout: Duration, exe_path: Box<Path>, bot_token: String) -> Self {
		Self { compilation_timeout, exe_path, http: Http::new(bot_token) }
	}

	#[must_use]
	#[instrument(skip_all)]
	pub fn handle(self: Arc<Self>, interaction: Interaction) -> InteractionResponse {
		match interaction {
			Interaction { id, kind: InteractionType::Ping, .. } => {
				info!(interaction_id = ?id, "received ping");
				InteractionResponse { kind: InteractionResponseType::Pong, data: None }
			}
			Interaction {
				id,
				user,
				member,
				guild_id,
				channel,
				kind: InteractionType::ApplicationCommand,
				data: Some(InteractionData::ApplicationCommand(cmd)),
				..
			} => {
				let user = member.and_then(|m| m.user).or(user).expect("user must be present");
				let channel_id = channel.map(|c| c.id);
				info!(interaction_id = ?id, user_id = ?user.id, ?guild_id, ?channel_id, "received application command");

				let CommandData { kind, name, .. } = *cmd;
				assert_eq!(kind, CommandType::ChatInput);

				// Just some constant strings that will be useful for commands later.
				const CODE_PLACEHOLDER: &str = "Hello, Typst!";

				match name.as_str() {
					"help" => InteractionResponse {
						kind: InteractionResponseType::ChannelMessageWithSource,
						data: Some(InteractionResponseData {
							flags: Some(MessageFlags::EPHEMERAL),
							components: Some(vec![Component::ActionRow(ActionRow {
								id: None,
								components: vec![
									Component::Button(Button {
										id: None,
										style: ButtonStyle::Link,
										emoji: Some(EmojiReactionType::Unicode {
											name: String::from('🐛'),
										}),
										label: Some(String::from("Report a Bug")),
										url: Some("https://github.com/BastiDood/typscord/issues/new".into()),
										custom_id: None,
										sku_id: None,
										disabled: false,
									}),
									Component::Button(Button {
										id: None,
										style: ButtonStyle::Link,
										emoji: Some(EmojiReactionType::Unicode {
											name: String::from('💻'),
										}),
										label: Some(String::from("Fork the Code")),
										url: Some("https://github.com/BastiDood/typscord/fork".into()),
										custom_id: None,
										sku_id: None,
										disabled: false,
									}),
								],
							})]),
							embeds: Some(vec![Embed {
								author: Some(EmbedAuthor {
									name: "Typscord".into(),
									url: Some("https://github.com/BastiDood/typscord".into()),
									icon_url: Some(
										"https://cdn.discordapp.com/avatars/1419611139448377366/ba3831b151e2c1868c0b7a8ad6d46146.png".into(),
									),
									proxy_icon_url: None,
								}),
								color: Some(0x7ad5d5),
								description: Some(
									"The `/typst` command is the main entry point to using Typscord. The command opens a modal that allows you to write Typst code in Discord. Upon submission, the Typst code will be rendered as an image in Discord. However, there are some limitations about the generated image.".into(),
								),
								fields: vec![
									EmbedField {
										name: "You may want to copy your Typst code before hitting submit.".into(),
										value: "Just in case the bot fails to respond, you can always paste your code back into the modal. It's not ideal, but it's the best we have for now.".into(),
										inline: false,
									},
									EmbedField {
										name: "Third-party packages are unsupported.".into(),
										value: "This is mostly for hosting and security reasons.".into(),
										inline: false,
									},
									EmbedField {
										name: "Image uploads are unsupported.".into(),
										value: "This requires that I allow users to add image attachments (which they can reference as relative paths in the Typst code), but I haven't gotten to that yet.".into(),
										inline: false,
									},
									EmbedField {
										name: "Only stock Typst fonts are supported.".into(),
										value: "Note that some emojis will fail to load.".into(),
										inline: false,
									},
									EmbedField {
										name: "The rendered image will be as wide as possible.".into(),
										value: "This unfortunately disables automatic line breaks. To opt into automatic line breaks, you have to wrap the code in a `#box` with the desired `width` yourself.".into(),
										inline: false,
									},
									EmbedField {
										name: "Compilations that take longer than a second will be timed out.".into(),
										value: "The bot is fast, but please don't abuse it with infinite loops, expensive compute, or anything like that. I'm hosting the service for free, and would greatly appreciate it if everyone played fair.".into(),
										inline: false,
									},
								],
								footer: Some(EmbedFooter {
									text: "By BastiDood".into(),
									icon_url: Some("https://avatars.githubusercontent.com/u/39114273".into()),
									proxy_icon_url: None,
								}),
								image: None,
								kind: "rich".into(),
								provider: None,
								thumbnail: None,
								timestamp: None,
								title: Some("How to Use Typscord".into()),
								url: None,
								video: None,
							}]),
							..Default::default()
						}),
					},
					"info" => InteractionResponse {
						kind: InteractionResponseType::ChannelMessageWithSource,
						data: Some(InteractionResponseData {
							allowed_mentions: None,
							components: Some(vec![Component::ActionRow(ActionRow {
								id: None,
								components: vec![
									Component::Button(Button {
										id: None,
										style: ButtonStyle::Link,
										emoji: Some(EmojiReactionType::Unicode {
											name: String::from('🤖'),
										}),
										label: Some(String::from("Install App")),
										url: Some("https://discord.com/oauth2/authorize?client_id=1419611139448377366".into()),
										custom_id: None,
										sku_id: None,
										disabled: false,
									}),
									Component::Button(Button {
										id: None,
										style: ButtonStyle::Link,
										emoji: Some(EmojiReactionType::Unicode {
											name: String::from('🐛'),
										}),
										label: Some(String::from("Report a Bug")),
										url: Some("https://github.com/BastiDood/typscord/issues/new".into()),
										custom_id: None,
										sku_id: None,
										disabled: false,
									}),
									Component::Button(Button {
										id: None,
										style: ButtonStyle::Link,
										emoji: Some(EmojiReactionType::Unicode {
											name: String::from('💻'),
										}),
										label: Some(String::from("Fork the Code")),
										url: Some("https://github.com/BastiDood/typscord/fork".into()),
										custom_id: None,
										sku_id: None,
										disabled: false,
									}),
								],
							})]),
							embeds: Some(vec![Embed {
								author: Some(EmbedAuthor {
									name: "Typscord".into(),
									url: Some("https://github.com/BastiDood/typscord".into()),
									icon_url: Some(
										"https://cdn.discordapp.com/avatars/1419611139448377366/ba3831b151e2c1868c0b7a8ad6d46146.png".into(),
									),
									proxy_icon_url: None,
								}),
								color: Some(0x7ad5d5),
								description: Some(
									"Typscord is a [free and open-source](https://github.com/BastiDood/typscord) Discord bot written in [Rust](https://www.rust-lang.org/) by [`@BastiDood`](https://github.com/BastiDood) for rendering [Typst](https://typst.app/) code.".into(),
								),
								fields: Vec::new(),
								footer: Some(EmbedFooter {
									text: "By BastiDood".into(),
									icon_url: Some("https://avatars.githubusercontent.com/u/39114273".into()),
									proxy_icon_url: None,
								}),
								image: None,
								kind: "rich".into(),
								provider: None,
								thumbnail: None,
								timestamp: None,
								title: Some("Typesetting for @everyone...".into()),
								url: None,
								video: None,
							}]),
							..Default::default()
						}),
					},
					"typst" => InteractionResponse {
						kind: InteractionResponseType::Modal,
						data: Some(InteractionResponseData {
							flags: Some(MessageFlags::IS_COMPONENTS_V2),
							custom_id: Some("typst".into()),
							title: Some("Render Typst Code".into()),
							components: Some(vec![
								Component::Label(Label {
									id: None,
									label: "Typst Code".into(),
									description: Some(
										"Third-party packages and images aren't supported yet. Long compilations will be aborted.".into(),
									),
									component: Box::new(Component::TextInput(TextInput {
										id: None,
										custom_id: "code".into(),
										#[expect(deprecated, reason = "not actually used")]
										label: None,
										style: TextInputStyle::Paragraph,
										max_length: Some(4000),
										placeholder: Some(CODE_PLACEHOLDER.into()),
										required: Some(true),
										value: None,
										min_length: None,
									})),
								}),
								Component::Label(Label {
									id: None,
									label: "Mark as Spoiler?".into(),
									description: Some(
										"Whether to hide the rendered image behind a spoiler.".into(),
									),
									component: Box::new(Component::SelectMenu(SelectMenu {
										id: None,
										custom_id: "spoiler".into(),
										kind: SelectMenuType::Text,
										disabled: false,
										options: Some(vec![
											SelectMenuOption {
												default: true,
												description: None,
												emoji: None,
												label: "No".into(),
												value: "no".into(),
											},
											SelectMenuOption {
												default: false,
												description: None,
												emoji: None,
												label: "Yes".into(),
												value: "yes".into(),
											},
										]),
										placeholder: None,
										min_values: None,
										max_values: None,
										default_values: None,
										channel_types: None,
										required: None,
									})),
								}),
							]),
							..Default::default()
						}),
					},
					name => {
						error!(name, "unknown command");
						unreachable!("unknown command");
					}
				}
			}
			Interaction {
				kind: InteractionType::ModalSubmit,
				id,
				user,
				member,
				guild_id,
				channel,
				application_id,
				token,
				data: Some(InteractionData::ModalSubmit(modal_data)),
				..
			} => {
				let ModalInteractionData { custom_id, components, .. } = *modal_data;

				let user = member.and_then(|m| m.user).or(user).expect("user must be present");
				let channel_id = channel.map(|c| c.id);
				info!(interaction_id = ?id, user_id = ?user.id, ?guild_id, ?channel_id, "received modal submit");

				assert_eq!(custom_id, "typst");

				// Extract code from Label > TextInput and spoiler from Label > StringSelect
				let mut code: Option<String> = None;
				let mut spoiler = false;

				for component in components {
					let ModalInteractionLabel { component: inner, .. } = match component {
						ModalInteractionComponent::Label(label) => label,
						_ => continue,
					};

					match *inner {
						ModalInteractionComponent::TextInput(ModalInteractionTextInput {
							custom_id,
							value,
							..
						}) if custom_id == "code" => {
							code = Some(value);
						}
						ModalInteractionComponent::StringSelect(ModalInteractionStringSelect {
							custom_id,
							values,
							..
						}) if custom_id == "spoiler" => {
							spoiler = values.first().is_some_and(|v| v == "yes");
						}
						_ => {}
					}
				}

				let value = code.expect("code input must be present");

				static TYPST_PREAMBLE: &str = include_str!("preamble.typ");
				let mut content = TYPST_PREAMBLE.to_owned();
				content.push_str(&value);

				let token = token.into_boxed_str();
				let handle = tokio::spawn(self.subprocess(
					application_id,
					token,
					content.into_boxed_str(),
					spoiler,
				));
				trace!(?handle, "spawned subprocess");

				InteractionResponse {
					kind: InteractionResponseType::DeferredChannelMessageWithSource,
					data: None,
				}
			}
			_ => unreachable!("unknown interaction"),
		}
	}

	#[instrument(skip(self))]
	async fn subprocess(
		self: Arc<Self>,
		application_id: ApplicationId,
		token: Box<str>,
		value: Box<str>,
		spoiler: bool,
	) {
		let mut command = Command::new(self.exe_path.as_os_str())
			.arg("worker")
			.stdin(Stdio::piped())
			.stdout(Stdio::piped())
			.spawn()
			.expect("worker process must be spawned");

		// TODO: attachment_size_limit

		command
			.stdin
			.take()
			.expect("stdin must have been piped")
			.write_all(value.as_bytes())
			.await
			.expect("stdin must be writable");

		let stdout = command.stdout.take().expect("stdout must have been piped");
		let mut stdout = BufReader::new(stdout);

		let now = Instant::now();
		let result =
			tokio::time::timeout(self.compilation_timeout, buffer::read_usize(&mut stdout)).await;
		let elapsed_ms = now.elapsed().as_millis();
		info!(millis = elapsed_ms, "compilation timer");

		let http = self.http.interaction(application_id, token);
		match result {
			Ok(result) => match Self::critical_section(stdout, &mut command, result).await {
				Ok((file, embeds)) => {
					// Should have exited by now
					drop(command);

					// Replace previously rendered code block with the rendered attachment
					if !file.is_empty() {
						http.replace_response_with_attachments(&[Attachment {
							description: None,
							file,
							filename: From::from(if spoiler {
								"SPOILER_typst.webp"
							} else {
								"typst.webp"
							}),
							id: 0,
						}])
						.await
						.expect("original response replacement must succeed");
					}

					let value = format!("Compiled in **{elapsed_ms}ms**.");
					http.create_ephemeral_followup_with_embeds(&value, &embeds)
						.await
						.expect("ephemeral followup must succeed");
				}
				Err(error) => {
					error!(?error, "worker process crashed");

					// Reap the resources from the child process for proper garbage collection
					command.kill().await.expect("crashed worker process must be killed");

					http.update_response_with_embeds(
						"The Typst renderer crashed. Please try again with simpler input.",
						&[],
					)
					.await
					.expect("original followup must succeed");
				}
			},
			Err(error) => {
				error!(?error, "timeout when compiling code");

				// We need to preemptively kill the process or else we'll risk running infinite
				// loops in the background.
				command.kill().await.expect("lagging worker process must be killed");
				drop(command);

				let value = format!(
					"Compilation timed out after **{elapsed_ms}ms**. Check your code for infinite loops and expensive operations."
				);

				http.update_response_with_embeds(&value, &[])
					.await
					.expect("original response edit must succeed");
			}
		}
	}

	#[instrument(skip_all)]
	async fn critical_section<Stdout: AsyncBufRead + Unpin>(
		mut stdout: Stdout,
		command: &mut Child,
		warning_count: io::Result<usize>,
	) -> io::Result<(Vec<u8>, Vec<Embed>)> {
		let warning_count = warning_count?;
		info!(warnings = warning_count, "read warning count");

		// Shared string buffer whose capacity can be reused by everyone
		let mut buffer = String::new();

		let mut warning_embed_fields = Vec::<EmbedField>::new();
		for _ in 0..warning_count {
			warning_embed_fields.push(EmbedField {
				name: buffer::read_line(&mut stdout, &mut buffer).await?,
				value: buffer::read_line(&mut stdout, &mut buffer).await?,
				inline: false,
			});
		}

		let error_count = buffer::read_usize(&mut stdout).await?;
		info!(errors = error_count, "reading error count");

		let mut error_embed_fields = Vec::<EmbedField>::new();
		for _ in 0..error_count {
			error_embed_fields.push(EmbedField {
				name: buffer::read_line(&mut stdout, &mut buffer).await?,
				value: buffer::read_line(&mut stdout, &mut buffer).await?,
				inline: false,
			});
		}

		// No need for the shared buffer after this point.
		drop(buffer);

		let mut file = Vec::new();
		stdout.read_to_end(&mut file).await?;

		// Should close the pipe after this point
		drop(stdout);

		// Subprocess has since exited already
		let status = command.wait().await?;
		info!(?status, "worker process exited");

		// Send errors/warnings as an ephemeral followup
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

		Ok((file, embeds))
	}
}
