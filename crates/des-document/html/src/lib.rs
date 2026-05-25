//! Browser-grade HTML ingestion for Data Engine Studio document markup.
//!
//! This crate parses HTML documents and fragments with HTML5 tree-construction
//! semantics, maps the resulting tree into `des-document` primitives, and keeps
//! behavior declarative through Rust command/event hooks. It does not execute
//! JavaScript and does not embed template logic in HTML.

use des_document::{
    Document, DocumentActionFrame, DocumentActionSurface, DocumentBuilder, DocumentCommandRegistry,
    DocumentInput, DocumentOutput, DocumentProjection, DocumentProjectionReport, DocumentView,
    Element, ElementBehaviorEvent, ElementBehaviorHook, ElementSpec, Size, StyleSheet, TextContent,
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

/// Common app-facing imports for browser-authored document UIs.
///
/// This prelude lets application code parse browser-grade HTML/CSS, create a
/// retained document view, project state, and collect typed Rust actions through
/// one intentional front door. It re-exports the `des-document` authoring
/// prelude because parsed HTML emits the same egui-free document contracts as
/// Rust-authored widgets.
pub mod prelude {
    pub use crate::{
        HtmlBehaviorHook, HtmlDiagnostic, HtmlDiagnosticCode, HtmlDocument, HtmlError, HtmlFile,
        HtmlNode, HtmlResult, HtmlSet, HtmlStylesheet, ReloadStatus,
    };
    pub use des_document::prelude::*;
}

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
    /// The parsed HTML emitted a document, but a state projection could not be applied.
    Document(String),
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
            Self::Document(message) => write!(f, "html document error: {message}"),
        }
    }
}

impl std::error::Error for HtmlError {}

impl From<std::io::Error> for HtmlError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error.to_string())
    }
}

impl From<des_document::DocumentError> for HtmlError {
    fn from(error: des_document::DocumentError) -> Self {
        Self::Document(error.to_string())
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

    /// Returns the top-level parsed HTML nodes in source order.
    pub fn children(&self) -> &[HtmlNode] {
        &self.children
    }

    /// Returns non-fatal diagnostics collected while mapping HTML semantics.
    pub fn diagnostics(&self) -> &[HtmlDiagnostic] {
        &self.diagnostics
    }

    /// Returns true when this parsed document has no authoring diagnostics.
    pub fn is_clean(&self) -> bool {
        self.diagnostics.is_empty()
    }

    /// Returns true when this parsed document has authoring diagnostics.
    pub fn has_diagnostics(&self) -> bool {
        !self.is_clean()
    }

    /// Finds the first parsed node with the supplied HTML id.
    pub fn find_by_id(&self, id: &str) -> Option<&HtmlNode> {
        self.children.iter().find_map(|child| child.find_by_id(id))
    }

    /// Finds the first parsed node with the supplied tag name.
    pub fn first_by_tag(&self, tag: &str) -> Option<&HtmlNode> {
        self.children
            .iter()
            .find_map(|child| child.first_by_tag(tag))
    }

    /// Returns parsed nodes with the supplied tag name in document order.
    pub fn nodes_by_tag(&self, tag: &str) -> Vec<&HtmlNode> {
        let mut nodes = Vec::new();
        for child in &self.children {
            child.collect_by_tag(tag, &mut nodes);
        }
        nodes
    }

    /// Returns parsed nodes with the supplied class in document order.
    pub fn nodes_with_class(&self, class: &str) -> Vec<&HtmlNode> {
        let mut nodes = Vec::new();
        for child in &self.children {
            child.collect_with_class(class, &mut nodes);
        }
        nodes
    }

    /// Creates a retained document from this HTML tree.
    pub fn to_document(&self, viewport: Size) -> HtmlResult<Document> {
        Ok(Document::build(viewport, |document| {
            self.write_to_document_builder(document);
        }))
    }

    /// Creates a ready-to-update retained document view with an empty stylesheet.
    pub fn to_view(&self, viewport: Size) -> HtmlResult<DocumentView> {
        Ok(DocumentView::new(
            self.to_document(viewport)?,
            StyleSheet::new(),
        ))
    }

    /// Pairs this parsed HTML tree with a typed document stylesheet.
    pub fn with_stylesheet(self, stylesheet: StyleSheet) -> HtmlStylesheet {
        HtmlStylesheet::new(self, stylesheet)
    }

    /// Parses CSS and pairs it with this parsed HTML tree.
    pub fn with_css(self, css: &str) -> HtmlResult<HtmlStylesheet> {
        Ok(self.with_stylesheet(parse_stylesheet(css)?))
    }

    /// Parses CSS with browser-like error recovery and pairs it with this HTML tree.
    pub fn with_css_forgiving(self, css: &str) -> HtmlResult<HtmlStylesheet> {
        Ok(self.with_stylesheet(parse_stylesheet_forgiving(css)?))
    }

    /// Creates a ready-to-update document view from this HTML tree and stylesheet.
    pub fn to_view_with_stylesheet(
        &self,
        viewport: Size,
        stylesheet: StyleSheet,
    ) -> HtmlResult<DocumentView> {
        Ok(DocumentView::new(self.to_document(viewport)?, stylesheet))
    }

    /// Parses CSS and creates a ready-to-update document view from this HTML tree.
    pub fn to_view_with_css(&self, viewport: Size, css: &str) -> HtmlResult<DocumentView> {
        self.to_view_with_stylesheet(viewport, parse_stylesheet(css)?)
    }

    /// Parses CSS with browser-like error recovery and creates a document view.
    pub fn to_view_with_css_forgiving(
        &self,
        viewport: Size,
        css: &str,
    ) -> HtmlResult<DocumentView> {
        self.to_view_with_stylesheet(viewport, parse_stylesheet_forgiving(css)?)
    }

    /// Creates an action surface from this HTML tree and typed Rust commands.
    pub fn to_action_surface<Action>(
        &self,
        viewport: Size,
        commands: DocumentCommandRegistry<Action>,
    ) -> HtmlResult<DocumentActionSurface<Action>> {
        Ok(self.to_view(viewport)?.action_surface(commands))
    }

    /// Creates an action surface and configures typed Rust commands in one hook.
    pub fn to_action_surface_with<Action>(
        &self,
        viewport: Size,
        configure: impl FnOnce(&mut DocumentCommandRegistry<Action>),
    ) -> HtmlResult<DocumentActionSurface<Action>> {
        Ok(self.to_view(viewport)?.action_surface_with(configure))
    }

    /// Creates an action surface from this HTML tree, stylesheet, and typed commands.
    pub fn to_action_surface_with_stylesheet<Action>(
        &self,
        viewport: Size,
        stylesheet: StyleSheet,
        commands: DocumentCommandRegistry<Action>,
    ) -> HtmlResult<DocumentActionSurface<Action>> {
        Ok(self
            .to_view_with_stylesheet(viewport, stylesheet)?
            .action_surface(commands))
    }

    /// Creates an action surface from this HTML tree and stylesheet, configuring commands in place.
    pub fn to_action_surface_with_stylesheet_and<Action>(
        &self,
        viewport: Size,
        stylesheet: StyleSheet,
        configure: impl FnOnce(&mut DocumentCommandRegistry<Action>),
    ) -> HtmlResult<DocumentActionSurface<Action>> {
        Ok(self
            .to_view_with_stylesheet(viewport, stylesheet)?
            .action_surface_with(configure))
    }

    /// Resolves this HTML tree with an empty stylesheet.
    pub fn update(&self, viewport: Size) -> HtmlResult<DocumentOutput> {
        self.to_view(viewport).map(|mut view| view.update())
    }

    /// Resolves this HTML tree with an empty stylesheet and collects typed Rust actions.
    pub fn update_actions<Action>(
        &self,
        viewport: Size,
        registry: &DocumentCommandRegistry<Action>,
    ) -> HtmlResult<DocumentActionFrame<Action>>
    where
        Action: Clone,
    {
        self.to_view(viewport)
            .map(|mut view| view.update_actions(registry))
    }

    /// Routes input through this HTML tree with an empty stylesheet.
    pub fn update_with_input(
        &self,
        viewport: Size,
        input: DocumentInput,
    ) -> HtmlResult<DocumentOutput> {
        self.to_view(viewport)
            .map(|mut view| view.update_with_input(input))
    }

    /// Routes input through this HTML tree and collects typed Rust actions.
    pub fn update_with_input_actions<Action>(
        &self,
        viewport: Size,
        input: DocumentInput,
        registry: &DocumentCommandRegistry<Action>,
    ) -> HtmlResult<DocumentActionFrame<Action>>
    where
        Action: Clone,
    {
        self.to_view(viewport)
            .map(|mut view| view.update_with_input_actions(input, registry))
    }

    /// Parses CSS and creates an action surface configured with typed Rust commands.
    pub fn to_action_surface_with_css<Action>(
        &self,
        viewport: Size,
        css: &str,
        configure: impl FnOnce(&mut DocumentCommandRegistry<Action>),
    ) -> HtmlResult<DocumentActionSurface<Action>> {
        Ok(self
            .to_view_with_css(viewport, css)?
            .action_surface_with(configure))
    }

    /// Parses forgiving CSS and creates an action surface with typed Rust commands.
    pub fn to_action_surface_with_css_forgiving<Action>(
        &self,
        viewport: Size,
        css: &str,
        configure: impl FnOnce(&mut DocumentCommandRegistry<Action>),
    ) -> HtmlResult<DocumentActionSurface<Action>> {
        Ok(self
            .to_view_with_css_forgiving(viewport, css)?
            .action_surface_with(configure))
    }

    /// Parses CSS, resolves this HTML tree, and returns the first output frame.
    pub fn update_with_css(&self, viewport: Size, css: &str) -> HtmlResult<DocumentOutput> {
        self.to_view_with_css(viewport, css)
            .map(|mut view| view.update())
    }

    /// Parses forgiving CSS, resolves this HTML tree, and returns the first output frame.
    pub fn update_with_css_forgiving(
        &self,
        viewport: Size,
        css: &str,
    ) -> HtmlResult<DocumentOutput> {
        self.to_view_with_css_forgiving(viewport, css)
            .map(|mut view| view.update())
    }

    /// Parses CSS, resolves this HTML tree, and collects typed Rust actions.
    pub fn update_actions_with_css<Action>(
        &self,
        viewport: Size,
        css: &str,
        registry: &DocumentCommandRegistry<Action>,
    ) -> HtmlResult<DocumentActionFrame<Action>>
    where
        Action: Clone,
    {
        self.to_view_with_css(viewport, css)
            .map(|mut view| view.update_actions(registry))
    }

    /// Parses forgiving CSS, resolves this HTML tree, and collects typed Rust actions.
    pub fn update_actions_with_css_forgiving<Action>(
        &self,
        viewport: Size,
        css: &str,
        registry: &DocumentCommandRegistry<Action>,
    ) -> HtmlResult<DocumentActionFrame<Action>>
    where
        Action: Clone,
    {
        self.to_view_with_css_forgiving(viewport, css)
            .map(|mut view| view.update_actions(registry))
    }

    /// Parses CSS, routes input through this HTML tree, and returns output.
    pub fn update_with_input_and_css(
        &self,
        viewport: Size,
        input: DocumentInput,
        css: &str,
    ) -> HtmlResult<DocumentOutput> {
        self.to_view_with_css(viewport, css)
            .map(|mut view| view.update_with_input(input))
    }

    /// Parses CSS, routes input through this HTML tree, and collects typed actions.
    pub fn update_with_input_actions_and_css<Action>(
        &self,
        viewport: Size,
        input: DocumentInput,
        css: &str,
        registry: &DocumentCommandRegistry<Action>,
    ) -> HtmlResult<DocumentActionFrame<Action>>
    where
        Action: Clone,
    {
        self.to_view_with_css(viewport, css)
            .map(|mut view| view.update_with_input_actions(input, registry))
    }

    /// Parses forgiving CSS, routes input through this HTML tree, and returns output.
    pub fn update_with_input_and_css_forgiving(
        &self,
        viewport: Size,
        input: DocumentInput,
        css: &str,
    ) -> HtmlResult<DocumentOutput> {
        self.to_view_with_css_forgiving(viewport, css)
            .map(|mut view| view.update_with_input(input))
    }

    /// Parses forgiving CSS, routes input through this HTML tree, and collects typed actions.
    pub fn update_with_input_actions_and_css_forgiving<Action>(
        &self,
        viewport: Size,
        input: DocumentInput,
        css: &str,
        registry: &DocumentCommandRegistry<Action>,
    ) -> HtmlResult<DocumentActionFrame<Action>>
    where
        Action: Clone,
    {
        self.to_view_with_css_forgiving(viewport, css)
            .map(|mut view| view.update_with_input_actions(input, registry))
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
    /// Pairs a parsed HTML tree with a parsed document stylesheet.
    pub fn new(html: HtmlDocument, stylesheet: StyleSheet) -> Self {
        Self { html, stylesheet }
    }

    /// Returns the browser-parsed HTML document or fragment.
    pub fn html(&self) -> &HtmlDocument {
        &self.html
    }

    /// Returns the parsed document stylesheet.
    pub fn stylesheet(&self) -> &StyleSheet {
        &self.stylesheet
    }

    /// Parses an HTML document and CSS stylesheet into typed document inputs.
    pub fn parse(html: &str, css: &str) -> HtmlResult<Self> {
        let stylesheet = parse_stylesheet(css)?;
        Ok(Self::new(HtmlDocument::parse(html)?, stylesheet))
    }

    /// Parses an HTML document and forgiving CSS stylesheet into typed inputs.
    pub fn parse_forgiving(html: &str, css: &str) -> HtmlResult<Self> {
        let stylesheet = parse_stylesheet_forgiving(css)?;
        Ok(Self::new(HtmlDocument::parse(html)?, stylesheet))
    }

    /// Parses an HTML fragment and CSS stylesheet into typed document inputs.
    pub fn parse_fragment(html: &str, css: &str) -> HtmlResult<Self> {
        let stylesheet = parse_stylesheet(css)?;
        Ok(Self::new(HtmlDocument::parse_fragment(html)?, stylesheet))
    }

    /// Parses an HTML fragment and forgiving CSS stylesheet into typed inputs.
    pub fn parse_fragment_forgiving(html: &str, css: &str) -> HtmlResult<Self> {
        let stylesheet = parse_stylesheet_forgiving(css)?;
        Ok(Self::new(HtmlDocument::parse_fragment(html)?, stylesheet))
    }

    /// Reads HTML and CSS files and parses them into typed document inputs.
    pub fn load_files(html_path: impl AsRef<Path>, css_path: impl AsRef<Path>) -> HtmlResult<Self> {
        Ok(Self::new(
            HtmlDocument::load(html_path)?,
            parse_stylesheet(&fs::read_to_string(css_path)?)?,
        ))
    }

    /// Reads HTML and CSS files, recovering from invalid CSS rules like browsers do.
    pub fn load_files_forgiving(
        html_path: impl AsRef<Path>,
        css_path: impl AsRef<Path>,
    ) -> HtmlResult<Self> {
        Ok(Self::new(
            HtmlDocument::load(html_path)?,
            parse_stylesheet_forgiving(&fs::read_to_string(css_path)?)?,
        ))
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

    /// Creates a ready-to-update action surface from parsed HTML/CSS and typed Rust commands.
    pub fn to_action_surface<Action>(
        &self,
        viewport: Size,
        commands: DocumentCommandRegistry<Action>,
    ) -> HtmlResult<DocumentActionSurface<Action>> {
        Ok(self.to_view(viewport)?.action_surface(commands))
    }

    /// Creates an action surface and configures typed Rust commands in one hook.
    pub fn to_action_surface_with<Action>(
        &self,
        viewport: Size,
        configure: impl FnOnce(&mut DocumentCommandRegistry<Action>),
    ) -> HtmlResult<DocumentActionSurface<Action>> {
        Ok(self.to_view(viewport)?.action_surface_with(configure))
    }

    /// Consumes the parsed HTML/CSS into an action surface paired with typed Rust commands.
    pub fn into_action_surface<Action>(
        self,
        viewport: Size,
        commands: DocumentCommandRegistry<Action>,
    ) -> HtmlResult<DocumentActionSurface<Action>> {
        Ok(self.into_view(viewport)?.action_surface(commands))
    }

    /// Consumes parsed HTML/CSS into an action surface configured in one hook.
    pub fn into_action_surface_with<Action>(
        self,
        viewport: Size,
        configure: impl FnOnce(&mut DocumentCommandRegistry<Action>),
    ) -> HtmlResult<DocumentActionSurface<Action>> {
        Ok(self.into_view(viewport)?.action_surface_with(configure))
    }

    /// Creates a view, applies retained state projection, and returns both.
    pub fn to_view_with_projection(
        &self,
        viewport: Size,
        projection: &DocumentProjection,
    ) -> HtmlResult<(DocumentProjectionReport, DocumentView)> {
        let mut view = self.to_view(viewport)?;
        let report = view.project(projection)?;
        Ok((report, view))
    }

    /// Creates a view and applies retained state projection built in place.
    pub fn to_view_projected_with(
        &self,
        viewport: Size,
        project: impl FnOnce(&mut DocumentProjection),
    ) -> HtmlResult<(DocumentProjectionReport, DocumentView)> {
        let mut projection = DocumentProjection::new();
        project(&mut projection);
        self.to_view_with_projection(viewport, &projection)
    }

    /// Creates a view, applies retained state projection, resolves it, and returns the frame.
    pub fn update_with_projection(
        &self,
        viewport: Size,
        projection: &DocumentProjection,
    ) -> HtmlResult<(DocumentProjectionReport, DocumentOutput)> {
        let mut view = self.to_view(viewport)?;
        Ok(view.project_and_update(projection)?)
    }

    /// Creates a view, builds retained state projection in place, and resolves it.
    pub fn update_projected_with(
        &self,
        viewport: Size,
        project: impl FnOnce(&mut DocumentProjection),
    ) -> HtmlResult<(DocumentProjectionReport, DocumentOutput)> {
        let mut projection = DocumentProjection::new();
        project(&mut projection);
        self.update_with_projection(viewport, &projection)
    }

    /// Creates a projected view, resolves it, and collects typed Rust actions.
    pub fn update_with_projection_actions<Action>(
        &self,
        viewport: Size,
        projection: &DocumentProjection,
        registry: &DocumentCommandRegistry<Action>,
    ) -> HtmlResult<(DocumentProjectionReport, DocumentActionFrame<Action>)>
    where
        Action: Clone,
    {
        let mut view = self.to_view(viewport)?;
        Ok(view.project_and_update_actions(projection, registry)?)
    }

    /// Builds retained state projection in place, resolves it, and collects typed actions.
    pub fn update_projected_with_actions<Action>(
        &self,
        viewport: Size,
        project: impl FnOnce(&mut DocumentProjection),
        registry: &DocumentCommandRegistry<Action>,
    ) -> HtmlResult<(DocumentProjectionReport, DocumentActionFrame<Action>)>
    where
        Action: Clone,
    {
        let mut projection = DocumentProjection::new();
        project(&mut projection);
        self.update_with_projection_actions(viewport, &projection, registry)
    }

    /// Creates a projected view, routes input, and returns the resolved output.
    pub fn update_with_input_and_projection(
        &self,
        viewport: Size,
        input: DocumentInput,
        projection: &DocumentProjection,
    ) -> HtmlResult<(DocumentProjectionReport, DocumentOutput)> {
        let mut view = self.to_view(viewport)?;
        Ok(view.project_and_update_with_input(projection, input)?)
    }

    /// Builds retained state projection in place, routes input, and returns the output.
    pub fn update_with_input_projected_with(
        &self,
        viewport: Size,
        input: DocumentInput,
        project: impl FnOnce(&mut DocumentProjection),
    ) -> HtmlResult<(DocumentProjectionReport, DocumentOutput)> {
        let mut projection = DocumentProjection::new();
        project(&mut projection);
        self.update_with_input_and_projection(viewport, input, &projection)
    }

    /// Creates a projected view, routes input, and collects typed Rust actions.
    pub fn update_with_input_projection_actions<Action>(
        &self,
        viewport: Size,
        input: DocumentInput,
        projection: &DocumentProjection,
        registry: &DocumentCommandRegistry<Action>,
    ) -> HtmlResult<(DocumentProjectionReport, DocumentActionFrame<Action>)>
    where
        Action: Clone,
    {
        let mut view = self.to_view(viewport)?;
        Ok(view.project_and_update_with_input_actions(projection, input, registry)?)
    }

    /// Builds retained state projection in place, routes input, and collects typed actions.
    pub fn update_with_input_projected_with_actions<Action>(
        &self,
        viewport: Size,
        input: DocumentInput,
        project: impl FnOnce(&mut DocumentProjection),
        registry: &DocumentCommandRegistry<Action>,
    ) -> HtmlResult<(DocumentProjectionReport, DocumentActionFrame<Action>)>
    where
        Action: Clone,
    {
        let mut projection = DocumentProjection::new();
        project(&mut projection);
        self.update_with_input_projection_actions(viewport, input, &projection, registry)
    }

    /// Creates a view, resolves the document, and returns the first output frame.
    pub fn update(&self, viewport: Size) -> HtmlResult<DocumentOutput> {
        Ok(self.to_view(viewport)?.update())
    }

    /// Creates a view, resolves the document, and collects typed Rust actions.
    pub fn update_actions<Action>(
        &self,
        viewport: Size,
        registry: &DocumentCommandRegistry<Action>,
    ) -> HtmlResult<DocumentActionFrame<Action>>
    where
        Action: Clone,
    {
        Ok(self.to_view(viewport)?.update_actions(registry))
    }

    /// Creates a view, routes input, and returns the resolved output frame.
    pub fn update_with_input(
        &self,
        viewport: Size,
        input: DocumentInput,
    ) -> HtmlResult<DocumentOutput> {
        Ok(self.to_view(viewport)?.update_with_input(input))
    }

    /// Creates a view, routes input, and collects typed Rust actions.
    pub fn update_with_input_actions<Action>(
        &self,
        viewport: Size,
        input: DocumentInput,
        registry: &DocumentCommandRegistry<Action>,
    ) -> HtmlResult<DocumentActionFrame<Action>>
    where
        Action: Clone,
    {
        Ok(self
            .to_view(viewport)?
            .update_with_input_actions(input, registry))
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

    /// Returns true when this parsed element has the supplied class.
    pub fn has_class(&self, class: &str) -> bool {
        self.classes.iter().any(|candidate| candidate == class)
    }

    /// Finds the first node in this subtree with the supplied HTML id.
    pub fn find_by_id(&self, id: &str) -> Option<&HtmlNode> {
        if self.id.as_deref() == Some(id) {
            return Some(self);
        }
        self.children.iter().find_map(|child| child.find_by_id(id))
    }

    /// Finds the first node in this subtree with the supplied tag name.
    pub fn first_by_tag(&self, tag: &str) -> Option<&HtmlNode> {
        if self.tag == tag {
            return Some(self);
        }
        self.children
            .iter()
            .find_map(|child| child.first_by_tag(tag))
    }

    /// Returns nodes in this subtree with the supplied tag name in document order.
    pub fn nodes_by_tag(&self, tag: &str) -> Vec<&HtmlNode> {
        let mut nodes = Vec::new();
        self.collect_by_tag(tag, &mut nodes);
        nodes
    }

    /// Returns nodes in this subtree with the supplied class in document order.
    pub fn nodes_with_class(&self, class: &str) -> Vec<&HtmlNode> {
        let mut nodes = Vec::new();
        self.collect_with_class(class, &mut nodes);
        nodes
    }

    fn collect_by_tag<'a>(&'a self, tag: &str, nodes: &mut Vec<&'a HtmlNode>) {
        if self.tag == tag {
            nodes.push(self);
        }
        for child in &self.children {
            child.collect_by_tag(tag, nodes);
        }
    }

    fn collect_with_class<'a>(&'a self, class: &str, nodes: &mut Vec<&'a HtmlNode>) {
        if self.has_class(class) {
            nodes.push(self);
        }
        for child in &self.children {
            child.collect_with_class(class, nodes);
        }
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
        let mut spec = ElementSpec::new(element_for_tag(&self.tag)).classes(self.classes.clone());
        if let Some(role) = &self.role {
            spec = spec.role(role.clone());
        }
        spec = spec.attributes(self.attributes.clone());
        spec = spec.behavior_hooks(
            self.behavior_hooks
                .iter()
                .map(HtmlBehaviorHook::to_element_hook),
        );
        if let Some(value) = self.attributes.get("value") {
            spec = spec.value(value.clone());
        }
        if self.attributes.contains_key("disabled") {
            spec = spec.disabled(true);
        }
        if self.attributes.contains_key("selected") {
            spec = spec.selected(true);
        }
        if self.attributes.contains_key("autofocus") {
            spec = spec.focused(true);
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

impl HtmlBehaviorHook {
    /// Creates a behavior hook from an HTML-authored event name.
    pub fn new(event: impl Into<String>, command: impl Into<String>) -> Self {
        Self {
            event: event.into(),
            command: command.into(),
        }
    }

    /// Creates a behavior hook from a typed document behavior intent.
    pub fn on(event: ElementBehaviorEvent, command: impl Into<String>) -> Self {
        Self::new(event.as_str(), command)
    }

    /// Returns the HTML-authored event name.
    pub fn event(&self) -> &str {
        &self.event
    }

    /// Returns the Rust command name declared by the author.
    pub fn command(&self) -> &str {
        &self.command
    }

    /// Returns the parsed typed event intent when this hook maps to a document intent.
    pub fn intent(&self) -> Option<ElementBehaviorEvent> {
        ElementBehaviorEvent::from_name(&self.event)
    }

    /// Returns true when this hook maps to the supplied document behavior intent.
    pub fn matches_intent(&self, intent: ElementBehaviorEvent) -> bool {
        self.intent() == Some(intent)
    }

    /// Converts the parsed HTML hook into the egui-free document hook contract.
    pub fn to_element_hook(&self) -> ElementBehaviorHook {
        ElementBehaviorHook::new(self.event.clone(), self.command.clone())
    }
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

    /// Returns the number of named HTML documents in the set.
    pub fn len(&self) -> usize {
        self.documents.len()
    }

    /// Returns true when the set contains no named HTML documents.
    pub fn is_empty(&self) -> bool {
        self.documents.is_empty()
    }

    /// Returns true when the set contains a document with this name.
    pub fn contains(&self, name: &str) -> bool {
        self.documents.contains_key(name)
    }

    /// Iterates document names in deterministic order.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.documents.keys().map(String::as_str)
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

    /// Creates a retained document from a named HTML document.
    pub fn to_document(&self, name: &str, viewport: Size) -> HtmlResult<Document> {
        self.get(name)?.to_document(viewport)
    }

    /// Creates a ready-to-update document view from a named HTML document.
    pub fn to_view(&self, name: &str, viewport: Size) -> HtmlResult<DocumentView> {
        self.get(name)?.to_view(viewport)
    }

    /// Creates a ready-to-update document view from a named document and stylesheet.
    pub fn to_view_with_stylesheet(
        &self,
        name: &str,
        viewport: Size,
        stylesheet: StyleSheet,
    ) -> HtmlResult<DocumentView> {
        self.get(name)?
            .to_view_with_stylesheet(viewport, stylesheet)
    }

    /// Creates an action surface from a named HTML document and typed commands.
    pub fn to_action_surface<Action>(
        &self,
        name: &str,
        viewport: Size,
        commands: DocumentCommandRegistry<Action>,
    ) -> HtmlResult<DocumentActionSurface<Action>> {
        self.get(name)?.to_action_surface(viewport, commands)
    }

    /// Creates an action surface from a named document and configures typed commands in place.
    pub fn to_action_surface_with<Action>(
        &self,
        name: &str,
        viewport: Size,
        configure: impl FnOnce(&mut DocumentCommandRegistry<Action>),
    ) -> HtmlResult<DocumentActionSurface<Action>> {
        self.get(name)?.to_action_surface_with(viewport, configure)
    }

    /// Creates an action surface from a named document, stylesheet, and typed commands.
    pub fn to_action_surface_with_stylesheet<Action>(
        &self,
        name: &str,
        viewport: Size,
        stylesheet: StyleSheet,
        commands: DocumentCommandRegistry<Action>,
    ) -> HtmlResult<DocumentActionSurface<Action>> {
        self.get(name)?
            .to_action_surface_with_stylesheet(viewport, stylesheet, commands)
    }

    /// Creates an action surface from a named document and stylesheet, configuring commands in place.
    pub fn to_action_surface_with_stylesheet_and<Action>(
        &self,
        name: &str,
        viewport: Size,
        stylesheet: StyleSheet,
        configure: impl FnOnce(&mut DocumentCommandRegistry<Action>),
    ) -> HtmlResult<DocumentActionSurface<Action>> {
        self.get(name)?
            .to_action_surface_with_stylesheet_and(viewport, stylesheet, configure)
    }

    /// Resolves a named HTML document with an empty stylesheet.
    pub fn update(&self, name: &str, viewport: Size) -> HtmlResult<DocumentOutput> {
        self.to_view(name, viewport).map(|mut view| view.update())
    }

    /// Resolves a named HTML document and collects typed Rust actions.
    pub fn update_actions<Action>(
        &self,
        name: &str,
        viewport: Size,
        registry: &DocumentCommandRegistry<Action>,
    ) -> HtmlResult<DocumentActionFrame<Action>>
    where
        Action: Clone,
    {
        self.get(name)?.update_actions(viewport, registry)
    }

    /// Routes input through a named HTML document.
    pub fn update_with_input(
        &self,
        name: &str,
        viewport: Size,
        input: DocumentInput,
    ) -> HtmlResult<DocumentOutput> {
        self.get(name)?.update_with_input(viewport, input)
    }

    /// Routes input through a named HTML document and collects typed Rust actions.
    pub fn update_with_input_actions<Action>(
        &self,
        name: &str,
        viewport: Size,
        input: DocumentInput,
        registry: &DocumentCommandRegistry<Action>,
    ) -> HtmlResult<DocumentActionFrame<Action>>
    where
        Action: Clone,
    {
        self.get(name)?
            .update_with_input_actions(viewport, input, registry)
    }

    /// Resolves a named HTML document with the supplied stylesheet.
    pub fn update_with_stylesheet(
        &self,
        name: &str,
        viewport: Size,
        stylesheet: StyleSheet,
    ) -> HtmlResult<DocumentOutput> {
        self.to_view_with_stylesheet(name, viewport, stylesheet)
            .map(|mut view| view.update())
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

fn parse_stylesheet_forgiving(css: &str) -> HtmlResult<StyleSheet> {
    StyleSheet::parse_css_forgiving(css).map_err(|error| HtmlError::Css(error.to_string()))
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
    behavior_hooks.push(HtmlBehaviorHook::new(event, command));
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
