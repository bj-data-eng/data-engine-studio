use crate::animation::{AnimationUpdate, update_element_style_animation};
use crate::element::{Document, Element, ElementId};
use crate::geometry::{Overflow, Point, Rect, ScrollAxis, Size};
use crate::layout::{hit_path, layout_element};
use crate::scroll::scroll_chrome;
use crate::state::{
    ChangeSet, DocumentDrag, DocumentEvent, DocumentInput, DocumentMetrics, DocumentOutput,
    DocumentTextSelection, ElementState, PointerInput, ResolvedElement, ScrollChrome,
};
use crate::style::{
    ChildPosition, ComputedStyle, StyleInvalidation, StyleSheet, classify_computed_style_change,
    resolve_style_with_position,
};
use crate::text::{FallbackTextMeasurer, TextMeasurer, TextMeasurerKey};
use std::collections::{BTreeSet, HashMap};

const POINTER_DRAG_ACTIVATION_DISTANCE: f32 = 5.0;

struct ScrollDrag {
    element_id: ElementId,
    axis: ScrollAxis,
    pointer_offset_from_handle_start: f32,
}

#[derive(Clone, Debug)]
struct PointerDrag {
    target: ElementId,
    origin: crate::geometry::Point,
    current: crate::geometry::Point,
    pointer_offset: crate::geometry::Point,
    activated: bool,
}

impl PointerDrag {
    fn document_drag(&self) -> DocumentDrag {
        DocumentDrag {
            target: self.target.clone(),
            origin: self.origin,
            current: self.current,
            delta: crate::geometry::Point::new(
                self.current.x - self.origin.x,
                self.current.y - self.origin.y,
            ),
            pointer_offset: self.pointer_offset,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
struct InputUpdate {
    hit_id: Option<ElementId>,
    active_drag: Option<DocumentDrag>,
    completed_drag: Option<DocumentDrag>,
    events: Vec<DocumentEvent>,
    changed: bool,
    layout_changed: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct InteractionSnapshot {
    scroll_x: f32,
    scroll_y: f32,
    hovered: bool,
    pressed: bool,
    dragging: bool,
    scrollbar_hovered_axis: Option<ScrollAxis>,
    scrollbar_dragged_axis: Option<ScrollAxis>,
    scrollbar_visual_width_x: Option<f32>,
    scrollbar_visual_width_y: Option<f32>,
    click_count: u32,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct PointerDragUpdate {
    changed: bool,
    active_drag: Option<DocumentDrag>,
    completed_drag: Option<DocumentDrag>,
}

#[derive(Default)]
pub struct DocumentEngine {
    states: HashMap<ElementId, ElementState>,
    scroll_limits: HashMap<ElementId, Size>,
    active_scroll_drag: Option<ScrollDrag>,
    active_pointer_drag: Option<PointerDrag>,
    text_selection: Option<DocumentTextSelection>,
    cached_layout: Option<ResolvedElement>,
    cached_document_root: Option<Element>,
    cached_text_measurer_key: Option<TextMeasurerKey>,
}

impl DocumentEngine {
    pub fn update(&mut self, document: &Document, stylesheet: &StyleSheet) -> DocumentOutput {
        self.update_with_input(document, stylesheet, DocumentInput::default())
    }

    pub fn update_with_input(
        &mut self,
        document: &Document,
        stylesheet: &StyleSheet,
        input: DocumentInput,
    ) -> DocumentOutput {
        let mut text_measurer = FallbackTextMeasurer;
        self.update_with_input_and_text_measurer(document, stylesheet, input, &mut text_measurer)
    }

    pub fn update_with_input_and_text_measurer(
        &mut self,
        document: &Document,
        stylesheet: &StyleSheet,
        input: DocumentInput,
        text_measurer: &mut dyn TextMeasurer,
    ) -> DocumentOutput {
        let changes = self.sync_element_states(document);
        let viewport_rect = Rect::new(0.0, 0.0, document.viewport.width, document.viewport.height);
        let text_measurer_key = text_measurer.cache_key();
        let reused_cached_layout = changes.created.is_empty()
            && changes.removed.is_empty()
            && self.cached_text_measurer_key == Some(text_measurer_key)
            && self.cached_layout_matches(viewport_rect, &document.root);
        let input_layout = if reused_cached_layout {
            self.cached_layout
                .clone()
                .expect("cached layout exists when it matches the viewport")
        } else {
            let mut scroll_limits = HashMap::new();
            let layout = layout_element(
                &document.root,
                viewport_rect,
                stylesheet,
                &self.states,
                &mut scroll_limits,
                text_measurer,
            );
            self.scroll_limits = scroll_limits;
            layout
        };
        let input_scroll_chrome = scroll_chrome(&input_layout, &self.states, &self.scroll_limits);
        let input_update = self.apply_input(&input_layout, &input_scroll_chrome, input);
        let input_style_invalidation = classify_resolved_style_invalidation(
            &input_layout,
            &document.root,
            stylesheet,
            &self.states,
        );
        let clamp_changed = self.clamp_scroll_states();
        let animation_update = self.update_style_animation(document, stylesheet);
        let scrollbar_animation_update = self.update_scrollbar_animation(&input_scroll_chrome);

        let needs_final_layout = input_update.layout_changed
            || clamp_changed
            || input_style_invalidation.layout_changed
            || animation_update.layout_changed;
        let (layout, scroll_chrome, reused_input_layout) = if needs_final_layout {
            let mut scroll_limits = HashMap::new();
            let layout = layout_element(
                &document.root,
                viewport_rect,
                stylesheet,
                &self.states,
                &mut scroll_limits,
                text_measurer,
            );
            self.scroll_limits = scroll_limits;
            self.clamp_scroll_states();
            let scroll_chrome = scroll_chrome(&layout, &self.states, &self.scroll_limits);
            (layout, scroll_chrome, false)
        } else {
            let mut layout = input_layout;
            if input_style_invalidation.paint_changed || animation_update.paint_changed {
                apply_resolved_styles(&mut layout, &document.root, stylesheet, &self.states);
            }
            let scroll_chrome = if input_update.changed
                || input_style_invalidation.paint_changed
                || animation_update.paint_changed
                || scrollbar_animation_update.paint_changed
            {
                scroll_chrome(&layout, &self.states, &self.scroll_limits)
            } else {
                input_scroll_chrome
            };
            (layout, scroll_chrome, true)
        };
        self.cached_layout = Some(layout.clone());
        self.cached_document_root = Some(document.root.clone());
        self.cached_text_measurer_key = Some(text_measurer_key);
        let element_count = count_resolved_elements(&layout);
        let scroll_chrome_count = scroll_chrome.len();

        DocumentOutput {
            changes,
            layout,
            hit_id: input_update.hit_id,
            active_drag: input_update.active_drag,
            completed_drag: input_update.completed_drag,
            text_selection: self.text_selection.clone(),
            events: input_update.events,
            scroll_chrome,
            animating: animation_update.animating || scrollbar_animation_update.animating,
            metrics: DocumentMetrics {
                element_count,
                scroll_chrome_count,
                reused_cached_layout,
                reused_input_layout,
                input_changed_state: input_update.changed || clamp_changed,
                animation_changed_style: input_style_invalidation.changed()
                    || animation_update.changed()
                    || scrollbar_animation_update.changed(),
                animation_changed_layout: input_style_invalidation.layout_changed
                    || animation_update.layout_changed,
                animation_changed_paint: input_style_invalidation.paint_changed
                    || animation_update.paint_changed
                    || scrollbar_animation_update.paint_changed,
            },
        }
    }

    pub fn element_state(&self, id: &str) -> Option<&ElementState> {
        self.states.get(&ElementId::new(id))
    }

    pub fn element_state_mut(&mut self, id: &str) -> Option<&mut ElementState> {
        self.cached_layout = None;
        self.states.get_mut(&ElementId::new(id))
    }

    pub fn text_selection(&self) -> Option<&DocumentTextSelection> {
        self.text_selection.as_ref()
    }

    pub fn snap_element_animation(&mut self, id: &str) -> bool {
        let Some(state) = self.states.get_mut(&ElementId::new(id)) else {
            return false;
        };

        let had_animation = state.rendered_style.is_some();
        state.rendered_style = None;
        if had_animation {
            self.cached_layout = None;
        }
        had_animation
    }

    pub fn scroll_element_by(&mut self, id: &str, delta: Point) -> bool {
        let id = ElementId::new(id);
        let max_scroll = self.scroll_limits.get(&id).copied().unwrap_or_default();
        let Some(state) = self.states.get_mut(&id) else {
            return false;
        };

        let scroll_x = (state.scroll_x + delta.x).clamp(0.0, max_scroll.width);
        let scroll_y = (state.scroll_y + delta.y).clamp(0.0, max_scroll.height);
        let mut changed = false;
        changed |= set_f32(&mut state.scroll_x, scroll_x);
        changed |= set_f32(&mut state.scroll_y, scroll_y);
        if changed {
            self.cached_layout = None;
        }
        changed
    }

    fn cached_layout_matches(&self, viewport_rect: Rect, document_root: &Element) -> bool {
        self.cached_layout.as_ref().is_some_and(|layout| {
            layout.rect == viewport_rect
                && self
                    .cached_document_root
                    .as_ref()
                    .is_some_and(|cached_root| cached_root == document_root)
        })
    }

    fn sync_element_states(&mut self, document: &Document) -> ChangeSet {
        let mut next_ids = BTreeSet::new();
        document.root.collect_ids(&mut next_ids);

        let existing_ids: BTreeSet<_> = self.states.keys().cloned().collect();
        let mut changes = ChangeSet::default();

        for id in &next_ids {
            if existing_ids.contains(id) {
                changes.retained.push(id.clone());
            } else {
                changes.created.push(id.clone());
                self.states.insert(id.clone(), ElementState::default());
            }
        }

        for id in existing_ids.difference(&next_ids) {
            changes.removed.push(id.clone());
            self.states.remove(id);
        }

        changes
    }

    fn apply_input(
        &mut self,
        layout: &ResolvedElement,
        scroll_chrome: &[ScrollChrome],
        input: DocumentInput,
    ) -> InputUpdate {
        let mut update = InputUpdate::default();
        let previous = interaction_snapshot(&self.states);
        for state in self.states.values_mut() {
            state.hovered = false;
            state.pressed = false;
            state.dragging = false;
            state.scrollbar_hovered_axis = None;
            state.scrollbar_dragged_axis = None;
        }

        let Some(pointer) = input.pointer else {
            if let Some(drag) = self.active_pointer_drag.take() {
                update.completed_drag = Some(drag.document_drag());
                update.changed = true;
            }
            update.changed |= self.finish_text_selection();
            return finalize_input_update(update, &self.states, &previous);
        };
        if self
            .active_pointer_drag
            .as_ref()
            .is_some_and(|drag| !drag.activated && !pointer.primary_down)
        {
            self.active_pointer_drag = None;
        }

        if self.active_pointer_drag.is_some() {
            let drag_update = self.update_pointer_drag(pointer, None);
            update.changed |= drag_update.changed;
            update.active_drag = drag_update.active_drag;
            update.completed_drag = drag_update.completed_drag;
            update.hit_id = self
                .active_pointer_drag
                .as_ref()
                .map(|drag| drag.target.clone())
                .or_else(|| {
                    update
                        .completed_drag
                        .as_ref()
                        .map(|drag| drag.target.clone())
                });
            if let Some(hit_id) = &update.hit_id
                && let Some(state) = self.states.get_mut(hit_id)
            {
                state.hovered = true;
                state.pressed = pointer.primary_down;
                state.dragging = self
                    .active_pointer_drag
                    .as_ref()
                    .is_some_and(|drag| drag.activated && pointer.primary_down);
                if update
                    .completed_drag
                    .as_ref()
                    .is_some_and(|drag| drag_delta_is_click(drag))
                {
                    state.click_count += 1;
                }
            }
            return finalize_input_update(update, &self.states, &previous);
        }

        if self
            .text_selection
            .as_ref()
            .is_some_and(|selection| selection.active)
        {
            update.changed |= self.update_active_text_selection(pointer);
            update.hit_id = self
                .text_selection
                .as_ref()
                .map(|selection| selection.target.clone());
            return finalize_input_update(update, &self.states, &previous);
        }

        let scrollbar_hit = scroll_chrome
            .iter()
            .rev()
            .find(|chrome| chrome.hit_rect.contains(pointer.position));
        update.changed |= self.apply_scrollbar_input(scroll_chrome, scrollbar_hit, pointer);

        if let Some(chrome) = scrollbar_hit {
            if let Some(state) = self.states.get_mut(&chrome.element_id) {
                state.hovered = true;
                state.pressed = pointer.primary_down;
                state.scrollbar_hovered_axis = Some(chrome.axis);
            }
            update.hit_id = Some(chrome.element_id.clone());
            return finalize_input_update(update, &self.states, &previous);
        }
        if let Some(active_drag) = &self.active_scroll_drag {
            if let Some(state) = self.states.get_mut(&active_drag.element_id) {
                state.hovered = true;
                state.pressed = true;
                state.scrollbar_dragged_axis = Some(active_drag.axis);
            }
            update.hit_id = Some(active_drag.element_id.clone());
            return finalize_input_update(update, &self.states, &previous);
        }

        let Some(path) = hit_path(layout, pointer.position) else {
            return finalize_input_update(update, &self.states, &previous);
        };
        if (input.scroll_delta.x.abs() > f32::EPSILON || input.scroll_delta.y.abs() > f32::EPSILON)
            && let Some(scroll_frame) = path.iter().rev().find(|frame| {
                frame.style.overflow_x == Overflow::Scroll
                    || frame.style.overflow_y == Overflow::Scroll
            })
            && let Some(state) = self.states.get_mut(&scroll_frame.id)
        {
            let max_scroll = self
                .scroll_limits
                .get(&scroll_frame.id)
                .copied()
                .unwrap_or_default();
            if scroll_frame.style.overflow_x == Overflow::Scroll {
                let scroll_x = (state.scroll_x - input.scroll_delta.x).clamp(0.0, max_scroll.width);
                update.changed |= set_f32(&mut state.scroll_x, scroll_x);
            }
            if scroll_frame.style.overflow_y == Overflow::Scroll {
                let scroll_y =
                    (state.scroll_y - input.scroll_delta.y).clamp(0.0, max_scroll.height);
                update.changed |= set_f32(&mut state.scroll_y, scroll_y);
            }
        }

        for frame in &path {
            if let Some(state) = self.states.get_mut(&frame.id) {
                state.hovered = true;
            }
        }

        let selectable_text_frame = path
            .iter()
            .rev()
            .find(|frame| frame.selectable_text && frame.text.is_some());
        if let Some(frame) = selectable_text_frame {
            update.hit_id = Some(frame.id.clone());
            if pointer.primary_down {
                update.changed |= self.start_text_selection(frame, pointer.position);
                return finalize_input_update(update, &self.states, &previous);
            }
        } else if pointer.primary_down {
            update.changed |= self.clear_text_selection();
        }

        update.hit_id = Some(
            path.iter()
                .rev()
                .find(|frame| frame.interactive)
                .unwrap_or_else(|| path.last().expect("hit path is never empty"))
                .id
                .clone(),
        );
        if let Some(hit_id) = &update.hit_id
            && let Some(state) = self.states.get_mut(hit_id)
        {
            state.pressed = pointer.primary_down;
            if pointer.primary_clicked {
                state.click_count += 1;
            }
        }
        let hit_frame = update
            .hit_id
            .as_ref()
            .and_then(|hit_id| path.iter().find(|frame| frame.id == *hit_id));
        let drag_update = self.update_pointer_drag(pointer, hit_frame.copied());
        update.changed |= drag_update.changed;
        update.active_drag = drag_update.active_drag;
        update.completed_drag = drag_update.completed_drag;
        if let Some(active_drag) = &self.active_pointer_drag
            && let Some(state) = self.states.get_mut(&active_drag.target)
        {
            state.dragging = active_drag.activated;
        }

        finalize_input_update(update, &self.states, &previous)
    }

    fn update_pointer_drag(
        &mut self,
        pointer: PointerInput,
        hit_frame: Option<&ResolvedElement>,
    ) -> PointerDragUpdate {
        if !pointer.primary_down {
            return self
                .active_pointer_drag
                .take()
                .filter(|drag| drag.activated)
                .map(|drag| PointerDragUpdate {
                    changed: true,
                    active_drag: None,
                    completed_drag: Some(drag.document_drag()),
                })
                .unwrap_or_default();
        }

        if let Some(drag) = &mut self.active_pointer_drag {
            let changed = drag.current != pointer.position;
            drag.current = pointer.position;
            let delta = crate::geometry::Point::new(
                drag.current.x - drag.origin.x,
                drag.current.y - drag.origin.y,
            );
            let distance = (delta.x * delta.x + delta.y * delta.y).sqrt();
            let activated = if drag.activated {
                false
            } else {
                distance >= POINTER_DRAG_ACTIVATION_DISTANCE
            };
            drag.activated |= activated;
            return PointerDragUpdate {
                changed: changed || activated,
                active_drag: drag.activated.then(|| drag.document_drag()),
                completed_drag: None,
            };
        }

        let Some(hit_frame) = hit_frame.filter(|frame| frame.interactive) else {
            return PointerDragUpdate::default();
        };
        self.active_pointer_drag = Some(PointerDrag {
            target: hit_frame.id.clone(),
            origin: pointer.position,
            current: pointer.position,
            pointer_offset: crate::geometry::Point::new(
                pointer.position.x - hit_frame.rect.origin.x,
                pointer.position.y - hit_frame.rect.origin.y,
            ),
            activated: false,
        });
        PointerDragUpdate {
            changed: false,
            active_drag: None,
            completed_drag: None,
        }
    }

    fn start_text_selection(&mut self, frame: &ResolvedElement, point: Point) -> bool {
        let next = DocumentTextSelection {
            target: frame.id.clone(),
            anchor: point,
            focus: point,
            active: true,
        };
        let changed = self.text_selection.as_ref() != Some(&next);
        self.text_selection = Some(next);
        self.active_pointer_drag = None;
        changed
    }

    fn update_active_text_selection(&mut self, pointer: PointerInput) -> bool {
        let Some(selection) = self.text_selection.as_mut() else {
            return false;
        };
        if pointer.primary_down {
            let mut changed = false;
            changed |= set_point(&mut selection.focus, pointer.position);
            changed |= set_bool(&mut selection.active, true);
            changed
        } else {
            self.finish_text_selection()
        }
    }

    fn finish_text_selection(&mut self) -> bool {
        self.text_selection
            .as_mut()
            .is_some_and(|selection| set_bool(&mut selection.active, false))
    }

    fn clear_text_selection(&mut self) -> bool {
        let changed = self.text_selection.is_some();
        self.text_selection = None;
        changed
    }

    fn apply_scrollbar_input(
        &mut self,
        scroll_chrome: &[ScrollChrome],
        hit: Option<&ScrollChrome>,
        pointer: PointerInput,
    ) -> bool {
        let mut changed = false;
        if !pointer.primary_down {
            changed |= self.active_scroll_drag.is_some();
            self.active_scroll_drag = None;
            return changed;
        }

        let active_id = self
            .active_scroll_drag
            .as_ref()
            .map(|drag| drag.element_id.clone());
        let active_chrome = active_id.as_ref().and_then(|id| {
            scroll_chrome.iter().find(|chrome| {
                &chrome.element_id == id
                    && self
                        .active_scroll_drag
                        .as_ref()
                        .is_some_and(|drag| drag.axis == chrome.axis)
            })
        });
        let chrome = active_chrome.or(hit);
        let Some(chrome) = chrome else {
            return changed;
        };

        if self
            .active_scroll_drag
            .as_ref()
            .is_none_or(|drag| drag.element_id != chrome.element_id || drag.axis != chrome.axis)
        {
            let pointer_main = pointer_axis_position(pointer.position, chrome.axis);
            let handle_start = rect_axis_origin(chrome.handle_rect, chrome.axis);
            let handle_length = rect_axis_length(chrome.handle_rect, chrome.axis);
            let offset = if chrome.handle_rect.contains(pointer.position) {
                pointer_main - handle_start
            } else {
                handle_length / 2.0
            };
            self.active_scroll_drag = Some(ScrollDrag {
                element_id: chrome.element_id.clone(),
                axis: chrome.axis,
                pointer_offset_from_handle_start: offset,
            });
            changed = true;
        }

        let Some(drag) = &self.active_scroll_drag else {
            return changed;
        };
        let track_travel = (rect_axis_length(chrome.track_rect, chrome.axis)
            - rect_axis_length(chrome.handle_rect, chrome.axis))
        .max(0.0);
        let handle_start = pointer_axis_position(pointer.position, chrome.axis)
            - drag.pointer_offset_from_handle_start;
        let handle_progress = if track_travel <= f32::EPSILON {
            0.0
        } else {
            ((handle_start - rect_axis_origin(chrome.track_rect, chrome.axis)) / track_travel)
                .clamp(0.0, 1.0)
        };
        if let Some(state) = self.states.get_mut(&chrome.element_id) {
            match chrome.axis {
                ScrollAxis::Horizontal => {
                    changed |= set_f32(&mut state.scroll_x, handle_progress * chrome.max_scroll)
                }
                ScrollAxis::Vertical => {
                    changed |= set_f32(&mut state.scroll_y, handle_progress * chrome.max_scroll)
                }
            }
            if state.scrollbar_dragged_axis != Some(chrome.axis) {
                state.scrollbar_dragged_axis = Some(chrome.axis);
                changed = true;
            }
        }
        changed
    }

    fn clamp_scroll_states(&mut self) -> bool {
        let mut changed = false;
        for (id, state) in &mut self.states {
            let max_scroll = self.scroll_limits.get(id).copied().unwrap_or_default();
            let scroll_x = state.scroll_x.clamp(0.0, max_scroll.width);
            let scroll_y = state.scroll_y.clamp(0.0, max_scroll.height);
            changed |= set_f32(&mut state.scroll_x, scroll_x);
            changed |= set_f32(&mut state.scroll_y, scroll_y);
        }
        changed
    }

    fn update_style_animation(
        &mut self,
        document: &Document,
        stylesheet: &StyleSheet,
    ) -> AnimationUpdate {
        const SNAP_EPSILON: f32 = 0.001;
        update_element_style_animation(&document.root, stylesheet, &mut self.states, SNAP_EPSILON)
    }

    fn update_scrollbar_animation(&mut self, scroll_chrome: &[ScrollChrome]) -> AnimationUpdate {
        const SNAP_EPSILON: f32 = 0.001;
        let mut update = AnimationUpdate::default();
        for chrome in scroll_chrome {
            update += self.update_scrollbar_animation_for_chrome(chrome, SNAP_EPSILON);
        }
        update
    }

    fn update_scrollbar_animation_for_chrome(
        &mut self,
        chrome: &ScrollChrome,
        snap_epsilon: f32,
    ) -> AnimationUpdate {
        let Some(state) = self.states.get_mut(&chrome.element_id) else {
            return AnimationUpdate::default();
        };
        let expanded = state.scrollbar_hovered_axis == Some(chrome.axis)
            || state.scrollbar_dragged_axis == Some(chrome.axis);
        let target = if expanded {
            chrome.expanded_visual_width
        } else {
            chrome.compact_visual_width
        };
        let current_slot = match chrome.axis {
            ScrollAxis::Horizontal => &mut state.scrollbar_visual_width_x,
            ScrollAxis::Vertical => &mut state.scrollbar_visual_width_y,
        };
        let current = current_slot.unwrap_or_else(|| {
            if expanded {
                chrome.compact_visual_width
            } else {
                target
            }
        });
        let amount = chrome
            .transition
            .map(|transition| transition.easing.sample(transition.step))
            .unwrap_or(1.0);
        let next = ease_f32(current, target, amount, snap_epsilon);
        let changed = current_slot.is_none_or(|value| (value - next).abs() > f32::EPSILON);
        *current_slot = Some(next);
        AnimationUpdate {
            paint_changed: changed,
            layout_changed: false,
            animating: (next - target).abs() > snap_epsilon,
        }
    }
}

fn classify_resolved_style_invalidation(
    frame: &ResolvedElement,
    element: &Element,
    stylesheet: &StyleSheet,
    states: &HashMap<ElementId, ElementState>,
) -> StyleInvalidation {
    classify_resolved_style_invalidation_at(frame, element, stylesheet, states, None)
}

fn classify_resolved_style_invalidation_at(
    frame: &ResolvedElement,
    element: &Element,
    stylesheet: &StyleSheet,
    states: &HashMap<ElementId, ElementState>,
    position: Option<ChildPosition>,
) -> StyleInvalidation {
    let next_style = resolved_style_for(element, states, stylesheet, position);
    let mut invalidation = classify_computed_style_change(Some(&frame.style), Some(&next_style));

    for (index, (child_frame, child_element)) in
        frame.children.iter().zip(&element.children).enumerate()
    {
        invalidation += classify_resolved_style_invalidation_at(
            child_frame,
            child_element,
            stylesheet,
            states,
            Some(ChildPosition::new(index, element.children.len())),
        );
    }

    invalidation
}

fn apply_resolved_styles(
    frame: &mut ResolvedElement,
    element: &Element,
    stylesheet: &StyleSheet,
    states: &HashMap<ElementId, ElementState>,
) {
    apply_resolved_styles_at(frame, element, stylesheet, states, None);
}

fn apply_resolved_styles_at(
    frame: &mut ResolvedElement,
    element: &Element,
    stylesheet: &StyleSheet,
    states: &HashMap<ElementId, ElementState>,
    position: Option<ChildPosition>,
) {
    frame.style = resolved_style_for(element, states, stylesheet, position);
    for (index, (child_frame, child_element)) in
        frame.children.iter_mut().zip(&element.children).enumerate()
    {
        apply_resolved_styles_at(
            child_frame,
            child_element,
            stylesheet,
            states,
            Some(ChildPosition::new(index, element.children.len())),
        );
    }
}

fn resolved_style_for(
    element: &Element,
    states: &HashMap<ElementId, ElementState>,
    stylesheet: &StyleSheet,
    position: Option<ChildPosition>,
) -> ComputedStyle {
    states
        .get(&element.id)
        .and_then(|state| state.rendered_style.clone())
        .unwrap_or_else(|| {
            resolve_style_with_position(element, stylesheet, states.get(&element.id), position)
        })
}

fn count_resolved_elements(frame: &ResolvedElement) -> usize {
    1 + frame
        .children
        .iter()
        .map(count_resolved_elements)
        .sum::<usize>()
}

fn set_f32(target: &mut f32, value: f32) -> bool {
    let changed = (*target - value).abs() > f32::EPSILON;
    *target = value;
    changed
}

fn set_bool(target: &mut bool, value: bool) -> bool {
    let changed = *target != value;
    *target = value;
    changed
}

fn set_point(target: &mut Point, value: Point) -> bool {
    let changed = *target != value;
    *target = value;
    changed
}

fn drag_delta_is_click(drag: &DocumentDrag) -> bool {
    drag.delta.x.abs() <= f32::EPSILON && drag.delta.y.abs() <= f32::EPSILON
}

fn interaction_snapshot(
    states: &HashMap<ElementId, ElementState>,
) -> HashMap<ElementId, InteractionSnapshot> {
    states
        .iter()
        .map(|(id, state)| {
            (
                id.clone(),
                InteractionSnapshot {
                    scroll_x: state.scroll_x,
                    scroll_y: state.scroll_y,
                    hovered: state.hovered,
                    pressed: state.pressed,
                    dragging: state.dragging,
                    scrollbar_hovered_axis: state.scrollbar_hovered_axis,
                    scrollbar_dragged_axis: state.scrollbar_dragged_axis,
                    scrollbar_visual_width_x: state.scrollbar_visual_width_x,
                    scrollbar_visual_width_y: state.scrollbar_visual_width_y,
                    click_count: state.click_count,
                },
            )
        })
        .collect()
}

fn interaction_changed(
    states: &HashMap<ElementId, ElementState>,
    previous: &HashMap<ElementId, InteractionSnapshot>,
) -> bool {
    states.iter().any(|(id, state)| {
        previous.get(id).copied().unwrap_or_default()
            != InteractionSnapshot {
                scroll_x: state.scroll_x,
                scroll_y: state.scroll_y,
                hovered: state.hovered,
                pressed: state.pressed,
                dragging: state.dragging,
                scrollbar_hovered_axis: state.scrollbar_hovered_axis,
                scrollbar_dragged_axis: state.scrollbar_dragged_axis,
                scrollbar_visual_width_x: state.scrollbar_visual_width_x,
                scrollbar_visual_width_y: state.scrollbar_visual_width_y,
                click_count: state.click_count,
            }
    })
}

fn finalize_input_update(
    mut update: InputUpdate,
    states: &HashMap<ElementId, ElementState>,
    previous: &HashMap<ElementId, InteractionSnapshot>,
) -> InputUpdate {
    let mut events = interaction_events(states, previous);
    events.append(&mut update.events);
    update.events = events;
    update.changed |= interaction_changed(states, previous);
    update.layout_changed |= scroll_position_changed(states, previous);
    update
}

fn interaction_events(
    states: &HashMap<ElementId, ElementState>,
    previous: &HashMap<ElementId, InteractionSnapshot>,
) -> Vec<DocumentEvent> {
    let mut ids: Vec<_> = states.keys().cloned().collect();
    ids.sort();

    let mut events = Vec::new();
    for id in ids {
        let Some(state) = states.get(&id) else {
            continue;
        };
        let previous = previous.get(&id).copied().unwrap_or_default();

        if !previous.hovered && state.hovered {
            events.push(DocumentEvent::pointer_entered(id.clone()));
        }
        if previous.hovered && !state.hovered {
            events.push(DocumentEvent::pointer_exited(id.clone()));
        }
        if !previous.pressed && state.pressed {
            events.push(DocumentEvent::pressed(id.clone()));
        }
        if previous.pressed && !state.pressed {
            events.push(DocumentEvent::released(id.clone()));
        }
        if !previous.dragging && state.dragging {
            events.push(DocumentEvent::drag_started(id.clone()));
        }
        if previous.dragging && state.dragging {
            events.push(DocumentEvent::drag_moved(id.clone()));
        }
        if previous.dragging && !state.dragging {
            events.push(DocumentEvent::drag_ended(id.clone()));
        }
        if state.click_count > previous.click_count {
            events.push(DocumentEvent::clicked(id.clone()));
        }
        if (state.scroll_x - previous.scroll_x).abs() > f32::EPSILON {
            events.push(DocumentEvent::scrolled(id.clone(), ScrollAxis::Horizontal));
        }
        if (state.scroll_y - previous.scroll_y).abs() > f32::EPSILON {
            events.push(DocumentEvent::scrolled(id, ScrollAxis::Vertical));
        }
    }

    events
}

fn scroll_position_changed(
    states: &HashMap<ElementId, ElementState>,
    previous: &HashMap<ElementId, InteractionSnapshot>,
) -> bool {
    states.iter().any(|(id, state)| {
        previous.get(id).is_some_and(|previous| {
            (state.scroll_x - previous.scroll_x).abs() > f32::EPSILON
                || (state.scroll_y - previous.scroll_y).abs() > f32::EPSILON
        })
    })
}

fn pointer_axis_position(point: crate::geometry::Point, axis: ScrollAxis) -> f32 {
    match axis {
        ScrollAxis::Horizontal => point.x,
        ScrollAxis::Vertical => point.y,
    }
}

fn rect_axis_origin(rect: Rect, axis: ScrollAxis) -> f32 {
    match axis {
        ScrollAxis::Horizontal => rect.origin.x,
        ScrollAxis::Vertical => rect.origin.y,
    }
}

fn rect_axis_length(rect: Rect, axis: ScrollAxis) -> f32 {
    match axis {
        ScrollAxis::Horizontal => rect.size.width,
        ScrollAxis::Vertical => rect.size.height,
    }
}

fn ease_f32(current: f32, target: f32, amount: f32, snap_epsilon: f32) -> f32 {
    if current == target {
        return target;
    }
    if !current.is_finite() || !target.is_finite() {
        return target;
    }

    let next = current + (target - current) * amount.clamp(0.0, 1.0);
    if (next - target).abs() <= snap_epsilon {
        target
    } else {
        next
    }
}
