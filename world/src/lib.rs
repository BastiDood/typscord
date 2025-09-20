mod file;
mod font;
mod library;

use file::File;
use font::{FONT_BOOK, FONTS};
use library::LIBRARY;
use std::collections::BTreeMap;
use time::{PrimitiveDateTime, UtcDateTime, UtcOffset};
use typst::{Document, compile};
use typst_library::{
	Library, World as TypstWorld,
	diag::{FileError, FileResult, SourceResult, Warned},
	foundations::{Bytes, Datetime},
	text::{Font, FontBook},
};
use typst_syntax::{FileId, Source, VirtualPath};
use typst_utils::LazyHash;

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
