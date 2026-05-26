use des_document::DocumentBuilder;
use des_html::HtmlDocument;
use std::borrow::Cow;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[cfg(not(debug_assertions))]
const TOPBAR_HTML: &str = include_str!("html/topbar.html");
#[cfg(not(debug_assertions))]
const NAV_HTML: &str = include_str!("html/nav.html");
#[cfg(not(debug_assertions))]
const INTERACTION_CARDS_HTML: &str = include_str!("html/interaction-cards.html");
#[cfg(not(debug_assertions))]
const INTERACTION_LOOP_HTML: &str = include_str!("html/interaction-loop.html");
#[cfg(not(debug_assertions))]
const NESTING_HTML: &str = include_str!("html/nesting.html");
#[cfg(not(debug_assertions))]
const GRAPH_HTML: &str = include_str!("html/graph.html");
#[cfg(not(debug_assertions))]
const STRUCTURAL_SELECTORS_HTML: &str = include_str!("html/structural-selectors.html");
#[cfg(not(debug_assertions))]
const TEXT_SPECIMENS_HTML: &str = include_str!("html/text-specimens.html");
#[cfg(not(debug_assertions))]
const TABLE_HTML: &str = include_str!("html/table.html");

pub(super) fn append_topbar(ui: &mut DocumentBuilder) {
    topbar_fragment().append_to_builder(ui);
}

pub(super) fn append_nav(ui: &mut DocumentBuilder) {
    nav_fragment().append_to_builder(ui);
}

pub(super) fn append_interaction_cards(ui: &mut DocumentBuilder) {
    interaction_cards_fragment().append_to_builder(ui);
}

pub(super) fn append_interaction_loop(ui: &mut DocumentBuilder) {
    interaction_loop_fragment().append_to_builder(ui);
}

pub(super) fn append_nesting(ui: &mut DocumentBuilder) {
    nesting_fragment().append_to_builder(ui);
}

pub(super) fn append_graph(ui: &mut DocumentBuilder) {
    graph_fragment().append_to_builder(ui);
}

pub(super) fn append_structural_selectors(ui: &mut DocumentBuilder) {
    structural_selectors_fragment().append_to_builder(ui);
}

pub(super) fn append_text_specimens(ui: &mut DocumentBuilder) {
    text_specimens_fragment().append_to_builder(ui);
}

pub(super) fn append_table(ui: &mut DocumentBuilder) {
    table_fragment().append_to_builder(ui);
}

pub(super) fn asset_revision() -> u64 {
    let mut hasher = DefaultHasher::new();
    topbar_source().hash(&mut hasher);
    nav_source().hash(&mut hasher);
    interaction_cards_source().hash(&mut hasher);
    interaction_loop_source().hash(&mut hasher);
    nesting_source().hash(&mut hasher);
    graph_source().hash(&mut hasher);
    structural_selectors_source().hash(&mut hasher);
    text_specimens_source().hash(&mut hasher);
    table_source().hash(&mut hasher);
    hasher.finish()
}

fn topbar_fragment() -> HtmlDocument {
    HtmlDocument::parse_fragment(&topbar_source()).expect("lab topbar HTML is valid")
}

fn nav_fragment() -> HtmlDocument {
    HtmlDocument::parse_fragment(&nav_source()).expect("lab nav HTML is valid")
}

fn interaction_cards_fragment() -> HtmlDocument {
    HtmlDocument::parse_fragment(&interaction_cards_source())
        .expect("lab interaction cards HTML is valid")
}

fn interaction_loop_fragment() -> HtmlDocument {
    HtmlDocument::parse_fragment(&interaction_loop_source())
        .expect("lab interaction loop HTML is valid")
}

fn nesting_fragment() -> HtmlDocument {
    HtmlDocument::parse_fragment(&nesting_source()).expect("lab nesting HTML is valid")
}

fn graph_fragment() -> HtmlDocument {
    HtmlDocument::parse_fragment(&graph_source()).expect("lab graph HTML is valid")
}

fn structural_selectors_fragment() -> HtmlDocument {
    HtmlDocument::parse_fragment(&structural_selectors_source())
        .expect("lab structural selectors HTML is valid")
}

fn text_specimens_fragment() -> HtmlDocument {
    HtmlDocument::parse_fragment(&text_specimens_source())
        .expect("lab text specimens HTML is valid")
}

fn table_fragment() -> HtmlDocument {
    HtmlDocument::parse_fragment(&table_source()).expect("lab table HTML is valid")
}

fn topbar_source() -> Cow<'static, str> {
    #[cfg(debug_assertions)]
    {
        return Cow::Owned(
            std::fs::read_to_string(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/ui_lab/html/topbar.html"
            ))
            .expect("lab topbar HTML file should be readable"),
        );
    }
    #[cfg(not(debug_assertions))]
    {
        Cow::Borrowed(TOPBAR_HTML)
    }
}

fn nav_source() -> Cow<'static, str> {
    #[cfg(debug_assertions)]
    {
        return Cow::Owned(
            std::fs::read_to_string(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/ui_lab/html/nav.html"
            ))
            .expect("lab nav HTML file should be readable"),
        );
    }
    #[cfg(not(debug_assertions))]
    {
        Cow::Borrowed(NAV_HTML)
    }
}

fn interaction_cards_source() -> Cow<'static, str> {
    #[cfg(debug_assertions)]
    {
        return Cow::Owned(
            std::fs::read_to_string(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/ui_lab/html/interaction-cards.html"
            ))
            .expect("lab interaction cards HTML file should be readable"),
        );
    }
    #[cfg(not(debug_assertions))]
    {
        Cow::Borrowed(INTERACTION_CARDS_HTML)
    }
}

fn interaction_loop_source() -> Cow<'static, str> {
    #[cfg(debug_assertions)]
    {
        return Cow::Owned(
            std::fs::read_to_string(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/ui_lab/html/interaction-loop.html"
            ))
            .expect("lab interaction loop HTML file should be readable"),
        );
    }
    #[cfg(not(debug_assertions))]
    {
        Cow::Borrowed(INTERACTION_LOOP_HTML)
    }
}

fn nesting_source() -> Cow<'static, str> {
    #[cfg(debug_assertions)]
    {
        return Cow::Owned(
            std::fs::read_to_string(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/ui_lab/html/nesting.html"
            ))
            .expect("lab nesting HTML file should be readable"),
        );
    }
    #[cfg(not(debug_assertions))]
    {
        Cow::Borrowed(NESTING_HTML)
    }
}

fn graph_source() -> Cow<'static, str> {
    #[cfg(debug_assertions)]
    {
        return Cow::Owned(
            std::fs::read_to_string(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/ui_lab/html/graph.html"
            ))
            .expect("lab graph HTML file should be readable"),
        );
    }
    #[cfg(not(debug_assertions))]
    {
        Cow::Borrowed(GRAPH_HTML)
    }
}

fn structural_selectors_source() -> Cow<'static, str> {
    #[cfg(debug_assertions)]
    {
        return Cow::Owned(
            std::fs::read_to_string(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/ui_lab/html/structural-selectors.html"
            ))
            .expect("lab structural selectors HTML file should be readable"),
        );
    }
    #[cfg(not(debug_assertions))]
    {
        Cow::Borrowed(STRUCTURAL_SELECTORS_HTML)
    }
}

fn text_specimens_source() -> Cow<'static, str> {
    #[cfg(debug_assertions)]
    {
        return Cow::Owned(
            std::fs::read_to_string(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/ui_lab/html/text-specimens.html"
            ))
            .expect("lab text specimens HTML file should be readable"),
        );
    }
    #[cfg(not(debug_assertions))]
    {
        Cow::Borrowed(TEXT_SPECIMENS_HTML)
    }
}

fn table_source() -> Cow<'static, str> {
    #[cfg(debug_assertions)]
    {
        return Cow::Owned(
            std::fs::read_to_string(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/ui_lab/html/table.html"
            ))
            .expect("lab table HTML file should be readable"),
        );
    }
    #[cfg(not(debug_assertions))]
    {
        Cow::Borrowed(TABLE_HTML)
    }
}
