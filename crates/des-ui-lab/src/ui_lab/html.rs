use des_document::{DocumentBuilder, Element, ElementSpec};
use des_html::{HtmlDocument, HtmlNode};
use std::borrow::Cow;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[cfg(not(debug_assertions))]
const LAB_ROOT_HTML: &str = include_str!("html/lab-root.html");
#[cfg(not(debug_assertions))]
const LAB_BODY_HTML: &str = include_str!("html/lab-body.html");
#[cfg(not(debug_assertions))]
const STAGE_HTML: &str = include_str!("html/stage.html");
#[cfg(not(debug_assertions))]
const DEBUG_OVERLAY_ROOT_HTML: &str = include_str!("html/debug-overlay-root.html");
#[cfg(not(debug_assertions))]
const DEBUG_OVERLAY_HTML: &str = include_str!("html/debug-overlay.html");
#[cfg(not(debug_assertions))]
const DEBUG_OVERLAY_TITLE_HTML: &str = include_str!("html/debug-overlay-title.html");
#[cfg(not(debug_assertions))]
const TOPBAR_HTML: &str = include_str!("html/topbar.html");
#[cfg(not(debug_assertions))]
const NAV_HTML: &str = include_str!("html/nav.html");
#[cfg(not(debug_assertions))]
const INTERACTION_SHELL_HTML: &str = include_str!("html/interaction-shell.html");
#[cfg(not(debug_assertions))]
const INTERACTION_CARDS_HTML: &str = include_str!("html/interaction-cards.html");
#[cfg(not(debug_assertions))]
const INTERACTION_CONTROLS_HTML: &str = include_str!("html/interaction-controls.html");
#[cfg(not(debug_assertions))]
const INTERACTION_LOOP_HTML: &str = include_str!("html/interaction-loop.html");
#[cfg(not(debug_assertions))]
const DRAGGABLE_SHELL_HTML: &str = include_str!("html/draggable-shell.html");
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
#[cfg(not(debug_assertions))]
const LAYOUT_HTML: &str = include_str!("html/layout.html");
#[cfg(not(debug_assertions))]
const SCROLLING_HTML: &str = include_str!("html/scrolling.html");
#[cfg(not(debug_assertions))]
const SHADOW_SPECIMENS_HTML: &str = include_str!("html/shadow-specimens.html");
#[cfg(not(debug_assertions))]
const STYLING_OVERVIEW_HTML: &str = include_str!("html/styling-overview.html");
#[cfg(not(debug_assertions))]
const ANIMATION_HTML: &str = include_str!("html/animation.html");
#[cfg(not(debug_assertions))]
const FLOATING_HTML: &str = include_str!("html/floating.html");

pub(super) fn append_lab_root(
    ui: &mut DocumentBuilder,
    children: impl FnOnce(&mut DocumentBuilder),
) {
    append_shell_slot(ui, lab_root_fragment(), "lab-root", children);
}

pub(super) fn append_lab_body(
    ui: &mut DocumentBuilder,
    children: impl FnOnce(&mut DocumentBuilder),
) {
    append_shell_slot(ui, lab_body_fragment(), "lab-body", children);
}

pub(super) fn append_stage(ui: &mut DocumentBuilder, children: impl FnOnce(&mut DocumentBuilder)) {
    append_shell_slot(ui, stage_fragment(), "stage", children);
}

pub(super) fn append_debug_overlay_root(
    ui: &mut DocumentBuilder,
    children: impl FnOnce(&mut DocumentBuilder),
) {
    append_shell_slot(
        ui,
        debug_overlay_root_fragment(),
        "debug-overlay-root",
        children,
    );
}

pub(super) fn append_debug_overlay(
    ui: &mut DocumentBuilder,
    children: impl FnOnce(&mut DocumentBuilder),
) {
    append_shell_slot(ui, debug_overlay_fragment(), "debug-overlay", |ui| {
        append_debug_overlay_title(ui);
        children(ui);
    });
}

fn append_debug_overlay_title(ui: &mut DocumentBuilder) {
    debug_overlay_title_fragment().append_to_builder(ui);
}

pub(super) fn append_topbar(ui: &mut DocumentBuilder) {
    topbar_fragment().append_to_builder(ui);
}

pub(super) fn append_nav(ui: &mut DocumentBuilder) {
    nav_fragment().append_to_builder(ui);
}

pub(super) fn append_interaction_shell(ui: &mut DocumentBuilder) {
    interaction_shell_fragment().append_to_builder(ui);
}

pub(super) fn append_interaction_cards(ui: &mut DocumentBuilder) {
    interaction_cards_fragment().append_to_builder(ui);
}

pub(super) fn append_interaction_controls(ui: &mut DocumentBuilder) {
    interaction_controls_fragment().append_to_builder(ui);
}

pub(super) fn append_interaction_loop(ui: &mut DocumentBuilder) {
    interaction_loop_fragment().append_to_builder(ui);
}

pub(super) fn append_draggable_shell(ui: &mut DocumentBuilder) {
    draggable_shell_fragment().append_to_builder(ui);
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

pub(super) fn append_layout(ui: &mut DocumentBuilder) {
    layout_fragment().append_to_builder(ui);
}

pub(super) fn append_scrolling(ui: &mut DocumentBuilder) {
    scrolling_fragment().append_to_builder(ui);
}

pub(super) fn append_shadow_specimens(ui: &mut DocumentBuilder) {
    shadow_specimens_fragment().append_to_builder(ui);
}

pub(super) fn append_styling_overview(ui: &mut DocumentBuilder) {
    styling_overview_fragment().append_to_builder(ui);
}

pub(super) fn append_animation(ui: &mut DocumentBuilder) {
    animation_fragment().append_to_builder(ui);
}

pub(super) fn append_floating(ui: &mut DocumentBuilder) {
    floating_fragment().append_to_builder(ui);
}

pub(super) fn asset_revision() -> u64 {
    let mut hasher = DefaultHasher::new();
    lab_root_source().hash(&mut hasher);
    lab_body_source().hash(&mut hasher);
    stage_source().hash(&mut hasher);
    debug_overlay_root_source().hash(&mut hasher);
    debug_overlay_source().hash(&mut hasher);
    debug_overlay_title_source().hash(&mut hasher);
    topbar_source().hash(&mut hasher);
    nav_source().hash(&mut hasher);
    interaction_shell_source().hash(&mut hasher);
    interaction_cards_source().hash(&mut hasher);
    interaction_controls_source().hash(&mut hasher);
    interaction_loop_source().hash(&mut hasher);
    draggable_shell_source().hash(&mut hasher);
    nesting_source().hash(&mut hasher);
    graph_source().hash(&mut hasher);
    structural_selectors_source().hash(&mut hasher);
    text_specimens_source().hash(&mut hasher);
    table_source().hash(&mut hasher);
    layout_source().hash(&mut hasher);
    scrolling_source().hash(&mut hasher);
    shadow_specimens_source().hash(&mut hasher);
    styling_overview_source().hash(&mut hasher);
    animation_source().hash(&mut hasher);
    floating_source().hash(&mut hasher);
    hasher.finish()
}

fn lab_root_fragment() -> HtmlDocument {
    HtmlDocument::parse_fragment(&lab_root_source()).expect("lab root HTML is valid")
}

fn lab_body_fragment() -> HtmlDocument {
    HtmlDocument::parse_fragment(&lab_body_source()).expect("lab body HTML is valid")
}

fn stage_fragment() -> HtmlDocument {
    HtmlDocument::parse_fragment(&stage_source()).expect("lab stage HTML is valid")
}

fn debug_overlay_root_fragment() -> HtmlDocument {
    HtmlDocument::parse_fragment(&debug_overlay_root_source())
        .expect("lab debug overlay root HTML is valid")
}

fn debug_overlay_fragment() -> HtmlDocument {
    HtmlDocument::parse_fragment(&debug_overlay_source()).expect("lab debug overlay HTML is valid")
}

fn debug_overlay_title_fragment() -> HtmlDocument {
    HtmlDocument::parse_fragment(&debug_overlay_title_source())
        .expect("lab debug overlay title HTML is valid")
}

fn topbar_fragment() -> HtmlDocument {
    HtmlDocument::parse_fragment(&topbar_source()).expect("lab topbar HTML is valid")
}

fn nav_fragment() -> HtmlDocument {
    HtmlDocument::parse_fragment(&nav_source()).expect("lab nav HTML is valid")
}

fn interaction_shell_fragment() -> HtmlDocument {
    HtmlDocument::parse_fragment(&interaction_shell_source())
        .expect("lab interaction shell HTML is valid")
}

fn interaction_cards_fragment() -> HtmlDocument {
    HtmlDocument::parse_fragment(&interaction_cards_source())
        .expect("lab interaction cards HTML is valid")
}

fn interaction_controls_fragment() -> HtmlDocument {
    HtmlDocument::parse_fragment(&interaction_controls_source())
        .expect("lab interaction controls HTML is valid")
}

fn interaction_loop_fragment() -> HtmlDocument {
    HtmlDocument::parse_fragment(&interaction_loop_source())
        .expect("lab interaction loop HTML is valid")
}

fn draggable_shell_fragment() -> HtmlDocument {
    HtmlDocument::parse_fragment(&draggable_shell_source())
        .expect("lab draggable shell HTML is valid")
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

fn layout_fragment() -> HtmlDocument {
    HtmlDocument::parse_fragment(&layout_source()).expect("lab layout HTML is valid")
}

fn scrolling_fragment() -> HtmlDocument {
    HtmlDocument::parse_fragment(&scrolling_source()).expect("lab scrolling HTML is valid")
}

fn shadow_specimens_fragment() -> HtmlDocument {
    HtmlDocument::parse_fragment(&shadow_specimens_source())
        .expect("lab shadow specimens HTML is valid")
}

fn styling_overview_fragment() -> HtmlDocument {
    HtmlDocument::parse_fragment(&styling_overview_source())
        .expect("lab styling overview HTML is valid")
}

fn animation_fragment() -> HtmlDocument {
    HtmlDocument::parse_fragment(&animation_source()).expect("lab animation HTML is valid")
}

fn floating_fragment() -> HtmlDocument {
    HtmlDocument::parse_fragment(&floating_source()).expect("lab floating HTML is valid")
}

fn append_shell_slot(
    ui: &mut DocumentBuilder,
    fragment: HtmlDocument,
    expected_id: &str,
    children: impl FnOnce(&mut DocumentBuilder),
) {
    let node = shell_node(&fragment, expected_id);
    ui.element(
        node.id()
            .expect("lab shell HTML fragment should declare an id")
            .to_owned(),
        shell_spec(node, expected_id),
        children,
    );
}

fn shell_node<'a>(fragment: &'a HtmlDocument, expected_id: &str) -> &'a HtmlNode {
    let nodes = fragment
        .children()
        .iter()
        .filter(|node| !html_whitespace_node(node))
        .collect::<Vec<_>>();
    assert_eq!(
        nodes.len(),
        1,
        "lab shell HTML fragment `{expected_id}` should contain exactly one root element"
    );
    let node = nodes[0];
    assert!(
        !node.is_text(),
        "lab shell HTML fragment `{expected_id}` should use an element root"
    );
    assert_eq!(
        node.id(),
        Some(expected_id),
        "lab shell HTML fragment id should match its slot"
    );
    assert!(
        node.text().is_none_or(str::is_empty) && node.children().iter().all(html_whitespace_node),
        "lab shell HTML fragment `{expected_id}` should leave children to Rust projection"
    );
    node
}

fn shell_spec(node: &HtmlNode, expected_id: &str) -> ElementSpec {
    let mut spec = ElementSpec::new(shell_element(node.tag(), expected_id))
        .classes(node.classes().iter().cloned())
        .attributes(node.attributes().clone())
        .behavior_hooks(
            node.behavior_hooks()
                .into_iter()
                .map(|hook| hook.to_element_hook()),
        );
    if let Some(role) = node.role() {
        spec = spec.role(role.to_owned());
    }
    spec
}

fn shell_element(tag: &str, expected_id: &str) -> Element {
    match tag {
        "div" => Element::Div,
        "main" => Element::Main,
        "section" => Element::Section,
        other => panic!("unsupported `{expected_id}` lab shell element `<{other}>`"),
    }
}

fn html_whitespace_node(node: &HtmlNode) -> bool {
    node.is_text() && node.text().is_none_or(|text| text.trim().is_empty())
}

fn lab_root_source() -> Cow<'static, str> {
    #[cfg(debug_assertions)]
    {
        Cow::Owned(
            std::fs::read_to_string(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/ui_lab/html/lab-root.html"
            ))
            .expect("lab root HTML file should be readable"),
        )
    }
    #[cfg(not(debug_assertions))]
    {
        Cow::Borrowed(LAB_ROOT_HTML)
    }
}

fn lab_body_source() -> Cow<'static, str> {
    #[cfg(debug_assertions)]
    {
        Cow::Owned(
            std::fs::read_to_string(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/ui_lab/html/lab-body.html"
            ))
            .expect("lab body HTML file should be readable"),
        )
    }
    #[cfg(not(debug_assertions))]
    {
        Cow::Borrowed(LAB_BODY_HTML)
    }
}

fn stage_source() -> Cow<'static, str> {
    #[cfg(debug_assertions)]
    {
        Cow::Owned(
            std::fs::read_to_string(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/ui_lab/html/stage.html"
            ))
            .expect("lab stage HTML file should be readable"),
        )
    }
    #[cfg(not(debug_assertions))]
    {
        Cow::Borrowed(STAGE_HTML)
    }
}

fn debug_overlay_root_source() -> Cow<'static, str> {
    #[cfg(debug_assertions)]
    {
        Cow::Owned(
            std::fs::read_to_string(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/ui_lab/html/debug-overlay-root.html"
            ))
            .expect("lab debug overlay root HTML file should be readable"),
        )
    }
    #[cfg(not(debug_assertions))]
    {
        Cow::Borrowed(DEBUG_OVERLAY_ROOT_HTML)
    }
}

fn debug_overlay_source() -> Cow<'static, str> {
    #[cfg(debug_assertions)]
    {
        Cow::Owned(
            std::fs::read_to_string(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/ui_lab/html/debug-overlay.html"
            ))
            .expect("lab debug overlay HTML file should be readable"),
        )
    }
    #[cfg(not(debug_assertions))]
    {
        Cow::Borrowed(DEBUG_OVERLAY_HTML)
    }
}

fn debug_overlay_title_source() -> Cow<'static, str> {
    #[cfg(debug_assertions)]
    {
        Cow::Owned(
            std::fs::read_to_string(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/ui_lab/html/debug-overlay-title.html"
            ))
            .expect("lab debug overlay title HTML file should be readable"),
        )
    }
    #[cfg(not(debug_assertions))]
    {
        Cow::Borrowed(DEBUG_OVERLAY_TITLE_HTML)
    }
}

fn topbar_source() -> Cow<'static, str> {
    #[cfg(debug_assertions)]
    {
        Cow::Owned(
            std::fs::read_to_string(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/ui_lab/html/topbar.html"
            ))
            .expect("lab topbar HTML file should be readable"),
        )
    }
    #[cfg(not(debug_assertions))]
    {
        Cow::Borrowed(TOPBAR_HTML)
    }
}

fn nav_source() -> Cow<'static, str> {
    #[cfg(debug_assertions)]
    {
        Cow::Owned(
            std::fs::read_to_string(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/ui_lab/html/nav.html"
            ))
            .expect("lab nav HTML file should be readable"),
        )
    }
    #[cfg(not(debug_assertions))]
    {
        Cow::Borrowed(NAV_HTML)
    }
}

fn interaction_shell_source() -> Cow<'static, str> {
    #[cfg(debug_assertions)]
    {
        Cow::Owned(
            std::fs::read_to_string(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/ui_lab/html/interaction-shell.html"
            ))
            .expect("lab interaction shell HTML file should be readable"),
        )
    }
    #[cfg(not(debug_assertions))]
    {
        Cow::Borrowed(INTERACTION_SHELL_HTML)
    }
}

fn interaction_cards_source() -> Cow<'static, str> {
    #[cfg(debug_assertions)]
    {
        Cow::Owned(
            std::fs::read_to_string(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/ui_lab/html/interaction-cards.html"
            ))
            .expect("lab interaction cards HTML file should be readable"),
        )
    }
    #[cfg(not(debug_assertions))]
    {
        Cow::Borrowed(INTERACTION_CARDS_HTML)
    }
}

fn interaction_controls_source() -> Cow<'static, str> {
    #[cfg(debug_assertions)]
    {
        Cow::Owned(
            std::fs::read_to_string(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/ui_lab/html/interaction-controls.html"
            ))
            .expect("lab interaction controls HTML file should be readable"),
        )
    }
    #[cfg(not(debug_assertions))]
    {
        Cow::Borrowed(INTERACTION_CONTROLS_HTML)
    }
}

fn interaction_loop_source() -> Cow<'static, str> {
    #[cfg(debug_assertions)]
    {
        Cow::Owned(
            std::fs::read_to_string(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/ui_lab/html/interaction-loop.html"
            ))
            .expect("lab interaction loop HTML file should be readable"),
        )
    }
    #[cfg(not(debug_assertions))]
    {
        Cow::Borrowed(INTERACTION_LOOP_HTML)
    }
}

fn draggable_shell_source() -> Cow<'static, str> {
    #[cfg(debug_assertions)]
    {
        Cow::Owned(
            std::fs::read_to_string(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/ui_lab/html/draggable-shell.html"
            ))
            .expect("lab draggable shell HTML file should be readable"),
        )
    }
    #[cfg(not(debug_assertions))]
    {
        Cow::Borrowed(DRAGGABLE_SHELL_HTML)
    }
}

fn nesting_source() -> Cow<'static, str> {
    #[cfg(debug_assertions)]
    {
        Cow::Owned(
            std::fs::read_to_string(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/ui_lab/html/nesting.html"
            ))
            .expect("lab nesting HTML file should be readable"),
        )
    }
    #[cfg(not(debug_assertions))]
    {
        Cow::Borrowed(NESTING_HTML)
    }
}

fn graph_source() -> Cow<'static, str> {
    #[cfg(debug_assertions)]
    {
        Cow::Owned(
            std::fs::read_to_string(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/ui_lab/html/graph.html"
            ))
            .expect("lab graph HTML file should be readable"),
        )
    }
    #[cfg(not(debug_assertions))]
    {
        Cow::Borrowed(GRAPH_HTML)
    }
}

fn structural_selectors_source() -> Cow<'static, str> {
    #[cfg(debug_assertions)]
    {
        Cow::Owned(
            std::fs::read_to_string(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/ui_lab/html/structural-selectors.html"
            ))
            .expect("lab structural selectors HTML file should be readable"),
        )
    }
    #[cfg(not(debug_assertions))]
    {
        Cow::Borrowed(STRUCTURAL_SELECTORS_HTML)
    }
}

fn text_specimens_source() -> Cow<'static, str> {
    #[cfg(debug_assertions)]
    {
        Cow::Owned(
            std::fs::read_to_string(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/ui_lab/html/text-specimens.html"
            ))
            .expect("lab text specimens HTML file should be readable"),
        )
    }
    #[cfg(not(debug_assertions))]
    {
        Cow::Borrowed(TEXT_SPECIMENS_HTML)
    }
}

fn table_source() -> Cow<'static, str> {
    #[cfg(debug_assertions)]
    {
        Cow::Owned(
            std::fs::read_to_string(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/ui_lab/html/table.html"
            ))
            .expect("lab table HTML file should be readable"),
        )
    }
    #[cfg(not(debug_assertions))]
    {
        Cow::Borrowed(TABLE_HTML)
    }
}

fn layout_source() -> Cow<'static, str> {
    #[cfg(debug_assertions)]
    {
        Cow::Owned(
            std::fs::read_to_string(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/ui_lab/html/layout.html"
            ))
            .expect("lab layout HTML file should be readable"),
        )
    }
    #[cfg(not(debug_assertions))]
    {
        Cow::Borrowed(LAYOUT_HTML)
    }
}

fn scrolling_source() -> Cow<'static, str> {
    #[cfg(debug_assertions)]
    {
        Cow::Owned(
            std::fs::read_to_string(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/ui_lab/html/scrolling.html"
            ))
            .expect("lab scrolling HTML file should be readable"),
        )
    }
    #[cfg(not(debug_assertions))]
    {
        Cow::Borrowed(SCROLLING_HTML)
    }
}

fn shadow_specimens_source() -> Cow<'static, str> {
    #[cfg(debug_assertions)]
    {
        Cow::Owned(
            std::fs::read_to_string(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/ui_lab/html/shadow-specimens.html"
            ))
            .expect("lab shadow specimens HTML file should be readable"),
        )
    }
    #[cfg(not(debug_assertions))]
    {
        Cow::Borrowed(SHADOW_SPECIMENS_HTML)
    }
}

fn styling_overview_source() -> Cow<'static, str> {
    #[cfg(debug_assertions)]
    {
        Cow::Owned(
            std::fs::read_to_string(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/ui_lab/html/styling-overview.html"
            ))
            .expect("lab styling overview HTML file should be readable"),
        )
    }
    #[cfg(not(debug_assertions))]
    {
        Cow::Borrowed(STYLING_OVERVIEW_HTML)
    }
}

fn animation_source() -> Cow<'static, str> {
    #[cfg(debug_assertions)]
    {
        Cow::Owned(
            std::fs::read_to_string(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/ui_lab/html/animation.html"
            ))
            .expect("lab animation HTML file should be readable"),
        )
    }
    #[cfg(not(debug_assertions))]
    {
        Cow::Borrowed(ANIMATION_HTML)
    }
}

fn floating_source() -> Cow<'static, str> {
    #[cfg(debug_assertions)]
    {
        Cow::Owned(
            std::fs::read_to_string(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/ui_lab/html/floating.html"
            ))
            .expect("lab floating HTML file should be readable"),
        )
    }
    #[cfg(not(debug_assertions))]
    {
        Cow::Borrowed(FLOATING_HTML)
    }
}
