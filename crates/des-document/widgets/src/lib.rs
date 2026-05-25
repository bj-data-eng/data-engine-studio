//! Reusable document-backed widgets and interaction policies.
//!
//! `des-widgets` builds higher-level UI behavior on top of
//! `des-document` without depending on a rendering host such as egui.

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

/// Common app-facing imports for reusable document-backed widgets.
///
/// Widget authors and app code can use this prelude to compose reusable widget
/// behavior into the retained document tree, contribute widget styles, project
/// widget state, and collect typed Rust actions without depending on a rendering
/// host such as egui.
pub mod prelude {
    pub use crate::{
        AutoScrollAction, AutoScrollOptions, AutoScroller, CONTEXT_MENU_CLASS,
        CONTEXT_MENU_ITEM_CLASS, CONTEXT_MENU_LABEL_CLASS, CONTEXT_MENU_SEPARATOR_CLASS,
        ContextMenu, ContextMenuEntry, ContextMenuItem, DropEdge, DropZoneId,
        SortableDocumentConfig, SortableDropPreview, SortableItemId, SortableModel,
        context_menu_surface_style,
    };
    pub use des_document::prelude::*;
}

#[cfg(test)]
mod tests {
    #[test]
    fn widgets_prelude_exposes_document_widget_authoring_conventions() {
        use crate::prelude::*;

        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        enum MenuAction {
            Copy,
        }

        let menu = ContextMenu::new("text-context-menu")
            .command_item("copy", "Copy", "copy-selection")
            .disabled_item("paste", "Paste");
        let registry = menu.command_registry(|item| {
            (item.command_name() == Some("copy-selection")).then_some(MenuAction::Copy)
        });
        let mut view = DocumentView::compose(Size::new(240.0, 140.0)).widget(&menu);

        let frame = view.update_with_input_actions(
            DocumentInput::primary_click(Point::new(2.0, 2.0)),
            &registry,
        );
        let copy = frame.output.snapshot().find("copy").unwrap();
        let paste = frame.output.snapshot().find("paste").unwrap();

        assert!(copy.has_class(CONTEXT_MENU_ITEM_CLASS));
        assert!(copy.interactive());
        assert!(!paste.interactive());
        assert_eq!(frame.actions.len(), 1);
        assert_eq!(frame.actions[0].target, ElementId::new("copy"));
        assert_eq!(frame.actions[0].action, MenuAction::Copy);
    }
}
