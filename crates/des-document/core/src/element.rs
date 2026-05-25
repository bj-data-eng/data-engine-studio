use crate::geometry::Point;
use crate::state::{DocumentEventKind, ResolvedElement};
use crate::table::{TableCellSpec, TableSpec};
use crate::text::TextContent;
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Element {
    Root,
    Div,
    Span,
    Main,
    Section,
    Article,
    Header,
    Footer,
    Nav,
    Aside,
    P,
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
    Text,
    Button,
    Input,
    Checkbox,
    Radio,
    Select,
    Option,
    Textarea,
    Label,
    Canvas,
    Icon,
    Table,
    Thead,
    Tbody,
    Tr,
    Th,
    Td,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Glyph {
    Check,
    ChevronDown,
    ChevronUp,
    DragHandle,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ElementStateSelector {
    Hovered,
    Pressed,
    Dragged,
    ScrollbarHovered,
    ScrollbarDragged,
    Focused,
    Selected,
    Disabled,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub(crate) fn lerp(self, target: Self, amount: f32) -> Self {
        fn channel(from: u8, to: u8, amount: f32) -> u8 {
            (from as f32 + (to as f32 - from as f32) * amount)
                .round()
                .clamp(0.0, 255.0) as u8
        }

        Self {
            r: channel(self.r, target.r, amount),
            g: channel(self.g, target.g, amount),
            b: channel(self.b, target.b, amount),
            a: channel(self.a, target.a, amount),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ElementId(String);

impl ElementId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for ElementId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for ElementId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ClassName(String);

impl ClassName {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for ClassName {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for ClassName {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ElementSpec {
    pub element: Element,
    pub classes: Vec<ClassName>,
    pub role: Option<String>,
    pub attributes: BTreeMap<String, String>,
    pub behavior_hooks: Vec<ElementBehaviorHook>,
    pub interactive: bool,
    pub selected: bool,
    pub disabled: bool,
    pub focused: bool,
    pub selectable_text: bool,
    pub copyable_text: bool,
    pub value: Option<String>,
    pub glyph: Option<Glyph>,
    pub table: Option<TableSpec>,
    pub table_cell: Option<TableCellSpec>,
    pub initial_scroll: Option<Point>,
}

impl ElementSpec {
    pub fn new(element: Element) -> Self {
        Self {
            element,
            classes: Vec::new(),
            role: None,
            attributes: BTreeMap::new(),
            behavior_hooks: Vec::new(),
            interactive: false,
            selected: false,
            disabled: false,
            focused: false,
            selectable_text: false,
            copyable_text: false,
            value: None,
            glyph: None,
            table: None,
            table_cell: None,
            initial_scroll: None,
        }
    }

    pub fn class(mut self, class: impl Into<ClassName>) -> Self {
        self.classes.push(class.into());
        self
    }

    pub fn role(mut self, role: impl Into<String>) -> Self {
        self.role = Some(role.into());
        self
    }

    pub fn attribute(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(name.into(), value.into());
        self
    }

    pub fn behavior_hook(mut self, event: impl Into<String>, command: impl Into<String>) -> Self {
        self.behavior_hooks
            .push(ElementBehaviorHook::new(event, command));
        self.interactive = true;
        self
    }

    pub fn on(mut self, event: ElementBehaviorEvent, command: impl Into<String>) -> Self {
        self.behavior_hooks
            .push(ElementBehaviorHook::on(event, command));
        self.interactive = true;
        self
    }

    pub fn on_click(self, command: impl Into<String>) -> Self {
        self.on(ElementBehaviorEvent::Click, command)
    }

    pub fn on_context_menu(self, command: impl Into<String>) -> Self {
        self.on(ElementBehaviorEvent::ContextMenu, command)
    }

    pub fn on_pointer_enter(self, command: impl Into<String>) -> Self {
        self.on(ElementBehaviorEvent::PointerEnter, command)
    }

    pub fn on_pointer_leave(self, command: impl Into<String>) -> Self {
        self.on(ElementBehaviorEvent::PointerLeave, command)
    }

    pub fn on_pointer_down(self, command: impl Into<String>) -> Self {
        self.on(ElementBehaviorEvent::PointerDown, command)
    }

    pub fn on_pointer_up(self, command: impl Into<String>) -> Self {
        self.on(ElementBehaviorEvent::PointerUp, command)
    }

    pub fn on_drag_start(self, command: impl Into<String>) -> Self {
        self.on(ElementBehaviorEvent::DragStart, command)
    }

    pub fn on_drag(self, command: impl Into<String>) -> Self {
        self.on(ElementBehaviorEvent::Drag, command)
    }

    pub fn on_drag_end(self, command: impl Into<String>) -> Self {
        self.on(ElementBehaviorEvent::DragEnd, command)
    }

    pub fn on_scroll(self, command: impl Into<String>) -> Self {
        self.on(ElementBehaviorEvent::Scroll, command)
    }

    pub fn interactive(mut self) -> Self {
        self.interactive = true;
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    pub fn selectable_text(mut self) -> Self {
        self.selectable_text = true;
        self.copyable_text = true;
        self
    }

    pub fn copyable_text(mut self, copyable_text: bool) -> Self {
        self.copyable_text = copyable_text;
        self
    }

    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    pub fn glyph(mut self, glyph: Glyph) -> Self {
        self.glyph = Some(glyph);
        self
    }

    pub fn initial_scroll(mut self, scroll: Point) -> Self {
        self.initial_scroll = Some(Point::new(scroll.x.max(0.0), scroll.y.max(0.0)));
        self
    }

    pub fn initial_scroll_x(mut self, scroll_x: f32) -> Self {
        let scroll = self.initial_scroll.unwrap_or(Point::ZERO);
        self.initial_scroll = Some(Point::new(scroll_x.max(0.0), scroll.y));
        self
    }

    pub fn initial_scroll_y(mut self, scroll_y: f32) -> Self {
        let scroll = self.initial_scroll.unwrap_or(Point::ZERO);
        self.initial_scroll = Some(Point::new(scroll.x, scroll_y.max(0.0)));
        self
    }

    pub fn table(mut self, table: TableSpec) -> Self {
        self.table = Some(table);
        self
    }

    pub fn table_cell(mut self, table_cell: TableCellSpec) -> Self {
        self.table_cell = Some(table_cell);
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ElementBehaviorHook {
    pub event: String,
    pub command: String,
}

impl ElementBehaviorHook {
    pub fn new(event: impl Into<String>, command: impl Into<String>) -> Self {
        Self {
            event: event.into(),
            command: command.into(),
        }
    }

    pub fn on(event: ElementBehaviorEvent, command: impl Into<String>) -> Self {
        Self::new(event.as_str(), command)
    }

    pub fn matches_document_event(&self, event: &DocumentEventKind) -> bool {
        ElementBehaviorEvent::from_name(&self.event)
            .is_some_and(|hook_event| hook_event.matches_document_event(event))
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ElementBehaviorEvent {
    Click,
    ContextMenu,
    PointerEnter,
    PointerLeave,
    PointerDown,
    PointerUp,
    DragStart,
    Drag,
    DragEnd,
    Scroll,
}

impl ElementBehaviorEvent {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Click => "click",
            Self::ContextMenu => "contextmenu",
            Self::PointerEnter => "pointerenter",
            Self::PointerLeave => "pointerleave",
            Self::PointerDown => "pointerdown",
            Self::PointerUp => "pointerup",
            Self::DragStart => "dragstart",
            Self::Drag => "drag",
            Self::DragEnd => "dragend",
            Self::Scroll => "scroll",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name.trim().to_ascii_lowercase().as_str() {
            "click" => Some(Self::Click),
            "contextmenu" => Some(Self::ContextMenu),
            "pointerenter" => Some(Self::PointerEnter),
            "pointerleave" | "pointerout" => Some(Self::PointerLeave),
            "pointerdown" => Some(Self::PointerDown),
            "pointerup" => Some(Self::PointerUp),
            "dragstart" => Some(Self::DragStart),
            "drag" => Some(Self::Drag),
            "dragend" => Some(Self::DragEnd),
            "scroll" | "scrollx" | "scroll-x" | "scrolly" | "scroll-y" => Some(Self::Scroll),
            _ => None,
        }
    }

    pub fn matches_document_event(self, event: &DocumentEventKind) -> bool {
        matches!(
            (self, event),
            (Self::Click, DocumentEventKind::Clicked)
                | (Self::ContextMenu, DocumentEventKind::ContextRequested)
                | (Self::PointerEnter, DocumentEventKind::PointerEntered)
                | (Self::PointerLeave, DocumentEventKind::PointerExited)
                | (Self::PointerDown, DocumentEventKind::Pressed)
                | (Self::PointerUp, DocumentEventKind::Released)
                | (Self::DragStart, DocumentEventKind::DragStarted)
                | (Self::Drag, DocumentEventKind::DragMoved)
                | (Self::DragEnd, DocumentEventKind::DragEnded)
                | (Self::Scroll, DocumentEventKind::Scrolled(_))
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct DocumentNode {
    pub id: ElementId,
    pub spec: ElementSpec,
    pub text: Option<TextContent>,
    pub children: Vec<DocumentNode>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct VisualElementClone {
    pub source_id: ElementId,
    pub element: Element,
    pub classes: Vec<ClassName>,
    pub role: Option<String>,
    pub attributes: BTreeMap<String, String>,
    pub behavior_hooks: Vec<ElementBehaviorHook>,
    pub text: Option<TextContent>,
    pub value: Option<String>,
    pub glyph: Option<Glyph>,
    pub children: Vec<VisualElementClone>,
}

impl VisualElementClone {
    pub fn from_resolved(element: &ResolvedElement) -> Self {
        Self {
            source_id: element.id.clone(),
            element: element.element,
            classes: element.classes.clone(),
            role: element.role.clone(),
            attributes: element.attributes.clone(),
            behavior_hooks: element.behavior_hooks.clone(),
            text: element.text.clone(),
            value: element.value.clone(),
            glyph: element.glyph,
            children: element.children.iter().map(Self::from_resolved).collect(),
        }
    }

    pub fn cloned_ids(&self, options: &VisualCloneOptions) -> Vec<ElementId> {
        let mut ids = Vec::new();
        self.collect_cloned_ids(options, true, &mut ids);
        ids
    }

    pub fn source_ids(&self) -> Vec<ElementId> {
        let mut ids = Vec::new();
        self.collect_source_ids(&mut ids);
        ids
    }

    fn collect_cloned_ids(
        &self,
        options: &VisualCloneOptions,
        is_root: bool,
        ids: &mut Vec<ElementId>,
    ) {
        ids.push(self.clone_id(options, is_root));
        for child in &self.children {
            child.collect_cloned_ids(options, false, ids);
        }
    }

    fn collect_source_ids(&self, ids: &mut Vec<ElementId>) {
        ids.push(self.source_id.clone());
        for child in &self.children {
            child.collect_source_ids(ids);
        }
    }

    pub(crate) fn to_element(&self, options: &VisualCloneOptions, is_root: bool) -> DocumentNode {
        let mut spec = ElementSpec::new(self.element);
        spec.classes = self.classes.clone();
        spec.role = self.role.clone();
        spec.attributes = self.attributes.clone();
        spec.behavior_hooks = self.behavior_hooks.clone();
        if is_root {
            spec.classes.extend(options.root_classes.iter().cloned());
        }
        spec.value = self.value.clone();
        spec.glyph = self.glyph;
        spec.interactive = options.interactive;

        DocumentNode {
            id: self.clone_id(options, is_root),
            spec,
            text: self.text.clone(),
            children: self
                .children
                .iter()
                .map(|child| child.to_element(options, false))
                .collect(),
        }
    }

    fn clone_id(&self, options: &VisualCloneOptions, is_root: bool) -> ElementId {
        if is_root {
            return options.root_id.clone();
        }
        ElementId::new(format!("{}{}", options.id_prefix, self.source_id.as_str()))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct VisualCloneOptions {
    pub root_id: ElementId,
    pub id_prefix: String,
    pub root_classes: Vec<ClassName>,
    pub interactive: bool,
}

impl VisualCloneOptions {
    pub fn new(root_id: impl Into<ElementId>, id_prefix: impl Into<String>) -> Self {
        Self {
            root_id: root_id.into(),
            id_prefix: id_prefix.into(),
            root_classes: Vec::new(),
            interactive: false,
        }
    }

    pub fn root_class(mut self, class: impl Into<ClassName>) -> Self {
        self.root_classes.push(class.into());
        self
    }

    pub fn interactive(mut self, interactive: bool) -> Self {
        self.interactive = interactive;
        self
    }
}
