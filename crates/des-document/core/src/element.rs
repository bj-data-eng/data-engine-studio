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

macro_rules! element_spec_constructors {
    ($($name:ident => $element:expr),+ $(,)?) => {
        $(
            pub fn $name() -> Self {
                Self::new($element)
            }
        )+
    };
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

    element_spec_constructors! {
        root => Element::Root,
        div => Element::Div,
        span => Element::Span,
        main => Element::Main,
        section => Element::Section,
        article => Element::Article,
        header => Element::Header,
        footer => Element::Footer,
        nav => Element::Nav,
        aside => Element::Aside,
        p => Element::P,
        h1 => Element::H1,
        h2 => Element::H2,
        h3 => Element::H3,
        h4 => Element::H4,
        h5 => Element::H5,
        h6 => Element::H6,
        text => Element::Text,
        button => Element::Button,
        input => Element::Input,
        checkbox => Element::Checkbox,
        radio => Element::Radio,
        select => Element::Select,
        option => Element::Option,
        textarea => Element::Textarea,
        label => Element::Label,
        canvas => Element::Canvas,
        icon => Element::Icon,
        table_element => Element::Table,
        thead => Element::Thead,
        tbody => Element::Tbody,
        tr => Element::Tr,
        th => Element::Th,
        td => Element::Td,
    }

    pub fn class(mut self, class: impl Into<ClassName>) -> Self {
        self.classes.push(class.into());
        self
    }

    pub fn classes<I, C>(mut self, classes: I) -> Self
    where
        I: IntoIterator<Item = C>,
        C: Into<ClassName>,
    {
        self.classes.extend(classes.into_iter().map(Into::into));
        self
    }

    pub fn class_if(self, class: impl Into<ClassName>, present: bool) -> Self {
        if present { self.class(class) } else { self }
    }

    pub fn classes_if<I, C>(self, classes: I, present: bool) -> Self
    where
        I: IntoIterator<Item = C>,
        C: Into<ClassName>,
    {
        if present { self.classes(classes) } else { self }
    }

    pub fn role(mut self, role: impl Into<String>) -> Self {
        self.role = Some(role.into());
        self
    }

    pub fn attribute(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(name.into(), value.into());
        self
    }

    pub fn attribute_if(
        self,
        name: impl Into<String>,
        value: impl Into<String>,
        present: bool,
    ) -> Self {
        if present {
            self.attribute(name, value)
        } else {
            self
        }
    }

    pub fn attributes<I, K, V>(mut self, attributes: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        self.attributes.extend(
            attributes
                .into_iter()
                .map(|(name, value)| (name.into(), value.into())),
        );
        self
    }

    pub fn data(self, name: impl AsRef<str>, value: impl Into<String>) -> Self {
        self.attribute(prefixed_attribute_name("data-", name), value)
    }

    pub fn data_if(self, name: impl AsRef<str>, value: impl Into<String>, present: bool) -> Self {
        if present {
            self.data(name, value)
        } else {
            self
        }
    }

    pub fn aria(self, name: impl AsRef<str>, value: impl Into<String>) -> Self {
        self.attribute(prefixed_attribute_name("aria-", name), value)
    }

    pub fn aria_if(self, name: impl AsRef<str>, value: impl Into<String>, present: bool) -> Self {
        if present {
            self.aria(name, value)
        } else {
            self
        }
    }

    pub fn behavior_hook(mut self, event: impl Into<String>, command: impl Into<String>) -> Self {
        self.behavior_hooks
            .push(ElementBehaviorHook::new(event, command));
        self.interactive = true;
        self
    }

    pub fn behavior_hooks<I, H>(mut self, hooks: I) -> Self
    where
        I: IntoIterator<Item = H>,
        H: Into<ElementBehaviorHook>,
    {
        let previous_len = self.behavior_hooks.len();
        self.behavior_hooks
            .extend(hooks.into_iter().map(Into::into));
        self.interactive |= self.behavior_hooks.len() > previous_len;
        self
    }

    pub fn on(mut self, event: ElementBehaviorEvent, command: impl Into<String>) -> Self {
        self.behavior_hooks
            .push(ElementBehaviorHook::on(event, command));
        self.interactive = true;
        self
    }

    pub fn on_if(
        self,
        event: ElementBehaviorEvent,
        command: impl Into<String>,
        present: bool,
    ) -> Self {
        if present {
            self.on(event, command)
        } else {
            self
        }
    }

    pub fn command(self, command: impl Into<String>) -> Self {
        self.on_click(command)
    }

    pub fn command_if(self, command: impl Into<String>, present: bool) -> Self {
        self.on_click_if(command, present)
    }

    pub fn command_on(self, event: ElementBehaviorEvent, command: impl Into<String>) -> Self {
        self.on(event, command)
    }

    pub fn command_on_if(
        self,
        event: ElementBehaviorEvent,
        command: impl Into<String>,
        present: bool,
    ) -> Self {
        self.on_if(event, command, present)
    }

    pub fn on_events<I, C>(mut self, events: I) -> Self
    where
        I: IntoIterator<Item = (ElementBehaviorEvent, C)>,
        C: Into<String>,
    {
        let previous_len = self.behavior_hooks.len();
        self.behavior_hooks.extend(
            events
                .into_iter()
                .map(|(event, command)| ElementBehaviorHook::on(event, command)),
        );
        self.interactive |= self.behavior_hooks.len() > previous_len;
        self
    }

    pub fn on_click(self, command: impl Into<String>) -> Self {
        self.on(ElementBehaviorEvent::Click, command)
    }

    pub fn on_click_if(self, command: impl Into<String>, present: bool) -> Self {
        self.on_if(ElementBehaviorEvent::Click, command, present)
    }

    pub fn on_context_menu(self, command: impl Into<String>) -> Self {
        self.on(ElementBehaviorEvent::ContextMenu, command)
    }

    pub fn on_context_menu_if(self, command: impl Into<String>, present: bool) -> Self {
        self.on_if(ElementBehaviorEvent::ContextMenu, command, present)
    }

    pub fn on_pointer_enter(self, command: impl Into<String>) -> Self {
        self.on(ElementBehaviorEvent::PointerEnter, command)
    }

    pub fn on_pointer_enter_if(self, command: impl Into<String>, present: bool) -> Self {
        self.on_if(ElementBehaviorEvent::PointerEnter, command, present)
    }

    pub fn on_pointer_leave(self, command: impl Into<String>) -> Self {
        self.on(ElementBehaviorEvent::PointerLeave, command)
    }

    pub fn on_pointer_leave_if(self, command: impl Into<String>, present: bool) -> Self {
        self.on_if(ElementBehaviorEvent::PointerLeave, command, present)
    }

    pub fn on_pointer_down(self, command: impl Into<String>) -> Self {
        self.on(ElementBehaviorEvent::PointerDown, command)
    }

    pub fn on_pointer_down_if(self, command: impl Into<String>, present: bool) -> Self {
        self.on_if(ElementBehaviorEvent::PointerDown, command, present)
    }

    pub fn on_pointer_up(self, command: impl Into<String>) -> Self {
        self.on(ElementBehaviorEvent::PointerUp, command)
    }

    pub fn on_pointer_up_if(self, command: impl Into<String>, present: bool) -> Self {
        self.on_if(ElementBehaviorEvent::PointerUp, command, present)
    }

    pub fn on_drag_start(self, command: impl Into<String>) -> Self {
        self.on(ElementBehaviorEvent::DragStart, command)
    }

    pub fn on_drag_start_if(self, command: impl Into<String>, present: bool) -> Self {
        self.on_if(ElementBehaviorEvent::DragStart, command, present)
    }

    pub fn on_drag(self, command: impl Into<String>) -> Self {
        self.on(ElementBehaviorEvent::Drag, command)
    }

    pub fn on_drag_if(self, command: impl Into<String>, present: bool) -> Self {
        self.on_if(ElementBehaviorEvent::Drag, command, present)
    }

    pub fn on_drag_end(self, command: impl Into<String>) -> Self {
        self.on(ElementBehaviorEvent::DragEnd, command)
    }

    pub fn on_drag_end_if(self, command: impl Into<String>, present: bool) -> Self {
        self.on_if(ElementBehaviorEvent::DragEnd, command, present)
    }

    pub fn on_scroll(self, command: impl Into<String>) -> Self {
        self.on(ElementBehaviorEvent::Scroll, command)
    }

    pub fn on_scroll_if(self, command: impl Into<String>, present: bool) -> Self {
        self.on_if(ElementBehaviorEvent::Scroll, command, present)
    }

    pub fn on_focus(self, command: impl Into<String>) -> Self {
        self.on(ElementBehaviorEvent::Focus, command)
    }

    pub fn on_focus_if(self, command: impl Into<String>, present: bool) -> Self {
        self.on_if(ElementBehaviorEvent::Focus, command, present)
    }

    pub fn on_blur(self, command: impl Into<String>) -> Self {
        self.on(ElementBehaviorEvent::Blur, command)
    }

    pub fn on_blur_if(self, command: impl Into<String>, present: bool) -> Self {
        self.on_if(ElementBehaviorEvent::Blur, command, present)
    }

    pub fn on_select(self, command: impl Into<String>) -> Self {
        self.on(ElementBehaviorEvent::Select, command)
    }

    pub fn on_select_if(self, command: impl Into<String>, present: bool) -> Self {
        self.on_if(ElementBehaviorEvent::Select, command, present)
    }

    pub fn on_key_down(self, command: impl Into<String>) -> Self {
        self.on(ElementBehaviorEvent::KeyDown, command)
    }

    pub fn on_key_down_if(self, command: impl Into<String>, present: bool) -> Self {
        self.on_if(ElementBehaviorEvent::KeyDown, command, present)
    }

    pub fn on_key_up(self, command: impl Into<String>) -> Self {
        self.on(ElementBehaviorEvent::KeyUp, command)
    }

    pub fn on_key_up_if(self, command: impl Into<String>, present: bool) -> Self {
        self.on_if(ElementBehaviorEvent::KeyUp, command, present)
    }

    pub fn interactive(mut self) -> Self {
        self.interactive = true;
        self
    }

    pub fn interactive_if(mut self, interactive: bool) -> Self {
        self.interactive |= interactive;
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn checked(self, checked: bool) -> Self {
        self.selected(checked)
    }

    pub fn check(self) -> Self {
        self.checked(true)
    }

    pub fn check_if(self, present: bool) -> Self {
        self.checked_if(true, present)
    }

    pub fn uncheck(self) -> Self {
        self.checked(false)
    }

    pub fn uncheck_if(self, present: bool) -> Self {
        self.checked_if(false, present)
    }

    pub fn checked_if(self, checked: bool, present: bool) -> Self {
        if present { self.checked(checked) } else { self }
    }

    pub fn deselect(self) -> Self {
        self.selected(false)
    }

    pub fn selected_if(self, present: bool) -> Self {
        if present { self.selected(true) } else { self }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn enabled(self, enabled: bool) -> Self {
        self.disabled(!enabled)
    }

    pub fn disabled_if(self, disabled: bool, present: bool) -> Self {
        if present {
            self.disabled(disabled)
        } else {
            self
        }
    }

    pub fn enabled_if(self, enabled: bool, present: bool) -> Self {
        if present { self.enabled(enabled) } else { self }
    }

    pub fn disable(self) -> Self {
        self.disabled(true)
    }

    pub fn enable(self) -> Self {
        self.disabled(false)
    }

    pub fn disable_if(self, present: bool) -> Self {
        if present { self.disable() } else { self }
    }

    pub fn enable_if(self, present: bool) -> Self {
        if present { self.enable() } else { self }
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    pub fn focus(self) -> Self {
        self.focused(true)
    }

    pub fn blur(self) -> Self {
        self.focused(false)
    }

    pub fn focus_if(self, present: bool) -> Self {
        if present { self.focus() } else { self }
    }

    pub fn blur_if(self, present: bool) -> Self {
        if present { self.blur() } else { self }
    }

    pub fn selectable_text(mut self) -> Self {
        self.selectable_text = true;
        self.copyable_text = true;
        self
    }

    pub fn selectable_text_if(self, present: bool) -> Self {
        if present {
            self.selectable_text()
        } else {
            self
        }
    }

    pub fn copyable_text(mut self, copyable_text: bool) -> Self {
        self.copyable_text = copyable_text;
        self
    }

    pub fn copyable_text_if(self, copyable_text: bool, present: bool) -> Self {
        if present {
            self.copyable_text(copyable_text)
        } else {
            self
        }
    }

    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    pub fn value_if(self, value: impl Into<String>, present: bool) -> Self {
        if present { self.value(value) } else { self }
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

fn prefixed_attribute_name(prefix: &str, name: impl AsRef<str>) -> String {
    let name = name.as_ref();
    if name.starts_with(prefix) {
        name.to_owned()
    } else {
        format!("{prefix}{name}")
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
            event: event.into().trim().to_owned(),
            command: command.into().trim().to_owned(),
        }
    }

    pub fn on(event: ElementBehaviorEvent, command: impl Into<String>) -> Self {
        Self::new(event.as_str(), command)
    }

    pub fn event(&self) -> &str {
        &self.event
    }

    pub fn command(&self) -> &str {
        &self.command
    }

    pub fn has_command(&self, command: &str) -> bool {
        self.command.trim() == command.trim()
    }

    pub fn intent(&self) -> Option<ElementBehaviorEvent> {
        ElementBehaviorEvent::from_name(&self.event)
    }

    pub fn matches_intent(&self, intent: ElementBehaviorEvent) -> bool {
        self.intent() == Some(intent)
    }

    pub fn matches_document_event(&self, event: &DocumentEventKind) -> bool {
        self.intent()
            .is_some_and(|hook_event| hook_event.matches_document_event(event))
    }

    pub fn is_click(&self) -> bool {
        self.matches_intent(ElementBehaviorEvent::Click)
    }

    pub fn is_context_menu(&self) -> bool {
        self.matches_intent(ElementBehaviorEvent::ContextMenu)
    }

    pub fn is_pointer_enter(&self) -> bool {
        self.matches_intent(ElementBehaviorEvent::PointerEnter)
    }

    pub fn is_pointer_leave(&self) -> bool {
        self.matches_intent(ElementBehaviorEvent::PointerLeave)
    }

    pub fn is_pointer_down(&self) -> bool {
        self.matches_intent(ElementBehaviorEvent::PointerDown)
    }

    pub fn is_pointer_up(&self) -> bool {
        self.matches_intent(ElementBehaviorEvent::PointerUp)
    }

    pub fn is_drag_start(&self) -> bool {
        self.matches_intent(ElementBehaviorEvent::DragStart)
    }

    pub fn is_drag(&self) -> bool {
        self.matches_intent(ElementBehaviorEvent::Drag)
    }

    pub fn is_drag_end(&self) -> bool {
        self.matches_intent(ElementBehaviorEvent::DragEnd)
    }

    pub fn is_any_drag(&self) -> bool {
        self.is_drag_start() || self.is_drag() || self.is_drag_end()
    }

    pub fn is_scroll(&self) -> bool {
        self.matches_intent(ElementBehaviorEvent::Scroll)
    }

    pub fn is_focus(&self) -> bool {
        self.matches_intent(ElementBehaviorEvent::Focus)
    }

    pub fn is_blur(&self) -> bool {
        self.matches_intent(ElementBehaviorEvent::Blur)
    }

    pub fn is_select(&self) -> bool {
        self.matches_intent(ElementBehaviorEvent::Select)
    }

    pub fn is_key_down(&self) -> bool {
        self.matches_intent(ElementBehaviorEvent::KeyDown)
    }

    pub fn is_key_up(&self) -> bool {
        self.matches_intent(ElementBehaviorEvent::KeyUp)
    }
}

impl<C> From<(ElementBehaviorEvent, C)> for ElementBehaviorHook
where
    C: Into<String>,
{
    fn from((event, command): (ElementBehaviorEvent, C)) -> Self {
        Self::on(event, command)
    }
}

impl<C> From<(&str, C)> for ElementBehaviorHook
where
    C: Into<String>,
{
    fn from((event, command): (&str, C)) -> Self {
        Self::new(event, command)
    }
}

impl<C> From<(String, C)> for ElementBehaviorHook
where
    C: Into<String>,
{
    fn from((event, command): (String, C)) -> Self {
        Self::new(event, command)
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
    Focus,
    Blur,
    Select,
    KeyDown,
    KeyUp,
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
            Self::Focus => "focus",
            Self::Blur => "blur",
            Self::Select => "select",
            Self::KeyDown => "keydown",
            Self::KeyUp => "keyup",
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
            "focus" | "focusin" => Some(Self::Focus),
            "blur" | "focusout" => Some(Self::Blur),
            "select" | "selectionchange" | "selection-change" => Some(Self::Select),
            "keydown" | "key-down" => Some(Self::KeyDown),
            "keyup" | "key-up" => Some(Self::KeyUp),
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
                | (Self::Focus, DocumentEventKind::Focused)
                | (Self::Blur, DocumentEventKind::Blurred)
                | (Self::Select, DocumentEventKind::SelectionStarted)
                | (Self::Select, DocumentEventKind::SelectionChanged)
                | (Self::Select, DocumentEventKind::SelectionEnded)
                | (Self::KeyDown, DocumentEventKind::KeyDown(_))
                | (Self::KeyUp, DocumentEventKind::KeyUp(_))
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
