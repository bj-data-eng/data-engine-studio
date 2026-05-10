//! Reusable document-backed widgets and interaction policies.
//!
//! `des-ui-widgets` builds higher-level UI behavior on top of
//! `des-ui-document` without depending on a rendering host such as egui.

mod auto_scroll;
mod context_menu;
mod sortable;

pub use auto_scroll::{AutoScrollAction, AutoScrollOptions, AutoScroller};
pub use context_menu::{
    CONTEXT_MENU_CLASS, CONTEXT_MENU_ITEM_CLASS, CONTEXT_MENU_LABEL_CLASS,
    CONTEXT_MENU_SEPARATOR_CLASS, ContextMenu, ContextMenuEntry, ContextMenuItem,
    context_menu_surface_style,
};
pub use sortable::{
    DropEdge, DropZoneId, SortableDocumentConfig, SortableDropPreview, SortableItemId,
    SortableModel,
};
