mod egui_adapter;
mod styles;
#[cfg(test)]
mod tests;
mod views;

use egui_adapter::{document_input, paint_frame, paint_scroll_chrome};
use styles::stylesheet;
use views::{render_nav, render_stage, render_topbar};

use des_ui_document::{
    Color, Document, DocumentDrag, DocumentEngine, DocumentEventKind, DocumentMetrics,
    DocumentOutput, DocumentUpdate, ElementId, ElementRole, ElementSpec, Length, Point,
    PointerInput, Size, Style, StyleSelector, StyleSheet,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LabView {
    Layout,
    Interaction,
    Styling,
    Animation,
    Scrolling,
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
    active_drag: Option<DocumentDrag>,
    drag_parent_offset: Option<Point>,
    drag_drop_preview: Option<DragDropPreview>,
    last_perf: UiLabPerf,
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
            active_drag: None,
            drag_parent_offset: None,
            drag_drop_preview: None,
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
        let origin = ui.max_rect().min;
        let viewport = ui.max_rect().size();
        let document_start = Instant::now();
        let document = self.document(Size::new(viewport.x, viewport.y), debug_overlay);
        let stylesheet = self.active_stylesheet();
        let document_time = document_start.elapsed();
        let input = document_input(ui, origin);
        let pointer = input.pointer;
        let engine_start = Instant::now();
        let output = self
            .document_engine
            .update_with_input(&document, &stylesheet, input);
        let engine_time = engine_start.elapsed();

        let paint_start = Instant::now();
        paint_frame(ui, origin, &output.layout);
        paint_scroll_chrome(ui, origin, &output.scroll_chrome);
        let paint_time = paint_start.elapsed();
        self.last_perf = UiLabPerf {
            document_time,
            engine_time,
            paint_time,
            metrics: output.metrics,
        };
        self.apply_document_events(ui, &output, pointer);
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
            && let Some(item) = drag_item_for_id(drag.target.as_str())
            && let Some(rect) = output
                .snapshot()
                .find(format!("drag-item-{item}").as_str())
                .map(|element| element.rect())
        {
            self.drag_parent_offset = Some(Point::new(
                drag.origin.x - rect.origin.x,
                drag.origin.y - rect.origin.y,
            ));
        }
        self.drag_drop_preview = self.active_drag.as_ref().and_then(|drag| {
            let active_item = self.active_drag_item()?;
            let preview = drag_drop_preview_at(output, drag.current, Some(active_item))?;
            self.drag_preview_changes_position(active_item, preview)
                .then_some(preview)
        });
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
                    if drag_item_for_id(event.target.as_str()).is_some() {
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

    fn finish_drag(&mut self, output: &DocumentOutput, drag: &DocumentDrag) {
        if let Some(item) = drag_item_for_id(drag.target.as_str())
            && let Some(preview) = drag_drop_preview_at(output, drag.current, Some(item))
            && self.drag_preview_changes_position(item, preview)
        {
            self.drag_item_cells[item] = preview.cell;
            self.apply_drag_order(item, preview);
        }
        self.active_drag = None;
        self.drag_parent_offset = None;
        self.drag_drop_preview = None;
    }

    fn apply_drag_order(&mut self, item: usize, preview: DragDropPreview) {
        let mut ordered_items: Vec<_> = (0..self.drag_item_cells.len()).collect();
        ordered_items.sort_by_key(|candidate| self.drag_item_order[*candidate]);
        ordered_items.retain(|candidate| *candidate != item);

        let insert_index = preview
            .nearest_item
            .and_then(|nearest| {
                ordered_items
                    .iter()
                    .position(|candidate| *candidate == nearest)
                    .map(|index| {
                        if preview.edge == DropEdge::After {
                            index + 1
                        } else {
                            index
                        }
                    })
            })
            .unwrap_or(ordered_items.len());
        ordered_items.insert(insert_index.min(ordered_items.len()), item);

        for (order, ordered_item) in ordered_items.into_iter().enumerate() {
            self.drag_item_order[ordered_item] = order;
        }
    }

    fn drag_preview_changes_position(&self, item: usize, preview: DragDropPreview) -> bool {
        if self.drag_item_cells[item] != preview.cell {
            return true;
        }

        let mut cell_items: Vec<_> = (0..self.drag_item_cells.len())
            .filter(|candidate| self.drag_item_cells[*candidate] == preview.cell)
            .collect();
        cell_items.sort_by_key(|candidate| self.drag_item_order[*candidate]);

        let Some(current_index) = cell_items.iter().position(|candidate| *candidate == item) else {
            return true;
        };

        let mut target_items = cell_items;
        target_items.retain(|candidate| *candidate != item);
        let target_index = preview
            .nearest_item
            .and_then(|nearest| {
                target_items
                    .iter()
                    .position(|candidate| *candidate == nearest)
                    .map(|index| {
                        if preview.edge == DropEdge::After {
                            index + 1
                        } else {
                            index
                        }
                    })
            })
            .unwrap_or(target_items.len())
            .min(target_items.len());

        target_index != current_index
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
                                self.active_drag_item(),
                                self.active_drag.as_ref().map(|drag| drag.current),
                                self.drag_drop_preview,
                            );
                        },
                    );
                },
            );
        });
        if self.view == LabView::Interaction {
            document.apply_update(&self.interaction_document_update());
        }
        document
    }

    fn active_drag_item(&self) -> Option<usize> {
        self.active_drag
            .as_ref()
            .and_then(|drag| drag_item_for_id(drag.target.as_str()))
    }

    fn active_stylesheet(&self) -> StyleSheet {
        let mut stylesheet = self.stylesheet.clone();
        if let Some(drag) = &self.active_drag {
            let offset = self.drag_parent_offset.unwrap_or(drag.pointer_offset);
            stylesheet.push_rule(
                StyleSelector::id("drag-overlay"),
                Style::default()
                    .absolute_viewport()
                    .left(Length::Px(drag.current.x - offset.x))
                    .top(Length::Px(drag.current.y - offset.y))
                    .z_index(100),
            );
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

fn drag_item_for_id(id: &str) -> Option<usize> {
    id.strip_prefix("drag-item-")
        .or_else(|| id.strip_prefix("drag-handle-"))
        .and_then(|suffix| suffix.parse::<usize>().ok())
        .filter(|index| *index < 3)
}

fn drag_drop_preview_at(
    output: &DocumentOutput,
    point: Point,
    active_item: Option<usize>,
) -> Option<DragDropPreview> {
    let cell = drop_cell_at(output, point)?;
    let nearest = output
        .snapshot()
        .elements_with_class("drag-item")
        .into_iter()
        .filter_map(|element| {
            let item = drag_item_for_id(element.id().as_str())?;
            if active_item == Some(item) {
                return None;
            }
            if drag_item_cell_for_id(output, item) != Some(cell) {
                return None;
            }
            let rect = element.rect();
            let center_y = rect.origin.y + rect.size.height / 2.0;
            Some((item, center_y, (point.y - center_y).abs()))
        })
        .min_by(|left, right| left.2.total_cmp(&right.2));

    let (nearest_item, edge) = nearest
        .map(|(item, center_y, _)| {
            (
                Some(item),
                if point.y > center_y {
                    DropEdge::After
                } else {
                    DropEdge::Before
                },
            )
        })
        .unwrap_or((None, DropEdge::After));

    Some(DragDropPreview {
        cell,
        nearest_item,
        edge,
    })
}

fn drag_item_cell_for_id(output: &DocumentOutput, item: usize) -> Option<usize> {
    let point = output
        .snapshot()
        .find(format!("drag-item-{item}").as_str())?
        .rect()
        .origin;
    output
        .snapshot()
        .elements_with_class("drag-cell")
        .into_iter()
        .filter(|element| element.rect().contains(point))
        .find_map(|element| drop_cell_for_id(element.id().as_str()))
}

fn drop_cell_at(output: &DocumentOutput, point: Point) -> Option<usize> {
    output
        .snapshot()
        .elements_with_class("drag-cell")
        .into_iter()
        .filter(|element| element.rect().contains(point))
        .find_map(|element| drop_cell_for_id(element.id().as_str()))
}

fn drop_cell_for_id(id: &str) -> Option<usize> {
    id.strip_prefix("drag-cell-")
        .and_then(|suffix| suffix.parse::<usize>().ok())
        .filter(|index| *index < 6)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct DragDropPreview {
    cell: usize,
    nearest_item: Option<usize>,
    edge: DropEdge,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DropEdge {
    Before,
    After,
}

fn lab_action_for_id(id: &str) -> Option<LabAction> {
    match id {
        "view-layout" => Some(LabAction::SelectView(LabView::Layout)),
        "view-interaction" => Some(LabAction::SelectView(LabView::Interaction)),
        "view-styling" => Some(LabAction::SelectView(LabView::Styling)),
        "view-animation" => Some(LabAction::SelectView(LabView::Animation)),
        "view-scrolling" => Some(LabAction::SelectView(LabView::Scrolling)),
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
