//! Reusable document-backed widgets and interaction policies.
//!
//! `des-ui-widgets` builds higher-level UI behavior on top of
//! `des-ui-document` without depending on a rendering host such as egui.

mod sortable;

pub use sortable::{
    DropEdge, DropZoneId, SortableDocumentConfig, SortableDropPreview, SortableItemId,
    SortableModel,
};
