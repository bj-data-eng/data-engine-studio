//! Runtime template language for Data Engine Studio document markup.
//!
//! The crate parses a constrained XML-like template into an AST, compiles that
//! AST into a reusable template handle, and renders it against an explicit data
//! context. Templates are pure: they do not perform IO, call arbitrary code, or
//! mutate app state while rendering.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Convenient result type for template operations.
pub type TemplateResult<T> = Result<T, TemplateError>;

/// Template parser, renderer, and hot-reload errors.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TemplateError {
    /// The template source is syntactically invalid.
    Parse { offset: usize, message: String },
    /// Rendering referenced data that does not exist or has the wrong shape.
    Render(String),
    /// The template file could not be read or inspected.
    Io(String),
}

impl fmt::Display for TemplateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse { offset, message } => {
                write!(f, "template parse error at {offset}: {message}")
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
                if value.fract() == 0.0 {
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
        Ok(Self {
            ast: Parser::new(source).parse()?,
        })
    }

    /// Renders the compiled template against an explicit context.
    pub fn render(&self, context: &TemplateContext) -> TemplateResult<Vec<RenderedNode>> {
        let mut scope = Scope::new(context);
        render_nodes(&self.ast.nodes, &mut scope)
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
    compiled: CompiledTemplate,
}

impl TemplateFile {
    /// Loads and compiles a template file.
    pub fn load(path: impl AsRef<Path>) -> TemplateResult<Self> {
        let path = path.as_ref().to_path_buf();
        let source = fs::read_to_string(&path)?;
        let metadata = fs::metadata(&path)?;
        Ok(Self {
            path,
            modified: metadata.modified().ok(),
            compiled: CompiledTemplate::compile(&source)?,
        })
    }

    /// Returns the current compiled template.
    pub fn compiled(&self) -> &CompiledTemplate {
        &self.compiled
    }

    /// Re-reads and recompiles the template if the file modification time changed.
    pub fn reload_if_changed(&mut self) -> TemplateResult<ReloadStatus> {
        let metadata = fs::metadata(&self.path)?;
        let modified = metadata.modified().ok();
        if modified == self.modified {
            return Ok(ReloadStatus { changed: false });
        }

        let source = fs::read_to_string(&self.path)?;
        self.compiled = CompiledTemplate::compile(&source)?;
        self.modified = modified;
        Ok(ReloadStatus { changed: true })
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
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PathExpr {
    segments: Vec<String>,
}

impl PathExpr {
    fn parse(raw: &str) -> TemplateResult<Self> {
        let raw = raw.trim();
        if raw.is_empty() {
            return Err(TemplateError::Parse {
                offset: 0,
                message: "empty expression".to_owned(),
            });
        }
        let segments = raw
            .split('.')
            .map(str::trim)
            .map(validate_identifier)
            .collect::<TemplateResult<Vec<_>>>()?;
        Ok(Self { segments })
    }
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

        let mut value = self
            .locals
            .iter()
            .rev()
            .find_map(|(name, value)| (name == first).then_some(value))
            .or_else(|| self.root.get(first))
            .ok_or_else(|| TemplateError::Render(format!("missing value `{first}`")))?;

        for segment in path.segments.iter().skip(1) {
            value = match value {
                Value::Object(object) => object.get(segment).ok_or_else(|| {
                    TemplateError::Render(format!("missing field `{segment}` in `{first}`"))
                })?,
                _ => {
                    return Err(TemplateError::Render(format!(
                        "cannot access field `{segment}` on non-object `{first}`"
                    )));
                }
            };
        }
        Ok(value)
    }
}

fn render_nodes(nodes: &[AstNode], scope: &mut Scope<'_>) -> TemplateResult<Vec<RenderedNode>> {
    let mut rendered = Vec::new();
    for node in nodes {
        match node {
            AstNode::Element(element) => rendered.push(render_element(element, scope)?),
            AstNode::Text(_) => {}
            AstNode::For {
                binding,
                source,
                body,
            } => {
                let values = match scope.resolve(source)? {
                    Value::List(values) => values.clone(),
                    _ => {
                        return Err(TemplateError::Render(format!(
                            "`{}` is not iterable",
                            source.segments.join(".")
                        )));
                    }
                };
                for value in values {
                    scope.push(binding.clone(), value);
                    rendered.extend(render_nodes(body, scope)?);
                    scope.pop();
                }
            }
            AstNode::If {
                condition,
                then_body,
                else_body,
            } => {
                if scope.resolve(condition)?.truthy() {
                    rendered.extend(render_nodes(then_body, scope)?);
                } else {
                    rendered.extend(render_nodes(else_body, scope)?);
                }
            }
        }
    }
    Ok(rendered)
}

fn render_element(element: &ElementNode, scope: &mut Scope<'_>) -> TemplateResult<RenderedNode> {
    let mut attributes = BTreeMap::new();
    let mut classes = Vec::new();

    for attribute in &element.attributes {
        let value = render_text_parts(&attribute.value, scope)?;
        if attribute.name == "class" {
            classes.extend(value.split_whitespace().map(str::to_owned));
        } else {
            attributes.insert(attribute.name.clone(), value);
        }
    }

    let mut children = Vec::new();
    let mut text = String::new();
    for child in &element.children {
        match child {
            AstNode::Text(parts) => text.push_str(&render_text_parts(parts, scope)?),
            AstNode::Element(_) | AstNode::For { .. } | AstNode::If { .. } => {
                children.extend(render_nodes(std::slice::from_ref(child), scope)?);
            }
        }
    }

    let text = if children.is_empty() {
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

fn render_text_parts(parts: &[TextPart], scope: &Scope<'_>) -> TemplateResult<String> {
    let mut rendered = String::new();
    for part in parts {
        match part {
            TextPart::Literal(value) => rendered.push_str(value),
            TextPart::Expr(path) => rendered.push_str(&scope.resolve(path)?.render_scalar()?),
        }
    }
    Ok(rendered)
}

struct Parser<'a> {
    source: &'a str,
    offset: usize,
}

impl<'a> Parser<'a> {
    fn new(source: &'a str) -> Self {
        Self { source, offset: 0 }
    }

    fn parse(mut self) -> TemplateResult<TemplateAst> {
        let nodes = self.parse_nodes(None)?;
        self.skip_ws();
        if !self.eof() {
            return self.error("unexpected trailing template source");
        }
        Ok(TemplateAst { nodes })
    }

    fn parse_nodes(&mut self, closing_tag: Option<&str>) -> TemplateResult<Vec<AstNode>> {
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
                nodes.push(self.parse_for()?);
            } else if self.starts_with("{if ") {
                nodes.push(self.parse_if()?);
            } else if self.starts_with("<") {
                nodes.push(AstNode::Element(self.parse_element()?));
            } else {
                let text = self.parse_text()?;
                if text.iter().any(|part| match part {
                    TextPart::Literal(value) => !value.trim().is_empty(),
                    TextPart::Expr(_) => true,
                }) {
                    nodes.push(AstNode::Text(text));
                }
            }
        }
    }

    fn parse_element(&mut self) -> TemplateResult<ElementNode> {
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
                let children = self.parse_nodes(Some(&tag))?;
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
            value: parse_text_parts(raw, start)?,
        })
    }

    fn parse_for(&mut self) -> TemplateResult<AstNode> {
        self.expect("{for ")?;
        let directive = self.read_until("}")?;
        let parts = directive.split_whitespace().collect::<Vec<_>>();
        if parts.len() != 3 || parts[1] != "in" {
            return self.error("expected `{for item in items}`");
        }
        let binding = validate_identifier(parts[0])?;
        let source = PathExpr::parse(parts[2])?;
        let body = self.parse_nodes(None)?;
        self.expect("{/for}")?;
        Ok(AstNode::For {
            binding,
            source,
            body,
        })
    }

    fn parse_if(&mut self) -> TemplateResult<AstNode> {
        self.expect("{if ")?;
        let condition = PathExpr::parse(self.read_until("}")?)?;
        let then_body = self.parse_nodes(None)?;
        let else_body = if self.starts_with("{else}") {
            self.expect("{else}")?;
            self.parse_nodes(None)?
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
        parse_text_parts(&self.source[start..self.offset], start)
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
        Err(TemplateError::Parse {
            offset: self.offset,
            message: message.to_owned(),
        })
    }
}

fn parse_text_parts(raw: &str, base_offset: usize) -> TemplateResult<Vec<TextPart>> {
    let mut parts = Vec::new();
    let mut cursor = 0;
    while cursor < raw.len() {
        let Some(relative_open) = raw[cursor..].find('{') else {
            parts.push(TextPart::Literal(raw[cursor..].to_owned()));
            break;
        };
        let open = cursor + relative_open;
        if open > cursor {
            parts.push(TextPart::Literal(raw[cursor..open].to_owned()));
        }
        let Some(relative_close) = raw[open..].find('}') else {
            return Err(TemplateError::Parse {
                offset: base_offset + open,
                message: "unterminated interpolation".to_owned(),
            });
        };
        let close = open + relative_close;
        parts.push(TextPart::Expr(PathExpr::parse(&raw[open + 1..close])?));
        cursor = close + 1;
    }
    Ok(parts)
}

fn validate_identifier(value: &str) -> TemplateResult<String> {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return Err(TemplateError::Parse {
            offset: 0,
            message: "empty identifier".to_owned(),
        });
    };
    if !(first.is_ascii_alphabetic() || first == '_') {
        return Err(TemplateError::Parse {
            offset: 0,
            message: format!("invalid identifier `{value}`"),
        });
    }
    if chars.any(|ch| !(ch.is_ascii_alphanumeric() || ch == '_')) {
        return Err(TemplateError::Parse {
            offset: 0,
            message: format!("invalid identifier `{value}`"),
        });
    }
    Ok(value.to_owned())
}
