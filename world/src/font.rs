use std::sync::LazyLock;
use ttf_parser::fonts_in_collection;
use typst_assets::fonts;
use typst_library::{foundations::Bytes, text::Font};

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
