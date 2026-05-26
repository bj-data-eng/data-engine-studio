use des_document::DocumentBuilder;
use des_html::HtmlDocument;

const TOPBAR_HTML: &str = include_str!("html/topbar.html");
const NAV_HTML: &str = include_str!("html/nav.html");

pub(super) fn append_topbar(ui: &mut DocumentBuilder) {
    topbar_fragment().append_to_builder(ui);
}

pub(super) fn append_nav(ui: &mut DocumentBuilder) {
    nav_fragment().append_to_builder(ui);
}

fn topbar_fragment() -> HtmlDocument {
    HtmlDocument::parse_fragment(TOPBAR_HTML).expect("lab topbar HTML is valid")
}

fn nav_fragment() -> HtmlDocument {
    HtmlDocument::parse_fragment(NAV_HTML).expect("lab nav HTML is valid")
}
