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
		let bytes = Bytes::new(text.clone().into_bytes());
		let source = Source::new(id, text);
		Self { bytes, source }
	}
}
