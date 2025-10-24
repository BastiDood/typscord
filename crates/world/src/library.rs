use std::sync::LazyLock;
use typst::{Library, LibraryExt as _, utils::LazyHash};

pub static LIBRARY: LazyLock<LazyHash<Library>> =
	LazyLock::new(|| LazyHash::new(Library::default()));
