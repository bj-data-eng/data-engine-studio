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
//! struct RunButton {
//!     ready: bool,
//! }
//!
//! impl DocumentWidget for RunButton {
//!     fn render(&self, ui: &mut DocumentBuilder) {
//!         ui.button("run")
//!             .classes(["control", "primary"])
//!             .aria("label", "Run query")
//!             .command("run")
//!             .text("Run");
//!     }
//!
//!     fn push_styles(&self, stylesheet: &mut StyleSheet) {
//!         stylesheet.push_class("control", Style::default().size(96.0, 36.0));
//!     }
//!
//!     fn push_projection(&self, projection: &mut DocumentProjection) {
//!         projection
//!             .element("run")
//!             .data("state", if self.ready { "ready" } else { "waiting" })
//!             .class_if("is-ready", self.ready);
//!     }
//! }
//!
//! impl DocumentActionWidget<AppAction> for RunButton {
//!     fn push_commands(&self, registry: &mut DocumentCommandRegistry<AppAction>) {
//!         registry.push_click("run", AppAction::Run);
//!     }
//! }
//!
//! let widget = RunButton { ready: false };
//! let mut surface = widget
//!     .action_surface_with_css(
//!         Size::new(320.0, 180.0),
//!         ".primary { background: rgb(222, 238, 255); }",
//!     )
//!     .expect("valid app stylesheet");
//!
//! let mut actions = Vec::new();
//! let (projection, frame, dispatch) = surface
//!     .project_with_and_update_with_input_and_dispatch_action_values(
//!         DocumentInput::primary_click(Point::new(8.0, 8.0)),
//!         |projection| {
//!             projection
//!                 .element("run")
//!                 .data("state", "ready")
//!                 .class_if("is-ready", true);
//!         },
//!         |action| actions.push(*action),
//!     )
//!     .expect("widget projection targets rendered elements");
//!
//! assert_eq!(projection.changed, 2);
//! assert_eq!(dispatch.handled_count(), 1);
//! assert_eq!(actions, vec![AppAction::Run]);
//! assert!(frame.output().snapshot().find("run").unwrap().has_class("is-ready"));
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
    DocumentKey, DocumentMetrics, DocumentOutput, DocumentTextSelection, ElementState, KeyInput,
    KeyModifiers, PointerInput, ResolvedElement, ResolvedFloating, ScrollChrome,
    TextSelectionGranularity,
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
pub use view::{DocumentActionFrame, DocumentActionSurface, DocumentView, DocumentViewBuilder};

/// Common app-facing imports for authoring document UIs.
///
/// This prelude intentionally collects the fluent Rust authoring surface,
/// browser-inspired style/document primitives, retained-state projection, and
/// interaction outputs most application code needs. Lower-level layout,
/// animation, and host-adapter details remain available from the crate root.
pub mod prelude {
    pub use crate::{
        AlignContent, AlignItems, AlignSelf, Anchor, AnchorPlacement, BorderStyle, ChangeSet,
        ClassName, ClipRect, Color, ComplexSelector, CompoundSelector, ComputedStyle,
        ContainerQuery, CornerRadii, CornerStyle, CssParseError, Direction, Display, Document,
        DocumentActionFrame, DocumentActionSurface, DocumentActionWidget, DocumentAuthoringError,
        DocumentAuthoringResult, DocumentBuilder, DocumentCommand, DocumentCommandAction,
        DocumentCommandActionRef, DocumentCommandBinding, DocumentCommandDispatchReport,
        DocumentCommandRef, DocumentCommandRegistry, DocumentDrag, DocumentEngine, DocumentError,
        DocumentEvent, DocumentEventKind, DocumentInput, DocumentKey, DocumentMetrics,
        DocumentOutput, DocumentProjection, DocumentProjectionOperation,
        DocumentProjectionOperationKind, DocumentProjectionReport, DocumentQueryError,
        DocumentResult, DocumentSnapshot, DocumentTextSelection, DocumentView, DocumentViewBuilder,
        DocumentWidget, Easing, EdgeStyle, Element, ElementBehaviorEvent, ElementBehaviorHook,
        ElementBuilder, ElementId, ElementProjection, ElementProjectionPatch, ElementSnapshot,
        ElementSpec, ElementState, ElementStateSelector, FallbackTextMeasurer, FlexDirection,
        FlexWrap, FloatingArrow, FloatingArrowData, FloatingAutoPlacement, FloatingAxisOffset,
        FloatingBoundary, FloatingFallbackAxisSideDirection, FloatingFallbackStrategy,
        FloatingFlip, FloatingFlipCrossAxis, FloatingHide, FloatingHideData, FloatingHideStrategy,
        FloatingInline, FloatingOffset, FloatingOptions, FloatingPlacement, FloatingShift,
        FloatingShiftLimiter, FloatingSize, FloatingVisibility, FontStretch, FontStyle, FontWeight,
        Glyph, GridAutoFlow, GridPlacement, GridPlacementLine, GridTemplateArea,
        GridTemplateComponent, GridTemplateRepetition, GridTrack, HitResult, InlineTextStyle,
        Insets, JustifyContent, KeyInput, KeyModifiers, Length, MaxTrackSizingFunction,
        MinTrackSizingFunction, NormalizedText, NthChildFormula, Overflow, OverflowWrap, Point,
        PointerInput, Position, PositionInsets, Rect, RepetitionCount, ResolvedElement,
        ResolvedFloating, ScrollAxis, ScrollChrome, SelectorCombinator, Shadow, Size, Style,
        StyleCondition, StyleRule, StyleSelector, StyleSheet, TableCellSpec, TableColumnId,
        TableColumnSpec, TableSpec, TableTrackSize, TextAlign, TextContent, TextDecoration,
        TextLayoutLine, TextLayoutRequest, TextLayoutResult, TextLayoutRun, TextLayoutStyle,
        TextMeasurer, TextMeasurerKey, TextOverflow, TextRun, TextSelectionGranularity,
        TextTransform, TextVerticalAlign, TextWrapMode, TrackSizingFunction, Transition,
        ViewportQuery, VisualCloneOptions, VisualElementClone, WhiteSpace, WhiteSpaceCollapse,
        WordBreak,
    };
}
