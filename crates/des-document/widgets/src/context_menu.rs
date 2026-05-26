use des_document::{
    CssParseError, DocumentActionSurface, DocumentAuthoringResult, DocumentBuilder,
    DocumentCommandRegistry, DocumentResult, DocumentView, DocumentWidget, ElementId,
    FloatingPlacement, FloatingShift, Insets, Length, Point, Size, Style, StyleSelector,
    StyleSheet,
};
use std::collections::BTreeMap;

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
    pub command: Option<String>,
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

    pub fn item_if(
        self,
        id: impl Into<ElementId>,
        label: impl Into<String>,
        present: bool,
    ) -> Self {
        self.item_with_if(ContextMenuItem::new(id, label), present)
    }

    pub fn command_item(
        self,
        id: impl Into<ElementId>,
        label: impl Into<String>,
        command: impl Into<String>,
    ) -> Self {
        self.item_with(ContextMenuItem::new(id, label).command(command))
    }

    pub fn command_item_if(
        self,
        id: impl Into<ElementId>,
        label: impl Into<String>,
        command: impl Into<String>,
        present: bool,
    ) -> Self {
        self.item_with_if(ContextMenuItem::new(id, label).command(command), present)
    }

    pub fn selected_item(self, id: impl Into<ElementId>, label: impl Into<String>) -> Self {
        self.item_with(ContextMenuItem::new(id, label).selected(true))
    }

    pub fn selected_item_if(
        self,
        id: impl Into<ElementId>,
        label: impl Into<String>,
        present: bool,
    ) -> Self {
        self.item_with_if(ContextMenuItem::new(id, label).selected(true), present)
    }

    pub fn disabled_item(self, id: impl Into<ElementId>, label: impl Into<String>) -> Self {
        self.item_with(ContextMenuItem::new(id, label).disabled(true))
    }

    pub fn disabled_item_if(
        self,
        id: impl Into<ElementId>,
        label: impl Into<String>,
        present: bool,
    ) -> Self {
        self.item_with_if(ContextMenuItem::new(id, label).disabled(true), present)
    }

    pub fn item_with(mut self, item: ContextMenuItem) -> Self {
        self.entries.push(ContextMenuEntry::Item(item));
        self
    }

    pub fn item_with_if(mut self, item: ContextMenuItem, present: bool) -> Self {
        if present {
            self.entries.push(ContextMenuEntry::Item(item));
        }
        self
    }

    pub fn items<I>(mut self, items: I) -> Self
    where
        I: IntoIterator<Item = ContextMenuItem>,
    {
        self.entries
            .extend(items.into_iter().map(ContextMenuEntry::Item));
        self
    }

    pub fn items_if<I>(self, items: I, present: bool) -> Self
    where
        I: IntoIterator<Item = ContextMenuItem>,
    {
        if present { self.items(items) } else { self }
    }

    pub fn separator(mut self, id: impl Into<ElementId>) -> Self {
        self.entries
            .push(ContextMenuEntry::Separator { id: id.into() });
        self
    }

    pub fn separator_if(mut self, id: impl Into<ElementId>, present: bool) -> Self {
        if present {
            self.entries
                .push(ContextMenuEntry::Separator { id: id.into() });
        }
        self
    }

    pub fn id(&self) -> &ElementId {
        &self.id
    }

    pub fn entries(&self) -> &[ContextMenuEntry] {
        &self.entries
    }

    /// Builds a typed command registry from enabled command items.
    ///
    /// The closure receives each enabled item with a command and can return the
    /// app action that should be dispatched for that item.
    pub fn command_registry<Action>(
        &self,
        action_for: impl FnMut(&ContextMenuItem) -> Option<Action>,
    ) -> DocumentCommandRegistry<Action> {
        let mut registry = DocumentCommandRegistry::new();
        self.push_commands(&mut registry, action_for);
        registry
    }

    /// Pushes typed actions for menu command names into a registry.
    pub fn push_command_actions<Action, Command>(
        &self,
        registry: &mut DocumentCommandRegistry<Action>,
        actions: impl IntoIterator<Item = (Command, Action)>,
    ) where
        Action: Clone,
        Command: AsRef<str>,
    {
        let actions = actions
            .into_iter()
            .map(|(command, action)| (command.as_ref().to_owned(), action))
            .collect::<BTreeMap<_, _>>();
        self.push_commands(registry, |item| actions.get(item.command_name()?).cloned());
    }

    /// Builds a typed command registry from `(command, action)` pairs.
    pub fn command_action_registry<Action, Command>(
        &self,
        actions: impl IntoIterator<Item = (Command, Action)>,
    ) -> DocumentCommandRegistry<Action>
    where
        Action: Clone,
        Command: AsRef<str>,
    {
        let mut registry = DocumentCommandRegistry::new();
        self.push_command_actions(&mut registry, actions);
        registry
    }

    /// Builds a ready-to-drive menu action surface from this widget.
    pub fn action_surface<Action>(
        &self,
        viewport: Size,
        action_for: impl FnMut(&ContextMenuItem) -> Option<Action>,
    ) -> DocumentActionSurface<Action> {
        self.action_surface_with_stylesheet(viewport, StyleSheet::new(), action_for)
    }

    /// Builds a menu action surface with caller-provided stylesheet rules.
    pub fn action_surface_with_stylesheet<Action>(
        &self,
        viewport: Size,
        stylesheet: StyleSheet,
        action_for: impl FnMut(&ContextMenuItem) -> Option<Action>,
    ) -> DocumentActionSurface<Action> {
        self.try_action_surface_with_stylesheet(viewport, stylesheet, action_for)
            .expect("context menu projection targets rendered elements")
    }

    /// Builds a menu action surface with strict CSS rules.
    pub fn action_surface_with_css<Action>(
        &self,
        viewport: Size,
        css: &str,
        action_for: impl FnMut(&ContextMenuItem) -> Option<Action>,
    ) -> Result<DocumentActionSurface<Action>, CssParseError> {
        let stylesheet = StyleSheet::from_css(css)?;
        Ok(self.action_surface_with_stylesheet(viewport, stylesheet, action_for))
    }

    /// Builds a menu action surface with browser-forgiving CSS rules.
    pub fn action_surface_with_css_forgiving<Action>(
        &self,
        viewport: Size,
        css: &str,
        action_for: impl FnMut(&ContextMenuItem) -> Option<Action>,
    ) -> Result<DocumentActionSurface<Action>, CssParseError> {
        let stylesheet = StyleSheet::from_css_forgiving(css)?;
        Ok(self.action_surface_with_stylesheet(viewport, stylesheet, action_for))
    }

    /// Tries to build a menu action surface, returning document projection errors.
    pub fn try_action_surface<Action>(
        &self,
        viewport: Size,
        action_for: impl FnMut(&ContextMenuItem) -> Option<Action>,
    ) -> DocumentResult<DocumentActionSurface<Action>> {
        self.try_action_surface_with_stylesheet(viewport, StyleSheet::new(), action_for)
    }

    /// Tries to build a menu action surface with caller-provided stylesheet rules.
    pub fn try_action_surface_with_stylesheet<Action>(
        &self,
        viewport: Size,
        stylesheet: StyleSheet,
        action_for: impl FnMut(&ContextMenuItem) -> Option<Action>,
    ) -> DocumentResult<DocumentActionSurface<Action>> {
        let view = DocumentView::compose(viewport)
            .stylesheet(stylesheet)
            .try_widget(self)?;
        Ok(view.action_surface(self.command_registry(action_for)))
    }

    /// Tries to build a menu action surface with strict CSS rules.
    pub fn try_action_surface_with_css<Action>(
        &self,
        viewport: Size,
        css: &str,
        action_for: impl FnMut(&ContextMenuItem) -> Option<Action>,
    ) -> DocumentAuthoringResult<DocumentActionSurface<Action>> {
        let stylesheet = StyleSheet::from_css(css)?;
        Ok(self.try_action_surface_with_stylesheet(viewport, stylesheet, action_for)?)
    }

    /// Tries to build a menu action surface with browser-forgiving CSS rules.
    pub fn try_action_surface_with_css_forgiving<Action>(
        &self,
        viewport: Size,
        css: &str,
        action_for: impl FnMut(&ContextMenuItem) -> Option<Action>,
    ) -> DocumentAuthoringResult<DocumentActionSurface<Action>> {
        let stylesheet = StyleSheet::from_css_forgiving(css)?;
        Ok(self.try_action_surface_with_stylesheet(viewport, stylesheet, action_for)?)
    }

    /// Pushes typed command bindings for enabled command items into a registry.
    pub fn push_commands<Action>(
        &self,
        registry: &mut DocumentCommandRegistry<Action>,
        mut action_for: impl FnMut(&ContextMenuItem) -> Option<Action>,
    ) {
        for entry in &self.entries {
            let ContextMenuEntry::Item(item) = entry else {
                continue;
            };
            if item.is_disabled() {
                continue;
            }
            let Some(command) = item.command_name() else {
                continue;
            };
            let Some(action) = action_for(item) else {
                continue;
            };
            registry.push_click(command, action);
        }
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
        if let Some(style) = self.anchor_style() {
            let anchor_id = self.anchor_id();
            stylesheet.push_id(anchor_id.as_str(), style);
        }
        stylesheet.push_id(self.id.as_str(), self.position_style());
    }

    pub fn render(&self, ui: &mut DocumentBuilder) {
        if matches!(self.anchor, ContextMenuAnchor::Point(_)) {
            let anchor_id = self.anchor_id();
            ui.div(anchor_id.as_str()).empty();
        }
        ui.div(self.id.as_str())
            .class(CONTEXT_MENU_CLASS)
            .interactive()
            .children(|ui| {
                for entry in &self.entries {
                    match entry {
                        ContextMenuEntry::Item(item) => item.render(ui),
                        ContextMenuEntry::Separator { id } => {
                            ui.div(id.as_str())
                                .class(CONTEXT_MENU_SEPARATOR_CLASS)
                                .empty();
                        }
                    }
                }
            });
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
            command: None,
            selected: false,
            disabled: false,
        }
    }

    pub fn command(mut self, command: impl Into<String>) -> Self {
        self.command = Some(command.into());
        self
    }

    pub fn id(&self) -> &ElementId {
        &self.id
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn command_name(&self) -> Option<&str> {
        self.command.as_deref()
    }

    pub fn is_selected(&self) -> bool {
        self.selected
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
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
        ui.button(self.id.as_str())
            .class(CONTEXT_MENU_ITEM_CLASS)
            .selected(self.selected)
            .disabled(self.disabled)
            .interactive_if(!self.disabled)
            .on_click_if(
                self.command.as_deref().unwrap_or_default(),
                !self.disabled && self.command.is_some(),
            )
            .children(|ui| {
                ui.text_node(format!("{}-label", self.id.as_str()))
                    .class(CONTEXT_MENU_LABEL_CLASS)
                    .selected(self.selected)
                    .disabled(self.disabled)
                    .text(self.label.as_str());
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
        Document, DocumentCommandRegistry, DocumentEngine, DocumentInput, DocumentView,
        FloatingPlacement, Size, StyleSheet,
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
    fn context_menu_supports_fluent_conditional_entries() {
        let include_debug = true;
        let include_hidden = false;
        let menu = ContextMenu::new("row-menu")
            .item("copy", "Copy")
            .item_if("hidden-item", "Hidden", include_hidden)
            .command_item_if("rename", "Rename", "rename-row", include_debug)
            .selected_item("sort-ascending", "Sort ascending")
            .selected_item_if("hidden-selected", "Hidden selected", include_hidden)
            .disabled_item_if("paste", "Paste", include_debug)
            .separator_if("debug-separator", include_debug)
            .separator_if("hidden-separator", include_hidden)
            .item_with_if(
                ContextMenuItem::new("inspect", "Inspect").command("inspect-row"),
                include_debug,
            )
            .items_if(
                [
                    ContextMenuItem::new("duplicate", "Duplicate").command("duplicate-row"),
                    ContextMenuItem::new("delete", "Delete").disabled(true),
                ],
                include_debug,
            )
            .items_if(
                [ContextMenuItem::new("hidden-batch", "Hidden batch")],
                include_hidden,
            );
        let mut view = DocumentView::compose(Size::new(260.0, 180.0)).widget(&menu);

        let output = view.update();
        let snapshot = output.snapshot();

        assert_eq!(menu.entries().len(), 8);
        assert!(snapshot.find("copy").is_some());
        assert!(snapshot.find("rename").unwrap().interactive());
        assert!(snapshot.find("sort-ascending").unwrap().selected());
        assert!(!snapshot.find("paste").unwrap().interactive());
        assert!(snapshot.find("debug-separator").is_some());
        assert!(snapshot.find("inspect").unwrap().interactive());
        assert!(snapshot.find("duplicate").unwrap().interactive());
        assert!(!snapshot.find("delete").unwrap().interactive());
        assert!(snapshot.find("hidden-item").is_none());
        assert!(snapshot.find("hidden-selected").is_none());
        assert!(snapshot.find("hidden-separator").is_none());
        assert!(snapshot.find("hidden-batch").is_none());
    }

    #[test]
    fn context_menu_implements_document_widget_contract() {
        let menu = ContextMenu::new("text-context-menu")
            .at(Point::new(40.0, 24.0))
            .item("copy", "Copy");
        let mut view = DocumentView::compose(Size::new(240.0, 140.0)).widget(&menu);

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
    fn context_menu_items_can_declare_document_commands() {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        enum MenuAction {
            Copy,
        }

        let menu = ContextMenu::new("text-context-menu")
            .command_item("copy", "Copy", "copy-selection")
            .disabled_item("paste", "Paste");
        let mut view = DocumentView::build(Size::new(240.0, 140.0), StyleSheet::new(), |ui| {
            ui.widget(&menu);
        });
        let output = view.update_with_input(DocumentInput::primary_click(Point::new(2.0, 2.0)));
        let registry = DocumentCommandRegistry::new().bind("copy-selection", MenuAction::Copy);
        let mut actions = Vec::new();
        let report = registry.dispatch(&output, |command| {
            actions.push((command.target.clone(), *command.action));
        });

        assert_eq!(report.commands, 1);
        assert_eq!(report.handled, 1);
        assert_eq!(report.unhandled, 0);
        assert_eq!(actions, vec![(ElementId::new("copy"), MenuAction::Copy)]);
        assert_eq!(output.commands()[0].command, "copy-selection");
        assert!(
            output
                .snapshot()
                .find("paste")
                .unwrap()
                .behavior_hooks()
                .is_empty()
        );
    }

    #[test]
    fn context_menu_builds_typed_command_registry_from_items() {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        enum MenuAction {
            Copy,
            Rename,
        }

        let menu = ContextMenu::new("row-menu")
            .command_item("copy", "Copy", "copy-selection")
            .command_item("rename", "Rename", "rename-selection")
            .disabled_item("paste", "Paste");
        let registry = menu.command_registry(|item| match item.id().as_str() {
            "copy" => Some(MenuAction::Copy),
            "rename" => Some(MenuAction::Rename),
            _ => None,
        });
        let mut pushed = DocumentCommandRegistry::new();
        menu.push_commands(&mut pushed, |item| match item.command_name() {
            Some("copy-selection") => Some(MenuAction::Copy),
            Some("rename-selection") => Some(MenuAction::Rename),
            _ => None,
        });
        let mapped_registry = menu.command_action_registry([
            ("copy-selection", MenuAction::Copy),
            ("rename-selection", MenuAction::Rename),
        ]);
        let mut pushed_mapped = DocumentCommandRegistry::new();
        menu.push_command_actions(
            &mut pushed_mapped,
            [
                ("copy-selection", MenuAction::Copy),
                ("rename-selection", MenuAction::Rename),
            ],
        );
        let mut view = DocumentView::compose(Size::new(240.0, 140.0)).widget(&menu);

        let frame = view.update_with_input_actions(
            DocumentInput::primary_click(Point::new(2.0, 2.0)),
            &registry,
        );
        let mapped_frame = view.update_with_input_actions(
            DocumentInput::primary_click(Point::new(2.0, 2.0)),
            &mapped_registry,
        );

        assert_eq!(menu.entries().len(), 3);
        assert_eq!(pushed.bindings().len(), 2);
        assert_eq!(mapped_registry.bindings().len(), 2);
        assert_eq!(pushed_mapped.bindings().len(), 2);
        assert!(frame.contains_action(&MenuAction::Copy));
        assert!(mapped_frame.contains_action_for_intent(
            des_document::ElementBehaviorEvent::Click,
            &MenuAction::Copy
        ));
        assert!(!frame.contains_action(&MenuAction::Rename));
        assert_eq!(
            frame
                .first_action_for_intent(des_document::ElementBehaviorEvent::Click)
                .unwrap()
                .target()
                .as_str(),
            "copy"
        );
    }

    #[test]
    fn context_menu_builds_action_surface_from_items() {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        enum MenuAction {
            Copy,
            Rename,
        }
        fn action_for(item: &ContextMenuItem) -> Option<MenuAction> {
            match item.command_name() {
                Some("copy-selection") => Some(MenuAction::Copy),
                Some("rename-selection") => Some(MenuAction::Rename),
                _ => None,
            }
        }

        let menu = ContextMenu::new("row-menu")
            .command_item("copy", "Copy", "copy-selection")
            .command_item("rename", "Rename", "rename-selection")
            .disabled_item("paste", "Paste");
        let mut surface = menu.action_surface(Size::new(240.0, 140.0), action_for);
        let mut mapped_surface = menu.action_surface(Size::new(240.0, 140.0), action_for);
        let mut css_surface = menu
            .action_surface_with_css(
                Size::new(240.0, 140.0),
                "#copy { height: 34px; }",
                action_for,
            )
            .expect("strict CSS should create a mapped context menu action surface");
        let mut css_mapped_surface = menu
            .action_surface_with_css(
                Size::new(240.0, 140.0),
                "#copy { height: 35px; }",
                action_for,
            )
            .expect("strict CSS should create a command/action context menu surface");
        let mut forgiving_css_surface = menu
            .try_action_surface_with_css_forgiving(
                Size::new(240.0, 140.0),
                ".ignored { unknown-property: yes; } #copy { height: 37px; }",
                action_for,
            )
            .expect("forgiving CSS should create a context menu action surface");
        let mut forgiving_css_mapped_surface = menu
            .try_action_surface_with_css_forgiving(
                Size::new(240.0, 140.0),
                ".ignored { unknown-property: yes; } #copy { height: 38px; }",
                action_for,
            )
            .expect("forgiving CSS should create a mapped context menu surface");

        let frame =
            surface.update_with_input_actions(DocumentInput::primary_click(Point::new(2.0, 2.0)));
        let mapped_frame = mapped_surface
            .update_with_input_actions(DocumentInput::primary_click(Point::new(2.0, 2.0)));
        let css_frame = css_surface
            .update_with_input_actions(DocumentInput::primary_click(Point::new(2.0, 2.0)));
        let css_mapped_frame = css_mapped_surface
            .update_with_input_actions(DocumentInput::primary_click(Point::new(2.0, 2.0)));
        let forgiving_css_frame = forgiving_css_surface
            .update_with_input_actions(DocumentInput::primary_click(Point::new(2.0, 2.0)));
        let forgiving_css_mapped_frame = forgiving_css_mapped_surface
            .update_with_input_actions(DocumentInput::primary_click(Point::new(2.0, 2.0)));
        let action_values: Vec<_> = frame.action_values().copied().collect();
        let mapped_action_values: Vec<_> = mapped_frame.action_values().copied().collect();
        let mut dispatch_surface = menu.action_surface(Size::new(240.0, 140.0), action_for);
        let mut dispatched = Vec::new();
        let dispatch_frame = dispatch_surface
            .update_with_input_actions(DocumentInput::primary_click(Point::new(2.0, 2.0)));
        let dispatch_report = dispatch_frame.dispatch(|action| {
            dispatched.push(*action.action());
        });
        let mut value_dispatch_surface = menu.action_surface(Size::new(240.0, 140.0), action_for);
        let mut dispatched_values = Vec::new();
        let value_dispatch_frame = value_dispatch_surface
            .update_with_input_actions(DocumentInput::primary_click(Point::new(2.0, 2.0)));
        let value_dispatch_report =
            value_dispatch_frame.dispatch_action_values(|action| dispatched_values.push(*action));
        let mut empty_surface = menu.action_surface(Size::new(240.0, 140.0), action_for);
        let empty_frame = empty_surface.update_actions();
        let copy = frame.output().snapshot().find("copy").unwrap();
        let paste = frame.output().snapshot().find("paste").unwrap();

        assert_eq!(surface.commands().bindings().len(), 2);
        assert_eq!(mapped_surface.commands().bindings().len(), 2);
        assert_eq!(css_surface.commands().bindings().len(), 2);
        assert_eq!(css_mapped_surface.commands().bindings().len(), 2);
        assert_eq!(forgiving_css_surface.commands().bindings().len(), 2);
        assert_eq!(forgiving_css_mapped_surface.commands().bindings().len(), 2);
        assert!(copy.has_class(CONTEXT_MENU_ITEM_CLASS));
        assert!(!paste.interactive());
        assert!(frame.contains_action_for_intent(
            des_document::ElementBehaviorEvent::Click,
            &MenuAction::Copy
        ));
        assert!(mapped_frame.contains_action_for_intent(
            des_document::ElementBehaviorEvent::Click,
            &MenuAction::Copy
        ));
        assert!(css_frame.contains_action_for_intent(
            des_document::ElementBehaviorEvent::Click,
            &MenuAction::Copy
        ));
        assert_eq!(
            css_frame
                .output()
                .snapshot()
                .find("copy")
                .unwrap()
                .rect()
                .size
                .height,
            34.0
        );
        assert!(css_mapped_frame.contains_action_for_intent(
            des_document::ElementBehaviorEvent::Click,
            &MenuAction::Copy
        ));
        assert_eq!(
            css_mapped_frame
                .output()
                .snapshot()
                .find("copy")
                .unwrap()
                .rect()
                .size
                .height,
            35.0
        );
        assert!(forgiving_css_frame.contains_action_for_intent(
            des_document::ElementBehaviorEvent::Click,
            &MenuAction::Copy
        ));
        assert_eq!(
            forgiving_css_frame
                .output()
                .snapshot()
                .find("copy")
                .unwrap()
                .rect()
                .size
                .height,
            37.0
        );
        assert!(forgiving_css_mapped_frame.contains_action_for_intent(
            des_document::ElementBehaviorEvent::Click,
            &MenuAction::Copy
        ));
        assert_eq!(
            forgiving_css_mapped_frame
                .output()
                .snapshot()
                .find("copy")
                .unwrap()
                .rect()
                .size
                .height,
            38.0
        );
        assert_eq!(action_values, vec![MenuAction::Copy]);
        assert_eq!(mapped_action_values, vec![MenuAction::Copy]);
        assert!(dispatch_frame.contains_action_for_intent(
            des_document::ElementBehaviorEvent::Click,
            &MenuAction::Copy
        ));
        assert_eq!(
            dispatch_report,
            des_document::DocumentCommandDispatchReport::new(1, 1, 0)
        );
        assert_eq!(dispatched, vec![MenuAction::Copy]);
        assert!(value_dispatch_frame.contains_action_for_intent(
            des_document::ElementBehaviorEvent::Click,
            &MenuAction::Copy
        ));
        assert_eq!(
            value_dispatch_report,
            des_document::DocumentCommandDispatchReport::new(1, 1, 0)
        );
        assert_eq!(dispatched_values, vec![MenuAction::Copy]);
        assert!(empty_frame.is_empty());
        assert!(!frame.contains_action(&MenuAction::Rename));
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
