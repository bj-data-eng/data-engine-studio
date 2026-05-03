mod egui_adapter;
mod styles;
#[cfg(test)]
mod tests;
mod views;

use egui_adapter::{
    EguiTextMeasurer, configure_text_selection_input, copy_selected_text_on_command,
    document_input, paint_frame, paint_scroll_chrome,
};
use styles::stylesheet;
use views::{render_drag_overlay_layer, render_nav, render_stage, render_topbar};

use des_ui_document::{
    Color, Document, DocumentDrag, DocumentEngine, DocumentEventKind, DocumentMetrics,
    DocumentOutput, DocumentUpdate, ElementId, ElementRole, ElementSpec, Length, Point,
    PointerInput, Size, Style, StyleSelector, StyleSheet, TableCellSpec, TableColumnSpec,
    TableSpec, TableTrackSize,
};
use des_ui_widgets::{
    AutoScrollOptions, AutoScroller, DropZoneId, SortableDocumentConfig, SortableDropPreview,
    SortableItemId, SortableModel,
};
use eframe::egui;
use std::time::{Duration, Instant};

const BACKGROUND: Color = Color::rgb(17, 20, 23);
const PANEL: Color = Color::rgb(27, 31, 35);
const PANEL_ALT: Color = Color::rgb(22, 26, 30);
const CARD: Color = Color::rgb(31, 37, 42);
const CARD_HOVER: Color = Color::rgb(38, 47, 54);
const CARD_SELECTED: Color = Color::rgb(35, 56, 78);
const CARD_PRESSED: Color = Color::rgb(45, 72, 98);
const STROKE: Color = Color::rgb(61, 68, 76);
const STROKE_SELECTED: Color = Color::rgb(88, 157, 230);
const TEXT: Color = Color::rgb(228, 234, 240);
const TEXT_MUTED: Color = Color::rgb(156, 166, 176);
const TEXT_ACCENT: Color = Color::rgb(113, 196, 255);
const GREEN: Color = Color::rgb(95, 204, 140);
const PURPLE: Color = Color::rgb(151, 93, 219);
const ANIMATION_FRAME_TIME: Duration = Duration::from_millis(16);
const DRAG_ITEM_COUNT: usize = 3;
const DROP_ZONE_COUNT: usize = 6;
const SCROLL_LIST_ITEM_COUNT: usize = 14;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LabView {
    Layout,
    Interaction,
    Styling,
    Animation,
    Scrolling,
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
            "styling" | "view-styling" => Some(Self::Styling),
            "animation" | "view-animation" => Some(Self::Animation),
            "scrolling" | "view-scrolling" => Some(Self::Scrolling),
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
            Self::Styling => "view-styling",
            Self::Animation => "view-animation",
            Self::Scrolling => "view-scrolling",
            Self::Table => "view-table",
            Self::Text => "view-text",
            Self::Nesting => "view-nesting",
            Self::Graph => "view-graph",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Layout => "Layout",
            Self::Interaction => "Interaction",
            Self::Styling => "Styling",
            Self::Animation => "Animation",
            Self::Scrolling => "Scrolling",
            Self::Table => "Table",
            Self::Text => "Text",
            Self::Nesting => "Nesting",
            Self::Graph => "Graph",
        }
    }
}

pub(crate) struct UiLabState {
    document_engine: DocumentEngine,
    stylesheet: StyleSheet,
    view: LabView,
    show_optional_card: bool,
    dense_mode: bool,
    checkbox_enabled: bool,
    radio_choice: usize,
    dropdown_open: bool,
    dropdown_choice: usize,
    loop_action_count: usize,
    drag_item_cells: [usize; 3],
    drag_item_order: [usize; 3],
    scroll_list_item_order: [usize; SCROLL_LIST_ITEM_COUNT],
    active_drag: Option<DocumentDrag>,
    drag_parent_offset: Option<Point>,
    drag_source_size: Option<Size>,
    drag_drop_preview: Option<SortableDropPreview>,
    scroll_list_drop_preview: Option<SortableDropPreview>,
    text_context_menu: Option<TextContextMenu>,
    last_perf: UiLabPerf,
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
            view: LabView::Layout,
            show_optional_card: true,
            dense_mode: false,
            checkbox_enabled: true,
            radio_choice: 0,
            dropdown_open: false,
            dropdown_choice: 1,
            loop_action_count: 0,
            drag_item_cells: [0, 2, 4],
            drag_item_order: [0, 1, 2],
            scroll_list_item_order: core::array::from_fn(|index| index),
            active_drag: None,
            drag_parent_offset: None,
            drag_source_size: None,
            drag_drop_preview: None,
            scroll_list_drop_preview: None,
            text_context_menu: None,
            last_perf: UiLabPerf::default(),
        }
    }
}

impl UiLabState {
    pub(crate) fn new(initial_view: Option<&str>) -> Self {
        let mut state = Self::default();
        if let Some(view) = initial_view.and_then(LabView::from_id) {
            state.view = view;
        }
        state
    }

    pub(crate) fn render(&mut self, ui: &mut egui::Ui, debug_overlay: bool) {
        configure_text_selection_input(ui.ctx());
        let origin = ui.max_rect().min;
        let viewport = ui.max_rect().size();
        let document_start = Instant::now();
        let document = self.document(Size::new(viewport.x, viewport.y), debug_overlay);
        let stylesheet = self.active_stylesheet();
        let document_time = document_start.elapsed();
        let input = document_input(ui, origin);
        let pointer = input.pointer;
        let engine_start = Instant::now();
        let mut text_measurer = EguiTextMeasurer::new(ui.ctx());
        let output = self.document_engine.update_with_input_and_text_measurer(
            &document,
            &stylesheet,
            input,
            &mut text_measurer,
        );
        let engine_time = engine_start.elapsed();
        copy_selected_text_on_command(ui, &output);

        let paint_start = Instant::now();
        paint_frame(ui, origin, &output.layout, output.text_selection.as_ref());
        paint_scroll_chrome(ui, origin, &output.scroll_chrome);
        let paint_time = paint_start.elapsed();
        self.last_perf = UiLabPerf {
            document_time,
            engine_time,
            paint_time,
            metrics: output.metrics,
        };
        self.apply_document_events(ui, &output, pointer);
        self.paint_text_context_menu(ui, origin, &output);
        if debug_overlay {
            self.paint_debug_overlay(ui);
        }
        if output.animating {
            ui.ctx().request_repaint_after(ANIMATION_FRAME_TIME);
        }
    }

    fn apply_document_events(
        &mut self,
        ui: &egui::Ui,
        output: &DocumentOutput,
        pointer: Option<PointerInput>,
    ) {
        let was_dropdown_open = self.dropdown_open;
        let primary_clicked = pointer
            .map(|pointer| pointer.primary_clicked)
            .unwrap_or(false);
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
            self.drag_parent_offset = Some(Point::new(
                drag.origin.x - rect.origin.x,
                drag.origin.y - rect.origin.y,
            ));
            self.drag_source_size = Some(rect.size);
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
        if let Some(drag) = &output.completed_drag {
            self.finish_drag(output, drag);
            ui.ctx().request_repaint();
        }
        for event in &output.events {
            match event.kind {
                DocumentEventKind::Clicked => {
                    if let Some(action) = lab_action_for_id(event.target.as_str()) {
                        self.apply_lab_action(action);
                        ui.ctx().request_repaint();
                    }
                }
                DocumentEventKind::Pressed => {
                    if source_item_element_id(event.target.as_str()).is_some() {
                        ui.ctx().request_repaint();
                    }
                }
                DocumentEventKind::ContextRequested => {
                    if let Some(pointer) = pointer {
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
    }

    fn paint_text_context_menu(
        &mut self,
        ui: &mut egui::Ui,
        origin: egui::Pos2,
        output: &DocumentOutput,
    ) {
        let Some(menu) = self.text_context_menu.clone() else {
            return;
        };
        if output
            .snapshot()
            .find(menu.target.as_str())
            .is_none_or(|frame| !frame.selectable_text())
        {
            self.text_context_menu = None;
            return;
        }

        let selected_text = menu.selected_text.clone();
        let menu_pos = egui::pos2(origin.x + menu.position.x, origin.y + menu.position.y);
        let area_response = egui::Area::new(egui::Id::new("text-selection-context-menu"))
            .order(egui::Order::Foreground)
            .fixed_pos(menu_pos)
            .show(ui.ctx(), |ui| {
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    ui.set_min_width(140.0);
                    let copy_enabled = selected_text.as_ref().is_some_and(|text| !text.is_empty());
                    if ui
                        .add_enabled(copy_enabled, egui::Button::new("Copy"))
                        .clicked()
                    {
                        if let Some(text) = selected_text.clone() {
                            ui.ctx().copy_text(text);
                        }
                        self.text_context_menu = None;
                    }
                });
            });
        let menu_rect = area_response.response.rect;
        let clicked_away = ui.input(|input| {
            input.pointer.primary_clicked()
                && input
                    .pointer
                    .interact_pos()
                    .is_some_and(|position| !menu_rect.contains(position))
        });
        if clicked_away {
            self.text_context_menu = None;
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

    fn apply_lab_action(&mut self, action: LabAction) {
        match action {
            LabAction::SelectView(view) => self.view = view,
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
        }
    }

    fn document(&self, viewport: Size, debug_overlay: bool) -> Document {
        let mut document = Document::build(viewport, |ui| {
            ui.element(
                "lab-root",
                ElementSpec::new(ElementRole::Panel).class("lab-root"),
                |ui| {
                    render_topbar(ui, debug_overlay);
                    ui.element(
                        "lab-body",
                        ElementSpec::new(ElementRole::Panel).class("lab-body"),
                        |ui| {
                            render_nav(ui, self.view);
                            render_stage(
                                ui,
                                self.view,
                                self.show_optional_card,
                                self.dense_mode,
                                self.checkbox_enabled,
                                self.radio_choice,
                                self.dropdown_open,
                                self.dropdown_choice,
                                self.drag_item_cells,
                                self.drag_item_order,
                                self.scroll_list_item_order,
                                self.active_drag_item(),
                                self.active_scroll_list_drag_item(),
                                self.active_drag.as_ref().map(|drag| drag.current),
                                self.drag_drop_preview,
                                self.scroll_list_drop_preview,
                            );
                        },
                    );
                    render_drag_overlay_layer(
                        ui,
                        self.active_drag_item(),
                        self.active_scroll_list_drag_item(),
                        self.active_drag.as_ref().map(|drag| drag.current),
                    );
                },
            );
        });
        if self.view == LabView::Interaction {
            document.apply_update(&self.interaction_document_update());
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
        stylesheet
    }

    fn interaction_document_update(&self) -> DocumentUpdate {
        let mut update = DocumentUpdate::new()
            .set_text(
                "loop-button-result",
                format!("Button events received: {}", self.loop_action_count),
            )
            .set_value(
                "loop-button-result-box",
                format!("button-count={}", self.loop_action_count),
            )
            .set_text(
                "loop-checkbox-result",
                if self.checkbox_enabled {
                    "Profiling: enabled by checkbox"
                } else {
                    "Profiling: disabled by checkbox"
                },
            )
            .set_selected("loop-checkbox-result-box", self.checkbox_enabled)
            .set_text(
                "loop-radio-result",
                format!(
                    "Runtime target: {}",
                    ["Local runtime", "Remote worker", "Hybrid"][self.radio_choice]
                ),
            )
            .set_text(
                "loop-dropdown-result",
                format!(
                    "Source adapter: {}",
                    ["CSV source", "DuckDB table", "Python node"][self.dropdown_choice]
                ),
            )
            .set_text(
                "loop-summary-result",
                format!(
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
                ),
            )
            .set_focused("loop-summary-result-box", self.loop_action_count > 0);

        for (index, class) in [
            "loop-runtime-local",
            "loop-runtime-remote",
            "loop-runtime-hybrid",
        ]
        .iter()
        .enumerate()
        {
            if self.radio_choice == index {
                update = update.add_class("loop-radio-result-box", *class);
            } else {
                update = update.remove_class("loop-radio-result-box", *class);
            }
        }

        for (index, class) in [
            "loop-source-csv",
            "loop-source-duckdb",
            "loop-source-python",
        ]
        .iter()
        .enumerate()
        {
            if self.dropdown_choice == index {
                update = update.add_class("loop-dropdown-result-box", *class);
            } else {
                update = update.remove_class("loop-dropdown-result-box", *class);
            }
        }

        update
    }

    fn paint_debug_overlay(&self, ui: &egui::Ui) {
        egui::Area::new("ui-lab-debug-overlay".into())
            .order(egui::Order::Foreground)
            .fixed_pos(ui.max_rect().right_top() + egui::vec2(-274.0, 12.0))
            .show(ui.ctx(), |ui| {
                egui::Frame::new()
                    .fill(egui::Color32::from_rgba_unmultiplied(13, 16, 19, 230))
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(61, 68, 76)))
                    .corner_radius(egui::CornerRadius::same(6))
                    .inner_margin(egui::Margin::symmetric(10, 8))
                    .show(ui, |ui| {
                        ui.set_width(250.0);
                        ui.label(
                            egui::RichText::new("UI Lab Runtime")
                                .color(egui::Color32::from_rgb(228, 234, 240))
                                .strong(),
                        );
                        ui.separator();
                        debug_row(ui, "document", self.last_perf.document_time);
                        debug_row(ui, "engine", self.last_perf.engine_time);
                        debug_row(ui, "paint", self.last_perf.paint_time);
                        ui.separator();
                        ui.label(format!(
                            "elements: {}",
                            self.last_perf.metrics.element_count
                        ));
                        ui.label(format!(
                            "scrollbars: {}",
                            self.last_perf.metrics.scroll_chrome_count
                        ));
                        ui.label(format!(
                            "input cache hit: {}",
                            self.last_perf.metrics.reused_cached_layout
                        ));
                        ui.label(format!(
                            "final relayout skipped: {}",
                            self.last_perf.metrics.reused_input_layout
                        ));
                        ui.label(format!(
                            "input changed: {}",
                            self.last_perf.metrics.input_changed_state
                        ));
                        ui.label(format!(
                            "style changed: {}",
                            self.last_perf.metrics.animation_changed_style
                        ));
                        ui.label(format!(
                            "layout changed: {}",
                            self.last_perf.metrics.animation_changed_layout
                        ));
                        ui.label(format!(
                            "paint changed: {}",
                            self.last_perf.metrics.animation_changed_paint
                        ));
                    });
            });
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
    match id {
        "view-layout" => Some(LabAction::SelectView(LabView::Layout)),
        "view-interaction" => Some(LabAction::SelectView(LabView::Interaction)),
        "view-styling" => Some(LabAction::SelectView(LabView::Styling)),
        "view-animation" => Some(LabAction::SelectView(LabView::Animation)),
        "view-scrolling" => Some(LabAction::SelectView(LabView::Scrolling)),
        "view-table" => Some(LabAction::SelectView(LabView::Table)),
        "view-text" => Some(LabAction::SelectView(LabView::Text)),
        "view-nesting" => Some(LabAction::SelectView(LabView::Nesting)),
        "view-graph" => Some(LabAction::SelectView(LabView::Graph)),
        "toggle-optional-card" => Some(LabAction::ToggleOptionalCard),
        "toggle-density" => Some(LabAction::ToggleDensity),
        "control-checkbox" => Some(LabAction::ToggleCheckbox),
        "control-radio-local" => Some(LabAction::SelectRadio(0)),
        "control-radio-remote" => Some(LabAction::SelectRadio(1)),
        "control-radio-hybrid" => Some(LabAction::SelectRadio(2)),
        "control-dropdown" => Some(LabAction::ToggleDropdown),
        "control-dropdown-option-csv" => Some(LabAction::SelectDropdown(0)),
        "control-dropdown-option-duckdb" => Some(LabAction::SelectDropdown(1)),
        "control-dropdown-option-python" => Some(LabAction::SelectDropdown(2)),
        "loop-action-button" => Some(LabAction::IncrementLoopAction),
        _ => None,
    }
}

fn is_dropdown_hit(hit_id: &Option<ElementId>) -> bool {
    hit_id.as_ref().is_some_and(|id| {
        id.as_str() == "control-dropdown"
            || id.as_str() == "control-dropdown-label"
            || id.as_str() == "control-dropdown-chevron"
            || id.as_str() == "control-dropdown-menu"
            || id.as_str().starts_with("control-dropdown-option-")
    })
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
}

#[derive(Clone, Copy, Debug, Default)]
struct UiLabPerf {
    document_time: Duration,
    engine_time: Duration,
    paint_time: Duration,
    metrics: DocumentMetrics,
}

fn debug_row(ui: &mut egui::Ui, label: &str, duration: Duration) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(format!("{:.2} ms", duration.as_secs_f64() * 1000.0));
        });
    });
}
