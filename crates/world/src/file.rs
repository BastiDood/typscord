use typst::{
	foundations::Bytes,
	syntax::{FileId, Source},
};

pub struct File {
	pub bytes: Bytes,
	pub source: Source,
}

impl File {
	pub fn new(id: FileId, text: String) -> Self {
		let bytes = Bytes::from_string(text.clone());
		let source = Source::new(id, text);
		Self { bytes, source }
	}
}
