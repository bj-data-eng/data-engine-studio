//! Runtime HTML language for Data Engine Studio document markup.
//!
//! The crate parses browser-grade HTML into a reusable document structure and
//! also supports a constrained dynamic HTML authoring layer for loops and
//! interpolation. HTML rendering is pure: it does not perform IO, call arbitrary
//! code, or mutate app state while rendering.

use des_document::{
    Document, DocumentBuilder, Element, ElementSpec, Size, StyleSheet, TextContent,
};
use html5ever::tendril::TendrilSink;
use html5ever::{QualName, local_name, ns, parse_document, parse_fragment};
use markup5ever_rcdom::{Handle, NodeData, RcDom};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::collections::hash_map::DefaultHasher;
use std::fmt;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Convenient result type for html operations.
pub type HtmlResult<T> = Result<T, HtmlError>;

/// Html parser, renderer, and hot-reload errors.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlError {
    /// The html source is syntactically invalid.
    Parse {
        offset: usize,
        line: usize,
        column: usize,
        message: String,
    },
    /// Rendering referenced data that does not exist or has the wrong shape.
    Render(String),
    /// The html file could not be read or inspected.
    Io(String),
}

impl fmt::Display for HtmlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse {
                offset,
                line,
                column,
                message,
            } => {
                write!(
                    f,
                    "html parse error at {line}:{column} (offset {offset}): {message}"
                )
            }
            Self::Render(message) => write!(f, "html render error: {message}"),
            Self::Io(message) => write!(f, "html io error: {message}"),
        }
    }
}

impl std::error::Error for HtmlError {}

impl From<std::io::Error> for HtmlError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error.to_string())
    }
}

/// Runtime value made available to dynamic HTML.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Value {
    /// Absence of a value.
    Null,
    /// Boolean value.
    Bool(bool),
    /// Floating-point numeric value.
    Number(f64),
    /// UTF-8 string value.
    String(String),
    /// Ordered list value.
    List(Vec<Value>),
    /// Object value addressable by field name.
    Object(BTreeMap<String, Value>),
}

impl Value {
    /// Creates a boolean value.
    pub fn bool(value: bool) -> Self {
        Self::Bool(value)
    }

    /// Creates a numeric value.
    pub fn number(value: impl Into<f64>) -> Self {
        Self::Number(value.into())
    }

    /// Creates a string value.
    pub fn string(value: impl Into<String>) -> Self {
        Self::String(value.into())
    }

    /// Creates a list value.
    pub fn list(values: impl IntoIterator<Item = Value>) -> Self {
        Self::List(values.into_iter().collect())
    }

    /// Creates an object value.
    pub fn object(values: BTreeMap<String, Value>) -> Self {
        Self::Object(values)
    }

    fn truthy(&self) -> bool {
        match self {
            Self::Null => false,
            Self::Bool(value) => *value,
            Self::Number(value) => *value != 0.0,
            Self::String(value) => !value.is_empty(),
            Self::List(value) => !value.is_empty(),
            Self::Object(value) => !value.is_empty(),
        }
    }

    fn render_scalar(&self) -> HtmlResult<String> {
        match self {
            Self::Null => Ok(String::new()),
            Self::Bool(value) => Ok(value.to_string()),
            Self::Number(value) => {
                if value.is_finite()
                    && value.fract() == 0.0
                    && *value >= i64::MIN as f64
                    && *value <= i64::MAX as f64
                {
                    Ok((*value as i64).to_string())
                } else {
                    Ok(value.to_string())
                }
            }
            Self::String(value) => Ok(value.clone()),
            Self::List(_) | Self::Object(_) => Err(HtmlError::Render(
                "cannot interpolate list or object value as text".to_owned(),
            )),
        }
    }
}

/// Context object passed to a html render.
pub type HtmlContext = BTreeMap<String, Value>;

/// Browser-parsed HTML document or fragment.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct HtmlDocument {
    /// Top-level HTML nodes in source order.
    pub children: Vec<HtmlNode>,
}

impl HtmlDocument {
    /// Parses an HTML document using HTML5 tree-construction rules.
    pub fn parse(source: &str) -> HtmlResult<Self> {
        let dom = parse_document(RcDom::default(), Default::default()).one(source);
        Ok(Self {
            children: rcdom_children_to_html(&dom.document.children.borrow()),
        })
    }

    /// Parses an HTML fragment using a `body` context element.
    pub fn parse_fragment(source: &str) -> HtmlResult<Self> {
        let context = QualName::new(None, ns!(html), local_name!("body"));
        let dom = parse_fragment(
            RcDom::default(),
            Default::default(),
            context,
            Vec::new(),
            false,
        )
        .one(source);
        Ok(Self {
            children: rcdom_fragment_children_to_html(&dom.document.children.borrow()),
        })
    }

    /// Creates a retained document from this HTML tree.
    pub fn to_document(&self, viewport: Size) -> HtmlResult<Document> {
        Ok(Document::build(viewport, |document| {
            self.write_to_document_builder(document);
        }))
    }

    /// Emits this parsed HTML tree into a caller-owned document builder.
    pub fn write_to_document_builder(&self, builder: &mut DocumentBuilder) {
        let mut path = Vec::new();
        for (index, child) in self.children.iter().enumerate() {
            path.push(index);
            child.write_to_document_builder(builder, &path);
            path.pop();
        }
    }
}

/// HTML and CSS assets compiled together for document authoring.
#[derive(Clone, Debug, PartialEq)]
pub struct HtmlStylesheet {
    /// Browser-parsed HTML document or fragment.
    pub html: HtmlDocument,
    /// Parsed document stylesheet.
    pub stylesheet: StyleSheet,
}

impl HtmlStylesheet {
    /// Parses HTML and CSS into the document pipeline's typed front doors.
    pub fn parse_fragment(html: &str, css: &str) -> HtmlResult<Self> {
        let stylesheet = StyleSheet::parse_css(css).map_err(|error| {
            HtmlError::Render(format!("CSS stylesheet failed to parse: {error}"))
        })?;
        Ok(Self {
            html: HtmlDocument::parse_fragment(html)?,
            stylesheet,
        })
    }
}

/// Rust behavior declared from HTML attributes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlBehaviorHook {
    /// Event name such as `click`, `input`, or `submit`.
    pub event: String,
    /// Rust command/event intent declared by the author.
    pub command: String,
}

/// Parsed html document.
#[derive(Clone, Debug, PartialEq)]
pub struct HtmlAst {
    nodes: Vec<AstNode>,
}

/// Reusable parsed html.
#[derive(Clone, Debug, PartialEq)]
pub struct CompiledHtml {
    ast: HtmlAst,
}

impl CompiledHtml {
    /// Parses and validates a html string.
    pub fn compile(source: &str) -> HtmlResult<Self> {
        Self::compile_with_options(source, &CompileOptions::default())
    }

    /// Parses and validates a html string with explicit compile options.
    pub fn compile_with_options(source: &str, options: &CompileOptions) -> HtmlResult<Self> {
        Ok(Self {
            ast: Parser::new(source, options.limits).parse()?,
        })
    }

    /// Renders the compiled html against an explicit context.
    pub fn render(&self, context: &HtmlContext) -> HtmlResult<Vec<HtmlNode>> {
        self.render_with_options(context, &RenderOptions::default())
    }

    /// Renders the compiled html with explicit render options.
    pub fn render_with_options(
        &self,
        context: &HtmlContext,
        options: &RenderOptions,
    ) -> HtmlResult<Vec<HtmlNode>> {
        self.render_into_with_options(context, options, VecSink::default())
    }

    /// Renders the compiled html into a caller-provided sink.
    pub fn render_into<S: HtmlSink>(
        &self,
        context: &HtmlContext,
        mut sink: S,
    ) -> HtmlResult<S::Output> {
        let mut scope = Scope::new(context);
        let mut render_context = RenderContext::new(HtmlRenderLimits::default());
        render_nodes_into_sink(
            &self.ast.nodes,
            &mut scope,
            &mut render_context,
            0,
            &mut sink,
        )?;
        Ok(sink.finish())
    }

    /// Renders the compiled html into a sink with explicit options.
    pub fn render_into_with_options<S: HtmlSink>(
        &self,
        context: &HtmlContext,
        options: &RenderOptions,
        mut sink: S,
    ) -> HtmlResult<S::Output> {
        let mut scope = Scope::new(context);
        let mut render_context = RenderContext::new(options.limits);
        render_nodes_into_sink(
            &self.ast.nodes,
            &mut scope,
            &mut render_context,
            0,
            &mut sink,
        )?;
        Ok(sink.finish())
    }
}

/// Options that control html compilation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CompileOptions {
    /// Resource limits enforced during parsing and validation.
    pub limits: HtmlCompileLimits,
}

impl CompileOptions {
    /// Creates compile options with default limits.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns options with updated limits.
    pub fn with_limits(mut self, limits: HtmlCompileLimits) -> Self {
        self.limits = limits;
        self
    }
}

impl Default for CompileOptions {
    fn default() -> Self {
        Self {
            limits: HtmlCompileLimits::default(),
        }
    }
}

/// Resource limits that keep html compilation bounded.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlCompileLimits {
    /// Maximum source bytes accepted by the parser.
    pub max_source_bytes: usize,
    /// Maximum parsed element/control/text nodes.
    pub max_nodes: usize,
    /// Maximum nested element/control depth.
    pub max_depth: usize,
    /// Maximum interpolation/text parts accepted.
    pub max_text_parts: usize,
}

impl Default for HtmlCompileLimits {
    fn default() -> Self {
        Self {
            max_source_bytes: 1_000_000,
            max_nodes: 100_000,
            max_depth: 256,
            max_text_parts: 100_000,
        }
    }
}

/// Options that control html rendering.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RenderOptions {
    /// Resource limits enforced during rendering.
    pub limits: HtmlRenderLimits,
}

impl RenderOptions {
    /// Creates render options with default limits.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns options with updated limits.
    pub fn with_limits(mut self, limits: HtmlRenderLimits) -> Self {
        self.limits = limits;
        self
    }
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            limits: HtmlRenderLimits::default(),
        }
    }
}

/// Resource limits that keep html rendering bounded.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlRenderLimits {
    /// Maximum number of rendered element nodes.
    pub max_nodes: usize,
    /// Maximum number of loop iterations across a render.
    pub max_loop_iterations: usize,
    /// Maximum nested render depth.
    pub max_depth: usize,
    /// Maximum text bytes produced for a single element text value.
    pub max_text_bytes: usize,
    /// Maximum bytes produced for a single attribute value.
    pub max_attribute_bytes: usize,
}

impl Default for HtmlRenderLimits {
    fn default() -> Self {
        Self {
            max_nodes: 100_000,
            max_loop_iterations: 100_000,
            max_depth: 1_024,
            max_text_bytes: 1_000_000,
            max_attribute_bytes: 64_000,
        }
    }
}

/// Consumer for rendered html nodes.
pub trait HtmlSink {
    /// Final sink output.
    type Output;

    /// Receives one top-level rendered node.
    fn element(&mut self, node: HtmlNode) -> HtmlResult<()>;

    /// Completes the sink and returns its output.
    fn finish(self) -> Self::Output;
}

#[derive(Default)]
struct VecSink {
    nodes: Vec<HtmlNode>,
}

impl HtmlSink for VecSink {
    type Output = Vec<HtmlNode>;

    fn element(&mut self, node: HtmlNode) -> HtmlResult<()> {
        self.nodes.push(node);
        Ok(())
    }

    fn finish(self) -> Self::Output {
        self.nodes
    }
}

/// A rendered element tree.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlNode {
    /// Element tag name.
    pub tag: String,
    /// Element id from the `id` attribute, when present.
    pub id: Option<String>,
    /// Resolved class names from the `class` attribute.
    pub classes: Vec<String>,
    /// Non-id/class attributes with parsed or interpolated values.
    pub attributes: BTreeMap<String, String>,
    /// Rust behavior hooks declared through `on:*` attributes.
    pub behavior_hooks: Vec<HtmlBehaviorHook>,
    /// Text content when the element contains rendered text and no child elements.
    pub text: Option<String>,
    /// Child elements.
    pub children: Vec<HtmlNode>,
}

impl HtmlNode {
    /// Creates an explicit text node for parsed HTML mixed content.
    pub fn text_node(text: impl Into<String>) -> Self {
        Self {
            tag: "#text".to_owned(),
            id: None,
            classes: Vec::new(),
            attributes: BTreeMap::new(),
            behavior_hooks: Vec::new(),
            text: Some(text.into()),
            children: Vec::new(),
        }
    }

    /// Returns true when this node represents parsed text.
    pub fn is_text(&self) -> bool {
        self.tag == "#text"
    }

    fn write_to_document_builder(&self, builder: &mut DocumentBuilder, path: &[usize]) {
        if self.is_text() {
            if let Some(text) = self.text.as_ref().filter(|text| !text.trim().is_empty()) {
                builder.text(stable_text_id(path), TextContent::plain(text.trim()));
            }
            return;
        }

        let id = self
            .id
            .clone()
            .unwrap_or_else(|| stable_element_id(&self.tag, path));
        let mut spec = ElementSpec::new(element_for_tag(&self.tag));
        for class in &self.classes {
            spec = spec.class(class.clone());
        }
        for (name, value) in &self.attributes {
            spec = spec.attribute(name.clone(), value.clone());
        }
        for hook in &self.behavior_hooks {
            spec = spec.behavior_hook(hook.event.clone(), hook.command.clone());
        }
        if let Some(value) = self.attributes.get("value") {
            spec = spec.value(value.clone());
        }
        if !self.behavior_hooks.is_empty()
            || self.attributes.contains_key("data-command")
            || self.attributes.keys().any(|name| name.starts_with("on:"))
        {
            spec = spec.interactive();
        }

        if self.children.is_empty() {
            if let Some(text) = &self.text {
                builder.text_element(id, spec, text.clone());
            } else {
                builder.element(id, spec, |_| {});
            }
            return;
        }

        builder.element(id, spec, |builder| {
            for (index, child) in self.children.iter().enumerate() {
                let mut child_path = path.to_vec();
                child_path.push(index);
                child.write_to_document_builder(builder, &child_path);
            }
        });
    }
}

/// Hot-reload status returned after checking a html file.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReloadStatus {
    /// True when the file changed and the html was reloaded.
    pub changed: bool,
}

/// File-backed html handle for polling-style hot reload.
#[derive(Clone, Debug)]
pub struct HtmlFile {
    path: PathBuf,
    modified: Option<SystemTime>,
    fingerprint: HtmlFingerprint,
    compiled: CompiledHtml,
}

impl HtmlFile {
    /// Loads and compiles a html file.
    pub fn load(path: impl AsRef<Path>) -> HtmlResult<Self> {
        let path = path.as_ref().to_path_buf();
        let source = fs::read_to_string(&path)?;
        let metadata = fs::metadata(&path)?;
        let fingerprint = HtmlFingerprint::new(&source);
        Ok(Self {
            path,
            modified: metadata.modified().ok(),
            fingerprint,
            compiled: CompiledHtml::compile(&source)?,
        })
    }

    /// Returns the current compiled html.
    pub fn compiled(&self) -> &CompiledHtml {
        &self.compiled
    }

    /// Re-reads and recompiles the html if the file modification time changed.
    pub fn reload_if_changed(&mut self) -> HtmlResult<ReloadStatus> {
        let source = fs::read_to_string(&self.path)?;
        let metadata = fs::metadata(&self.path)?;
        let modified = metadata.modified().ok();
        let fingerprint = HtmlFingerprint::new(&source);
        if modified == self.modified && fingerprint == self.fingerprint {
            return Ok(ReloadStatus { changed: false });
        }

        self.compiled = CompiledHtml::compile(&source)?;
        self.modified = modified;
        self.fingerprint = fingerprint;
        Ok(ReloadStatus { changed: true })
    }
}

/// Collection of named htmls for compiled and hot-reloaded modes.
#[derive(Clone, Debug, Default)]
pub struct HtmlSet {
    htmls: BTreeMap<String, HtmlEntry>,
}

impl HtmlSet {
    /// Creates an empty html set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds or replaces an inline compiled html.
    pub fn add_html(&mut self, name: impl Into<String>, source: &str) -> HtmlResult<()> {
        self.htmls.insert(
            name.into(),
            HtmlEntry::Inline(CompiledHtml::compile(source)?),
        );
        Ok(())
    }

    /// Adds or replaces a file-backed html.
    pub fn add_file(&mut self, name: impl Into<String>, path: impl AsRef<Path>) -> HtmlResult<()> {
        self.htmls
            .insert(name.into(), HtmlEntry::File(HtmlFile::load(path)?));
        Ok(())
    }

    /// Returns a named compiled html.
    pub fn get(&self, name: &str) -> HtmlResult<&CompiledHtml> {
        self.htmls
            .get(name)
            .map(HtmlEntry::compiled)
            .ok_or_else(|| HtmlError::Render(format!("missing html `{name}`")))
    }

    /// Renders a named html.
    pub fn render(&self, name: &str, context: &HtmlContext) -> HtmlResult<Vec<HtmlNode>> {
        self.get(name)?.render(context)
    }

    /// Renders a named html with explicit options.
    pub fn render_with_options(
        &self,
        name: &str,
        context: &HtmlContext,
        options: &RenderOptions,
    ) -> HtmlResult<Vec<HtmlNode>> {
        self.get(name)?.render_with_options(context, options)
    }

    /// Re-reads file-backed htmls and returns names that changed.
    pub fn reload_changed(&mut self) -> HtmlResult<Vec<String>> {
        let mut updated = self.htmls.clone();
        let mut changed = Vec::new();
        for (name, entry) in &mut updated {
            if let HtmlEntry::File(file) = entry
                && file.reload_if_changed()?.changed
            {
                changed.push(name.clone());
            }
        }
        self.htmls = updated;
        Ok(changed)
    }
}

#[derive(Clone, Debug)]
enum HtmlEntry {
    Inline(CompiledHtml),
    File(HtmlFile),
}

impl HtmlEntry {
    fn compiled(&self) -> &CompiledHtml {
        match self {
            Self::Inline(html) => html,
            Self::File(file) => file.compiled(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct HtmlFingerprint {
    len: usize,
    hash: u64,
}

impl HtmlFingerprint {
    fn new(source: &str) -> Self {
        let mut hasher = DefaultHasher::new();
        source.hash(&mut hasher);
        Self {
            len: source.len(),
            hash: hasher.finish(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum AstNode {
    Element(ElementNode),
    Text(Vec<TextPart>),
    For {
        binding: String,
        source: PathExpr,
        body: Vec<AstNode>,
    },
    If {
        condition: PathExpr,
        then_body: Vec<AstNode>,
        else_body: Vec<AstNode>,
    },
}

#[derive(Clone, Debug, PartialEq)]
struct ElementNode {
    tag: String,
    attributes: Vec<AttributeNode>,
    children: Vec<AstNode>,
}

#[derive(Clone, Debug, PartialEq)]
struct AttributeNode {
    name: String,
    value: Vec<TextPart>,
}

#[derive(Clone, Debug, PartialEq)]
enum TextPart {
    Literal(String),
    Expr(PathExpr),
    If {
        condition: PathExpr,
        then_parts: Vec<TextPart>,
        else_parts: Vec<TextPart>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PathExpr {
    root: PathRoot,
    segments: Vec<PathSegment>,
}

impl PathExpr {
    fn parse(raw: &str, base_offset: usize, source: &str) -> HtmlResult<Self> {
        let trimmed_start = raw.trim_start();
        let base_offset = base_offset + raw.len().saturating_sub(trimmed_start.len());
        let raw = trimmed_start.trim_end();
        if raw.is_empty() {
            return Err(parse_error_at(source, base_offset, "empty expression"));
        }
        let (root, body) = if let Some(rest) = raw.strip_prefix("@root.") {
            (PathRoot::Document, rest)
        } else if raw == "@root" {
            (PathRoot::Document, "")
        } else {
            (PathRoot::Scope, raw)
        };
        let body_offset = match root {
            PathRoot::Scope => base_offset,
            PathRoot::Document => base_offset + raw.len().saturating_sub(body.len()),
        };
        let segments = parse_path_segments(body, body_offset, source)?;
        if segments.is_empty() {
            return Err(parse_error_at(source, base_offset, "empty expression"));
        }
        Ok(Self { root, segments })
    }

    fn display(&self) -> String {
        let mut rendered = match self.root {
            PathRoot::Scope => String::new(),
            PathRoot::Document => "@root".to_owned(),
        };
        for segment in &self.segments {
            match segment {
                PathSegment::Field(field) if rendered.is_empty() => rendered.push_str(field),
                PathSegment::Field(field) => {
                    rendered.push('.');
                    rendered.push_str(field);
                }
                PathSegment::Index(index) => rendered.push_str(&format!("[{index}]")),
            }
        }
        rendered
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PathRoot {
    Scope,
    Document,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum PathSegment {
    Field(String),
    Index(usize),
}

struct Scope<'a> {
    root: &'a HtmlContext,
    locals: Vec<(String, Value)>,
}

impl<'a> Scope<'a> {
    fn new(root: &'a HtmlContext) -> Self {
        Self {
            root,
            locals: Vec::new(),
        }
    }

    fn push(&mut self, name: String, value: Value) {
        self.locals.push((name, value));
    }

    fn pop(&mut self) {
        self.locals.pop();
    }

    fn resolve(&self, path: &PathExpr) -> HtmlResult<&Value> {
        let Some(first) = path.segments.first() else {
            return Err(HtmlError::Render("empty expression".to_owned()));
        };

        let PathSegment::Field(first) = first else {
            return Err(HtmlError::Render(format!(
                "`{}` cannot start with an index",
                path.display()
            )));
        };

        let mut value = match path.root {
            PathRoot::Scope => self
                .locals
                .iter()
                .rev()
                .find_map(|(name, value)| (name == first).then_some(value))
                .or_else(|| self.root.get(first)),
            PathRoot::Document => self.root.get(first),
        }
        .ok_or_else(|| HtmlError::Render(format!("missing value `{first}`")))?;

        for segment in path.segments.iter().skip(1) {
            value = match (value, segment) {
                (Value::Object(object), PathSegment::Field(field)) => {
                    object.get(field).ok_or_else(|| {
                        HtmlError::Render(format!("missing field `{field}` in `{first}`"))
                    })?
                }
                (Value::List(list), PathSegment::Index(index)) => {
                    list.get(*index).ok_or_else(|| {
                        HtmlError::Render(format!(
                            "index `{index}` is out of bounds in `{}`",
                            path.display()
                        ))
                    })?
                }
                (Value::List(_), PathSegment::Field(field)) => {
                    return Err(HtmlError::Render(format!(
                        "cannot access field `{field}` on list `{first}`"
                    )));
                }
                (Value::Object(_), PathSegment::Index(index)) => {
                    return Err(HtmlError::Render(format!(
                        "cannot access index `{index}` on object `{first}`"
                    )));
                }
                (_, PathSegment::Field(field)) => {
                    return Err(HtmlError::Render(format!(
                        "cannot access field `{field}` on scalar `{first}`"
                    )));
                }
                (_, PathSegment::Index(index)) => {
                    return Err(HtmlError::Render(format!(
                        "cannot access index `{index}` on scalar `{first}`"
                    )));
                }
            };
        }
        Ok(value)
    }
}

struct RenderContext {
    limits: HtmlRenderLimits,
    nodes: usize,
    loop_iterations: usize,
}

impl RenderContext {
    fn new(limits: HtmlRenderLimits) -> Self {
        Self {
            limits,
            nodes: 0,
            loop_iterations: 0,
        }
    }

    fn enter_depth(&self, depth: usize) -> HtmlResult<()> {
        if depth > self.limits.max_depth {
            return Err(HtmlError::Render(format!(
                "html depth limit exceeded: {} > {}",
                depth, self.limits.max_depth
            )));
        }
        Ok(())
    }

    fn track_node(&mut self) -> HtmlResult<()> {
        self.nodes += 1;
        if self.nodes > self.limits.max_nodes {
            return Err(HtmlError::Render(format!(
                "node limit exceeded: {} > {}",
                self.nodes, self.limits.max_nodes
            )));
        }
        Ok(())
    }

    fn track_loop_iteration(&mut self) -> HtmlResult<()> {
        self.loop_iterations += 1;
        if self.loop_iterations > self.limits.max_loop_iterations {
            return Err(HtmlError::Render(format!(
                "loop iteration limit exceeded: {} > {}",
                self.loop_iterations, self.limits.max_loop_iterations
            )));
        }
        Ok(())
    }

    fn check_loop_capacity(&self, additional_iterations: usize) -> HtmlResult<()> {
        let requested = self.loop_iterations.saturating_add(additional_iterations);
        if requested > self.limits.max_loop_iterations {
            return Err(HtmlError::Render(format!(
                "loop iteration limit exceeded: {} > {}",
                requested, self.limits.max_loop_iterations
            )));
        }
        Ok(())
    }
}

#[derive(Clone, Copy)]
enum TextLimitKind {
    Text,
    Attribute,
}

impl TextLimitKind {
    fn max_bytes(self, limits: HtmlRenderLimits) -> usize {
        match self {
            Self::Text => limits.max_text_bytes,
            Self::Attribute => limits.max_attribute_bytes,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Attribute => "attribute",
        }
    }
}

struct LimitedString {
    value: String,
    max_bytes: usize,
    label: &'static str,
}

impl LimitedString {
    fn new(kind: TextLimitKind, limits: HtmlRenderLimits) -> Self {
        Self {
            value: String::new(),
            max_bytes: kind.max_bytes(limits),
            label: kind.label(),
        }
    }

    fn push_str(&mut self, value: &str) -> HtmlResult<()> {
        if self.value.len().saturating_add(value.len()) > self.max_bytes {
            return Err(HtmlError::Render(format!(
                "{} byte limit exceeded: {} > {}",
                self.label,
                self.value.len().saturating_add(value.len()),
                self.max_bytes
            )));
        }
        self.value.push_str(value);
        Ok(())
    }

    fn finish(self) -> String {
        self.value
    }
}

fn rcdom_children_to_html(children: &[Handle]) -> Vec<HtmlNode> {
    let mut nodes = Vec::new();
    for child in children {
        append_rcdom_node(child, &mut nodes);
    }
    nodes
}

fn rcdom_fragment_children_to_html(children: &[Handle]) -> Vec<HtmlNode> {
    let mut nodes = rcdom_children_to_html(children);
    loop {
        if nodes.len() != 1 {
            return nodes;
        }
        match nodes[0].tag.as_str() {
            "html" | "body" => nodes = nodes.remove(0).children,
            _ => return nodes,
        }
    }
}

fn append_rcdom_node(handle: &Handle, nodes: &mut Vec<HtmlNode>) {
    match &handle.data {
        NodeData::Document => nodes.extend(rcdom_children_to_html(&handle.children.borrow())),
        NodeData::Doctype { .. } | NodeData::Comment { .. } => {}
        NodeData::Text { contents } => {
            let text = contents.borrow().to_string();
            if !text.trim().is_empty() {
                nodes.push(HtmlNode::text_node(text));
            }
        }
        NodeData::Element { name, attrs, .. } => {
            let tag = name.local.to_string();
            let mut id = None;
            let mut classes = Vec::new();
            let mut attributes = BTreeMap::new();
            let mut behavior_hooks = Vec::new();

            for attr in attrs.borrow().iter() {
                let name = html_attribute_name(&attr.name);
                let value = attr.value.to_string();
                if name == "id" {
                    id = Some(value);
                } else if name == "class" {
                    classes.extend(value.split_whitespace().map(str::to_owned));
                } else if let Some(event) = name.strip_prefix("on:") {
                    behavior_hooks.push(HtmlBehaviorHook {
                        event: event.to_owned(),
                        command: value,
                    });
                } else if let Some(command_event) = name.strip_prefix("data-command") {
                    let event = command_event
                        .strip_prefix(':')
                        .filter(|event| !event.is_empty())
                        .unwrap_or("click");
                    behavior_hooks.push(HtmlBehaviorHook {
                        event: event.to_owned(),
                        command: value.clone(),
                    });
                    attributes.insert(name, value);
                } else {
                    attributes.insert(name, value);
                }
            }

            let children = rcdom_children_to_html(&handle.children.borrow());
            let text = if children.len() == 1 && children[0].is_text() {
                children[0].text.clone()
            } else {
                None
            };
            let children = if text.is_some() { Vec::new() } else { children };
            nodes.push(HtmlNode {
                tag,
                id,
                classes,
                attributes,
                behavior_hooks,
                text,
                children,
            });
        }
        NodeData::ProcessingInstruction { .. } => {}
    }
}

fn html_attribute_name(name: &QualName) -> String {
    if let Some(prefix) = &name.prefix {
        format!("{prefix}:{}", name.local)
    } else {
        name.local.to_string()
    }
}

fn element_for_tag(tag: &str) -> Element {
    match tag {
        "main" => Element::Main,
        "section" => Element::Section,
        "article" => Element::Article,
        "header" => Element::Header,
        "footer" => Element::Footer,
        "nav" => Element::Nav,
        "aside" => Element::Aside,
        "p" => Element::P,
        "h1" => Element::H1,
        "h2" => Element::H2,
        "h3" => Element::H3,
        "h4" => Element::H4,
        "h5" => Element::H5,
        "h6" => Element::H6,
        "span" => Element::Span,
        "button" => Element::Button,
        "input" => Element::Input,
        "select" => Element::Select,
        "option" => Element::Option,
        "textarea" => Element::Textarea,
        "label" => Element::Label,
        "canvas" => Element::Canvas,
        "table" => Element::Table,
        "thead" => Element::Thead,
        "tbody" => Element::Tbody,
        "tr" => Element::Tr,
        "th" => Element::Th,
        "td" => Element::Td,
        _ => Element::Div,
    }
}

fn stable_element_id(tag: &str, path: &[usize]) -> String {
    format!("html/{tag}-{}", stable_path(path))
}

fn stable_text_id(path: &[usize]) -> String {
    stable_text_id_with_suffix(path, "text")
}

fn stable_text_id_with_suffix(path: &[usize], suffix: &str) -> String {
    format!("html/{suffix}-{}", stable_path(path))
}

fn stable_path(path: &[usize]) -> String {
    path.iter()
        .map(usize::to_string)
        .collect::<Vec<_>>()
        .join("-")
}

fn render_nodes(
    nodes: &[AstNode],
    scope: &mut Scope<'_>,
    render_context: &mut RenderContext,
    depth: usize,
) -> HtmlResult<Vec<HtmlNode>> {
    render_context.enter_depth(depth)?;
    let mut rendered = Vec::new();
    for node in nodes {
        match node {
            AstNode::Element(element) => {
                rendered.push(render_element(element, scope, render_context, depth)?)
            }
            AstNode::Text(_) => {}
            AstNode::For {
                binding,
                source,
                body,
            } => {
                let values = match scope.resolve(source)? {
                    Value::List(values) => {
                        render_context.check_loop_capacity(values.len())?;
                        values.clone()
                    }
                    _ => {
                        return Err(HtmlError::Render(format!(
                            "`{}` is not iterable",
                            source.display()
                        )));
                    }
                };
                let len = values.len();
                for (index, value) in values.into_iter().enumerate() {
                    render_context.track_loop_iteration()?;
                    scope.push(binding.clone(), value);
                    scope.push("loop".to_owned(), loop_value(index, len));
                    rendered.extend(render_nodes(body, scope, render_context, depth)?);
                    scope.pop();
                    scope.pop();
                }
            }
            AstNode::If {
                condition,
                then_body,
                else_body,
            } => {
                if scope.resolve(condition)?.truthy() {
                    rendered.extend(render_nodes(then_body, scope, render_context, depth)?);
                } else {
                    rendered.extend(render_nodes(else_body, scope, render_context, depth)?);
                }
            }
        }
    }
    Ok(rendered)
}

fn render_nodes_into_sink<S: HtmlSink>(
    nodes: &[AstNode],
    scope: &mut Scope<'_>,
    render_context: &mut RenderContext,
    depth: usize,
    sink: &mut S,
) -> HtmlResult<()> {
    render_context.enter_depth(depth)?;
    for node in nodes {
        match node {
            AstNode::Element(element) => {
                sink.element(render_element(element, scope, render_context, depth)?)?;
            }
            AstNode::Text(_) => {}
            AstNode::For {
                binding,
                source,
                body,
            } => {
                let values = match scope.resolve(source)? {
                    Value::List(values) => {
                        render_context.check_loop_capacity(values.len())?;
                        values.clone()
                    }
                    _ => {
                        return Err(HtmlError::Render(format!(
                            "`{}` is not iterable",
                            source.display()
                        )));
                    }
                };
                let len = values.len();
                for (index, value) in values.into_iter().enumerate() {
                    render_context.track_loop_iteration()?;
                    scope.push(binding.clone(), value);
                    scope.push("loop".to_owned(), loop_value(index, len));
                    render_nodes_into_sink(body, scope, render_context, depth, sink)?;
                    scope.pop();
                    scope.pop();
                }
            }
            AstNode::If {
                condition,
                then_body,
                else_body,
            } => {
                if scope.resolve(condition)?.truthy() {
                    render_nodes_into_sink(then_body, scope, render_context, depth, sink)?;
                } else {
                    render_nodes_into_sink(else_body, scope, render_context, depth, sink)?;
                }
            }
        }
    }
    Ok(())
}

fn render_element(
    element: &ElementNode,
    scope: &mut Scope<'_>,
    render_context: &mut RenderContext,
    depth: usize,
) -> HtmlResult<HtmlNode> {
    render_context.track_node()?;
    let mut attributes = BTreeMap::new();
    let mut classes = Vec::new();
    let mut id = None;
    let mut behavior_hooks = Vec::new();

    for attribute in &element.attributes {
        let value = render_text_parts(
            &attribute.value,
            scope,
            render_context,
            TextLimitKind::Attribute,
        )?;
        if attribute.name == "class" {
            classes.extend(value.split_whitespace().map(str::to_owned));
        } else if attribute.name == "id" {
            id = Some(value);
        } else if let Some(event) = attribute.name.strip_prefix("on:") {
            behavior_hooks.push(HtmlBehaviorHook {
                event: event.to_owned(),
                command: value,
            });
        } else {
            attributes.insert(attribute.name.clone(), value);
        }
    }

    let mut children = Vec::new();
    let mut text = LimitedString::new(TextLimitKind::Text, render_context.limits);
    for child in &element.children {
        match child {
            AstNode::Text(parts) => text.push_str(&render_text_parts(
                parts,
                scope,
                render_context,
                TextLimitKind::Text,
            )?)?,
            AstNode::Element(_) | AstNode::For { .. } | AstNode::If { .. } => {
                children.extend(render_nodes(
                    std::slice::from_ref(child),
                    scope,
                    render_context,
                    depth + 1,
                )?);
            }
        }
    }

    let text = if children.is_empty() {
        let text = text.finish();
        let trimmed = text.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_owned())
    } else {
        None
    };

    Ok(HtmlNode {
        tag: element.tag.clone(),
        id,
        classes,
        attributes,
        behavior_hooks,
        text,
        children,
    })
}

fn render_text_parts(
    parts: &[TextPart],
    scope: &Scope<'_>,
    render_context: &RenderContext,
    kind: TextLimitKind,
) -> HtmlResult<String> {
    let mut rendered = LimitedString::new(kind, render_context.limits);
    for part in parts {
        match part {
            TextPart::Literal(value) => rendered.push_str(value)?,
            TextPart::Expr(path) => rendered.push_str(&scope.resolve(path)?.render_scalar()?)?,
            TextPart::If {
                condition,
                then_parts,
                else_parts,
            } => {
                if scope.resolve(condition)?.truthy() {
                    rendered.push_str(&render_text_parts(
                        then_parts,
                        scope,
                        render_context,
                        kind,
                    )?)?;
                } else {
                    rendered.push_str(&render_text_parts(
                        else_parts,
                        scope,
                        render_context,
                        kind,
                    )?)?;
                }
            }
        }
    }
    Ok(rendered.finish())
}

fn loop_value(index: usize, len: usize) -> Value {
    let mut value = BTreeMap::new();
    value.insert("index0".to_owned(), Value::number(index as f64));
    value.insert("index".to_owned(), Value::number((index + 1) as f64));
    value.insert("len".to_owned(), Value::number(len as f64));
    value.insert("first".to_owned(), Value::bool(index == 0));
    value.insert("last".to_owned(), Value::bool(index + 1 == len));
    Value::object(value)
}

struct CompileContext {
    limits: HtmlCompileLimits,
    nodes: usize,
    text_parts: usize,
}

impl CompileContext {
    fn new(limits: HtmlCompileLimits) -> Self {
        Self {
            limits,
            nodes: 0,
            text_parts: 0,
        }
    }

    fn check_source(&self, source: &str) -> HtmlResult<()> {
        if source.len() > self.limits.max_source_bytes {
            return Err(parse_error_at(
                source,
                self.limits.max_source_bytes,
                "compile source byte limit exceeded",
            ));
        }
        Ok(())
    }

    fn enter_depth(&self, depth: usize, source: &str, offset: usize) -> HtmlResult<()> {
        if depth > self.limits.max_depth {
            return Err(parse_error_at(
                source,
                offset,
                "compile depth limit exceeded",
            ));
        }
        Ok(())
    }

    fn track_node(&mut self, source: &str, offset: usize) -> HtmlResult<()> {
        self.nodes += 1;
        if self.nodes > self.limits.max_nodes {
            return Err(parse_error_at(
                source,
                offset,
                "compile node limit exceeded",
            ));
        }
        Ok(())
    }
}

struct Parser<'a> {
    source: &'a str,
    offset: usize,
    compile_context: CompileContext,
}

impl<'a> Parser<'a> {
    fn new(source: &'a str, limits: HtmlCompileLimits) -> Self {
        Self {
            source,
            offset: 0,
            compile_context: CompileContext::new(limits),
        }
    }

    fn parse(mut self) -> HtmlResult<HtmlAst> {
        self.compile_context.check_source(self.source)?;
        let nodes = self.parse_nodes(None, 0)?;
        self.skip_ws();
        if !self.eof() {
            return self.error("unexpected trailing html source");
        }
        Ok(HtmlAst { nodes })
    }

    fn parse_nodes(&mut self, closing_tag: Option<&str>, depth: usize) -> HtmlResult<Vec<AstNode>> {
        self.compile_context
            .enter_depth(depth, self.source, self.offset)?;
        let mut nodes = Vec::new();
        loop {
            self.skip_ws();
            if self.eof() {
                if let Some(tag) = closing_tag {
                    return self.error(&format!("missing closing tag </{tag}>"));
                }
                return Ok(nodes);
            }
            if let Some(tag) = closing_tag {
                let closing = format!("</{tag}>");
                if self.starts_with(&closing) {
                    self.offset += closing.len();
                    return Ok(nodes);
                }
            }
            if self.starts_with("</") {
                return self.error("unexpected closing tag");
            }
            if self.starts_with("{/for}") || self.starts_with("{/if}") || self.starts_with("{else}")
            {
                return Ok(nodes);
            }
            if self.starts_with("{for ") {
                self.compile_context.track_node(self.source, self.offset)?;
                nodes.push(self.parse_for(depth)?);
            } else if self.starts_with("{if ") {
                self.compile_context.track_node(self.source, self.offset)?;
                nodes.push(self.parse_if(depth)?);
            } else if self.starts_with("<") {
                self.compile_context.track_node(self.source, self.offset)?;
                nodes.push(AstNode::Element(self.parse_element(depth)?));
            } else {
                let text = self.parse_text()?;
                if text.iter().any(|part| match part {
                    TextPart::Literal(value) => !value.trim().is_empty(),
                    TextPart::Expr(_) | TextPart::If { .. } => true,
                }) {
                    self.compile_context.track_node(self.source, self.offset)?;
                    nodes.push(AstNode::Text(text));
                }
            }
        }
    }

    fn parse_element(&mut self, depth: usize) -> HtmlResult<ElementNode> {
        self.expect("<")?;
        let tag = self.parse_identifier()?;
        let mut attributes = Vec::new();
        loop {
            self.skip_ws();
            if self.starts_with("/>") {
                self.offset += 2;
                return Ok(ElementNode {
                    tag,
                    attributes,
                    children: Vec::new(),
                });
            }
            if self.starts_with(">") {
                self.offset += 1;
                if tag == "text" {
                    let closing = format!("</{tag}>");
                    let start = self.offset;
                    let Some(relative_end) = self.source[self.offset..].find(&closing) else {
                        return self.error(&format!("missing closing tag </{tag}>"));
                    };
                    let end = self.offset + relative_end;
                    self.offset = end + closing.len();
                    let children = vec![AstNode::Text(parse_text_parts(
                        &self.source[start..end],
                        start,
                        self.source,
                        self.compile_context.limits,
                        &mut self.compile_context.text_parts,
                        0,
                    )?)];
                    return Ok(ElementNode {
                        tag,
                        attributes,
                        children,
                    });
                }
                let children = self.parse_nodes(Some(&tag), depth + 1)?;
                return Ok(ElementNode {
                    tag,
                    attributes,
                    children,
                });
            }
            attributes.push(self.parse_attribute()?);
        }
    }

    fn parse_attribute(&mut self) -> HtmlResult<AttributeNode> {
        let name = self.parse_identifier()?;
        self.skip_ws();
        self.expect("=")?;
        self.skip_ws();
        self.expect("\"")?;
        let start = self.offset;
        while !self.eof() && !self.starts_with("\"") {
            self.offset += self.next_char_len();
        }
        if self.eof() {
            return self.error("unterminated attribute value");
        }
        let raw = &self.source[start..self.offset];
        self.expect("\"")?;
        Ok(AttributeNode {
            name,
            value: parse_text_parts(
                raw,
                start,
                self.source,
                self.compile_context.limits,
                &mut self.compile_context.text_parts,
                0,
            )?,
        })
    }

    fn parse_for(&mut self, depth: usize) -> HtmlResult<AstNode> {
        self.expect("{for ")?;
        let directive = self.read_until("}")?;
        let parts = directive.split_whitespace().collect::<Vec<_>>();
        if parts.len() != 3 || parts[1] != "in" {
            return self.error("expected `{for item in items}`");
        }
        let binding = validate_identifier(parts[0])?;
        let source = PathExpr::parse(
            parts[2],
            self.offset.saturating_sub(directive.len() + 1),
            self.source,
        )?;
        let body = self.parse_nodes(None, depth + 1)?;
        self.expect("{/for}")?;
        Ok(AstNode::For {
            binding,
            source,
            body,
        })
    }

    fn parse_if(&mut self, depth: usize) -> HtmlResult<AstNode> {
        self.expect("{if ")?;
        let start = self.offset;
        let condition = PathExpr::parse(self.read_until("}")?, start, self.source)?;
        let then_body = self.parse_nodes(None, depth + 1)?;
        let else_body = if self.starts_with("{else}") {
            self.expect("{else}")?;
            self.parse_nodes(None, depth + 1)?
        } else {
            Vec::new()
        };
        self.expect("{/if}")?;
        Ok(AstNode::If {
            condition,
            then_body,
            else_body,
        })
    }

    fn parse_text(&mut self) -> HtmlResult<Vec<TextPart>> {
        let start = self.offset;
        while !self.eof()
            && !self.starts_with("<")
            && !self.starts_with("{for ")
            && !self.starts_with("{if ")
            && !self.starts_with("{/for}")
            && !self.starts_with("{/if}")
            && !self.starts_with("{else}")
        {
            self.offset += self.next_char_len();
        }
        parse_text_parts(
            &self.source[start..self.offset],
            start,
            self.source,
            self.compile_context.limits,
            &mut self.compile_context.text_parts,
            0,
        )
    }

    fn parse_identifier(&mut self) -> HtmlResult<String> {
        let start = self.offset;
        while !self.eof() {
            let Some(ch) = self.source[self.offset..].chars().next() else {
                break;
            };
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == ':' {
                self.offset += ch.len_utf8();
            } else {
                break;
            }
        }
        if start == self.offset {
            return self.error("expected identifier");
        }
        Ok(self.source[start..self.offset].to_owned())
    }

    fn read_until(&mut self, delimiter: &str) -> HtmlResult<&'a str> {
        let start = self.offset;
        let Some(relative_end) = self.source[self.offset..].find(delimiter) else {
            return self.error(&format!("missing `{delimiter}`"));
        };
        let end = self.offset + relative_end;
        self.offset = end + delimiter.len();
        Ok(&self.source[start..end])
    }

    fn expect(&mut self, expected: &str) -> HtmlResult<()> {
        if self.starts_with(expected) {
            self.offset += expected.len();
            Ok(())
        } else {
            self.error(&format!("expected `{expected}`"))
        }
    }

    fn starts_with(&self, value: &str) -> bool {
        self.source[self.offset..].starts_with(value)
    }

    fn skip_ws(&mut self) {
        while !self.eof() {
            let Some(ch) = self.source[self.offset..].chars().next() else {
                break;
            };
            if ch.is_whitespace() {
                self.offset += ch.len_utf8();
            } else {
                break;
            }
        }
    }

    fn eof(&self) -> bool {
        self.offset >= self.source.len()
    }

    fn next_char_len(&self) -> usize {
        self.source[self.offset..]
            .chars()
            .next()
            .map(char::len_utf8)
            .unwrap_or(0)
    }

    fn error<T>(&self, message: &str) -> HtmlResult<T> {
        Err(parse_error_at(self.source, self.offset, message))
    }
}

fn parse_text_parts(
    raw: &str,
    base_offset: usize,
    source: &str,
    limits: HtmlCompileLimits,
    text_parts: &mut usize,
    depth: usize,
) -> HtmlResult<Vec<TextPart>> {
    let mut parts = Vec::new();
    let mut cursor = 0;
    while cursor < raw.len() {
        let Some(relative_open) = raw[cursor..].find('{') else {
            track_text_part(source, base_offset + cursor, limits, text_parts)?;
            parts.push(TextPart::Literal(raw[cursor..].to_owned()));
            break;
        };
        let open = cursor + relative_open;
        if open > cursor {
            track_text_part(source, base_offset + cursor, limits, text_parts)?;
            parts.push(TextPart::Literal(raw[cursor..open].to_owned()));
        }
        let Some(relative_close) = raw[open..].find('}') else {
            return Err(parse_error_at(
                source,
                base_offset + open,
                "unterminated interpolation",
            ));
        };
        let close = open + relative_close;
        let directive = raw[open + 1..close].trim();
        if let Some(condition) = directive.strip_prefix("if ") {
            if depth + 1 > limits.max_depth {
                return Err(parse_error_at(
                    source,
                    base_offset + open,
                    "compile depth limit exceeded",
                ));
            }
            let (relative_else, body_end) =
                find_inline_if_bounds(raw, close + 1, source, base_offset + open)?;
            let body_start = close + 1;
            let body = &raw[body_start..body_end];
            let (then_raw, else_raw) = if let Some(relative_else) = relative_else {
                let relative_else = relative_else - body_start;
                (
                    &body[..relative_else],
                    &body[relative_else + "{else}".len()..],
                )
            } else {
                (body, "")
            };
            track_text_part(source, base_offset + open, limits, text_parts)?;
            parts.push(TextPart::If {
                condition: PathExpr::parse(condition, base_offset + open + 1, source)?,
                then_parts: parse_text_parts(
                    then_raw,
                    base_offset + body_start,
                    source,
                    limits,
                    text_parts,
                    depth + 1,
                )?,
                else_parts: parse_text_parts(
                    else_raw,
                    base_offset + body_start + body.len().saturating_sub(else_raw.len()),
                    source,
                    limits,
                    text_parts,
                    depth + 1,
                )?,
            });
            cursor = body_end + "{/if}".len();
        } else {
            track_text_part(source, base_offset + open, limits, text_parts)?;
            parts.push(TextPart::Expr(PathExpr::parse(
                &raw[open + 1..close],
                base_offset + open + 1,
                source,
            )?));
            cursor = close + 1;
        }
    }
    Ok(parts)
}

fn track_text_part(
    source: &str,
    offset: usize,
    limits: HtmlCompileLimits,
    text_parts: &mut usize,
) -> HtmlResult<()> {
    *text_parts += 1;
    if *text_parts > limits.max_text_parts {
        return Err(parse_error_at(
            source,
            offset,
            "compile text part limit exceeded",
        ));
    }
    Ok(())
}

fn find_inline_if_bounds(
    raw: &str,
    body_start: usize,
    source: &str,
    error_offset: usize,
) -> HtmlResult<(Option<usize>, usize)> {
    let mut depth = 0usize;
    let mut cursor = body_start;
    let mut else_at = None;
    while cursor < raw.len() {
        let rest = &raw[cursor..];
        let next_if = rest.find("{if ").map(|offset| cursor + offset);
        let next_else = rest.find("{else}").map(|offset| cursor + offset);
        let next_end = rest.find("{/if}").map(|offset| cursor + offset);
        let Some(next) = [next_if, next_else, next_end].into_iter().flatten().min() else {
            break;
        };

        if raw[next..].starts_with("{if ") {
            depth += 1;
            cursor = next + "{if ".len();
        } else if raw[next..].starts_with("{else}") {
            if depth == 0 && else_at.is_none() {
                else_at = Some(next);
            }
            cursor = next + "{else}".len();
        } else {
            if depth == 0 {
                return Ok((else_at, next));
            }
            depth -= 1;
            cursor = next + "{/if}".len();
        }
    }

    Err(parse_error_at(source, error_offset, "missing `{/if}`"))
}

fn parse_path_segments(
    raw: &str,
    base_offset: usize,
    source: &str,
) -> HtmlResult<Vec<PathSegment>> {
    let mut segments = Vec::new();
    let mut cursor = 0;
    while cursor < raw.len() {
        if raw[cursor..].starts_with('.') {
            return Err(parse_error_at(
                source,
                base_offset + cursor,
                "malformed path expression",
            ));
        }
        if raw[cursor..].starts_with('[') {
            if segments.is_empty() {
                return Err(parse_error_at(
                    source,
                    base_offset + cursor,
                    "path cannot start with an index",
                ));
            }
            let Some(relative_close) = raw[cursor..].find(']') else {
                return Err(parse_error_at(source, base_offset + cursor, "missing `]`"));
            };
            let close = cursor + relative_close;
            let index = raw[cursor + 1..close]
                .trim()
                .parse::<usize>()
                .map_err(|_| {
                    parse_error_at(source, base_offset + cursor, "expected numeric list index")
                })?;
            segments.push(PathSegment::Index(index));
            cursor = close + 1;
            if cursor >= raw.len() {
                continue;
            }
            if raw[cursor..].starts_with('[') {
                continue;
            }
            if raw[cursor..].starts_with('.') {
                cursor += 1;
                if cursor >= raw.len()
                    || raw[cursor..].starts_with('.')
                    || raw[cursor..].starts_with('[')
                {
                    return Err(parse_error_at(
                        source,
                        base_offset + cursor.saturating_sub(1),
                        "malformed path expression",
                    ));
                }
                continue;
            }
            return Err(parse_error_at(
                source,
                base_offset + cursor,
                "expected `.` or `[` in path expression",
            ));
        }

        let start = cursor;
        while cursor < raw.len() {
            let Some(ch) = raw[cursor..].chars().next() else {
                break;
            };
            if ch.is_ascii_alphanumeric() || ch == '_' {
                cursor += ch.len_utf8();
            } else {
                break;
            }
        }
        if start == cursor {
            return Err(parse_error_at(
                source,
                base_offset + cursor,
                "expected path segment in path expression",
            ));
        }
        segments.push(PathSegment::Field(validate_identifier_at(
            &raw[start..cursor],
            base_offset + start,
            source,
        )?));

        if cursor >= raw.len() {
            continue;
        }
        if raw[cursor..].starts_with('[') {
            continue;
        }
        if raw[cursor..].starts_with('.') {
            cursor += 1;
            if cursor >= raw.len()
                || raw[cursor..].starts_with('.')
                || raw[cursor..].starts_with('[')
            {
                return Err(parse_error_at(
                    source,
                    base_offset + cursor.saturating_sub(1),
                    "malformed path expression",
                ));
            }
            continue;
        }
        {
            return Err(parse_error_at(
                source,
                base_offset + cursor,
                "expected `.` or `[` in path expression",
            ));
        }
    }
    Ok(segments)
}

fn validate_identifier(value: &str) -> HtmlResult<String> {
    validate_identifier_at(value, 0, "")
}

fn validate_identifier_at(value: &str, offset: usize, source: &str) -> HtmlResult<String> {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return Err(parse_error_at(source, offset, "empty identifier"));
    };
    if !(first.is_ascii_alphabetic() || first == '_') {
        return Err(parse_error_at(
            source,
            offset,
            &format!("invalid identifier `{value}`"),
        ));
    }
    if chars.any(|ch| !(ch.is_ascii_alphanumeric() || ch == '_')) {
        return Err(parse_error_at(
            source,
            offset,
            &format!("invalid identifier `{value}`"),
        ));
    }
    Ok(value.to_owned())
}

fn parse_error_at(source: &str, offset: usize, message: &str) -> HtmlError {
    let (line, column) = line_column(source, offset.min(source.len()));
    HtmlError::Parse {
        offset,
        line,
        column,
        message: message.to_owned(),
    }
}

fn line_column(source: &str, offset: usize) -> (usize, usize) {
    let mut line = 1;
    let mut column = 1;
    for (index, ch) in source.char_indices() {
        if index >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
    (line, column)
}
