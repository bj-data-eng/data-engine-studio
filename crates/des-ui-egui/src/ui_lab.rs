mod egui_adapter;
mod styles;
#[cfg(test)]
mod tests;
mod views;

use egui_adapter::{document_input, paint_frame, paint_scroll_chrome};
use styles::stylesheet;
use views::{render_nav, render_stage, render_topbar};

use des_ui_document::{
    Color, Document, DocumentEngine, DocumentMetrics, DocumentOutput, ElementRole, ElementSpec,
    Size, StyleSheet,
};
use eframe::egui;
use std::collections::BTreeMap;
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
    last_click_counts: BTreeMap<&'static str, u32>,
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
            last_click_counts: BTreeMap::new(),
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
        let document_time = document_start.elapsed();
        let engine_start = Instant::now();
        let output = self.document_engine.update_with_input(
            &document,
            &self.stylesheet,
            document_input(ui, origin),
        );
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
        self.apply_clicks(ui, &output);
        if debug_overlay {
            self.paint_debug_overlay(ui);
        }
        if output.animating {
            ui.ctx().request_repaint_after(ANIMATION_FRAME_TIME);
        }
    }

    fn apply_clicks(&mut self, ui: &egui::Ui, _output: &DocumentOutput) {
        for (id, action) in [
            ("view-layout", LabAction::SelectView(LabView::Layout)),
            (
                "view-interaction",
                LabAction::SelectView(LabView::Interaction),
            ),
            ("view-styling", LabAction::SelectView(LabView::Styling)),
            ("view-animation", LabAction::SelectView(LabView::Animation)),
            ("view-scrolling", LabAction::SelectView(LabView::Scrolling)),
            ("view-nesting", LabAction::SelectView(LabView::Nesting)),
            ("view-graph", LabAction::SelectView(LabView::Graph)),
            ("toggle-optional-card", LabAction::ToggleOptionalCard),
            ("toggle-density", LabAction::ToggleDensity),
        ] {
            let count = self
                .document_engine
                .element_state(id)
                .map(|state| state.click_count)
                .unwrap_or_default();
            let previous = self.last_click_counts.insert(id, count).unwrap_or_default();
            if count > previous {
                match action {
                    LabAction::SelectView(view) => self.view = view,
                    LabAction::ToggleOptionalCard => {
                        self.show_optional_card = !self.show_optional_card
                    }
                    LabAction::ToggleDensity => self.dense_mode = !self.dense_mode,
                }
                ui.ctx().request_repaint();
            }
        }
    }

    fn document(&self, viewport: Size, debug_overlay: bool) -> Document {
        Document::build(viewport, |ui| {
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
                            render_stage(ui, self.view, self.show_optional_card, self.dense_mode);
                        },
                    );
                },
            );
        })
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
                            "cached layout: {}",
                            self.last_perf.metrics.reused_cached_layout
                        ));
                        ui.label(format!(
                            "reused layout: {}",
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

#[derive(Clone, Copy)]
enum LabAction {
    SelectView(LabView),
    ToggleOptionalCard,
    ToggleDensity,
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
