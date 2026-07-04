use std::io;
use tokio::io::{AsyncReadExt as _, AsyncWrite, AsyncWriteExt as _};
use tracing::{error, info, instrument};
use typscord_world::{Render, SourceDiagnostic, Warned, World};

/// Discord only allows up to 25 fields per embed.
pub const MAX_DIAGNOSTIC_COUNT: usize = 25;

#[instrument]
pub async fn render() -> io::Result<()> {
	let mut content = String::new();
	{
		let size = tokio::io::stdin().read_to_string(&mut content).await?;
		info!(%size, "read content from stdin");
	}

	let world = World::from_single_source(content);
	let Warned { output, mut warnings } = world.render();

	let warning_count = warnings.len();
	info!(warnings = warning_count, "document render complete");

	// Only show the most important warnings
	warnings.truncate(MAX_DIAGNOSTIC_COUNT);

	let mut stdout = tokio::io::stdout();

	stdout.write_all(&warning_count.to_be_bytes()).await?; // warnings
	write_diagnostics(&mut stdout, warnings).await?;

	match output {
		Ok(Render { buffer, .. }) => {
			let buffer_size = buffer.len(); // image
			info!(size = buffer_size, "image rendered");

			if buffer_size >= 1024 * 1024 * 8 {
				// 8 MiB is Discord's limit.
				error!(size = buffer_size, "maximum file size exceeded");
				return Err(io::ErrorKind::FileTooLarge.into());
			}

			// communicate that there is no error
			stdout.write_all(&0usize.to_be_bytes()).await?;
			stdout.write_all(&buffer).await?;
		}
		Err(mut errors) => {
			let error_count = errors.len();
			info!(errors = error_count, "errors encountered");

			// Only show the most important errors
			errors.truncate(MAX_DIAGNOSTIC_COUNT);

			stdout.write_all(&error_count.to_be_bytes()).await?; // errors
			write_diagnostics(&mut stdout, errors).await?;
		}
	}

	drop(stdout);
	Ok(())
}

async fn write_diagnostics(
	stdout: &mut (impl AsyncWrite + Unpin),
	diagnostics: impl IntoIterator<Item = SourceDiagnostic>,
) -> io::Result<()> {
	for SourceDiagnostic { message, hints, .. } in diagnostics {
		stdout.write_all(message.as_bytes()).await?; // name
		stdout.write_all(b"\n").await?;
		let hint = hints.first().map(|hint| hint.v.as_str()).unwrap_or("No hints provided.");
		stdout.write_all(hint.as_bytes()).await?; // value
		stdout.write_all(b"\n").await?;
	}
	Ok(())
}
