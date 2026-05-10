//! Retained document and style primitives for host-rendered user interfaces.
//!
//! `des-ui-document` owns the retained document tree, deterministic style
//! resolution, retained element state, resolved layout output, and input
//! routing. Rendering hosts such as egui translate platform input into
//! [`DocumentInput`] and paint [`DocumentOutput::layout`].

mod animation;
mod document;
#[cfg(test)]
mod document_tests;
mod element;
mod engine;
mod geometry;
mod layout;
mod query;
mod scroll;
mod state;
mod style;
mod table;
mod text;

pub use document::{Document, DocumentBuilder, DocumentError, DocumentResult, ElementBuilder};
pub use element::{
    ClassName, Color, Element, ElementId, ElementSpec, ElementStateSelector, Glyph,
    VisualCloneOptions, VisualElementClone,
};
pub use engine::DocumentEngine;
pub use geometry::{
    AlignContent, AlignItems, AlignSelf, CornerRadii, FlexDirection, FlexWrap, Insets,
    JustifyContent, Length, Overflow, Point, Position, PositionInsets, Rect, ScrollAxis, Size,
};
pub use query::{DocumentSnapshot, ElementSnapshot, HitResult};
pub use state::{
    ChangeSet, DocumentDrag, DocumentEvent, DocumentEventKind, DocumentInput, DocumentMetrics,
    DocumentOutput, DocumentTextSelection, ElementState, PointerInput, ResolvedElement,
    ScrollChrome, TextSelectionGranularity,
};
pub use style::{
    Anchor, AnchorPlacement, CompoundSelector, ComputedStyle, ContainerQuery, CornerStyle, Display,
    Easing, EdgeStyle, GridAutoFlow, GridPlacement, GridPlacementLine, GridTemplateArea,
    GridTemplateComponent, GridTemplateRepetition, GridTrack, MaxTrackSizingFunction,
    MinTrackSizingFunction, RepetitionCount, Shadow, Style, StyleCondition, StyleRule,
    StyleSelector, StyleSheet, TrackSizingFunction, Transition, ViewportQuery,
};
pub use table::{TableCellSpec, TableColumnId, TableColumnSpec, TableSpec, TableTrackSize};
pub use text::{
    FallbackTextMeasurer, TextLayoutRequest, TextLayoutResult, TextMeasurer, TextMeasurerKey,
    TextWrapMode,
};
