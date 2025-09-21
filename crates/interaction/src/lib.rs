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
		MessageFlags,
		component::{ActionRow, Component, TextInput, TextInputStyle},
	},
	http::interaction::{InteractionResponseData, InteractionResponseType},
};

pub use twilight_model::http::interaction::InteractionResponse;

#[must_use]
pub fn handle(interaction: Interaction) -> InteractionResponse {
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
					custom_id: Some("modal".to_string()),
					title: Some("Render Typst Code".to_string()),
					components: Some(vec![
						Component::ActionRow(ActionRow {
							components: vec![
								Component::TextInput(TextInput {
									custom_id: "code".to_string(),
									label: "Typst Code".to_string(),
									style: TextInputStyle::Paragraph,
									max_length: Some(4000),
									placeholder: Some("Enter your Typst code here. Third-party packages are not yet supported.".to_string()),
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

			// TODO: Ship the request to a subprocess
			let value = value.expect("modal text input has required value");

			InteractionResponse {
				kind: InteractionResponseType::DeferredChannelMessageWithSource,
				data: Some(InteractionResponseData {
					content: Some("Rendering your Typst code...".to_string()),
					flags: Some(MessageFlags::EPHEMERAL),
					..Default::default()
				}),
			}
		}
		_ => unreachable!("unknown interaction"),
	}
}
