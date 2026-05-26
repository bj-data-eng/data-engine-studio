mod framework;

use des_document::StyleSheet;
use std::borrow::Cow;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[cfg(not(debug_assertions))]
const LAB_CSS: &str = include_str!("lab.css");
const RESPONSIVE_CSS: &str = r#"
@media (max-width: 1268px) {
  .drag-workbench {
    flex-direction: column;
  }

  .drag-scroll-list-card {
    width: 100%;
    flex-basis: auto;
    flex-grow: 0;
  }

  .drag-grid {
    width: 100%;
    flex-basis: auto;
    flex-grow: 0;
  }
}
"#;

pub(super) fn stylesheet() -> StyleSheet {
    let mut stylesheet = lab_stylesheet();
    stylesheet.extend(framework::stylesheet());
    stylesheet
        .extend_css(RESPONSIVE_CSS)
        .expect("lab responsive CSS stylesheet is valid");
    stylesheet
}

pub(super) fn asset_revision() -> u64 {
    let mut hasher = DefaultHasher::new();
    source().hash(&mut hasher);
    RESPONSIVE_CSS.hash(&mut hasher);
    framework::asset_revision().hash(&mut hasher);
    hasher.finish()
}

fn lab_stylesheet() -> StyleSheet {
    StyleSheet::parse_css(&source()).expect("lab CSS stylesheet is valid")
}

fn source() -> Cow<'static, str> {
    #[cfg(debug_assertions)]
    {
        return Cow::Owned(
            std::fs::read_to_string(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/ui_lab/styles/lab.css"
            ))
            .expect("lab CSS file should be readable"),
        );
    }
    #[cfg(not(debug_assertions))]
    {
        Cow::Borrowed(LAB_CSS)
    }
}
