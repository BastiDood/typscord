mod file;
mod font;
mod library;

use bytemuck::cast_slice;
use ecow::EcoVec;
use file::File;
use font::{FONT_BOOK, FONTS};
use image::{ColorType, ImageFormat, write_buffer_with_format};
use library::LIBRARY;
use std::collections::BTreeMap;
use std::io::Cursor;
use time::{PrimitiveDateTime, UtcDateTime, UtcOffset};
use typst::{
	Document, Library, World as TypstWorld, compile,
	diag::{FileError, FileResult, SourceResult},
	foundations::{Bytes, Datetime},
	layout::{Abs, PagedDocument},
	syntax::{FileId, Source, VirtualPath},
	text::{Font, FontBook},
	utils::LazyHash,
};
use typst_render::render_merged;

pub use typst::diag::{SourceDiagnostic, Warned};

type Diagnostics = EcoVec<SourceDiagnostic>;
pub struct Render {
	pub document: PagedDocument,
	pub buffer: Vec<u8>,
}

pub struct World {
	sources: BTreeMap<FileId, File>,
}

impl World {
	pub fn from_single_source(contents: String) -> Self {
		// Entry point is basically a single file named `main.typ`
		let entry_file_id = FileId::new_fake(VirtualPath::new("/main.typ"));
		let entry_source = File::new(entry_file_id, contents);
		Self { sources: BTreeMap::from([(entry_file_id, entry_source)]) }
	}

	pub fn compile<D: Document>(&self) -> Warned<SourceResult<D>> {
		compile(self)
	}

	pub fn render(&self) -> Warned<Result<Render, Diagnostics>> {
		let Warned { output, warnings } = self.compile::<PagedDocument>();
		Warned {
			warnings,
			output: output.map(|document| {
				let pixel_map = render_merged(&document, 4., Abs::zero(), None);
				let mut buffer = Cursor::<Vec<_>>::default();
				write_buffer_with_format(
					&mut buffer,
					cast_slice(pixel_map.pixels()),
					pixel_map.width(),
					pixel_map.height(),
					ColorType::Rgba8,
					ImageFormat::WebP,
				)
				.expect("writing to Vec must be infallible");
				Render { document, buffer: buffer.into_inner() }
			}),
		}
	}
}

impl TypstWorld for World {
	fn library(&self) -> &LazyHash<Library> {
		&LIBRARY
	}

	fn book(&self) -> &LazyHash<FontBook> {
		&FONT_BOOK
	}

	fn font(&self, index: usize) -> Option<Font> {
		FONTS.get(index).cloned()
	}

	fn today(&self, offset: Option<i64>) -> Option<Datetime> {
		let now = UtcDateTime::now();
		let offset = offset
			.and_then(|offset| {
				let offset = offset.try_into().ok()?;
				UtcOffset::from_hms(offset, 0, 0).ok()
			})
			.unwrap_or(UtcOffset::UTC);
		let now = now.to_offset(offset);
		Some(Datetime::Datetime(PrimitiveDateTime::new(now.date(), now.time())))
	}

	fn main(&self) -> FileId {
		let (&id, _) = self.sources.first_key_value().expect("root source must be present");
		id
	}

	fn source(&self, id: FileId) -> FileResult<Source> {
		// TODO: Support external packages.
		let File { source, .. } = self.sources.get(&id).ok_or(FileError::NotSource)?;
		Ok(source.clone())
	}

	fn file(&self, id: FileId) -> FileResult<Bytes> {
		// TODO: Support external packages.
		let File { bytes, .. } = self.sources.get(&id).ok_or(FileError::NotSource)?;
		Ok(bytes.clone())
	}
}
