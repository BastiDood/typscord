use std::sync::LazyLock;
use typst::{
	foundations::Bytes,
	text::{Font, FontBook},
	utils::LazyHash,
};
use typst_assets::fonts;

type Fonts = Box<[Font]>;
pub static FONTS: LazyLock<Fonts> =
	LazyLock::new(|| fonts().flat_map(|bytes| Font::iter(Bytes::new(bytes))).collect());

pub static FONT_BOOK: LazyLock<LazyHash<FontBook>> =
	LazyLock::new(|| LazyHash::new(FontBook::from_fonts(FONTS.iter())));
