use des_document::{DocumentEngine, Element, Size};
use des_html::{
    CompileOptions, CompiledHtml, HtmlCompileLimits, HtmlDocument, HtmlFile, HtmlNode,
    HtmlRenderLimits, HtmlSet, HtmlSink, HtmlStylesheet, RenderOptions, Value,
};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

struct TempHtmlPath {
    path: PathBuf,
}

impl TempHtmlPath {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "{name}-{}.xml",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system clock should be after epoch")
                .as_nanos()
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
        <main id="app" class="shell primary" data-mode="demo">
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
    assert_eq!(
        main.attributes.get("data-mode").map(String::as_str),
        Some("demo")
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
fn html_document_emits_typed_document_nodes_with_stable_ids() {
    let html = HtmlDocument::parse_fragment(
        r#"<main id="app" class="shell"><button id="run" class="primary" on:click="run">Run</button><p>Ready</p></main>"#,
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
fn compiled_html_renders_markup_interpolation_and_loops() {
    let html = CompiledHtml::compile(
        r#"
        <panel class="orders-panel">
          <text>{title}</text>
          <list>
            {for row in rows}
              <row class="order-row">
                <text>{row.customer}</text>
                <text>{row.total}</text>
              </row>
            {/for}
          </list>
        </panel>
        "#,
    )
    .expect("html should compile");

    let mut first = BTreeMap::new();
    first.insert("customer".to_owned(), Value::string("Acme"));
    first.insert("total".to_owned(), Value::number(42.5));

    let mut second = BTreeMap::new();
    second.insert("customer".to_owned(), Value::string("Globex"));
    second.insert("total".to_owned(), Value::number(10.0));

    let mut context = BTreeMap::new();
    context.insert("title".to_owned(), Value::string("Open orders"));
    context.insert(
        "rows".to_owned(),
        Value::list([Value::object(first), Value::object(second)]),
    );

    let rendered = html.render(&context).expect("html should render");

    assert_eq!(rendered.len(), 1);
    assert_eq!(rendered[0].tag, "panel");
    assert_eq!(rendered[0].classes, ["orders-panel"]);
    assert_eq!(rendered[0].children[0].tag, "text");
    assert_eq!(node_text(&rendered[0].children[0]), Some("Open orders"));
    assert_eq!(rendered[0].children[1].tag, "list");
    assert_eq!(rendered[0].children[1].children.len(), 2);
    assert_eq!(
        rendered[0].children[1].children[0].children[0]
            .text
            .as_ref()
            .map(String::as_str),
        Some("Acme")
    );
    assert_eq!(
        rendered[0].children[1].children[1].children[1]
            .text
            .as_deref(),
        Some("10")
    );
}

#[test]
fn compiled_html_renders_conditionals() {
    let html = CompiledHtml::compile(
        r#"
        <panel>
          {if loading}
            <text>Loading</text>
          {else}
            <text>Ready</text>
          {/if}
        </panel>
        "#,
    )
    .expect("html should compile");

    let mut context = BTreeMap::new();
    context.insert("loading".to_owned(), Value::bool(false));

    let rendered = html.render(&context).expect("html should render");

    assert_eq!(rendered[0].children.len(), 1);
    assert_eq!(node_text(&rendered[0].children[0]), Some("Ready"));
}

#[test]
fn html_file_hot_reloads_when_source_changes() {
    let fixture = TempHtmlPath::new("des-html-hot-reload");
    let path = fixture.path();

    fs::write(&path, "<text>{title}</text>").expect("html fixture should be writable");
    let mut file = HtmlFile::load(&path).expect("html file should load");

    let mut context = BTreeMap::new();
    context.insert("title".to_owned(), Value::string("Before"));
    assert_eq!(
        file.compiled()
            .render(&context)
            .expect("html should render")[0]
            .text
            .as_deref(),
        Some("Before")
    );

    std::thread::sleep(Duration::from_millis(5));
    fs::write(
        &path,
        "<panel class=\"changed\"><text>{title}</text></panel>",
    )
    .expect("html fixture should update");

    let status = file
        .reload_if_changed()
        .expect("html file should hot reload");

    assert!(status.changed);
    assert_eq!(
        file.compiled()
            .render(&context)
            .expect("html should render")[0]
            .classes,
        ["changed"]
    );
}

#[test]
fn compiled_html_locks_attribute_class_and_text_normalization() {
    let html = CompiledHtml::compile(
        r#"
        <panel class="primary  elevated" data-mode="old" data-mode="{mode}">
          <text>  {label}  </text>
        </panel>
        "#,
    )
    .expect("html should compile");
    let mut context = BTreeMap::new();
    context.insert("mode".to_owned(), Value::string("active"));
    context.insert("label".to_owned(), Value::string("Ready"));

    let rendered = html.render(&context).expect("html should render");

    assert_eq!(rendered[0].classes, ["primary", "elevated"]);
    assert_eq!(
        rendered[0].attributes.get("data-mode").map(String::as_str),
        Some("active"),
        "later duplicate attributes intentionally replace earlier values"
    );
    assert_eq!(node_text(&rendered[0].children[0]), Some("Ready"));
}

#[test]
fn compiled_html_renders_markup_like_interpolation_as_text() {
    let html = CompiledHtml::compile(r#"<text>{message}</text>"#).expect("html should compile");
    let mut context = BTreeMap::new();
    context.insert(
        "message".to_owned(),
        Value::string("<strong>not markup</strong>"),
    );

    let rendered = html.render(&context).expect("html should render");

    assert_eq!(node_text(&rendered[0]), Some("<strong>not markup</strong>"));
    assert!(rendered[0].children.is_empty());
}

#[test]
fn compiled_html_resolves_indexed_paths_root_paths_and_loop_metadata() {
    let html = CompiledHtml::compile(
        r#"
        <list>
          {for row in rows}
            <row class="{if row.active}active{else}inactive{/if}">
              <text>{@root.title}:{loop.index}/{loop.len}:{loop.first}:{loop.last}:{rows[0].customer}:{row.customer}</text>
            </row>
          {/for}
        </list>
        "#,
    )
    .expect("html should compile");

    let mut first = BTreeMap::new();
    first.insert("customer".to_owned(), Value::string("Acme"));
    first.insert("active".to_owned(), Value::bool(true));

    let mut second = BTreeMap::new();
    second.insert("customer".to_owned(), Value::string("Globex"));
    second.insert("active".to_owned(), Value::bool(false));

    let mut context = BTreeMap::new();
    context.insert("title".to_owned(), Value::string("Orders"));
    context.insert(
        "rows".to_owned(),
        Value::list([Value::object(first), Value::object(second)]),
    );

    let rendered = html.render(&context).expect("html should render");

    assert_eq!(rendered[0].children[0].classes, ["active"]);
    assert_eq!(
        node_text(&rendered[0].children[0].children[0]),
        Some("Orders:1/2:true:false:Acme:Acme")
    );
    assert_eq!(rendered[0].children[1].classes, ["inactive"]);
    assert_eq!(
        node_text(&rendered[0].children[1].children[0]),
        Some("Orders:2/2:false:true:Acme:Globex")
    );
}

#[test]
fn compiled_html_enforces_render_limits() {
    let html = CompiledHtml::compile(
        r#"
        <list>
          {for row in rows}
            <row><text>{row.customer}</text></row>
          {/for}
        </list>
        "#,
    )
    .expect("html should compile");

    let mut first = BTreeMap::new();
    first.insert("customer".to_owned(), Value::string("Acme"));

    let mut second = BTreeMap::new();
    second.insert("customer".to_owned(), Value::string("Globex"));

    let mut context = BTreeMap::new();
    context.insert(
        "rows".to_owned(),
        Value::list([Value::object(first), Value::object(second)]),
    );

    let err = html
        .render_with_options(
            &context,
            &RenderOptions::new().with_limits(HtmlRenderLimits {
                max_loop_iterations: 1,
                ..HtmlRenderLimits::default()
            }),
        )
        .expect_err("second loop item should exceed the limit");

    assert!(err.to_string().contains("loop iteration limit"));
}

#[test]
fn compiled_html_can_render_into_a_sink() {
    #[derive(Default)]
    struct CountingSink {
        count: usize,
    }

    impl HtmlSink for CountingSink {
        type Output = usize;

        fn element(&mut self, node: HtmlNode) -> des_html::HtmlResult<()> {
            self.count += 1 + count_children(&node);
            Ok(())
        }

        fn finish(self) -> Self::Output {
            self.count
        }
    }

    fn count_children(node: &HtmlNode) -> usize {
        node.children
            .iter()
            .map(|child| 1 + count_children(child))
            .sum()
    }

    let html = CompiledHtml::compile("<panel><text>Hello</text><text>World</text></panel>")
        .expect("html should compile");

    let count = html
        .render_into(&BTreeMap::new(), CountingSink::default())
        .expect("html should render into sink");

    assert_eq!(count, 3);
}

#[test]
fn compiled_html_render_into_stops_when_sink_errors() {
    #[derive(Default)]
    struct StopSink;

    impl HtmlSink for StopSink {
        type Output = ();

        fn element(&mut self, _node: HtmlNode) -> des_html::HtmlResult<()> {
            Err(des_html::HtmlError::Render("sink stopped".to_owned()))
        }

        fn finish(self) -> Self::Output {}
    }

    let html = CompiledHtml::compile("<text>First</text><text>{missing}</text>")
        .expect("html should compile");

    let err = html
        .render_into(&BTreeMap::new(), StopSink)
        .expect_err("sink error should stop before second node renders");

    assert!(err.to_string().contains("sink stopped"));
}

#[test]
fn compiled_html_rejects_oversized_attribute_before_element_is_emitted() {
    let html =
        CompiledHtml::compile("<panel class=\"{class_name}\"/>").expect("html should compile");
    let mut context = BTreeMap::new();
    context.insert("class_name".to_owned(), Value::string("x".repeat(16)));

    let err = html
        .render_with_options(
            &context,
            &RenderOptions::new().with_limits(HtmlRenderLimits {
                max_attribute_bytes: 4,
                ..HtmlRenderLimits::default()
            }),
        )
        .expect_err("attribute should exceed configured limit");

    assert!(err.to_string().contains("attribute byte limit"));
}

#[test]
fn compiled_html_compile_options_reject_excessive_nesting() {
    let source = "<a><b><c/></b></a>";

    let err = CompiledHtml::compile_with_options(
        source,
        &CompileOptions::new().with_limits(HtmlCompileLimits {
            max_depth: 1,
            ..HtmlCompileLimits::default()
        }),
    )
    .expect_err("nested element compile should exceed configured depth");

    assert!(err.to_string().contains("compile depth limit"));
}

#[test]
fn compiled_html_source_limit_handles_utf8_boundary() {
    let err = CompiledHtml::compile_with_options(
        "éé",
        &CompileOptions::new().with_limits(HtmlCompileLimits {
            max_source_bytes: 1,
            ..HtmlCompileLimits::default()
        }),
    )
    .expect_err("source limit should return an error instead of panicking");

    assert!(err.to_string().contains("compile source byte limit"));
}

#[test]
fn compiled_html_compile_options_limit_inline_conditional_depth() {
    let source = "<text>{if flag}{if flag}nested{/if}{/if}</text>";

    let err = CompiledHtml::compile_with_options(
        source,
        &CompileOptions::new().with_limits(HtmlCompileLimits {
            max_depth: 1,
            ..HtmlCompileLimits::default()
        }),
    )
    .expect_err("nested inline conditional should exceed configured depth");

    assert!(err.to_string().contains("compile depth limit"));
}

#[test]
fn compiled_html_loop_limit_is_checked_before_rendering_items() {
    let html = CompiledHtml::compile("{for row in rows}<text>{row.missing}</text>{/for}")
        .expect("html should compile");

    let mut row = BTreeMap::new();
    row.insert("payload".to_owned(), Value::string("large".repeat(1_000)));

    let mut context = BTreeMap::new();
    context.insert("rows".to_owned(), Value::list([Value::object(row)]));

    let err = html
        .render_with_options(
            &context,
            &RenderOptions::new().with_limits(HtmlRenderLimits {
                max_loop_iterations: 0,
                ..HtmlRenderLimits::default()
            }),
        )
        .expect_err("loop limit should fail before rendering row body");

    assert!(err.to_string().contains("loop iteration limit"));
}

#[test]
fn html_file_hot_reload_detects_same_mtime_content_changes() {
    let path = std::env::temp_dir().join(format!(
        "des-html-hot-reload-fingerprint-{}.xml",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after epoch")
            .as_nanos()
    ));

    fs::write(&path, "<text>Before</text>").expect("html fixture should be writable");
    let mut file = HtmlFile::load(&path).expect("html file should load");
    let original_modified = fs::metadata(&path)
        .expect("html fixture should have metadata")
        .modified()
        .ok();

    fs::write(&path, "<text>After</text>").expect("html fixture should update");
    if let Some(modified) = original_modified {
        let filetime = filetime::FileTime::from_system_time(modified);
        filetime::set_file_mtime(&path, filetime).expect("mtime should be restorable");
    }

    let status = file
        .reload_if_changed()
        .expect("html file should hot reload");

    assert!(status.changed);
    assert_eq!(
        file.compiled()
            .render(&BTreeMap::new())
            .expect("html should render")[0]
            .text
            .as_deref(),
        Some("After")
    );

    let _ = fs::remove_file(path);
}

#[test]
fn html_set_manages_named_compiled_and_file_backed_htmls() {
    let path = std::env::temp_dir().join(format!(
        "des-html-set-{}.xml",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after epoch")
            .as_nanos()
    ));
    fs::write(&path, "<text>{title}</text>").expect("html fixture should be writable");

    let mut set = HtmlSet::new();
    set.add_html("inline", "<panel><text>{title}</text></panel>")
        .expect("inline html should compile");
    set.add_file("file", &path)
        .expect("file html should compile");

    let mut context = BTreeMap::new();
    context.insert("title".to_owned(), Value::string("Before"));

    assert_eq!(
        set.render("inline", &context)
            .expect("inline html should render")[0]
            .children[0]
            .text
            .as_deref(),
        Some("Before")
    );
    assert_eq!(
        set.render("file", &context)
            .expect("file html should render")[0]
            .text
            .as_deref(),
        Some("Before")
    );

    fs::write(&path, "<text>After</text>").expect("html fixture should update");
    let changed = set.reload_changed().expect("html set should reload");

    assert_eq!(changed, ["file"]);
    assert_eq!(
        set.render("file", &context)
            .expect("file html should render")[0]
            .text
            .as_deref(),
        Some("After")
    );

    let _ = fs::remove_file(path);
}

#[test]
fn html_set_reload_changed_is_atomic_when_a_reload_fails() {
    let valid_path = std::env::temp_dir().join(format!(
        "des-html-set-valid-{}.xml",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after epoch")
            .as_nanos()
    ));
    let invalid_path = std::env::temp_dir().join(format!(
        "des-html-set-invalid-{}.xml",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after epoch")
            .as_nanos()
    ));

    fs::write(&valid_path, "<text>Before</text>").expect("valid fixture should be writable");
    fs::write(&invalid_path, "<text>Still valid</text>")
        .expect("invalid fixture should start valid");

    let mut set = HtmlSet::new();
    set.add_file("valid", &valid_path)
        .expect("valid file should load");
    set.add_file("invalid", &invalid_path)
        .expect("invalid file should initially load");

    fs::write(&valid_path, "<text>After</text>").expect("valid fixture should update");
    fs::write(&invalid_path, "<text>Broken").expect("invalid fixture should update");

    let err = set
        .reload_changed()
        .expect_err("one broken html should fail the reload batch");
    assert!(err.to_string().contains("missing closing tag"));

    assert_eq!(
        set.render("valid", &BTreeMap::new())
            .expect("previous valid html should remain active")[0]
            .text
            .as_deref(),
        Some("Before")
    );

    let _ = fs::remove_file(valid_path);
    let _ = fs::remove_file(invalid_path);
}

#[test]
fn compiled_html_rejects_malformed_paths() {
    for source in [
        "<text>{.title}</text>",
        "<text>{row..customer}</text>",
        "<text>{row.}</text>",
        "<text>{items.[0]}</text>",
    ] {
        let err = CompiledHtml::compile(source).expect_err("path should be rejected");
        assert!(
            err.to_string().contains("path"),
            "unexpected error for {source}: {err}"
        );
    }
}

#[test]
fn compiled_html_renders_nested_inline_conditionals() {
    let html = CompiledHtml::compile(
        r#"<text>{if enabled}{if selected}selected{else}enabled{/if}{else}disabled{/if}</text>"#,
    )
    .expect("html should compile");

    let mut context = BTreeMap::new();
    context.insert("enabled".to_owned(), Value::bool(true));
    context.insert("selected".to_owned(), Value::bool(false));

    let rendered = html.render(&context).expect("html should render");

    assert_eq!(node_text(&rendered[0]), Some("enabled"));
}

#[test]
fn compiled_html_reports_absolute_line_and_column_for_sliced_errors() {
    let err = CompiledHtml::compile(
        r#"
<panel>
  <text>{row..customer}</text>
</panel>
"#,
    )
    .expect_err("malformed nested path should fail");

    let message = err.to_string();
    assert!(
        message.contains("3:13"),
        "expected absolute line/column in error, got {message}"
    );
}

#[test]
fn compiled_html_reports_trimmed_expression_line_and_column() {
    let err = CompiledHtml::compile("<text>{   row..customer}</text>")
        .expect_err("malformed path should fail");

    let message = err.to_string();
    assert!(
        message.contains("1:14"),
        "expected trimmed expression column in error, got {message}"
    );
}

#[test]
fn compiled_html_renders_large_integer_like_numbers_without_saturating() {
    let html = CompiledHtml::compile("<text>{n}</text>").expect("html should compile");
    let mut context = BTreeMap::new();
    context.insert("n".to_owned(), Value::number(1e20));

    let rendered = html.render(&context).expect("html should render");

    assert_eq!(node_text(&rendered[0]), Some("100000000000000000000"));
}
