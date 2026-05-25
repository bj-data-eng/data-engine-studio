use crate::element::{
    ClassName, Color, Element, ElementBehaviorEvent, ElementBehaviorHook, ElementId, Glyph,
};
use crate::geometry::{ClipRect, Point, Rect, ScrollAxis, Size};
use crate::query::DocumentSnapshot;
use crate::style::{
    ComputedStyle, FloatingHideData, FloatingPlacement, FloatingVisibility, Transition,
};
use crate::text::{NormalizedText, TextContent, TextLayoutResult};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ElementState {
    pub scroll_x: f32,
    pub scroll_y: f32,
    pub hovered: bool,
    pub pressed: bool,
    pub dragging: bool,
    pub scrollbar_hovered_axis: Option<ScrollAxis>,
    pub scrollbar_dragged_axis: Option<ScrollAxis>,
    pub(crate) scrollbar_visual_width_x: Option<f32>,
    pub(crate) scrollbar_visual_width_y: Option<f32>,
    pub focused: bool,
    pub click_count: u32,
    pub(crate) rendered_style: Option<ComputedStyle>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ChangeSet {
    pub created: Vec<ElementId>,
    pub retained: Vec<ElementId>,
    pub removed: Vec<ElementId>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedElement {
    pub id: ElementId,
    pub element: Element,
    pub classes: Vec<ClassName>,
    pub role: Option<String>,
    pub attributes: std::collections::BTreeMap<String, String>,
    pub behavior_hooks: Vec<ElementBehaviorHook>,
    pub rect: Rect,
    pub clip_rect: ClipRect,
    pub style: ComputedStyle,
    pub text: Option<TextContent>,
    pub normalized_text: Option<NormalizedText>,
    pub text_layout: Option<TextLayoutResult>,
    pub selectable_text: bool,
    pub copyable_text: bool,
    pub value: Option<String>,
    pub glyph: Option<Glyph>,
    pub interactive: bool,
    pub selected: bool,
    pub disabled: bool,
    pub focused: bool,
    pub floating: Option<ResolvedFloating>,
    pub children: Vec<ResolvedElement>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResolvedFloating {
    pub placement: FloatingPlacement,
    pub arrow_offset: Option<Point>,
    pub arrow_center_offset: Option<f32>,
    pub arrow_size: Option<Size>,
    pub available_size: Size,
    pub hide: Option<FloatingHideData>,
    pub visibility: FloatingVisibility,
}

impl ResolvedElement {
    pub fn find(&self, id: &str) -> Option<&Self> {
        if self.id.as_str() == id {
            return Some(self);
        }
        self.children.iter().find_map(|child| child.find(id))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DocumentOutput {
    pub changes: ChangeSet,
    pub layout: ResolvedElement,
    pub hit_id: Option<ElementId>,
    pub active_drag: Option<DocumentDrag>,
    pub completed_drag: Option<DocumentDrag>,
    pub text_selection: Option<DocumentTextSelection>,
    pub events: Vec<DocumentEvent>,
    pub scroll_chrome: Vec<ScrollChrome>,
    pub animating: bool,
    pub metrics: DocumentMetrics,
}

impl DocumentOutput {
    pub fn snapshot(&self) -> DocumentSnapshot<'_> {
        DocumentSnapshot::new(&self.layout)
    }

    pub fn hit_id(&self) -> Option<&ElementId> {
        self.hit_id.as_ref()
    }

    pub fn hit_is(&self, id: &str) -> bool {
        self.hit_id.as_ref().is_some_and(|hit| hit.as_str() == id)
    }

    pub fn hit_element(&self) -> Option<crate::ElementSnapshot<'_>> {
        self.hit_id
            .as_ref()
            .and_then(|id| self.snapshot().find(id.as_str()))
    }

    pub fn first_event(&self) -> Option<&DocumentEvent> {
        self.events.first()
    }

    pub fn events_for<'a>(&'a self, target: &'a str) -> impl Iterator<Item = &'a DocumentEvent> {
        self.events
            .iter()
            .filter(move |event| event.target.as_str() == target)
    }

    pub fn first_event_for(&self, target: &str) -> Option<&DocumentEvent> {
        self.events
            .iter()
            .find(|event| event.target.as_str() == target)
    }

    pub fn events_of_kind(&self, kind: DocumentEventKind) -> impl Iterator<Item = &DocumentEvent> {
        self.events.iter().filter(move |event| event.kind == kind)
    }

    pub fn first_event_of_kind(&self, kind: DocumentEventKind) -> Option<&DocumentEvent> {
        self.events_of_kind(kind).next()
    }

    pub fn event_targets_of_kind(
        &self,
        kind: DocumentEventKind,
    ) -> impl Iterator<Item = &ElementId> {
        self.events_of_kind(kind).map(|event| &event.target)
    }

    pub fn first_event_target(&self, kind: DocumentEventKind) -> Option<&ElementId> {
        self.event_targets_of_kind(kind).next()
    }

    pub fn clicked_targets(&self) -> impl Iterator<Item = &ElementId> {
        self.event_targets_of_kind(DocumentEventKind::Clicked)
    }

    pub fn first_clicked_target(&self) -> Option<&ElementId> {
        self.clicked_targets().next()
    }

    pub fn pressed_targets(&self) -> impl Iterator<Item = &ElementId> {
        self.event_targets_of_kind(DocumentEventKind::Pressed)
    }

    pub fn first_pressed_target(&self) -> Option<&ElementId> {
        self.pressed_targets().next()
    }

    pub fn released_targets(&self) -> impl Iterator<Item = &ElementId> {
        self.event_targets_of_kind(DocumentEventKind::Released)
    }

    pub fn first_released_target(&self) -> Option<&ElementId> {
        self.released_targets().next()
    }

    pub fn was_clicked(&self, target: &str) -> bool {
        self.has_event(target, DocumentEventKind::Clicked)
    }

    pub fn was_pressed(&self, target: &str) -> bool {
        self.has_event(target, DocumentEventKind::Pressed)
    }

    pub fn was_released(&self, target: &str) -> bool {
        self.has_event(target, DocumentEventKind::Released)
    }

    pub fn context_requested_targets(&self) -> impl Iterator<Item = &ElementId> {
        self.event_targets_of_kind(DocumentEventKind::ContextRequested)
    }

    pub fn first_context_requested_target(&self) -> Option<&ElementId> {
        self.context_requested_targets().next()
    }

    pub fn context_requested_for(&self, target: &str) -> bool {
        self.has_event(target, DocumentEventKind::ContextRequested)
    }

    pub fn drag_started_targets(&self) -> impl Iterator<Item = &ElementId> {
        self.event_targets_of_kind(DocumentEventKind::DragStarted)
    }

    pub fn first_drag_started_target(&self) -> Option<&ElementId> {
        self.drag_started_targets().next()
    }

    pub fn drag_moved_targets(&self) -> impl Iterator<Item = &ElementId> {
        self.event_targets_of_kind(DocumentEventKind::DragMoved)
    }

    pub fn first_drag_moved_target(&self) -> Option<&ElementId> {
        self.drag_moved_targets().next()
    }

    pub fn drag_ended_targets(&self) -> impl Iterator<Item = &ElementId> {
        self.event_targets_of_kind(DocumentEventKind::DragEnded)
    }

    pub fn first_drag_ended_target(&self) -> Option<&ElementId> {
        self.drag_ended_targets().next()
    }

    pub fn drag_started_for(&self, target: &str) -> bool {
        self.has_event(target, DocumentEventKind::DragStarted)
    }

    pub fn drag_moved_for(&self, target: &str) -> bool {
        self.has_event(target, DocumentEventKind::DragMoved)
    }

    pub fn drag_ended_for(&self, target: &str) -> bool {
        self.has_event(target, DocumentEventKind::DragEnded)
    }

    pub fn active_drag(&self) -> Option<&DocumentDrag> {
        self.active_drag.as_ref()
    }

    pub fn completed_drag(&self) -> Option<&DocumentDrag> {
        self.completed_drag.as_ref()
    }

    pub fn has_active_drag(&self) -> bool {
        self.active_drag.is_some()
    }

    pub fn has_completed_drag(&self) -> bool {
        self.completed_drag.is_some()
    }

    pub fn active_drag_target(&self) -> Option<&ElementId> {
        self.active_drag.as_ref().map(|drag| &drag.target)
    }

    pub fn completed_drag_target(&self) -> Option<&ElementId> {
        self.completed_drag.as_ref().map(|drag| &drag.target)
    }

    pub fn active_drag_target_is(&self, target: &str) -> bool {
        self.active_drag
            .as_ref()
            .is_some_and(|drag| drag.target_is(target))
    }

    pub fn completed_drag_target_is(&self, target: &str) -> bool {
        self.completed_drag
            .as_ref()
            .is_some_and(|drag| drag.target_is(target))
    }

    pub fn key_down_events(&self) -> impl Iterator<Item = (&ElementId, KeyInput)> {
        self.events.iter().filter_map(|event| match event.kind {
            DocumentEventKind::KeyDown(key) => Some((&event.target, key)),
            _ => None,
        })
    }

    pub fn key_down_for<'a>(&'a self, target: &'a str) -> impl Iterator<Item = KeyInput> + 'a {
        self.key_down_events()
            .filter(move |(id, _)| id.as_str() == target)
            .map(|(_, key)| key)
    }

    pub fn has_key_down(&self, target: &str, key: KeyInput) -> bool {
        self.key_down_for(target).any(|event_key| event_key == key)
    }

    pub fn key_up_events(&self) -> impl Iterator<Item = (&ElementId, KeyInput)> {
        self.events.iter().filter_map(|event| match event.kind {
            DocumentEventKind::KeyUp(key) => Some((&event.target, key)),
            _ => None,
        })
    }

    pub fn key_up_for<'a>(&'a self, target: &'a str) -> impl Iterator<Item = KeyInput> + 'a {
        self.key_up_events()
            .filter(move |(id, _)| id.as_str() == target)
            .map(|(_, key)| key)
    }

    pub fn has_key_up(&self, target: &str, key: KeyInput) -> bool {
        self.key_up_for(target).any(|event_key| event_key == key)
    }

    pub fn has_event(&self, target: &str, kind: DocumentEventKind) -> bool {
        self.events_for(target).any(|event| event.kind == kind)
    }

    pub fn has_event_kind(&self, kind: DocumentEventKind) -> bool {
        self.events.iter().any(|event| event.kind == kind)
    }

    pub fn commands(&self) -> Vec<DocumentCommand> {
        self.command_events()
            .map(|command| DocumentCommand {
                target: command.target.clone(),
                event: command.event,
                command: command.command.to_owned(),
            })
            .collect()
    }

    pub fn command_events(&self) -> DocumentCommandIter<'_> {
        DocumentCommandIter {
            output: self,
            event_index: 0,
            hook_index: 0,
        }
    }

    pub fn first_command(&self) -> Option<DocumentCommandRef<'_>> {
        self.command_events().next()
    }

    pub fn commands_of_kind(
        &self,
        kind: DocumentEventKind,
    ) -> impl Iterator<Item = DocumentCommandRef<'_>> {
        self.command_events()
            .filter(move |command| command.event == kind)
    }

    pub fn first_command_of_kind(&self, kind: DocumentEventKind) -> Option<DocumentCommandRef<'_>> {
        self.commands_of_kind(kind).next()
    }

    pub fn commands_for_intent(
        &self,
        intent: ElementBehaviorEvent,
    ) -> impl Iterator<Item = DocumentCommandRef<'_>> {
        self.command_events()
            .filter(move |command| intent.matches_document_event(&command.event))
    }

    pub fn first_command_for_intent(
        &self,
        intent: ElementBehaviorEvent,
    ) -> Option<DocumentCommandRef<'_>> {
        self.commands_for_intent(intent).next()
    }

    pub fn commands_for<'a>(
        &'a self,
        target: &'a str,
    ) -> impl Iterator<Item = DocumentCommandRef<'a>> + 'a {
        self.command_events()
            .filter(move |command| command.target.as_str() == target)
    }

    pub fn first_command_for<'a>(&'a self, target: &'a str) -> Option<DocumentCommandRef<'a>> {
        self.commands_for(target).next()
    }

    pub fn has_command(&self, target: &str, command: &str) -> bool {
        self.commands_for(target)
            .any(|event| event.command == command)
    }

    pub fn has_command_kind(&self, target: &str, kind: DocumentEventKind, command: &str) -> bool {
        self.commands_for(target)
            .any(|event| event.event == kind && event.command == command)
    }

    pub fn has_command_intent(
        &self,
        target: &str,
        intent: ElementBehaviorEvent,
        command: &str,
    ) -> bool {
        self.commands_for(target)
            .any(|event| intent.matches_document_event(&event.event) && event.command == command)
    }

    pub fn text_selection(&self) -> Option<&DocumentTextSelection> {
        self.text_selection.as_ref()
    }

    pub fn has_text_selection(&self) -> bool {
        self.text_selection
            .as_ref()
            .is_some_and(|selection| !selection.is_empty())
    }

    pub fn text_selection_is_active(&self) -> bool {
        self.text_selection
            .as_ref()
            .is_some_and(|selection| selection.active)
    }

    pub fn text_selection_target(&self) -> Option<&ElementId> {
        self.text_selection
            .as_ref()
            .map(|selection| &selection.target)
    }

    pub fn text_selection_target_is(&self, target: &str) -> bool {
        self.text_selection
            .as_ref()
            .is_some_and(|selection| selection.target.as_str() == target)
    }

    pub fn text_selection_range(&self) -> Option<std::ops::Range<usize>> {
        self.text_selection
            .as_ref()
            .map(DocumentTextSelection::char_range)
    }

    pub fn text_selection_granularity(&self) -> Option<TextSelectionGranularity> {
        self.text_selection
            .as_ref()
            .map(|selection| selection.granularity)
    }

    pub fn selected_text(&self) -> Option<String> {
        let selection = self.text_selection.as_ref()?;
        let frame = self.layout.find(selection.target.as_str())?;
        if !frame.copyable_text {
            return None;
        }
        let text = frame.text.as_ref()?.semantic_text();
        selection.selected_text_from(text)
    }

    pub fn selected_text_for(&self, target: &str) -> Option<String> {
        if !self.text_selection_target_is(target) {
            return None;
        }
        self.selected_text()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentCommand {
    pub target: ElementId,
    pub event: DocumentEventKind,
    pub command: String,
}

impl DocumentCommand {
    /// Returns the element that emitted this command.
    pub fn target(&self) -> &ElementId {
        &self.target
    }

    /// Returns true when this command was emitted by the supplied element id.
    pub fn target_is(&self, target: &str) -> bool {
        self.target.as_str() == target
    }

    /// Returns the resolved document event that emitted this command.
    pub fn event(&self) -> DocumentEventKind {
        self.event
    }

    /// Returns the authored command name.
    pub fn command(&self) -> &str {
        &self.command
    }

    /// Returns true when this command was emitted by the supplied behavior intent.
    pub fn matches_intent(&self, intent: ElementBehaviorEvent) -> bool {
        intent.matches_document_event(&self.event)
    }

    /// Returns true when this command was emitted by click intent.
    pub fn is_click(&self) -> bool {
        self.matches_intent(ElementBehaviorEvent::Click)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DocumentCommandRef<'a> {
    pub target: &'a ElementId,
    pub event: DocumentEventKind,
    pub command: &'a str,
}

impl DocumentCommandRef<'_> {
    /// Returns the element that emitted this command.
    pub fn target(&self) -> &ElementId {
        self.target
    }

    /// Returns true when this command was emitted by the supplied element id.
    pub fn target_is(&self, target: &str) -> bool {
        self.target.as_str() == target
    }

    /// Returns the resolved document event that emitted this command.
    pub fn event(&self) -> DocumentEventKind {
        self.event
    }

    /// Returns the authored command name.
    pub fn command(&self) -> &str {
        self.command
    }

    /// Returns true when this command was emitted by the supplied behavior intent.
    pub fn matches_intent(&self, intent: ElementBehaviorEvent) -> bool {
        intent.matches_document_event(&self.event)
    }

    /// Returns true when this command was emitted by click intent.
    pub fn is_click(&self) -> bool {
        self.matches_intent(ElementBehaviorEvent::Click)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentCommandBinding<Action> {
    pub event: Option<ElementBehaviorEvent>,
    pub command: String,
    pub action: Action,
}

impl<Action> DocumentCommandBinding<Action> {
    pub fn new(command: impl Into<String>, action: Action) -> Self {
        Self {
            event: None,
            command: command.into(),
            action,
        }
    }

    pub fn on(event: ElementBehaviorEvent, command: impl Into<String>, action: Action) -> Self {
        Self {
            event: Some(event),
            command: command.into(),
            action,
        }
    }

    pub fn click(command: impl Into<String>, action: Action) -> Self {
        Self::on(ElementBehaviorEvent::Click, command, action)
    }

    pub fn context_menu(command: impl Into<String>, action: Action) -> Self {
        Self::on(ElementBehaviorEvent::ContextMenu, command, action)
    }

    pub fn pointer_enter(command: impl Into<String>, action: Action) -> Self {
        Self::on(ElementBehaviorEvent::PointerEnter, command, action)
    }

    pub fn pointer_leave(command: impl Into<String>, action: Action) -> Self {
        Self::on(ElementBehaviorEvent::PointerLeave, command, action)
    }

    pub fn pointer_down(command: impl Into<String>, action: Action) -> Self {
        Self::on(ElementBehaviorEvent::PointerDown, command, action)
    }

    pub fn pointer_up(command: impl Into<String>, action: Action) -> Self {
        Self::on(ElementBehaviorEvent::PointerUp, command, action)
    }

    pub fn drag_start(command: impl Into<String>, action: Action) -> Self {
        Self::on(ElementBehaviorEvent::DragStart, command, action)
    }

    pub fn drag(command: impl Into<String>, action: Action) -> Self {
        Self::on(ElementBehaviorEvent::Drag, command, action)
    }

    pub fn drag_end(command: impl Into<String>, action: Action) -> Self {
        Self::on(ElementBehaviorEvent::DragEnd, command, action)
    }

    pub fn scroll(command: impl Into<String>, action: Action) -> Self {
        Self::on(ElementBehaviorEvent::Scroll, command, action)
    }

    pub fn key_down(command: impl Into<String>, action: Action) -> Self {
        Self::on(ElementBehaviorEvent::KeyDown, command, action)
    }

    pub fn key_up(command: impl Into<String>, action: Action) -> Self {
        Self::on(ElementBehaviorEvent::KeyUp, command, action)
    }

    fn matches(&self, command: DocumentCommandRef<'_>) -> bool {
        self.command == command.command.trim()
            && self
                .event
                .is_none_or(|event| event.matches_document_event(&command.event))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentCommandRegistry<Action> {
    bindings: Vec<DocumentCommandBinding<Action>>,
}

impl<Action> Default for DocumentCommandRegistry<Action> {
    fn default() -> Self {
        Self {
            bindings: Vec::new(),
        }
    }
}

impl<Action> DocumentCommandRegistry<Action> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bind(mut self, command: impl Into<String>, action: Action) -> Self {
        self.push(command, action);
        self
    }

    pub fn bind_if(mut self, command: impl Into<String>, action: Action, present: bool) -> Self {
        self.push_if(command, action, present);
        self
    }

    pub fn bind_on(
        mut self,
        event: ElementBehaviorEvent,
        command: impl Into<String>,
        action: Action,
    ) -> Self {
        self.push_on(event, command, action);
        self
    }

    pub fn bind_on_if(
        mut self,
        event: ElementBehaviorEvent,
        command: impl Into<String>,
        action: Action,
        present: bool,
    ) -> Self {
        self.push_on_if(event, command, action, present);
        self
    }

    pub fn bind_binding(mut self, binding: impl Into<DocumentCommandBinding<Action>>) -> Self {
        self.push_binding(binding);
        self
    }

    pub fn bind_binding_if(
        mut self,
        binding: impl Into<DocumentCommandBinding<Action>>,
        present: bool,
    ) -> Self {
        self.push_binding_if(binding, present);
        self
    }

    pub fn bind_click(mut self, command: impl Into<String>, action: Action) -> Self {
        self.push_click(command, action);
        self
    }

    pub fn bind_click_if(
        mut self,
        command: impl Into<String>,
        action: Action,
        present: bool,
    ) -> Self {
        self.push_click_if(command, action, present);
        self
    }

    pub fn bind_context_menu(mut self, command: impl Into<String>, action: Action) -> Self {
        self.push_context_menu(command, action);
        self
    }

    pub fn bind_context_menu_if(
        mut self,
        command: impl Into<String>,
        action: Action,
        present: bool,
    ) -> Self {
        self.push_context_menu_if(command, action, present);
        self
    }

    pub fn bind_pointer_enter(mut self, command: impl Into<String>, action: Action) -> Self {
        self.push_pointer_enter(command, action);
        self
    }

    pub fn bind_pointer_enter_if(
        mut self,
        command: impl Into<String>,
        action: Action,
        present: bool,
    ) -> Self {
        self.push_pointer_enter_if(command, action, present);
        self
    }

    pub fn bind_pointer_leave(mut self, command: impl Into<String>, action: Action) -> Self {
        self.push_pointer_leave(command, action);
        self
    }

    pub fn bind_pointer_leave_if(
        mut self,
        command: impl Into<String>,
        action: Action,
        present: bool,
    ) -> Self {
        self.push_pointer_leave_if(command, action, present);
        self
    }

    pub fn bind_pointer_down(mut self, command: impl Into<String>, action: Action) -> Self {
        self.push_pointer_down(command, action);
        self
    }

    pub fn bind_pointer_down_if(
        mut self,
        command: impl Into<String>,
        action: Action,
        present: bool,
    ) -> Self {
        self.push_pointer_down_if(command, action, present);
        self
    }

    pub fn bind_pointer_up(mut self, command: impl Into<String>, action: Action) -> Self {
        self.push_pointer_up(command, action);
        self
    }

    pub fn bind_pointer_up_if(
        mut self,
        command: impl Into<String>,
        action: Action,
        present: bool,
    ) -> Self {
        self.push_pointer_up_if(command, action, present);
        self
    }

    pub fn bind_drag_start(mut self, command: impl Into<String>, action: Action) -> Self {
        self.push_drag_start(command, action);
        self
    }

    pub fn bind_drag_start_if(
        mut self,
        command: impl Into<String>,
        action: Action,
        present: bool,
    ) -> Self {
        self.push_drag_start_if(command, action, present);
        self
    }

    pub fn bind_drag(mut self, command: impl Into<String>, action: Action) -> Self {
        self.push_drag(command, action);
        self
    }

    pub fn bind_drag_if(
        mut self,
        command: impl Into<String>,
        action: Action,
        present: bool,
    ) -> Self {
        self.push_drag_if(command, action, present);
        self
    }

    pub fn bind_drag_end(mut self, command: impl Into<String>, action: Action) -> Self {
        self.push_drag_end(command, action);
        self
    }

    pub fn bind_drag_end_if(
        mut self,
        command: impl Into<String>,
        action: Action,
        present: bool,
    ) -> Self {
        self.push_drag_end_if(command, action, present);
        self
    }

    pub fn bind_scroll(mut self, command: impl Into<String>, action: Action) -> Self {
        self.push_scroll(command, action);
        self
    }

    pub fn bind_scroll_if(
        mut self,
        command: impl Into<String>,
        action: Action,
        present: bool,
    ) -> Self {
        self.push_scroll_if(command, action, present);
        self
    }

    pub fn bind_key_down(mut self, command: impl Into<String>, action: Action) -> Self {
        self.push_key_down(command, action);
        self
    }

    pub fn bind_key_down_if(
        mut self,
        command: impl Into<String>,
        action: Action,
        present: bool,
    ) -> Self {
        self.push_key_down_if(command, action, present);
        self
    }

    pub fn bind_key_up(mut self, command: impl Into<String>, action: Action) -> Self {
        self.push_key_up(command, action);
        self
    }

    pub fn bind_key_up_if(
        mut self,
        command: impl Into<String>,
        action: Action,
        present: bool,
    ) -> Self {
        self.push_key_up_if(command, action, present);
        self
    }

    pub fn bind_many<I, Command>(mut self, bindings: I) -> Self
    where
        I: IntoIterator<Item = (Command, Action)>,
        Command: Into<String>,
    {
        self.push_many(bindings);
        self
    }

    pub fn bind_many_if<I, Command>(mut self, bindings: I, present: bool) -> Self
    where
        I: IntoIterator<Item = (Command, Action)>,
        Command: Into<String>,
    {
        self.push_many_if(bindings, present);
        self
    }

    pub fn bind_bindings<I, B>(mut self, bindings: I) -> Self
    where
        I: IntoIterator<Item = B>,
        B: Into<DocumentCommandBinding<Action>>,
    {
        self.push_bindings(bindings);
        self
    }

    pub fn bind_bindings_if<I, B>(mut self, bindings: I, present: bool) -> Self
    where
        I: IntoIterator<Item = B>,
        B: Into<DocumentCommandBinding<Action>>,
    {
        self.push_bindings_if(bindings, present);
        self
    }

    pub fn push(&mut self, command: impl Into<String>, action: Action) {
        self.bindings
            .push(DocumentCommandBinding::new(command, action));
    }

    pub fn push_if(&mut self, command: impl Into<String>, action: Action, present: bool) {
        if present {
            self.push(command, action);
        }
    }

    pub fn push_on(
        &mut self,
        event: ElementBehaviorEvent,
        command: impl Into<String>,
        action: Action,
    ) {
        self.bindings
            .push(DocumentCommandBinding::on(event, command, action));
    }

    pub fn push_on_if(
        &mut self,
        event: ElementBehaviorEvent,
        command: impl Into<String>,
        action: Action,
        present: bool,
    ) {
        if present {
            self.push_on(event, command, action);
        }
    }

    pub fn push_binding(&mut self, binding: impl Into<DocumentCommandBinding<Action>>) {
        self.bindings.push(binding.into());
    }

    pub fn push_binding_if(
        &mut self,
        binding: impl Into<DocumentCommandBinding<Action>>,
        present: bool,
    ) {
        if present {
            self.push_binding(binding);
        }
    }

    pub fn push_click(&mut self, command: impl Into<String>, action: Action) {
        self.bindings
            .push(DocumentCommandBinding::click(command, action));
    }

    pub fn push_click_if(&mut self, command: impl Into<String>, action: Action, present: bool) {
        if present {
            self.push_click(command, action);
        }
    }

    pub fn push_context_menu(&mut self, command: impl Into<String>, action: Action) {
        self.push_on(ElementBehaviorEvent::ContextMenu, command, action);
    }

    pub fn push_context_menu_if(
        &mut self,
        command: impl Into<String>,
        action: Action,
        present: bool,
    ) {
        if present {
            self.push_context_menu(command, action);
        }
    }

    pub fn push_pointer_enter(&mut self, command: impl Into<String>, action: Action) {
        self.push_on(ElementBehaviorEvent::PointerEnter, command, action);
    }

    pub fn push_pointer_enter_if(
        &mut self,
        command: impl Into<String>,
        action: Action,
        present: bool,
    ) {
        if present {
            self.push_pointer_enter(command, action);
        }
    }

    pub fn push_pointer_leave(&mut self, command: impl Into<String>, action: Action) {
        self.push_on(ElementBehaviorEvent::PointerLeave, command, action);
    }

    pub fn push_pointer_leave_if(
        &mut self,
        command: impl Into<String>,
        action: Action,
        present: bool,
    ) {
        if present {
            self.push_pointer_leave(command, action);
        }
    }

    pub fn push_pointer_down(&mut self, command: impl Into<String>, action: Action) {
        self.push_on(ElementBehaviorEvent::PointerDown, command, action);
    }

    pub fn push_pointer_down_if(
        &mut self,
        command: impl Into<String>,
        action: Action,
        present: bool,
    ) {
        if present {
            self.push_pointer_down(command, action);
        }
    }

    pub fn push_pointer_up(&mut self, command: impl Into<String>, action: Action) {
        self.push_on(ElementBehaviorEvent::PointerUp, command, action);
    }

    pub fn push_pointer_up_if(
        &mut self,
        command: impl Into<String>,
        action: Action,
        present: bool,
    ) {
        if present {
            self.push_pointer_up(command, action);
        }
    }

    pub fn push_drag_start(&mut self, command: impl Into<String>, action: Action) {
        self.push_on(ElementBehaviorEvent::DragStart, command, action);
    }

    pub fn push_drag_start_if(
        &mut self,
        command: impl Into<String>,
        action: Action,
        present: bool,
    ) {
        if present {
            self.push_drag_start(command, action);
        }
    }

    pub fn push_drag(&mut self, command: impl Into<String>, action: Action) {
        self.push_on(ElementBehaviorEvent::Drag, command, action);
    }

    pub fn push_drag_if(&mut self, command: impl Into<String>, action: Action, present: bool) {
        if present {
            self.push_drag(command, action);
        }
    }

    pub fn push_drag_end(&mut self, command: impl Into<String>, action: Action) {
        self.push_on(ElementBehaviorEvent::DragEnd, command, action);
    }

    pub fn push_drag_end_if(&mut self, command: impl Into<String>, action: Action, present: bool) {
        if present {
            self.push_drag_end(command, action);
        }
    }

    pub fn push_scroll(&mut self, command: impl Into<String>, action: Action) {
        self.push_on(ElementBehaviorEvent::Scroll, command, action);
    }

    pub fn push_scroll_if(&mut self, command: impl Into<String>, action: Action, present: bool) {
        if present {
            self.push_scroll(command, action);
        }
    }

    pub fn push_key_down(&mut self, command: impl Into<String>, action: Action) {
        self.push_on(ElementBehaviorEvent::KeyDown, command, action);
    }

    pub fn push_key_down_if(&mut self, command: impl Into<String>, action: Action, present: bool) {
        if present {
            self.push_key_down(command, action);
        }
    }

    pub fn push_key_up(&mut self, command: impl Into<String>, action: Action) {
        self.push_on(ElementBehaviorEvent::KeyUp, command, action);
    }

    pub fn push_key_up_if(&mut self, command: impl Into<String>, action: Action, present: bool) {
        if present {
            self.push_key_up(command, action);
        }
    }

    pub fn push_many<I, Command>(&mut self, bindings: I)
    where
        I: IntoIterator<Item = (Command, Action)>,
        Command: Into<String>,
    {
        self.bindings.extend(
            bindings
                .into_iter()
                .map(|(command, action)| DocumentCommandBinding::new(command, action)),
        );
    }

    pub fn push_many_if<I, Command>(&mut self, bindings: I, present: bool)
    where
        I: IntoIterator<Item = (Command, Action)>,
        Command: Into<String>,
    {
        if present {
            self.push_many(bindings);
        }
    }

    pub fn push_bindings<I, B>(&mut self, bindings: I)
    where
        I: IntoIterator<Item = B>,
        B: Into<DocumentCommandBinding<Action>>,
    {
        self.bindings.extend(bindings.into_iter().map(Into::into));
    }

    pub fn push_bindings_if<I, B>(&mut self, bindings: I, present: bool)
    where
        I: IntoIterator<Item = B>,
        B: Into<DocumentCommandBinding<Action>>,
    {
        if present {
            self.push_bindings(bindings);
        }
    }

    pub fn action_for(&self, command: &str) -> Option<&Action> {
        let command = command.trim();
        self.bindings
            .iter()
            .find(|binding| binding.event.is_none() && binding.command == command)
            .map(|binding| &binding.action)
    }

    pub fn action_for_event(&self, command: DocumentCommandRef<'_>) -> Option<&Action> {
        self.bindings
            .iter()
            .find(|binding| binding.event.is_some() && binding.matches(command))
            .or_else(|| {
                self.bindings
                    .iter()
                    .find(|binding| binding.event.is_none() && binding.matches(command))
            })
            .map(|binding| &binding.action)
    }

    pub fn command_actions<'a>(
        &'a self,
        output: &'a DocumentOutput,
    ) -> impl Iterator<Item = DocumentCommandActionRef<'a, Action>> + 'a {
        output.command_events().filter_map(|command| {
            let action = self.action_for_event(command)?;
            Some(DocumentCommandActionRef {
                target: command.target,
                event: command.event,
                command: command.command,
                action,
            })
        })
    }

    pub fn collect_actions(&self, output: &DocumentOutput) -> Vec<DocumentCommandAction<Action>>
    where
        Action: Clone,
    {
        self.command_actions(output)
            .map(DocumentCommandAction::from)
            .collect()
    }

    pub fn command_actions_of_kind<'a>(
        &'a self,
        output: &'a DocumentOutput,
        kind: DocumentEventKind,
    ) -> impl Iterator<Item = DocumentCommandActionRef<'a, Action>> + 'a {
        output.commands_of_kind(kind).filter_map(|command| {
            let action = self.action_for_event(command)?;
            Some(DocumentCommandActionRef {
                target: command.target,
                event: command.event,
                command: command.command,
                action,
            })
        })
    }

    pub fn collect_actions_of_kind(
        &self,
        output: &DocumentOutput,
        kind: DocumentEventKind,
    ) -> Vec<DocumentCommandAction<Action>>
    where
        Action: Clone,
    {
        self.command_actions_of_kind(output, kind)
            .map(DocumentCommandAction::from)
            .collect()
    }

    pub fn command_actions_for_intent<'a>(
        &'a self,
        output: &'a DocumentOutput,
        intent: ElementBehaviorEvent,
    ) -> impl Iterator<Item = DocumentCommandActionRef<'a, Action>> + 'a {
        output.commands_for_intent(intent).filter_map(|command| {
            let action = self.action_for_event(command)?;
            Some(DocumentCommandActionRef {
                target: command.target,
                event: command.event,
                command: command.command,
                action,
            })
        })
    }

    pub fn collect_actions_for_intent(
        &self,
        output: &DocumentOutput,
        intent: ElementBehaviorEvent,
    ) -> Vec<DocumentCommandAction<Action>>
    where
        Action: Clone,
    {
        self.command_actions_for_intent(output, intent)
            .map(DocumentCommandAction::from)
            .collect()
    }

    pub fn command_actions_for<'a>(
        &'a self,
        output: &'a DocumentOutput,
        target: &'a str,
    ) -> impl Iterator<Item = DocumentCommandActionRef<'a, Action>> + 'a {
        self.command_actions(output)
            .filter(move |command| command.target.as_str() == target)
    }

    pub fn collect_actions_for(
        &self,
        output: &DocumentOutput,
        target: &str,
    ) -> Vec<DocumentCommandAction<Action>>
    where
        Action: Clone,
    {
        self.command_actions_for(output, target)
            .map(DocumentCommandAction::from)
            .collect()
    }

    pub fn clicked_actions<'a>(
        &'a self,
        output: &'a DocumentOutput,
    ) -> impl Iterator<Item = DocumentCommandActionRef<'a, Action>> + 'a {
        self.command_actions_for_intent(output, ElementBehaviorEvent::Click)
    }

    pub fn collect_clicked_actions(
        &self,
        output: &DocumentOutput,
    ) -> Vec<DocumentCommandAction<Action>>
    where
        Action: Clone,
    {
        self.clicked_actions(output)
            .map(DocumentCommandAction::from)
            .collect()
    }

    pub fn bindings(&self) -> &[DocumentCommandBinding<Action>] {
        &self.bindings
    }

    pub fn dispatch<'a, Handler>(
        &'a self,
        output: &'a DocumentOutput,
        mut handler: Handler,
    ) -> DocumentCommandDispatchReport
    where
        Handler: FnMut(DocumentCommandActionRef<'a, Action>),
    {
        let mut report = DocumentCommandDispatchReport::default();
        for command in output.command_events() {
            report.commands += 1;
            let Some(action) = self.action_for_event(command) else {
                report.unhandled += 1;
                continue;
            };
            report.handled += 1;
            handler(DocumentCommandActionRef {
                target: command.target,
                event: command.event,
                command: command.command,
                action,
            });
        }
        report
    }

    pub fn dispatch_kind<'a, Handler>(
        &'a self,
        output: &'a DocumentOutput,
        kind: DocumentEventKind,
        mut handler: Handler,
    ) -> DocumentCommandDispatchReport
    where
        Handler: FnMut(DocumentCommandActionRef<'a, Action>),
    {
        let mut report = DocumentCommandDispatchReport::default();
        for command in output.commands_of_kind(kind) {
            report.commands += 1;
            let Some(action) = self.action_for_event(command) else {
                report.unhandled += 1;
                continue;
            };
            report.handled += 1;
            handler(DocumentCommandActionRef {
                target: command.target,
                event: command.event,
                command: command.command,
                action,
            });
        }
        report
    }

    pub fn dispatch_intent<'a, Handler>(
        &'a self,
        output: &'a DocumentOutput,
        intent: ElementBehaviorEvent,
        mut handler: Handler,
    ) -> DocumentCommandDispatchReport
    where
        Handler: FnMut(DocumentCommandActionRef<'a, Action>),
    {
        let mut report = DocumentCommandDispatchReport::default();
        for command in output.commands_for_intent(intent) {
            report.commands += 1;
            let Some(action) = self.action_for_event(command) else {
                report.unhandled += 1;
                continue;
            };
            report.handled += 1;
            handler(DocumentCommandActionRef {
                target: command.target,
                event: command.event,
                command: command.command,
                action,
            });
        }
        report
    }

    pub fn dispatch_clicked<'a, Handler>(
        &'a self,
        output: &'a DocumentOutput,
        handler: Handler,
    ) -> DocumentCommandDispatchReport
    where
        Handler: FnMut(DocumentCommandActionRef<'a, Action>),
    {
        self.dispatch_intent(output, ElementBehaviorEvent::Click, handler)
    }
}

impl<Action> std::iter::FromIterator<DocumentCommandBinding<Action>>
    for DocumentCommandRegistry<Action>
{
    fn from_iter<I: IntoIterator<Item = DocumentCommandBinding<Action>>>(iter: I) -> Self {
        Self::new().bind_bindings(iter)
    }
}

impl<Action, Command> std::iter::FromIterator<(Command, Action)> for DocumentCommandRegistry<Action>
where
    Command: Into<String>,
{
    fn from_iter<I: IntoIterator<Item = (Command, Action)>>(iter: I) -> Self {
        Self::new().bind_many(iter)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DocumentCommandActionRef<'a, Action> {
    pub target: &'a ElementId,
    pub event: DocumentEventKind,
    pub command: &'a str,
    pub action: &'a Action,
}

impl<Action> DocumentCommandActionRef<'_, Action> {
    /// Returns the element that emitted this action.
    pub fn target(&self) -> &ElementId {
        self.target
    }

    /// Returns true when this action was emitted by the supplied element id.
    pub fn target_is(&self, target: &str) -> bool {
        self.target.as_str() == target
    }

    /// Returns the resolved document event that emitted this action.
    pub fn event(&self) -> DocumentEventKind {
        self.event
    }

    /// Returns the authored command name that mapped to this action.
    pub fn command(&self) -> &str {
        self.command
    }

    /// Returns the typed app action mapped from this command.
    pub fn action(&self) -> &Action {
        self.action
    }

    /// Returns true when this action was emitted by the supplied behavior intent.
    pub fn matches_intent(&self, intent: ElementBehaviorEvent) -> bool {
        intent.matches_document_event(&self.event)
    }

    /// Returns true when this action was emitted by click intent.
    pub fn is_click(&self) -> bool {
        self.matches_intent(ElementBehaviorEvent::Click)
    }

    /// Returns true when this command mapped to the supplied typed app action.
    pub fn is_action(&self, action: &Action) -> bool
    where
        Action: PartialEq,
    {
        self.action == action
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentCommandAction<Action> {
    pub target: ElementId,
    pub event: DocumentEventKind,
    pub command: String,
    pub action: Action,
}

impl<Action> DocumentCommandAction<Action> {
    /// Returns the element that emitted this action.
    pub fn target(&self) -> &ElementId {
        &self.target
    }

    /// Returns true when this action was emitted by the supplied element id.
    pub fn target_is(&self, target: &str) -> bool {
        self.target.as_str() == target
    }

    /// Returns the resolved document event that emitted this action.
    pub fn event(&self) -> DocumentEventKind {
        self.event
    }

    /// Returns the authored command name that mapped to this action.
    pub fn command(&self) -> &str {
        &self.command
    }

    /// Returns the typed app action mapped from this command.
    pub fn action(&self) -> &Action {
        &self.action
    }

    /// Returns true when this action was emitted by the supplied behavior intent.
    pub fn matches_intent(&self, intent: ElementBehaviorEvent) -> bool {
        intent.matches_document_event(&self.event)
    }

    /// Returns true when this action was emitted by click intent.
    pub fn is_click(&self) -> bool {
        self.matches_intent(ElementBehaviorEvent::Click)
    }

    /// Returns true when this command mapped to the supplied typed app action.
    pub fn is_action(&self, action: &Action) -> bool
    where
        Action: PartialEq,
    {
        &self.action == action
    }
}

impl<Action: Clone> From<DocumentCommandActionRef<'_, Action>> for DocumentCommandAction<Action> {
    fn from(command: DocumentCommandActionRef<'_, Action>) -> Self {
        Self {
            target: command.target.clone(),
            event: command.event,
            command: command.command.to_owned(),
            action: command.action.clone(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DocumentCommandDispatchReport {
    pub commands: usize,
    pub handled: usize,
    pub unhandled: usize,
}

impl DocumentCommandDispatchReport {
    /// Returns true when no commands were inspected by this dispatch pass.
    pub fn is_empty(&self) -> bool {
        self.commands == 0
    }

    /// Returns true when at least one command was inspected by this dispatch pass.
    pub fn has_commands(&self) -> bool {
        self.commands > 0
    }

    /// Returns true when at least one command was mapped to a typed action.
    pub fn has_handled(&self) -> bool {
        self.handled > 0
    }

    /// Returns true when at least one command did not have a typed action binding.
    pub fn has_unhandled(&self) -> bool {
        self.unhandled > 0
    }

    /// Returns true when every inspected command was mapped to a typed action.
    pub fn all_handled(&self) -> bool {
        self.unhandled == 0
    }
}

pub struct DocumentCommandIter<'a> {
    output: &'a DocumentOutput,
    event_index: usize,
    hook_index: usize,
}

impl<'a> Iterator for DocumentCommandIter<'a> {
    type Item = DocumentCommandRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.event_index < self.output.events.len() {
            let event = &self.output.events[self.event_index];
            let Some(element) = self.output.layout.find(event.target.as_str()) else {
                self.event_index += 1;
                self.hook_index = 0;
                continue;
            };
            while self.hook_index < element.behavior_hooks.len() {
                let hook = &element.behavior_hooks[self.hook_index];
                self.hook_index += 1;
                if hook.matches_document_event(&event.kind) {
                    return Some(DocumentCommandRef {
                        target: &event.target,
                        event: event.kind,
                        command: hook.command.as_str(),
                    });
                }
            }
            self.event_index += 1;
            self.hook_index = 0;
        }
        None
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentEvent {
    pub target: ElementId,
    pub kind: DocumentEventKind,
}

impl DocumentEvent {
    pub fn new(target: impl Into<ElementId>, kind: DocumentEventKind) -> Self {
        Self {
            target: target.into(),
            kind,
        }
    }

    /// Returns the element that emitted this event.
    pub fn target(&self) -> &ElementId {
        &self.target
    }

    /// Returns true when this event was emitted by the supplied element id.
    pub fn target_is(&self, target: &str) -> bool {
        self.target.as_str() == target
    }

    /// Returns the resolved document event kind.
    pub fn kind(&self) -> DocumentEventKind {
        self.kind
    }

    /// Returns true when this event matches the supplied authored behavior intent.
    pub fn matches_intent(&self, intent: ElementBehaviorEvent) -> bool {
        intent.matches_document_event(&self.kind)
    }

    /// Returns true when this event is click intent.
    pub fn is_click(&self) -> bool {
        self.matches_intent(ElementBehaviorEvent::Click)
    }

    /// Returns true when this event is key-down intent.
    pub fn is_key_down(&self) -> bool {
        self.matches_intent(ElementBehaviorEvent::KeyDown)
    }

    /// Returns true when this event is key-up intent.
    pub fn is_key_up(&self) -> bool {
        self.matches_intent(ElementBehaviorEvent::KeyUp)
    }

    /// Returns true when this event is any pointer drag intent.
    pub fn is_drag(&self) -> bool {
        self.matches_intent(ElementBehaviorEvent::DragStart)
            || self.matches_intent(ElementBehaviorEvent::Drag)
            || self.matches_intent(ElementBehaviorEvent::DragEnd)
    }

    pub fn pointer_entered(target: impl Into<ElementId>) -> Self {
        Self::new(target, DocumentEventKind::PointerEntered)
    }

    pub fn pointer_exited(target: impl Into<ElementId>) -> Self {
        Self::new(target, DocumentEventKind::PointerExited)
    }

    pub fn pressed(target: impl Into<ElementId>) -> Self {
        Self::new(target, DocumentEventKind::Pressed)
    }

    pub fn released(target: impl Into<ElementId>) -> Self {
        Self::new(target, DocumentEventKind::Released)
    }

    pub fn clicked(target: impl Into<ElementId>) -> Self {
        Self::new(target, DocumentEventKind::Clicked)
    }

    pub fn context_requested(target: impl Into<ElementId>) -> Self {
        Self::new(target, DocumentEventKind::ContextRequested)
    }

    pub fn scrolled(target: impl Into<ElementId>, axis: ScrollAxis) -> Self {
        Self::new(target, DocumentEventKind::Scrolled(axis))
    }

    pub fn drag_started(target: impl Into<ElementId>) -> Self {
        Self::new(target, DocumentEventKind::DragStarted)
    }

    pub fn drag_moved(target: impl Into<ElementId>) -> Self {
        Self::new(target, DocumentEventKind::DragMoved)
    }

    pub fn drag_ended(target: impl Into<ElementId>) -> Self {
        Self::new(target, DocumentEventKind::DragEnded)
    }

    pub fn key_down(target: impl Into<ElementId>, key: KeyInput) -> Self {
        Self::new(target, DocumentEventKind::KeyDown(key))
    }

    pub fn key_up(target: impl Into<ElementId>, key: KeyInput) -> Self {
        Self::new(target, DocumentEventKind::KeyUp(key))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocumentEventKind {
    PointerEntered,
    PointerExited,
    Pressed,
    Released,
    Clicked,
    ContextRequested,
    DragStarted,
    DragMoved,
    DragEnded,
    Scrolled(ScrollAxis),
    KeyDown(KeyInput),
    KeyUp(KeyInput),
}

#[derive(Clone, Debug, PartialEq)]
pub struct DocumentDrag {
    pub target: ElementId,
    pub origin: Point,
    pub current: Point,
    pub delta: Point,
    pub pointer_offset: Point,
}

impl DocumentDrag {
    /// Returns the element whose pointer drag is active or completed.
    pub fn target(&self) -> &ElementId {
        &self.target
    }

    /// Returns true when this drag belongs to the supplied element id.
    pub fn target_is(&self, target: &str) -> bool {
        self.target.as_str() == target
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DocumentTextSelection {
    pub target: ElementId,
    pub anchor: Point,
    pub focus: Point,
    pub anchor_index: usize,
    pub focus_index: usize,
    pub anchor_range_start: usize,
    pub anchor_range_end: usize,
    pub granularity: TextSelectionGranularity,
    pub active: bool,
}

impl DocumentTextSelection {
    pub fn char_range(&self) -> std::ops::Range<usize> {
        let start = self.anchor_index.min(self.focus_index);
        let end = self.anchor_index.max(self.focus_index);
        start..end
    }

    pub fn is_empty(&self) -> bool {
        self.anchor_index == self.focus_index
    }

    pub fn selected_text_from(&self, text: &str) -> Option<String> {
        let range = self.char_range();
        if range.is_empty() {
            return None;
        }
        Some(slice_char_range(text, range))
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TextSelectionGranularity {
    #[default]
    Character,
    Word,
    Paragraph,
}

fn slice_char_range(text: &str, range: std::ops::Range<usize>) -> String {
    text.chars()
        .skip(range.start)
        .take(range.end.saturating_sub(range.start))
        .collect()
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DocumentMetrics {
    pub element_count: usize,
    pub scroll_chrome_count: usize,
    pub style_nodes_visited: usize,
    pub reused_cached_layout: bool,
    pub reused_input_layout: bool,
    pub input_changed_state: bool,
    pub animation_changed_style: bool,
    pub animation_changed_layout: bool,
    pub animation_changed_paint: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DocumentInput {
    pub pointer: Option<PointerInput>,
    pub scroll_delta: Point,
    pub keys: Vec<KeyInput>,
}

impl DocumentInput {
    pub fn pointer(pointer: PointerInput) -> Self {
        Self {
            pointer: Some(pointer),
            ..Self::default()
        }
    }

    pub fn pointer_at(position: Point) -> Self {
        Self::pointer(PointerInput::at(position))
    }

    pub fn pointer_at_time(position: Point, time_seconds: f64) -> Self {
        Self::pointer(PointerInput::new(position, time_seconds))
    }

    pub fn primary_click(position: Point) -> Self {
        Self::pointer(PointerInput::at(position).primary_clicked())
    }

    pub fn primary_press(position: Point) -> Self {
        Self::pointer(PointerInput::at(position).primary_press())
    }

    pub fn primary_down(position: Point) -> Self {
        Self::pointer(PointerInput::at(position).primary_down())
    }

    pub fn primary_drag(position: Point, delta: Point) -> Self {
        Self::pointer(
            PointerInput::at(position)
                .primary_down()
                .with_primary_delta(delta),
        )
    }

    pub fn primary_double_click(position: Point) -> Self {
        Self::pointer(PointerInput::at(position).primary_double_clicked())
    }

    pub fn primary_triple_click(position: Point) -> Self {
        Self::pointer(PointerInput::at(position).primary_triple_clicked())
    }

    pub fn secondary_click(position: Point) -> Self {
        Self::pointer(PointerInput::at(position).secondary_clicked())
    }

    pub fn scroll(delta: Point) -> Self {
        Self {
            scroll_delta: delta,
            ..Self::default()
        }
    }

    pub fn key_down(key: DocumentKey) -> Self {
        Self::key(KeyInput::down(key))
    }

    pub fn key_down_with_modifiers(key: DocumentKey, modifiers: KeyModifiers) -> Self {
        Self::key(KeyInput::down_with_modifiers(key, modifiers))
    }

    pub fn key_up(key: DocumentKey) -> Self {
        Self::key(KeyInput::up(key))
    }

    pub fn key_up_with_modifiers(key: DocumentKey, modifiers: KeyModifiers) -> Self {
        Self::key(KeyInput::up_with_modifiers(key, modifiers))
    }

    pub fn key(key: KeyInput) -> Self {
        Self {
            keys: vec![key],
            ..Self::default()
        }
    }

    pub fn with_pointer(mut self, pointer: PointerInput) -> Self {
        self.pointer = Some(pointer);
        self
    }

    pub fn with_scroll(mut self, delta: Point) -> Self {
        self.scroll_delta = delta;
        self
    }

    pub fn with_key(mut self, key: KeyInput) -> Self {
        self.keys.push(key);
        self
    }

    pub fn with_keys(mut self, keys: impl IntoIterator<Item = KeyInput>) -> Self {
        self.keys.extend(keys);
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointerInput {
    pub position: Point,
    pub primary_delta: Point,
    pub primary_down: bool,
    pub primary_pressed: bool,
    pub primary_clicked: bool,
    pub primary_click_count: u8,
    pub secondary_clicked: bool,
    pub time_seconds: f64,
}

impl PointerInput {
    pub fn new(position: Point, time_seconds: f64) -> Self {
        Self {
            position,
            primary_delta: Point::ZERO,
            primary_down: false,
            primary_pressed: false,
            primary_clicked: false,
            primary_click_count: 0,
            secondary_clicked: false,
            time_seconds,
        }
    }

    pub fn at(position: Point) -> Self {
        Self::new(position, 0.0)
    }

    pub fn with_position(mut self, position: Point) -> Self {
        self.position = position;
        self
    }

    pub fn with_time(mut self, time_seconds: f64) -> Self {
        self.time_seconds = time_seconds;
        self
    }

    pub fn with_primary_delta(mut self, primary_delta: Point) -> Self {
        self.primary_delta = primary_delta;
        self
    }

    pub fn with_primary_down(mut self, primary_down: bool) -> Self {
        self.primary_down = primary_down;
        self
    }

    pub fn with_primary_pressed(mut self, primary_pressed: bool) -> Self {
        self.primary_pressed = primary_pressed;
        self
    }

    pub fn with_primary_clicked(mut self, click_count: u8) -> Self {
        self.primary_clicked = click_count > 0;
        self.primary_click_count = click_count;
        self
    }

    pub fn with_secondary_clicked(mut self, secondary_clicked: bool) -> Self {
        self.secondary_clicked = secondary_clicked;
        self
    }

    pub fn primary_down(self) -> Self {
        self.with_primary_down(true)
    }

    pub fn primary_pressed(self) -> Self {
        self.with_primary_pressed(true)
    }

    pub fn primary_press(self) -> Self {
        self.primary_down().primary_pressed()
    }

    pub fn primary_clicked(self) -> Self {
        self.with_primary_clicked(1)
    }

    pub fn primary_double_clicked(self) -> Self {
        self.with_primary_clicked(2)
    }

    pub fn primary_triple_clicked(self) -> Self {
        self.with_primary_clicked(3)
    }

    pub fn secondary_clicked(self) -> Self {
        self.with_secondary_clicked(true)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KeyInput {
    pub key: DocumentKey,
    pub modifiers: KeyModifiers,
    pub pressed: bool,
}

impl KeyInput {
    pub fn down(key: DocumentKey) -> Self {
        Self {
            key,
            modifiers: KeyModifiers::default(),
            pressed: true,
        }
    }

    pub fn down_with_modifiers(key: DocumentKey, modifiers: KeyModifiers) -> Self {
        Self::down(key).with_modifiers(modifiers)
    }

    pub fn up(key: DocumentKey) -> Self {
        Self {
            key,
            modifiers: KeyModifiers::default(),
            pressed: false,
        }
    }

    pub fn up_with_modifiers(key: DocumentKey, modifiers: KeyModifiers) -> Self {
        Self::up(key).with_modifiers(modifiers)
    }

    pub fn with_modifiers(mut self, modifiers: KeyModifiers) -> Self {
        self.modifiers = modifiers;
        self
    }

    pub fn alt(mut self) -> Self {
        self.modifiers = self.modifiers.alt();
        self
    }

    pub fn ctrl(mut self) -> Self {
        self.modifiers = self.modifiers.ctrl();
        self
    }

    pub fn shift(mut self) -> Self {
        self.modifiers = self.modifiers.shift();
        self
    }

    pub fn command(mut self) -> Self {
        self.modifiers = self.modifiers.command();
        self
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct KeyModifiers {
    pub alt: bool,
    pub ctrl: bool,
    pub shift: bool,
    pub command: bool,
}

impl KeyModifiers {
    pub const fn new() -> Self {
        Self {
            alt: false,
            ctrl: false,
            shift: false,
            command: false,
        }
    }

    pub const fn alt(mut self) -> Self {
        self.alt = true;
        self
    }

    pub const fn ctrl(mut self) -> Self {
        self.ctrl = true;
        self
    }

    pub const fn shift(mut self) -> Self {
        self.shift = true;
        self
    }

    pub const fn command(mut self) -> Self {
        self.command = true;
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocumentKey {
    Enter,
    Escape,
    Tab,
    Space,
    Backspace,
    Delete,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    Home,
    End,
    PageUp,
    PageDown,
    Character(char),
    Unknown,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScrollChrome {
    pub element_id: ElementId,
    pub axis: ScrollAxis,
    pub track_rect: Rect,
    pub hit_rect: Rect,
    pub handle_rect: Rect,
    pub handle_color: Color,
    pub track_color: Option<Color>,
    pub handle_border_color: Option<Color>,
    pub handle_border_width: f32,
    pub radius: f32,
    pub max_scroll: f32,
    pub visible: bool,
    pub expanded: bool,
    pub hovered: bool,
    pub dragged: bool,
    pub(crate) compact_visual_width: f32,
    pub(crate) expanded_visual_width: f32,
    pub(crate) transition: Option<Transition>,
}
