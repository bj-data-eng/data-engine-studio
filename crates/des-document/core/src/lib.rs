//! Retained document and style primitives for host-rendered user interfaces.
//!
//! `des-document` owns the retained document tree, deterministic style
//! resolution, retained element state, resolved layout output, and input
//! routing. Rendering hosts such as egui translate platform input into
//! [`DocumentInput`] and paint [`DocumentOutput::layout`].
//!
//! ## App-facing authoring model
//!
//! Application code should usually start from [`prelude`], compose a
//! [`DocumentView`] from typed Rust widgets or browser-grade HTML/CSS, project
//! app state into retained document state, then translate document commands
//! back into typed app actions. The document layer keeps this loop independent
//! from egui so structure, style, behavior intent, and state projection can be
//! tested without a host renderer.
//!
//! ```
//! use des_document::prelude::*;
//!
//! #[derive(Clone, Copy, Debug, Eq, PartialEq)]
//! enum AppAction {
//!     Run,
//! }
//!
//! struct QueryToolbar {
//!     ready: bool,
//! }
//!
//! impl DocumentWidget for QueryToolbar {
//!     fn render(&self, ui: &mut DocumentBuilder) {
//!         ui.header("query-toolbar")
//!             .class("toolbar")
//!             .children(|ui| {
//!                 ui.button("run-query")
//!                     .classes(["button", "primary"])
//!                     .enabled(self.ready)
//!                     .aria("label", "Run query")
//!                     .command("query.run")
//!                     .text(if self.ready { "Run" } else { "Waiting" });
//!
//!                 ui.button("query-menu")
//!                     .class("icon-button")
//!                     .on_context_menu("query.menu")
//!                     .text("Menu");
//!             });
//!     }
//!
//!     fn push_styles(&self, stylesheet: &mut StyleSheet) {
//!         stylesheet
//!             .extend_css_forgiving(
//!                 ".toolbar { display: flex; gap: 8px; padding: 12px; }
//!                  .button { width: 96px; height: 32px; }
//!                  .icon-button { width: 72px; height: 32px; }",
//!             )
//!             .expect("widget CSS should parse");
//!     }
//!
//!     fn push_projection(&self, projection: &mut DocumentProjection) {
//!         projection
//!             .element("run-query")
//!             .data("state", if self.ready { "ready" } else { "waiting" })
//!             .class_if("is-ready", self.ready)
//!             .enabled(self.ready);
//!     }
//! }
//!
//! impl DocumentActionWidget<AppAction> for QueryToolbar {
//!     fn push_commands(&self, registry: &mut DocumentCommandRegistry<AppAction>) {
//!         registry.push_click("query.run", AppAction::Run);
//!     }
//! }
//!
//! let widget = QueryToolbar { ready: false };
//! let mut surface = widget
//!     .action_surface_with_css(
//!         Size::new(320.0, 180.0),
//!         ".primary { background: rgb(222, 238, 255); }",
//!     )
//!     .expect("valid app stylesheet");
//!
//! let mut actions = Vec::new();
//! let frame = surface
//!     .update_request()
//!     .input(DocumentInput::primary_click(Point::new(20.0, 20.0)))
//!     .project_with(|projection| {
//!         projection
//!             .element("run-query")
//!             .data("state", "ready")
//!             .class_if("is-ready", true)
//!             .enabled(true);
//!     })
//!     .dispatch_action_values(|action| actions.push(*action))
//!     .expect("widget projection targets rendered elements");
//!
//! assert_eq!(actions, vec![AppAction::Run]);
//! assert!(frame.output().snapshot().find("run-query").unwrap().has_class("is-ready"));
//! ```
//!
//! HTML/CSS entry points live in the sibling `des-html` crate and produce the
//! same document/view contracts, including `on:*` behavior hooks and
//! `data-command` command metadata for typed Rust dispatch.

mod animation;
mod css;
mod document;
#[cfg(test)]
mod document_tests;
mod element;
mod engine;
mod geometry;
mod layout;
mod projection;
mod query;
mod scroll;
mod state;
mod style;
mod table;
mod text;
mod view;

pub use css::CssParseError;
pub use document::{
    Document, DocumentActionWidget, DocumentAuthoringError, DocumentAuthoringResult,
    DocumentBuilder, DocumentError, DocumentResult, DocumentWidget, ElementBuilder,
};
pub use element::{
    ClassName, Color, Element, ElementBehaviorEvent, ElementBehaviorHook, ElementId, ElementSpec,
    ElementStateSelector, Glyph, VisualCloneOptions, VisualElementClone,
};
pub use engine::DocumentEngine;
pub use geometry::{
    AlignContent, AlignItems, AlignSelf, ClipRect, CornerRadii, FlexDirection, FlexWrap, Insets,
    JustifyContent, Length, Overflow, Point, Position, PositionInsets, Rect, ScrollAxis, Size,
};
pub use projection::{
    DocumentProjection, DocumentProjectionOperation, DocumentProjectionOperationKind,
    DocumentProjectionReport, ElementProjection, ElementProjectionPatch,
};
pub use query::{DocumentQueryError, DocumentSnapshot, ElementSnapshot, HitResult};
pub use state::{
    ChangeSet, DocumentCommand, DocumentCommandAction, DocumentCommandActionRef,
    DocumentCommandBinding, DocumentCommandDispatchReport, DocumentCommandIter, DocumentCommandRef,
    DocumentCommandRegistry, DocumentDrag, DocumentEvent, DocumentEventKind, DocumentInput,
    DocumentInteractionState, DocumentKey, DocumentMetrics, DocumentOutput, DocumentTextSelection,
    ElementState, KeyInput, KeyModifiers, PointerInput, ResolvedElement, ResolvedFloating,
    ScrollChrome, TextSelectionGranularity,
};
pub use style::{
    Anchor, AnchorPlacement, BorderStyle, ComplexSelector, ComplexSelectorPart, CompoundSelector,
    ComputedStyle, ContainerQuery, CornerStyle, Direction, Display, Easing, EdgeStyle,
    FloatingArrow, FloatingArrowData, FloatingAutoPlacement, FloatingAxisOffset, FloatingBoundary,
    FloatingFallbackAxisSideDirection, FloatingFallbackStrategy, FloatingFlip,
    FloatingFlipCrossAxis, FloatingHide, FloatingHideData, FloatingHideStrategy, FloatingInline,
    FloatingOffset, FloatingOptions, FloatingPlacement, FloatingShift, FloatingShiftLimiter,
    FloatingSize, FloatingVisibility, GridAutoFlow, GridPlacement, GridPlacementLine,
    GridTemplateArea, GridTemplateComponent, GridTemplateRepetition, GridTrack,
    MaxTrackSizingFunction, MinTrackSizingFunction, NthChildFormula, RepetitionCount,
    SelectorCombinator, Shadow, Style, StyleCondition, StyleRule, StyleSelector, StyleSheet,
    TrackSizingFunction, Transition, ViewportQuery,
};
pub use table::{TableCellSpec, TableColumnId, TableColumnSpec, TableSpec, TableTrackSize};
pub use text::{
    FallbackTextMeasurer, FontStretch, FontStyle, FontWeight, InlineTextStyle, NormalizedText,
    OverflowWrap, TextAlign, TextContent, TextDecoration, TextLayoutLine, TextLayoutRequest,
    TextLayoutResult, TextLayoutRun, TextLayoutStyle, TextMeasurer, TextMeasurerKey, TextOverflow,
    TextRun, TextTransform, TextVerticalAlign, TextWrapMode, WhiteSpace, WhiteSpaceCollapse,
    WordBreak,
};
pub use view::{
    DocumentActionFrame, DocumentActionSurface, DocumentActionSurfaceUpdateRequest,
    DocumentActionUpdateFrame, DocumentUpdateFrame, DocumentUpdateRequest, DocumentView,
    DocumentViewBuilder,
};

/// Common app-facing imports for authoring document UIs.
///
/// This prelude intentionally stays small: it collects the fluent Rust
/// authoring surface, common style/document primitives, retained-state
/// projection, input, output, and typed command APIs most application and
/// widget code needs. Specialized layout, text shaping, floating, grid, table,
/// animation, and host-adapter types remain available from the crate root.
pub mod prelude {
    pub use crate::{
        AlignContent, AlignItems, AlignSelf, BorderStyle, ChangeSet, ClassName, Color,
        ComplexSelector, CompoundSelector, ComputedStyle, ContainerQuery, CornerRadii,
        CssParseError, Direction, Display, Document, DocumentActionFrame, DocumentActionSurface,
        DocumentActionSurfaceUpdateRequest, DocumentActionUpdateFrame, DocumentActionWidget,
        DocumentAuthoringError, DocumentAuthoringResult, DocumentBuilder, DocumentCommand,
        DocumentCommandAction, DocumentCommandActionRef, DocumentCommandBinding,
        DocumentCommandDispatchReport, DocumentCommandRef, DocumentCommandRegistry, DocumentDrag,
        DocumentEngine, DocumentError, DocumentEvent, DocumentEventKind, DocumentInput,
        DocumentInteractionState, DocumentKey, DocumentMetrics, DocumentOutput, DocumentProjection,
        DocumentProjectionOperation, DocumentProjectionOperationKind, DocumentProjectionReport,
        DocumentQueryError, DocumentResult, DocumentSnapshot, DocumentTextSelection,
        DocumentUpdateFrame, DocumentUpdateRequest, DocumentView, DocumentViewBuilder,
        DocumentWidget, EdgeStyle, Element, ElementBehaviorEvent, ElementBehaviorHook,
        ElementBuilder, ElementId, ElementProjection, ElementProjectionPatch, ElementSnapshot,
        ElementSpec, ElementState, ElementStateSelector, FlexDirection, FlexWrap, HitResult,
        Insets, JustifyContent, KeyInput, KeyModifiers, Length, Overflow, Point, PointerInput,
        Position, PositionInsets, Rect, ResolvedElement, ScrollAxis, ScrollChrome, Shadow, Size,
        Style, StyleCondition, StyleRule, StyleSelector, StyleSheet, TextAlign, TextContent,
        TextDecoration, TextOverflow, TextRun, TextSelectionGranularity, TextTransform,
        TextWrapMode, Transition, ViewportQuery, WhiteSpace,
    };
}
