//! Runtime template language for Data Engine Studio document markup.
//!
//! The crate parses a constrained XML-like template into an AST, compiles that
//! AST into a reusable template handle, and renders it against an explicit data
//! context. Templates are pure: they do not perform IO, call arbitrary code, or
//! mutate app state while rendering.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::collections::hash_map::DefaultHasher;
use std::fmt;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Convenient result type for template operations.
pub type TemplateResult<T> = Result<T, TemplateError>;

/// Template parser, renderer, and hot-reload errors.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TemplateError {
    /// The template source is syntactically invalid.
    Parse {
        offset: usize,
        line: usize,
        column: usize,
        message: String,
    },
    /// Rendering referenced data that does not exist or has the wrong shape.
    Render(String),
    /// The template file could not be read or inspected.
    Io(String),
}

impl fmt::Display for TemplateError {
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
                    "template parse error at {line}:{column} (offset {offset}): {message}"
                )
            }
            Self::Render(message) => write!(f, "template render error: {message}"),
            Self::Io(message) => write!(f, "template io error: {message}"),
        }
    }
}

impl std::error::Error for TemplateError {}

impl From<std::io::Error> for TemplateError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error.to_string())
    }
}

/// Runtime value made available to templates.
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

    fn render_scalar(&self) -> TemplateResult<String> {
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
            Self::List(_) | Self::Object(_) => Err(TemplateError::Render(
                "cannot interpolate list or object value as text".to_owned(),
            )),
        }
    }
}

/// Context object passed to a template render.
pub type TemplateContext = BTreeMap<String, Value>;

/// Parsed template document.
#[derive(Clone, Debug, PartialEq)]
pub struct TemplateAst {
    nodes: Vec<AstNode>,
}

/// Reusable parsed template.
#[derive(Clone, Debug, PartialEq)]
pub struct CompiledTemplate {
    ast: TemplateAst,
}

impl CompiledTemplate {
    /// Parses and validates a template string.
    pub fn compile(source: &str) -> TemplateResult<Self> {
        Self::compile_with_options(source, &CompileOptions::default())
    }

    /// Parses and validates a template string with explicit compile options.
    pub fn compile_with_options(source: &str, options: &CompileOptions) -> TemplateResult<Self> {
        Ok(Self {
            ast: Parser::new(source, options.limits).parse()?,
        })
    }

    /// Renders the compiled template against an explicit context.
    pub fn render(&self, context: &TemplateContext) -> TemplateResult<Vec<RenderedNode>> {
        self.render_with_options(context, &RenderOptions::default())
    }

    /// Renders the compiled template with explicit render options.
    pub fn render_with_options(
        &self,
        context: &TemplateContext,
        options: &RenderOptions,
    ) -> TemplateResult<Vec<RenderedNode>> {
        self.render_into_with_options(context, options, VecSink::default())
    }

    /// Renders the compiled template into a caller-provided sink.
    pub fn render_into<S: TemplateSink>(
        &self,
        context: &TemplateContext,
        mut sink: S,
    ) -> TemplateResult<S::Output> {
        let mut scope = Scope::new(context);
        let mut render_context = RenderContext::new(TemplateLimits::default());
        render_nodes_into_sink(
            &self.ast.nodes,
            &mut scope,
            &mut render_context,
            0,
            &mut sink,
        )?;
        Ok(sink.finish())
    }

    /// Renders the compiled template into a sink with explicit options.
    pub fn render_into_with_options<S: TemplateSink>(
        &self,
        context: &TemplateContext,
        options: &RenderOptions,
        mut sink: S,
    ) -> TemplateResult<S::Output> {
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

/// Options that control template compilation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CompileOptions {
    /// Resource limits enforced during parsing and validation.
    pub limits: TemplateCompileLimits,
}

impl CompileOptions {
    /// Creates compile options with default limits.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns options with updated limits.
    pub fn with_limits(mut self, limits: TemplateCompileLimits) -> Self {
        self.limits = limits;
        self
    }
}

impl Default for CompileOptions {
    fn default() -> Self {
        Self {
            limits: TemplateCompileLimits::default(),
        }
    }
}

/// Resource limits that keep template compilation bounded.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TemplateCompileLimits {
    /// Maximum source bytes accepted by the parser.
    pub max_source_bytes: usize,
    /// Maximum parsed element/control/text nodes.
    pub max_nodes: usize,
    /// Maximum nested element/control depth.
    pub max_depth: usize,
    /// Maximum interpolation/text parts accepted.
    pub max_text_parts: usize,
}

impl Default for TemplateCompileLimits {
    fn default() -> Self {
        Self {
            max_source_bytes: 1_000_000,
            max_nodes: 100_000,
            max_depth: 256,
            max_text_parts: 100_000,
        }
    }
}

/// Options that control template rendering.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RenderOptions {
    /// Resource limits enforced during rendering.
    pub limits: TemplateLimits,
}

impl RenderOptions {
    /// Creates render options with default limits.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns options with updated limits.
    pub fn with_limits(mut self, limits: TemplateLimits) -> Self {
        self.limits = limits;
        self
    }
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            limits: TemplateLimits::default(),
        }
    }
}

/// Resource limits that keep template rendering bounded.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TemplateLimits {
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

impl Default for TemplateLimits {
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

/// Consumer for rendered template nodes.
pub trait TemplateSink {
    /// Final sink output.
    type Output;

    /// Receives one top-level rendered node.
    fn element(&mut self, node: RenderedNode) -> TemplateResult<()>;

    /// Completes the sink and returns its output.
    fn finish(self) -> Self::Output;
}

#[derive(Default)]
struct VecSink {
    nodes: Vec<RenderedNode>,
}

impl TemplateSink for VecSink {
    type Output = Vec<RenderedNode>;

    fn element(&mut self, node: RenderedNode) -> TemplateResult<()> {
        self.nodes.push(node);
        Ok(())
    }

    fn finish(self) -> Self::Output {
        self.nodes
    }
}

/// A rendered element tree.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenderedNode {
    /// Element tag name.
    pub tag: String,
    /// Resolved class names from the `class` attribute.
    pub classes: Vec<String>,
    /// Non-class attributes with interpolated values.
    pub attributes: BTreeMap<String, String>,
    /// Text content when the element contains rendered text and no child elements.
    pub text: Option<String>,
    /// Child elements.
    pub children: Vec<RenderedNode>,
}

/// Hot-reload status returned after checking a template file.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReloadStatus {
    /// True when the file changed and the template was reloaded.
    pub changed: bool,
}

/// File-backed template handle for polling-style hot reload.
#[derive(Clone, Debug)]
pub struct TemplateFile {
    path: PathBuf,
    modified: Option<SystemTime>,
    fingerprint: TemplateFingerprint,
    compiled: CompiledTemplate,
}

impl TemplateFile {
    /// Loads and compiles a template file.
    pub fn load(path: impl AsRef<Path>) -> TemplateResult<Self> {
        let path = path.as_ref().to_path_buf();
        let source = fs::read_to_string(&path)?;
        let metadata = fs::metadata(&path)?;
        let fingerprint = TemplateFingerprint::new(&source);
        Ok(Self {
            path,
            modified: metadata.modified().ok(),
            fingerprint,
            compiled: CompiledTemplate::compile(&source)?,
        })
    }

    /// Returns the current compiled template.
    pub fn compiled(&self) -> &CompiledTemplate {
        &self.compiled
    }

    /// Re-reads and recompiles the template if the file modification time changed.
    pub fn reload_if_changed(&mut self) -> TemplateResult<ReloadStatus> {
        let source = fs::read_to_string(&self.path)?;
        let metadata = fs::metadata(&self.path)?;
        let modified = metadata.modified().ok();
        let fingerprint = TemplateFingerprint::new(&source);
        if modified == self.modified && fingerprint == self.fingerprint {
            return Ok(ReloadStatus { changed: false });
        }

        self.compiled = CompiledTemplate::compile(&source)?;
        self.modified = modified;
        self.fingerprint = fingerprint;
        Ok(ReloadStatus { changed: true })
    }
}

/// Collection of named templates for compiled and hot-reloaded modes.
#[derive(Clone, Debug, Default)]
pub struct TemplateSet {
    templates: BTreeMap<String, TemplateEntry>,
}

impl TemplateSet {
    /// Creates an empty template set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds or replaces an inline compiled template.
    pub fn add_template(&mut self, name: impl Into<String>, source: &str) -> TemplateResult<()> {
        self.templates.insert(
            name.into(),
            TemplateEntry::Inline(CompiledTemplate::compile(source)?),
        );
        Ok(())
    }

    /// Adds or replaces a file-backed template.
    pub fn add_file(
        &mut self,
        name: impl Into<String>,
        path: impl AsRef<Path>,
    ) -> TemplateResult<()> {
        self.templates
            .insert(name.into(), TemplateEntry::File(TemplateFile::load(path)?));
        Ok(())
    }

    /// Returns a named compiled template.
    pub fn get(&self, name: &str) -> TemplateResult<&CompiledTemplate> {
        self.templates
            .get(name)
            .map(TemplateEntry::compiled)
            .ok_or_else(|| TemplateError::Render(format!("missing template `{name}`")))
    }

    /// Renders a named template.
    pub fn render(
        &self,
        name: &str,
        context: &TemplateContext,
    ) -> TemplateResult<Vec<RenderedNode>> {
        self.get(name)?.render(context)
    }

    /// Renders a named template with explicit options.
    pub fn render_with_options(
        &self,
        name: &str,
        context: &TemplateContext,
        options: &RenderOptions,
    ) -> TemplateResult<Vec<RenderedNode>> {
        self.get(name)?.render_with_options(context, options)
    }

    /// Re-reads file-backed templates and returns names that changed.
    pub fn reload_changed(&mut self) -> TemplateResult<Vec<String>> {
        let mut updated = self.templates.clone();
        let mut changed = Vec::new();
        for (name, entry) in &mut updated {
            if let TemplateEntry::File(file) = entry
                && file.reload_if_changed()?.changed
            {
                changed.push(name.clone());
            }
        }
        self.templates = updated;
        Ok(changed)
    }
}

#[derive(Clone, Debug)]
enum TemplateEntry {
    Inline(CompiledTemplate),
    File(TemplateFile),
}

impl TemplateEntry {
    fn compiled(&self) -> &CompiledTemplate {
        match self {
            Self::Inline(template) => template,
            Self::File(file) => file.compiled(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct TemplateFingerprint {
    len: usize,
    hash: u64,
}

impl TemplateFingerprint {
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
    fn parse(raw: &str, base_offset: usize, source: &str) -> TemplateResult<Self> {
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
    root: &'a TemplateContext,
    locals: Vec<(String, Value)>,
}

impl<'a> Scope<'a> {
    fn new(root: &'a TemplateContext) -> Self {
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

    fn resolve(&self, path: &PathExpr) -> TemplateResult<&Value> {
        let Some(first) = path.segments.first() else {
            return Err(TemplateError::Render("empty expression".to_owned()));
        };

        let PathSegment::Field(first) = first else {
            return Err(TemplateError::Render(format!(
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
        .ok_or_else(|| TemplateError::Render(format!("missing value `{first}`")))?;

        for segment in path.segments.iter().skip(1) {
            value = match (value, segment) {
                (Value::Object(object), PathSegment::Field(field)) => {
                    object.get(field).ok_or_else(|| {
                        TemplateError::Render(format!("missing field `{field}` in `{first}`"))
                    })?
                }
                (Value::List(list), PathSegment::Index(index)) => {
                    list.get(*index).ok_or_else(|| {
                        TemplateError::Render(format!(
                            "index `{index}` is out of bounds in `{}`",
                            path.display()
                        ))
                    })?
                }
                (Value::List(_), PathSegment::Field(field)) => {
                    return Err(TemplateError::Render(format!(
                        "cannot access field `{field}` on list `{first}`"
                    )));
                }
                (Value::Object(_), PathSegment::Index(index)) => {
                    return Err(TemplateError::Render(format!(
                        "cannot access index `{index}` on object `{first}`"
                    )));
                }
                (_, PathSegment::Field(field)) => {
                    return Err(TemplateError::Render(format!(
                        "cannot access field `{field}` on scalar `{first}`"
                    )));
                }
                (_, PathSegment::Index(index)) => {
                    return Err(TemplateError::Render(format!(
                        "cannot access index `{index}` on scalar `{first}`"
                    )));
                }
            };
        }
        Ok(value)
    }
}

struct RenderContext {
    limits: TemplateLimits,
    nodes: usize,
    loop_iterations: usize,
}

impl RenderContext {
    fn new(limits: TemplateLimits) -> Self {
        Self {
            limits,
            nodes: 0,
            loop_iterations: 0,
        }
    }

    fn enter_depth(&self, depth: usize) -> TemplateResult<()> {
        if depth > self.limits.max_depth {
            return Err(TemplateError::Render(format!(
                "template depth limit exceeded: {} > {}",
                depth, self.limits.max_depth
            )));
        }
        Ok(())
    }

    fn track_node(&mut self) -> TemplateResult<()> {
        self.nodes += 1;
        if self.nodes > self.limits.max_nodes {
            return Err(TemplateError::Render(format!(
                "node limit exceeded: {} > {}",
                self.nodes, self.limits.max_nodes
            )));
        }
        Ok(())
    }

    fn track_loop_iteration(&mut self) -> TemplateResult<()> {
        self.loop_iterations += 1;
        if self.loop_iterations > self.limits.max_loop_iterations {
            return Err(TemplateError::Render(format!(
                "loop iteration limit exceeded: {} > {}",
                self.loop_iterations, self.limits.max_loop_iterations
            )));
        }
        Ok(())
    }

    fn check_loop_capacity(&self, additional_iterations: usize) -> TemplateResult<()> {
        let requested = self.loop_iterations.saturating_add(additional_iterations);
        if requested > self.limits.max_loop_iterations {
            return Err(TemplateError::Render(format!(
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
    fn max_bytes(self, limits: TemplateLimits) -> usize {
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
    fn new(kind: TextLimitKind, limits: TemplateLimits) -> Self {
        Self {
            value: String::new(),
            max_bytes: kind.max_bytes(limits),
            label: kind.label(),
        }
    }

    fn push_str(&mut self, value: &str) -> TemplateResult<()> {
        if self.value.len().saturating_add(value.len()) > self.max_bytes {
            return Err(TemplateError::Render(format!(
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

fn render_nodes(
    nodes: &[AstNode],
    scope: &mut Scope<'_>,
    render_context: &mut RenderContext,
    depth: usize,
) -> TemplateResult<Vec<RenderedNode>> {
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
                        return Err(TemplateError::Render(format!(
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

fn render_nodes_into_sink<S: TemplateSink>(
    nodes: &[AstNode],
    scope: &mut Scope<'_>,
    render_context: &mut RenderContext,
    depth: usize,
    sink: &mut S,
) -> TemplateResult<()> {
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
                        return Err(TemplateError::Render(format!(
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
) -> TemplateResult<RenderedNode> {
    render_context.track_node()?;
    let mut attributes = BTreeMap::new();
    let mut classes = Vec::new();

    for attribute in &element.attributes {
        let value = render_text_parts(
            &attribute.value,
            scope,
            render_context,
            TextLimitKind::Attribute,
        )?;
        if attribute.name == "class" {
            classes.extend(value.split_whitespace().map(str::to_owned));
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

    Ok(RenderedNode {
        tag: element.tag.clone(),
        classes,
        attributes,
        text,
        children,
    })
}

fn render_text_parts(
    parts: &[TextPart],
    scope: &Scope<'_>,
    render_context: &RenderContext,
    kind: TextLimitKind,
) -> TemplateResult<String> {
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
    limits: TemplateCompileLimits,
    nodes: usize,
    text_parts: usize,
}

impl CompileContext {
    fn new(limits: TemplateCompileLimits) -> Self {
        Self {
            limits,
            nodes: 0,
            text_parts: 0,
        }
    }

    fn check_source(&self, source: &str) -> TemplateResult<()> {
        if source.len() > self.limits.max_source_bytes {
            return Err(parse_error_at(
                source,
                self.limits.max_source_bytes,
                "compile source byte limit exceeded",
            ));
        }
        Ok(())
    }

    fn enter_depth(&self, depth: usize, source: &str, offset: usize) -> TemplateResult<()> {
        if depth > self.limits.max_depth {
            return Err(parse_error_at(
                source,
                offset,
                "compile depth limit exceeded",
            ));
        }
        Ok(())
    }

    fn track_node(&mut self, source: &str, offset: usize) -> TemplateResult<()> {
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
    fn new(source: &'a str, limits: TemplateCompileLimits) -> Self {
        Self {
            source,
            offset: 0,
            compile_context: CompileContext::new(limits),
        }
    }

    fn parse(mut self) -> TemplateResult<TemplateAst> {
        self.compile_context.check_source(self.source)?;
        let nodes = self.parse_nodes(None, 0)?;
        self.skip_ws();
        if !self.eof() {
            return self.error("unexpected trailing template source");
        }
        Ok(TemplateAst { nodes })
    }

    fn parse_nodes(
        &mut self,
        closing_tag: Option<&str>,
        depth: usize,
    ) -> TemplateResult<Vec<AstNode>> {
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

    fn parse_element(&mut self, depth: usize) -> TemplateResult<ElementNode> {
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

    fn parse_attribute(&mut self) -> TemplateResult<AttributeNode> {
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

    fn parse_for(&mut self, depth: usize) -> TemplateResult<AstNode> {
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

    fn parse_if(&mut self, depth: usize) -> TemplateResult<AstNode> {
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

    fn parse_text(&mut self) -> TemplateResult<Vec<TextPart>> {
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

    fn parse_identifier(&mut self) -> TemplateResult<String> {
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

    fn read_until(&mut self, delimiter: &str) -> TemplateResult<&'a str> {
        let start = self.offset;
        let Some(relative_end) = self.source[self.offset..].find(delimiter) else {
            return self.error(&format!("missing `{delimiter}`"));
        };
        let end = self.offset + relative_end;
        self.offset = end + delimiter.len();
        Ok(&self.source[start..end])
    }

    fn expect(&mut self, expected: &str) -> TemplateResult<()> {
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

    fn error<T>(&self, message: &str) -> TemplateResult<T> {
        Err(parse_error_at(self.source, self.offset, message))
    }
}

fn parse_text_parts(
    raw: &str,
    base_offset: usize,
    source: &str,
    limits: TemplateCompileLimits,
    text_parts: &mut usize,
    depth: usize,
) -> TemplateResult<Vec<TextPart>> {
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
    limits: TemplateCompileLimits,
    text_parts: &mut usize,
) -> TemplateResult<()> {
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
) -> TemplateResult<(Option<usize>, usize)> {
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
) -> TemplateResult<Vec<PathSegment>> {
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

fn validate_identifier(value: &str) -> TemplateResult<String> {
    validate_identifier_at(value, 0, "")
}

fn validate_identifier_at(value: &str, offset: usize, source: &str) -> TemplateResult<String> {
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

fn parse_error_at(source: &str, offset: usize, message: &str) -> TemplateError {
    let (line, column) = line_column(source, offset.min(source.len()));
    TemplateError::Parse {
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
