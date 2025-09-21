use discordyst_world::{Render, SourceDiagnostic, Warned, World};
use std::io::{self, Read as _, Write as _};
use tracing::info;

pub fn main() -> io::Result<()> {
	let mut content = String::new();

	{
		let mut stdin = io::stdin();
		let size = stdin.read_to_string(&mut content)?;
		info!(%size, "read content from stdin");
	}

	let world = World::from_single_source(content);
	let Warned { output, warnings } = world.render();

	let warning_count = warnings.len();
	info!(warnings = warning_count, "document render complete");

	let stdout = io::stdout();
	let mut stdout = stdout.lock();

	stdout.write_all(&warning_count.to_be_bytes())?; // warnings
	for SourceDiagnostic { message, hints, .. } in warnings {
		writeln!(stdout, "{message}")?; // name
		let hint = hints.first().map(AsRef::as_ref).unwrap_or("No hints provided.");
		writeln!(stdout, "{hint}")?; // value
	}

	match output {
		Ok(Render { buffer, .. }) => {
			// communicate that there is no error
			stdout.write_all(&0usize.to_be_bytes())?;

			let buffer_size = buffer.len(); // image
			info!(size = buffer_size, "image rendered");

			stdout.write_all(&buffer_size.to_be_bytes())?;
			stdout.write_all(&buffer)?;
		}
		Err(errors) => {
			let error_count = errors.len();
			info!(errors = error_count, "errors encountered");

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
