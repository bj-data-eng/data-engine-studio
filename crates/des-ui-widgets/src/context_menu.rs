use des_ui_document::{
    DocumentBuilder, Element, ElementId, ElementSpec, Insets, Length, Point, Style, StyleSelector,
};

pub const CONTEXT_MENU_CLASS: &str = "context-menu";
pub const CONTEXT_MENU_ITEM_CLASS: &str = "context-menu-item";
pub const CONTEXT_MENU_LABEL_CLASS: &str = "context-menu-label";
pub const CONTEXT_MENU_SEPARATOR_CLASS: &str = "context-menu-separator";

#[derive(Clone, Debug, PartialEq)]
pub struct ContextMenu {
    id: ElementId,
    position: Point,
    entries: Vec<ContextMenuEntry>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ContextMenuEntry {
    Item(ContextMenuItem),
    Separator { id: ElementId },
}

#[derive(Clone, Debug, PartialEq)]
pub struct ContextMenuItem {
    pub id: ElementId,
    pub label: String,
    pub selected: bool,
    pub disabled: bool,
}

impl ContextMenu {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            position: Point::ZERO,
            entries: Vec::new(),
        }
    }

    pub fn at(mut self, position: Point) -> Self {
        self.position = position;
        self
    }

    pub fn item(self, id: impl Into<ElementId>, label: impl Into<String>) -> Self {
        self.item_with(ContextMenuItem::new(id, label))
    }

    pub fn disabled_item(self, id: impl Into<ElementId>, label: impl Into<String>) -> Self {
        self.item_with(ContextMenuItem::new(id, label).disabled(true))
    }

    pub fn item_with(mut self, item: ContextMenuItem) -> Self {
        self.entries.push(ContextMenuEntry::Item(item));
        self
    }

    pub fn separator(mut self, id: impl Into<ElementId>) -> Self {
        self.entries
            .push(ContextMenuEntry::Separator { id: id.into() });
        self
    }

    pub fn id(&self) -> &ElementId {
        &self.id
    }

    pub fn position(&self) -> Point {
        self.position
    }

    pub fn position_selector(&self) -> StyleSelector {
        StyleSelector::id(self.id.as_str())
    }

    pub fn position_style(&self) -> Style {
        Style::default()
            .absolute_viewport()
            .left(Length::Px(self.position.x))
            .top(Length::Px(self.position.y))
    }

    pub fn render(&self, ui: &mut DocumentBuilder) {
        ui.element(
            self.id.as_str(),
            ElementSpec::new(Element::Div)
                .class(CONTEXT_MENU_CLASS)
                .interactive(),
            |ui| {
                for entry in &self.entries {
                    match entry {
                        ContextMenuEntry::Item(item) => item.render(ui),
                        ContextMenuEntry::Separator { id } => {
                            ui.element(
                                id.as_str(),
                                ElementSpec::new(Element::Div).class(CONTEXT_MENU_SEPARATOR_CLASS),
                                |_| {},
                            );
                        }
                    }
                }
            },
        );
    }
}

impl ContextMenuItem {
    pub fn new(id: impl Into<ElementId>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            selected: false,
            disabled: false,
        }
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    fn render(&self, ui: &mut DocumentBuilder) {
        let mut spec = ElementSpec::new(Element::Button)
            .class(CONTEXT_MENU_ITEM_CLASS)
            .selected(self.selected)
            .disabled(self.disabled);
        if !self.disabled {
            spec = spec.interactive();
        }
        ui.element(self.id.as_str(), spec, |ui| {
            ui.text_element(
                format!("{}-label", self.id.as_str()),
                ElementSpec::new(Element::Text)
                    .class(CONTEXT_MENU_LABEL_CLASS)
                    .selected(self.selected)
                    .disabled(self.disabled),
                self.label.as_str(),
            );
        });
    }
}

pub fn context_menu_surface_style() -> Style {
    Style::default()
        .width(Length::Px(142.0))
        .height(Length::Auto)
        .padding(Insets::symmetric(6.0, 5.0))
        .gap(4.0)
        .z_index(2000)
}

#[cfg(test)]
mod tests {
    use super::*;
    use des_ui_document::{Document, DocumentEngine, Size, StyleSheet};

    #[test]
    fn context_menu_renders_menu_items_and_separators() {
        let menu = ContextMenu::new("text-context-menu")
            .item("copy", "Copy")
            .separator("primary-separator")
            .disabled_item("paste", "Paste");
        let mut document = Document::build(Size::new(240.0, 140.0), |ui| menu.render(ui));

        let output = DocumentEngine::default().update(&mut document, &StyleSheet::new());

        let menu_frame = output.snapshot().find("text-context-menu").unwrap();
        assert!(menu_frame.has_class("context-menu"));
        assert!(menu_frame.interactive());

        let copy = output.snapshot().find("copy").unwrap();
        assert!(copy.has_class("context-menu-item"));
        assert!(copy.interactive());

        let paste = output.snapshot().find("paste").unwrap();
        assert!(paste.has_class("context-menu-item"));
        assert!(!paste.interactive());

        assert!(
            output
                .snapshot()
                .find("primary-separator")
                .unwrap()
                .has_class("context-menu-separator")
        );
    }
}
