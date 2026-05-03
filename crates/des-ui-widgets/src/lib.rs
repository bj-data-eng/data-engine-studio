//! Reusable document-backed widgets and interaction policies.
//!
//! `des-ui-widgets` builds higher-level UI behavior on top of
//! `des-ui-document` without depending on a rendering host such as egui.

mod auto_scroll;
mod sortable;

pub use auto_scroll::{AutoScrollAction, AutoScrollOptions, AutoScroller};
pub use sortable::{
    DropEdge, DropZoneId, SortableDocumentConfig, SortableDropPreview, SortableItemId,
    SortableModel,
};
