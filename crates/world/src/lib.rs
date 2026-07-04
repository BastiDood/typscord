mod file;
mod font;
mod library;

use bytemuck::cast_slice;
use ecow::EcoVec;
use file::File;
use font::{FONT_BOOK, FONTS};
use image::{ColorType, ImageFormat, write_buffer_with_format};
use library::LIBRARY;
use std::io::Cursor;
use time::{UtcDateTime, UtcOffset};
use typst::{
	Library, World as TypstWorld, compile,
	diag::{FileError, FileResult, SourceResult},
	foundations::{Bytes, Datetime, Duration as TypstDuration, Output},
	layout::Abs,
	syntax::{FileId, RootedPath, Source, VirtualPath, VirtualRoot},
	text::{Font, FontBook},
	utils::{LazyHash, Scalar},
};
use typst_layout::PagedDocument;
use typst_render::{RenderOptions, render_merged};

pub use typst::diag::{SourceDiagnostic, Warned};

type Diagnostics = EcoVec<SourceDiagnostic>;
pub struct Render {
	pub document: PagedDocument,
	pub buffer: Vec<u8>,
}

pub struct World {
	main: FileId,
	file: File,
}

impl World {
	pub fn from_single_source(contents: String) -> Self {
		// Entry point is basically a single file named `main.typ`
		let entry_path = RootedPath::new(
			VirtualRoot::Project,
			VirtualPath::new("main.typ").expect("static path must be valid"),
		);
		let entry_file_id = FileId::unique(entry_path);
		let entry_source = File::new(entry_file_id, contents);
		Self { main: entry_file_id, file: entry_source }
	}

	pub fn compile<T: Output>(&self) -> Warned<SourceResult<T>> {
		compile(self)
	}

	pub fn render(&self) -> Warned<Result<Render, Diagnostics>> {
		let Warned { output, warnings } = self.compile::<PagedDocument>();
		Warned {
			warnings,
			output: output.map(|document| {
				let pixel_map = render_merged(
					&document,
					&RenderOptions { pixel_per_pt: Scalar::new(4.0), ..Default::default() },
					Abs::zero(),
					None,
				);
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

	fn entry(&self, id: FileId) -> FileResult<&File> {
		if id == self.main { Ok(&self.file) } else { Err(FileError::NotSource) }
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

	fn today(&self, offset: Option<TypstDuration>) -> Option<Datetime> {
		let now = UtcDateTime::now();
		let offset = offset
			.and_then(|offset| {
				let offset: time::Duration = offset.into();
				let seconds = offset.whole_seconds().try_into().ok()?;
				UtcOffset::from_whole_seconds(seconds).ok()
			})
			.unwrap_or(UtcOffset::UTC);
		let now = now.to_offset(offset);
		Datetime::from_ymd(now.year(), now.month().into(), now.day())
	}

	fn main(&self) -> FileId {
		self.main
	}

	fn source(&self, id: FileId) -> FileResult<Source> {
		// TODO: Support external packages.
		let File { source, .. } = self.entry(id)?;
		Ok(source.clone())
	}

	fn file(&self, id: FileId) -> FileResult<Bytes> {
		// TODO: Support external packages.
		let File { bytes, .. } = self.entry(id)?;
		Ok(bytes.clone())
	}
}
