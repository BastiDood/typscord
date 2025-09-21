use serenity::{
	builder::{CreateActionRow, CreateInputText, CreateInteractionResponseMessage, CreateModal},
	model::application::{
		ActionRow, ActionRowComponent, CommandData, CommandInteraction, CommandType, InputText,
		InputTextStyle, Interaction, InteractionResponseFlags, ModalInteraction,
		ModalInteractionData,
	},
};

pub use serenity::builder::CreateInteractionResponse;

#[must_use]
pub fn handle(interaction: Interaction) -> CreateInteractionResponse {
	match interaction {
		Interaction::Ping(_) => CreateInteractionResponse::Pong,
		Interaction::Command(CommandInteraction {
			data: CommandData { kind: CommandType::ChatInput, name, .. },
			..
		}) => {
			assert_eq!(name, "typst");
			CreateInteractionResponse::Modal(
				CreateModal::new("modal", "Render Typst Code").components(vec![
					CreateActionRow::InputText(
						CreateInputText::new(InputTextStyle::Paragraph, "Typst Code", "code")
							.max_length(4000)
							.placeholder(
								"Enter your Typst code here. Third-party packages are not yet supported.",
							),
					),
				]),
			)
		}
		Interaction::Modal(ModalInteraction {
			token,
			data: ModalInteractionData { custom_id, mut components, .. },
			..
		}) => {
			assert_eq!(custom_id, "modal");

			let action_row = components.pop().expect("modal must have at least one action row");
			assert!(components.is_empty(), "modal must have exactly one action row");

			let ActionRow { mut components, .. } = action_row;
			let input_text = components.pop().expect("modal must have at least one component");
			assert!(components.is_empty(), "modal must have exactly one component");

			let ActionRowComponent::InputText(InputText { custom_id, value, .. }) = input_text
			else {
				unreachable!("modal must be exactly one text input");
			};
			assert_eq!(custom_id, "code");

			// Ship the request to a subprocess
			let value = value.expect("modal text input has required value");

			CreateInteractionResponse::Defer(
				CreateInteractionResponseMessage::new()
					.flags(InteractionResponseFlags::EPHEMERAL)
					.content("Rendering your Typst code..."),
			)

			/*
			match output {
				Ok(Render { buffer, .. }) => CreateInteractionResponse::Defer(
					CreateInteractionResponseMessage::new()
						.flags(InteractionResponseFlags::EPHEMERAL)
						.content("Rendering your Typst code..."),
				),
				Err(errors) => CreateInteractionResponse::Message(
					CreateInteractionResponseMessage::new()
						.flags(InteractionResponseFlags::EPHEMERAL)
						.content(
							"The code could not be rendered due to the following compilation errors.",
						)
						.embeds({
							let mut embeds = vec![
								CreateEmbed::new()
									.title("Errors")
									.description("Only the first 25 errors are shown.")
									.fields(errors.into_iter().take(25).map(
										|SourceDiagnostic { message, hints, .. }| {
											(message, hints.join("\n"), false)
										},
									)),
							];
							if !warnings.is_empty() {
								embeds.push(
									CreateEmbed::new()
										.title("Warnings")
										.description("Only the first 25 warnings are shown.")
										.fields(warnings.into_iter().take(25).map(
											|SourceDiagnostic { message, hints, .. }| {
												(message, hints.join("\n"), false)
											},
										)),
								);
							}
							embeds
						}),
				),
			}
			*/
		}
		_ => unreachable!("unknown interaction"),
	}
}
