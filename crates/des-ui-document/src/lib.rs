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
mod scroll;
mod state;
mod style;

pub use element::{
    ClassName, Color, Document, DocumentBuilder, Element, ElementId, ElementRole, ElementSpec,
    ElementStateSelector,
};
pub use engine::DocumentEngine;
pub use geometry::{CornerRadii, Direction, Insets, Length, Overflow, Point, Rect, Size};
pub use state::{
    ChangeSet, DocumentInput, DocumentOutput, ElementState, PointerInput, ResolvedElement,
    ScrollChrome,
};
pub use style::{
    ComputedStyle, CornerStyle, Easing, EdgeStyle, Style, StyleRule, StyleSelector, StyleSheet,
    Transition,
};
