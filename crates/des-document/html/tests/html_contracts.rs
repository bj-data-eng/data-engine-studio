use des_document::{
    DocumentCommandDispatchReport, DocumentCommandRegistry, DocumentEngine, DocumentInput,
    DocumentKey, DocumentProjection, Element, ElementBehaviorEvent, ElementId, Length, Point, Size,
    Style, StyleSheet,
};
use des_html::{
    HtmlBehaviorHook, HtmlDiagnosticCode, HtmlDocument, HtmlFile, HtmlNode, HtmlSet, HtmlStylesheet,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

struct TempHtmlPath {
    path: PathBuf,
}

impl TempHtmlPath {
    fn new(name: &str, extension: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "{name}-{}.{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system clock should be after epoch")
                .as_nanos(),
            extension
        ));
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempHtmlPath {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn node_text(node: &HtmlNode) -> Option<&str> {
    node.text.as_deref()
}

#[test]
fn html_document_parser_recovers_like_browser_html() {
    let html = HtmlDocument::parse_fragment("<section><p>One<p>Two</section>")
        .expect("browser-grade parser should recover malformed nesting");

    assert_eq!(html.children.len(), 1);
    assert_eq!(html.children[0].tag, "section");
    assert_eq!(html.children[0].children.len(), 2);
    assert_eq!(html.children[0].children[0].tag, "p");
    assert_eq!(html.children[0].children[0].text.as_deref(), Some("One"));
    assert_eq!(html.children[0].children[1].tag, "p");
    assert_eq!(html.children[0].children[1].text.as_deref(), Some("Two"));
}

#[test]
fn html_document_parser_handles_void_elements_entities_and_comments() {
    let html = HtmlDocument::parse_fragment(
        r#"
        <main id="app" class="shell primary" role="application" data-mode="demo" aria-label="Workspace">
          Hello &amp; welcome
          <!-- ignored comment -->
          <input id="search" value="A &quot;quote&quot;">
        </main>
        "#,
    )
    .expect("browser-grade parser should parse common HTML syntax");

    let main = &html.children[0];
    assert_eq!(main.tag, "main");
    assert_eq!(main.id.as_deref(), Some("app"));
    assert_eq!(main.classes, ["shell", "primary"]);
    assert_eq!(main.role.as_deref(), Some("application"));
    assert_eq!(
        main.attributes.get("data-mode").map(String::as_str),
        Some("demo")
    );
    assert_eq!(
        main.attributes.get("aria-label").map(String::as_str),
        Some("Workspace")
    );
    assert!(main.children.iter().any(|child| {
        child
            .text
            .as_deref()
            .is_some_and(|text| text.contains("Hello & welcome"))
    }));

    let input = main
        .children
        .iter()
        .find(|child| child.tag == "input")
        .expect("input should be a parsed void element");
    assert_eq!(input.id.as_deref(), Some("search"));
    assert_eq!(
        input.attributes.get("value").map(String::as_str),
        Some("A \"quote\"")
    );
}

#[test]
fn html_document_exposes_parsed_tree_queries() {
    let html = HtmlDocument::parse_fragment(
        r#"
        <main id="app" class="shell">
          <section id="primary" class="panel selected"><button id="run" class="control">Run</button></section>
          <section id="secondary" class="panel"><button id="stop" class="control danger">Stop</button></section>
        </main>
        "#,
    )
    .expect("HTML should parse");

    let app = html.find_by_id("app").expect("app id should be queryable");
    let primary = app
        .find_by_id("primary")
        .expect("subtree ids should be queryable");
    let controls = html.nodes_with_class("control");
    let sections = html.nodes_by_tag("section");

    assert!(app.has_class("shell"));
    assert!(primary.has_class("selected"));
    assert_eq!(
        html.first_by_tag("button").unwrap().id.as_deref(),
        Some("run")
    );
    assert_eq!(
        app.first_by_tag("button").unwrap().id.as_deref(),
        Some("run")
    );
    assert_eq!(
        app.nodes_by_tag("button")
            .into_iter()
            .map(|node| node.id.as_deref())
            .collect::<Vec<_>>(),
        [Some("run"), Some("stop")]
    );
    assert_eq!(
        sections
            .into_iter()
            .map(|node| node.id.as_deref())
            .collect::<Vec<_>>(),
        [Some("primary"), Some("secondary")]
    );
    assert_eq!(
        controls
            .into_iter()
            .map(|node| node.id.as_deref())
            .collect::<Vec<_>>(),
        [Some("run"), Some("stop")]
    );
    assert_eq!(
        app.nodes_with_class("danger")[0].id.as_deref(),
        Some("stop")
    );
}

#[test]
fn html_document_preserves_braces_as_text_not_logic() {
    let html = HtmlDocument::parse_fragment("<section>{if loading}<p>{title}</p>{/if}</section>")
        .expect("curly-brace text should be valid HTML text");

    let section = &html.children[0];
    assert_eq!(section.tag, "section");
    assert!(
        section
            .children
            .iter()
            .any(|child| node_text(child).is_some_and(|text| text.contains("{if loading}")))
    );
    assert_eq!(section.children[1].tag, "p");
    assert_eq!(section.children[1].text.as_deref(), Some("{title}"));
}

#[test]
fn html_document_parser_extracts_rust_behavior_hooks() {
    let html = HtmlDocument::parse_fragment(
        r#"<button id="open" class="primary" on:click="project.open" data-command:input="project.filter">Open</button>"#,
    )
    .expect("HTML should parse");

    let button = &html.children[0];
    assert_eq!(button.id.as_deref(), Some("open"));
    assert_eq!(button.behavior_hooks.len(), 2);
    assert_eq!(button.behavior_hooks[0].event, "click");
    assert_eq!(button.behavior_hooks[0].command, "project.open");
    assert_eq!(button.behavior_hooks[0].event(), "click");
    assert_eq!(button.behavior_hooks[0].command(), "project.open");
    assert!(button.behavior_hooks[0].has_command("project.open"));
    assert_eq!(
        button.behavior_hooks[0].intent(),
        Some(ElementBehaviorEvent::Click)
    );
    assert!(button.behavior_hooks[0].matches_intent(ElementBehaviorEvent::Click));
    assert!(button.behavior_hooks[0].is_click());
    assert!(!button.behavior_hooks[0].is_key_down());
    assert!(button.has_behavior_hook(ElementBehaviorEvent::Click, "project.open"));
    assert!(button.has_command_hook("project.filter"));
    assert_eq!(
        button.behavior_hooks_for(ElementBehaviorEvent::Click).len(),
        1
    );
    assert_eq!(
        button
            .first_behavior_hook_for(ElementBehaviorEvent::Click)
            .map(HtmlBehaviorHook::command),
        Some("project.open")
    );
    assert!(html.has_behavior_hook(ElementBehaviorEvent::Click, "project.open"));
    assert!(html.has_command_hook("project.filter"));
    assert_eq!(
        html.behavior_hooks_for(ElementBehaviorEvent::Click).len(),
        1
    );
    assert_eq!(
        html.first_behavior_hook_for(ElementBehaviorEvent::Click)
            .map(HtmlBehaviorHook::command),
        Some("project.open")
    );
    assert_eq!(button.behavior_hooks[1].event, "input");
    assert_eq!(button.behavior_hooks[1].command, "project.filter");
    assert_eq!(button.behavior_hooks[1].intent(), None);
}

#[test]
fn html_document_parser_reports_behavior_hook_diagnostics() {
    let html =
        HtmlDocument::parse_fragment(r#"<button on:click="" data-command:input="">Run</button>"#)
            .expect("HTML should parse with non-fatal diagnostics");

    assert!(html.children[0].behavior_hooks.is_empty());
    assert_eq!(html.diagnostics.len(), 2);
    assert_eq!(
        html.diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code)
            .collect::<Vec<_>>(),
        vec![
            HtmlDiagnosticCode::EmptyBehaviorCommand,
            HtmlDiagnosticCode::EmptyBehaviorCommand,
        ]
    );
    assert!(html.diagnostics.iter().all(|diagnostic| {
        diagnostic.tag.as_deref() == Some("button")
            && diagnostic.message.contains("missing a Rust command")
    }));
}

#[test]
fn html_document_parser_keeps_javascript_out_of_runtime() {
    let html = HtmlDocument::parse_fragment(
        r#"<section><script>alert("nope")</script><button onclick="alert(1)" on:click="run">Run</button></section>"#,
    )
    .expect("HTML should parse with JavaScript diagnostics");

    let section = &html.children[0];
    assert!(
        section.children.iter().all(|child| child.tag != "script"),
        "script elements should not be emitted into document HTML"
    );
    let button = section
        .children
        .iter()
        .find(|child| child.tag == "button")
        .unwrap();
    assert_eq!(button.behavior_hooks.len(), 1);
    assert_eq!(button.behavior_hooks[0].command, "run");
    assert_eq!(
        html.diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code)
            .collect::<Vec<_>>(),
        vec![
            HtmlDiagnosticCode::ScriptElementIgnored,
            HtmlDiagnosticCode::JavaScriptEventAttributeIgnored,
        ]
    );
}

#[test]
fn html_document_emits_typed_document_nodes_with_stable_ids() {
    let html = HtmlDocument::parse_fragment(
        r#"<main id="app" class="shell" role="application" data-workspace="demo" aria-label="Workspace"><button id="run" class="primary" on:click="run">Run</button><p>Ready</p></main>"#,
    )
    .expect("HTML should parse");
    let mut document = html
        .to_document(Size::new(320.0, 200.0))
        .expect("HTML should emit a document");
    let stylesheet = des_document::StyleSheet::parse_css(
        r#"
        .shell { width: 320px; height: 200px; }
        .primary { width: 80px; height: 32px; }
        "#,
    )
    .expect("CSS should parse");
    let mut engine = DocumentEngine::default();

    let output = engine.update(&mut document, &stylesheet);
    let app = output
        .snapshot()
        .find("app")
        .expect("main id should be retained");
    let run = output
        .snapshot()
        .find("run")
        .expect("button id should be retained");
    let generated_paragraph = output
        .snapshot()
        .find("html/p-0-1")
        .expect("missing ids should receive stable path ids");

    assert_eq!(app.element(), Element::Main);
    assert_eq!(app.role(), Some("application"));
    assert_eq!(app.attribute("data-workspace"), Some("demo"));
    assert_eq!(app.attribute("aria-label"), Some("Workspace"));
    assert_eq!(run.element(), Element::Button);
    assert!(run.interactive());
    assert_eq!(run.behavior_hooks()[0].event, "click");
    assert_eq!(run.behavior_hooks()[0].command, "run");
    assert_eq!(run.text(), Some("Run".to_owned()));
    assert_eq!(generated_paragraph.text(), Some("Ready".to_owned()));
    assert_eq!(run.rect().size.width, 80.0);
}

#[test]
fn html_document_maps_browser_boolean_state_attributes() {
    let html = HtmlDocument::parse_fragment(
        r#"
        <section id="form">
          <button id="save" disabled on:click="save">Save</button>
          <select id="mode"><option id="fast" selected>Fast</option></select>
          <input id="filter" autofocus value="ready">
        </section>
        "#,
    )
    .expect("HTML should parse browser boolean attributes");
    let mut document = html
        .to_document(Size::new(320.0, 180.0))
        .expect("HTML should emit a document");
    let mut engine = DocumentEngine::default();

    let output = engine.update(&mut document, &des_document::StyleSheet::new());
    let snapshot = output.snapshot();
    let save = snapshot.find("save").expect("button id should be retained");
    let fast = snapshot.find("fast").expect("option id should be retained");
    let filter = snapshot
        .find("filter")
        .expect("input id should be retained");

    assert!(save.disabled());
    assert!(!save.interactive());
    assert!(save.has_behavior_hook(ElementBehaviorEvent::Click, "save"));
    assert_eq!(save.attribute("disabled"), Some(""));
    assert!(fast.selected());
    assert_eq!(fast.attribute("selected"), Some(""));
    assert!(filter.focused());
    assert_eq!(filter.value(), Some("ready"));
    assert_eq!(snapshot.count_disabled(), 1);
    assert_eq!(snapshot.count_selected(), 1);
    assert_eq!(snapshot.count_focused(), 1);
}

#[test]
fn html_stylesheet_parses_html_and_css_together() {
    let bundle = HtmlStylesheet::parse_fragment(
        r#"<section id="panel" class="card">Panel</section>"#,
        r#".card { width: 144px; height: 48px; }"#,
    )
    .expect("HTML and CSS should compile together");
    let mut document = bundle
        .html
        .to_document(Size::new(240.0, 160.0))
        .expect("HTML should emit a document");
    let mut engine = DocumentEngine::default();

    let output = engine.update(&mut document, &bundle.stylesheet);
    let panel = output.snapshot().find("panel").unwrap();

    assert_eq!(panel.rect().size.width, 144.0);
    assert_eq!(panel.attribute("class"), None);
    assert_eq!(panel.text(), Some("Panel".to_owned()));
}

#[test]
fn html_stylesheet_parses_full_html_documents() {
    let bundle = HtmlStylesheet::parse(
        r#"
        <!doctype html>
        <html>
          <head><title>Ignored shell metadata</title></head>
          <body><main id="app" class="shell">App</main></body>
        </html>
        "#,
        r#".shell { width: 200px; height: 80px; }"#,
    )
    .expect("HTML document and CSS should compile together");
    let mut view = bundle
        .to_view(Size::new(320.0, 180.0))
        .expect("full HTML bundle should create a document view");

    let output = view.update();
    let app = output.snapshot().find("app").unwrap();

    assert_eq!(app.element(), Element::Main);
    assert_eq!(app.text(), Some("App".to_owned()));
    assert_eq!(app.rect().size.width, 200.0);
}

#[test]
fn html_stylesheet_can_create_ready_to_update_document_view() {
    let bundle = HtmlStylesheet::parse_fragment(
        r#"<button id="run" class="primary" on:click="run">Run</button>"#,
        r#".primary { width: 96px; height: 32px; }"#,
    )
    .expect("HTML and CSS should compile together");
    let mut view = bundle
        .to_view(Size::new(320.0, 180.0))
        .expect("HTML bundle should create a document view");

    let output = view.update_with_input(DocumentInput::primary_click(Point::new(8.0, 8.0)));
    let run = output.snapshot().find("run").unwrap();

    assert_eq!(run.rect().size.width, 96.0);
    assert!(run.interactive());
    assert_eq!(run.behavior_hooks()[0].command, "run");
    assert_eq!(output.commands()[0].command, "run");
}

#[test]
fn html_document_can_pair_with_css_without_manual_bundle_plumbing() {
    let html = HtmlDocument::parse_fragment(r#"<section id="panel" class="card">Panel</section>"#)
        .expect("HTML should parse");
    let bundle = html
        .clone()
        .with_css(r#".card { width: 144px; height: 48px; }"#)
        .expect("CSS should parse onto HTML");
    let typed_bundle = html.clone().with_stylesheet_if(
        StyleSheet::new().class("card", Style::default().radius(4.0)),
        true,
    );
    let skipped_typed_bundle = html.clone().with_stylesheet_if(
        StyleSheet::new().class("skipped", Style::default().radius(99.0)),
        false,
    );
    let conditional_bundle = html
        .clone()
        .with_css_if(true, r#".card { padding: 2px; }"#)
        .expect("conditional CSS should parse onto HTML");
    let skipped_conditional_bundle = html
        .clone()
        .with_css_if(false, r#".card { width: ; }"#)
        .expect("skipped conditional CSS should not parse");
    let forgiving_bundle = html
        .clone()
        .with_css_forgiving_if(
            true,
            r#".card { unknown-property: 1px; } .card { margin: 1px; }"#,
        )
        .expect("conditional forgiving CSS should parse onto HTML");
    let skipped_forgiving_bundle = html
        .clone()
        .with_css_forgiving_if(false, "/* unclosed")
        .expect("skipped forgiving CSS should not parse");
    let mut view = html
        .to_view_with_css(
            Size::new(240.0, 160.0),
            r#".card { width: 120px; height: 40px; }"#,
        )
        .expect("HTML and CSS should compose directly into a view");
    let mut typed_view = html
        .to_view_with_stylesheet_if(
            Size::new(240.0, 160.0),
            StyleSheet::new().class("card", Style::default().height(Length::Px(36.0))),
            true,
        )
        .expect("conditional typed stylesheet should compose directly into a view");
    let mut skipped_view = html
        .to_view_with_css_if(Size::new(240.0, 160.0), false, r#".card { width: ; }"#)
        .expect("skipped view CSS should not parse");
    let mut forgiving_view = html
        .to_view_with_css_forgiving_if(
            Size::new(240.0, 160.0),
            true,
            r#".card { unknown-property: 1px; } .card { width: 132px; }"#,
        )
        .expect("conditional forgiving view CSS should compose");

    let output = view.update();
    let typed_output = typed_view.update();
    let skipped_output = skipped_view.update();
    let forgiving_output = forgiving_view.update();
    let panel = output.snapshot().find("panel").unwrap();
    let typed_panel = typed_output.snapshot().find("panel").unwrap();
    let skipped_panel = skipped_output.snapshot().find("panel").unwrap();
    let forgiving_panel = forgiving_output.snapshot().find("panel").unwrap();

    assert_eq!(bundle.stylesheet.rule_count(), 1);
    assert_eq!(typed_bundle.stylesheet().rule_count(), 1);
    assert_eq!(skipped_typed_bundle.stylesheet().rule_count(), 0);
    assert_eq!(conditional_bundle.stylesheet().rule_count(), 1);
    assert_eq!(skipped_conditional_bundle.stylesheet().rule_count(), 0);
    assert!(forgiving_bundle.stylesheet().rule_count() >= 1);
    assert!(forgiving_bundle.stylesheet().has_rule_for_class("card"));
    assert_eq!(skipped_forgiving_bundle.stylesheet().rule_count(), 0);
    assert_eq!(panel.rect().size.width, 120.0);
    assert_eq!(panel.rect().size.height, 40.0);
    assert_eq!(typed_panel.rect().size.height, 36.0);
    assert_eq!(skipped_panel.element(), Element::Section);
    assert_eq!(forgiving_panel.rect().size.width, 132.0);
}

#[test]
fn html_css_entry_points_can_recover_like_browser_stylesheets() {
    let html = HtmlDocument::parse_fragment(r#"<section id="panel" class="card">Panel</section>"#)
        .expect("HTML should parse");
    let css = r#"
        .broken {
          color: rgb(10, 20, );
        }
        .card { width: 144px; height: 48px; }
    "#;
    let bundle = html
        .clone()
        .with_css_forgiving(css)
        .expect("forgiving CSS should recover valid rules");
    let mut view = html
        .to_view_with_css_forgiving(Size::new(240.0, 160.0), css)
        .expect("forgiving CSS should compose directly into a view");

    let output = view.update();
    let panel = output.snapshot().find("panel").unwrap();

    assert!(bundle.stylesheet().rule_count() >= 1);
    assert_eq!(panel.rect().size.width, 144.0);
    assert_eq!(panel.rect().size.height, 48.0);
}

#[test]
fn html_document_updates_with_css_and_collects_actions_directly() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum HtmlAction {
        Run,
        Filter,
    }

    let html = HtmlDocument::parse_fragment(
        r#"
        <section id="panel" class="card">
          <button id="run" data-command="project.run">Run</button>
          <input id="filter" autofocus on:keydown="project.filter">
        </section>
        "#,
    )
    .expect("HTML should parse");
    let css = r#"
        .card { width: 180px; height: 72px; }
        button { width: 80px; height: 28px; }
    "#;
    let registry = DocumentCommandRegistry::new()
        .bind("project.run", HtmlAction::Run)
        .bind_on(
            ElementBehaviorEvent::KeyDown,
            "project.filter",
            HtmlAction::Filter,
        );

    let output = html
        .update_with_css(Size::new(240.0, 160.0), css)
        .expect("HTML and CSS should resolve directly");
    let click_frame = html
        .update_with_input_actions_and_css(
            Size::new(240.0, 160.0),
            DocumentInput::primary_click(Point::new(8.0, 8.0)),
            css,
            &registry,
        )
        .expect("HTML and CSS should collect click actions directly");
    let forgiving_frame = html
        .update_with_input_actions_and_css_forgiving(
            Size::new(240.0, 160.0),
            DocumentInput::key_down(DocumentKey::Enter),
            ".broken { width: ; } .card { width: 180px; height: 72px; }",
            &registry,
        )
        .expect("forgiving CSS should collect keyboard actions directly");
    let configured_click_frame = html
        .update_with_input_actions_and_css_with(
            Size::new(240.0, 160.0),
            DocumentInput::primary_click(Point::new(8.0, 8.0)),
            css,
            |commands| {
                commands.push("project.run", HtmlAction::Run);
                commands.push_key_down("project.filter", HtmlAction::Filter);
            },
        )
        .expect("HTML and CSS should configure typed actions for one update");
    let configured_forgiving_frame = html
        .update_with_input_actions_and_css_forgiving_with(
            Size::new(240.0, 160.0),
            DocumentInput::key_down(DocumentKey::Enter),
            ".broken { width: ; } .card { width: 180px; height: 72px; }",
            |commands| {
                commands.push("project.run", HtmlAction::Run);
                commands.push_key_down("project.filter", HtmlAction::Filter);
            },
        )
        .expect("forgiving CSS should configure typed actions for one update");
    let mut surface = html
        .to_action_surface_with_css(Size::new(240.0, 160.0), css, |commands| {
            commands.push("project.run", HtmlAction::Run);
            commands.push_key_down("project.filter", HtmlAction::Filter);
        })
        .expect("HTML and CSS should create a typed action surface directly");
    let surface_click =
        surface.update_with_input_actions(DocumentInput::primary_click(Point::new(8.0, 8.0)));
    let mut mapped_surface = html
        .to_action_surface_with_css_and_actions(
            Size::new(240.0, 160.0),
            ".card { width: 182px; height: 72px; } button { width: 82px; height: 28px; }",
            [
                ("project.run", HtmlAction::Run),
                ("project.filter", HtmlAction::Filter),
            ],
        )
        .expect("HTML and CSS should create a mapped action surface directly");
    let mapped_surface_click = mapped_surface
        .update_with_input_actions(DocumentInput::primary_click(Point::new(8.0, 8.0)));
    let mut intent_surface = html
        .to_action_surface_with_css_and_intent_actions(
            Size::new(240.0, 160.0),
            ".card { width: 183px; height: 72px; }",
            [
                (ElementBehaviorEvent::Click, "project.run", HtmlAction::Run),
                (
                    ElementBehaviorEvent::KeyDown,
                    "project.filter",
                    HtmlAction::Filter,
                ),
            ],
        )
        .expect("HTML and CSS should create an intent-mapped action surface directly");
    let intent_surface_key =
        intent_surface.update_with_input_actions(DocumentInput::key_down(DocumentKey::Enter));
    let mut skipped_surface = html
        .to_action_surface_with_css_if(
            Size::new(240.0, 160.0),
            false,
            ".card { width: ; }",
            |commands| {
                commands.push("project.run", HtmlAction::Run);
            },
        )
        .expect("skipped action-surface CSS should not parse");
    let skipped_surface_click = skipped_surface
        .update_with_input_actions(DocumentInput::primary_click(Point::new(8.0, 8.0)));
    let mut forgiving_surface = html
        .to_action_surface_with_css_forgiving(
            Size::new(240.0, 160.0),
            ".broken { width: ; } .card { width: 180px; height: 72px; }",
            |commands| {
                commands.push("project.run", HtmlAction::Run);
                commands.push_key_down("project.filter", HtmlAction::Filter);
            },
        )
        .expect("forgiving CSS should create a typed action surface directly");
    let surface_key =
        forgiving_surface.update_with_input_actions(DocumentInput::key_down(DocumentKey::Enter));
    let mut forgiving_mapped_surface = html
        .to_action_surface_with_css_forgiving_and_actions(
            Size::new(240.0, 160.0),
            ".broken { width: ; } .card { width: 184px; height: 72px; }",
            [
                ("project.run", HtmlAction::Run),
                ("project.filter", HtmlAction::Filter),
            ],
        )
        .expect("forgiving CSS should create a mapped action surface directly");
    let forgiving_mapped_surface_click = forgiving_mapped_surface
        .update_with_input_actions(DocumentInput::primary_click(Point::new(8.0, 8.0)));
    let mut forgiving_intent_surface = html
        .to_action_surface_with_css_forgiving_and_intent_actions(
            Size::new(240.0, 160.0),
            ".broken { width: ; } .card { width: 185px; height: 72px; }",
            [
                (ElementBehaviorEvent::Click, "project.run", HtmlAction::Run),
                (
                    ElementBehaviorEvent::KeyDown,
                    "project.filter",
                    HtmlAction::Filter,
                ),
            ],
        )
        .expect("forgiving CSS should create an intent-mapped action surface directly");
    let forgiving_intent_surface_key = forgiving_intent_surface
        .update_with_input_actions(DocumentInput::key_down(DocumentKey::Enter));
    let mut skipped_forgiving_surface = html
        .to_action_surface_with_css_forgiving_if(
            Size::new(240.0, 160.0),
            false,
            "/* unclosed",
            |commands| {
                commands.push("project.run", HtmlAction::Run);
            },
        )
        .expect("skipped forgiving action-surface CSS should not parse");
    let skipped_forgiving_surface_click = skipped_forgiving_surface
        .update_with_input_actions(DocumentInput::primary_click(Point::new(8.0, 8.0)));
    let mut dispatched = Vec::new();
    let (dispatch_frame, dispatch_report) = html
        .update_with_input_and_css_and_dispatch(
            Size::new(240.0, 160.0),
            DocumentInput::primary_click(Point::new(8.0, 8.0)),
            css,
            &registry,
            |action| {
                dispatched.push(*action.action());
            },
        )
        .expect("HTML and CSS should dispatch typed actions directly");
    let mut dispatched_values = Vec::new();
    let (value_dispatch_frame, value_dispatch_report) = html
        .update_with_input_and_css_and_dispatch_action_values(
            Size::new(240.0, 160.0),
            DocumentInput::primary_click(Point::new(8.0, 8.0)),
            css,
            &registry,
            |action| dispatched_values.push(*action),
        )
        .expect("HTML and CSS should dispatch typed action values directly");
    let mut configured_dispatched = Vec::new();
    let (configured_dispatch_frame, configured_dispatch_report) = html
        .update_with_input_and_css_and_dispatch_with(
            Size::new(240.0, 160.0),
            DocumentInput::primary_click(Point::new(8.0, 8.0)),
            css,
            |commands| {
                commands.push("project.run", HtmlAction::Run);
                commands.push_key_down("project.filter", HtmlAction::Filter);
            },
            |action| {
                configured_dispatched.push(*action.action());
            },
        )
        .expect("HTML and CSS should configure and dispatch typed actions directly");
    let mut configured_dispatched_values = Vec::new();
    let (configured_value_dispatch_frame, configured_value_dispatch_report) = html
        .update_with_input_and_css_and_dispatch_action_values_with(
            Size::new(240.0, 160.0),
            DocumentInput::primary_click(Point::new(8.0, 8.0)),
            css,
            |commands| {
                commands.push("project.run", HtmlAction::Run);
                commands.push_key_down("project.filter", HtmlAction::Filter);
            },
            |action| configured_dispatched_values.push(*action),
        )
        .expect("HTML and CSS should configure and dispatch typed action values directly");
    let mut forgiving_dispatched = Vec::new();
    let (forgiving_dispatch_frame, forgiving_dispatch_report) = html
        .update_with_input_and_css_forgiving_and_dispatch(
            Size::new(240.0, 160.0),
            DocumentInput::key_down(DocumentKey::Enter),
            ".broken { width: ; } .card { width: 180px; height: 72px; }",
            &registry,
            |action| {
                forgiving_dispatched.push(*action.action());
            },
        )
        .expect("forgiving CSS should dispatch typed actions directly");
    let mut forgiving_dispatched_values = Vec::new();
    let (forgiving_value_dispatch_frame, forgiving_value_dispatch_report) = html
        .update_with_input_and_css_forgiving_and_dispatch_action_values(
            Size::new(240.0, 160.0),
            DocumentInput::key_down(DocumentKey::Enter),
            ".broken { width: ; } .card { width: 180px; height: 72px; }",
            &registry,
            |action| forgiving_dispatched_values.push(*action),
        )
        .expect("forgiving CSS should dispatch typed action values directly");
    let mut forgiving_configured_dispatched_values = Vec::new();
    let (forgiving_configured_value_dispatch_frame, forgiving_configured_value_dispatch_report) =
        html.update_with_input_and_css_forgiving_and_dispatch_action_values_with(
            Size::new(240.0, 160.0),
            DocumentInput::key_down(DocumentKey::Enter),
            ".broken { width: ; } .card { width: 180px; height: 72px; }",
            |commands| {
                commands.push("project.run", HtmlAction::Run);
                commands.push_key_down("project.filter", HtmlAction::Filter);
            },
            |action| forgiving_configured_dispatched_values.push(*action),
        )
        .expect("forgiving CSS should configure and dispatch typed action values directly");

    assert_eq!(
        output.snapshot().find("panel").unwrap().rect().size.width,
        180.0
    );
    assert_eq!(
        click_frame
            .output()
            .snapshot()
            .find("run")
            .unwrap()
            .rect()
            .size
            .width,
        80.0
    );
    assert!(click_frame.contains_action(&HtmlAction::Run));
    assert!(forgiving_frame.contains_action(&HtmlAction::Filter));
    assert!(configured_click_frame.contains_action(&HtmlAction::Run));
    assert!(configured_forgiving_frame.contains_action(&HtmlAction::Filter));
    assert!(surface_click.contains_action(&HtmlAction::Run));
    assert!(mapped_surface_click.contains_action(&HtmlAction::Run));
    assert_eq!(
        mapped_surface_click
            .output()
            .snapshot()
            .find("panel")
            .unwrap()
            .rect()
            .size
            .width,
        182.0
    );
    assert!(intent_surface_key.contains_key_down_action(&HtmlAction::Filter));
    assert_eq!(
        intent_surface_key
            .output()
            .snapshot()
            .find("panel")
            .unwrap()
            .rect()
            .size
            .width,
        183.0
    );
    assert!(surface_key.contains_action(&HtmlAction::Filter));
    assert!(forgiving_mapped_surface_click.contains_action(&HtmlAction::Run));
    assert_eq!(
        forgiving_mapped_surface_click
            .output()
            .snapshot()
            .find("panel")
            .unwrap()
            .rect()
            .size
            .width,
        184.0
    );
    assert!(forgiving_intent_surface_key.contains_key_down_action(&HtmlAction::Filter));
    assert_eq!(
        forgiving_intent_surface_key
            .output()
            .snapshot()
            .find("panel")
            .unwrap()
            .rect()
            .size
            .width,
        185.0
    );
    assert!(skipped_surface_click.contains_action(&HtmlAction::Run));
    assert!(skipped_forgiving_surface_click.contains_action(&HtmlAction::Run));
    assert_eq!(
        skipped_surface_click
            .output()
            .snapshot()
            .find("panel")
            .unwrap()
            .element(),
        Element::Section
    );
    assert!(dispatch_frame.contains_clicked_action(&HtmlAction::Run));
    assert_eq!(dispatch_report, DocumentCommandDispatchReport::new(1, 1, 0));
    assert_eq!(dispatched, vec![HtmlAction::Run]);
    assert!(value_dispatch_frame.contains_clicked_action(&HtmlAction::Run));
    assert_eq!(
        value_dispatch_report,
        DocumentCommandDispatchReport::new(1, 1, 0)
    );
    assert_eq!(dispatched_values, vec![HtmlAction::Run]);
    assert!(configured_dispatch_frame.contains_action(&HtmlAction::Run));
    assert_eq!(
        configured_dispatch_report,
        DocumentCommandDispatchReport::new(1, 1, 0)
    );
    assert_eq!(configured_dispatched, vec![HtmlAction::Run]);
    assert!(configured_value_dispatch_frame.contains_action(&HtmlAction::Run));
    assert_eq!(
        configured_value_dispatch_report,
        DocumentCommandDispatchReport::new(1, 1, 0)
    );
    assert_eq!(configured_dispatched_values, vec![HtmlAction::Run]);
    assert!(forgiving_dispatch_frame.contains_action(&HtmlAction::Filter));
    assert_eq!(
        forgiving_dispatch_report,
        DocumentCommandDispatchReport::new(1, 1, 0)
    );
    assert_eq!(forgiving_dispatched, vec![HtmlAction::Filter]);
    assert!(forgiving_value_dispatch_frame.contains_action(&HtmlAction::Filter));
    assert_eq!(
        forgiving_value_dispatch_report,
        DocumentCommandDispatchReport::new(1, 1, 0)
    );
    assert_eq!(forgiving_dispatched_values, vec![HtmlAction::Filter]);
    assert!(forgiving_configured_value_dispatch_frame.contains_action(&HtmlAction::Filter));
    assert_eq!(
        forgiving_configured_value_dispatch_report,
        DocumentCommandDispatchReport::new(1, 1, 0)
    );
    assert_eq!(
        forgiving_configured_dispatched_values,
        vec![HtmlAction::Filter]
    );
}

#[test]
fn html_stylesheet_forgiving_constructors_compile_author_assets() {
    let bundle = HtmlStylesheet::parse_fragment_forgiving(
        r#"<button id="run" class="primary" on:click="run">Run</button>"#,
        r#"
        .discard-me { width: ; }
        .primary { width: 96px; height: 32px; }
        "#,
    )
    .expect("fragment and forgiving CSS should compile together");
    let document_bundle = HtmlStylesheet::parse_forgiving(
        r#"<html><body><main id="app" class="shell">App</main></body></html>"#,
        r#".bad { width: ; } .shell { width: 200px; height: 80px; }"#,
    )
    .expect("document and forgiving CSS should compile together");

    let mut view = bundle
        .to_view(Size::new(320.0, 180.0))
        .expect("bundle should create a view");
    let output = view.update_with_input(DocumentInput::primary_click(Point::new(8.0, 8.0)));
    let run = output.snapshot().find("run").unwrap();
    let mut document_view = document_bundle
        .into_view(Size::new(320.0, 180.0))
        .expect("document bundle should create a view");
    let document_output = document_view.update();

    assert!(bundle.stylesheet().rule_count() >= 1);
    assert_eq!(run.rect().size.width, 96.0);
    assert_eq!(output.commands()[0].command, "run");
    assert_eq!(
        document_output
            .snapshot()
            .find("app")
            .unwrap()
            .rect()
            .size
            .width,
        200.0
    );
}

#[test]
fn html_document_can_create_ready_to_update_document_view_without_css() {
    let html = HtmlDocument::parse_fragment(
        r#"<main id="app" class="shell compact" data-workspace="demo" aria-label="Workspace"><button id="run" on:click="project.run">Run</button></main>"#,
    )
    .expect("HTML should parse");
    let mut view = html
        .to_view(Size::new(320.0, 180.0))
        .expect("HTML document should create a document view");

    let output = view.update_with_input(DocumentInput::primary_click(Point::new(0.0, 0.0)));
    let app = output.snapshot().find("app").unwrap();
    let run = output.snapshot().find("run").unwrap();

    assert!(app.has_all_classes(["shell", "compact"]));
    assert_eq!(app.data("workspace"), Some("demo"));
    assert_eq!(app.aria("label"), Some("Workspace"));
    assert!(run.interactive());
    assert_eq!(output.commands()[0].command, "project.run");
}

#[test]
fn html_document_updates_and_collects_actions_without_css_plumbing() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum HtmlAction {
        Run,
        Search,
    }

    let html = HtmlDocument::parse_fragment(
        r#"
        <main id="app">
          <button id="run" on:click="project.run">Run</button>
          <input id="search" autofocus on:keydown="project.search">
        </main>
        "#,
    )
    .expect("HTML should parse");
    let registry = DocumentCommandRegistry::new()
        .bind_click("project.run", HtmlAction::Run)
        .bind_key_down("project.search", HtmlAction::Search);

    let output = html
        .update(Size::new(320.0, 180.0))
        .expect("HTML should resolve without CSS");
    let click_frame = html
        .update_with_input_actions(
            Size::new(320.0, 180.0),
            DocumentInput::primary_click(Point::new(8.0, 8.0)),
            &registry,
        )
        .expect("HTML should collect click actions without CSS");
    let key_frame = html
        .update_actions(Size::new(320.0, 180.0), &registry)
        .expect("HTML should collect actions directly after update");
    let configured_key_frame = html
        .update_with_input_actions_with(
            Size::new(320.0, 180.0),
            DocumentInput::key_down(DocumentKey::Enter),
            |commands| {
                commands.push_click("project.run", HtmlAction::Run);
                commands.push_key_down("project.search", HtmlAction::Search);
            },
        )
        .expect("HTML should configure typed actions for one update");
    let configured_empty_frame = html
        .update_actions_with(Size::new(320.0, 180.0), |commands| {
            commands.push_click("project.run", HtmlAction::Run);
        })
        .expect("HTML should configure typed actions for a no-input update");
    let key_input_frame = html
        .update_with_input_actions(
            Size::new(320.0, 180.0),
            DocumentInput::key_down(DocumentKey::Enter),
            &registry,
        )
        .expect("HTML should collect keyboard actions without CSS");
    let mut dispatched = Vec::new();
    let (dispatch_frame, dispatch_report) = html
        .update_with_input_and_dispatch(
            Size::new(320.0, 180.0),
            DocumentInput::primary_click(Point::new(8.0, 8.0)),
            &registry,
            |action| {
                dispatched.push(*action.action());
            },
        )
        .expect("HTML should dispatch typed actions without CSS");
    let mut dispatched_values = Vec::new();
    let (value_dispatch_frame, value_dispatch_report) = html
        .update_with_input_and_dispatch_action_values(
            Size::new(320.0, 180.0),
            DocumentInput::primary_click(Point::new(8.0, 8.0)),
            &registry,
            |action| dispatched_values.push(*action),
        )
        .expect("HTML should dispatch typed action values without CSS");
    let mut configured_dispatched = Vec::new();
    let (configured_dispatch_frame, configured_dispatch_report) = html
        .update_with_input_and_dispatch_with(
            Size::new(320.0, 180.0),
            DocumentInput::key_down(DocumentKey::Enter),
            |commands| {
                commands.push_click("project.run", HtmlAction::Run);
                commands.push_key_down("project.search", HtmlAction::Search);
            },
            |action| {
                configured_dispatched.push(*action.action());
            },
        )
        .expect("HTML should configure and dispatch typed actions without CSS");
    let mut configured_dispatched_values = Vec::new();
    let (configured_value_dispatch_frame, configured_value_dispatch_report) = html
        .update_with_input_and_dispatch_action_values_with(
            Size::new(320.0, 180.0),
            DocumentInput::key_down(DocumentKey::Enter),
            |commands| {
                commands.push_click("project.run", HtmlAction::Run);
                commands.push_key_down("project.search", HtmlAction::Search);
            },
            |action| configured_dispatched_values.push(*action),
        )
        .expect("HTML should configure and dispatch typed action values without CSS");
    let input_output = html
        .update_with_input(
            Size::new(320.0, 180.0),
            DocumentInput::primary_click(Point::new(8.0, 8.0)),
        )
        .expect("HTML should route input without CSS");

    assert!(output.snapshot().find("app").is_some());
    assert!(input_output.was_clicked("run"));
    assert!(click_frame.contains_action(&HtmlAction::Run));
    assert!(key_input_frame.contains_action(&HtmlAction::Search));
    assert!(configured_key_frame.contains_action(&HtmlAction::Search));
    assert!(configured_empty_frame.is_empty());
    assert!(key_frame.is_empty());
    assert!(dispatch_frame.contains_action(&HtmlAction::Run));
    assert_eq!(dispatch_report, DocumentCommandDispatchReport::new(1, 1, 0));
    assert_eq!(dispatched, vec![HtmlAction::Run]);
    assert!(value_dispatch_frame.contains_action(&HtmlAction::Run));
    assert_eq!(
        value_dispatch_report,
        DocumentCommandDispatchReport::new(1, 1, 0)
    );
    assert_eq!(dispatched_values, vec![HtmlAction::Run]);
    assert!(configured_dispatch_frame.contains_action(&HtmlAction::Search));
    assert_eq!(
        configured_dispatch_report,
        DocumentCommandDispatchReport::new(1, 1, 0)
    );
    assert_eq!(configured_dispatched, vec![HtmlAction::Search]);
    assert!(configured_value_dispatch_frame.contains_action(&HtmlAction::Search));
    assert_eq!(
        configured_value_dispatch_report,
        DocumentCommandDispatchReport::new(1, 1, 0)
    );
    assert_eq!(configured_dispatched_values, vec![HtmlAction::Search]);
}

#[test]
fn html_document_projects_state_without_css_plumbing() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum HtmlAction {
        Run,
    }

    let html = HtmlDocument::parse_fragment(
        r#"
        <main id="app">
          <button id="run" on:click="project.run">Run</button>
        </main>
        "#,
    )
    .expect("HTML should parse");
    let projection = DocumentProjection::new()
        .set_text("run", "Ready")
        .add_class("run", "is-ready")
        .set_data("app", "state", "ready");
    let registry = DocumentCommandRegistry::new().bind_click("project.run", HtmlAction::Run);

    let (report, output) = html
        .update_with_projection(Size::new(320.0, 180.0), &projection)
        .expect("HTML document should project retained state before update");
    let run = output.snapshot().find("run").unwrap();

    assert_eq!(report.operations, 3);
    assert_eq!(report.changed, 3);
    assert_eq!(
        output.snapshot().find("app").unwrap().data("state"),
        Some("ready")
    );
    assert_eq!(run.text(), Some("Ready".to_owned()));
    assert!(run.has_class("is-ready"));

    let (mapped_report, mapped_frame) = html
        .update_with_input_projected_with_and_actions(
            Size::new(320.0, 180.0),
            DocumentInput::primary_click(Point::new(8.0, 8.0)),
            |projection| {
                projection
                    .element("run")
                    .text("Running")
                    .add_class("is-ready");
            },
            [("project.run", HtmlAction::Run)],
        )
        .expect("HTML document should project state and map actions without CSS");
    let mapped_run = mapped_frame.output().snapshot().find("run").unwrap();

    assert_eq!(mapped_report.operations, 2);
    assert_eq!(mapped_report.changed, 2);
    assert_eq!(mapped_run.text(), Some("Running".to_owned()));
    assert!(mapped_frame.contains_clicked_action(&HtmlAction::Run));

    let mut dispatched = Vec::new();
    let (dispatch_report, dispatch_frame, action_report) = html
        .update_with_input_projection_and_dispatch(
            Size::new(320.0, 180.0),
            DocumentInput::primary_click(Point::new(8.0, 8.0)),
            &projection,
            &registry,
            |action| {
                dispatched.push(*action.action());
            },
        )
        .expect("HTML document should project state and dispatch actions without CSS");
    let dispatch_run = dispatch_frame.output().snapshot().find("run").unwrap();
    let mut value_dispatched = Vec::new();
    let (value_dispatch_report, value_dispatch_frame, value_action_report) = html
        .update_with_input_projection_and_dispatch_action_values(
            Size::new(320.0, 180.0),
            DocumentInput::primary_click(Point::new(8.0, 8.0)),
            &projection,
            &registry,
            |action| value_dispatched.push(*action),
        )
        .expect("HTML document should project state and dispatch action values without CSS");
    let value_dispatch_run = value_dispatch_frame
        .output()
        .snapshot()
        .find("run")
        .unwrap();
    let mut configured_dispatched = Vec::new();
    let (configured_report, configured_frame, configured_action_report) = html
        .update_with_input_projected_with_and_dispatch_with(
            Size::new(320.0, 180.0),
            DocumentInput::primary_click(Point::new(8.0, 8.0)),
            |projection| {
                projection.element("run").text("Configured");
            },
            |commands| {
                commands.push_click("project.run", HtmlAction::Run);
            },
            |action| {
                configured_dispatched.push(*action.action());
            },
        )
        .expect("HTML document should build projection, configure actions, and dispatch");
    let configured_run = configured_frame.output().snapshot().find("run").unwrap();
    let mut configured_value_dispatched = Vec::new();
    let (configured_value_report, configured_value_frame, configured_value_action_report) = html
        .update_with_input_projected_with_and_dispatch_action_values_with(
            Size::new(320.0, 180.0),
            DocumentInput::primary_click(Point::new(8.0, 8.0)),
            |projection| {
                projection.element("run").text("Configured value");
            },
            |commands| {
                commands.push_click("project.run", HtmlAction::Run);
            },
            |action| configured_value_dispatched.push(*action),
        )
        .expect("HTML document should build projection, configure actions, and dispatch values");
    let configured_value_run = configured_value_frame
        .output()
        .snapshot()
        .find("run")
        .unwrap();

    assert_eq!(dispatch_report.operations, 3);
    assert_eq!(dispatch_report.changed, 3);
    assert_eq!(dispatch_run.text(), Some("Ready".to_owned()));
    assert_eq!(action_report, DocumentCommandDispatchReport::new(1, 1, 0));
    assert_eq!(dispatched, vec![HtmlAction::Run]);
    assert!(dispatch_frame.contains_clicked_action(&HtmlAction::Run));
    assert_eq!(value_dispatch_report.operations, 3);
    assert_eq!(value_dispatch_report.changed, 3);
    assert_eq!(value_dispatch_run.text(), Some("Ready".to_owned()));
    assert_eq!(
        value_action_report,
        DocumentCommandDispatchReport::new(1, 1, 0)
    );
    assert_eq!(value_dispatched, vec![HtmlAction::Run]);
    assert!(value_dispatch_frame.contains_clicked_action(&HtmlAction::Run));
    assert_eq!(configured_report.operations, 1);
    assert_eq!(configured_report.changed, 1);
    assert_eq!(configured_run.text(), Some("Configured".to_owned()));
    assert_eq!(
        configured_action_report,
        DocumentCommandDispatchReport::new(1, 1, 0)
    );
    assert_eq!(configured_dispatched, vec![HtmlAction::Run]);
    assert!(configured_frame.contains_clicked_action(&HtmlAction::Run));
    assert_eq!(configured_value_report.operations, 1);
    assert_eq!(configured_value_report.changed, 1);
    assert_eq!(
        configured_value_run.text(),
        Some("Configured value".to_owned())
    );
    assert_eq!(
        configured_value_action_report,
        DocumentCommandDispatchReport::new(1, 1, 0)
    );
    assert_eq!(configured_value_dispatched, vec![HtmlAction::Run]);
    assert!(configured_value_frame.contains_clicked_action(&HtmlAction::Run));

    let (surface_report, mut surface) = html
        .to_action_surface_projected_with_actions(
            Size::new(320.0, 180.0),
            |projection| {
                projection.element("run").text("Ready");
            },
            [("project.run", HtmlAction::Run)],
        )
        .expect("HTML document should create projected mapped action surfaces without CSS");
    let surface_frame =
        surface.update_with_input_actions(DocumentInput::primary_click(Point::new(8.0, 8.0)));

    assert_eq!(surface_report.operations, 1);
    assert_eq!(surface_report.changed, 1);
    assert!(surface_frame.contains_clicked_action(&HtmlAction::Run));
    assert_eq!(
        surface_frame
            .output()
            .snapshot()
            .find("run")
            .unwrap()
            .text(),
        Some("Ready".to_owned())
    );
}

#[test]
fn html_document_can_create_action_surfaces_without_css_plumbing() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum HtmlAction {
        Run,
        Menu,
    }

    let html = HtmlDocument::parse_fragment(
        r#"
        <main id="app">
          <button id="run" on:click="project.run" on:contextmenu="project.menu">Run</button>
        </main>
        "#,
    )
    .expect("HTML should parse");
    let registry = DocumentCommandRegistry::new().bind_click("project.run", HtmlAction::Run);
    let mut surface = html
        .to_action_surface(Size::new(320.0, 180.0), registry)
        .expect("HTML should create an action surface without CSS");
    let mut configured_surface = html
        .to_action_surface_with(Size::new(320.0, 180.0), |commands| {
            commands.push_click("project.run", HtmlAction::Run);
        })
        .expect("HTML should configure an action surface without CSS");
    let mut mapped_surface = html
        .to_action_surface_with_actions(
            Size::new(320.0, 180.0),
            [
                ("project.run", HtmlAction::Run),
                ("project.menu", HtmlAction::Menu),
            ],
        )
        .expect("HTML should map command names into an action surface");
    let stylesheet = des_document::StyleSheet::parse_css("#run { width: 96px; height: 32px; }")
        .expect("CSS should parse");
    let mut styled_surface = html
        .to_action_surface_with_stylesheet_and(Size::new(320.0, 180.0), stylesheet, |commands| {
            commands.push_click("project.run", HtmlAction::Run);
            commands.push_context_menu("project.menu", HtmlAction::Menu);
        })
        .expect("HTML should create an action surface with a typed stylesheet");
    let skipped_stylesheet = des_document::StyleSheet::new().id(
        "run",
        des_document::Style::default().width(des_document::Length::Px(999.0)),
    );
    let mut skipped_styled_surface = html
        .to_action_surface_with_stylesheet_if_and(
            Size::new(320.0, 180.0),
            skipped_stylesheet,
            false,
            |commands| {
                commands.push_click("project.run", HtmlAction::Run);
            },
        )
        .expect("skipped typed stylesheet should still create an action surface");
    let conditional_registry =
        DocumentCommandRegistry::new().bind_click("project.run", HtmlAction::Run);
    let mut conditional_styled_surface = html
        .to_action_surface_with_stylesheet_if(
            Size::new(320.0, 180.0),
            des_document::StyleSheet::new().id(
                "run",
                des_document::Style::default().width(des_document::Length::Px(104.0)),
            ),
            true,
            conditional_registry,
        )
        .expect("conditional typed stylesheet should create an action surface");
    let intent_html = HtmlDocument::parse_fragment(
        r#"<button id="shared" on:click="project.open" on:contextmenu="project.open">Open</button>"#,
    )
    .expect("HTML should parse shared command hooks");
    let mut intent_surface = intent_html
        .to_action_surface_with_intent_actions(
            Size::new(320.0, 180.0),
            [
                (ElementBehaviorEvent::Click, "project.open", HtmlAction::Run),
                (
                    ElementBehaviorEvent::ContextMenu,
                    "project.open",
                    HtmlAction::Menu,
                ),
            ],
        )
        .expect("HTML should map one command differently by event intent");

    let click_frame =
        surface.update_with_input_actions(DocumentInput::primary_click(Point::new(8.0, 8.0)));
    let configured_frame = configured_surface
        .update_with_input_actions(DocumentInput::primary_click(Point::new(8.0, 8.0)));
    let mapped_context_frame = mapped_surface
        .update_with_input_actions(DocumentInput::secondary_click(Point::new(8.0, 8.0)));
    let context_frame = styled_surface
        .update_with_input_actions(DocumentInput::secondary_click(Point::new(8.0, 8.0)));
    let skipped_styled_frame = skipped_styled_surface
        .update_with_input_actions(DocumentInput::primary_click(Point::new(8.0, 8.0)));
    let conditional_styled_frame = conditional_styled_surface
        .update_with_input_actions(DocumentInput::primary_click(Point::new(8.0, 8.0)));
    let intent_click_frame = intent_surface
        .update_with_input_actions(DocumentInput::primary_click(Point::new(8.0, 8.0)));
    let intent_context_frame = intent_surface
        .update_with_input_actions(DocumentInput::secondary_click(Point::new(8.0, 8.0)));
    let run = context_frame.output().snapshot().find("run").unwrap();

    assert!(click_frame.contains_action(&HtmlAction::Run));
    assert!(configured_frame.contains_action(&HtmlAction::Run));
    assert!(mapped_context_frame.contains_action(&HtmlAction::Menu));
    assert_eq!(mapped_surface.commands().bindings().len(), 2);
    assert!(intent_click_frame.contains_clicked_action(&HtmlAction::Run));
    assert!(intent_context_frame.contains_action_for_target_intent(
        "shared",
        ElementBehaviorEvent::ContextMenu,
        &HtmlAction::Menu
    ));
    assert_eq!(intent_surface.commands().bindings().len(), 2);
    assert!(context_frame.contains_action(&HtmlAction::Menu));
    assert_eq!(run.rect().size.width, 96.0);
    assert_eq!(styled_surface.commands().bindings().len(), 2);
    assert!(skipped_styled_frame.contains_action(&HtmlAction::Run));
    assert_ne!(
        skipped_styled_frame
            .output()
            .snapshot()
            .find("run")
            .unwrap()
            .rect()
            .size
            .width,
        999.0
    );
    assert!(conditional_styled_frame.contains_action(&HtmlAction::Run));
    assert_eq!(
        conditional_styled_frame
            .output()
            .snapshot()
            .find("run")
            .unwrap()
            .rect()
            .size
            .width,
        104.0
    );
}

#[test]
fn html_authored_commands_dispatch_to_typed_rust_actions() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum HtmlAction {
        Run,
        CommitByClick,
        CommitByKeyboard,
    }

    let bundle = HtmlStylesheet::parse_fragment(
        r#"
        <section id="panel">
          <button id="run" class="primary" data-command="project.run">Run</button>
          <button id="commit" autofocus data-command="project.commit" on:keydown="project.commit">Commit</button>
        </section>
        "#,
        r#".primary { width: 96px; height: 32px; }"#,
    )
    .expect("HTML and CSS should compile together");
    let mut view = bundle
        .to_view(Size::new(320.0, 180.0))
        .expect("HTML bundle should create a document view");
    let output = view.update_with_input(DocumentInput::primary_click(Point::new(8.0, 8.0)));
    let registry = DocumentCommandRegistry::new().bind("project.run", HtmlAction::Run);
    let mut actions = Vec::new();
    let report = registry.dispatch(&output, |command| {
        actions.push((command.target.clone(), *command.action));
    });

    assert_eq!(report.commands, 1);
    assert_eq!(report.handled, 1);
    assert_eq!(report.unhandled, 0);
    assert_eq!(actions, vec![(ElementId::new("run"), HtmlAction::Run)]);

    let hooks = bundle.behavior_hooks();
    assert_eq!(hooks.len(), 3);
    assert!(hooks.iter().any(|hook| {
        hook.command() == "project.commit" && hook.matches_intent(ElementBehaviorEvent::KeyDown)
    }));
    assert!(bundle.has_behavior_hook(ElementBehaviorEvent::Click, "project.run"));
    assert!(bundle.has_command_hook("project.commit"));
    assert_eq!(
        bundle
            .behavior_hooks_for(ElementBehaviorEvent::KeyDown)
            .len(),
        1
    );
    assert_eq!(
        bundle
            .first_behavior_hook_for(ElementBehaviorEvent::KeyDown)
            .map(HtmlBehaviorHook::command),
        Some("project.commit")
    );

    let mapped_registry = bundle.command_action_registry([
        ("project.run", HtmlAction::Run),
        ("project.commit", HtmlAction::CommitByKeyboard),
    ]);
    let mut pushed_mapped_registry = DocumentCommandRegistry::new();
    bundle.push_command_actions(
        &mut pushed_mapped_registry,
        [
            ("project.run", HtmlAction::Run),
            ("project.commit", HtmlAction::CommitByKeyboard),
        ],
    );
    let mapped_click_frame = bundle
        .update_with_input_actions(
            Size::new(320.0, 180.0),
            DocumentInput::primary_click(Point::new(8.0, 8.0)),
            &mapped_registry,
        )
        .expect("mapped HTML commands should route click actions");
    let mapped_key_frame = bundle
        .update_with_input_actions(
            Size::new(320.0, 180.0),
            DocumentInput::key_down(DocumentKey::Enter),
            &mapped_registry,
        )
        .expect("mapped HTML commands should route keyboard actions");

    assert_eq!(mapped_registry.bindings().len(), 3);
    assert_eq!(
        pushed_mapped_registry.bindings(),
        mapped_registry.bindings()
    );
    assert!(mapped_click_frame.contains_clicked_action(&HtmlAction::Run));
    assert!(mapped_key_frame.contains_key_down_action(&HtmlAction::CommitByKeyboard));

    let intent_registry = bundle.command_intent_action_registry([
        (ElementBehaviorEvent::Click, "project.run", HtmlAction::Run),
        (
            ElementBehaviorEvent::Click,
            "project.commit",
            HtmlAction::CommitByClick,
        ),
        (
            ElementBehaviorEvent::KeyDown,
            "project.commit",
            HtmlAction::CommitByKeyboard,
        ),
    ]);
    let mut pushed_intent_registry = DocumentCommandRegistry::new();
    bundle.push_command_intent_actions(
        &mut pushed_intent_registry,
        [
            (ElementBehaviorEvent::Click, "project.run", HtmlAction::Run),
            (
                ElementBehaviorEvent::Click,
                "project.commit",
                HtmlAction::CommitByClick,
            ),
            (
                ElementBehaviorEvent::KeyDown,
                "project.commit",
                HtmlAction::CommitByKeyboard,
            ),
        ],
    );
    let intent_click_frame = bundle
        .update_with_input_actions(
            Size::new(320.0, 180.0),
            DocumentInput::primary_click(Point::new(8.0, 32.0)),
            &intent_registry,
        )
        .expect("intent mapped HTML commands should route click actions");
    let intent_key_frame = bundle
        .update_with_input_actions(
            Size::new(320.0, 180.0),
            DocumentInput::key_down(DocumentKey::Enter),
            &intent_registry,
        )
        .expect("intent mapped HTML commands should route keyboard actions");

    assert_eq!(
        pushed_intent_registry.bindings(),
        intent_registry.bindings()
    );
    assert!(intent_click_frame.contains_clicked_action(&HtmlAction::CommitByClick));
    assert!(intent_key_frame.contains_key_down_action(&HtmlAction::CommitByKeyboard));
    assert!(!intent_click_frame.contains_clicked_action(&HtmlAction::CommitByKeyboard));

    let registry = bundle.command_registry(|hook| match (hook.command(), hook.intent()) {
        ("project.run", Some(ElementBehaviorEvent::Click)) => Some(HtmlAction::Run),
        ("project.commit", Some(ElementBehaviorEvent::Click)) => Some(HtmlAction::CommitByClick),
        ("project.commit", Some(ElementBehaviorEvent::KeyDown)) => {
            Some(HtmlAction::CommitByKeyboard)
        }
        _ => None,
    });
    let click_frame = bundle
        .update_with_input_actions(
            Size::new(320.0, 180.0),
            DocumentInput::primary_click(Point::new(8.0, 8.0)),
            &registry,
        )
        .expect("HTML-authored command registry should route click actions");
    let key_frame = bundle
        .update_with_input_actions(
            Size::new(320.0, 180.0),
            DocumentInput::key_down(DocumentKey::Enter),
            &registry,
        )
        .expect("HTML-authored command registry should route keyboard actions");

    assert!(click_frame.contains_clicked_action(&HtmlAction::Run));
    assert!(!click_frame.contains_action(&HtmlAction::CommitByKeyboard));
    assert!(key_frame.contains_key_down_action(&HtmlAction::CommitByKeyboard));
    assert!(!key_frame.contains_clicked_action(&HtmlAction::CommitByClick));
}

#[test]
fn html_stylesheet_updates_and_collects_typed_actions_through_one_front_door() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum HtmlAction {
        Run,
        Filter,
    }

    let bundle = HtmlStylesheet::parse_fragment(
        r#"
        <section id="panel" class="card">
          <button id="run" class="primary" data-command="project.run">Run</button>
          <input id="filter" value="active" autofocus on:keydown="project.filter">
        </section>
        "#,
        r#"
        .card { width: 220px; height: 96px; }
        .primary { width: 96px; height: 32px; }
        "#,
    )
    .expect("HTML and CSS should compile together");
    let registry = DocumentCommandRegistry::new()
        .bind("project.run", HtmlAction::Run)
        .bind_on(
            des_document::ElementBehaviorEvent::KeyDown,
            "project.filter",
            HtmlAction::Filter,
        );

    let output = bundle
        .update(Size::new(320.0, 180.0))
        .expect("HTML bundle should resolve directly");
    let click_frame = bundle
        .update_with_input_actions(
            Size::new(320.0, 180.0),
            DocumentInput::primary_click(Point::new(8.0, 8.0)),
            &registry,
        )
        .expect("HTML bundle should collect click actions directly");
    let key_frame = bundle
        .update_with_input_actions(
            Size::new(320.0, 180.0),
            DocumentInput::key_down(des_document::DocumentKey::Enter),
            &registry,
        )
        .expect("HTML bundle should collect keyboard actions directly");
    let configured_click_frame = bundle
        .update_with_input_actions_with(
            Size::new(320.0, 180.0),
            DocumentInput::primary_click(Point::new(8.0, 8.0)),
            |commands| {
                commands.push("project.run", HtmlAction::Run);
                commands.push_key_down("project.filter", HtmlAction::Filter);
            },
        )
        .expect("HTML bundle should configure typed actions for one update");
    let configured_empty_frame = bundle
        .update_actions_with(Size::new(320.0, 180.0), |commands| {
            commands.push("project.run", HtmlAction::Run);
        })
        .expect("HTML bundle should configure typed actions for a no-input update");
    let mut dispatched = Vec::new();
    let (dispatch_frame, dispatch_report) = bundle
        .update_with_input_and_dispatch(
            Size::new(320.0, 180.0),
            DocumentInput::primary_click(Point::new(8.0, 8.0)),
            &registry,
            |action| {
                dispatched.push(*action.action());
            },
        )
        .expect("HTML bundle should dispatch typed actions directly");
    let mut dispatched_values = Vec::new();
    let (value_dispatch_frame, value_dispatch_report) = bundle
        .update_with_input_and_dispatch_action_values(
            Size::new(320.0, 180.0),
            DocumentInput::primary_click(Point::new(8.0, 8.0)),
            &registry,
            |action| dispatched_values.push(*action),
        )
        .expect("HTML bundle should dispatch typed action values directly");
    let mut configured_dispatched = Vec::new();
    let (configured_dispatch_frame, configured_dispatch_report) = bundle
        .update_with_input_and_dispatch_with(
            Size::new(320.0, 180.0),
            DocumentInput::key_down(DocumentKey::Enter),
            |commands| {
                commands.push("project.run", HtmlAction::Run);
                commands.push_key_down("project.filter", HtmlAction::Filter);
            },
            |action| {
                configured_dispatched.push(*action.action());
            },
        )
        .expect("HTML bundle should configure and dispatch typed actions directly");
    let mut configured_dispatched_values = Vec::new();
    let (configured_value_dispatch_frame, configured_value_dispatch_report) = bundle
        .update_with_input_and_dispatch_action_values_with(
            Size::new(320.0, 180.0),
            DocumentInput::key_down(DocumentKey::Enter),
            |commands| {
                commands.push("project.run", HtmlAction::Run);
                commands.push_key_down("project.filter", HtmlAction::Filter);
            },
            |action| configured_dispatched_values.push(*action),
        )
        .expect("HTML bundle should configure and dispatch typed action values directly");

    assert_eq!(
        output.snapshot().find("panel").unwrap().rect().size.width,
        220.0
    );
    assert!(click_frame.contains_action(&HtmlAction::Run));
    assert!(key_frame.contains_action(&HtmlAction::Filter));
    assert!(configured_click_frame.contains_action(&HtmlAction::Run));
    assert!(configured_empty_frame.is_empty());
    assert!(dispatch_frame.contains_action(&HtmlAction::Run));
    assert_eq!(dispatch_report, DocumentCommandDispatchReport::new(1, 1, 0));
    assert_eq!(dispatched, vec![HtmlAction::Run]);
    assert!(value_dispatch_frame.contains_action(&HtmlAction::Run));
    assert_eq!(
        value_dispatch_report,
        DocumentCommandDispatchReport::new(1, 1, 0)
    );
    assert_eq!(dispatched_values, vec![HtmlAction::Run]);
    assert!(configured_dispatch_frame.contains_action(&HtmlAction::Filter));
    assert_eq!(
        configured_dispatch_report,
        DocumentCommandDispatchReport::new(1, 1, 0)
    );
    assert_eq!(configured_dispatched, vec![HtmlAction::Filter]);
    assert!(configured_value_dispatch_frame.contains_action(&HtmlAction::Filter));
    assert_eq!(
        configured_value_dispatch_report,
        DocumentCommandDispatchReport::new(1, 1, 0)
    );
    assert_eq!(configured_dispatched_values, vec![HtmlAction::Filter]);
    assert_eq!(
        click_frame
            .output()
            .snapshot()
            .find("run")
            .unwrap()
            .rect()
            .size
            .width,
        96.0
    );
    assert_eq!(
        key_frame
            .output()
            .snapshot()
            .find("filter")
            .unwrap()
            .value(),
        Some("active")
    );
}

#[test]
fn html_stylesheet_projects_app_state_through_one_front_door() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum HtmlAction {
        Run,
        Select,
    }

    let bundle = HtmlStylesheet::parse_fragment(
        r#"
        <section id="panel" class="card">
          <button id="run" class="primary" data-command="project.run">Run</button>
          <button id="select" class="secondary" on:click="project.select">Select</button>
        </section>
        "#,
        r#"
        .card { width: 220px; height: 96px; }
        .is-ready { width: 104px; height: 32px; }
        .is-selected { height: 32px; }
        "#,
    )
    .expect("HTML and CSS should compile together");
    let projection = DocumentProjection::new()
        .set_text("run", "Ready")
        .add_class("run", "is-ready")
        .set_data("panel", "state", "ready")
        .with_elements(["run", "select"], |mut control| {
            control.aria("pressed", "false").add_class("control");
        });
    let registry = DocumentCommandRegistry::new()
        .bind("project.run", HtmlAction::Run)
        .bind_click("project.select", HtmlAction::Select);

    let (report, output) = bundle
        .update_with_projection(Size::new(320.0, 180.0), &projection)
        .expect("HTML bundle should project app state before update");
    let run = output.snapshot().find("run").unwrap();

    assert_eq!(report.operations, 7);
    assert_eq!(report.changed, 7);
    assert_eq!(
        output.snapshot().find("panel").unwrap().data("state"),
        Some("ready")
    );
    assert_eq!(run.text(), Some("Ready".to_owned()));
    assert!(run.has_class("is-ready"));
    assert_eq!(run.aria("pressed"), Some("false"));
    assert_eq!(run.rect().size.width, 104.0);

    let (report, frame) = bundle
        .update_with_input_projected_with_actions(
            Size::new(320.0, 180.0),
            DocumentInput::primary_click(Point::new(8.0, 8.0)),
            |projection| {
                projection
                    .element("run")
                    .text("Running")
                    .aria("pressed", "true")
                    .add_class("is-ready");
                projection.element("select").add_class("is-selected");
            },
            &registry,
        )
        .expect("HTML bundle should project app state and collect actions");
    let run = frame.output().snapshot().find("run").unwrap();
    let (mapped_report, mapped_update_frame) = bundle
        .update_with_input_projected_with_and_actions(
            Size::new(320.0, 180.0),
            DocumentInput::primary_click(Point::new(8.0, 32.0)),
            |projection| {
                projection.element("select").add_class("is-selected");
            },
            [
                ("project.run", HtmlAction::Run),
                ("project.select", HtmlAction::Select),
            ],
        )
        .expect("HTML bundle should project state and map command actions in one call");
    let mapped_select = mapped_update_frame
        .output()
        .snapshot()
        .find("select")
        .unwrap();
    let mut projected_dispatched = Vec::new();
    let (dispatch_report, dispatch_frame, action_report) = bundle
        .update_with_input_projection_and_dispatch(
            Size::new(320.0, 180.0),
            DocumentInput::primary_click(Point::new(8.0, 32.0)),
            &projection,
            &registry,
            |action| {
                projected_dispatched.push(*action.action());
            },
        )
        .expect("HTML bundle should project state and dispatch actions in one call");
    let dispatch_select = dispatch_frame.output().snapshot().find("select").unwrap();
    let mut projected_value_dispatched = Vec::new();
    let (value_dispatch_report, value_dispatch_frame, value_action_report) = bundle
        .update_with_input_projection_and_dispatch_action_values(
            Size::new(320.0, 180.0),
            DocumentInput::primary_click(Point::new(8.0, 32.0)),
            &projection,
            &registry,
            |action| projected_value_dispatched.push(*action),
        )
        .expect("HTML bundle should project state and dispatch action values in one call");
    let value_dispatch_select = value_dispatch_frame
        .output()
        .snapshot()
        .find("select")
        .unwrap();
    let mut projected_with_dispatched = Vec::new();
    let (dispatch_with_report, dispatch_with_frame, dispatch_with_action_report) = bundle
        .update_with_input_projected_with_and_dispatch_with(
            Size::new(320.0, 180.0),
            DocumentInput::primary_click(Point::new(8.0, 8.0)),
            |projection| {
                projection.element("run").text("Dispatch configured");
            },
            |commands| {
                commands.push("project.run", HtmlAction::Run);
                commands.push_click("project.select", HtmlAction::Select);
            },
            |action| {
                projected_with_dispatched.push(*action.action());
            },
        )
        .expect("HTML bundle should build projection, configure actions, and dispatch");
    let dispatch_with_run = dispatch_with_frame.output().snapshot().find("run").unwrap();
    let mut projected_with_value_dispatched = Vec::new();
    let (dispatch_with_value_report, dispatch_with_value_frame, dispatch_with_value_action_report) =
        bundle
            .update_with_input_projected_with_and_dispatch_action_values_with(
                Size::new(320.0, 180.0),
                DocumentInput::primary_click(Point::new(8.0, 8.0)),
                |projection| {
                    projection.element("run").text("Dispatch value");
                },
                |commands| {
                    commands.push("project.run", HtmlAction::Run);
                    commands.push_click("project.select", HtmlAction::Select);
                },
                |action| projected_with_value_dispatched.push(*action),
            )
            .expect("HTML bundle should build projection and dispatch action values");
    let dispatch_with_value_run = dispatch_with_value_frame
        .output()
        .snapshot()
        .find("run")
        .unwrap();

    assert_eq!(report.operations, 4);
    assert_eq!(report.changed, 4);
    assert_eq!(run.text(), Some("Running".to_owned()));
    assert_eq!(run.aria("pressed"), Some("true"));
    assert!(frame.contains_action(&HtmlAction::Run));
    assert!(!frame.contains_action(&HtmlAction::Select));
    assert_eq!(mapped_report.operations, 1);
    assert_eq!(mapped_report.changed, 1);
    assert!(mapped_select.has_class("is-selected"));
    assert!(mapped_update_frame.contains_clicked_action(&HtmlAction::Select));
    assert_eq!(dispatch_report.operations, 7);
    assert_eq!(dispatch_report.changed, 7);
    assert_eq!(action_report, DocumentCommandDispatchReport::new(1, 1, 0));
    assert_eq!(projected_dispatched, vec![HtmlAction::Select]);
    assert_eq!(dispatch_select.aria("pressed"), Some("false"));
    assert!(dispatch_frame.contains_clicked_action(&HtmlAction::Select));
    assert_eq!(value_dispatch_report.operations, 7);
    assert_eq!(value_dispatch_report.changed, 7);
    assert_eq!(
        value_action_report,
        DocumentCommandDispatchReport::new(1, 1, 0)
    );
    assert_eq!(projected_value_dispatched, vec![HtmlAction::Select]);
    assert_eq!(value_dispatch_select.aria("pressed"), Some("false"));
    assert!(value_dispatch_frame.contains_clicked_action(&HtmlAction::Select));
    assert_eq!(dispatch_with_report.operations, 1);
    assert_eq!(dispatch_with_report.changed, 1);
    assert_eq!(
        dispatch_with_action_report,
        DocumentCommandDispatchReport::new(1, 1, 0)
    );
    assert_eq!(projected_with_dispatched, vec![HtmlAction::Run]);
    assert_eq!(
        dispatch_with_run.text(),
        Some("Dispatch configured".to_owned())
    );
    assert!(dispatch_with_frame.contains_clicked_action(&HtmlAction::Run));
    assert_eq!(dispatch_with_value_report.operations, 1);
    assert_eq!(dispatch_with_value_report.changed, 1);
    assert_eq!(
        dispatch_with_value_action_report,
        DocumentCommandDispatchReport::new(1, 1, 0)
    );
    assert_eq!(projected_with_value_dispatched, vec![HtmlAction::Run]);
    assert_eq!(
        dispatch_with_value_run.text(),
        Some("Dispatch value".to_owned())
    );
    assert!(dispatch_with_value_frame.contains_clicked_action(&HtmlAction::Run));

    let projected_registry = bundle.command_action_registry([
        ("project.run", HtmlAction::Run),
        ("project.select", HtmlAction::Select),
    ]);
    let mut mapped_surface = bundle
        .to_action_surface_with_actions(
            Size::new(320.0, 180.0),
            [
                ("project.run", HtmlAction::Run),
                ("project.select", HtmlAction::Select),
            ],
        )
        .expect("HTML bundle should create mapped action surfaces");
    let mapped_frame = mapped_surface
        .update_with_input_actions(DocumentInput::primary_click(Point::new(8.0, 32.0)));
    let (surface_report, mut surface) = bundle
        .to_action_surface_with_projection(Size::new(320.0, 180.0), &projection, projected_registry)
        .expect("HTML bundle should create a projected action surface");
    let (mapped_surface_report, mut mapped_projected_surface) = bundle
        .to_action_surface_projected_with_actions(
            Size::new(320.0, 180.0),
            |projection| {
                projection.element("select").add_class("is-selected");
            },
            [
                ("project.run", HtmlAction::Run),
                ("project.select", HtmlAction::Select),
            ],
        )
        .expect("HTML bundle should create mapped projected action surfaces");
    let (intent_surface_report, mut intent_projected_surface) = bundle
        .to_action_surface_projected_with_intent_actions(
            Size::new(320.0, 180.0),
            |projection| {
                projection.element("select").add_class("is-selected");
            },
            [
                (
                    ElementBehaviorEvent::Click,
                    "project.select",
                    HtmlAction::Select,
                ),
                (ElementBehaviorEvent::Click, "project.run", HtmlAction::Run),
            ],
        )
        .expect("HTML bundle should create intent-mapped projected action surfaces");
    let surface_frame =
        surface.update_with_input_actions(DocumentInput::primary_click(Point::new(8.0, 0.0)));
    let mapped_projected_surface_frame = mapped_projected_surface
        .update_with_input_actions(DocumentInput::primary_click(Point::new(8.0, 32.0)));
    let intent_projected_surface_frame = intent_projected_surface
        .update_with_input_actions(DocumentInput::primary_click(Point::new(8.0, 32.0)));
    let surface_run = surface_frame.output().snapshot().find("run").unwrap();
    let mapped_projected_select = mapped_projected_surface_frame
        .output()
        .snapshot()
        .find("select")
        .unwrap();
    let intent_projected_select = intent_projected_surface_frame
        .output()
        .snapshot()
        .find("select")
        .unwrap();

    assert_eq!(surface_report.operations, 7);
    assert_eq!(surface_report.changed, 7);
    assert_eq!(surface_run.text(), Some("Ready".to_owned()));
    assert!(surface_run.has_class("is-ready"));
    assert!(mapped_frame.contains_clicked_action(&HtmlAction::Select));
    assert!(surface_frame.contains_clicked_action(&HtmlAction::Run));
    assert_eq!(mapped_surface_report.operations, 1);
    assert_eq!(mapped_surface_report.changed, 1);
    assert!(mapped_projected_select.has_class("is-selected"));
    assert!(mapped_projected_surface_frame.contains_clicked_action(&HtmlAction::Select));
    assert_eq!(intent_surface_report.operations, 1);
    assert_eq!(intent_surface_report.changed, 1);
    assert!(intent_projected_select.has_class("is-selected"));
    assert!(intent_projected_surface_frame.contains_clicked_action(&HtmlAction::Select));

    let (configured_report, mut configured_surface) = bundle
        .to_action_surface_projected_with_and(
            Size::new(320.0, 180.0),
            |projection| {
                projection.element("select").add_class("is-selected");
            },
            |commands| {
                commands.push_click("project.run", HtmlAction::Run);
                commands.push_click("project.select", HtmlAction::Select);
            },
        )
        .expect("HTML bundle should configure projected action surfaces in one call");
    let configured_frame = configured_surface
        .update_with_input_actions(DocumentInput::primary_click(Point::new(8.0, 32.0)));
    let configured_select = configured_frame.output().snapshot().find("select").unwrap();

    assert_eq!(configured_report.operations, 1);
    assert_eq!(configured_report.changed, 1);
    assert!(configured_select.has_class("is-selected"));
    assert!(configured_frame.contains_clicked_action(&HtmlAction::Select));
}

#[test]
fn html_stylesheet_projection_errors_remain_explicit() {
    let bundle = HtmlStylesheet::parse_fragment(
        r#"<section id="panel">Panel</section>"#,
        r#"#panel { width: 120px; height: 40px; }"#,
    )
    .expect("HTML and CSS should compile together");
    let error = bundle
        .update_projected_with(Size::new(320.0, 180.0), |projection| {
            projection.element("missing").add_class("is-ready");
        })
        .expect_err("missing projection targets should stay visible");

    assert!(
        error.to_string().contains("html document error"),
        "unexpected error: {error}"
    );
}

#[test]
fn html_prelude_exposes_browser_document_authoring_surface() {
    use des_html::prelude::*;

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum HtmlAction {
        Run,
    }

    let html = HtmlDocument::parse_fragment(
        r#"<script>ignored()</script><button id="run" class="primary" on:click="project.run">Run</button>"#,
    )
    .map(|html| {
        html.with(|html| html.diagnostics.clear())
            .when(false, |html| html.children.clear())
    })
    .and_then(|html| html.try_with(|html| {
        assert!(html.find_by_id("run").is_some());
        Ok::<_, HtmlError>(())
    }))
    .and_then(|html| html.try_when(true, |html| {
        assert!(html.has_command_hook("project.run"));
        Ok::<_, HtmlError>(())
    }))
    .expect("HTML should compile and configure from the prelude");
    let bundle = html
        .with_css(r#".primary { width: 96px; height: 32px; }"#)
        .and_then(|bundle| bundle.with_css(".primary { padding: 2px; }"))
        .and_then(|bundle| bundle.with_css_if(false, ".primary { width: ; }"))
        .and_then(|bundle| {
            bundle
                .with_css_forgiving(".primary { unknown-property: 1px; } .primary { margin: 1px; }")
        })
        .map(|bundle| {
            bundle
                .with(|bundle| {
                    bundle.extend_stylesheet(
                        StyleSheet::new().class("primary", Style::default().radius(4.0)),
                    );
                })
                .when(false, |bundle| {
                    bundle.replace_stylesheet(StyleSheet::new());
                })
        })
        .and_then(|bundle| {
            bundle.try_with(|bundle| {
                bundle.extend_css_forgiving(".primary { unknown-property: 2px; }")?;
                Ok::<_, HtmlError>(())
            })
        })
        .and_then(|bundle| {
            bundle.try_when(true, |bundle| {
                bundle.extend_stylesheet(
                    StyleSheet::new()
                        .class("primary", Style::default().border(Color::rgb(90, 120, 180))),
                );
                Ok::<_, HtmlError>(())
            })
        })
        .expect("HTML and CSS should compile together from the prelude");
    assert!(bundle.html().is_clean());
    assert_eq!(bundle.html().children()[0].id.as_deref(), Some("run"));
    assert!(bundle.stylesheet().rule_count() >= 4);
    assert!(bundle.stylesheet().has_rule_for_class("primary"));

    let mut surface = bundle
        .into_action_surface_with_actions(
            Size::new(320.0, 180.0),
            [("project.run", HtmlAction::Run)],
        )
        .expect("prelude-authored HTML should create an action surface");
    let frame: DocumentActionFrame<HtmlAction> =
        surface.update_with_input_actions(DocumentInput::primary_click(Point::new(8.0, 8.0)));
    let run = frame.output.snapshot().find("run").unwrap();

    assert_eq!(run.rect().size.width, 96.0);
    assert_eq!(run.style().padding, Insets::all(2.0));
    assert_eq!(run.style().margin, Insets::all(1.0));
    assert_eq!(run.style().radius, CornerRadii::all(4.0));
    assert_eq!(run.style().border, Some(Color::rgb(90, 120, 180)));
    assert_eq!(surface.commands().bindings().len(), 1);
    assert_eq!(frame.actions.len(), 1);
    assert_eq!(frame.actions[0].action, HtmlAction::Run);
    assert_eq!(frame.actions[0].target, ElementId::new("run"));
}

#[test]
fn html_document_and_stylesheet_load_from_files() {
    let html_fixture = TempHtmlPath::new("des-html-document-load", "html");
    let css_fixture = TempHtmlPath::new("des-html-stylesheet-load", "css");
    fs::write(
        html_fixture.path(),
        r#"<section id="loaded" class="card">Loaded</section>"#,
    )
    .expect("HTML fixture should be writable");
    fs::write(css_fixture.path(), ".card { width: 88px; height: 24px; }")
        .expect("CSS fixture should be writable");

    let html = HtmlDocument::load(html_fixture.path()).expect("HTML document should load");
    assert!(
        html.children
            .iter()
            .any(|node| node.tag == "html" || node.id.as_deref() == Some("loaded")),
        "document parsing should return browser document structure"
    );

    let bundle = HtmlStylesheet::load_files(html_fixture.path(), css_fixture.path())
        .expect("HTML+CSS files should load");
    assert!(bundle.stylesheet.rule_count() > 0);
}

#[test]
fn html_document_and_forgiving_stylesheet_load_from_files() {
    let html_fixture = TempHtmlPath::new("des-html-forgiving-document-load", "html");
    let css_fixture = TempHtmlPath::new("des-html-forgiving-stylesheet-load", "css");
    fs::write(
        html_fixture.path(),
        r#"<section id="loaded" class="card">Loaded</section>"#,
    )
    .expect("HTML fixture should be writable");
    fs::write(
        css_fixture.path(),
        r#"
        .bad { height: ; }
        .card { width: 88px; height: 24px; }
        "#,
    )
    .expect("CSS fixture should be writable");

    let bundle = HtmlStylesheet::load_files_forgiving(html_fixture.path(), css_fixture.path())
        .expect("HTML+forgiving CSS files should load");
    let mut view = bundle
        .to_view(Size::new(240.0, 160.0))
        .expect("file-backed bundle should create a view");
    let output = view.update();
    let loaded = output.snapshot().find("loaded").unwrap();

    assert_eq!(loaded.rect().size.width, 88.0);
    assert_eq!(loaded.rect().size.height, 24.0);
}

#[test]
fn html_file_hot_reloads_when_source_changes() {
    let fixture = TempHtmlPath::new("des-html-hot-reload", "html");
    let path = fixture.path();

    fs::write(path, "<section id=\"status\">Before</section>")
        .expect("html fixture should be writable");
    let mut file = HtmlFile::load(path).expect("html file should load");
    assert!(
        file.document()
            .find_by_id("status")
            .is_some_and(|node| node.text.as_deref() == Some("Before"))
    );

    std::thread::sleep(Duration::from_millis(5));
    fs::write(
        path,
        "<section id=\"status\" class=\"changed\">After</section>",
    )
    .expect("html fixture should update");

    let status = file
        .reload_if_changed()
        .expect("html file should hot reload");

    assert!(status.changed);
    assert!(
        file.document().find_by_id("status").is_some_and(
            |node| node.classes == ["changed"] && node.text.as_deref() == Some("After")
        )
    );
}

#[test]
fn html_file_hot_reload_detects_same_mtime_content_changes() {
    let fixture = TempHtmlPath::new("des-html-hot-reload-fingerprint", "html");
    let path = fixture.path();

    fs::write(path, "<section id=\"status\">Before</section>")
        .expect("html fixture should be writable");
    let mut file = HtmlFile::load(path).expect("html file should load");
    let original_modified = fs::metadata(path)
        .expect("html fixture should have metadata")
        .modified()
        .ok();

    fs::write(path, "<section id=\"status\">After</section>").expect("html fixture should update");
    if let Some(modified) = original_modified {
        let filetime = filetime::FileTime::from_system_time(modified);
        filetime::set_file_mtime(path, filetime).expect("mtime should be restorable");
    }

    let status = file
        .reload_if_changed()
        .expect("html file should hot reload");

    assert!(status.changed);
    assert!(
        file.document()
            .find_by_id("status")
            .is_some_and(|node| node.text.as_deref() == Some("After"))
    );
}

#[test]
fn html_set_manages_named_inline_and_file_backed_documents() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum SetAction {
        Run,
        Open,
        Menu,
    }

    let fixture = TempHtmlPath::new("des-html-set", "html");
    let path = fixture.path();
    fs::write(
        path,
        "<button id=\"file\" on:click=\"file.open\">Before</button>",
    )
    .expect("html fixture should be writable");

    let mut set = HtmlSet::new();
    set.add_fragment(
        "inline",
        "<button id=\"inline\" on:click=\"inline.run\">Inline</button>",
    )
    .expect("inline html should parse");
    set.add_fragment(
        "shared",
        r#"<button id="shared" on:click="shared.open" on:contextmenu="shared.open">Shared</button>"#,
    )
    .expect("shared html should parse");
    set.add_file("file", path).expect("file html should parse");
    let registry = DocumentCommandRegistry::new()
        .bind_click("inline.run", SetAction::Run)
        .bind_click("file.open", SetAction::Open);
    let mapped_registry = set
        .command_action_registry("inline", [("inline.run", SetAction::Run)])
        .expect("named document should build mapped command actions");
    let mut pushed_mapped_registry = DocumentCommandRegistry::new();
    set.push_command_actions(
        "inline",
        &mut pushed_mapped_registry,
        [("inline.run", SetAction::Run)],
    )
    .expect("named document should push mapped command actions");

    assert_eq!(set.len(), 3);
    assert!(!set.is_empty());
    assert!(set.contains("inline"));
    assert_eq!(
        set.names().collect::<Vec<_>>(),
        ["file", "inline", "shared"]
    );
    assert!(
        set.get("inline")
            .expect("inline document should exist")
            .children
            .iter()
            .any(|node| node.id.as_deref() == Some("inline"))
    );
    assert!(
        set.get("file")
            .expect("file document should exist")
            .find_by_id("file")
            .is_some()
    );
    let output = set
        .update_with_stylesheet(
            "inline",
            Size::new(240.0, 160.0),
            des_document::StyleSheet::parse_css("#inline { width: 120px; height: 32px; }")
                .expect("CSS should parse"),
        )
        .expect("named document should resolve through the set front door");
    let css_output = set
        .update_with_css(
            "inline",
            Size::new(240.0, 160.0),
            "#inline { width: 128px; height: 32px; }",
        )
        .expect("named document should resolve strict CSS through the set front door");
    let css_forgiving_output = set
        .update_with_css_forgiving(
            "inline",
            Size::new(240.0, 160.0),
            ".ignored { unknown-property: yes; } #inline { width: 136px; height: 32px; }",
        )
        .expect("named document should resolve forgiving CSS through the set front door");
    let mut css_view = set
        .to_view_with_css(
            "inline",
            Size::new(240.0, 160.0),
            "#inline { width: 140px; height: 32px; }",
        )
        .expect("named document should create strict CSS views through the set front door");
    let css_view_output = css_view.update();
    let mut css_forgiving_view = set
        .to_view_with_css_forgiving(
            "inline",
            Size::new(240.0, 160.0),
            ".ignored { unknown-property: yes; } #inline { width: 144px; height: 32px; }",
        )
        .expect("named document should create forgiving CSS views through the set front door");
    let css_forgiving_view_output = css_forgiving_view.update();
    let inline = output.snapshot().find("inline").unwrap();

    assert_eq!(inline.text(), Some("Inline".to_owned()));
    assert_eq!(inline.rect().size.width, 120.0);
    assert_eq!(
        css_output
            .snapshot()
            .find("inline")
            .unwrap()
            .rect()
            .size
            .width,
        128.0
    );
    assert_eq!(
        css_forgiving_output
            .snapshot()
            .find("inline")
            .unwrap()
            .rect()
            .size
            .width,
        136.0
    );
    assert_eq!(
        css_view_output
            .snapshot()
            .find("inline")
            .unwrap()
            .rect()
            .size
            .width,
        140.0
    );
    assert_eq!(
        css_forgiving_view_output
            .snapshot()
            .find("inline")
            .unwrap()
            .rect()
            .size
            .width,
        144.0
    );
    let input_output = set
        .update_with_input(
            "inline",
            Size::new(240.0, 160.0),
            DocumentInput::primary_click(Point::new(8.0, 8.0)),
        )
        .expect("named document should route input through the set front door");
    let input_css_output = set
        .update_with_input_and_css(
            "inline",
            Size::new(240.0, 160.0),
            DocumentInput::primary_click(Point::new(8.0, 8.0)),
            "#inline { width: 146px; height: 32px; }",
        )
        .expect("named document should route input through strict CSS");
    let input_css_forgiving_output = set
        .update_with_input_and_css_forgiving(
            "inline",
            Size::new(240.0, 160.0),
            DocumentInput::primary_click(Point::new(8.0, 8.0)),
            ".ignored { unknown-property: yes; } #inline { width: 147px; height: 32px; }",
        )
        .expect("named document should route input through forgiving CSS");
    let action_frame = set
        .update_with_input_actions(
            "inline",
            Size::new(240.0, 160.0),
            DocumentInput::primary_click(Point::new(8.0, 8.0)),
            &registry,
        )
        .expect("named document should collect typed actions through the set front door");
    let mut dispatched = Vec::new();
    let (dispatch_frame, dispatch_report) = set
        .update_with_input_and_dispatch(
            "inline",
            Size::new(240.0, 160.0),
            DocumentInput::primary_click(Point::new(8.0, 8.0)),
            &registry,
            |action| {
                dispatched.push(*action.action());
            },
        )
        .expect("named document should dispatch typed actions through the set front door");
    let mut dispatched_values = Vec::new();
    let (value_dispatch_frame, value_dispatch_report) = set
        .update_with_input_and_dispatch_action_values(
            "inline",
            Size::new(240.0, 160.0),
            DocumentInput::primary_click(Point::new(8.0, 8.0)),
            &registry,
            |action| dispatched_values.push(*action),
        )
        .expect("named document should dispatch typed action values through the set front door");
    let mut configured_dispatched = Vec::new();
    let (configured_dispatch_frame, configured_dispatch_report) = set
        .update_with_input_and_dispatch_with(
            "inline",
            Size::new(240.0, 160.0),
            DocumentInput::primary_click(Point::new(8.0, 8.0)),
            |commands| {
                commands.push_click("inline.run", SetAction::Run);
            },
            |action| {
                configured_dispatched.push(*action.action());
            },
        )
        .expect(
            "named document should configure and dispatch typed actions through the set front door",
        );
    let mut configured_dispatched_values = Vec::new();
    let (configured_value_dispatch_frame, configured_value_dispatch_report) = set
        .update_with_input_and_dispatch_action_values_with(
            "inline",
            Size::new(240.0, 160.0),
            DocumentInput::primary_click(Point::new(8.0, 8.0)),
            |commands| {
                commands.push_click("inline.run", SetAction::Run);
            },
            |action| configured_dispatched_values.push(*action),
        )
        .expect("named document should configure and dispatch typed action values");
    let empty_frame = set
        .update_actions("inline", Size::new(240.0, 160.0), &registry)
        .expect("named document should update and collect actions through the set front door");
    let mut surface = set
        .to_action_surface_with("inline", Size::new(240.0, 160.0), |commands| {
            commands.push_click("inline.run", SetAction::Run);
        })
        .expect("named document should build an action surface through the set front door");
    let surface_frame =
        surface.update_with_input_actions(DocumentInput::primary_click(Point::new(8.0, 8.0)));
    let mut mapped_surface = set
        .to_action_surface_with_actions(
            "inline",
            Size::new(240.0, 160.0),
            [("inline.run", SetAction::Run)],
        )
        .expect("named document should build mapped action surfaces");
    let mapped_surface_frame = mapped_surface
        .update_with_input_actions(DocumentInput::primary_click(Point::new(8.0, 8.0)));
    let mut intent_surface = set
        .to_action_surface_with_intent_actions(
            "shared",
            Size::new(240.0, 160.0),
            [
                (ElementBehaviorEvent::Click, "shared.open", SetAction::Open),
                (
                    ElementBehaviorEvent::ContextMenu,
                    "shared.open",
                    SetAction::Menu,
                ),
            ],
        )
        .expect("named document should build intent-mapped action surfaces");
    let intent_surface_frame = intent_surface
        .update_with_input_actions(DocumentInput::secondary_click(Point::new(8.0, 8.0)));
    let css_action_frame = set
        .update_with_input_actions_and_css(
            "inline",
            Size::new(240.0, 160.0),
            DocumentInput::primary_click(Point::new(8.0, 8.0)),
            "#inline { width: 148px; height: 32px; }",
            &registry,
        )
        .expect("named document should route input through strict CSS action helpers");
    let css_configured_frame = set
        .update_with_input_actions_and_css_with(
            "inline",
            Size::new(240.0, 160.0),
            DocumentInput::primary_click(Point::new(8.0, 8.0)),
            "#inline { width: 152px; height: 32px; }",
            |commands| {
                commands.push_click("inline.run", SetAction::Run);
            },
        )
        .expect("named document should configure actions through strict CSS helpers");
    let css_forgiving_action_frame = set
        .update_with_input_actions_and_css_forgiving(
            "inline",
            Size::new(240.0, 160.0),
            DocumentInput::primary_click(Point::new(8.0, 8.0)),
            ".ignored { unknown-property: yes; } #inline { width: 156px; height: 32px; }",
            &registry,
        )
        .expect("named document should route input through forgiving CSS action helpers");
    let mut css_dispatched = Vec::new();
    let (css_dispatch_frame, css_dispatch_report) = set
        .update_with_input_and_css_and_dispatch(
            "inline",
            Size::new(240.0, 160.0),
            DocumentInput::primary_click(Point::new(8.0, 8.0)),
            "#inline { width: 168px; height: 32px; }",
            &registry,
            |action| {
                css_dispatched.push(*action.action());
            },
        )
        .expect("named document should dispatch strict CSS action frames");
    let mut css_dispatched_values = Vec::new();
    let (css_value_dispatch_frame, css_value_dispatch_report) = set
        .update_with_input_and_css_and_dispatch_action_values(
            "inline",
            Size::new(240.0, 160.0),
            DocumentInput::primary_click(Point::new(8.0, 8.0)),
            "#inline { width: 169px; height: 32px; }",
            &registry,
            |action| css_dispatched_values.push(*action),
        )
        .expect("named document should dispatch strict CSS action values");
    let mut css_configured_dispatched = Vec::new();
    let (css_configured_dispatch_frame, css_configured_dispatch_report) = set
        .update_with_input_and_css_and_dispatch_with(
            "inline",
            Size::new(240.0, 160.0),
            DocumentInput::primary_click(Point::new(8.0, 8.0)),
            "#inline { width: 172px; height: 32px; }",
            |commands| {
                commands.push_click("inline.run", SetAction::Run);
            },
            |action| {
                css_configured_dispatched.push(*action.action());
            },
        )
        .expect("named document should configure and dispatch strict CSS action frames");
    let mut css_configured_dispatched_values = Vec::new();
    let (css_configured_value_dispatch_frame, css_configured_value_dispatch_report) = set
        .update_with_input_and_css_and_dispatch_action_values_with(
            "inline",
            Size::new(240.0, 160.0),
            DocumentInput::primary_click(Point::new(8.0, 8.0)),
            "#inline { width: 173px; height: 32px; }",
            |commands| {
                commands.push_click("inline.run", SetAction::Run);
            },
            |action| css_configured_dispatched_values.push(*action),
        )
        .expect("named document should configure and dispatch strict CSS action values");
    let mut css_forgiving_dispatched = Vec::new();
    let (css_forgiving_dispatch_frame, css_forgiving_dispatch_report) = set
        .update_with_input_and_css_forgiving_and_dispatch(
            "inline",
            Size::new(240.0, 160.0),
            DocumentInput::primary_click(Point::new(8.0, 8.0)),
            ".ignored { unknown-property: yes; } #inline { width: 176px; height: 32px; }",
            &registry,
            |action| {
                css_forgiving_dispatched.push(*action.action());
            },
        )
        .expect("named document should dispatch forgiving CSS action frames");
    let mut css_forgiving_dispatched_values = Vec::new();
    let (css_forgiving_value_dispatch_frame, css_forgiving_value_dispatch_report) = set
        .update_with_input_and_css_forgiving_and_dispatch_action_values(
            "inline",
            Size::new(240.0, 160.0),
            DocumentInput::primary_click(Point::new(8.0, 8.0)),
            ".ignored { unknown-property: yes; } #inline { width: 177px; height: 32px; }",
            &registry,
            |action| css_forgiving_dispatched_values.push(*action),
        )
        .expect("named document should dispatch forgiving CSS action values");
    let mut css_forgiving_configured_dispatched = Vec::new();
    let (css_forgiving_configured_dispatch_frame, css_forgiving_configured_dispatch_report) = set
        .update_with_input_and_css_forgiving_and_dispatch_with(
            "inline",
            Size::new(240.0, 160.0),
            DocumentInput::primary_click(Point::new(8.0, 8.0)),
            ".ignored { unknown-property: yes; } #inline { width: 180px; height: 32px; }",
            |commands| {
                commands.push_click("inline.run", SetAction::Run);
            },
            |action| {
                css_forgiving_configured_dispatched.push(*action.action());
            },
        )
        .expect("named document should configure and dispatch forgiving CSS action frames");
    let mut css_forgiving_configured_dispatched_values = Vec::new();
    let (
        css_forgiving_configured_value_dispatch_frame,
        css_forgiving_configured_value_dispatch_report,
    ) = set
        .update_with_input_and_css_forgiving_and_dispatch_action_values_with(
            "inline",
            Size::new(240.0, 160.0),
            DocumentInput::primary_click(Point::new(8.0, 8.0)),
            ".ignored { unknown-property: yes; } #inline { width: 181px; height: 32px; }",
            |commands| {
                commands.push_click("inline.run", SetAction::Run);
            },
            |action| css_forgiving_configured_dispatched_values.push(*action),
        )
        .expect("named document should configure and dispatch forgiving CSS action values");
    let styled_surface = set
        .to_action_surface_with_stylesheet(
            "inline",
            Size::new(240.0, 160.0),
            des_document::StyleSheet::parse_css("#inline { width: 132px; height: 32px; }")
                .expect("CSS should parse"),
            registry.clone(),
        )
        .expect("named document should build a styled action surface through the set front door");
    let mut css_surface = set
        .to_action_surface_with_css(
            "inline",
            Size::new(240.0, 160.0),
            "#inline { width: 160px; height: 32px; }",
            |commands| {
                commands.push_click("inline.run", SetAction::Run);
            },
        )
        .expect("named document should build strict CSS action surfaces");
    let css_surface_frame =
        css_surface.update_with_input_actions(DocumentInput::primary_click(Point::new(8.0, 8.0)));
    let mut css_mapped_surface = set
        .to_action_surface_with_css_and_actions(
            "inline",
            Size::new(240.0, 160.0),
            "#inline { width: 161px; height: 32px; }",
            [("inline.run", SetAction::Run)],
        )
        .expect("named document should build mapped strict CSS action surfaces");
    let css_mapped_surface_frame = css_mapped_surface
        .update_with_input_actions(DocumentInput::primary_click(Point::new(8.0, 8.0)));
    let mut css_intent_surface = set
        .to_action_surface_with_css_and_intent_actions(
            "shared",
            Size::new(240.0, 160.0),
            "#shared { width: 162px; height: 32px; }",
            [
                (ElementBehaviorEvent::Click, "shared.open", SetAction::Open),
                (
                    ElementBehaviorEvent::ContextMenu,
                    "shared.open",
                    SetAction::Menu,
                ),
            ],
        )
        .expect("named document should build intent-mapped strict CSS action surfaces");
    let css_intent_surface_frame = css_intent_surface
        .update_with_input_actions(DocumentInput::secondary_click(Point::new(8.0, 8.0)));
    let mut skipped_css_surface = set
        .to_action_surface_with_css_if(
            "inline",
            Size::new(240.0, 160.0),
            false,
            "#inline { width: ; }",
            |commands| {
                commands.push_click("inline.run", SetAction::Run);
            },
        )
        .expect("named skipped strict CSS action surface should not parse");
    let skipped_css_surface_frame = skipped_css_surface
        .update_with_input_actions(DocumentInput::primary_click(Point::new(8.0, 8.0)));
    let mut css_forgiving_surface = set
        .to_action_surface_with_css_forgiving(
            "inline",
            Size::new(240.0, 160.0),
            ".ignored { unknown-property: yes; } #inline { width: 164px; height: 32px; }",
            |commands| {
                commands.push_click("inline.run", SetAction::Run);
            },
        )
        .expect("named document should build forgiving CSS action surfaces");
    let css_forgiving_surface_frame = css_forgiving_surface
        .update_with_input_actions(DocumentInput::primary_click(Point::new(8.0, 8.0)));
    let mut css_forgiving_mapped_surface = set
        .to_action_surface_with_css_forgiving_and_actions(
            "inline",
            Size::new(240.0, 160.0),
            ".ignored { unknown-property: yes; } #inline { width: 165px; height: 32px; }",
            [("inline.run", SetAction::Run)],
        )
        .expect("named document should build mapped forgiving CSS action surfaces");
    let css_forgiving_mapped_surface_frame = css_forgiving_mapped_surface
        .update_with_input_actions(DocumentInput::primary_click(Point::new(8.0, 8.0)));
    let mut css_forgiving_intent_surface = set
        .to_action_surface_with_css_forgiving_and_intent_actions(
            "shared",
            Size::new(240.0, 160.0),
            ".ignored { unknown-property: yes; } #shared { width: 166px; height: 32px; }",
            [
                (ElementBehaviorEvent::Click, "shared.open", SetAction::Open),
                (
                    ElementBehaviorEvent::ContextMenu,
                    "shared.open",
                    SetAction::Menu,
                ),
            ],
        )
        .expect("named document should build intent-mapped forgiving CSS action surfaces");
    let css_forgiving_intent_surface_frame = css_forgiving_intent_surface
        .update_with_input_actions(DocumentInput::secondary_click(Point::new(8.0, 8.0)));
    let mut skipped_css_forgiving_surface = set
        .to_action_surface_with_css_forgiving_if(
            "inline",
            Size::new(240.0, 160.0),
            false,
            "/* unclosed",
            |commands| {
                commands.push_click("inline.run", SetAction::Run);
            },
        )
        .expect("named skipped forgiving CSS action surface should not parse");
    let skipped_css_forgiving_surface_frame = skipped_css_forgiving_surface
        .update_with_input_actions(DocumentInput::primary_click(Point::new(8.0, 8.0)));
    let (project_report, projected_output) = set
        .update_projected_with("inline", Size::new(240.0, 160.0), |projection| {
            projection
                .element("inline")
                .text("Projected")
                .add_class("is-ready");
        })
        .expect("named document should project state through the set front door");
    let projected_inline = projected_output.snapshot().find("inline").unwrap();
    let (mapped_project_report, mapped_project_frame) = set
        .update_with_input_projected_with_and_actions(
            "inline",
            Size::new(240.0, 160.0),
            DocumentInput::primary_click(Point::new(8.0, 8.0)),
            |projection| {
                projection.element("inline").text("Mapped");
            },
            [("inline.run", SetAction::Run)],
        )
        .expect("named document should project state and map actions through the set front door");
    let mut projected_dispatched = Vec::new();
    let (dispatch_project_report, dispatch_project_frame, dispatch_project_action_report) = set
        .update_with_input_projected_with_and_dispatch(
            "inline",
            Size::new(240.0, 160.0),
            DocumentInput::primary_click(Point::new(8.0, 8.0)),
            |projection| {
                projection.element("inline").text("Dispatched");
            },
            &registry,
            |action| {
                projected_dispatched.push(*action.action());
            },
        )
        .expect("named document should project state and dispatch actions");
    let mut projected_value_dispatched = Vec::new();
    let (value_project_report, value_project_frame, value_project_action_report) = set
        .update_with_input_projected_with_and_dispatch_action_values(
            "inline",
            Size::new(240.0, 160.0),
            DocumentInput::primary_click(Point::new(8.0, 8.0)),
            |projection| {
                projection.element("inline").text("Value dispatched");
            },
            &registry,
            |action| projected_value_dispatched.push(*action),
        )
        .expect("named document should project state and dispatch action values");
    let mut configured_projected_dispatched = Vec::new();
    let (
        configured_dispatch_project_report,
        configured_dispatch_project_frame,
        configured_dispatch_project_action_report,
    ) = set
        .update_with_input_projected_with_and_dispatch_with(
            "inline",
            Size::new(240.0, 160.0),
            DocumentInput::primary_click(Point::new(8.0, 8.0)),
            |projection| {
                projection.element("inline").text("Configured dispatch");
            },
            |commands| {
                commands.push_click("inline.run", SetAction::Run);
            },
            |action| {
                configured_projected_dispatched.push(*action.action());
            },
        )
        .expect("named document should configure projected dispatch actions");
    let mut configured_projected_value_dispatched = Vec::new();
    let (
        configured_value_project_report,
        configured_value_project_frame,
        configured_value_project_action_report,
    ) = set
        .update_with_input_projected_with_and_dispatch_action_values_with(
            "inline",
            Size::new(240.0, 160.0),
            DocumentInput::primary_click(Point::new(8.0, 8.0)),
            |projection| {
                projection.element("inline").text("Configured value");
            },
            |commands| {
                commands.push_click("inline.run", SetAction::Run);
            },
            |action| configured_projected_value_dispatched.push(*action),
        )
        .expect("named document should configure projected value dispatch actions");
    let (intent_project_report, intent_project_frame) = set
        .update_with_input_projected_with_and_intent_actions(
            "shared",
            Size::new(240.0, 160.0),
            DocumentInput::secondary_click(Point::new(8.0, 8.0)),
            |projection| {
                projection.element("shared").text("Menu");
            },
            [
                (ElementBehaviorEvent::Click, "shared.open", SetAction::Open),
                (
                    ElementBehaviorEvent::ContextMenu,
                    "shared.open",
                    SetAction::Menu,
                ),
            ],
        )
        .expect("named document should project state and map intent-scoped actions");
    let (projected_surface_report, mut projected_surface) = set
        .to_action_surface_projected_with_actions(
            "inline",
            Size::new(240.0, 160.0),
            |projection| {
                projection.element("inline").text("Surface");
            },
            [("inline.run", SetAction::Run)],
        )
        .expect("named document should build projected mapped action surfaces");
    let projected_surface_frame = projected_surface
        .update_with_input_actions(DocumentInput::primary_click(Point::new(8.0, 8.0)));
    let (intent_surface_report, mut intent_projected_surface) = set
        .to_action_surface_projected_with_intent_actions(
            "shared",
            Size::new(240.0, 160.0),
            |projection| {
                projection.element("shared").text("Surface menu");
            },
            [
                (ElementBehaviorEvent::Click, "shared.open", SetAction::Open),
                (
                    ElementBehaviorEvent::ContextMenu,
                    "shared.open",
                    SetAction::Menu,
                ),
            ],
        )
        .expect("named document should build projected intent-mapped action surfaces");
    let intent_projected_surface_frame = intent_projected_surface
        .update_with_input_actions(DocumentInput::secondary_click(Point::new(8.0, 8.0)));

    assert!(input_output.was_clicked("inline"));
    assert!(input_css_output.was_clicked("inline"));
    assert_eq!(
        input_css_output
            .snapshot()
            .find("inline")
            .unwrap()
            .rect()
            .size
            .width,
        146.0
    );
    assert!(input_css_forgiving_output.was_clicked("inline"));
    assert_eq!(
        input_css_forgiving_output
            .snapshot()
            .find("inline")
            .unwrap()
            .rect()
            .size
            .width,
        147.0
    );
    assert!(action_frame.contains_action(&SetAction::Run));
    assert!(dispatch_frame.contains_action(&SetAction::Run));
    assert_eq!(dispatch_report, DocumentCommandDispatchReport::new(1, 1, 0));
    assert_eq!(dispatched, vec![SetAction::Run]);
    assert!(value_dispatch_frame.contains_action(&SetAction::Run));
    assert_eq!(
        value_dispatch_report,
        DocumentCommandDispatchReport::new(1, 1, 0)
    );
    assert_eq!(dispatched_values, vec![SetAction::Run]);
    assert!(configured_dispatch_frame.contains_action(&SetAction::Run));
    assert_eq!(
        configured_dispatch_report,
        DocumentCommandDispatchReport::new(1, 1, 0)
    );
    assert_eq!(configured_dispatched, vec![SetAction::Run]);
    assert!(configured_value_dispatch_frame.contains_action(&SetAction::Run));
    assert_eq!(
        configured_value_dispatch_report,
        DocumentCommandDispatchReport::new(1, 1, 0)
    );
    assert_eq!(configured_dispatched_values, vec![SetAction::Run]);
    assert!(empty_frame.is_empty());
    assert_eq!(
        mapped_registry.bindings(),
        pushed_mapped_registry.bindings()
    );
    assert_eq!(mapped_registry.bindings().len(), 1);
    assert!(surface_frame.contains_action(&SetAction::Run));
    assert!(mapped_surface_frame.contains_action(&SetAction::Run));
    assert!(intent_surface_frame.contains_action_for_target_intent(
        "shared",
        ElementBehaviorEvent::ContextMenu,
        &SetAction::Menu
    ));
    assert!(css_action_frame.contains_action(&SetAction::Run));
    assert_eq!(
        css_action_frame
            .output()
            .snapshot()
            .find("inline")
            .unwrap()
            .rect()
            .size
            .width,
        148.0
    );
    assert!(css_configured_frame.contains_action(&SetAction::Run));
    assert_eq!(
        css_configured_frame
            .output()
            .snapshot()
            .find("inline")
            .unwrap()
            .rect()
            .size
            .width,
        152.0
    );
    assert!(css_forgiving_action_frame.contains_action(&SetAction::Run));
    assert_eq!(
        css_forgiving_action_frame
            .output()
            .snapshot()
            .find("inline")
            .unwrap()
            .rect()
            .size
            .width,
        156.0
    );
    assert!(css_dispatch_frame.contains_action(&SetAction::Run));
    assert_eq!(
        css_dispatch_report,
        DocumentCommandDispatchReport::new(1, 1, 0)
    );
    assert_eq!(css_dispatched, vec![SetAction::Run]);
    assert_eq!(
        css_dispatch_frame
            .output()
            .snapshot()
            .find("inline")
            .unwrap()
            .rect()
            .size
            .width,
        168.0
    );
    assert!(css_value_dispatch_frame.contains_action(&SetAction::Run));
    assert_eq!(
        css_value_dispatch_report,
        DocumentCommandDispatchReport::new(1, 1, 0)
    );
    assert_eq!(css_dispatched_values, vec![SetAction::Run]);
    assert_eq!(
        css_value_dispatch_frame
            .output()
            .snapshot()
            .find("inline")
            .unwrap()
            .rect()
            .size
            .width,
        169.0
    );
    assert!(css_configured_dispatch_frame.contains_action(&SetAction::Run));
    assert_eq!(
        css_configured_dispatch_report,
        DocumentCommandDispatchReport::new(1, 1, 0)
    );
    assert_eq!(css_configured_dispatched, vec![SetAction::Run]);
    assert_eq!(
        css_configured_dispatch_frame
            .output()
            .snapshot()
            .find("inline")
            .unwrap()
            .rect()
            .size
            .width,
        172.0
    );
    assert!(css_configured_value_dispatch_frame.contains_action(&SetAction::Run));
    assert_eq!(
        css_configured_value_dispatch_report,
        DocumentCommandDispatchReport::new(1, 1, 0)
    );
    assert_eq!(css_configured_dispatched_values, vec![SetAction::Run]);
    assert_eq!(
        css_configured_value_dispatch_frame
            .output()
            .snapshot()
            .find("inline")
            .unwrap()
            .rect()
            .size
            .width,
        173.0
    );
    assert!(css_forgiving_dispatch_frame.contains_action(&SetAction::Run));
    assert_eq!(
        css_forgiving_dispatch_report,
        DocumentCommandDispatchReport::new(1, 1, 0)
    );
    assert_eq!(css_forgiving_dispatched, vec![SetAction::Run]);
    assert_eq!(
        css_forgiving_dispatch_frame
            .output()
            .snapshot()
            .find("inline")
            .unwrap()
            .rect()
            .size
            .width,
        176.0
    );
    assert!(css_forgiving_value_dispatch_frame.contains_action(&SetAction::Run));
    assert_eq!(
        css_forgiving_value_dispatch_report,
        DocumentCommandDispatchReport::new(1, 1, 0)
    );
    assert_eq!(css_forgiving_dispatched_values, vec![SetAction::Run]);
    assert_eq!(
        css_forgiving_value_dispatch_frame
            .output()
            .snapshot()
            .find("inline")
            .unwrap()
            .rect()
            .size
            .width,
        177.0
    );
    assert!(css_forgiving_configured_dispatch_frame.contains_action(&SetAction::Run));
    assert_eq!(
        css_forgiving_configured_dispatch_report,
        DocumentCommandDispatchReport::new(1, 1, 0)
    );
    assert_eq!(css_forgiving_configured_dispatched, vec![SetAction::Run]);
    assert_eq!(
        css_forgiving_configured_dispatch_frame
            .output()
            .snapshot()
            .find("inline")
            .unwrap()
            .rect()
            .size
            .width,
        180.0
    );
    assert!(css_forgiving_configured_value_dispatch_frame.contains_action(&SetAction::Run));
    assert_eq!(
        css_forgiving_configured_value_dispatch_report,
        DocumentCommandDispatchReport::new(1, 1, 0)
    );
    assert_eq!(
        css_forgiving_configured_dispatched_values,
        vec![SetAction::Run]
    );
    assert_eq!(
        css_forgiving_configured_value_dispatch_frame
            .output()
            .snapshot()
            .find("inline")
            .unwrap()
            .rect()
            .size
            .width,
        181.0
    );
    assert_eq!(styled_surface.commands().bindings().len(), 2);
    assert!(css_surface_frame.contains_action(&SetAction::Run));
    assert_eq!(
        css_surface_frame
            .output()
            .snapshot()
            .find("inline")
            .unwrap()
            .rect()
            .size
            .width,
        160.0
    );
    assert!(css_mapped_surface_frame.contains_action(&SetAction::Run));
    assert_eq!(
        css_mapped_surface_frame
            .output()
            .snapshot()
            .find("inline")
            .unwrap()
            .rect()
            .size
            .width,
        161.0
    );
    assert!(css_intent_surface_frame.contains_action_for_target_intent(
        "shared",
        ElementBehaviorEvent::ContextMenu,
        &SetAction::Menu
    ));
    assert_eq!(
        css_intent_surface_frame
            .output()
            .snapshot()
            .find("shared")
            .unwrap()
            .rect()
            .size
            .width,
        162.0
    );
    assert!(skipped_css_surface_frame.contains_action(&SetAction::Run));
    assert_eq!(
        skipped_css_surface_frame
            .output()
            .snapshot()
            .find("inline")
            .unwrap()
            .element(),
        Element::Button
    );
    assert!(css_forgiving_surface_frame.contains_action(&SetAction::Run));
    assert_eq!(
        css_forgiving_surface_frame
            .output()
            .snapshot()
            .find("inline")
            .unwrap()
            .rect()
            .size
            .width,
        164.0
    );
    assert!(css_forgiving_mapped_surface_frame.contains_action(&SetAction::Run));
    assert_eq!(
        css_forgiving_mapped_surface_frame
            .output()
            .snapshot()
            .find("inline")
            .unwrap()
            .rect()
            .size
            .width,
        165.0
    );
    assert!(
        css_forgiving_intent_surface_frame.contains_action_for_target_intent(
            "shared",
            ElementBehaviorEvent::ContextMenu,
            &SetAction::Menu
        )
    );
    assert_eq!(
        css_forgiving_intent_surface_frame
            .output()
            .snapshot()
            .find("shared")
            .unwrap()
            .rect()
            .size
            .width,
        166.0
    );
    assert!(skipped_css_forgiving_surface_frame.contains_action(&SetAction::Run));
    assert_eq!(
        skipped_css_forgiving_surface_frame
            .output()
            .snapshot()
            .find("inline")
            .unwrap()
            .element(),
        Element::Button
    );
    assert_eq!(project_report.operations, 2);
    assert_eq!(project_report.changed, 2);
    assert_eq!(projected_inline.text(), Some("Projected".to_owned()));
    assert!(projected_inline.has_class("is-ready"));
    assert_eq!(mapped_project_report.operations, 1);
    assert_eq!(mapped_project_report.changed, 1);
    assert!(mapped_project_frame.contains_action(&SetAction::Run));
    assert_eq!(
        mapped_project_frame
            .output()
            .snapshot()
            .find("inline")
            .unwrap()
            .text(),
        Some("Mapped".to_owned())
    );
    assert_eq!(dispatch_project_report.operations, 1);
    assert_eq!(dispatch_project_report.changed, 1);
    assert!(dispatch_project_frame.contains_action(&SetAction::Run));
    assert_eq!(
        dispatch_project_action_report,
        DocumentCommandDispatchReport::new(1, 1, 0)
    );
    assert_eq!(projected_dispatched, vec![SetAction::Run]);
    assert_eq!(
        dispatch_project_frame
            .output()
            .snapshot()
            .find("inline")
            .unwrap()
            .text(),
        Some("Dispatched".to_owned())
    );
    assert_eq!(value_project_report.operations, 1);
    assert_eq!(value_project_report.changed, 1);
    assert!(value_project_frame.contains_action(&SetAction::Run));
    assert_eq!(
        value_project_action_report,
        DocumentCommandDispatchReport::new(1, 1, 0)
    );
    assert_eq!(projected_value_dispatched, vec![SetAction::Run]);
    assert_eq!(
        value_project_frame
            .output()
            .snapshot()
            .find("inline")
            .unwrap()
            .text(),
        Some("Value dispatched".to_owned())
    );
    assert_eq!(configured_dispatch_project_report.operations, 1);
    assert_eq!(configured_dispatch_project_report.changed, 1);
    assert!(configured_dispatch_project_frame.contains_action(&SetAction::Run));
    assert_eq!(
        configured_dispatch_project_action_report,
        DocumentCommandDispatchReport::new(1, 1, 0)
    );
    assert_eq!(configured_projected_dispatched, vec![SetAction::Run]);
    assert_eq!(
        configured_dispatch_project_frame
            .output()
            .snapshot()
            .find("inline")
            .unwrap()
            .text(),
        Some("Configured dispatch".to_owned())
    );
    assert_eq!(configured_value_project_report.operations, 1);
    assert_eq!(configured_value_project_report.changed, 1);
    assert!(configured_value_project_frame.contains_action(&SetAction::Run));
    assert_eq!(
        configured_value_project_action_report,
        DocumentCommandDispatchReport::new(1, 1, 0)
    );
    assert_eq!(configured_projected_value_dispatched, vec![SetAction::Run]);
    assert_eq!(
        configured_value_project_frame
            .output()
            .snapshot()
            .find("inline")
            .unwrap()
            .text(),
        Some("Configured value".to_owned())
    );
    assert_eq!(intent_project_report.operations, 1);
    assert_eq!(intent_project_report.changed, 1);
    assert!(intent_project_frame.contains_action_for_target_intent(
        "shared",
        ElementBehaviorEvent::ContextMenu,
        &SetAction::Menu
    ));
    assert_eq!(
        intent_project_frame
            .output()
            .snapshot()
            .find("shared")
            .unwrap()
            .text(),
        Some("Menu".to_owned())
    );
    assert_eq!(projected_surface_report.operations, 1);
    assert_eq!(projected_surface_report.changed, 1);
    assert!(projected_surface_frame.contains_action(&SetAction::Run));
    assert_eq!(
        projected_surface_frame
            .output()
            .snapshot()
            .find("inline")
            .unwrap()
            .text(),
        Some("Surface".to_owned())
    );
    assert_eq!(intent_surface_report.operations, 1);
    assert_eq!(intent_surface_report.changed, 1);
    assert!(
        intent_projected_surface_frame.contains_action_for_target_intent(
            "shared",
            ElementBehaviorEvent::ContextMenu,
            &SetAction::Menu
        )
    );
    assert_eq!(
        intent_projected_surface_frame
            .output()
            .snapshot()
            .find("shared")
            .unwrap()
            .text(),
        Some("Surface menu".to_owned())
    );

    fs::write(
        path,
        "<button id=\"file\" on:click=\"file.open\">After</button>",
    )
    .expect("html fixture should update");
    let changed = set.reload_changed().expect("html set should reload");

    assert_eq!(changed, ["file"]);
    assert!(
        set.get("file")
            .expect("file document should exist")
            .find_by_id("file")
            .is_some_and(|node| node.text.as_deref() == Some("After"))
    );
}
