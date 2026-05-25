use des_document::{
    DocumentCommandRegistry, DocumentEngine, DocumentInput, DocumentKey, DocumentProjection,
    Element, ElementBehaviorEvent, ElementId, Point, Size,
};
use des_html::{HtmlDiagnosticCode, HtmlDocument, HtmlFile, HtmlNode, HtmlSet, HtmlStylesheet};
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
    assert_eq!(button.behavior_hooks[1].event, "input");
    assert_eq!(button.behavior_hooks[1].command, "project.filter");
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
    let mut view = html
        .to_view_with_css(
            Size::new(240.0, 160.0),
            r#".card { width: 120px; height: 40px; }"#,
        )
        .expect("HTML and CSS should compose directly into a view");

    let output = view.update();
    let panel = output.snapshot().find("panel").unwrap();

    assert_eq!(bundle.stylesheet.rule_count(), 1);
    assert_eq!(panel.rect().size.width, 120.0);
    assert_eq!(panel.rect().size.height, 40.0);
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
fn html_authored_commands_dispatch_to_typed_rust_actions() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum HtmlAction {
        Run,
    }

    let bundle = HtmlStylesheet::parse_fragment(
        r#"<button id="run" class="primary" data-command="project.run">Run</button>"#,
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

    assert_eq!(
        output.snapshot().find("panel").unwrap().rect().size.width,
        220.0
    );
    assert!(click_frame.contains_action(&HtmlAction::Run));
    assert!(key_frame.contains_action(&HtmlAction::Filter));
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

    assert_eq!(report.operations, 4);
    assert_eq!(report.changed, 4);
    assert_eq!(run.text(), Some("Running".to_owned()));
    assert_eq!(run.aria("pressed"), Some("true"));
    assert!(frame.contains_action(&HtmlAction::Run));
    assert!(!frame.contains_action(&HtmlAction::Select));
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

    let bundle = HtmlStylesheet::parse_fragment(
        r#"<button id="run" class="primary" on:click="project.run">Run</button>"#,
        r#".primary { width: 96px; height: 32px; }"#,
    )
    .expect("HTML and CSS should compile together from the prelude");
    assert!(bundle.html().is_clean());
    assert_eq!(bundle.html().children()[0].id.as_deref(), Some("run"));
    assert_eq!(bundle.stylesheet().rule_count(), 1);

    let registry = DocumentCommandRegistry::new().bind("project.run", HtmlAction::Run);
    let mut view = bundle
        .into_view(Size::new(320.0, 180.0))
        .expect("prelude-authored HTML should create a document view");
    let frame: DocumentActionFrame<HtmlAction> = view.update_with_input_actions(
        DocumentInput::primary_click(Point::new(8.0, 8.0)),
        &registry,
    );
    let run = frame.output.snapshot().find("run").unwrap();

    assert_eq!(run.rect().size.width, 96.0);
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
    assert!(file.document().children.iter().any(|node| {
        node.find_by_id("status")
            .is_some_and(|node| node.text.as_deref() == Some("Before"))
    }));

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
        file.document()
            .children
            .iter()
            .any(|node| node.find_by_id("status").is_some_and(|node| {
                node.classes == ["changed"] && node.text.as_deref() == Some("After")
            }))
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
    assert!(file.document().children.iter().any(|node| {
        node.find_by_id("status")
            .is_some_and(|node| node.text.as_deref() == Some("After"))
    }));
}

#[test]
fn html_set_manages_named_inline_and_file_backed_documents() {
    let fixture = TempHtmlPath::new("des-html-set", "html");
    let path = fixture.path();
    fs::write(path, "<section id=\"file\">Before</section>")
        .expect("html fixture should be writable");

    let mut set = HtmlSet::new();
    set.add_fragment("inline", "<section id=\"inline\">Inline</section>")
        .expect("inline html should parse");
    set.add_file("file", path).expect("file html should parse");

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
            .children
            .iter()
            .any(|node| node.find_by_id("file").is_some())
    );

    fs::write(path, "<section id=\"file\">After</section>").expect("html fixture should update");
    let changed = set.reload_changed().expect("html set should reload");

    assert_eq!(changed, ["file"]);
    assert!(
        set.get("file")
            .expect("file document should exist")
            .children
            .iter()
            .any(|node| node
                .find_by_id("file")
                .is_some_and(|node| node.text.as_deref() == Some("After")))
    );
}

trait HtmlNodeTestExt {
    fn find_by_id(&self, id: &str) -> Option<&HtmlNode>;
}

impl HtmlNodeTestExt for HtmlNode {
    fn find_by_id(&self, id: &str) -> Option<&HtmlNode> {
        if self.id.as_deref() == Some(id) {
            return Some(self);
        }
        self.children.iter().find_map(|child| child.find_by_id(id))
    }
}
