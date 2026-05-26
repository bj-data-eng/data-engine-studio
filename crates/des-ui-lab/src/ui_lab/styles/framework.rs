use des_document::StyleSheet;
use std::borrow::Cow;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[cfg(not(debug_assertions))]
const FRAMEWORK_CSS: &str = include_str!("framework.css");

pub(super) fn stylesheet() -> StyleSheet {
    StyleSheet::parse_css(&source()).expect("lab framework CSS stylesheet is valid")
}

pub(super) fn asset_revision() -> u64 {
    hash_source(&source())
}

fn source() -> Cow<'static, str> {
    #[cfg(debug_assertions)]
    {
        return Cow::Owned(
            std::fs::read_to_string(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/ui_lab/styles/framework.css"
            ))
            .expect("lab framework CSS file should be readable"),
        );
    }
    #[cfg(not(debug_assertions))]
    {
        Cow::Borrowed(FRAMEWORK_CSS)
    }
}

fn hash_source(source: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    source.hash(&mut hasher);
    hasher.finish()
}
