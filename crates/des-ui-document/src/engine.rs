use crate::animation::update_element_style_animation;
use crate::element::{Document, ElementId};
use crate::geometry::{Overflow, Rect, ScrollAxis, Size};
use crate::layout::{hit_path, layout_element};
use crate::scroll::scroll_chrome;
use crate::state::{
    ChangeSet, DocumentInput, DocumentOutput, ElementState, PointerInput, ResolvedElement,
    ScrollChrome,
};
use crate::style::StyleSheet;
use std::collections::{BTreeSet, HashMap};

struct ScrollDrag {
    element_id: ElementId,
    axis: ScrollAxis,
    pointer_offset_from_handle_start: f32,
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
        let hit_id = self.apply_input(&input_layout, &input_scroll_chrome, input);
        self.clamp_scroll_states();
        let input_animating = self.update_style_animation(document, stylesheet);

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

        DocumentOutput {
            changes,
            layout,
            hit_id,
            scroll_chrome,
            animating: input_animating,
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
    ) -> Option<ElementId> {
        for state in self.states.values_mut() {
            state.hovered = false;
            state.pressed = false;
            state.scrollbar_hovered = false;
            state.scrollbar_dragged = false;
        }

        let pointer = input.pointer?;
        let scrollbar_hit = scroll_chrome
            .iter()
            .rev()
            .find(|chrome| chrome.hit_rect.contains(pointer.position));
        self.apply_scrollbar_input(scroll_chrome, scrollbar_hit, pointer);

        if let Some(chrome) = scrollbar_hit {
            if let Some(state) = self.states.get_mut(&chrome.element_id) {
                state.hovered = true;
                state.pressed = pointer.primary_down;
                state.scrollbar_hovered = true;
            }
            return Some(chrome.element_id.clone());
        }
        if let Some(active_drag) = &self.active_scroll_drag {
            if let Some(state) = self.states.get_mut(&active_drag.element_id) {
                state.hovered = true;
                state.pressed = true;
                state.scrollbar_dragged = true;
            }
            return Some(active_drag.element_id.clone());
        }

        let path = hit_path(layout, pointer.position)?;
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
                state.scroll_x =
                    (state.scroll_x - input.scroll_delta.x).clamp(0.0, max_scroll.width);
            }
            if scroll_frame.style.overflow_y == Overflow::Scroll {
                state.scroll_y =
                    (state.scroll_y - input.scroll_delta.y).clamp(0.0, max_scroll.height);
            }
        }

        for frame in &path {
            if let Some(state) = self.states.get_mut(&frame.id) {
                state.hovered = true;
            }
        }

        let hit_id = path
            .iter()
            .rev()
            .find(|frame| frame.interactive)
            .unwrap_or_else(|| path.last().expect("hit path is never empty"))
            .id
            .clone();
        if let Some(state) = self.states.get_mut(&hit_id) {
            state.pressed = pointer.primary_down;
            if pointer.primary_clicked {
                state.click_count += 1;
            }
        }

        Some(hit_id)
    }

    fn apply_scrollbar_input(
        &mut self,
        scroll_chrome: &[ScrollChrome],
        hit: Option<&ScrollChrome>,
        pointer: PointerInput,
    ) {
        if !pointer.primary_down {
            self.active_scroll_drag = None;
            return;
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
            return;
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
        }

        let Some(drag) = &self.active_scroll_drag else {
            return;
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
                ScrollAxis::Horizontal => state.scroll_x = handle_progress * chrome.max_scroll,
                ScrollAxis::Vertical => state.scroll_y = handle_progress * chrome.max_scroll,
            }
            state.scrollbar_dragged = true;
        }
    }

    fn clamp_scroll_states(&mut self) {
        for (id, state) in &mut self.states {
            let max_scroll = self.scroll_limits.get(id).copied().unwrap_or_default();
            state.scroll_x = state.scroll_x.clamp(0.0, max_scroll.width);
            state.scroll_y = state.scroll_y.clamp(0.0, max_scroll.height);
        }
    }

    fn update_style_animation(&mut self, document: &Document, stylesheet: &StyleSheet) -> bool {
        const SNAP_EPSILON: f32 = 0.001;
        update_element_style_animation(&document.root, stylesheet, &mut self.states, SNAP_EPSILON)
    }
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
