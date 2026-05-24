use des_template::{
    CompileOptions, CompiledTemplate, RenderOptions, RenderedNode, TemplateCompileLimits,
    TemplateFile, TemplateLimits, TemplateSet, TemplateSink, Value,
};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

struct TempTemplatePath {
    path: PathBuf,
}

impl TempTemplatePath {
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

impl Drop for TempTemplatePath {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn node_text(node: &RenderedNode) -> Option<&str> {
    node.text.as_deref()
}

#[test]
fn compiled_template_renders_markup_interpolation_and_loops() {
    let template = CompiledTemplate::compile(
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
    .expect("template should compile");

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

    let rendered = template.render(&context).expect("template should render");

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
fn compiled_template_renders_conditionals() {
    let template = CompiledTemplate::compile(
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
    .expect("template should compile");

    let mut context = BTreeMap::new();
    context.insert("loading".to_owned(), Value::bool(false));

    let rendered = template.render(&context).expect("template should render");

    assert_eq!(rendered[0].children.len(), 1);
    assert_eq!(node_text(&rendered[0].children[0]), Some("Ready"));
}

#[test]
fn template_file_hot_reloads_when_source_changes() {
    let fixture = TempTemplatePath::new("des-template-hot-reload");
    let path = fixture.path();

    fs::write(&path, "<text>{title}</text>").expect("template fixture should be writable");
    let mut file = TemplateFile::load(&path).expect("template file should load");

    let mut context = BTreeMap::new();
    context.insert("title".to_owned(), Value::string("Before"));
    assert_eq!(
        file.compiled()
            .render(&context)
            .expect("template should render")[0]
            .text
            .as_deref(),
        Some("Before")
    );

    std::thread::sleep(Duration::from_millis(5));
    fs::write(
        &path,
        "<panel class=\"changed\"><text>{title}</text></panel>",
    )
    .expect("template fixture should update");

    let status = file
        .reload_if_changed()
        .expect("template file should hot reload");

    assert!(status.changed);
    assert_eq!(
        file.compiled()
            .render(&context)
            .expect("template should render")[0]
            .classes,
        ["changed"]
    );
}

#[test]
fn compiled_template_locks_attribute_class_and_text_normalization() {
    let template = CompiledTemplate::compile(
        r#"
        <panel class="primary  elevated" data-mode="old" data-mode="{mode}">
          <text>  {label}  </text>
        </panel>
        "#,
    )
    .expect("template should compile");
    let mut context = BTreeMap::new();
    context.insert("mode".to_owned(), Value::string("active"));
    context.insert("label".to_owned(), Value::string("Ready"));

    let rendered = template.render(&context).expect("template should render");

    assert_eq!(rendered[0].classes, ["primary", "elevated"]);
    assert_eq!(
        rendered[0].attributes.get("data-mode").map(String::as_str),
        Some("active"),
        "later duplicate attributes intentionally replace earlier values"
    );
    assert_eq!(node_text(&rendered[0].children[0]), Some("Ready"));
}

#[test]
fn compiled_template_renders_markup_like_interpolation_as_text() {
    let template =
        CompiledTemplate::compile(r#"<text>{message}</text>"#).expect("template should compile");
    let mut context = BTreeMap::new();
    context.insert(
        "message".to_owned(),
        Value::string("<strong>not markup</strong>"),
    );

    let rendered = template.render(&context).expect("template should render");

    assert_eq!(node_text(&rendered[0]), Some("<strong>not markup</strong>"));
    assert!(rendered[0].children.is_empty());
}

#[test]
fn compiled_template_resolves_indexed_paths_root_paths_and_loop_metadata() {
    let template = CompiledTemplate::compile(
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
    .expect("template should compile");

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

    let rendered = template.render(&context).expect("template should render");

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
fn compiled_template_enforces_render_limits() {
    let template = CompiledTemplate::compile(
        r#"
        <list>
          {for row in rows}
            <row><text>{row.customer}</text></row>
          {/for}
        </list>
        "#,
    )
    .expect("template should compile");

    let mut first = BTreeMap::new();
    first.insert("customer".to_owned(), Value::string("Acme"));

    let mut second = BTreeMap::new();
    second.insert("customer".to_owned(), Value::string("Globex"));

    let mut context = BTreeMap::new();
    context.insert(
        "rows".to_owned(),
        Value::list([Value::object(first), Value::object(second)]),
    );

    let err = template
        .render_with_options(
            &context,
            &RenderOptions::new().with_limits(TemplateLimits {
                max_loop_iterations: 1,
                ..TemplateLimits::default()
            }),
        )
        .expect_err("second loop item should exceed the limit");

    assert!(err.to_string().contains("loop iteration limit"));
}

#[test]
fn compiled_template_can_render_into_a_sink() {
    #[derive(Default)]
    struct CountingSink {
        count: usize,
    }

    impl TemplateSink for CountingSink {
        type Output = usize;

        fn element(&mut self, node: RenderedNode) -> des_template::TemplateResult<()> {
            self.count += 1 + count_children(&node);
            Ok(())
        }

        fn finish(self) -> Self::Output {
            self.count
        }
    }

    fn count_children(node: &RenderedNode) -> usize {
        node.children
            .iter()
            .map(|child| 1 + count_children(child))
            .sum()
    }

    let template = CompiledTemplate::compile("<panel><text>Hello</text><text>World</text></panel>")
        .expect("template should compile");

    let count = template
        .render_into(&BTreeMap::new(), CountingSink::default())
        .expect("template should render into sink");

    assert_eq!(count, 3);
}

#[test]
fn compiled_template_render_into_stops_when_sink_errors() {
    #[derive(Default)]
    struct StopSink;

    impl TemplateSink for StopSink {
        type Output = ();

        fn element(&mut self, _node: RenderedNode) -> des_template::TemplateResult<()> {
            Err(des_template::TemplateError::Render(
                "sink stopped".to_owned(),
            ))
        }

        fn finish(self) -> Self::Output {}
    }

    let template = CompiledTemplate::compile("<text>First</text><text>{missing}</text>")
        .expect("template should compile");

    let err = template
        .render_into(&BTreeMap::new(), StopSink)
        .expect_err("sink error should stop before second node renders");

    assert!(err.to_string().contains("sink stopped"));
}

#[test]
fn compiled_template_rejects_oversized_attribute_before_element_is_emitted() {
    let template = CompiledTemplate::compile("<panel class=\"{class_name}\"/>")
        .expect("template should compile");
    let mut context = BTreeMap::new();
    context.insert("class_name".to_owned(), Value::string("x".repeat(16)));

    let err = template
        .render_with_options(
            &context,
            &RenderOptions::new().with_limits(TemplateLimits {
                max_attribute_bytes: 4,
                ..TemplateLimits::default()
            }),
        )
        .expect_err("attribute should exceed configured limit");

    assert!(err.to_string().contains("attribute byte limit"));
}

#[test]
fn compiled_template_compile_options_reject_excessive_nesting() {
    let source = "<a><b><c/></b></a>";

    let err = CompiledTemplate::compile_with_options(
        source,
        &CompileOptions::new().with_limits(TemplateCompileLimits {
            max_depth: 1,
            ..TemplateCompileLimits::default()
        }),
    )
    .expect_err("nested element compile should exceed configured depth");

    assert!(err.to_string().contains("compile depth limit"));
}

#[test]
fn compiled_template_source_limit_handles_utf8_boundary() {
    let err = CompiledTemplate::compile_with_options(
        "éé",
        &CompileOptions::new().with_limits(TemplateCompileLimits {
            max_source_bytes: 1,
            ..TemplateCompileLimits::default()
        }),
    )
    .expect_err("source limit should return an error instead of panicking");

    assert!(err.to_string().contains("compile source byte limit"));
}

#[test]
fn compiled_template_compile_options_limit_inline_conditional_depth() {
    let source = "<text>{if flag}{if flag}nested{/if}{/if}</text>";

    let err = CompiledTemplate::compile_with_options(
        source,
        &CompileOptions::new().with_limits(TemplateCompileLimits {
            max_depth: 1,
            ..TemplateCompileLimits::default()
        }),
    )
    .expect_err("nested inline conditional should exceed configured depth");

    assert!(err.to_string().contains("compile depth limit"));
}

#[test]
fn compiled_template_loop_limit_is_checked_before_rendering_items() {
    let template = CompiledTemplate::compile("{for row in rows}<text>{row.missing}</text>{/for}")
        .expect("template should compile");

    let mut row = BTreeMap::new();
    row.insert("payload".to_owned(), Value::string("large".repeat(1_000)));

    let mut context = BTreeMap::new();
    context.insert("rows".to_owned(), Value::list([Value::object(row)]));

    let err = template
        .render_with_options(
            &context,
            &RenderOptions::new().with_limits(TemplateLimits {
                max_loop_iterations: 0,
                ..TemplateLimits::default()
            }),
        )
        .expect_err("loop limit should fail before rendering row body");

    assert!(err.to_string().contains("loop iteration limit"));
}

#[test]
fn template_file_hot_reload_detects_same_mtime_content_changes() {
    let path = std::env::temp_dir().join(format!(
        "des-template-hot-reload-fingerprint-{}.xml",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after epoch")
            .as_nanos()
    ));

    fs::write(&path, "<text>Before</text>").expect("template fixture should be writable");
    let mut file = TemplateFile::load(&path).expect("template file should load");
    let original_modified = fs::metadata(&path)
        .expect("template fixture should have metadata")
        .modified()
        .ok();

    fs::write(&path, "<text>After</text>").expect("template fixture should update");
    if let Some(modified) = original_modified {
        let filetime = filetime::FileTime::from_system_time(modified);
        filetime::set_file_mtime(&path, filetime).expect("mtime should be restorable");
    }

    let status = file
        .reload_if_changed()
        .expect("template file should hot reload");

    assert!(status.changed);
    assert_eq!(
        file.compiled()
            .render(&BTreeMap::new())
            .expect("template should render")[0]
            .text
            .as_deref(),
        Some("After")
    );

    let _ = fs::remove_file(path);
}

#[test]
fn template_set_manages_named_compiled_and_file_backed_templates() {
    let path = std::env::temp_dir().join(format!(
        "des-template-set-{}.xml",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after epoch")
            .as_nanos()
    ));
    fs::write(&path, "<text>{title}</text>").expect("template fixture should be writable");

    let mut set = TemplateSet::new();
    set.add_template("inline", "<panel><text>{title}</text></panel>")
        .expect("inline template should compile");
    set.add_file("file", &path)
        .expect("file template should compile");

    let mut context = BTreeMap::new();
    context.insert("title".to_owned(), Value::string("Before"));

    assert_eq!(
        set.render("inline", &context)
            .expect("inline template should render")[0]
            .children[0]
            .text
            .as_deref(),
        Some("Before")
    );
    assert_eq!(
        set.render("file", &context)
            .expect("file template should render")[0]
            .text
            .as_deref(),
        Some("Before")
    );

    fs::write(&path, "<text>After</text>").expect("template fixture should update");
    let changed = set.reload_changed().expect("template set should reload");

    assert_eq!(changed, ["file"]);
    assert_eq!(
        set.render("file", &context)
            .expect("file template should render")[0]
            .text
            .as_deref(),
        Some("After")
    );

    let _ = fs::remove_file(path);
}

#[test]
fn template_set_reload_changed_is_atomic_when_a_reload_fails() {
    let valid_path = std::env::temp_dir().join(format!(
        "des-template-set-valid-{}.xml",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after epoch")
            .as_nanos()
    ));
    let invalid_path = std::env::temp_dir().join(format!(
        "des-template-set-invalid-{}.xml",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after epoch")
            .as_nanos()
    ));

    fs::write(&valid_path, "<text>Before</text>").expect("valid fixture should be writable");
    fs::write(&invalid_path, "<text>Still valid</text>")
        .expect("invalid fixture should start valid");

    let mut set = TemplateSet::new();
    set.add_file("valid", &valid_path)
        .expect("valid file should load");
    set.add_file("invalid", &invalid_path)
        .expect("invalid file should initially load");

    fs::write(&valid_path, "<text>After</text>").expect("valid fixture should update");
    fs::write(&invalid_path, "<text>Broken").expect("invalid fixture should update");

    let err = set
        .reload_changed()
        .expect_err("one broken template should fail the reload batch");
    assert!(err.to_string().contains("missing closing tag"));

    assert_eq!(
        set.render("valid", &BTreeMap::new())
            .expect("previous valid template should remain active")[0]
            .text
            .as_deref(),
        Some("Before")
    );

    let _ = fs::remove_file(valid_path);
    let _ = fs::remove_file(invalid_path);
}

#[test]
fn compiled_template_rejects_malformed_paths() {
    for source in [
        "<text>{.title}</text>",
        "<text>{row..customer}</text>",
        "<text>{row.}</text>",
        "<text>{items.[0]}</text>",
    ] {
        let err = CompiledTemplate::compile(source).expect_err("path should be rejected");
        assert!(
            err.to_string().contains("path"),
            "unexpected error for {source}: {err}"
        );
    }
}

#[test]
fn compiled_template_renders_nested_inline_conditionals() {
    let template = CompiledTemplate::compile(
        r#"<text>{if enabled}{if selected}selected{else}enabled{/if}{else}disabled{/if}</text>"#,
    )
    .expect("template should compile");

    let mut context = BTreeMap::new();
    context.insert("enabled".to_owned(), Value::bool(true));
    context.insert("selected".to_owned(), Value::bool(false));

    let rendered = template.render(&context).expect("template should render");

    assert_eq!(node_text(&rendered[0]), Some("enabled"));
}

#[test]
fn compiled_template_reports_absolute_line_and_column_for_sliced_errors() {
    let err = CompiledTemplate::compile(
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
fn compiled_template_reports_trimmed_expression_line_and_column() {
    let err = CompiledTemplate::compile("<text>{   row..customer}</text>")
        .expect_err("malformed path should fail");

    let message = err.to_string();
    assert!(
        message.contains("1:14"),
        "expected trimmed expression column in error, got {message}"
    );
}

#[test]
fn compiled_template_renders_large_integer_like_numbers_without_saturating() {
    let template = CompiledTemplate::compile("<text>{n}</text>").expect("template should compile");
    let mut context = BTreeMap::new();
    context.insert("n".to_owned(), Value::number(1e20));

    let rendered = template.render(&context).expect("template should render");

    assert_eq!(node_text(&rendered[0]), Some("100000000000000000000"));
}
