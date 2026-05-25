//! Retained document and style primitives for host-rendered user interfaces.
//!
//! `des-document` owns the retained document tree, deterministic style
//! resolution, retained element state, resolved layout output, and input
//! routing. Rendering hosts such as egui translate platform input into
//! [`DocumentInput`] and paint [`DocumentOutput::layout`].

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
    Document, DocumentBuilder, DocumentError, DocumentResult, DocumentWidget, ElementBuilder,
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
    DocumentProjection, DocumentProjectionOperation, DocumentProjectionReport, ElementProjection,
};
pub use query::{DocumentSnapshot, ElementSnapshot, HitResult};
pub use state::{
    ChangeSet, DocumentCommand, DocumentCommandActionRef, DocumentCommandBinding,
    DocumentCommandDispatchReport, DocumentCommandIter, DocumentCommandRef,
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
pub use view::{DocumentView, DocumentViewBuilder};

/// Common app-facing imports for authoring document UIs.
///
/// This prelude intentionally collects the fluent Rust authoring surface,
/// browser-inspired style/document primitives, retained-state projection, and
/// interaction outputs most application code needs. Lower-level layout,
/// animation, and host-adapter details remain available from the crate root.
pub mod prelude {
    pub use crate::{
        AlignContent, AlignItems, AlignSelf, Anchor, AnchorPlacement, BorderStyle, ChangeSet,
        ClassName, Color, ComplexSelector, CompoundSelector, ComputedStyle, ContainerQuery,
        CornerRadii, CssParseError, Direction, Display, Document, DocumentBuilder, DocumentCommand,
        DocumentCommandActionRef, DocumentCommandBinding, DocumentCommandDispatchReport,
        DocumentCommandRef, DocumentCommandRegistry, DocumentDrag, DocumentEngine, DocumentError,
        DocumentEvent, DocumentEventKind, DocumentInput, DocumentKey, DocumentMetrics,
        DocumentOutput, DocumentProjection, DocumentProjectionOperation, DocumentProjectionReport,
        DocumentResult, DocumentSnapshot, DocumentTextSelection, DocumentView, DocumentViewBuilder,
        DocumentWidget, Easing, EdgeStyle, Element, ElementBehaviorEvent, ElementBehaviorHook,
        ElementBuilder, ElementId, ElementProjection, ElementSnapshot, ElementSpec, ElementState,
        ElementStateSelector, FallbackTextMeasurer, FlexDirection, FlexWrap, FloatingBoundary,
        FloatingPlacement, Glyph, GridAutoFlow, GridPlacement, GridTemplateArea,
        GridTemplateComponent, GridTrack, HitResult, InlineTextStyle, Insets, JustifyContent,
        KeyInput, KeyModifiers, Length, NthChildFormula, Overflow, Point, PointerInput, Position,
        PositionInsets, Rect, ResolvedElement, ResolvedFloating, ScrollAxis, ScrollChrome, Shadow,
        Size, Style, StyleCondition, StyleRule, StyleSelector, StyleSheet, TableCellSpec,
        TableColumnId, TableColumnSpec, TableSpec, TableTrackSize, TextAlign, TextContent,
        TextDecoration, TextLayoutRequest, TextLayoutResult, TextLayoutStyle, TextMeasurer,
        TextOverflow, TextRun, TextSelectionGranularity, TextTransform, TextVerticalAlign,
        TextWrapMode, Transition, ViewportQuery, VisualCloneOptions, VisualElementClone,
        WhiteSpace,
    };
}
