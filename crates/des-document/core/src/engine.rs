use crate::animation::{AnimationUpdate, update_element_style_animation};
use crate::document::{Document, StyleResolutionReport};
use crate::element::ElementId;
use crate::geometry::{Point, Rect, ScrollAxis, Size};
use crate::layout::{hit_path, to_layout_point, to_scroll_axis, to_scroll_rect};
use crate::scroll::scroll_chrome;
use crate::state::{
    ChangeSet, DocumentDrag, DocumentEvent, DocumentInput, DocumentMetrics, DocumentOutput,
    DocumentTextSelection, ElementState, PointerInput, ResolvedElement, ScrollChrome,
    TextSelectionGranularity,
};
use crate::style::StyleSheet;
use crate::text::{
    FallbackTextMeasurer, NormalizedText, TextLayoutRequest, TextMeasurer, TextMeasurerKey,
};
use std::collections::{BTreeSet, HashMap};

const POINTER_DRAG_ACTIVATION_DISTANCE: f32 = 5.0;
const TEXT_CLICK_INTERVAL_SECONDS: f64 = 0.8;
const TEXT_CLICK_DISTANCE: f32 = 6.0;

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
    selector_state_changed: bool,
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

#[derive(Clone, Debug, PartialEq)]
struct TextClickSequence {
    target: ElementId,
    position: Point,
    time_seconds: f64,
    count: u8,
}

#[derive(Default)]
pub struct DocumentEngine {
    states: HashMap<ElementId, ElementState>,
    scroll_limits: HashMap<ElementId, Size>,
    active_scroll_drag: Option<ScrollDrag>,
    active_pointer_drag: Option<PointerDrag>,
    text_selection: Option<DocumentTextSelection>,
    last_text_click: Option<TextClickSequence>,
    cached_layout: Option<ResolvedElement>,
    cached_document_instance_id: Option<u64>,
    cached_document_revision: Option<u64>,
    cached_document_stylesheet: Option<StyleSheet>,
    cached_text_measurer_key: Option<TextMeasurerKey>,
}

impl DocumentEngine {
    pub fn update(&mut self, document: &mut Document, stylesheet: &StyleSheet) -> DocumentOutput {
        self.update_with_input(document, stylesheet, DocumentInput::default())
    }

    pub fn update_with_input(
        &mut self,
        document: &mut Document,
        stylesheet: &StyleSheet,
        input: DocumentInput,
    ) -> DocumentOutput {
        let mut text_measurer = FallbackTextMeasurer;
        self.update_with_input_and_text_measurer(document, stylesheet, input, &mut text_measurer)
    }

    pub fn update_with_input_and_text_measurer(
        &mut self,
        document: &mut Document,
        stylesheet: &StyleSheet,
        input: DocumentInput,
        text_measurer: &mut dyn TextMeasurer,
    ) -> DocumentOutput {
        let changes = self.sync_document_states(document);
        let text_measurer_key = text_measurer.cache_key();
        let text_measurer_changed = self.cached_text_measurer_key != Some(text_measurer_key);
        let reused_document_cache = changes.created.is_empty()
            && changes.removed.is_empty()
            && self.cached_document_matches(document, stylesheet, text_measurer_key);
        let (input_layout, mut style_nodes_visited) = if reused_document_cache {
            (
                self.cached_layout
                    .clone()
                    .expect("cached document layout exists when cache metadata matches"),
                0,
            )
        } else {
            let input_style_report = document
                .apply_stylesheet_without_container_queries(stylesheet, &self.states)
                .expect("document styles can be resolved");
            if text_measurer_changed {
                document
                    .mark_layout_dirty()
                    .expect("document layout can be marked dirty");
            }
            if text_measurer_changed
                || input_style_report.layout_changed
                || document
                    .layout_dirty(document.root().clone())
                    .expect("document root dirty state can be resolved")
            {
                document
                    .compute_layout_with_text_measurer(text_measurer)
                    .expect("document layout can be computed");
            }
            let mut style_nodes_visited = input_style_report.visited;
            if stylesheet.has_container_rules() {
                let container_style_report = document
                    .apply_stylesheet(stylesheet, &self.states)
                    .expect("document container styles can be resolved");
                style_nodes_visited += container_style_report.visited;
                if container_style_report.layout_changed {
                    document
                        .compute_layout_with_text_measurer(text_measurer)
                        .expect("document layout can be computed after container styles");
                }
            }
            let input_layout = document
                .resolved_layout_with_text_measurer(text_measurer)
                .expect("document layout can be resolved");
            self.scroll_limits = document
                .scroll_limits()
                .expect("document scroll limits can be resolved");
            (input_layout, style_nodes_visited)
        };
        let input_scroll_chrome = scroll_chrome(&input_layout, &self.states, &self.scroll_limits);
        let input_update =
            self.apply_input(&input_layout, &input_scroll_chrome, input, text_measurer);
        let clamp_changed = self.clamp_scroll_states();
        let animation_update = self.update_style_animation(document, stylesheet);
        let scrollbar_animation_update = self.update_scrollbar_animation(&input_scroll_chrome);
        let mut final_style_report = StyleResolutionReport::default();
        if input_update.selector_state_changed || animation_update.changed() {
            final_style_report = document
                .apply_stylesheet(stylesheet, &self.states)
                .expect("document styles can be resolved");
            style_nodes_visited += final_style_report.visited;
        }
        let needs_final_layout =
            input_update.layout_changed || clamp_changed || final_style_report.layout_changed;

        let (layout, scroll_chrome, reused_input_layout) = if needs_final_layout {
            if final_style_report.layout_changed
                || document
                    .layout_dirty(document.root().clone())
                    .expect("document root dirty state can be resolved")
            {
                document
                    .compute_layout_with_text_measurer(text_measurer)
                    .expect("document layout can be computed");
            }
            self.scroll_limits = document
                .scroll_limits()
                .expect("document scroll limits can be resolved");
            self.clamp_scroll_states();
            document.apply_scroll_offsets(&self.states);
            let layout = document
                .resolved_layout_with_text_measurer(text_measurer)
                .expect("document layout can be resolved");
            let scroll_chrome = scroll_chrome(&layout, &self.states, &self.scroll_limits);
            (layout, scroll_chrome, false)
        } else {
            let layout = if final_style_report.paint_changed || animation_update.paint_changed {
                document
                    .resolved_layout_with_text_measurer(text_measurer)
                    .expect("document layout can be resolved")
            } else {
                input_layout
            };
            let scroll_chrome = if input_update.changed
                || final_style_report.paint_changed
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
        self.cached_document_instance_id = Some(document.instance_id());
        self.cached_document_revision = Some(document.revision());
        self.cached_document_stylesheet = Some(stylesheet.clone());
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
                style_nodes_visited,
                reused_cached_layout: false,
                reused_input_layout,
                input_changed_state: input_update.changed || clamp_changed,
                animation_changed_style: animation_update.changed()
                    || final_style_report.changed()
                    || scrollbar_animation_update.changed(),
                animation_changed_layout: animation_update.layout_changed
                    || final_style_report.layout_changed,
                animation_changed_paint: animation_update.paint_changed
                    || final_style_report.paint_changed
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

    pub fn scroll_position(&self, id: &str) -> Option<Point> {
        let state = self.states.get(&ElementId::new(id))?;
        Some(Point::new(state.scroll_x, state.scroll_y))
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

        let scroll = des_layout::scroll::clamp_scroll_offset(
            to_layout_point(Point::new(
                state.scroll_x + delta.x,
                state.scroll_y + delta.y,
            )),
            des_layout::geometry::Size {
                width: max_scroll.width,
                height: max_scroll.height,
            },
        );
        let mut changed = false;
        changed |= set_f32(&mut state.scroll_x, scroll.x);
        changed |= set_f32(&mut state.scroll_y, scroll.y);
        if changed {
            self.cached_layout = None;
        }
        changed
    }

    pub fn scroll_element_to(&mut self, id: &str, scroll: Point) -> bool {
        let id = ElementId::new(id);
        let max_scroll = self.scroll_limits.get(&id).copied().unwrap_or_default();
        let Some(state) = self.states.get_mut(&id) else {
            return false;
        };

        let scroll = des_layout::scroll::clamp_scroll_offset(
            to_layout_point(scroll),
            des_layout::geometry::Size {
                width: max_scroll.width,
                height: max_scroll.height,
            },
        );
        let mut changed = false;
        changed |= set_f32(&mut state.scroll_x, scroll.x);
        changed |= set_f32(&mut state.scroll_y, scroll.y);
        if changed {
            self.cached_layout = None;
        }
        changed
    }

    fn cached_document_matches(
        &self,
        document: &Document,
        stylesheet: &StyleSheet,
        text_measurer_key: TextMeasurerKey,
    ) -> bool {
        self.cached_layout.is_some()
            && self.cached_document_instance_id == Some(document.instance_id())
            && self.cached_document_revision == Some(document.revision())
            && self.cached_document_stylesheet.as_ref() == Some(stylesheet)
            && self.cached_text_measurer_key == Some(text_measurer_key)
    }

    fn sync_document_states(&mut self, document: &Document) -> ChangeSet {
        let next_ids = document.element_ids().into_iter().collect::<BTreeSet<_>>();
        let existing_ids: BTreeSet<_> = self.states.keys().cloned().collect();
        let mut changes = ChangeSet::default();

        for id in &next_ids {
            if existing_ids.contains(id) {
                changes.retained.push(id.clone());
            } else {
                changes.created.push(id.clone());
                let initial_scroll = document
                    .element_spec(id)
                    .ok()
                    .and_then(|spec| spec.initial_scroll)
                    .unwrap_or(Point::ZERO);
                self.states.insert(
                    id.clone(),
                    ElementState {
                        scroll_x: initial_scroll.x,
                        scroll_y: initial_scroll.y,
                        ..ElementState::default()
                    },
                );
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
        text_measurer: &mut dyn TextMeasurer,
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
            update.changed |= self.update_active_text_selection(pointer, layout, text_measurer);
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
                frame.style.overflow_x.is_scrollable() || frame.style.overflow_y.is_scrollable()
            })
            && let Some(state) = self.states.get_mut(&scroll_frame.id)
        {
            let max_scroll = self
                .scroll_limits
                .get(&scroll_frame.id)
                .copied()
                .unwrap_or_default();
            if scroll_frame.style.overflow_x.is_scrollable() {
                let scroll_x = des_layout::scroll::clamp_scroll_value(
                    state.scroll_x - input.scroll_delta.x,
                    max_scroll.width,
                );
                update.changed |= set_f32(&mut state.scroll_x, scroll_x);
            }
            if scroll_frame.style.overflow_y.is_scrollable() {
                let scroll_y = des_layout::scroll::clamp_scroll_value(
                    state.scroll_y - input.scroll_delta.y,
                    max_scroll.height,
                );
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
            if pointer.secondary_clicked {
                update
                    .events
                    .push(DocumentEvent::context_requested(frame.id.clone()));
                return finalize_input_update(update, &self.states, &previous);
            }
            if pointer.primary_down {
                update.changed |= self.start_text_selection(frame, pointer, text_measurer);
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
        if pointer.secondary_clicked
            && let Some(hit_id) = &update.hit_id
        {
            update
                .events
                .push(DocumentEvent::context_requested(hit_id.clone()));
        }
        if let Some(hit_id) = &update.hit_id
            && let Some(state) = self.states.get_mut(hit_id)
        {
            state.pressed = pointer.primary_down;
            if primary_click_fired(pointer) {
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

    fn start_text_selection(
        &mut self,
        frame: &ResolvedElement,
        pointer: PointerInput,
        text_measurer: &mut dyn TextMeasurer,
    ) -> bool {
        let text_index = text_index_at_point(frame, pointer.position, text_measurer);
        let click_count = self.text_click_count(frame, pointer);
        let granularity = match click_count {
            3..=u8::MAX => TextSelectionGranularity::Paragraph,
            2 => TextSelectionGranularity::Word,
            _ => TextSelectionGranularity::Character,
        };
        let (anchor_index, focus_index) = selection_range_for_granularity(
            &frame
                .text
                .as_ref()
                .map(|text| text.semantic_text())
                .unwrap_or_default(),
            text_index,
            granularity,
        );
        let next = DocumentTextSelection {
            target: frame.id.clone(),
            anchor: pointer.position,
            focus: pointer.position,
            anchor_index,
            focus_index,
            anchor_range_start: anchor_index.min(focus_index),
            anchor_range_end: anchor_index.max(focus_index),
            granularity,
            active: true,
        };
        let changed = self.text_selection.as_ref() != Some(&next);
        self.text_selection = Some(next);
        self.active_pointer_drag = None;
        changed
    }

    fn text_click_count(&mut self, frame: &ResolvedElement, pointer: PointerInput) -> u8 {
        if !pointer.primary_pressed {
            return pointer.primary_click_count.max(1);
        }

        let count = self
            .last_text_click
            .as_ref()
            .filter(|click| click.target == frame.id)
            .filter(|click| {
                pointer.time_seconds - click.time_seconds <= TEXT_CLICK_INTERVAL_SECONDS
            })
            .filter(|click| point_distance(click.position, pointer.position) <= TEXT_CLICK_DISTANCE)
            .map(|click| click.count.saturating_add(1).min(3))
            .unwrap_or(1);
        self.last_text_click = Some(TextClickSequence {
            target: frame.id.clone(),
            position: pointer.position,
            time_seconds: pointer.time_seconds,
            count,
        });
        count
    }

    fn update_active_text_selection(
        &mut self,
        pointer: PointerInput,
        layout: &ResolvedElement,
        text_measurer: &mut dyn TextMeasurer,
    ) -> bool {
        let Some(selection) = self.text_selection.as_mut() else {
            return false;
        };
        if pointer.primary_down {
            let mut changed = false;
            let focus_index = layout
                .find(selection.target.as_str())
                .map(|frame| {
                    let text_index = text_index_at_point(frame, pointer.position, text_measurer);
                    let (anchor_index, focus_index) = selection_indices_for_granularity(
                        &frame
                            .text
                            .as_ref()
                            .map(|text| text.semantic_text())
                            .unwrap_or_default(),
                        selection.anchor_range_start,
                        selection.anchor_range_end,
                        text_index,
                        selection.granularity,
                    );
                    selection.anchor_index = anchor_index;
                    focus_index
                })
                .unwrap_or(selection.focus_index);
            changed |= set_point(&mut selection.focus, pointer.position);
            changed |= set_usize(&mut selection.focus_index, focus_index);
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
            let axis = to_scroll_axis(chrome.axis);
            let pointer_main = axis.position(to_layout_point(pointer.position));
            let handle_rect = to_scroll_rect(chrome.handle_rect);
            let handle_start = axis.rect_origin(handle_rect);
            let handle_length = axis.rect_length(handle_rect);
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
        let scroll_offset = des_layout::scroll::scroll_offset_from_handle_drag(
            to_scroll_axis(chrome.axis),
            to_scroll_rect(chrome.track_rect),
            to_scroll_rect(chrome.handle_rect),
            to_layout_point(pointer.position),
            drag.pointer_offset_from_handle_start,
            chrome.max_scroll,
        );
        if let Some(state) = self.states.get_mut(&chrome.element_id) {
            match chrome.axis {
                ScrollAxis::Horizontal => changed |= set_f32(&mut state.scroll_x, scroll_offset),
                ScrollAxis::Vertical => changed |= set_f32(&mut state.scroll_y, scroll_offset),
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
            let scroll = des_layout::scroll::clamp_scroll_offset(
                to_layout_point(Point::new(state.scroll_x, state.scroll_y)),
                des_layout::geometry::Size {
                    width: max_scroll.width,
                    height: max_scroll.height,
                },
            );
            changed |= set_f32(&mut state.scroll_x, scroll.x);
            changed |= set_f32(&mut state.scroll_y, scroll.y);
        }
        changed
    }

    fn update_style_animation(
        &mut self,
        document: &Document,
        stylesheet: &StyleSheet,
    ) -> AnimationUpdate {
        const SNAP_EPSILON: f32 = 0.001;
        let root = document
            .element_tree()
            .expect("document element tree can be resolved");
        update_element_style_animation(
            &root,
            stylesheet,
            &mut self.states,
            SNAP_EPSILON,
            document.viewport(),
            &|id| document.parent_container_size(id).ok().flatten(),
        )
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

fn set_usize(target: &mut usize, value: usize) -> bool {
    let changed = *target != value;
    *target = value;
    changed
}

fn set_point(target: &mut Point, value: Point) -> bool {
    let changed = *target != value;
    *target = value;
    changed
}

fn point_distance(left: Point, right: Point) -> f32 {
    let dx = left.x - right.x;
    let dy = left.y - right.y;
    (dx * dx + dy * dy).sqrt()
}

fn text_index_at_point(
    frame: &ResolvedElement,
    point: Point,
    text_measurer: &mut dyn TextMeasurer,
) -> usize {
    let Some(text) = frame.text.as_ref() else {
        return 0;
    };
    let text_rect = text_content_rect(frame);
    let local_point = Point::new(point.x - text_rect.origin.x, point.y - text_rect.origin.y);
    let normalized = NormalizedText::from_content(text, frame.style.text_layout);
    let wrap_width = match frame.style.text_layout.text_wrap_mode {
        crate::text::TextWrapMode::NoWrap => f32::INFINITY,
        crate::text::TextWrapMode::Wrap => text_rect.size.width,
    };
    text_measurer.text_index_at(
        TextLayoutRequest {
            text: &normalized,
            font_size: frame.style.font_size,
            color: frame.style.text_color,
            wrap_width,
            layout_style: frame.style.text_layout,
            line_height: frame.style.line_height,
        },
        local_point,
    )
}

fn text_content_rect(frame: &ResolvedElement) -> Rect {
    frame
        .rect
        .inset(frame.style.border_width)
        .inset(frame.style.padding)
}

fn selection_range_for_granularity(
    text: &str,
    index: usize,
    granularity: TextSelectionGranularity,
) -> (usize, usize) {
    match granularity {
        TextSelectionGranularity::Character => (index, index),
        TextSelectionGranularity::Word => word_range_at(text, index),
        TextSelectionGranularity::Paragraph => paragraph_range_at(text, index),
    }
}

fn selection_indices_for_granularity(
    text: &str,
    anchor_start: usize,
    anchor_end: usize,
    focus_index: usize,
    granularity: TextSelectionGranularity,
) -> (usize, usize) {
    match granularity {
        TextSelectionGranularity::Character => (anchor_start, focus_index),
        TextSelectionGranularity::Word => {
            let (start, end) = word_range_at(text, focus_index);
            if focus_index < anchor_start {
                (anchor_end, start)
            } else if focus_index > anchor_end {
                (anchor_start, end)
            } else {
                (anchor_start, anchor_end)
            }
        }
        TextSelectionGranularity::Paragraph => {
            let (start, end) = paragraph_range_at(text, focus_index);
            if focus_index < anchor_start {
                (anchor_end, start)
            } else if focus_index > anchor_end {
                (anchor_start, end)
            } else {
                (anchor_start, anchor_end)
            }
        }
    }
}

fn word_range_at(text: &str, index: usize) -> (usize, usize) {
    let chars: Vec<char> = text.chars().collect();
    if chars.is_empty() {
        return (0, 0);
    }
    let mut cursor = index.min(chars.len().saturating_sub(1));
    if cursor > 0 && !is_word_char(chars[cursor]) && is_word_char(chars[cursor - 1]) {
        cursor -= 1;
    }
    if !is_word_char(chars[cursor]) {
        return (index.min(chars.len()), index.min(chars.len()));
    }

    let mut start = cursor;
    while start > 0 && is_word_char(chars[start - 1]) {
        start -= 1;
    }
    let mut end = cursor + 1;
    while end < chars.len() && is_word_char(chars[end]) {
        end += 1;
    }
    (start, end)
}

fn paragraph_range_at(text: &str, index: usize) -> (usize, usize) {
    let chars: Vec<char> = text.chars().collect();
    if chars.is_empty() {
        return (0, 0);
    }
    let cursor = index.min(chars.len());
    let mut start = cursor;
    while start > 0 && chars[start - 1] != '\n' {
        start -= 1;
    }
    let mut end = cursor;
    while end < chars.len() && chars[end] != '\n' {
        end += 1;
    }
    (start, end)
}

fn is_word_char(value: char) -> bool {
    value.is_alphanumeric() || value == '_'
}

fn drag_delta_is_click(drag: &DocumentDrag) -> bool {
    drag.delta.x.abs() <= f32::EPSILON && drag.delta.y.abs() <= f32::EPSILON
}

fn primary_click_fired(pointer: PointerInput) -> bool {
    pointer.primary_clicked || pointer.primary_click_count > 0
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
    update.selector_state_changed |= selector_state_changed(states, previous);
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

fn selector_state_changed(
    states: &HashMap<ElementId, ElementState>,
    previous: &HashMap<ElementId, InteractionSnapshot>,
) -> bool {
    states.iter().any(|(id, state)| {
        previous.get(id).is_some_and(|previous| {
            state.hovered != previous.hovered
                || state.pressed != previous.pressed
                || state.dragging != previous.dragging
                || state.scrollbar_hovered_axis != previous.scrollbar_hovered_axis
                || state.scrollbar_dragged_axis != previous.scrollbar_dragged_axis
        })
    })
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
