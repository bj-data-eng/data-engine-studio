//! # Data Engine Studio Layout Engine
//!
//! A flexible, high-performance engine for computing UI layout.
//! It currently implements the Flexbox, Grid and Block layout algorithms from the CSS specification.
//!
//! ## Architecture
//!
//! The engine is based on a tree of UI nodes. Each node has:
//!   - A [`Style`] struct which holds a set of CSS styles which function as the primary input to the layout computations.
//!   - A [`Layout`] struct containing a position (x/y) and a size (width/height) which function as the output of the layout computations.
//!   - Optionally:
//!       - A `Vec` set of child nodes
//!       - "Context": arbitrary user-defined data (which you can access when using a "measure function" to integrate this engine with other kinds of layout such as text layout)
//!
//! Usage consists of constructing a tree of UI nodes, then asking the engine to translate those styles,
//! parent-child relationships and measure functions into a size and position for each node.
//!
//! ## High-level API vs. Low-level API
//!
//! The engine has two APIs: a high-level API that owns the layout tree, and a low-level API that lets
//! callers provide their own tree storage and caching.
//!
//! ### High-level API
//!
//! The high-level API consists of the [`LayoutTree`] struct which contains a tree implementation and provides methods that allow you to construct
//! a tree of UI nodes. Once constructed, you can call the [`compute_layout_with_measure`](crate::LayoutTree::compute_layout_with_measure) method to compute the layout (passing in a "measure function" closure which is used to compute the size of leaf nodes), and then access
//! the layout of each node using the [`layout`](crate::LayoutTree::layout) method.
//!
//! When using the high-level API, [`LayoutTree`] takes care of node storage, caching and dispatching to the correct layout algorithm for a given node.
//! See the [`LayoutTree`] struct for more details on this API.
//!
//! ### Low-level API
//!
//! The low-level API consists of a [set of traits](crate::tree::traits) (notably the [`LayoutPartialTree`] trait) which define an interface behind which you must implement your own
//! tree implementation, and a [set of functions](crate::compute) such as [`compute_flexbox_layout`] and [`compute_grid_layout`] which implement the layout algorithms (for a single node at a time), and are designed to be flexible
//! and easy to integrate into a wider layout or UI system.
//!
//! When using this API, you must handle node storage, caching, and dispatching to the correct layout algorithm for a given node yourself.
//! See the [`crate::tree::traits`] module for more details on this API.
//!

// document the feature flags for the crate by extracting the comments from Cargo.toml
#![cfg_attr(feature = "document-features", doc = document_features::document_features!())]
// annotate items with their required features (gated by docsrs flag as this requires the nightly toolchain)
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(not(feature = "std"), no_std)]
#![deny(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]
// Disable "unused_x" warnings when default features aren't enabled.
#![cfg_attr(not(feature = "default"), allow(dead_code))]
#![cfg_attr(not(feature = "default"), allow(unused_imports))]
#![cfg_attr(not(feature = "default"), allow(unused_variables))]
#![cfg_attr(not(feature = "default"), allow(unused_mut))]

// We always need std for the tests
// See <https://github.com/la10736/rstest/issues/149#issuecomment-1156402989>
#[cfg(all(test, not(feature = "std")))]
#[macro_use]
extern crate std;

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

#[cfg_attr(feature = "serde", macro_use)]
#[cfg(feature = "serde")]
extern crate serde;

pub mod compute;
pub mod floating;
pub mod geometry;
pub mod prelude;
pub mod style;
pub mod style_helpers;
pub mod tree;
#[macro_use]
pub mod util;

#[cfg(feature = "block_layout")]
#[doc(inline)]
pub use crate::compute::compute_block_layout;
#[cfg(feature = "flexbox")]
#[doc(inline)]
pub use crate::compute::compute_flexbox_layout;
#[cfg(feature = "grid")]
#[doc(inline)]
pub use crate::compute::compute_grid_layout;
#[cfg(feature = "detailed_layout_info")]
pub use crate::compute::detailed_info::*;
#[doc(inline)]
pub use crate::compute::{
    compute_cached_layout, compute_hidden_layout, compute_leaf_layout, compute_root_layout,
    round_layout,
};
#[doc(inline)]
pub use crate::style::Style;
#[doc(inline)]
pub use crate::tree::traits::*;
#[cfg(feature = "layout_tree")]
#[doc(inline)]
pub use crate::tree::LayoutTree;
#[cfg(feature = "std")]
#[doc(inline)]
pub use crate::util::print_tree;

#[cfg(feature = "parse")]
pub use parse::{ParseError, ParseResult};

pub use crate::compute::*;
pub use crate::floating::*;
pub use crate::geometry::*;
pub use crate::style::*;
pub use crate::tree::*;
pub use crate::util::*;
