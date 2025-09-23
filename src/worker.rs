use std::io::{self, Read as _, Write as _};
use tracing::info;
use typscord_world::{Render, SourceDiagnostic, Warned, World};

/// Discord only allows up to 25 fields per embed.
pub const MAX_DIAGNOSTIC_COUNT: usize = 25;

pub fn main() -> io::Result<()> {
	let mut content = String::new();

	{
		let size = io::stdin().read_to_string(&mut content)?;
		info!(%size, "read content from stdin");
	}

	let world = World::from_single_source(content);
	let Warned { output, mut warnings } = world.render();

	let warning_count = warnings.len();
	info!(warnings = warning_count, "document render complete");

	// Only show the most important warnings
	warnings.truncate(MAX_DIAGNOSTIC_COUNT);

	let mut stdout = io::stdout().lock();

	stdout.write_all(&warning_count.to_be_bytes())?; // warnings
	for SourceDiagnostic { message, hints, .. } in warnings {
		writeln!(stdout, "{message}")?; // name
		let hint = hints.first().map(AsRef::as_ref).unwrap_or("No hints provided.");
		writeln!(stdout, "{hint}")?; // value
	}

	match output {
		Ok(Render { buffer, .. }) => {
			let buffer_size = buffer.len(); // image
			info!(size = buffer_size, "image rendered");
			assert!(buffer_size < 1024 * 1024 * 8, "maximum file size exceeded"); // 8 MiB

			// communicate that there is no error
			stdout.write_all(&0usize.to_be_bytes())?;
			stdout.write_all(&buffer)?;
		}
		Err(mut errors) => {
			let error_count = errors.len();
			info!(errors = error_count, "errors encountered");

			// Only show the most important errors
			errors.truncate(MAX_DIAGNOSTIC_COUNT);

			stdout.write_all(&error_count.to_be_bytes())?; // errors
			for SourceDiagnostic { message, hints, .. } in errors {
				writeln!(stdout, "{message}")?; // name
				let hint = hints.first().map(AsRef::as_ref).unwrap_or("No hints provided.");
				writeln!(stdout, "{hint}")?; // value
			}
		}
	}

	drop(stdout);
	Ok(())
}
