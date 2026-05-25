//! Browser-grade HTML ingestion for Data Engine Studio document markup.
//!
//! This crate parses HTML documents and fragments with HTML5 tree-construction
//! semantics, maps the resulting tree into `des-document` primitives, and keeps
//! behavior declarative through Rust command/event hooks. It does not execute
//! JavaScript and does not embed template logic in HTML.

use des_document::{
    Document, DocumentBuilder, DocumentView, Element, ElementSpec, Size, StyleSheet, TextContent,
};
use html5ever::tendril::TendrilSink;
use html5ever::{QualName, local_name, ns, parse_document, parse_fragment};
use markup5ever_rcdom::{Handle, NodeData, RcDom};
use std::collections::BTreeMap;
use std::collections::hash_map::DefaultHasher;
use std::fmt;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Convenient result type for HTML operations.
pub type HtmlResult<T> = Result<T, HtmlError>;

/// HTML ingestion, CSS ingestion, and file errors.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlError {
    /// The HTML source could not be mapped into the document contract.
    Parse {
        offset: usize,
        line: usize,
        column: usize,
        message: String,
    },
    /// The companion CSS stylesheet is invalid for the document style model.
    Css(String),
    /// The HTML or CSS file could not be read or inspected.
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
            } => write!(
                f,
                "html parse error at {line}:{column} (offset {offset}): {message}"
            ),
            Self::Css(message) => write!(f, "html css error: {message}"),
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

/// Browser-parsed HTML document or fragment.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct HtmlDocument {
    /// Top-level HTML nodes in source order.
    pub children: Vec<HtmlNode>,
    /// Non-fatal authoring diagnostics collected while mapping HTML semantics.
    pub diagnostics: Vec<HtmlDiagnostic>,
}

impl HtmlDocument {
    /// Parses an HTML document using HTML5 tree-construction rules.
    pub fn parse(source: &str) -> HtmlResult<Self> {
        let dom = parse_document(RcDom::default(), Default::default()).one(source);
        let mut diagnostics = Vec::new();
        Ok(Self {
            children: rcdom_children_to_html(&dom.document.children.borrow(), &mut diagnostics),
            diagnostics,
        })
    }

    /// Reads and parses an HTML document file using HTML5 tree-construction rules.
    pub fn load(path: impl AsRef<Path>) -> HtmlResult<Self> {
        Self::parse(&fs::read_to_string(path)?)
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
        let mut diagnostics = Vec::new();
        Ok(Self {
            children: rcdom_fragment_children_to_html(
                &dom.document.children.borrow(),
                &mut diagnostics,
            ),
            diagnostics,
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
    /// Parses an HTML fragment and CSS stylesheet into typed document inputs.
    pub fn parse_fragment(html: &str, css: &str) -> HtmlResult<Self> {
        let stylesheet = parse_stylesheet(css)?;
        Ok(Self {
            html: HtmlDocument::parse_fragment(html)?,
            stylesheet,
        })
    }

    /// Reads HTML and CSS files and parses them into typed document inputs.
    pub fn load_files(html_path: impl AsRef<Path>, css_path: impl AsRef<Path>) -> HtmlResult<Self> {
        Ok(Self {
            html: HtmlDocument::load(html_path)?,
            stylesheet: parse_stylesheet(&fs::read_to_string(css_path)?)?,
        })
    }

    /// Creates a ready-to-update retained document view from the parsed assets.
    pub fn to_view(&self, viewport: Size) -> HtmlResult<DocumentView> {
        Ok(DocumentView::new(
            self.html.to_document(viewport)?,
            self.stylesheet.clone(),
        ))
    }

    /// Consumes the parsed assets into a ready-to-update retained document view.
    pub fn into_view(self, viewport: Size) -> HtmlResult<DocumentView> {
        Ok(DocumentView::new(
            self.html.to_document(viewport)?,
            self.stylesheet,
        ))
    }
}

/// A parsed HTML element or text node.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlNode {
    /// Element tag name, or `#text` for explicit text nodes.
    pub tag: String,
    /// Element id from the `id` attribute, when present.
    pub id: Option<String>,
    /// Class names from the `class` attribute.
    pub classes: Vec<String>,
    /// Element role from the `role` attribute, when present.
    pub role: Option<String>,
    /// Non-id/class/role attributes.
    pub attributes: BTreeMap<String, String>,
    /// Rust behavior hooks declared through `on:*` or `data-command` attributes.
    pub behavior_hooks: Vec<HtmlBehaviorHook>,
    /// Text content when this is a text node or a text-only element.
    pub text: Option<String>,
    /// Child nodes.
    pub children: Vec<HtmlNode>,
}

impl HtmlNode {
    /// Creates an explicit text node for parsed HTML mixed content.
    pub fn text_node(text: impl Into<String>) -> Self {
        Self {
            tag: "#text".to_owned(),
            id: None,
            classes: Vec::new(),
            role: None,
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
        if let Some(role) = &self.role {
            spec = spec.role(role.clone());
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

/// Rust behavior declared from HTML attributes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlBehaviorHook {
    /// Event name such as `click`, `input`, or `submit`.
    pub event: String,
    /// Rust command/event intent declared by the author.
    pub command: String,
}

/// Non-fatal HTML authoring diagnostic.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlDiagnostic {
    /// Stable diagnostic category.
    pub code: HtmlDiagnosticCode,
    /// Human-readable diagnostic message.
    pub message: String,
    /// Best-effort element tag connected to the diagnostic.
    pub tag: Option<String>,
    /// Best-effort attribute name connected to the diagnostic.
    pub attribute: Option<String>,
}

impl HtmlDiagnostic {
    fn new(
        code: HtmlDiagnosticCode,
        message: impl Into<String>,
        tag: Option<String>,
        attribute: Option<String>,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            tag,
            attribute,
        }
    }
}

/// Stable HTML diagnostic codes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlDiagnosticCode {
    EmptyBehaviorEvent,
    EmptyBehaviorCommand,
    ScriptElementIgnored,
    JavaScriptEventAttributeIgnored,
}

/// Hot-reload status returned after checking an HTML file.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReloadStatus {
    /// True when the file changed and the HTML document was reloaded.
    pub changed: bool,
}

/// File-backed browser HTML document for polling-style hot reload.
#[derive(Clone, Debug)]
pub struct HtmlFile {
    path: PathBuf,
    modified: Option<SystemTime>,
    fingerprint: HtmlFingerprint,
    document: HtmlDocument,
}

impl HtmlFile {
    /// Loads and parses an HTML file.
    pub fn load(path: impl AsRef<Path>) -> HtmlResult<Self> {
        let path = path.as_ref().to_path_buf();
        let source = fs::read_to_string(&path)?;
        let metadata = fs::metadata(&path)?;
        let fingerprint = HtmlFingerprint::new(&source);
        Ok(Self {
            path,
            modified: metadata.modified().ok(),
            fingerprint,
            document: HtmlDocument::parse(&source)?,
        })
    }

    /// Returns the current parsed document.
    pub fn document(&self) -> &HtmlDocument {
        &self.document
    }

    /// Re-reads and reparses the HTML document if the file changed.
    pub fn reload_if_changed(&mut self) -> HtmlResult<ReloadStatus> {
        let source = fs::read_to_string(&self.path)?;
        let metadata = fs::metadata(&self.path)?;
        let modified = metadata.modified().ok();
        let fingerprint = HtmlFingerprint::new(&source);
        if modified == self.modified && fingerprint == self.fingerprint {
            return Ok(ReloadStatus { changed: false });
        }

        self.document = HtmlDocument::parse(&source)?;
        self.modified = modified;
        self.fingerprint = fingerprint;
        Ok(ReloadStatus { changed: true })
    }
}

/// Collection of named parsed HTML documents.
#[derive(Clone, Debug, Default)]
pub struct HtmlSet {
    documents: BTreeMap<String, HtmlEntry>,
}

impl HtmlSet {
    /// Creates an empty HTML set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds or replaces an inline parsed HTML fragment.
    pub fn add_fragment(&mut self, name: impl Into<String>, source: &str) -> HtmlResult<()> {
        self.documents.insert(
            name.into(),
            HtmlEntry::Inline(HtmlDocument::parse_fragment(source)?),
        );
        Ok(())
    }

    /// Adds or replaces a file-backed HTML document.
    pub fn add_file(&mut self, name: impl Into<String>, path: impl AsRef<Path>) -> HtmlResult<()> {
        self.documents
            .insert(name.into(), HtmlEntry::File(HtmlFile::load(path)?));
        Ok(())
    }

    /// Returns a named parsed HTML document.
    pub fn get(&self, name: &str) -> HtmlResult<&HtmlDocument> {
        self.documents
            .get(name)
            .map(HtmlEntry::document)
            .ok_or_else(|| HtmlError::Parse {
                offset: 0,
                line: 1,
                column: 1,
                message: format!("missing html document `{name}`"),
            })
    }

    /// Re-reads file-backed HTML documents and returns names that changed.
    pub fn reload_changed(&mut self) -> HtmlResult<Vec<String>> {
        let mut updated = self.documents.clone();
        let mut changed = Vec::new();
        for (name, entry) in &mut updated {
            if let HtmlEntry::File(file) = entry
                && file.reload_if_changed()?.changed
            {
                changed.push(name.clone());
            }
        }
        self.documents = updated;
        Ok(changed)
    }
}

#[derive(Clone, Debug)]
enum HtmlEntry {
    Inline(HtmlDocument),
    File(HtmlFile),
}

impl HtmlEntry {
    fn document(&self) -> &HtmlDocument {
        match self {
            Self::Inline(document) => document,
            Self::File(file) => file.document(),
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

fn parse_stylesheet(css: &str) -> HtmlResult<StyleSheet> {
    StyleSheet::parse_css(css).map_err(|error| HtmlError::Css(error.to_string()))
}

fn rcdom_children_to_html(
    children: &[Handle],
    diagnostics: &mut Vec<HtmlDiagnostic>,
) -> Vec<HtmlNode> {
    let mut nodes = Vec::new();
    for child in children {
        append_rcdom_node(child, &mut nodes, diagnostics);
    }
    nodes
}

fn rcdom_fragment_children_to_html(
    children: &[Handle],
    diagnostics: &mut Vec<HtmlDiagnostic>,
) -> Vec<HtmlNode> {
    let mut nodes = rcdom_children_to_html(children, diagnostics);
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

fn append_rcdom_node(
    handle: &Handle,
    nodes: &mut Vec<HtmlNode>,
    diagnostics: &mut Vec<HtmlDiagnostic>,
) {
    match &handle.data {
        NodeData::Document => nodes.extend(rcdom_children_to_html(
            &handle.children.borrow(),
            diagnostics,
        )),
        NodeData::Doctype { .. } | NodeData::Comment { .. } => {}
        NodeData::Text { contents } => {
            let text = contents.borrow().to_string();
            if !text.trim().is_empty() {
                nodes.push(HtmlNode::text_node(text));
            }
        }
        NodeData::Element { name, attrs, .. } => {
            let tag = name.local.to_string();
            if tag == "script" {
                diagnostics.push(HtmlDiagnostic::new(
                    HtmlDiagnosticCode::ScriptElementIgnored,
                    "script elements are ignored; JavaScript is not part of the document runtime",
                    Some(tag),
                    None,
                ));
                return;
            }

            let mut id = None;
            let mut classes = Vec::new();
            let mut role = None;
            let mut attributes = BTreeMap::new();
            let mut behavior_hooks = Vec::new();

            for attr in attrs.borrow().iter() {
                let name = html_attribute_name(&attr.name);
                let value = attr.value.to_string();
                if name == "id" {
                    id = Some(value);
                } else if name == "class" {
                    classes.extend(value.split_whitespace().map(str::to_owned));
                } else if name == "role" {
                    role = Some(value);
                } else if let Some(event) = name.strip_prefix("on:") {
                    push_behavior_hook(&tag, &name, event, value, &mut behavior_hooks, diagnostics);
                } else if let Some(command_event) = name.strip_prefix("data-command") {
                    let event = command_event
                        .strip_prefix(':')
                        .filter(|event| !event.is_empty())
                        .unwrap_or("click");
                    push_behavior_hook(
                        &tag,
                        &name,
                        event,
                        value.clone(),
                        &mut behavior_hooks,
                        diagnostics,
                    );
                    attributes.insert(name, value);
                } else if is_javascript_event_attribute(&name) {
                    diagnostics.push(HtmlDiagnostic::new(
                        HtmlDiagnosticCode::JavaScriptEventAttributeIgnored,
                        "JavaScript event attributes are ignored; use `on:event` Rust hooks",
                        Some(tag.clone()),
                        Some(name),
                    ));
                } else {
                    attributes.insert(name, value);
                }
            }

            let children = rcdom_children_to_html(&handle.children.borrow(), diagnostics);
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
                role,
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

fn push_behavior_hook(
    tag: &str,
    attribute: &str,
    event: &str,
    command: String,
    behavior_hooks: &mut Vec<HtmlBehaviorHook>,
    diagnostics: &mut Vec<HtmlDiagnostic>,
) {
    if event.trim().is_empty() {
        diagnostics.push(HtmlDiagnostic::new(
            HtmlDiagnosticCode::EmptyBehaviorEvent,
            "behavior hook is missing an event name",
            Some(tag.to_owned()),
            Some(attribute.to_owned()),
        ));
        return;
    }
    if command.trim().is_empty() {
        diagnostics.push(HtmlDiagnostic::new(
            HtmlDiagnosticCode::EmptyBehaviorCommand,
            "behavior hook is missing a Rust command",
            Some(tag.to_owned()),
            Some(attribute.to_owned()),
        ));
        return;
    }
    behavior_hooks.push(HtmlBehaviorHook {
        event: event.to_owned(),
        command,
    });
}

fn is_javascript_event_attribute(name: &str) -> bool {
    name.len() > 2 && name.starts_with("on") && !name.starts_with("on:")
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
    format!("html/text-{}", stable_path(path))
}

fn stable_path(path: &[usize]) -> String {
    path.iter()
        .map(usize::to_string)
        .collect::<Vec<_>>()
        .join("-")
}
