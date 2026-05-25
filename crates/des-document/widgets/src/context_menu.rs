use des_document::{
    DocumentBuilder, DocumentWidget, Element, ElementId, ElementSpec, FloatingPlacement,
    FloatingShift, Insets, Length, Point, Style, StyleSelector, StyleSheet,
};

pub const CONTEXT_MENU_CLASS: &str = "context-menu";
pub const CONTEXT_MENU_ITEM_CLASS: &str = "context-menu-item";
pub const CONTEXT_MENU_LABEL_CLASS: &str = "context-menu-label";
pub const CONTEXT_MENU_SEPARATOR_CLASS: &str = "context-menu-separator";

#[derive(Clone, Debug, PartialEq)]
pub struct ContextMenu {
    id: ElementId,
    anchor: ContextMenuAnchor,
    placement: FloatingPlacement,
    offset: Point,
    arrow: Option<ContextMenuArrow>,
    entries: Vec<ContextMenuEntry>,
}

#[derive(Clone, Debug, PartialEq)]
enum ContextMenuAnchor {
    Point(Point),
    Element(ElementId),
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct ContextMenuArrow {
    width: f32,
    height: f32,
    padding: f32,
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
            anchor: ContextMenuAnchor::Point(Point::ZERO),
            placement: FloatingPlacement::BottomStart,
            offset: Point::ZERO,
            arrow: None,
            entries: Vec::new(),
        }
    }

    pub fn at(mut self, position: Point) -> Self {
        self.anchor = ContextMenuAnchor::Point(position);
        self
    }

    pub fn anchored_to(mut self, target: impl Into<ElementId>) -> Self {
        self.anchor = ContextMenuAnchor::Element(target.into());
        self
    }

    pub fn floating_placement(mut self, placement: FloatingPlacement) -> Self {
        self.placement = placement;
        self
    }

    pub fn floating_offset(mut self, main_axis: f32, cross_axis: f32) -> Self {
        self.offset = Point::new(main_axis, cross_axis);
        self
    }

    pub fn floating_arrow(mut self, width: f32, height: f32, padding: f32) -> Self {
        self.arrow = Some(ContextMenuArrow {
            width,
            height,
            padding,
        });
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
        match &self.anchor {
            ContextMenuAnchor::Point(position) => *position,
            ContextMenuAnchor::Element(_) => Point::ZERO,
        }
    }

    pub fn position_selector(&self) -> StyleSelector {
        StyleSelector::id(self.id.as_str())
    }

    pub fn position_style(&self) -> Style {
        let mut style = Style::default()
            .floating_to(self.anchor_id())
            .floating_placement(self.placement)
            .floating_offset(self.offset.x, self.offset.y)
            .floating_flip(true)
            .floating_shift(FloatingShift::main_and_cross_axis());
        if let Some(arrow) = self.arrow {
            style = style.floating_arrow_size(arrow.width, arrow.height, arrow.padding);
        }
        style
    }

    pub fn anchor_selector(&self) -> Option<StyleSelector> {
        match self.anchor {
            ContextMenuAnchor::Point(_) => Some(StyleSelector::id(self.anchor_id().as_str())),
            ContextMenuAnchor::Element(_) => None,
        }
    }

    pub fn anchor_style(&self) -> Option<Style> {
        match self.anchor {
            ContextMenuAnchor::Point(position) => Some(
                Style::default()
                    .absolute_viewport()
                    .left(Length::Px(position.x))
                    .top(Length::Px(position.y))
                    .size(0.0, 0.0),
            ),
            ContextMenuAnchor::Element(_) => None,
        }
    }

    pub fn push_styles(&self, stylesheet: &mut StyleSheet) {
        if let (Some(selector), Some(style)) = (self.anchor_selector(), self.anchor_style()) {
            stylesheet.push_rule(selector, style);
        }
        stylesheet.push_rule(self.position_selector(), self.position_style());
    }

    pub fn render(&self, ui: &mut DocumentBuilder) {
        if matches!(self.anchor, ContextMenuAnchor::Point(_)) {
            let anchor_id = self.anchor_id();
            ui.element(anchor_id.as_str(), ElementSpec::new(Element::Div), |_| {});
        }
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

    fn anchor_id(&self) -> ElementId {
        match &self.anchor {
            ContextMenuAnchor::Point(_) => ElementId::new(format!("{}-anchor", self.id.as_str())),
            ContextMenuAnchor::Element(id) => id.clone(),
        }
    }
}

impl DocumentWidget for ContextMenu {
    fn render(&self, ui: &mut DocumentBuilder) {
        ContextMenu::render(self, ui);
    }

    fn push_styles(&self, stylesheet: &mut StyleSheet) {
        ContextMenu::push_styles(self, stylesheet);
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
    use des_document::{
        Document, DocumentEngine, DocumentView, FloatingPlacement, Size, StyleSheet,
    };

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

    #[test]
    fn context_menu_implements_document_widget_contract() {
        let menu = ContextMenu::new("text-context-menu")
            .at(Point::new(40.0, 24.0))
            .item("copy", "Copy");
        let mut view = DocumentView::build(Size::new(240.0, 140.0), StyleSheet::new(), |ui| {
            ui.widget(&menu);
        });
        view.push_widget_styles(&menu);

        let output = view.update();

        assert!(output.snapshot().find("copy").unwrap().interactive());
        assert_eq!(
            output
                .snapshot()
                .find("text-context-menu-anchor")
                .unwrap()
                .rect()
                .origin,
            Point::new(40.0, 24.0)
        );
    }

    #[test]
    fn context_menu_pushes_floating_styles_for_point_anchor() {
        let menu = ContextMenu::new("text-context-menu")
            .at(Point::new(40.0, 24.0))
            .floating_placement(FloatingPlacement::BottomStart)
            .floating_offset(4.0, 0.0)
            .item("copy", "Copy");
        let mut stylesheet = StyleSheet::new();
        menu.push_styles(&mut stylesheet);
        let mut document = Document::build(Size::new(240.0, 140.0), |ui| menu.render(ui));

        let output = DocumentEngine::default().update(&mut document, &stylesheet);

        let anchor = output.layout.find("text-context-menu-anchor").unwrap();
        assert_eq!(anchor.rect.origin, Point::new(40.0, 24.0));

        let menu_frame = output.layout.find("text-context-menu").unwrap();
        assert_eq!(menu_frame.rect.origin, Point::new(40.0, 28.0));
    }

    #[test]
    fn context_menu_arrow_is_opt_in() {
        let plain = ContextMenu::new("plain-menu").at(Point::new(40.0, 24.0));
        let with_arrow = ContextMenu::new("arrow-menu")
            .at(Point::new(40.0, 24.0))
            .floating_arrow(12.0, 6.0, 3.0);
        let mut stylesheet = StyleSheet::new();
        plain.push_styles(&mut stylesheet);
        with_arrow.push_styles(&mut stylesheet);
        let mut document = Document::build(Size::new(240.0, 140.0), |ui| {
            plain.render(ui);
            with_arrow.render(ui);
        });

        let output = DocumentEngine::default().update(&mut document, &stylesheet);

        assert_eq!(
            output
                .snapshot()
                .find("plain-menu")
                .unwrap()
                .floating()
                .unwrap()
                .arrow_size,
            None
        );
        assert_eq!(
            output
                .snapshot()
                .find("arrow-menu")
                .unwrap()
                .floating()
                .unwrap()
                .arrow_size,
            Some(Size::new(12.0, 6.0))
        );
    }
}
