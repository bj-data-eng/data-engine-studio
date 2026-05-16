use des_template::{CompiledTemplate, TemplateFile, Value};
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
