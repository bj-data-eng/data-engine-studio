//! Document and style primitives for building host-rendered user interfaces.
//!
//! `des-ui-document` owns a DOM-like document tree, deterministic style
//! resolution, retained element state, resolved layout output, and input
//! routing. Rendering hosts such as egui translate platform input into
//! [`DocumentInput`] and paint [`DocumentOutput::layout`].

mod animation;
mod element;
mod engine;
mod geometry;
mod layout;
mod query;
mod scene;
mod scroll;
mod state;
mod style;
mod table;
mod text;
mod update;

pub use element::{
    ClassName, Color, Document, DocumentBuilder, Element, ElementId, ElementRole, ElementSpec,
    ElementStateSelector, Glyph, VisualCloneOptions, VisualElementClone,
};
pub use engine::DocumentEngine;
pub use geometry::{
    AlignItems, CornerRadii, FlexDirection, FlexWrap, Insets, JustifyContent, Length, Overflow,
    Point, Position, PositionInsets, Rect, ScrollAxis, Size,
};
pub use query::{DocumentSnapshot, ElementSnapshot, HitResult};
pub use scene::{DocumentScene, SceneElement, SceneError, SceneResult, StyleApplicationReport};
pub use state::{
    ChangeSet, DocumentDrag, DocumentEvent, DocumentEventKind, DocumentInput, DocumentMetrics,
    DocumentOutput, DocumentTextSelection, ElementState, PointerInput, ResolvedElement,
    ScrollChrome, TextSelectionGranularity,
};
pub use style::{
    Anchor, AnchorPlacement, CompoundSelector, ComputedStyle, CornerStyle, Easing, EdgeStyle,
    Shadow, Style, StyleRule, StyleSelector, StyleSheet, Transition,
};
pub use table::{TableCellSpec, TableColumnId, TableColumnSpec, TableSpec, TableTrackSize};
pub use text::{
    FallbackTextMeasurer, TextLayoutRequest, TextLayoutResult, TextMeasurer, TextMeasurerKey,
    TextWrapMode,
};
pub use update::{DocumentUpdate, DocumentUpdateReport};
