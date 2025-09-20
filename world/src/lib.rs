mod file;
mod font;

use file::File;
use font::FONTS;
use std::collections::BTreeMap;
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
	library: LazyHash<Library>,
	book: LazyHash<FontBook>,
	sources: BTreeMap<FileId, File>,
}

impl World {
	pub fn from_single_source(contents: String) -> Self {
		// Entry point is basically a single file named `main.typ`
		let entry_file_id = FileId::new_fake(VirtualPath::new("/main.typ"));
		let entry_source = File::new(entry_file_id, contents);
		Self {
			library: LazyHash::default(),
			book: LazyHash::default(),
			sources: BTreeMap::from([(entry_file_id, entry_source)]),
		}
	}

	pub fn compile<D: Document>(&self) -> Warned<SourceResult<D>> {
		compile(self)
	}
}

impl TypstWorld for World {
	fn library(&self) -> &LazyHash<Library> {
		&self.library
	}

	fn book(&self) -> &LazyHash<FontBook> {
		&self.book
	}

	fn font(&self, index: usize) -> Option<Font> {
		FONTS.get(index).cloned()
	}

	fn today(&self, _: Option<i64>) -> Option<Datetime> {
		// TODO: Use `jiff` for the `Datetime`.
		None
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
