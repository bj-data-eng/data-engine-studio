//! Product-specific UI runtime primitives.
//!
//! `des-ui-runtime` owns the DOM-like element tree, deterministic style
//! resolution, retained interaction state, layout frames, and input routing.
//! Rendering hosts such as egui should translate platform input into
//! [`RuntimeInput`] and paint [`RuntimeOutput::layout`].

mod animation;
mod element;
mod geometry;
mod layout;
mod runtime;
mod scroll;
mod state;
mod style;

pub use element::{
    ClassName, Color, Element, ElementId, ElementRole, ElementSpec, ElementStateSelector, Scene, Ui,
};
pub use geometry::{Direction, Insets, Length, Overflow, Point, Rect, Size};
pub use runtime::Runtime;
pub use state::{
    ChangeSet, ElementState, LayoutFrame, PointerInput, RuntimeInput, RuntimeOutput, ScrollChrome,
};
pub use style::{
    ComputedStyle, Easing, StylePatch, StyleRule, StyleSelector, StyleSheet, Transition,
};
