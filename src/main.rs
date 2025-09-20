use discordyst_world::World;
use image::{ColorType, ImageFormat, write_buffer_with_format};
use std::io::{Cursor, Read as _, Result, Write as _, stderr, stdin, stdout};
use typst::layout::PagedDocument;
use typst_library::{diag::Warned, layout::Abs};
use typst_render::render_merged;

fn main() -> Result<()> {
	let stdout = stdout();
	let mut stdout = stdout.lock();

	let stderr = stderr();
	let mut stderr = stderr.lock();

	let mut contents = String::new();
	let bytes = stdin().read_to_string(&mut contents)?;
	writeln!(stderr, "Read {bytes} bytes ✅")?;

	let world = World::from_single_source(contents);
	let Warned { output, warnings, .. } = world.compile::<PagedDocument>();

	for warning in warnings {
		writeln!(stderr, "{warning:#?}")?;
	}

	match output {
		Ok(output) => {
			writeln!(stderr, "Rendering output...")?;
			let pixel_map = render_merged(&output, 4., Abs::zero(), None);

			writeln!(stderr, "Encoding output as WebP...")?;
			let mut buffer = Cursor::<Vec<_>>::default();
			if let Err(error) = write_buffer_with_format(
				&mut buffer,
				bytemuck::cast_slice(pixel_map.pixels()),
				pixel_map.width(),
				pixel_map.height(),
				ColorType::Rgba8,
				ImageFormat::WebP,
			) {
				writeln!(stderr, "{error:#?}")?;
				return Err(std::io::ErrorKind::Other.into());
			}

			writeln!(stderr, "Writing output...")?;
			let buffer = buffer.into_inner();
			stdout.write_all(&buffer)?;
		}
		Err(errors) => {
			for error in errors {
				writeln!(stderr, "{error:#?}")?;
			}
			return Err(std::io::ErrorKind::Other.into());
		}
	}

	writeln!(stderr, "Done ✅")?;
	Ok(())
}
