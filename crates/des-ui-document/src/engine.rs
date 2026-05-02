use crate::animation::{AnimationUpdate, update_element_style_animation};
use crate::element::{Document, ElementId};
use crate::geometry::{Overflow, Rect, ScrollAxis, Size};
use crate::layout::{hit_path, layout_element};
use crate::scroll::scroll_chrome;
use crate::state::{
    ChangeSet, DocumentInput, DocumentMetrics, DocumentOutput, ElementState, PointerInput,
    ResolvedElement, ScrollChrome,
};
use crate::style::StyleSheet;
use std::collections::{BTreeSet, HashMap};

struct ScrollDrag {
    element_id: ElementId,
    axis: ScrollAxis,
    pointer_offset_from_handle_start: f32,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct InputUpdate {
    hit_id: Option<ElementId>,
    changed: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct InteractionSnapshot {
    scroll_x: f32,
    scroll_y: f32,
    hovered: bool,
    pressed: bool,
    scrollbar_hovered_axis: Option<ScrollAxis>,
    scrollbar_dragged_axis: Option<ScrollAxis>,
    scrollbar_visual_width_x: Option<f32>,
    scrollbar_visual_width_y: Option<f32>,
    click_count: u32,
}

#[derive(Default)]
pub struct DocumentEngine {
    states: HashMap<ElementId, ElementState>,
    scroll_limits: HashMap<ElementId, Size>,
    active_scroll_drag: Option<ScrollDrag>,
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
        let changes = self.sync_element_states(document);
        let mut scroll_limits = HashMap::new();
        let input_layout = layout_element(
            &document.root,
            Rect::new(0.0, 0.0, document.viewport.width, document.viewport.height),
            stylesheet,
            &self.states,
            &mut scroll_limits,
        );
        self.scroll_limits = scroll_limits;
        let input_scroll_chrome = scroll_chrome(&input_layout, &self.states, &self.scroll_limits);
        let input_update = self.apply_input(&input_layout, &input_scroll_chrome, input);
        let clamp_changed = self.clamp_scroll_states();
        let animation_update = self.update_style_animation(document, stylesheet);
        let scrollbar_animation_update = self.update_scrollbar_animation(&input_layout);

        let needs_final_layout = input_update.changed
            || clamp_changed
            || animation_update.changed
            || scrollbar_animation_update.changed;
        let (layout, scroll_chrome, reused_input_layout) = if needs_final_layout {
            let mut scroll_limits = HashMap::new();
            let layout = layout_element(
                &document.root,
                Rect::new(0.0, 0.0, document.viewport.width, document.viewport.height),
                stylesheet,
                &self.states,
                &mut scroll_limits,
            );
            self.scroll_limits = scroll_limits;
            self.clamp_scroll_states();
            let scroll_chrome = scroll_chrome(&layout, &self.states, &self.scroll_limits);
            (layout, scroll_chrome, false)
        } else {
            (input_layout, input_scroll_chrome, true)
        };
        let element_count = count_resolved_elements(&layout);
        let scroll_chrome_count = scroll_chrome.len();

        DocumentOutput {
            changes,
            layout,
            hit_id: input_update.hit_id,
            scroll_chrome,
            animating: animation_update.animating || scrollbar_animation_update.animating,
            metrics: DocumentMetrics {
                element_count,
                scroll_chrome_count,
                reused_input_layout,
                input_changed_state: input_update.changed || clamp_changed,
                animation_changed_style: animation_update.changed
                    || scrollbar_animation_update.changed,
            },
        }
    }

    pub fn element_state(&self, id: &str) -> Option<&ElementState> {
        self.states.get(&ElementId::new(id))
    }

    pub fn element_state_mut(&mut self, id: &str) -> Option<&mut ElementState> {
        self.states.get_mut(&ElementId::new(id))
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
            state.scrollbar_hovered_axis = None;
            state.scrollbar_dragged_axis = None;
        }

        let Some(pointer) = input.pointer else {
            update.changed = interaction_changed(&self.states, &previous);
            return update;
        };
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
            update.changed |= interaction_changed(&self.states, &previous);
            return update;
        }
        if let Some(active_drag) = &self.active_scroll_drag {
            if let Some(state) = self.states.get_mut(&active_drag.element_id) {
                state.hovered = true;
                state.pressed = true;
                state.scrollbar_dragged_axis = Some(active_drag.axis);
            }
            update.hit_id = Some(active_drag.element_id.clone());
            update.changed |= interaction_changed(&self.states, &previous);
            return update;
        }

        let Some(path) = hit_path(layout, pointer.position) else {
            update.changed |= interaction_changed(&self.states, &previous);
            return update;
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

        update.changed |= interaction_changed(&self.states, &previous);
        update
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

    fn update_scrollbar_animation(&mut self, layout: &ResolvedElement) -> AnimationUpdate {
        const SNAP_EPSILON: f32 = 0.001;
        let mut update = AnimationUpdate::default();
        update +=
            self.update_scrollbar_animation_for_frame(layout, ScrollAxis::Vertical, SNAP_EPSILON);
        update +=
            self.update_scrollbar_animation_for_frame(layout, ScrollAxis::Horizontal, SNAP_EPSILON);
        for child in &layout.children {
            update += self.update_scrollbar_animation(child);
        }
        update
    }

    fn update_scrollbar_animation_for_frame(
        &mut self,
        frame: &ResolvedElement,
        axis: ScrollAxis,
        snap_epsilon: f32,
    ) -> AnimationUpdate {
        let max_scroll = self
            .scroll_limits
            .get(&frame.id)
            .copied()
            .unwrap_or_default();
        let can_scroll = match axis {
            ScrollAxis::Horizontal => {
                frame.style.overflow_x == Overflow::Scroll && max_scroll.width > 0.0
            }
            ScrollAxis::Vertical => {
                frame.style.overflow_y == Overflow::Scroll && max_scroll.height > 0.0
            }
        };
        if !can_scroll {
            return AnimationUpdate::default();
        }

        let Some(state) = self.states.get_mut(&frame.id) else {
            return AnimationUpdate::default();
        };
        let expanded = state.scrollbar_hovered_axis == Some(axis)
            || state.scrollbar_dragged_axis == Some(axis);
        let target = if expanded {
            frame.style.scrollbar_expanded_width
        } else {
            frame.style.scrollbar_width
        }
        .max(0.0);
        let current_slot = match axis {
            ScrollAxis::Horizontal => &mut state.scrollbar_visual_width_x,
            ScrollAxis::Vertical => &mut state.scrollbar_visual_width_y,
        };
        let current = current_slot.unwrap_or_else(|| {
            if expanded {
                frame.style.scrollbar_width.max(0.0)
            } else {
                target
            }
        });
        let amount = frame
            .style
            .transition
            .map(|transition| transition.easing.sample(transition.step))
            .unwrap_or(1.0);
        let next = ease_f32(current, target, amount, snap_epsilon);
        let changed = current_slot.is_none_or(|value| (value - next).abs() > f32::EPSILON);
        *current_slot = Some(next);
        AnimationUpdate {
            changed,
            animating: (next - target).abs() > snap_epsilon,
        }
    }
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
                scrollbar_hovered_axis: state.scrollbar_hovered_axis,
                scrollbar_dragged_axis: state.scrollbar_dragged_axis,
                scrollbar_visual_width_x: state.scrollbar_visual_width_x,
                scrollbar_visual_width_y: state.scrollbar_visual_width_y,
                click_count: state.click_count,
            }
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
