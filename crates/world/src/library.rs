use std::sync::LazyLock;
use typst::{Library, utils::LazyHash};

pub static LIBRARY: LazyLock<LazyHash<Library>> = LazyLock::new(Default::default);
