use std::sync::LazyLock;
use typst_library::Library;
use typst_utils::LazyHash;

pub static LIBRARY: LazyLock<LazyHash<Library>> = LazyLock::new(Default::default);
