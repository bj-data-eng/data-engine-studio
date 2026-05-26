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

pub(super) fn asset_revision() -> u64 {
    let mut hasher = DefaultHasher::new();
    topbar_source().hash(&mut hasher);
    nav_source().hash(&mut hasher);
    interaction_cards_source().hash(&mut hasher);
    interaction_loop_source().hash(&mut hasher);
    nesting_source().hash(&mut hasher);
    graph_source().hash(&mut hasher);
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
