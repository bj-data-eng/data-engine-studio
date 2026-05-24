mod framework;

use des_document::StyleSheet;

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

fn lab_stylesheet() -> StyleSheet {
    StyleSheet::parse_css(LAB_CSS).expect("lab CSS stylesheet is valid")
}
