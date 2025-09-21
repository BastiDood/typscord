use std::sync::LazyLock;
use ttf_parser::fonts_in_collection;
use typst::{
	foundations::Bytes,
	text::{Font, FontBook},
	utils::LazyHash,
};
use typst_assets::fonts;

type Fonts = Box<[Font]>;
pub static FONTS: LazyLock<Fonts> = LazyLock::new(|| {
	fonts()
		.flat_map(|bytes| {
			let faces = fonts_in_collection(bytes).unwrap_or(1);
			let bytes = Bytes::new(bytes);
			(0..faces).flat_map(move |index| Font::new(bytes.clone(), index))
		})
		.collect()
});

pub static FONT_BOOK: LazyLock<LazyHash<FontBook>> =
	LazyLock::new(|| LazyHash::new(FontBook::from_fonts(FONTS.iter())));
