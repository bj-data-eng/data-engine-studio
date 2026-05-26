//! Browser-grade HTML ingestion for Data Engine Studio document markup.
//!
//! This crate parses HTML documents and fragments with HTML5 tree-construction
//! semantics, maps the resulting tree into `des-document` primitives, and keeps
//! behavior declarative through Rust command/event hooks. It does not execute
//! JavaScript and does not embed template logic in HTML.
//!
//! ```
//! use des_html::prelude::*;
//!
//! #[derive(Clone, Copy, Debug, Eq, PartialEq)]
//! enum AppAction {
//!     RunQuery,
//! }
//!
//! let ui = HtmlDocument::parse_fragment(
//!     r#"<button id="run" class="primary" on:click="query.run">Run</button>"#,
//! )
//! .expect("HTML should parse")
//! .with_css(
//!     r#"
//!     .primary {
//!         width: 96px;
//!         height: 32px;
//!         background: rgb(222, 238, 255);
//!     }
//!     "#,
//! )
//! .expect("CSS should parse");
//!
//! let mut view = ui
//!     .to_view(Size::new(320.0, 180.0))
//!     .expect("HTML should create a document view");
//! let registry = DocumentCommandRegistry::new().bind_click("query.run", AppAction::RunQuery);
//! let frame = view
//!     .update_request()
//!     .input(DocumentInput::primary_click(Point::new(8.0, 8.0)))
//!     .update_actions(&registry)
//!     .expect("HTML-authored commands should map to typed Rust actions");
//!
//! assert_eq!(frame.actions()[0].action(), &AppAction::RunQuery);
//! ```

use des_document::{
    Color, Document, DocumentActionFrame, DocumentActionSurface, DocumentBuilder,
    DocumentCommandRegistry, DocumentInput, DocumentOutput, DocumentProjection,
    DocumentProjectionReport, DocumentView, Element, ElementBehaviorEvent, ElementBehaviorHook,
    ElementSpec, FontStretch, FontStyle, FontWeight, Glyph, InlineTextStyle, Size, StyleSheet,
    TableCellSpec, TableColumnSpec, TableSpec, TableTrackSize, TextContent, TextDecoration,
    TextRun, TextTransform, TextVerticalAlign,
};
use html5ever::tendril::TendrilSink;
use html5ever::{ParseOpts, QualName, local_name, ns, parse_document, parse_fragment};
use markup5ever_rcdom::{Handle, NodeData, RcDom};
#[cfg(debug_assertions)]
use std::collections::hash_map::DefaultHasher;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
#[cfg(debug_assertions)]
use std::hash::{Hash, Hasher};
use std::path::Path;
#[cfg(debug_assertions)]
use std::path::PathBuf;
#[cfg(debug_assertions)]
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
        HtmlNode, HtmlResult, HtmlSet, HtmlStylesheet,
    };
    #[cfg(debug_assertions)]
    pub use crate::{HtmlStylesheetFile, ReloadStatus};
    pub use des_document::prelude::*;
}

/// Convenient result type for HTML operations.
pub type HtmlResult<T> = Result<T, HtmlError>;

/// HTML ingestion, CSS ingestion, and file errors.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlError {
    /// The HTML source could not be mapped into the document contract.
    Parse {
        source: Option<String>,
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
                source,
                offset,
                line,
                column,
                message,
            } => {
                f.write_str("html parse error")?;
                if let Some(source) = source {
                    write!(f, " in {source}")?;
                }
                write!(f, " at {line}:{column} (offset {offset}): {message}")
            }
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum HtmlParseKind {
    Document,
    Fragment,
}

/// Browser-parsed HTML document or fragment.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct HtmlDocument {
    /// Top-level HTML nodes in source order.
    children: Vec<HtmlNode>,
    /// Non-fatal authoring diagnostics collected while mapping HTML semantics.
    diagnostics: Vec<HtmlDiagnostic>,
}

impl HtmlDocument {
    /// Parses an HTML document using HTML5 tree-construction rules.
    pub fn parse(source: &str) -> HtmlResult<Self> {
        Self::parse_with_source(source, "inline HTML", HtmlParseKind::Document)
    }

    fn parse_with_source(
        source: &str,
        source_label: impl Into<String>,
        kind: HtmlParseKind,
    ) -> HtmlResult<Self> {
        let source_label = source_label.into();
        validate_html_source(source, &source_label)?;
        let opts = strict_html_parse_opts();
        let dom = match kind {
            HtmlParseKind::Document => parse_document(RcDom::default(), opts).one(source),
            HtmlParseKind::Fragment => {
                let context = QualName::new(None, ns!(html), local_name!("body"));
                parse_fragment(RcDom::default(), opts, context, Vec::new(), false).one(source)
            }
        };
        let mut diagnostics = Vec::new();
        let children = match kind {
            HtmlParseKind::Document => {
                rcdom_document_children_to_html(&dom.document.children.borrow(), &mut diagnostics)?
            }
            HtmlParseKind::Fragment => {
                rcdom_fragment_children_to_html(&dom.document.children.borrow(), &mut diagnostics)?
            }
        };
        if let Some(diagnostic) = diagnostics.first() {
            return Err(html_parse_error(
                source,
                &source_label,
                0,
                diagnostic.message.clone(),
            ));
        }
        Ok(Self {
            children,
            diagnostics,
        })
    }

    /// Reads and parses an HTML document file using HTML5 tree-construction rules.
    pub fn load(path: impl AsRef<Path>) -> HtmlResult<Self> {
        let path = path.as_ref();
        Self::parse_with_source(
            &fs::read_to_string(path)?,
            path.display().to_string(),
            HtmlParseKind::Document,
        )
    }

    /// Reads and parses an HTML fragment file using a `body` context element.
    pub fn load_fragment(path: impl AsRef<Path>) -> HtmlResult<Self> {
        let path = path.as_ref();
        Self::parse_with_source(
            &fs::read_to_string(path)?,
            path.display().to_string(),
            HtmlParseKind::Fragment,
        )
    }

    /// Parses an HTML fragment using a `body` context element.
    pub fn parse_fragment(source: &str) -> HtmlResult<Self> {
        Self::parse_with_source(source, "inline HTML", HtmlParseKind::Fragment)
    }

    /// Configures the parsed HTML tree and returns it.
    pub fn with(mut self, configure: impl FnOnce(&mut Self)) -> Self {
        configure(&mut self);
        self
    }

    /// Fallibly configures the parsed HTML tree and returns it.
    pub fn try_with<E>(
        mut self,
        configure: impl FnOnce(&mut Self) -> Result<(), E>,
    ) -> Result<Self, E> {
        configure(&mut self)?;
        Ok(self)
    }

    /// Conditionally configures the parsed HTML tree and returns it.
    pub fn when(mut self, present: bool, configure: impl FnOnce(&mut Self)) -> Self {
        if present {
            configure(&mut self);
        }
        self
    }

    /// Conditionally and fallibly configures the parsed HTML tree and returns it.
    pub fn try_when<E>(
        mut self,
        present: bool,
        configure: impl FnOnce(&mut Self) -> Result<(), E>,
    ) -> Result<Self, E> {
        if present {
            configure(&mut self)?;
        }
        Ok(self)
    }

    /// Returns the top-level parsed HTML nodes in source order.
    pub fn children(&self) -> &[HtmlNode] {
        &self.children
    }

    /// Appends the parsed HTML nodes into an existing document builder.
    ///
    /// This lets applications use browser-authored fragments for stable chrome
    /// while continuing to project dynamic Rust-owned regions into the same
    /// retained document.
    pub fn append_to_builder(&self, builder: &mut DocumentBuilder) {
        for (index, child) in self.children.iter().enumerate() {
            child.write_to_document_builder(builder, &[index]);
        }
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

    /// Returns the first parsed node with the supplied HTML id, or an explicit HTML error.
    pub fn require_by_id(&self, id: &str) -> HtmlResult<&HtmlNode> {
        self.find_by_id(id).ok_or_else(|| HtmlError::Parse {
            source: None,
            offset: 0,
            line: 1,
            column: 1,
            message: format!("missing html node with id `{id}`"),
        })
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

    /// Returns Rust behavior hooks declared by HTML nodes in document order.
    pub fn behavior_hooks(&self) -> Vec<&HtmlBehaviorHook> {
        let mut hooks = Vec::new();
        for child in &self.children {
            child.collect_behavior_hooks(&mut hooks);
        }
        hooks
    }

    /// Returns Rust behavior hooks matching a typed document intent.
    pub fn behavior_hooks_for(&self, event: ElementBehaviorEvent) -> Vec<&HtmlBehaviorHook> {
        self.behavior_hooks()
            .into_iter()
            .filter(|hook| hook.matches_intent(event))
            .collect()
    }

    /// Returns the first Rust behavior hook matching a typed document intent.
    pub fn first_behavior_hook_for(
        &self,
        event: ElementBehaviorEvent,
    ) -> Option<&HtmlBehaviorHook> {
        self.behavior_hooks()
            .into_iter()
            .find(|hook| hook.matches_intent(event))
    }

    /// Returns true when a matching behavior hook is declared by the parsed HTML.
    pub fn has_behavior_hook(&self, event: ElementBehaviorEvent, command: &str) -> bool {
        self.behavior_hooks()
            .into_iter()
            .any(|hook| hook.matches_intent(event) && hook.has_command(command))
    }

    /// Returns true when any parsed behavior hook declares the supplied command.
    pub fn has_command_hook(&self, command: &str) -> bool {
        self.behavior_hooks()
            .into_iter()
            .any(|hook| hook.has_command(command))
    }

    /// Pushes typed Rust command bindings for HTML-authored behavior hooks.
    ///
    /// The mapper receives each parsed `on:*` or `data-command` hook in document
    /// order. Returning `Some(action)` binds the hook's declared command to the
    /// parsed document event intent; hooks whose event name is not part of the
    /// document input model are left unbound because they cannot be emitted.
    pub fn push_commands<Action>(
        &self,
        registry: &mut DocumentCommandRegistry<Action>,
        mut action_for: impl FnMut(&HtmlBehaviorHook) -> Option<Action>,
    ) {
        for hook in self.behavior_hooks() {
            let Some(intent) = hook.intent() else {
                continue;
            };
            if let Some(action) = action_for(hook) {
                registry.push_on(intent, hook.command(), action);
            }
        }
    }

    /// Creates typed Rust command bindings for HTML-authored behavior hooks.
    pub fn command_registry<Action>(
        &self,
        action_for: impl FnMut(&HtmlBehaviorHook) -> Option<Action>,
    ) -> DocumentCommandRegistry<Action> {
        let mut registry = DocumentCommandRegistry::new();
        self.push_commands(&mut registry, action_for);
        registry
    }

    /// Pushes typed Rust actions for HTML command names while preserving hook intent.
    ///
    /// Each `(command, action)` pair maps an authored `on:*` or `data-command`
    /// command string to a Rust action. If the command is declared by multiple
    /// event hooks, the action is cloned into each hook's parsed document intent.
    pub fn push_command_actions<Action, Command>(
        &self,
        registry: &mut DocumentCommandRegistry<Action>,
        actions: impl IntoIterator<Item = (Command, Action)>,
    ) where
        Action: Clone,
        Command: AsRef<str>,
    {
        let actions = actions
            .into_iter()
            .map(|(command, action)| (command.as_ref().to_owned(), action))
            .collect::<BTreeMap<_, _>>();
        self.push_commands(registry, |hook| actions.get(hook.command()).cloned());
    }

    /// Creates typed Rust action bindings from `(command, action)` pairs.
    pub fn command_action_registry<Action, Command>(
        &self,
        actions: impl IntoIterator<Item = (Command, Action)>,
    ) -> DocumentCommandRegistry<Action>
    where
        Action: Clone,
        Command: AsRef<str>,
    {
        let mut registry = DocumentCommandRegistry::new();
        self.push_command_actions(&mut registry, actions);
        registry
    }

    /// Pushes typed Rust actions for `(event intent, command)` pairs.
    ///
    /// This is the concise path when one HTML command name is reused by several
    /// event hooks but each event should map to a distinct Rust action.
    pub fn push_command_intent_actions<Action, Command>(
        &self,
        registry: &mut DocumentCommandRegistry<Action>,
        actions: impl IntoIterator<Item = (ElementBehaviorEvent, Command, Action)>,
    ) where
        Action: Clone,
        Command: AsRef<str>,
    {
        let actions = actions
            .into_iter()
            .map(|(intent, command, action)| (intent, command.as_ref().to_owned(), action))
            .collect::<Vec<_>>();
        self.push_commands(registry, |hook| {
            let intent = hook.intent()?;
            actions
                .iter()
                .find(|(action_intent, command, _)| {
                    *action_intent == intent && command == hook.command()
                })
                .map(|(_, _, action)| action.clone())
        });
    }

    /// Creates typed Rust action bindings from `(event intent, command, action)` tuples.
    pub fn command_intent_action_registry<Action, Command>(
        &self,
        actions: impl IntoIterator<Item = (ElementBehaviorEvent, Command, Action)>,
    ) -> DocumentCommandRegistry<Action>
    where
        Action: Clone,
        Command: AsRef<str>,
    {
        let mut registry = DocumentCommandRegistry::new();
        self.push_command_intent_actions(&mut registry, actions);
        registry
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

    /// Conditionally pairs this parsed HTML tree with typed stylesheet rules.
    pub fn with_stylesheet_if(self, stylesheet: StyleSheet, present: bool) -> HtmlStylesheet {
        if present {
            self.with_stylesheet(stylesheet)
        } else {
            self.with_stylesheet(StyleSheet::new())
        }
    }

    /// Parses CSS and pairs it with this parsed HTML tree.
    pub fn with_css(self, css: &str) -> HtmlResult<HtmlStylesheet> {
        Ok(self.with_stylesheet(parse_stylesheet(css, "inline CSS")?))
    }

    /// Conditionally parses CSS and pairs it with this parsed HTML tree.
    pub fn with_css_if(self, present: bool, css: &str) -> HtmlResult<HtmlStylesheet> {
        if present {
            self.with_css(css)
        } else {
            Ok(self.with_stylesheet(StyleSheet::new()))
        }
    }

    /// Creates a ready-to-update document view from this HTML tree and stylesheet.
    pub fn to_view_with_stylesheet(
        &self,
        viewport: Size,
        stylesheet: StyleSheet,
    ) -> HtmlResult<DocumentView> {
        Ok(DocumentView::new(self.to_document(viewport)?, stylesheet))
    }

    /// Conditionally creates a view with typed stylesheet rules.
    pub fn to_view_with_stylesheet_if(
        &self,
        viewport: Size,
        stylesheet: StyleSheet,
        present: bool,
    ) -> HtmlResult<DocumentView> {
        if present {
            self.to_view_with_stylesheet(viewport, stylesheet)
        } else {
            self.to_view(viewport)
        }
    }

    /// Parses CSS and creates a ready-to-update document view from this HTML tree.
    pub fn to_view_with_css(&self, viewport: Size, css: &str) -> HtmlResult<DocumentView> {
        self.to_view_with_stylesheet(viewport, parse_stylesheet(css, "inline CSS")?)
    }

    /// Conditionally parses CSS and creates a ready-to-update document view.
    pub fn to_view_with_css_if(
        &self,
        viewport: Size,
        present: bool,
        css: &str,
    ) -> HtmlResult<DocumentView> {
        if present {
            self.to_view_with_css(viewport, css)
        } else {
            self.to_view(viewport)
        }
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

    /// Conditionally creates an action surface with typed stylesheet rules.
    pub fn to_action_surface_with_stylesheet_if<Action>(
        &self,
        viewport: Size,
        stylesheet: StyleSheet,
        present: bool,
        commands: DocumentCommandRegistry<Action>,
    ) -> HtmlResult<DocumentActionSurface<Action>> {
        Ok(self
            .to_view_with_stylesheet_if(viewport, stylesheet, present)?
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

    /// Conditionally creates a styled action surface and configures commands in place.
    pub fn to_action_surface_with_stylesheet_if_and<Action>(
        &self,
        viewport: Size,
        stylesheet: StyleSheet,
        present: bool,
        configure: impl FnOnce(&mut DocumentCommandRegistry<Action>),
    ) -> HtmlResult<DocumentActionSurface<Action>> {
        Ok(self
            .to_view_with_stylesheet_if(viewport, stylesheet, present)?
            .action_surface_with(configure))
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

    /// Creates an action surface, applies retained state projection, and returns both.
    pub fn to_action_surface_with_projection<Action>(
        &self,
        viewport: Size,
        projection: &DocumentProjection,
        commands: DocumentCommandRegistry<Action>,
    ) -> HtmlResult<(DocumentProjectionReport, DocumentActionSurface<Action>)> {
        let mut surface = self.to_action_surface(viewport, commands)?;
        let report = surface.project(projection)?;
        Ok((report, surface))
    }

    /// Creates an action surface, builds retained state projection in place, and applies it.
    pub fn to_action_surface_projected_with<Action>(
        &self,
        viewport: Size,
        project: impl FnOnce(&mut DocumentProjection),
        commands: DocumentCommandRegistry<Action>,
    ) -> HtmlResult<(DocumentProjectionReport, DocumentActionSurface<Action>)> {
        let mut projection = DocumentProjection::new();
        project(&mut projection);
        self.to_action_surface_with_projection(viewport, &projection, commands)
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

    /// Resolves this HTML tree and configures typed Rust actions in one hook.
    pub fn update_actions_with<Action>(
        &self,
        viewport: Size,
        configure: impl FnOnce(&mut DocumentCommandRegistry<Action>),
    ) -> HtmlResult<DocumentActionFrame<Action>>
    where
        Action: Clone,
    {
        let mut registry = DocumentCommandRegistry::new();
        configure(&mut registry);
        self.update_actions(viewport, &registry)
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

    /// Routes input through this HTML tree and configures typed Rust actions in one hook.
    pub fn update_with_input_actions_with<Action>(
        &self,
        viewport: Size,
        input: DocumentInput,
        configure: impl FnOnce(&mut DocumentCommandRegistry<Action>),
    ) -> HtmlResult<DocumentActionFrame<Action>>
    where
        Action: Clone,
    {
        let mut registry = DocumentCommandRegistry::new();
        configure(&mut registry);
        self.update_with_input_actions(viewport, input, &registry)
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

    /// Conditionally parses CSS and creates an action surface with typed Rust commands.
    pub fn to_action_surface_with_css_if<Action>(
        &self,
        viewport: Size,
        present: bool,
        css: &str,
        configure: impl FnOnce(&mut DocumentCommandRegistry<Action>),
    ) -> HtmlResult<DocumentActionSurface<Action>> {
        Ok(self
            .to_view_with_css_if(viewport, present, css)?
            .action_surface_with(configure))
    }

    /// Parses CSS, resolves this HTML tree, and returns the first output frame.
    pub fn update_with_css(&self, viewport: Size, css: &str) -> HtmlResult<DocumentOutput> {
        self.to_view_with_css(viewport, css)
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

    /// Parses CSS, resolves this HTML tree, and configures typed Rust actions in one hook.
    pub fn update_actions_with_css_and<Action>(
        &self,
        viewport: Size,
        css: &str,
        configure: impl FnOnce(&mut DocumentCommandRegistry<Action>),
    ) -> HtmlResult<DocumentActionFrame<Action>>
    where
        Action: Clone,
    {
        let mut registry = DocumentCommandRegistry::new();
        configure(&mut registry);
        self.update_actions_with_css(viewport, css, &registry)
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

    /// Parses CSS, routes input, and configures typed Rust actions in one hook.
    pub fn update_with_input_actions_and_css_with<Action>(
        &self,
        viewport: Size,
        input: DocumentInput,
        css: &str,
        configure: impl FnOnce(&mut DocumentCommandRegistry<Action>),
    ) -> HtmlResult<DocumentActionFrame<Action>>
    where
        Action: Clone,
    {
        let mut registry = DocumentCommandRegistry::new();
        configure(&mut registry);
        self.update_with_input_actions_and_css(viewport, input, css, &registry)
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

    /// Configures this parsed HTML/CSS bundle and returns it.
    pub fn with(mut self, configure: impl FnOnce(&mut Self)) -> Self {
        configure(&mut self);
        self
    }

    /// Fallibly configures this parsed HTML/CSS bundle and returns it.
    pub fn try_with<E>(
        mut self,
        configure: impl FnOnce(&mut Self) -> Result<(), E>,
    ) -> Result<Self, E> {
        configure(&mut self)?;
        Ok(self)
    }

    /// Conditionally configures this parsed HTML/CSS bundle and returns it.
    pub fn when(mut self, present: bool, configure: impl FnOnce(&mut Self)) -> Self {
        if present {
            configure(&mut self);
        }
        self
    }

    /// Conditionally and fallibly configures this parsed HTML/CSS bundle and returns it.
    pub fn try_when<E>(
        mut self,
        present: bool,
        configure: impl FnOnce(&mut Self) -> Result<(), E>,
    ) -> Result<Self, E> {
        if present {
            configure(&mut self)?;
        }
        Ok(self)
    }

    /// Returns the browser-parsed HTML document or fragment.
    pub fn html(&self) -> &HtmlDocument {
        &self.html
    }

    /// Returns the parsed document stylesheet.
    pub fn stylesheet(&self) -> &StyleSheet {
        &self.stylesheet
    }

    /// Returns the parsed document stylesheet for controlled extension.
    pub fn stylesheet_mut(&mut self) -> &mut StyleSheet {
        &mut self.stylesheet
    }

    /// Replaces the parsed document stylesheet.
    pub fn replace_stylesheet(&mut self, stylesheet: StyleSheet) {
        self.stylesheet = stylesheet;
    }

    /// Extends the parsed stylesheet with typed rules.
    pub fn extend_stylesheet(&mut self, stylesheet: StyleSheet) -> &mut Self {
        self.stylesheet.extend(stylesheet);
        self
    }

    /// Extends the parsed stylesheet with typed rules and returns the bundle.
    pub fn with_stylesheet(mut self, stylesheet: StyleSheet) -> Self {
        self.extend_stylesheet(stylesheet);
        self
    }

    /// Conditionally extends the parsed stylesheet with typed rules.
    pub fn extend_stylesheet_if(&mut self, stylesheet: StyleSheet, present: bool) -> &mut Self {
        self.stylesheet.extend_if(stylesheet, present);
        self
    }

    /// Conditionally extends the parsed stylesheet and returns the bundle.
    pub fn with_stylesheet_if(mut self, stylesheet: StyleSheet, present: bool) -> Self {
        self.extend_stylesheet_if(stylesheet, present);
        self
    }

    /// Parses strict CSS into the parsed stylesheet.
    pub fn extend_css(&mut self, css: &str) -> HtmlResult<&mut Self> {
        self.stylesheet.extend(parse_stylesheet(css, "inline CSS")?);
        Ok(self)
    }

    /// Parses strict CSS into the parsed stylesheet and returns the bundle.
    pub fn with_css(mut self, css: &str) -> HtmlResult<Self> {
        self.extend_css(css)?;
        Ok(self)
    }

    /// Conditionally parses strict CSS into the parsed stylesheet.
    pub fn extend_css_if(&mut self, present: bool, css: &str) -> HtmlResult<&mut Self> {
        if present {
            self.extend_css(css)?;
        }
        Ok(self)
    }

    /// Conditionally parses strict CSS into the parsed stylesheet and returns the bundle.
    pub fn with_css_if(mut self, present: bool, css: &str) -> HtmlResult<Self> {
        self.extend_css_if(present, css)?;
        Ok(self)
    }

    /// Finds the first parsed HTML node with the supplied id.
    pub fn find_by_id(&self, id: &str) -> Option<&HtmlNode> {
        self.html.find_by_id(id)
    }

    /// Returns the first parsed HTML node with the supplied id, or an explicit HTML error.
    pub fn require_by_id(&self, id: &str) -> HtmlResult<&HtmlNode> {
        self.html.require_by_id(id)
    }

    /// Finds the first parsed HTML node with the supplied tag name.
    pub fn first_by_tag(&self, tag: &str) -> Option<&HtmlNode> {
        self.html.first_by_tag(tag)
    }

    /// Returns parsed HTML nodes with the supplied tag name in document order.
    pub fn nodes_by_tag(&self, tag: &str) -> Vec<&HtmlNode> {
        self.html.nodes_by_tag(tag)
    }

    /// Returns parsed HTML nodes with the supplied class in document order.
    pub fn nodes_with_class(&self, class: &str) -> Vec<&HtmlNode> {
        self.html.nodes_with_class(class)
    }

    /// Returns Rust behavior hooks declared by the parsed HTML in document order.
    pub fn behavior_hooks(&self) -> Vec<&HtmlBehaviorHook> {
        self.html.behavior_hooks()
    }

    /// Returns Rust behavior hooks matching a typed document intent.
    pub fn behavior_hooks_for(&self, event: ElementBehaviorEvent) -> Vec<&HtmlBehaviorHook> {
        self.html.behavior_hooks_for(event)
    }

    /// Returns the first Rust behavior hook matching a typed document intent.
    pub fn first_behavior_hook_for(
        &self,
        event: ElementBehaviorEvent,
    ) -> Option<&HtmlBehaviorHook> {
        self.html.first_behavior_hook_for(event)
    }

    /// Returns true when a matching behavior hook is declared by the parsed HTML.
    pub fn has_behavior_hook(&self, event: ElementBehaviorEvent, command: &str) -> bool {
        self.html.has_behavior_hook(event, command)
    }

    /// Returns true when any parsed behavior hook declares the supplied command.
    pub fn has_command_hook(&self, command: &str) -> bool {
        self.html.has_command_hook(command)
    }

    /// Pushes typed Rust command bindings for HTML-authored behavior hooks.
    pub fn push_commands<Action>(
        &self,
        registry: &mut DocumentCommandRegistry<Action>,
        action_for: impl FnMut(&HtmlBehaviorHook) -> Option<Action>,
    ) {
        self.html.push_commands(registry, action_for);
    }

    /// Creates typed Rust command bindings for HTML-authored behavior hooks.
    pub fn command_registry<Action>(
        &self,
        action_for: impl FnMut(&HtmlBehaviorHook) -> Option<Action>,
    ) -> DocumentCommandRegistry<Action> {
        self.html.command_registry(action_for)
    }

    /// Pushes typed Rust actions for HTML command names while preserving hook intent.
    pub fn push_command_actions<Action, Command>(
        &self,
        registry: &mut DocumentCommandRegistry<Action>,
        actions: impl IntoIterator<Item = (Command, Action)>,
    ) where
        Action: Clone,
        Command: AsRef<str>,
    {
        self.html.push_command_actions(registry, actions);
    }

    /// Creates typed Rust action bindings from `(command, action)` pairs.
    pub fn command_action_registry<Action, Command>(
        &self,
        actions: impl IntoIterator<Item = (Command, Action)>,
    ) -> DocumentCommandRegistry<Action>
    where
        Action: Clone,
        Command: AsRef<str>,
    {
        self.html.command_action_registry(actions)
    }

    /// Pushes typed Rust actions for `(event intent, command)` pairs.
    pub fn push_command_intent_actions<Action, Command>(
        &self,
        registry: &mut DocumentCommandRegistry<Action>,
        actions: impl IntoIterator<Item = (ElementBehaviorEvent, Command, Action)>,
    ) where
        Action: Clone,
        Command: AsRef<str>,
    {
        self.html.push_command_intent_actions(registry, actions);
    }

    /// Creates typed Rust action bindings from `(event intent, command, action)` tuples.
    pub fn command_intent_action_registry<Action, Command>(
        &self,
        actions: impl IntoIterator<Item = (ElementBehaviorEvent, Command, Action)>,
    ) -> DocumentCommandRegistry<Action>
    where
        Action: Clone,
        Command: AsRef<str>,
    {
        self.html.command_intent_action_registry(actions)
    }

    /// Parses an HTML document and CSS stylesheet into typed document inputs.
    pub fn parse(html: &str, css: &str) -> HtmlResult<Self> {
        let stylesheet = parse_stylesheet(css, "inline CSS")?;
        Ok(Self::new(HtmlDocument::parse(html)?, stylesheet))
    }

    /// Parses an HTML fragment and CSS stylesheet into typed document inputs.
    pub fn parse_fragment(html: &str, css: &str) -> HtmlResult<Self> {
        let stylesheet = parse_stylesheet(css, "inline CSS")?;
        Ok(Self::new(HtmlDocument::parse_fragment(html)?, stylesheet))
    }

    /// Reads HTML and CSS files and parses them into typed document inputs.
    pub fn load_files(html_path: impl AsRef<Path>, css_path: impl AsRef<Path>) -> HtmlResult<Self> {
        let css_path = css_path.as_ref();
        let css = fs::read_to_string(css_path)?;
        Ok(Self::new(
            HtmlDocument::load(html_path)?,
            parse_stylesheet(&css, css_path.display().to_string())?,
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

    /// Creates an action surface, applies retained state projection, and returns both.
    pub fn to_action_surface_with_projection<Action>(
        &self,
        viewport: Size,
        projection: &DocumentProjection,
        commands: DocumentCommandRegistry<Action>,
    ) -> HtmlResult<(DocumentProjectionReport, DocumentActionSurface<Action>)> {
        let mut surface = self.to_action_surface(viewport, commands)?;
        let report = surface.project(projection)?;
        Ok((report, surface))
    }

    /// Creates an action surface, builds retained state projection in place, and applies it.
    pub fn to_action_surface_projected_with<Action>(
        &self,
        viewport: Size,
        project: impl FnOnce(&mut DocumentProjection),
        commands: DocumentCommandRegistry<Action>,
    ) -> HtmlResult<(DocumentProjectionReport, DocumentActionSurface<Action>)> {
        let mut projection = DocumentProjection::new();
        project(&mut projection);
        self.to_action_surface_with_projection(viewport, &projection, commands)
    }

    /// Creates an action surface, applies projection, and configures typed commands in one hook.
    pub fn to_action_surface_with_projection_and<Action>(
        &self,
        viewport: Size,
        projection: &DocumentProjection,
        configure: impl FnOnce(&mut DocumentCommandRegistry<Action>),
    ) -> HtmlResult<(DocumentProjectionReport, DocumentActionSurface<Action>)> {
        let mut surface = self.to_action_surface_with(viewport, configure)?;
        let report = surface.project(projection)?;
        Ok((report, surface))
    }

    /// Creates an action surface, builds projection, and configures typed commands in one hook.
    pub fn to_action_surface_projected_with_and<Action>(
        &self,
        viewport: Size,
        project: impl FnOnce(&mut DocumentProjection),
        configure: impl FnOnce(&mut DocumentCommandRegistry<Action>),
    ) -> HtmlResult<(DocumentProjectionReport, DocumentActionSurface<Action>)> {
        let mut projection = DocumentProjection::new();
        project(&mut projection);
        self.to_action_surface_with_projection_and(viewport, &projection, configure)
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

    /// Creates a view, resolves it, and configures typed Rust actions in one hook.
    pub fn update_actions_with<Action>(
        &self,
        viewport: Size,
        configure: impl FnOnce(&mut DocumentCommandRegistry<Action>),
    ) -> HtmlResult<DocumentActionFrame<Action>>
    where
        Action: Clone,
    {
        let mut registry = DocumentCommandRegistry::new();
        configure(&mut registry);
        self.update_actions(viewport, &registry)
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

    /// Creates a view, routes input, and configures typed Rust actions in one hook.
    pub fn update_with_input_actions_with<Action>(
        &self,
        viewport: Size,
        input: DocumentInput,
        configure: impl FnOnce(&mut DocumentCommandRegistry<Action>),
    ) -> HtmlResult<DocumentActionFrame<Action>>
    where
        Action: Clone,
    {
        let mut registry = DocumentCommandRegistry::new();
        configure(&mut registry);
        self.update_with_input_actions(viewport, input, &registry)
    }
}

/// A parsed HTML element or text node.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlNode {
    /// Element tag name, or `#text` for explicit text nodes.
    tag: String,
    /// Element id from the `id` attribute, when present.
    id: Option<String>,
    /// Class names from the `class` attribute.
    classes: Vec<String>,
    /// Element role from the `role` attribute, when present.
    role: Option<String>,
    /// Non-id/class/role attributes.
    attributes: BTreeMap<String, String>,
    /// Rust behavior hooks declared through `on:*` or `data-command` attributes.
    behavior_hooks: Vec<HtmlBehaviorHook>,
    /// Text content when this is a text node or a text-only element.
    text: Option<String>,
    /// Child nodes.
    children: Vec<HtmlNode>,
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
            text: Some(sanitize_author_string_lossy(text.into())),
            children: Vec::new(),
        }
    }

    /// Returns true when this node represents parsed text.
    pub fn is_text(&self) -> bool {
        self.tag == "#text"
    }

    fn is_whitespace_text_node(&self) -> bool {
        self.is_text() && self.text.as_ref().is_none_or(|text| text.trim().is_empty())
    }

    /// Returns the parsed tag name, or `#text` for explicit text nodes.
    pub fn tag(&self) -> &str {
        &self.tag
    }

    /// Returns the parsed HTML id when present.
    pub fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }

    /// Returns parsed class names in source order.
    pub fn classes(&self) -> &[String] {
        &self.classes
    }

    /// Returns the parsed role when present.
    pub fn role(&self) -> Option<&str> {
        self.role.as_deref()
    }

    /// Returns parsed non-id/class/role attributes.
    pub fn attributes(&self) -> &BTreeMap<String, String> {
        &self.attributes
    }

    /// Returns parsed text content when present.
    pub fn text(&self) -> Option<&str> {
        self.text.as_deref()
    }

    /// Returns parsed child nodes in source order.
    pub fn children(&self) -> &[HtmlNode] {
        &self.children
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

    /// Returns the first node in this subtree with the supplied HTML id, or an explicit HTML error.
    pub fn require_by_id(&self, id: &str) -> HtmlResult<&HtmlNode> {
        self.find_by_id(id).ok_or_else(|| HtmlError::Parse {
            source: None,
            offset: 0,
            line: 1,
            column: 1,
            message: format!("missing html node with id `{id}`"),
        })
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

    /// Returns Rust behavior hooks declared in this subtree in document order.
    pub fn behavior_hooks(&self) -> Vec<&HtmlBehaviorHook> {
        let mut hooks = Vec::new();
        self.collect_behavior_hooks(&mut hooks);
        hooks
    }

    /// Returns Rust behavior hooks in this subtree matching a typed document intent.
    pub fn behavior_hooks_for(&self, event: ElementBehaviorEvent) -> Vec<&HtmlBehaviorHook> {
        self.behavior_hooks()
            .into_iter()
            .filter(|hook| hook.matches_intent(event))
            .collect()
    }

    /// Returns the first Rust behavior hook matching a typed document intent.
    pub fn first_behavior_hook_for(
        &self,
        event: ElementBehaviorEvent,
    ) -> Option<&HtmlBehaviorHook> {
        self.behavior_hooks()
            .into_iter()
            .find(|hook| hook.matches_intent(event))
    }

    /// Returns true when this subtree declares a matching behavior hook.
    pub fn has_behavior_hook(&self, event: ElementBehaviorEvent, command: &str) -> bool {
        self.behavior_hooks()
            .into_iter()
            .any(|hook| hook.matches_intent(event) && hook.has_command(command))
    }

    /// Returns true when this subtree declares the supplied command hook.
    pub fn has_command_hook(&self, command: &str) -> bool {
        self.behavior_hooks()
            .into_iter()
            .any(|hook| hook.has_command(command))
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

    fn collect_behavior_hooks<'a>(&'a self, hooks: &mut Vec<&'a HtmlBehaviorHook>) {
        hooks.extend(self.behavior_hooks.iter());
        for child in &self.children {
            child.collect_behavior_hooks(hooks);
        }
    }

    fn write_to_document_builder(&self, builder: &mut DocumentBuilder, path: &[usize]) {
        if self.is_text() {
            if let Some(text) = self.text.as_ref().filter(|text| !text.trim().is_empty()) {
                builder.text(stable_text_id(path), TextContent::plain(text.clone()));
            }
            return;
        }

        let id = self
            .id
            .clone()
            .unwrap_or_else(|| stable_element_id(&self.tag, path));
        let mut spec = ElementSpec::new(element_for_node(self)).classes(self.classes.clone());
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
        if let Some(glyph) = self
            .attributes
            .get("data-glyph")
            .map(|value| parse_glyph(value))
            .transpose()
            .expect("HTML glyph metadata should be validated while parsing")
        {
            spec = spec.glyph(glyph);
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
        if html_boolean_attribute(&self.attributes, "data-selectable-text") {
            spec = spec.selectable_text();
        }
        if self.attributes.contains_key("data-copyable-text") {
            spec = spec.copyable_text(html_boolean_attribute(
                &self.attributes,
                "data-copyable-text",
            ));
        }
        if html_boolean_attribute(&self.attributes, "data-interactive") {
            spec = spec.interactive();
        }
        if !self.behavior_hooks.is_empty()
            || self.attributes.contains_key("data-command")
            || self.attributes.keys().any(|name| name.starts_with("on:"))
        {
            spec = spec.interactive();
        }
        if let Some(table) = parse_table_spec_attributes(&self.attributes)
            .expect("HTML table metadata should be validated while parsing")
        {
            spec = spec.table(table);
        }
        if let Some(column) = self.attributes.get("data-column") {
            spec = spec.table_cell(TableCellSpec::new(column.trim().to_owned()));
        }

        if html_boolean_attribute(&self.attributes, "data-rich-text") {
            builder.text_element(id, spec, self.rich_text_content());
            return;
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

    fn rich_text_content(&self) -> TextContent {
        let mut runs = Vec::new();
        let base_style = inline_style_from_style_attr(self.attributes.get("style"));
        if let Some(text) = &self.text {
            push_text_run(&mut runs, text, base_style.clone());
        }
        for child in &self.children {
            child.push_rich_text_runs(&mut runs, base_style.clone());
        }
        TextContent::new(runs)
    }

    fn push_rich_text_runs(&self, runs: &mut Vec<TextRun>, inherited: InlineTextStyle) {
        if self.is_text() {
            if let Some(text) = &self.text {
                push_text_run(runs, text, inherited);
            }
            return;
        }

        let mut style = inherited;
        apply_inline_tag_style(&mut style, &self.tag);
        apply_inline_style(
            &mut style,
            inline_style_from_style_attr(self.attributes.get("style")),
        );

        if let Some(text) = &self.text {
            push_text_run(runs, text, style);
            return;
        }
        for child in &self.children {
            child.push_rich_text_runs(runs, style.clone());
        }
    }
}

/// Rust behavior declared from HTML attributes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlBehaviorHook {
    /// Event name such as `click`, `input`, or `submit`.
    event: String,
    /// Rust command/event intent declared by the author.
    command: String,
}

impl HtmlBehaviorHook {
    /// Creates a behavior hook from an HTML-authored event name.
    pub fn new(event: impl Into<String>, command: impl Into<String>) -> Self {
        Self {
            event: sanitize_author_string_lossy(event.into()).trim().to_owned(),
            command: sanitize_author_string_lossy(command.into())
                .trim()
                .to_owned(),
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

    /// Returns true when this hook declares the supplied command.
    pub fn has_command(&self, command: &str) -> bool {
        self.command == command.trim()
    }

    /// Returns the parsed typed event intent when this hook maps to a document intent.
    pub fn intent(&self) -> Option<ElementBehaviorEvent> {
        ElementBehaviorEvent::from_name(&self.event)
    }

    /// Returns true when this hook maps to the supplied document behavior intent.
    pub fn matches_intent(&self, intent: ElementBehaviorEvent) -> bool {
        self.intent() == Some(intent)
    }

    /// Returns true when this hook maps to click intent.
    pub fn is_click(&self) -> bool {
        self.matches_intent(ElementBehaviorEvent::Click)
    }

    /// Returns true when this hook maps to context-menu intent.
    pub fn is_context_menu(&self) -> bool {
        self.matches_intent(ElementBehaviorEvent::ContextMenu)
    }

    /// Returns true when this hook maps to pointer-enter intent.
    pub fn is_pointer_enter(&self) -> bool {
        self.matches_intent(ElementBehaviorEvent::PointerEnter)
    }

    /// Returns true when this hook maps to pointer-leave intent.
    pub fn is_pointer_leave(&self) -> bool {
        self.matches_intent(ElementBehaviorEvent::PointerLeave)
    }

    /// Returns true when this hook maps to pointer-down intent.
    pub fn is_pointer_down(&self) -> bool {
        self.matches_intent(ElementBehaviorEvent::PointerDown)
    }

    /// Returns true when this hook maps to pointer-up intent.
    pub fn is_pointer_up(&self) -> bool {
        self.matches_intent(ElementBehaviorEvent::PointerUp)
    }

    /// Returns true when this hook maps to drag-start intent.
    pub fn is_drag_start(&self) -> bool {
        self.matches_intent(ElementBehaviorEvent::DragStart)
    }

    /// Returns true when this hook maps to drag intent.
    pub fn is_drag(&self) -> bool {
        self.matches_intent(ElementBehaviorEvent::Drag)
    }

    /// Returns true when this hook maps to drag-end intent.
    pub fn is_drag_end(&self) -> bool {
        self.matches_intent(ElementBehaviorEvent::DragEnd)
    }

    /// Returns true when this hook maps to any pointer drag intent.
    pub fn is_any_drag(&self) -> bool {
        self.is_drag_start() || self.is_drag() || self.is_drag_end()
    }

    /// Returns true when this hook maps to scroll intent.
    pub fn is_scroll(&self) -> bool {
        self.matches_intent(ElementBehaviorEvent::Scroll)
    }

    /// Returns true when this hook maps to focus intent.
    pub fn is_focus(&self) -> bool {
        self.matches_intent(ElementBehaviorEvent::Focus)
    }

    /// Returns true when this hook maps to blur intent.
    pub fn is_blur(&self) -> bool {
        self.matches_intent(ElementBehaviorEvent::Blur)
    }

    /// Returns true when this hook maps to text-selection intent.
    pub fn is_select(&self) -> bool {
        self.matches_intent(ElementBehaviorEvent::Select)
    }

    /// Returns true when this hook maps to key-down intent.
    pub fn is_key_down(&self) -> bool {
        self.matches_intent(ElementBehaviorEvent::KeyDown)
    }

    /// Returns true when this hook maps to key-up intent.
    pub fn is_key_up(&self) -> bool {
        self.matches_intent(ElementBehaviorEvent::KeyUp)
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
#[cfg(debug_assertions)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReloadStatus {
    /// True when the file changed and the HTML document was reloaded.
    pub changed: bool,
}

/// File-backed browser HTML document for polling-style hot reload.
#[derive(Clone, Debug)]
pub struct HtmlFile {
    #[cfg(debug_assertions)]
    path: PathBuf,
    #[cfg(debug_assertions)]
    modified: Option<SystemTime>,
    #[cfg(debug_assertions)]
    fingerprint: HtmlFingerprint,
    document: HtmlDocument,
}

impl HtmlFile {
    /// Loads and parses an HTML file.
    pub fn load(path: impl AsRef<Path>) -> HtmlResult<Self> {
        let path = path.as_ref().to_path_buf();
        let source = fs::read_to_string(&path)?;
        #[cfg(debug_assertions)]
        let metadata = fs::metadata(&path)?;
        #[cfg(debug_assertions)]
        let fingerprint = HtmlFingerprint::new(&source);
        let document = HtmlDocument::parse_with_source(
            &source,
            path.display().to_string(),
            HtmlParseKind::Document,
        )?;
        Ok(Self {
            #[cfg(debug_assertions)]
            path,
            #[cfg(debug_assertions)]
            modified: metadata.modified().ok(),
            #[cfg(debug_assertions)]
            fingerprint,
            document,
        })
    }

    /// Returns the current parsed document.
    pub fn document(&self) -> &HtmlDocument {
        &self.document
    }

    /// Re-reads and reparses the HTML document if the file changed.
    #[cfg(debug_assertions)]
    pub fn reload_if_changed(&mut self) -> HtmlResult<ReloadStatus> {
        let source = fs::read_to_string(&self.path)?;
        let metadata = fs::metadata(&self.path)?;
        let modified = metadata.modified().ok();
        let fingerprint = HtmlFingerprint::new(&source);
        if modified == self.modified && fingerprint == self.fingerprint {
            return Ok(ReloadStatus { changed: false });
        }

        self.document = HtmlDocument::parse_with_source(
            &source,
            self.path.display().to_string(),
            HtmlParseKind::Document,
        )?;
        self.modified = modified;
        self.fingerprint = fingerprint;
        Ok(ReloadStatus { changed: true })
    }
}

/// Dev-only file-backed HTML/CSS bundle for polling-style hot reload.
#[cfg(debug_assertions)]
#[derive(Clone, Debug)]
pub struct HtmlStylesheetFile {
    html_path: PathBuf,
    css_path: PathBuf,
    html_modified: Option<SystemTime>,
    css_modified: Option<SystemTime>,
    html_fingerprint: HtmlFingerprint,
    css_fingerprint: HtmlFingerprint,
    bundle: HtmlStylesheet,
}

#[cfg(debug_assertions)]
impl HtmlStylesheetFile {
    /// Loads and parses an HTML file plus companion CSS file.
    pub fn load(html_path: impl AsRef<Path>, css_path: impl AsRef<Path>) -> HtmlResult<Self> {
        let html_path = html_path.as_ref().to_path_buf();
        let css_path = css_path.as_ref().to_path_buf();
        let html_source = fs::read_to_string(&html_path)?;
        let css_source = fs::read_to_string(&css_path)?;
        let html_metadata = fs::metadata(&html_path)?;
        let css_metadata = fs::metadata(&css_path)?;
        let html_fingerprint = HtmlFingerprint::new(&html_source);
        let css_fingerprint = HtmlFingerprint::new(&css_source);
        let html = HtmlDocument::parse_with_source(
            &html_source,
            html_path.display().to_string(),
            HtmlParseKind::Document,
        )?;
        let stylesheet = parse_stylesheet(&css_source, css_path.display().to_string())?;
        Ok(Self {
            html_path,
            css_path,
            html_modified: html_metadata.modified().ok(),
            css_modified: css_metadata.modified().ok(),
            html_fingerprint,
            css_fingerprint,
            bundle: HtmlStylesheet::new(html, stylesheet),
        })
    }

    /// Returns the current parsed HTML/CSS bundle.
    pub fn bundle(&self) -> &HtmlStylesheet {
        &self.bundle
    }

    /// Re-reads and reparses the HTML/CSS bundle if either file changed.
    pub fn reload_if_changed(&mut self) -> HtmlResult<ReloadStatus> {
        let html_source = fs::read_to_string(&self.html_path)?;
        let css_source = fs::read_to_string(&self.css_path)?;
        let html_metadata = fs::metadata(&self.html_path)?;
        let css_metadata = fs::metadata(&self.css_path)?;
        let html_modified = html_metadata.modified().ok();
        let css_modified = css_metadata.modified().ok();
        let html_fingerprint = HtmlFingerprint::new(&html_source);
        let css_fingerprint = HtmlFingerprint::new(&css_source);
        if html_modified == self.html_modified
            && css_modified == self.css_modified
            && html_fingerprint == self.html_fingerprint
            && css_fingerprint == self.css_fingerprint
        {
            return Ok(ReloadStatus { changed: false });
        }

        let html = HtmlDocument::parse_with_source(
            &html_source,
            self.html_path.display().to_string(),
            HtmlParseKind::Document,
        )?;
        let stylesheet = parse_stylesheet(&css_source, self.css_path.display().to_string())?;
        self.bundle = HtmlStylesheet::new(html, stylesheet);
        self.html_modified = html_modified;
        self.css_modified = css_modified;
        self.html_fingerprint = html_fingerprint;
        self.css_fingerprint = css_fingerprint;
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
                source: None,
                offset: 0,
                line: 1,
                column: 1,
                message: format!("missing html document `{name}`"),
            })
    }

    /// Pushes typed Rust actions for a named document's HTML command hooks.
    pub fn push_command_actions<Action, Command>(
        &self,
        name: &str,
        registry: &mut DocumentCommandRegistry<Action>,
        actions: impl IntoIterator<Item = (Command, Action)>,
    ) -> HtmlResult<()>
    where
        Action: Clone,
        Command: AsRef<str>,
    {
        self.get(name)?.push_command_actions(registry, actions);
        Ok(())
    }

    /// Creates typed Rust action bindings for a named document from `(command, action)` pairs.
    pub fn command_action_registry<Action, Command>(
        &self,
        name: &str,
        actions: impl IntoIterator<Item = (Command, Action)>,
    ) -> HtmlResult<DocumentCommandRegistry<Action>>
    where
        Action: Clone,
        Command: AsRef<str>,
    {
        Ok(self.get(name)?.command_action_registry(actions))
    }

    /// Pushes typed Rust actions for `(event intent, command)` pairs in a named document.
    pub fn push_command_intent_actions<Action, Command>(
        &self,
        name: &str,
        registry: &mut DocumentCommandRegistry<Action>,
        actions: impl IntoIterator<Item = (ElementBehaviorEvent, Command, Action)>,
    ) -> HtmlResult<()>
    where
        Action: Clone,
        Command: AsRef<str>,
    {
        self.get(name)?
            .push_command_intent_actions(registry, actions);
        Ok(())
    }

    /// Creates typed Rust action bindings from `(event intent, command, action)` tuples.
    pub fn command_intent_action_registry<Action, Command>(
        &self,
        name: &str,
        actions: impl IntoIterator<Item = (ElementBehaviorEvent, Command, Action)>,
    ) -> HtmlResult<DocumentCommandRegistry<Action>>
    where
        Action: Clone,
        Command: AsRef<str>,
    {
        Ok(self.get(name)?.command_intent_action_registry(actions))
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

    /// Parses CSS and creates a ready-to-update document view from a named document.
    pub fn to_view_with_css(
        &self,
        name: &str,
        viewport: Size,
        css: &str,
    ) -> HtmlResult<DocumentView> {
        self.get(name)?.to_view_with_css(viewport, css)
    }

    /// Creates a named document view, applies retained state projection, and returns both.
    pub fn to_view_with_projection(
        &self,
        name: &str,
        viewport: Size,
        projection: &DocumentProjection,
    ) -> HtmlResult<(DocumentProjectionReport, DocumentView)> {
        self.get(name)?
            .to_view_with_projection(viewport, projection)
    }

    /// Creates a named document view and applies projection built in place.
    pub fn to_view_projected_with(
        &self,
        name: &str,
        viewport: Size,
        project: impl FnOnce(&mut DocumentProjection),
    ) -> HtmlResult<(DocumentProjectionReport, DocumentView)> {
        self.get(name)?.to_view_projected_with(viewport, project)
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

    /// Creates a named action surface, applies projection, and returns both.
    pub fn to_action_surface_with_projection<Action>(
        &self,
        name: &str,
        viewport: Size,
        projection: &DocumentProjection,
        commands: DocumentCommandRegistry<Action>,
    ) -> HtmlResult<(DocumentProjectionReport, DocumentActionSurface<Action>)> {
        self.get(name)?
            .to_action_surface_with_projection(viewport, projection, commands)
    }

    /// Creates a named action surface, builds projection, and returns both.
    pub fn to_action_surface_projected_with<Action>(
        &self,
        name: &str,
        viewport: Size,
        project: impl FnOnce(&mut DocumentProjection),
        commands: DocumentCommandRegistry<Action>,
    ) -> HtmlResult<(DocumentProjectionReport, DocumentActionSurface<Action>)> {
        self.get(name)?
            .to_action_surface_projected_with(viewport, project, commands)
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

    /// Conditionally creates a named action surface with typed stylesheet rules.
    pub fn to_action_surface_with_stylesheet_if<Action>(
        &self,
        name: &str,
        viewport: Size,
        stylesheet: StyleSheet,
        present: bool,
        commands: DocumentCommandRegistry<Action>,
    ) -> HtmlResult<DocumentActionSurface<Action>> {
        self.get(name)?
            .to_action_surface_with_stylesheet_if(viewport, stylesheet, present, commands)
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

    /// Conditionally creates a named styled action surface and configures commands.
    pub fn to_action_surface_with_stylesheet_if_and<Action>(
        &self,
        name: &str,
        viewport: Size,
        stylesheet: StyleSheet,
        present: bool,
        configure: impl FnOnce(&mut DocumentCommandRegistry<Action>),
    ) -> HtmlResult<DocumentActionSurface<Action>> {
        self.get(name)?
            .to_action_surface_with_stylesheet_if_and(viewport, stylesheet, present, configure)
    }

    /// Parses CSS and creates an action surface for a named document.
    pub fn to_action_surface_with_css<Action>(
        &self,
        name: &str,
        viewport: Size,
        css: &str,
        configure: impl FnOnce(&mut DocumentCommandRegistry<Action>),
    ) -> HtmlResult<DocumentActionSurface<Action>> {
        self.get(name)?
            .to_action_surface_with_css(viewport, css, configure)
    }

    /// Conditionally parses CSS and creates an action surface for a named document.
    pub fn to_action_surface_with_css_if<Action>(
        &self,
        name: &str,
        viewport: Size,
        present: bool,
        css: &str,
        configure: impl FnOnce(&mut DocumentCommandRegistry<Action>),
    ) -> HtmlResult<DocumentActionSurface<Action>> {
        self.get(name)?
            .to_action_surface_with_css_if(viewport, present, css, configure)
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

    /// Applies projection and resolves a named HTML document.
    pub fn update_with_projection(
        &self,
        name: &str,
        viewport: Size,
        projection: &DocumentProjection,
    ) -> HtmlResult<(DocumentProjectionReport, DocumentOutput)> {
        self.get(name)?.update_with_projection(viewport, projection)
    }

    /// Builds projection in place and resolves a named HTML document.
    pub fn update_projected_with(
        &self,
        name: &str,
        viewport: Size,
        project: impl FnOnce(&mut DocumentProjection),
    ) -> HtmlResult<(DocumentProjectionReport, DocumentOutput)> {
        self.get(name)?.update_projected_with(viewport, project)
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

    /// Parses CSS and resolves a named HTML document.
    pub fn update_with_css(
        &self,
        name: &str,
        viewport: Size,
        css: &str,
    ) -> HtmlResult<DocumentOutput> {
        self.get(name)?.update_with_css(viewport, css)
    }

    /// Parses CSS and routes input through a named HTML document.
    pub fn update_with_input_and_css(
        &self,
        name: &str,
        viewport: Size,
        input: DocumentInput,
        css: &str,
    ) -> HtmlResult<DocumentOutput> {
        self.get(name)?
            .update_with_input_and_css(viewport, input, css)
    }

    /// Parses CSS, routes input, and collects typed Rust actions.
    pub fn update_with_input_actions_and_css<Action>(
        &self,
        name: &str,
        viewport: Size,
        input: DocumentInput,
        css: &str,
        registry: &DocumentCommandRegistry<Action>,
    ) -> HtmlResult<DocumentActionFrame<Action>>
    where
        Action: Clone,
    {
        self.get(name)?
            .update_with_input_actions_and_css(viewport, input, css, registry)
    }

    /// Parses CSS, routes input, and configures typed Rust actions in one hook.
    pub fn update_with_input_actions_and_css_with<Action>(
        &self,
        name: &str,
        viewport: Size,
        input: DocumentInput,
        css: &str,
        configure: impl FnOnce(&mut DocumentCommandRegistry<Action>),
    ) -> HtmlResult<DocumentActionFrame<Action>>
    where
        Action: Clone,
    {
        self.get(name)?
            .update_with_input_actions_and_css_with(viewport, input, css, configure)
    }

    /// Re-reads file-backed HTML documents and returns names that changed.
    #[cfg(debug_assertions)]
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

#[cfg(debug_assertions)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct HtmlFingerprint {
    len: usize,
    hash: u64,
}

#[cfg(debug_assertions)]
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

fn parse_stylesheet(css: &str, source: impl Into<String>) -> HtmlResult<StyleSheet> {
    StyleSheet::parse_css(css)
        .map_err(|error| HtmlError::Css(error.with_source_label(source).to_string()))
}

fn strict_html_parse_opts() -> ParseOpts {
    let mut opts = ParseOpts::default();
    opts.tokenizer.exact_errors = true;
    opts.tree_builder.exact_errors = true;
    opts
}

fn validate_html_source(source: &str, source_label: &str) -> HtmlResult<()> {
    for (offset, ch) in source.char_indices() {
        if ch == '\0' {
            return Err(html_parse_error(
                source,
                source_label,
                offset,
                "HTML contains a null character",
            ));
        }
        if ch.is_control() && !matches!(ch, '\t' | '\n' | '\r') {
            return Err(html_parse_error(
                source,
                source_label,
                offset,
                format!(
                    "HTML contains unsupported control character U+{:04X}",
                    ch as u32
                ),
            ));
        }
    }

    let mut stack: Vec<HtmlOpenTag> = Vec::new();
    let mut cursor = 0;
    while let Some(relative) = source[cursor..].find('<') {
        let open = cursor + relative;
        if source[open..].starts_with("<!--") {
            let Some(end) = source[open + 4..].find("-->") else {
                return Err(html_parse_error(
                    source,
                    source_label,
                    open,
                    "HTML comment is missing closing `-->`",
                ));
            };
            cursor = open + 4 + end + 3;
            continue;
        }
        if source[open..].starts_with("<!") || source[open..].starts_with("<?") {
            let Some(end) = source[open..].find('>') else {
                return Err(html_parse_error(
                    source,
                    source_label,
                    open,
                    "HTML declaration is missing closing `>`",
                ));
            };
            cursor = open + end + 1;
            continue;
        }
        let Some(end) = source[open..].find('>').map(|offset| open + offset) else {
            return Err(html_parse_error(
                source,
                source_label,
                open,
                "HTML tag is missing closing `>`",
            ));
        };
        let body = source[open + 1..end].trim();
        if body.is_empty() {
            return Err(html_parse_error(
                source,
                source_label,
                open,
                "HTML tag is missing a name",
            ));
        }
        if let Some(close_name) = body.strip_prefix('/') {
            let close_name = read_html_tag_name(close_name.trim_start());
            if close_name.is_empty() {
                return Err(html_parse_error(
                    source,
                    source_label,
                    open,
                    "HTML closing tag is missing a name",
                ));
            }
            let close_name = close_name.to_ascii_lowercase();
            let Some(open_tag) = stack.pop() else {
                return Err(html_parse_error(
                    source,
                    source_label,
                    open,
                    format!("HTML closing tag `</{close_name}>` has no open element"),
                ));
            };
            if open_tag.name != close_name {
                return Err(html_parse_error(
                    source,
                    source_label,
                    open,
                    format!(
                        "HTML closing tag `</{close_name}>` does not match open `<{}>`",
                        open_tag.name
                    ),
                ));
            }
        } else {
            let tag_name = read_html_tag_name(body).to_ascii_lowercase();
            if tag_name.is_empty() {
                return Err(html_parse_error(
                    source,
                    source_label,
                    open,
                    "HTML tag is missing a name",
                ));
            }
            if tag_name == "script" {
                return Err(html_parse_error(
                    source,
                    source_label,
                    open,
                    "script elements are not allowed in document HTML",
                ));
            }
            let self_closing = body.ends_with('/');
            if !self_closing && !is_html_void_element(&tag_name) {
                stack.push(HtmlOpenTag {
                    name: tag_name,
                    offset: open,
                });
            }
        }
        cursor = end + 1;
    }

    if let Some(open_tag) = stack.pop() {
        return Err(html_parse_error(
            source,
            source_label,
            open_tag.offset,
            format!(
                "HTML element `<{}>` is missing a closing tag",
                open_tag.name
            ),
        ));
    }

    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct HtmlOpenTag {
    name: String,
    offset: usize,
}

fn read_html_tag_name(input: &str) -> &str {
    let end = input
        .char_indices()
        .find_map(|(offset, ch)| {
            if ch.is_whitespace() || matches!(ch, '/' | '>') {
                Some(offset)
            } else {
                None
            }
        })
        .unwrap_or(input.len());
    &input[..end]
}

fn is_html_void_element(name: &str) -> bool {
    matches!(
        name,
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}

fn html_parse_error(
    source: &str,
    source_label: impl Into<String>,
    offset: usize,
    message: impl Into<String>,
) -> HtmlError {
    let (line, column) = html_source_location(source, offset);
    HtmlError::Parse {
        source: Some(source_label.into()),
        offset,
        line,
        column,
        message: message.into(),
    }
}

fn html_source_location(source: &str, target: usize) -> (usize, usize) {
    let mut line = 1;
    let mut column = 1;
    for (offset, ch) in source.char_indices() {
        if offset >= target {
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

fn sanitize_parsed_html_string(
    value: impl Into<String>,
    context: impl AsRef<str>,
) -> HtmlResult<String> {
    let value = value.into();
    for (offset, ch) in value.char_indices() {
        if is_malformed_html_string_char(ch) {
            return Err(HtmlError::Parse {
                source: None,
                offset,
                line: 1,
                column: offset + 1,
                message: format!(
                    "HTML {} contains malformed string character U+{:04X}",
                    context.as_ref(),
                    ch as u32
                ),
            });
        }
    }
    Ok(value)
}

fn sanitize_author_string_lossy(value: String) -> String {
    value
        .chars()
        .filter(|ch| !is_malformed_html_string_char(*ch))
        .collect()
}

fn is_malformed_html_string_char(ch: char) -> bool {
    ch == '\0' || ch == '\u{FFFD}' || (ch.is_control() && !matches!(ch, '\t' | '\n' | '\r'))
}

fn rcdom_children_to_html(
    children: &[Handle],
    diagnostics: &mut Vec<HtmlDiagnostic>,
) -> HtmlResult<Vec<HtmlNode>> {
    let mut nodes = Vec::new();
    for child in children {
        append_rcdom_node(child, &mut nodes, diagnostics)?;
    }
    Ok(nodes)
}

fn rcdom_document_children_to_html(
    children: &[Handle],
    diagnostics: &mut Vec<HtmlDiagnostic>,
) -> HtmlResult<Vec<HtmlNode>> {
    let mut body = None;
    for child in children {
        find_body_handle(child, &mut body);
    }
    if let Some(body) = body {
        rcdom_children_to_html(&body.children.borrow(), diagnostics).map(trim_boundary_whitespace)
    } else {
        rcdom_children_to_html(children, diagnostics).map(trim_boundary_whitespace)
    }
}

fn rcdom_fragment_children_to_html(
    children: &[Handle],
    diagnostics: &mut Vec<HtmlDiagnostic>,
) -> HtmlResult<Vec<HtmlNode>> {
    let mut nodes = trim_boundary_whitespace(rcdom_children_to_html(children, diagnostics)?);
    loop {
        if nodes.len() != 1 {
            return Ok(nodes);
        }
        match nodes[0].tag.as_str() {
            "html" | "body" => nodes = trim_boundary_whitespace(nodes.remove(0).children),
            _ => return Ok(nodes),
        }
    }
}

fn trim_boundary_whitespace(nodes: Vec<HtmlNode>) -> Vec<HtmlNode> {
    let first = nodes
        .iter()
        .position(|node| !node.is_whitespace_text_node())
        .unwrap_or(nodes.len());
    let last = nodes
        .iter()
        .rposition(|node| !node.is_whitespace_text_node())
        .map(|index| index + 1)
        .unwrap_or(first);
    nodes[first..last].to_vec()
}

fn find_body_handle(handle: &Handle, body: &mut Option<Handle>) {
    if body.is_some() {
        return;
    }
    if let NodeData::Element { name, .. } = &handle.data
        && name.local.as_ref() == "body"
    {
        *body = Some(handle.clone());
        return;
    }
    for child in handle.children.borrow().iter() {
        find_body_handle(child, body);
    }
}

fn append_rcdom_node(
    handle: &Handle,
    nodes: &mut Vec<HtmlNode>,
    diagnostics: &mut Vec<HtmlDiagnostic>,
) -> HtmlResult<()> {
    match &handle.data {
        NodeData::Document => nodes.extend(rcdom_children_to_html(
            &handle.children.borrow(),
            diagnostics,
        )?),
        NodeData::Doctype { .. } | NodeData::Comment { .. } => {}
        NodeData::Text { contents } => {
            let text = sanitize_parsed_html_string(contents.borrow().to_string(), "text content")?;
            nodes.push(HtmlNode::text_node(text));
        }
        NodeData::Element { name, attrs, .. } => {
            let tag = sanitize_parsed_html_string(name.local.to_string(), "tag name")?;
            if tag == "script" {
                diagnostics.push(HtmlDiagnostic::new(
                    HtmlDiagnosticCode::ScriptElementIgnored,
                    "script elements are ignored; JavaScript is not part of the document runtime",
                    Some(tag),
                    None,
                ));
                return Ok(());
            }

            let mut id = None;
            let mut classes = Vec::new();
            let mut role = None;
            let mut attributes = BTreeMap::new();
            let mut behavior_hooks = Vec::new();

            for attr in attrs.borrow().iter() {
                let name =
                    sanitize_parsed_html_string(html_attribute_name(&attr.name), "attribute name")?;
                let mut value = sanitize_parsed_html_string(
                    attr.value.to_string(),
                    format!("attribute `{name}`"),
                )?;
                if name.starts_with("data-") {
                    value = value.trim().to_owned();
                }
                if name == "id" {
                    id = Some(value);
                } else if name == "class" {
                    classes.extend(value.split_whitespace().map(str::to_owned));
                } else if name == "role" {
                    role = Some(value);
                } else if let Some(event) = name.strip_prefix("on:") {
                    push_behavior_hook(&tag, &name, event, value, &mut behavior_hooks, diagnostics);
                } else if let Some(event) = command_attribute_event(&name) {
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
                    if name == "style" {
                        parse_inline_text_style(&value).map_err(|message| HtmlError::Parse {
                            source: None,
                            offset: 0,
                            line: 1,
                            column: 1,
                            message: format!(
                                "invalid inline style on `<{tag}>` attribute `style`: {message}"
                            ),
                        })?;
                    }
                    attributes.insert(name, value);
                }
            }
            parse_table_spec_attributes(&attributes).map_err(|message| HtmlError::Parse {
                source: None,
                offset: 0,
                line: 1,
                column: 1,
                message: format!("invalid table metadata on `<{tag}>`: {message}"),
            })?;
            if attributes
                .get("data-column")
                .is_some_and(|value| value.trim().is_empty())
            {
                return Err(HtmlError::Parse {
                    source: None,
                    offset: 0,
                    line: 1,
                    column: 1,
                    message: format!("invalid table cell metadata on `<{tag}>`: empty data-column"),
                });
            }
            if let Some(glyph) = attributes.get("data-glyph") {
                parse_glyph(glyph).map_err(|message| HtmlError::Parse {
                    source: None,
                    offset: 0,
                    line: 1,
                    column: 1,
                    message: format!("invalid glyph metadata on `<{tag}>`: {message}"),
                })?;
            }

            let children = rcdom_children_to_html(&handle.children.borrow(), diagnostics)?;
            let text = if children.len() == 1
                && children[0].is_text()
                && children[0]
                    .text
                    .as_ref()
                    .is_some_and(|text| !text.trim().is_empty())
            {
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
    Ok(())
}

fn html_attribute_name(name: &QualName) -> String {
    if let Some(prefix) = &name.prefix {
        format!("{prefix}:{}", name.local)
    } else {
        name.local.to_string()
    }
}

fn html_boolean_attribute(attributes: &BTreeMap<String, String>, name: &str) -> bool {
    attributes
        .get(name)
        .is_some_and(|value| !matches!(value.trim().to_ascii_lowercase().as_str(), "false" | "0"))
}

fn parse_glyph(input: &str) -> Result<Glyph, String> {
    match input.trim() {
        "check" => Ok(Glyph::Check),
        "chevron-down" => Ok(Glyph::ChevronDown),
        "chevron-up" => Ok(Glyph::ChevronUp),
        "drag-handle" => Ok(Glyph::DragHandle),
        "" => Err("data-glyph cannot be empty".to_owned()),
        value => Err(format!("unknown glyph `{value}`")),
    }
}

fn parse_table_spec_attributes(
    attributes: &BTreeMap<String, String>,
) -> Result<Option<TableSpec>, String> {
    let Some(columns) = attributes.get("data-table-columns") else {
        if attributes.contains_key("data-table-header-height")
            || attributes.contains_key("data-table-row-height")
        {
            return Err("table height metadata requires data-table-columns".to_owned());
        }
        return Ok(None);
    };
    let mut table = TableSpec::new(parse_table_columns(columns)?);
    if let Some(height) = attributes.get("data-table-header-height") {
        table = table.header_height(parse_table_px(height, "data-table-header-height")?);
    }
    if let Some(height) = attributes.get("data-table-row-height") {
        table = table.row_height(parse_table_px(height, "data-table-row-height")?);
    }
    Ok(Some(table))
}

fn parse_table_columns(input: &str) -> Result<Vec<TableColumnSpec>, String> {
    let columns = input
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(parse_table_column)
        .collect::<Result<Vec<_>, _>>()?;
    if columns.is_empty() {
        return Err("data-table-columns must declare at least one column".to_owned());
    }
    let mut ids = BTreeSet::new();
    for column in &columns {
        if !ids.insert(column.id.as_str()) {
            return Err(format!(
                "data-table-columns declares duplicate column `{}`",
                column.id.as_str()
            ));
        }
    }
    Ok(columns)
}

fn parse_table_column(input: &str) -> Result<TableColumnSpec, String> {
    let parts = input.split(':').map(str::trim).collect::<Vec<_>>();
    if parts.len() != 4 {
        return Err(format!(
            "table column `{input}` must use id:title:width:min-width"
        ));
    }
    let id = parts[0];
    let title = parts[1];
    let width = parts[2];
    let min_width = parts[3];
    if id.is_empty() {
        return Err(format!("table column `{input}` has an empty id"));
    }
    if title.is_empty() {
        return Err(format!("table column `{input}` has an empty title"));
    }
    Ok(TableColumnSpec::new(id, title)
        .width(parse_table_track_size(width)?)
        .min_width(parse_table_px(min_width, "table column min-width")?))
}

fn parse_table_track_size(input: &str) -> Result<TableTrackSize, String> {
    if let Some(value) = input.strip_suffix("px") {
        return Ok(TableTrackSize::px(parse_table_f32(
            value,
            "table column width",
        )?));
    }
    if let Some(value) = input.strip_suffix("fr") {
        return Ok(TableTrackSize::flex(parse_table_f32(
            value,
            "table column width",
        )?));
    }
    Err(format!("table column width `{input}` must use px or fr"))
}

fn parse_table_px(input: &str, context: &str) -> Result<f32, String> {
    let Some(value) = input.strip_suffix("px") else {
        return Err(format!("{context} `{input}` must use px"));
    };
    parse_table_f32(value, context)
}

fn parse_table_f32(input: &str, context: &str) -> Result<f32, String> {
    let value = input
        .parse::<f32>()
        .map_err(|_| format!("{context} expects a number, got `{input}`"))?;
    if !value.is_finite() || value < 0.0 {
        return Err(format!("{context} expects a non-negative finite number"));
    }
    Ok(value)
}

fn push_text_run(runs: &mut Vec<TextRun>, text: &str, style: InlineTextStyle) {
    if !text.is_empty() {
        runs.push(TextRun::styled(text.to_owned(), style));
    }
}

fn apply_inline_tag_style(style: &mut InlineTextStyle, tag: &str) {
    match tag {
        "b" | "strong" => style.font_weight = Some(FontWeight::BOLD),
        "i" | "em" => style.font_style = Some(FontStyle::Italic),
        "u" => merge_text_decoration(style, TextDecoration::UNDERLINE),
        "s" | "strike" => merge_text_decoration(style, TextDecoration::LINE_THROUGH),
        "sub" => style.vertical_align = Some(TextVerticalAlign::Sub),
        "sup" => style.vertical_align = Some(TextVerticalAlign::Super),
        _ => {}
    }
}

fn inline_style_from_style_attr(value: Option<&String>) -> InlineTextStyle {
    value
        .and_then(|value| parse_inline_text_style(value).ok())
        .unwrap_or_default()
}

fn apply_inline_style(style: &mut InlineTextStyle, next: InlineTextStyle) {
    if next.color.is_some() {
        style.color = next.color;
    }
    if next.font_size.is_some() {
        style.font_size = next.font_size;
    }
    if next.line_height.is_some() {
        style.line_height = next.line_height;
    }
    if next.letter_spacing.is_some() {
        style.letter_spacing = next.letter_spacing;
    }
    if next.font_family.is_some() {
        style.font_family = next.font_family;
    }
    if next.font_weight.is_some() {
        style.font_weight = next.font_weight;
    }
    if next.font_stretch.is_some() {
        style.font_stretch = next.font_stretch;
    }
    if next.font_style.is_some() {
        style.font_style = next.font_style;
    }
    if next.text_transform.is_some() {
        style.text_transform = next.text_transform;
    }
    if next.text_decoration.is_some() {
        style.text_decoration = next.text_decoration;
    }
    if next.vertical_align.is_some() {
        style.vertical_align = next.vertical_align;
    }
    if next.background.is_some() {
        style.background = next.background;
    }
}

fn merge_text_decoration(style: &mut InlineTextStyle, next: TextDecoration) {
    let mut decoration = style.text_decoration.unwrap_or_default();
    decoration.underline |= next.underline;
    decoration.overline |= next.overline;
    decoration.line_through |= next.line_through;
    if next.color.is_some() {
        decoration.color = next.color;
    }
    if next.thickness.is_some() {
        decoration.thickness = next.thickness;
    }
    style.text_decoration = Some(decoration);
}

fn parse_inline_text_style(input: &str) -> Result<InlineTextStyle, String> {
    let mut style = InlineTextStyle::default();
    for declaration in input.split(';') {
        let declaration = declaration.trim();
        if declaration.is_empty() {
            continue;
        }
        let (property, value) = declaration
            .split_once(':')
            .ok_or_else(|| format!("inline style declaration `{declaration}` is missing `:`"))?;
        apply_inline_text_declaration(&mut style, property.trim(), value.trim())?;
    }
    Ok(style)
}

fn apply_inline_text_declaration(
    style: &mut InlineTextStyle,
    property: &str,
    value: &str,
) -> Result<(), String> {
    match property {
        "color" | "text-color" => style.color = Some(parse_inline_color(value)?),
        "background" | "background-color" => style.background = Some(parse_inline_color(value)?),
        "font-size" => style.font_size = Some(parse_inline_px(value)?),
        "line-height" => style.line_height = Some(parse_inline_px(value)?),
        "letter-spacing" => style.letter_spacing = Some(parse_inline_px(value)?),
        "font-family" => style.font_family = Some(value.to_owned()),
        "font-weight" => style.font_weight = Some(parse_inline_font_weight(value)?),
        "font-stretch" => style.font_stretch = Some(parse_inline_font_stretch(value)?),
        "font-style" => style.font_style = Some(parse_inline_font_style(value)?),
        "text-transform" => style.text_transform = Some(parse_inline_text_transform(value)?),
        "text-decoration" => style.text_decoration = Some(parse_inline_text_decoration(value)?),
        "vertical-align" => style.vertical_align = Some(parse_inline_vertical_align(value)?),
        other => return Err(format!("unsupported inline text property `{other}`")),
    }
    Ok(())
}

fn parse_inline_font_weight(input: &str) -> Result<FontWeight, String> {
    match input {
        "normal" => Ok(FontWeight::NORMAL),
        "bold" => Ok(FontWeight::BOLD),
        value => value
            .parse::<u16>()
            .map(FontWeight::new)
            .map_err(|_| format!("unsupported font-weight `{input}`")),
    }
}

fn parse_inline_font_stretch(input: &str) -> Result<FontStretch, String> {
    match input {
        "normal" => Ok(FontStretch::NORMAL),
        "condensed" => Ok(FontStretch::CONDENSED),
        "expanded" => Ok(FontStretch::EXPANDED),
        value => value
            .strip_suffix('%')
            .ok_or_else(|| format!("unsupported font-stretch `{input}`"))?
            .parse::<f32>()
            .map(FontStretch::percent)
            .map_err(|_| format!("unsupported font-stretch `{input}`")),
    }
}

fn parse_inline_font_style(input: &str) -> Result<FontStyle, String> {
    match input {
        "normal" => Ok(FontStyle::Normal),
        "italic" => Ok(FontStyle::Italic),
        "oblique" => Ok(FontStyle::Oblique),
        _ => Err(format!("unsupported font-style `{input}`")),
    }
}

fn parse_inline_text_transform(input: &str) -> Result<TextTransform, String> {
    match input {
        "none" => Ok(TextTransform::None),
        "uppercase" => Ok(TextTransform::Uppercase),
        "lowercase" => Ok(TextTransform::Lowercase),
        "capitalize" => Ok(TextTransform::Capitalize),
        _ => Err(format!("unsupported text-transform `{input}`")),
    }
}

fn parse_inline_vertical_align(input: &str) -> Result<TextVerticalAlign, String> {
    match input {
        "baseline" => Ok(TextVerticalAlign::Baseline),
        "top" => Ok(TextVerticalAlign::Top),
        "middle" => Ok(TextVerticalAlign::Middle),
        "bottom" => Ok(TextVerticalAlign::Bottom),
        "sub" => Ok(TextVerticalAlign::Sub),
        "super" => Ok(TextVerticalAlign::Super),
        _ => Err(format!("unsupported vertical-align `{input}`")),
    }
}

fn parse_inline_text_decoration(input: &str) -> Result<TextDecoration, String> {
    if input == "none" {
        return Ok(TextDecoration::NONE);
    }

    let mut decoration = TextDecoration::NONE;
    for part in input.split_whitespace() {
        match part {
            "underline" => decoration.underline = true,
            "overline" => decoration.overline = true,
            "line-through" => decoration.line_through = true,
            value if value.ends_with("px") => decoration.thickness = Some(parse_inline_px(value)?),
            value if value.starts_with('#') => decoration.color = Some(parse_inline_color(value)?),
            other => return Err(format!("unsupported text-decoration token `{other}`")),
        }
    }
    Ok(decoration)
}

fn parse_inline_px(input: &str) -> Result<f32, String> {
    input
        .trim_end_matches("px")
        .parse::<f32>()
        .map_err(|_| format!("expected pixel length, got `{input}`"))
}

fn parse_inline_color(input: &str) -> Result<Color, String> {
    let hex = input
        .strip_prefix('#')
        .ok_or_else(|| format!("expected hex color, got `{input}`"))?;
    let channel = |range: std::ops::Range<usize>| -> Result<u8, String> {
        u8::from_str_radix(&hex[range], 16)
            .map_err(|_| format!("expected hex color, got `{input}`"))
    };
    match hex.len() {
        6 => Ok(Color::rgb(channel(0..2)?, channel(2..4)?, channel(4..6)?)),
        8 => Ok(Color::rgba(
            channel(0..2)?,
            channel(2..4)?,
            channel(4..6)?,
            channel(6..8)?,
        )),
        _ => Err(format!("expected 6 or 8 digit hex color, got `{input}`")),
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
    matches!(
        name,
        "onabort"
            | "onanimationcancel"
            | "onanimationend"
            | "onanimationiteration"
            | "onanimationstart"
            | "onauxclick"
            | "onbeforeinput"
            | "onbeforematch"
            | "onbeforetoggle"
            | "onblur"
            | "oncancel"
            | "oncanplay"
            | "oncanplaythrough"
            | "onchange"
            | "onclick"
            | "onclose"
            | "oncontextmenu"
            | "oncopy"
            | "oncuechange"
            | "oncut"
            | "ondblclick"
            | "ondrag"
            | "ondragend"
            | "ondragenter"
            | "ondragleave"
            | "ondragover"
            | "ondragstart"
            | "ondrop"
            | "ondurationchange"
            | "onemptied"
            | "onended"
            | "onerror"
            | "onfocus"
            | "onformdata"
            | "oninput"
            | "oninvalid"
            | "onkeydown"
            | "onkeypress"
            | "onkeyup"
            | "onload"
            | "onloadeddata"
            | "onloadedmetadata"
            | "onloadstart"
            | "onmousedown"
            | "onmouseenter"
            | "onmouseleave"
            | "onmousemove"
            | "onmouseout"
            | "onmouseover"
            | "onmouseup"
            | "onpaste"
            | "onpause"
            | "onplay"
            | "onplaying"
            | "onpointercancel"
            | "onpointerdown"
            | "onpointerenter"
            | "onpointerleave"
            | "onpointermove"
            | "onpointerout"
            | "onpointerover"
            | "onpointerrawupdate"
            | "onpointerup"
            | "onprogress"
            | "onratechange"
            | "onreset"
            | "onresize"
            | "onscroll"
            | "onscrollend"
            | "onsecuritypolicyviolation"
            | "onseeked"
            | "onseeking"
            | "onselect"
            | "onslotchange"
            | "onstalled"
            | "onsubmit"
            | "onsuspend"
            | "ontimeupdate"
            | "ontoggle"
            | "ontouchcancel"
            | "ontouchend"
            | "ontouchmove"
            | "ontouchstart"
            | "ontransitioncancel"
            | "ontransitionend"
            | "ontransitionrun"
            | "ontransitionstart"
            | "onvolumechange"
            | "onwaiting"
            | "onwebkitanimationend"
            | "onwebkitanimationiteration"
            | "onwebkitanimationstart"
            | "onwebkittransitionend"
            | "onwheel"
    )
}

fn command_attribute_event(name: &str) -> Option<&str> {
    if name == "data-command" {
        Some("click")
    } else {
        name.strip_prefix("data-command:")
            .filter(|event| !event.is_empty())
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
        "b" | "strong" | "i" | "em" | "u" | "s" | "strike" | "sub" | "sup" => Element::Span,
        "button" => Element::Button,
        "input" => Element::Input,
        "select" => Element::Select,
        "option" => Element::Option,
        "textarea" => Element::Textarea,
        "label" => Element::Label,
        "canvas" => Element::Canvas,
        "icon" => Element::Icon,
        "table" => Element::Table,
        "thead" => Element::Thead,
        "tbody" => Element::Tbody,
        "tr" => Element::Tr,
        "th" => Element::Th,
        "td" => Element::Td,
        _ => Element::Div,
    }
}

fn element_for_node(node: &HtmlNode) -> Element {
    match node.role.as_deref() {
        Some("checkbox") => Element::Checkbox,
        Some("radio") => Element::Radio,
        Some("combobox") => Element::Select,
        Some("textbox" | "searchbox") => Element::Input,
        _ => element_for_tag(&node.tag),
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
