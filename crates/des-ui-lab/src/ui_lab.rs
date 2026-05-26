mod html;
mod shadow_styler;
mod styles;
#[cfg(test)]
mod tests;
mod views;

use des_egui::adapter::{
    CosmicTextPaintResources, TextPaintStats, configure_text_selection_input,
    copy_selected_text_on_command, document_input, paint_frame_with_text_resources,
    paint_scroll_chrome,
};
use styles::stylesheet;
use views::{
    DragLabState, StageRenderState, render_debug_overlay_layer, render_drag_overlay_layer,
    render_stage,
};

use des_document::{
    Color, Document, DocumentCommandRegistry, DocumentDrag, DocumentEngine, DocumentEventKind,
    DocumentInput, DocumentMetrics, DocumentOutput, DocumentProjection, DocumentWidget, Element,
    ElementId, ElementSpec, Length, Point, PointerInput, Rect, Size, Style, StyleSelector,
    StyleSheet, VisualCloneOptions, VisualElementClone,
};
use des_widgets::{
    AutoScrollOptions, AutoScroller, ContextMenu, DropZoneId, SortableDocumentConfig,
    SortableDropPreview, SortableItemId, SortableModel,
};
use eframe::egui;
use shadow_styler::{ShadowStyler, ShadowStylerAction, ShadowTuneState, ShadowTuneTarget};
use std::time::{Duration, Instant};

#[cfg(test)]
const PANEL: Color = Color::rgb(255, 251, 254);
#[cfg(test)]
const STROKE: Color = Color::rgb(202, 196, 208);
const TEXT: Color = Color::rgb(29, 27, 32);
#[cfg(test)]
const PURPLE: Color = Color::rgb(103, 80, 164);
#[cfg(test)]
const SURFACE_CONTAINER: Color = Color::rgb(243, 237, 247);
#[cfg(test)]
const PRIMARY_CONTAINER: Color = Color::rgb(234, 221, 255);
#[cfg(test)]
const SECONDARY_CONTAINER: Color = Color::rgb(232, 222, 248);
#[cfg(test)]
const SUCCESS_CONTAINER: Color = Color::rgb(205, 239, 221);
const SHADOW_COLOR: Color = Color::rgb(0, 0, 0);
const ANIMATION_FRAME_TIME: Duration = Duration::from_millis(16);
const DRAG_ITEM_COUNT: usize = 3;
const DROP_ZONE_COUNT: usize = 6;
const SCROLL_LIST_ITEM_COUNT: usize = 14;
const TEXT_CONTEXT_MENU_ID: &str = "text-context-menu";
const TEXT_CONTEXT_MENU_COPY_ID: &str = "text-context-menu-copy";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LabView {
    Layout,
    Interaction,
    Draggable,
    Styling,
    Animation,
    Scrolling,
    Floating,
    Table,
    Text,
    Nesting,
    Graph,
}

impl LabView {
    fn from_id(id: &str) -> Option<Self> {
        match id {
            "layout" | "view-layout" => Some(Self::Layout),
            "interaction" | "view-interaction" => Some(Self::Interaction),
            "draggable" | "view-draggable" => Some(Self::Draggable),
            "styling" | "view-styling" => Some(Self::Styling),
            "animation" | "view-animation" => Some(Self::Animation),
            "scrolling" | "view-scrolling" => Some(Self::Scrolling),
            "floating" | "view-floating" => Some(Self::Floating),
            "table" | "view-table" => Some(Self::Table),
            "text" | "view-text" => Some(Self::Text),
            "nesting" | "view-nesting" => Some(Self::Nesting),
            "graph" | "view-graph" => Some(Self::Graph),
            _ => None,
        }
    }

    fn id(self) -> &'static str {
        match self {
            Self::Layout => "view-layout",
            Self::Interaction => "view-interaction",
            Self::Draggable => "view-draggable",
            Self::Styling => "view-styling",
            Self::Animation => "view-animation",
            Self::Scrolling => "view-scrolling",
            Self::Floating => "view-floating",
            Self::Table => "view-table",
            Self::Text => "view-text",
            Self::Nesting => "view-nesting",
            Self::Graph => "view-graph",
        }
    }
}

fn lab_subtitle(debug_overlay: bool) -> &'static str {
    if debug_overlay {
        "document layout, style, input, and graph experiments / debug"
    } else {
        "document layout, style, input, and graph experiments"
    }
}

fn lab_asset_revision() -> u64 {
    styles::asset_revision().wrapping_mul(31) ^ html::asset_revision()
}

pub(crate) struct UiLabState {
    document_engine: DocumentEngine,
    stylesheet: StyleSheet,
    asset_revision: u64,
    command_registry: DocumentCommandRegistry<LabAction>,
    view: LabView,
    show_optional_card: bool,
    dense_mode: bool,
    checkbox_enabled: bool,
    radio_choice: usize,
    dropdown_open: bool,
    dropdown_choice: usize,
    loop_action_count: usize,
    shadow_tune: ShadowTuneState,
    shadow_hover_tune: ShadowTuneState,
    drag_item_cells: [usize; 3],
    drag_item_order: [usize; 3],
    scroll_list_item_order: [usize; SCROLL_LIST_ITEM_COUNT],
    active_drag: Option<DocumentDrag>,
    drag_parent_offset: Option<Point>,
    drag_source_size: Option<Size>,
    drag_visual_clone: Option<VisualElementClone>,
    pressed_drag_source: Option<String>,
    drag_drop_preview: Option<SortableDropPreview>,
    scroll_list_drop_preview: Option<SortableDropPreview>,
    text_context_menu: Option<TextContextMenu>,
    text_paint_resources: CosmicTextPaintResources,
    pending_stage_scroll: Option<Point>,
    lab_document: Option<RetainedLabDocument<LabDocumentKey>>,
    last_output: Option<RetainedLabOutput<LabDocumentKey>>,
    last_perf: UiLabPerf,
}

struct RetainedLabDocument<Key> {
    viewport: Size,
    debug_overlay: bool,
    key: Key,
    document: Document,
}

struct RetainedLabOutput<Key> {
    viewport: Size,
    debug_overlay: bool,
    key: Key,
    output: DocumentOutput,
}

#[derive(Clone, Debug, PartialEq)]
struct LabDocumentKey {
    view: LabView,
    show_optional_card: bool,
    dense_mode: bool,
    checkbox_enabled: bool,
    radio_choice: usize,
    dropdown_open: bool,
    dropdown_choice: usize,
    loop_action_count: usize,
    shadow_tune: ShadowTuneState,
    shadow_hover_tune: ShadowTuneState,
    drag_item_cells: [usize; DRAG_ITEM_COUNT],
    drag_item_order: [usize; DRAG_ITEM_COUNT],
    scroll_list_item_order: [usize; SCROLL_LIST_ITEM_COUNT],
    pressed_drag_source: Option<String>,
    active_drag_item: Option<SortableItemId>,
    active_scroll_list_drag_item: Option<SortableItemId>,
    drag_drop_preview: Option<SortableDropPreview>,
    scroll_list_drop_preview: Option<SortableDropPreview>,
    drag_overlay_active: bool,
    drag_visual_clone: Option<VisualElementClone>,
    text_context_menu: Option<TextContextMenu>,
    asset_revision: u64,
}

#[derive(Clone, Debug, PartialEq)]
struct TextContextMenu {
    target: ElementId,
    position: Point,
    selected_text: Option<String>,
}

impl Default for UiLabState {
    fn default() -> Self {
        Self {
            document_engine: DocumentEngine::default(),
            stylesheet: stylesheet(),
            asset_revision: lab_asset_revision(),
            command_registry: lab_action_registry(),
            view: LabView::Layout,
            show_optional_card: true,
            dense_mode: false,
            checkbox_enabled: true,
            radio_choice: 0,
            dropdown_open: false,
            dropdown_choice: 1,
            loop_action_count: 0,
            shadow_tune: ShadowTuneState::default(),
            shadow_hover_tune: ShadowTuneState::hover_default(),
            drag_item_cells: [0, 2, 4],
            drag_item_order: [0, 1, 2],
            scroll_list_item_order: core::array::from_fn(|index| index),
            active_drag: None,
            drag_parent_offset: None,
            drag_source_size: None,
            drag_visual_clone: None,
            pressed_drag_source: None,
            drag_drop_preview: None,
            scroll_list_drop_preview: None,
            text_context_menu: None,
            text_paint_resources: CosmicTextPaintResources::new(des_egui::document_text_renderer()),
            pending_stage_scroll: None,
            lab_document: None,
            last_output: None,
            last_perf: UiLabPerf::default(),
        }
    }
}

impl UiLabState {
    pub(crate) fn new(initial_view: Option<&str>) -> Self {
        Self::new_with_stage_scroll(initial_view, None)
    }

    pub(crate) fn new_with_stage_scroll(
        initial_view: Option<&str>,
        stage_scroll: Option<Point>,
    ) -> Self {
        let mut state = Self::default();
        if let Some(view) = initial_view.and_then(LabView::from_id) {
            state.view = view;
        }
        state.pending_stage_scroll = stage_scroll;
        state
    }

    pub(crate) fn render(&mut self, ui: &mut egui::Ui, debug_overlay: bool) {
        self.reload_lab_assets_if_changed();
        configure_text_selection_input(ui.ctx());
        let origin = ui.max_rect().min;
        let viewport = ui.max_rect().size();
        let viewport_size = Size::new(viewport.x, viewport.y);
        let key = self.lab_document_key();
        self.text_paint_resources.begin_frame();

        let input = document_input(ui, origin);
        let pointer = input.pointer;
        if self.can_reuse_last_output(viewport_size, debug_overlay, &key, &input) {
            let paint_start = Instant::now();
            let output = &self
                .last_output
                .as_ref()
                .expect("reuse check proves retained output exists")
                .output;
            copy_selected_text_on_command(ui, output);
            apply_cursor_icon(ui, output);
            paint_frame_with_text_resources(
                ui,
                origin,
                &output.layout,
                output.text_selection.as_ref(),
                &mut self.text_paint_resources,
            );
            if debug_overlay && self.view == LabView::Text {
                paint_legacy_text_path_comparison(ui, origin, output);
            }
            paint_scroll_chrome(ui, origin, &output.scroll_chrome);
            let paint_time = paint_start.elapsed();
            let text_paint = self.text_paint_resources.stats();
            self.last_perf = UiLabPerf {
                stylesheet_time: Duration::ZERO,
                document_time: Duration::ZERO,
                engine_time: Duration::ZERO,
                paint_time,
                text_paint,
                metrics: output.metrics,
            };
            if debug_overlay {
                self.paint_debug_overlay_document(ui, origin, viewport);
            }
            return;
        }

        let document_start = Instant::now();
        let mut retained = self.take_lab_document(viewport_size, debug_overlay);
        let document_time = document_start.elapsed();

        let stylesheet_start = Instant::now();
        let dynamic_stylesheet = self.dynamic_stylesheet();
        let stylesheet = dynamic_stylesheet.as_ref().unwrap_or(&self.stylesheet);
        let stylesheet_time = stylesheet_start.elapsed();

        if let Some(scroll) = self.pending_stage_scroll.take() {
            self.document_engine
                .update(&mut retained.document, stylesheet);
            self.document_engine.scroll_element_to("stage", scroll);
        }

        let engine_start = Instant::now();
        let mut output = self.document_engine.update_with_input_and_text_measurer(
            &mut retained.document,
            stylesheet,
            input.clone(),
            &mut self.text_paint_resources,
        );
        self.lab_document = Some(retained);

        if self.sync_drag_state(ui, &output) {
            let mut retained = self.take_lab_document(viewport_size, debug_overlay);
            let dynamic_stylesheet = self.dynamic_stylesheet();
            let stylesheet = dynamic_stylesheet.as_ref().unwrap_or(&self.stylesheet);
            output = self.document_engine.update_with_input_and_text_measurer(
                &mut retained.document,
                stylesheet,
                input.clone(),
                &mut self.text_paint_resources,
            );
            self.lab_document = Some(retained);
            self.sync_drag_state(ui, &output);
        }
        if self.sync_drag_press_state(&output, pointer) {
            let mut retained = self.take_lab_document(viewport_size, debug_overlay);
            let dynamic_stylesheet = self.dynamic_stylesheet();
            let stylesheet = dynamic_stylesheet.as_ref().unwrap_or(&self.stylesheet);
            output = self.document_engine.update_with_input_and_text_measurer(
                &mut retained.document,
                stylesheet,
                input.clone(),
                &mut self.text_paint_resources,
            );
            self.lab_document = Some(retained);
            self.sync_drag_state(ui, &output);
        }
        if self.apply_clicked_document_actions(ui, &output) {
            let mut retained = self.take_lab_document(viewport_size, debug_overlay);
            let dynamic_stylesheet = self.dynamic_stylesheet();
            let stylesheet = dynamic_stylesheet.as_ref().unwrap_or(&self.stylesheet);
            output = self.document_engine.update_with_input_and_text_measurer(
                &mut retained.document,
                stylesheet,
                repaint_input_after_action(input),
                &mut self.text_paint_resources,
            );
            self.lab_document = Some(retained);
            self.sync_drag_state(ui, &output);
            ui.ctx().request_repaint();
        }
        let engine_time = engine_start.elapsed();

        copy_selected_text_on_command(ui, &output);
        apply_cursor_icon(ui, &output);

        let paint_start = Instant::now();
        paint_frame_with_text_resources(
            ui,
            origin,
            &output.layout,
            output.text_selection.as_ref(),
            &mut self.text_paint_resources,
        );
        if debug_overlay && self.view == LabView::Text {
            paint_legacy_text_path_comparison(ui, origin, &output);
        }
        paint_scroll_chrome(ui, origin, &output.scroll_chrome);
        let paint_time = paint_start.elapsed();
        let text_paint = self.text_paint_resources.stats();
        self.last_perf = UiLabPerf {
            stylesheet_time,
            document_time,
            engine_time,
            paint_time,
            text_paint,
            metrics: output.metrics,
        };
        self.apply_document_events(ui, &output, pointer);
        if output.animating {
            ui.ctx().request_repaint_after(ANIMATION_FRAME_TIME);
        }
        if debug_overlay {
            self.paint_debug_overlay_document(ui, origin, viewport);
        }
        self.last_output = Some(RetainedLabOutput {
            viewport: viewport_size,
            debug_overlay,
            key,
            output,
        });
    }

    fn reload_lab_assets_if_changed(&mut self) {
        let revision = lab_asset_revision();
        if revision == self.asset_revision {
            return;
        }

        self.stylesheet = stylesheet();
        self.asset_revision = revision;
        self.lab_document = None;
        self.last_output = None;
    }

    fn can_reuse_last_output(
        &self,
        viewport: Size,
        debug_overlay: bool,
        key: &LabDocumentKey,
        input: &DocumentInput,
    ) -> bool {
        if !inert_pointer_move(input)
            || self.pending_stage_scroll.is_some()
            || self.active_drag.is_some()
            || self.dropdown_open
            || self.text_context_menu.is_some()
        {
            return false;
        }
        let Some(retained) = self.last_output.as_ref() else {
            return false;
        };
        if retained.viewport != viewport
            || retained.debug_overlay != debug_overlay
            || &retained.key != key
        {
            return false;
        }
        let output = &retained.output;
        if output.animating
            || output.active_drag.is_some()
            || output.completed_drag.is_some()
            || !output.events.is_empty()
            || output
                .text_selection
                .as_ref()
                .is_some_and(|selection| selection.active)
        {
            return false;
        }
        let Some(pointer) = input.pointer else {
            return false;
        };
        let Some(previous_hit) = output.hit_id.as_ref() else {
            return false;
        };
        let Some(hit) = deepest_frame_at(&output.layout, pointer.position) else {
            return false;
        };
        hit.id == *previous_hit && !hit.interactive && !hit.selectable_text
    }

    fn take_lab_document(
        &mut self,
        viewport: Size,
        debug_overlay: bool,
    ) -> RetainedLabDocument<LabDocumentKey> {
        let key = self.lab_document_key();
        if let Some(retained) = self.lab_document.take()
            && retained.viewport == viewport
            && retained.debug_overlay == debug_overlay
            && retained.key == key
        {
            return retained;
        }

        RetainedLabDocument {
            viewport,
            debug_overlay,
            key,
            document: self.document(viewport, debug_overlay),
        }
    }

    fn lab_document_key(&self) -> LabDocumentKey {
        LabDocumentKey {
            view: self.view,
            show_optional_card: self.show_optional_card,
            dense_mode: self.dense_mode,
            checkbox_enabled: self.checkbox_enabled,
            radio_choice: self.radio_choice,
            dropdown_open: self.dropdown_open,
            dropdown_choice: self.dropdown_choice,
            loop_action_count: self.loop_action_count,
            shadow_tune: self.shadow_tune,
            shadow_hover_tune: self.shadow_hover_tune,
            drag_item_cells: self.drag_item_cells,
            drag_item_order: self.drag_item_order,
            scroll_list_item_order: self.scroll_list_item_order,
            pressed_drag_source: self.pressed_drag_source.clone(),
            active_drag_item: self.active_drag_item(),
            active_scroll_list_drag_item: self.active_scroll_list_drag_item(),
            drag_drop_preview: self.drag_drop_preview,
            scroll_list_drop_preview: self.scroll_list_drop_preview,
            drag_overlay_active: self.active_drag.is_some(),
            drag_visual_clone: self.drag_visual_clone.clone(),
            text_context_menu: self.text_context_menu.clone(),
            asset_revision: self.asset_revision,
        }
    }

    fn apply_document_events(
        &mut self,
        ui: &egui::Ui,
        output: &DocumentOutput,
        pointer: Option<PointerInput>,
    ) {
        let was_dropdown_open = self.dropdown_open;
        let was_text_context_menu_open = self.text_context_menu.is_some();
        let primary_clicked = pointer
            .map(|pointer| pointer.primary_clicked)
            .unwrap_or(false);
        self.sync_drag_press_state(output, pointer);
        self.sync_drag_state(ui, output);
        if let Some(drag) = &output.completed_drag {
            self.finish_drag(output, drag);
            ui.ctx().request_repaint();
        }
        for event in &output.events {
            match event.kind {
                DocumentEventKind::Pressed
                    if source_item_element_id(event.target.as_str()).is_some() =>
                {
                    ui.ctx().request_repaint();
                }
                DocumentEventKind::ContextRequested => {
                    if let Some(pointer) = pointer
                        && output
                            .snapshot()
                            .find(event.target.as_str())
                            .is_some_and(|frame| frame.selectable_text())
                    {
                        self.text_context_menu = Some(TextContextMenu {
                            target: event.target.clone(),
                            position: pointer.position,
                            selected_text: output.selected_text(),
                        });
                        ui.ctx().request_repaint();
                    }
                }
                _ => {}
            }
        }

        if was_dropdown_open
            && self.dropdown_open
            && primary_clicked
            && !is_dropdown_hit(&output.hit_id)
        {
            self.dropdown_open = false;
            ui.ctx().request_repaint();
        }
        if was_text_context_menu_open
            && self.text_context_menu.is_some()
            && primary_clicked
            && !is_text_context_menu_hit(&output.hit_id)
        {
            self.text_context_menu = None;
            ui.ctx().request_repaint();
        }
    }

    fn apply_clicked_document_actions(&mut self, ui: &egui::Ui, output: &DocumentOutput) -> bool {
        let mut changed = false;
        let mut handled_commands = Vec::new();
        let command_actions = self
            .command_registry
            .command_actions_of_kind(output, DocumentEventKind::Clicked)
            .map(|command| (command.target.clone(), command.event, *command.action))
            .collect::<Vec<_>>();
        for (target, event, action) in command_actions {
            self.apply_triggered_lab_action(ui, action);
            handled_commands.push((target, event));
            changed = true;
        }
        for command in output.commands_of_kind(DocumentEventKind::Clicked) {
            if !handled_commands
                .iter()
                .any(|(target, kind)| *kind == command.event && target == command.target)
                && let Some(action) = lab_action_for_command(command.command)
            {
                self.apply_triggered_lab_action(ui, action);
                handled_commands.push((command.target.clone(), command.event));
                changed = true;
            }
        }
        for event in &output.events {
            if event.kind == DocumentEventKind::Clicked
                && !handled_commands
                    .iter()
                    .any(|(target, kind)| *kind == event.kind && target == &event.target)
                && let Some(action) = lab_action_for_id(event.target.as_str())
            {
                self.apply_triggered_lab_action(ui, action);
                changed = true;
            }
        }
        changed
    }

    fn apply_triggered_lab_action(&mut self, ui: &egui::Ui, action: LabAction) {
        match action {
            LabAction::CopyTextSelection => {
                if let Some(text) = self
                    .text_context_menu
                    .as_ref()
                    .and_then(|menu| menu.selected_text.clone())
                    .filter(|text| !text.is_empty())
                {
                    ui.ctx().copy_text(text);
                }
                self.text_context_menu = None;
            }
            _ => self.apply_lab_action(action),
        }
    }

    fn sync_drag_state(&mut self, ui: &egui::Ui, output: &DocumentOutput) -> bool {
        let previous_drag = self.active_drag.clone();
        self.active_drag = output.active_drag.clone();
        if previous_drag.is_none()
            && let Some(drag) = &self.active_drag
            && let Some(item_id) = source_item_element_id(drag.target.as_str())
            && let Some(rect) = output
                .snapshot()
                .find(item_id.as_str())
                .map(|element| element.rect())
        {
            self.drag_visual_clone = output
                .snapshot()
                .find(item_id.as_str())
                .map(|element| element.visual_clone());
            self.drag_parent_offset = Some(Point::new(
                drag.origin.x - rect.origin.x,
                drag.origin.y - rect.origin.y,
            ));
            self.drag_source_size = Some(rect.size);
            self.snap_drag_pickup_animation(item_id.as_str());
        }
        self.drag_drop_preview = None;
        self.scroll_list_drop_preview = None;
        if let Some(drag) = &self.active_drag {
            if let Some(active_item) = active_grid_drag_item(drag.target.as_str()) {
                self.drag_drop_preview = sortable_config()
                    .preview_at(output, drag.current, Some(active_item))
                    .filter(|preview| {
                        self.sortable_model()
                            .preview_changes_position(active_item, *preview)
                    });
            } else if let Some(active_item) = active_scroll_list_drag_item(drag.target.as_str()) {
                self.scroll_list_drop_preview = scroll_list_config()
                    .preview_at(output, drag.current, Some(active_item))
                    .filter(|preview| {
                        self.scroll_list_model()
                            .preview_changes_position(active_item, *preview)
                    });
            }
        }
        if let Some(drag) = &self.active_drag
            && AutoScroller::new(AutoScrollOptions {
                threshold_x: 0.0,
                threshold_y: 0.24,
                acceleration: 10.0,
                ..AutoScrollOptions::default()
            })
            .scroll_drag_with_filter(&mut self.document_engine, output, drag.current, |id| {
                id.as_str().starts_with("drag-scroll-list-")
            })
            .is_some()
        {
            ui.ctx().request_repaint_after(ANIMATION_FRAME_TIME);
        }
        if self.active_drag != previous_drag {
            ui.ctx().request_repaint();
        }
        previous_drag.is_none() && self.active_drag.is_some()
    }

    fn sync_drag_press_state(
        &mut self,
        output: &DocumentOutput,
        pointer: Option<PointerInput>,
    ) -> bool {
        let next = if output.active_drag.is_none()
            && pointer.is_some_and(|pointer| pointer.primary_down)
        {
            output
                .hit_id
                .as_ref()
                .and_then(|id| source_item_element_id(id.as_str()))
        } else {
            None
        };
        if self.pressed_drag_source == next {
            return false;
        }
        self.pressed_drag_source = next;
        true
    }

    fn snap_drag_pickup_animation(&mut self, item_id: &str) {
        self.document_engine.snap_element_animation(item_id);
        if let Some(clone) = &self.drag_visual_clone {
            for id in clone.source_ids() {
                self.document_engine.snap_element_animation(id.as_str());
            }
            let options =
                VisualCloneOptions::new("drag-overlay", "drag-overlay/").root_class("drag-overlay");
            for id in clone.cloned_ids(&options) {
                self.document_engine.snap_element_animation(id.as_str());
            }
        }
    }

    fn finish_drag(&mut self, output: &DocumentOutput, drag: &DocumentDrag) {
        if let Some(item) = active_grid_drag_item(drag.target.as_str()) {
            let preview = sortable_config()
                .preview_at(output, drag.current, Some(item))
                .filter(|preview| {
                    self.sortable_model()
                        .preview_changes_position(item, *preview)
                });
            if let Some(preview) = preview {
                let mut model = self.sortable_model();
                model.apply_drop(item, preview);
                self.apply_sortable_model(&model);
            }
            self.snap_drag_drop_animation(item, preview);
        } else if let Some(item) = active_scroll_list_drag_item(drag.target.as_str()) {
            let preview = scroll_list_config()
                .preview_at(output, drag.current, Some(item))
                .filter(|preview| {
                    self.scroll_list_model()
                        .preview_changes_position(item, *preview)
                });
            if let Some(preview) = preview {
                let mut model = self.scroll_list_model();
                model.apply_drop(item, preview);
                self.apply_scroll_list_model(&model);
            }
            scroll_list_config().snap_drop_animation(&mut self.document_engine, item, preview);
        }
        self.active_drag = None;
        self.drag_parent_offset = None;
        self.drag_source_size = None;
        self.drag_visual_clone = None;
        self.pressed_drag_source = None;
        self.drag_drop_preview = None;
        self.scroll_list_drop_preview = None;
    }

    fn snap_drag_drop_animation(
        &mut self,
        item: SortableItemId,
        preview: Option<SortableDropPreview>,
    ) {
        sortable_config().snap_drop_animation(&mut self.document_engine, item, preview);
    }

    fn sortable_model(&self) -> SortableModel {
        SortableModel::new(
            self.drag_item_cells.map(DropZoneId).to_vec(),
            self.drag_item_order.to_vec(),
        )
    }

    fn apply_sortable_model(&mut self, model: &SortableModel) {
        for (index, zone) in model.item_zones().iter().enumerate() {
            self.drag_item_cells[index] = zone.0;
        }
        for (index, order) in model.item_order_values().iter().enumerate() {
            self.drag_item_order[index] = *order;
        }
    }

    fn scroll_list_model(&self) -> SortableModel {
        SortableModel::new(
            vec![DropZoneId(0); SCROLL_LIST_ITEM_COUNT],
            self.scroll_list_item_order.to_vec(),
        )
    }

    fn apply_scroll_list_model(&mut self, model: &SortableModel) {
        for (index, order) in model.item_order_values().iter().enumerate() {
            self.scroll_list_item_order[index] = *order;
        }
    }

    #[cfg(test)]
    fn drag_source_placeholder_visible(&self) -> bool {
        self.active_drag.is_some() && self.drag_drop_preview.is_none()
    }

    #[cfg(test)]
    fn clone_for_retained_test(&self) -> Self {
        let mut state = Self::new(Some(self.view.id()));
        state.show_optional_card = self.show_optional_card;
        state.dense_mode = self.dense_mode;
        state.checkbox_enabled = self.checkbox_enabled;
        state.radio_choice = self.radio_choice;
        state.dropdown_open = self.dropdown_open;
        state.dropdown_choice = self.dropdown_choice;
        state.loop_action_count = self.loop_action_count;
        state.shadow_tune = self.shadow_tune;
        state.shadow_hover_tune = self.shadow_hover_tune;
        state.drag_item_cells = self.drag_item_cells;
        state.drag_item_order = self.drag_item_order;
        state.scroll_list_item_order = self.scroll_list_item_order;
        state.active_drag = self.active_drag.clone();
        state.drag_parent_offset = self.drag_parent_offset;
        state.drag_source_size = self.drag_source_size;
        state.drag_visual_clone = self.drag_visual_clone.clone();
        state.pressed_drag_source = self.pressed_drag_source.clone();
        state.drag_drop_preview = self.drag_drop_preview;
        state.scroll_list_drop_preview = self.scroll_list_drop_preview;
        state.text_context_menu = self.text_context_menu.clone();
        state.pending_stage_scroll = self.pending_stage_scroll;
        state
    }

    fn apply_lab_action(&mut self, action: LabAction) {
        match action {
            LabAction::SelectView(view) => {
                self.view = view;
                self.text_context_menu = None;
            }
            LabAction::ToggleOptionalCard => self.show_optional_card = !self.show_optional_card,
            LabAction::ToggleDensity => self.dense_mode = !self.dense_mode,
            LabAction::ToggleCheckbox => self.checkbox_enabled = !self.checkbox_enabled,
            LabAction::SelectRadio(choice) => self.radio_choice = choice,
            LabAction::ToggleDropdown => self.dropdown_open = !self.dropdown_open,
            LabAction::SelectDropdown(choice) => {
                self.dropdown_choice = choice;
                self.dropdown_open = false;
            }
            LabAction::IncrementLoopAction => self.loop_action_count += 1,
            LabAction::ShadowStyler(action) => self.apply_shadow_styler_action(action),
            LabAction::CopyTextSelection => {}
        }
    }

    fn document(&self, viewport: Size, debug_overlay: bool) -> Document {
        let mut document = Document::build(viewport, |ui| {
            ui.element(
                "lab-root",
                ElementSpec::new(Element::Div).class("lab-root"),
                |ui| {
                    html::append_topbar(ui);
                    ui.element(
                        "lab-body",
                        ElementSpec::new(Element::Div).class("lab-body"),
                        |ui| {
                            html::append_nav(ui);
                            render_stage(
                                ui,
                                StageRenderState::new(self.view)
                                    .optional_card(self.show_optional_card)
                                    .dense_mode(self.dense_mode)
                                    .drag(
                                        DragLabState::new(
                                            self.drag_item_cells,
                                            self.drag_item_order,
                                            self.scroll_list_item_order,
                                        )
                                        .active(
                                            self.pressed_drag_source.as_deref(),
                                            self.active_drag_item(),
                                            self.active_scroll_list_drag_item(),
                                            self.active_drag.as_ref().map(|drag| drag.current),
                                        )
                                        .previews(
                                            self.drag_drop_preview,
                                            self.scroll_list_drop_preview,
                                        ),
                                    )
                                    .shadows(self.shadow_tune, self.shadow_hover_tune),
                            );
                        },
                    );
                    render_drag_overlay_layer(
                        ui,
                        self.active_drag.as_ref().map(|drag| drag.current),
                        self.drag_visual_clone.as_ref(),
                    );
                    render_text_context_menu(ui, self.text_context_menu.as_ref());
                },
            );
        });
        document
            .set_text("subtitle", lab_subtitle(debug_overlay))
            .expect("lab HTML subtitle should exist");
        document
            .select(self.view.id())
            .expect("lab HTML nav item should exist");
        if self.view == LabView::Interaction {
            self.apply_interaction_document_state(&mut document);
        }
        document
    }

    fn active_drag_item(&self) -> Option<SortableItemId> {
        self.active_drag
            .as_ref()
            .and_then(|drag| active_grid_drag_item(drag.target.as_str()))
    }

    fn active_scroll_list_drag_item(&self) -> Option<SortableItemId> {
        self.active_drag
            .as_ref()
            .and_then(|drag| active_scroll_list_drag_item(drag.target.as_str()))
    }

    fn active_stylesheet(&self) -> StyleSheet {
        self.dynamic_stylesheet()
            .unwrap_or_else(|| self.stylesheet.clone())
    }

    fn dynamic_stylesheet(&self) -> Option<StyleSheet> {
        if self.active_drag.is_none()
            && self.view != LabView::Styling
            && self.text_context_menu.is_none()
        {
            return None;
        }

        let mut stylesheet = self.stylesheet.clone();
        if let Some(drag) = &self.active_drag {
            let offset = self.drag_parent_offset.unwrap_or(drag.pointer_offset);
            let mut overlay_style = Style::default()
                .absolute_viewport()
                .left(Length::Px(drag.current.x - offset.x))
                .top(Length::Px(drag.current.y - offset.y))
                .z_index(1000);
            if let Some(size) = self.drag_source_size {
                overlay_style = overlay_style.size(size.width, size.height);
            }
            stylesheet.push_rule(StyleSelector::id("drag-overlay"), overlay_style);
        }
        if self.view == LabView::Styling {
            self.shadow_styler().push_styles(&mut stylesheet);
        }
        if let Some(menu) = self.text_context_menu.as_ref() {
            let menu = text_context_menu_widget(menu);
            menu.push_styles(&mut stylesheet);
        }
        Some(stylesheet)
    }

    fn shadow_tune_mut(&mut self, target: ShadowTuneTarget) -> &mut ShadowTuneState {
        match target {
            ShadowTuneTarget::Base => &mut self.shadow_tune,
            ShadowTuneTarget::Hover => &mut self.shadow_hover_tune,
        }
    }

    fn shadow_styler(&self) -> ShadowStyler {
        ShadowStyler::new(self.shadow_tune, self.shadow_hover_tune).shadow_color(SHADOW_COLOR)
    }

    fn apply_shadow_styler_action(&mut self, action: ShadowStylerAction) {
        match action {
            ShadowStylerAction::Adjust {
                target,
                layer,
                field,
                direction,
            } => self.shadow_tune_mut(target).adjust(layer, field, direction),
            ShadowStylerAction::ToggleLayer { target, layer } => {
                self.shadow_tune_mut(target).toggle(layer)
            }
        }
    }

    fn paint_debug_overlay_document(
        &mut self,
        ui: &mut egui::Ui,
        origin: egui::Pos2,
        viewport: egui::Vec2,
    ) {
        let mut document = Document::build(Size::new(viewport.x, viewport.y), |ui| {
            ui.element(
                "debug-overlay-root",
                ElementSpec::new(Element::Div).class("debug-overlay-root"),
                |ui| {
                    render_debug_overlay_layer(ui, self.last_perf);
                },
            );
        });
        let mut engine = DocumentEngine::default();
        let stylesheet = self.active_stylesheet();
        let output = engine.update_with_input_and_text_measurer(
            &mut document,
            &stylesheet,
            DocumentInput::default(),
            &mut self.text_paint_resources,
        );
        paint_frame_with_text_resources(
            ui,
            origin,
            &output.layout,
            None,
            &mut self.text_paint_resources,
        );
    }

    fn apply_interaction_document_state(&self, document: &mut Document) {
        self.interaction_projection()
            .apply_to(document)
            .expect("interaction document contains projected loop elements");
    }

    fn interaction_projection(&self) -> DocumentProjection {
        let mut projection = DocumentProjection::new();
        projection
            .element("control-checkbox")
            .selected(self.checkbox_enabled);
        projection
            .element("control-checkbox-mark")
            .selected(self.checkbox_enabled);
        projection
            .element("control-checkbox-glyph")
            .selected(self.checkbox_enabled);
        for (index, id) in [
            "control-radio-local",
            "control-radio-remote",
            "control-radio-hybrid",
        ]
        .iter()
        .enumerate()
        {
            let selected = self.radio_choice == index;
            projection.element(*id).selected(selected);
            projection.element(format!("{id}-dot")).selected(selected);
            projection
                .element(format!("{id}-dot-fill"))
                .selected(selected);
        }
        projection
            .element("control-dropdown-trigger")
            .selected(self.dropdown_open);
        projection
            .element("control-dropdown-label")
            .text(["CSV source", "DuckDB table", "Python node"][self.dropdown_choice]);
        projection
            .element("control-dropdown-menu")
            .class("dropdown-menu-open", self.dropdown_open);
        projection
            .element("control-dropdown-chevron-down")
            .class("dropdown-chevron-visible", !self.dropdown_open);
        projection
            .element("control-dropdown-chevron-up")
            .class("dropdown-chevron-visible", self.dropdown_open);
        for (index, id) in [
            "control-dropdown-option-csv",
            "control-dropdown-option-duckdb",
            "control-dropdown-option-python",
        ]
        .iter()
        .enumerate()
        {
            let selected = self.dropdown_choice == index;
            projection.element(*id).selected(selected);
            projection.element(format!("{id}-label")).selected(selected);
        }
        projection.element("control-input-name").focused(true);
        projection.element("control-input-name-label").focused(true);
        projection.element("control-input-disabled").disabled(true);
        projection
            .element("control-input-disabled-label")
            .disabled(true);
        projection.element("loop-button-result").text(format!(
            "Button events received: {}",
            self.loop_action_count
        ));
        projection
            .element("loop-button-result-box")
            .value(format!("button-count={}", self.loop_action_count));
        projection
            .element("loop-checkbox-result")
            .text(if self.checkbox_enabled {
                "Profiling: enabled by checkbox"
            } else {
                "Profiling: disabled by checkbox"
            });
        projection
            .element("loop-checkbox-result-box")
            .selected(self.checkbox_enabled);
        projection.element("loop-radio-result").text(format!(
            "Runtime target: {}",
            ["Local runtime", "Remote worker", "Hybrid"][self.radio_choice]
        ));
        projection.element("loop-dropdown-result").text(format!(
            "Source adapter: {}",
            ["CSV source", "DuckDB table", "Python node"][self.dropdown_choice]
        ));
        projection.element("loop-summary-result").text(format!(
            "{} | {} | {} | {} click{}",
            if self.checkbox_enabled {
                "profile on"
            } else {
                "profile off"
            },
            ["local", "remote", "hybrid"][self.radio_choice],
            ["csv", "duckdb", "python"][self.dropdown_choice],
            self.loop_action_count,
            if self.loop_action_count == 1 { "" } else { "s" }
        ));
        projection
            .element("loop-summary-result-box")
            .focused(self.loop_action_count > 0);
        for (index, class) in [
            "loop-runtime-local",
            "loop-runtime-remote",
            "loop-runtime-hybrid",
        ]
        .iter()
        .enumerate()
        {
            projection
                .element("loop-radio-result-box")
                .class(*class, self.radio_choice == index);
        }

        for (index, class) in [
            "loop-source-csv",
            "loop-source-duckdb",
            "loop-source-python",
        ]
        .iter()
        .enumerate()
        {
            projection
                .element("loop-dropdown-result-box")
                .class(*class, self.dropdown_choice == index);
        }
        projection
    }

    #[cfg(test)]
    fn lab_document_output_for_test(&mut self, viewport: Size) -> DocumentOutput {
        self.reload_lab_assets_if_changed();
        let stylesheet = self.active_stylesheet();
        let mut retained = self.take_lab_document(viewport, false);
        let output = self.document_engine.update_with_input_and_text_measurer(
            &mut retained.document,
            &stylesheet,
            DocumentInput::default(),
            &mut self.text_paint_resources,
        );
        self.lab_document = Some(retained);
        output
    }

    #[cfg(test)]
    fn lab_document_output_with_stage_scroll_for_test(
        &mut self,
        viewport: Size,
        scroll_y: f32,
    ) -> DocumentOutput {
        self.reload_lab_assets_if_changed();
        let stylesheet = self.active_stylesheet();
        let mut retained = self.take_lab_document(viewport, false);
        self.document_engine.update_with_input_and_text_measurer(
            &mut retained.document,
            &stylesheet,
            DocumentInput::default(),
            &mut self.text_paint_resources,
        );
        self.document_engine
            .scroll_element_to("stage", Point::new(0.0, scroll_y));
        let output = self.document_engine.update_with_input_and_text_measurer(
            &mut retained.document,
            &stylesheet,
            DocumentInput::default(),
            &mut self.text_paint_resources,
        );
        self.lab_document = Some(retained);
        output
    }

    #[cfg(test)]
    fn lab_document_output_with_text_measurer_for_test(
        &mut self,
        viewport: Size,
        text_measurer: &mut dyn des_document::TextMeasurer,
    ) -> DocumentOutput {
        self.reload_lab_assets_if_changed();
        let stylesheet = self.active_stylesheet();
        let mut retained = self.take_lab_document(viewport, false);
        let output = self.document_engine.update_with_input_and_text_measurer(
            &mut retained.document,
            &stylesheet,
            DocumentInput::default(),
            text_measurer,
        );
        self.lab_document = Some(retained);
        output
    }

    #[cfg(test)]
    fn lab_document_output_with_input_for_test(
        &mut self,
        viewport: Size,
        input: DocumentInput,
    ) -> DocumentOutput {
        self.reload_lab_assets_if_changed();
        let stylesheet = self.active_stylesheet();
        let mut retained = self.take_lab_document(viewport, false);
        let output = self.document_engine.update_with_input_and_text_measurer(
            &mut retained.document,
            &stylesheet,
            input,
            &mut self.text_paint_resources,
        );
        self.lab_document = Some(retained);
        output
    }
}

fn sortable_config() -> SortableDocumentConfig {
    SortableDocumentConfig::new(
        "drag-item",
        "drag-cell",
        "drag-item-",
        "drag-handle-",
        "drag-cell-",
        DRAG_ITEM_COUNT,
        DROP_ZONE_COUNT,
    )
}

fn scroll_list_config() -> SortableDocumentConfig {
    SortableDocumentConfig::new(
        "drag-scroll-item",
        "drag-scroll-list",
        "drag-scroll-item-",
        "drag-scroll-handle-",
        "drag-scroll-list-",
        SCROLL_LIST_ITEM_COUNT,
        1,
    )
}

fn text_context_menu_widget(menu: &TextContextMenu) -> ContextMenu {
    let widget = ContextMenu::new(TEXT_CONTEXT_MENU_ID).at(menu.position);
    if menu
        .selected_text
        .as_ref()
        .is_some_and(|text| !text.is_empty())
    {
        widget.item(TEXT_CONTEXT_MENU_COPY_ID, "Copy")
    } else {
        widget.disabled_item(TEXT_CONTEXT_MENU_COPY_ID, "Copy")
    }
}

fn render_text_context_menu(
    ui: &mut des_document::DocumentBuilder,
    menu: Option<&TextContextMenu>,
) {
    let Some(menu) = menu else {
        return;
    };
    text_context_menu_widget(menu).render(ui);
}

fn active_grid_drag_item(id: &str) -> Option<SortableItemId> {
    sortable_config().item_for_element_id(id)
}

fn active_scroll_list_drag_item(id: &str) -> Option<SortableItemId> {
    scroll_list_config().item_for_element_id(id)
}

fn source_item_element_id(id: &str) -> Option<String> {
    active_grid_drag_item(id)
        .map(|item| format!("drag-item-{}", item.0))
        .or_else(|| {
            active_scroll_list_drag_item(id).map(|item| format!("drag-scroll-item-{}", item.0))
        })
}

#[cfg(test)]
fn drop_cell_at(output: &DocumentOutput, point: Point) -> Option<usize> {
    sortable_config()
        .drop_zone_at(output, point)
        .map(|zone| zone.0)
}

fn lab_action_for_id(id: &str) -> Option<LabAction> {
    lab_action_for_command(id)
}

fn lab_action_for_command(command: &str) -> Option<LabAction> {
    let command = command.trim();
    lab_action_registry()
        .action_for(command)
        .copied()
        .or_else(|| ShadowStyler::action_for_command(command).map(LabAction::ShadowStyler))
}

fn lab_action_registry() -> DocumentCommandRegistry<LabAction> {
    DocumentCommandRegistry::new()
        .bind("view-layout", LabAction::SelectView(LabView::Layout))
        .bind(
            "view-interaction",
            LabAction::SelectView(LabView::Interaction),
        )
        .bind("view-draggable", LabAction::SelectView(LabView::Draggable))
        .bind("view-styling", LabAction::SelectView(LabView::Styling))
        .bind("view-animation", LabAction::SelectView(LabView::Animation))
        .bind("view-scrolling", LabAction::SelectView(LabView::Scrolling))
        .bind("view-floating", LabAction::SelectView(LabView::Floating))
        .bind("view-table", LabAction::SelectView(LabView::Table))
        .bind("view-text", LabAction::SelectView(LabView::Text))
        .bind("view-nesting", LabAction::SelectView(LabView::Nesting))
        .bind("view-graph", LabAction::SelectView(LabView::Graph))
        .bind("toggle-optional-card", LabAction::ToggleOptionalCard)
        .bind("toggle-density", LabAction::ToggleDensity)
        .bind("control-checkbox", LabAction::ToggleCheckbox)
        .bind("control-radio-local", LabAction::SelectRadio(0))
        .bind("control-radio-remote", LabAction::SelectRadio(1))
        .bind("control-radio-hybrid", LabAction::SelectRadio(2))
        .bind("control-dropdown", LabAction::ToggleDropdown)
        .bind("control-dropdown-option-csv", LabAction::SelectDropdown(0))
        .bind(
            "control-dropdown-option-duckdb",
            LabAction::SelectDropdown(1),
        )
        .bind(
            "control-dropdown-option-python",
            LabAction::SelectDropdown(2),
        )
        .bind("loop-action-button", LabAction::IncrementLoopAction)
        .bind(TEXT_CONTEXT_MENU_COPY_ID, LabAction::CopyTextSelection)
}

fn is_dropdown_hit(hit_id: &Option<ElementId>) -> bool {
    hit_id.as_ref().is_some_and(|id| {
        id.as_str() == "control-dropdown"
            || id.as_str() == "control-dropdown-trigger"
            || id.as_str() == "control-dropdown-label"
            || id.as_str().starts_with("control-dropdown-chevron")
            || id.as_str() == "control-dropdown-menu"
            || id.as_str().starts_with("control-dropdown-option-")
    })
}

fn is_text_context_menu_hit(hit_id: &Option<ElementId>) -> bool {
    hit_id.as_ref().is_some_and(|id| {
        id.as_str() == TEXT_CONTEXT_MENU_ID || id.as_str().starts_with("text-context-menu-")
    })
}

fn repaint_input_after_action(input: DocumentInput) -> DocumentInput {
    DocumentInput {
        pointer: input.pointer.map(|mut pointer| {
            pointer.primary_pressed = false;
            pointer.primary_clicked = false;
            pointer.primary_click_count = 0;
            pointer.secondary_clicked = false;
            pointer
        }),
        scroll_delta: Point::ZERO,
        keys: Vec::new(),
    }
}

fn apply_cursor_icon(ui: &egui::Ui, output: &DocumentOutput) {
    if let Some(cursor) = cursor_icon_for_output(output) {
        ui.ctx().set_cursor_icon(cursor);
    }
}

fn cursor_icon_for_output(output: &DocumentOutput) -> Option<egui::CursorIcon> {
    if output.active_drag.is_some() {
        return Some(egui::CursorIcon::PointingHand);
    }
    if output
        .hit_element()
        .is_some_and(|frame| frame.has_class("drag-handle"))
    {
        return Some(egui::CursorIcon::PointingHand);
    }
    None
}

#[derive(Clone, Copy)]
enum LabAction {
    SelectView(LabView),
    ToggleOptionalCard,
    ToggleDensity,
    ToggleCheckbox,
    SelectRadio(usize),
    ToggleDropdown,
    SelectDropdown(usize),
    IncrementLoopAction,
    ShadowStyler(ShadowStylerAction),
    CopyTextSelection,
}

fn paint_legacy_text_path_comparison(ui: &egui::Ui, origin: egui::Pos2, output: &DocumentOutput) {
    let Some(pane) = output.layout.find("text-legacy-100-pane") else {
        return;
    };

    let content_rect = pane.rect.inset(pane.style.padding);
    let clip_rect = document_rect_to_egui(
        origin,
        Rect::new(
            content_rect.origin.x,
            content_rect.origin.y + 35.0,
            content_rect.size.width,
            (content_rect.size.height - 35.0).max(0.0),
        ),
    );
    let color = egui_color(TEXT);
    let galley = ui.painter().layout_job(legacy_simple_text_job(
        "Ag 100px".to_owned(),
        100.0,
        color,
        f32::INFINITY,
    ));
    ui.painter()
        .with_clip_rect(clip_rect)
        .galley(clip_rect.min, galley, color);
}

fn legacy_simple_text_job(
    text: String,
    font_size: f32,
    color: egui::Color32,
    wrap_width: f32,
) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::simple(
        text,
        egui::FontId::proportional(font_size),
        color,
        wrap_width,
    );
    job.wrap.max_width = wrap_width;
    job
}

fn document_rect_to_egui(origin: egui::Pos2, rect: Rect) -> egui::Rect {
    egui::Rect::from_min_size(
        egui::pos2(origin.x + rect.origin.x, origin.y + rect.origin.y),
        egui::vec2(rect.size.width, rect.size.height),
    )
}

fn egui_color(color: Color) -> egui::Color32 {
    egui::Color32::from_rgba_premultiplied(color.r, color.g, color.b, color.a)
}

fn inert_pointer_move(input: &DocumentInput) -> bool {
    input.scroll_delta == Point::ZERO
        && input.keys.is_empty()
        && input.pointer.is_some_and(|pointer| {
            !pointer.primary_down
                && !pointer.primary_pressed
                && !pointer.primary_clicked
                && pointer.primary_click_count == 0
                && !pointer.secondary_clicked
        })
}

fn deepest_frame_at(
    frame: &des_document::ResolvedElement,
    point: Point,
) -> Option<&des_document::ResolvedElement> {
    if !frame.rect.contains(point) {
        return None;
    }
    frame
        .children
        .iter()
        .rev()
        .find_map(|child| deepest_frame_at(child, point))
        .or(Some(frame))
}

#[derive(Clone, Copy, Debug, Default)]
struct UiLabPerf {
    stylesheet_time: Duration,
    document_time: Duration,
    engine_time: Duration,
    paint_time: Duration,
    text_paint: TextPaintStats,
    metrics: DocumentMetrics,
}
