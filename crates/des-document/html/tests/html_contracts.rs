use des_document::{
    DocumentCommandRegistry, DocumentEngine, DocumentInput, Element, ElementId, Point,
    PointerInput, Size,
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

    let output = view.update_with_input(DocumentInput::pointer(PointerInput {
        position: Point::new(8.0, 8.0),
        primary_delta: Point::ZERO,
        primary_down: true,
        primary_pressed: false,
        primary_clicked: true,
        primary_click_count: 1,
        secondary_clicked: false,
        time_seconds: 0.0,
    }));
    let run = output.snapshot().find("run").unwrap();

    assert_eq!(run.rect().size.width, 96.0);
    assert!(run.interactive());
    assert_eq!(run.behavior_hooks()[0].command, "run");
    assert_eq!(output.commands()[0].command, "run");
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
    let output = view.update_with_input(DocumentInput::pointer(PointerInput {
        position: Point::new(8.0, 8.0),
        primary_delta: Point::ZERO,
        primary_down: true,
        primary_pressed: false,
        primary_clicked: true,
        primary_click_count: 1,
        secondary_clicked: false,
        time_seconds: 0.0,
    }));
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
