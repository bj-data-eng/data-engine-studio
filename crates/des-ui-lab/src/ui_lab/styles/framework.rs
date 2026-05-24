use des_document::StyleSheet;

const FRAMEWORK_CSS: &str = include_str!("framework.css");

pub(super) fn stylesheet() -> StyleSheet {
    StyleSheet::parse_css(FRAMEWORK_CSS).expect("lab framework CSS stylesheet is valid")
}
