use des_template::{
    CompiledTemplate, RenderOptions, RenderedNode, TemplateFile, TemplateLimits, TemplateSet,
    TemplateSink, Value,
};
use std::collections::BTreeMap;
use std::fs;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

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
    assert_eq!(rendered[0].children[0].text.as_deref(), Some("Open orders"));
    assert_eq!(rendered[0].children[1].tag, "list");
    assert_eq!(rendered[0].children[1].children.len(), 2);
    assert_eq!(
        rendered[0].children[1].children[0].children[0]
            .text
            .as_deref(),
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
    assert_eq!(rendered[0].children[0].text.as_deref(), Some("Ready"));
}

#[test]
fn template_file_hot_reloads_when_source_changes() {
    let path = std::env::temp_dir().join(format!(
        "des-template-hot-reload-{}.xml",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after epoch")
            .as_nanos()
    ));

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

    let _ = fs::remove_file(path);
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
        rendered[0].children[0].children[0].text.as_deref(),
        Some("Orders:1/2:true:false:Acme:Acme")
    );
    assert_eq!(rendered[0].children[1].classes, ["inactive"]);
    assert_eq!(
        rendered[0].children[1].children[0].text.as_deref(),
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
