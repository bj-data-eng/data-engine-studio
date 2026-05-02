mod runtime_adapter;
mod styles;
mod views;

use runtime_adapter::{paint_frame, paint_scroll_chrome, runtime_input};
use styles::stylesheet;
use views::{render_nav, render_stage, render_topbar};

use des_ui_runtime::{
    Color, Direction, ElementRole, ElementSpec, ElementStateSelector, Insets, LayoutFrame, Length,
    Overflow, Point, PointerInput, Runtime, RuntimeInput, RuntimeOutput, Scene, Size, StylePatch,
    StyleSelector, StyleSheet, Transition,
};
use eframe::egui;
use std::collections::BTreeMap;

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LabView {
    Layout,
    Interaction,
    Styling,
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
            Self::Scrolling => "Scrolling",
            Self::Nesting => "Nesting",
            Self::Graph => "Graph",
        }
    }
}

pub(crate) struct UiLabState {
    runtime: Runtime,
    view: LabView,
    show_optional_card: bool,
    dense_mode: bool,
    last_click_counts: BTreeMap<&'static str, u32>,
}

impl Default for UiLabState {
    fn default() -> Self {
        Self {
            runtime: Runtime::default(),
            view: LabView::Layout,
            show_optional_card: true,
            dense_mode: false,
            last_click_counts: BTreeMap::new(),
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
        let stylesheet = stylesheet();
        let scene = self.scene(Size::new(viewport.x, viewport.y), debug_overlay);
        let output = self
            .runtime
            .update_with_input(&scene, &stylesheet, runtime_input(ui, origin));

        paint_frame(ui, origin, &output.layout);
        paint_scroll_chrome(ui, origin, &output.scroll_chrome);
        self.apply_clicks(ui, &output);
        if output.animating {
            ui.ctx().request_repaint();
        }
    }

    fn apply_clicks(&mut self, ui: &egui::Ui, _output: &RuntimeOutput) {
        for (id, action) in [
            ("view-layout", LabAction::SelectView(LabView::Layout)),
            (
                "view-interaction",
                LabAction::SelectView(LabView::Interaction),
            ),
            ("view-styling", LabAction::SelectView(LabView::Styling)),
            ("view-scrolling", LabAction::SelectView(LabView::Scrolling)),
            ("view-nesting", LabAction::SelectView(LabView::Nesting)),
            ("view-graph", LabAction::SelectView(LabView::Graph)),
            ("toggle-optional-card", LabAction::ToggleOptionalCard),
            ("toggle-density", LabAction::ToggleDensity),
        ] {
            let count = self
                .runtime
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

    fn scene(&self, viewport: Size, debug_overlay: bool) -> Scene {
        Scene::build(viewport, |ui| {
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
}

#[derive(Clone, Copy)]
enum LabAction {
    SelectView(LabView),
    ToggleOptionalCard,
    ToggleDensity,
}

#[cfg(test)]
mod graphical_tests {
    use super::*;
    use crate::test_graphics::{
        TEST_HEIGHT, TEST_WIDTH, assert_exact_image_match, compare_images, image_stats,
        render_harness, test_harness,
    };
    use egui_kittest::Harness;

    fn lab_harness(initial_view: &str) -> Harness<'_, UiLabState> {
        test_harness(UiLabState::new(Some(initial_view)), |ui, state| {
            state.render(ui, false);
        })
    }

    fn lab_image(initial_view: &str) -> image::RgbaImage {
        render_harness(&mut lab_harness(initial_view))
    }

    fn lab_rect(id: &str) -> des_ui_runtime::Rect {
        let mut runtime = Runtime::default();
        let scene =
            UiLabState::new(Some("layout")).scene(Size::new(TEST_WIDTH, TEST_HEIGHT), false);
        let output = runtime.update(&scene, &stylesheet());
        find_frame(&output.layout, id)
            .unwrap_or_else(|| panic!("expected layout frame for {id}"))
            .rect
    }

    fn lab_output(initial_view: &str) -> RuntimeOutput {
        let mut runtime = Runtime::default();
        let scene =
            UiLabState::new(Some(initial_view)).scene(Size::new(TEST_WIDTH, TEST_HEIGHT), false);
        runtime.update(&scene, &stylesheet())
    }

    fn find_frame<'a>(frame: &'a LayoutFrame, id: &str) -> Option<&'a LayoutFrame> {
        if frame.id.as_str() == id {
            return Some(frame);
        }
        frame
            .children
            .iter()
            .find_map(|child| find_frame(child, id))
    }

    fn frame<'a>(output: &'a RuntimeOutput, id: &str) -> &'a LayoutFrame {
        find_frame(&output.layout, id).unwrap_or_else(|| panic!("expected layout frame for {id}"))
    }

    fn assert_close(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() < 0.01,
            "expected {actual} to be close to {expected}"
        );
    }

    #[test]
    fn kittest_renders_lab_frame_to_shapes() {
        let mut harness = lab_harness("layout");

        harness.run();

        assert!(
            harness.output().shapes.len() > 20,
            "expected the UI lab to produce a non-trivial painted scene"
        );
    }

    #[test]
    fn kittest_renders_lab_frame_to_pixels() {
        let image = lab_image("layout");
        let stats = image_stats(&image);

        assert_eq!(stats.width, TEST_WIDTH as u32);
        assert_eq!(stats.height, TEST_HEIGHT as u32);
        assert!(
            stats.non_transparent_pixels > stats.total_pixels / 4,
            "expected the rendered UI lab image to contain visible pixels"
        );
    }

    #[test]
    fn kittest_pointer_click_reaches_runtime_owned_nav_item() {
        let mut harness = lab_harness("layout");
        let rect = lab_rect("view-interaction");
        let interaction_nav_item = egui::pos2(
            rect.origin.x + rect.size.width / 2.0,
            rect.origin.y + rect.size.height / 2.0,
        );

        harness.hover_at(interaction_nav_item);
        harness.drag_at(interaction_nav_item);
        harness.drop_at(interaction_nav_item);
        harness.run();

        assert_eq!(harness.state().view, LabView::Interaction);
    }

    #[test]
    fn graphical_comparison_matches_identical_lab_views() {
        let first = lab_image("layout");
        let second = lab_image("layout");

        assert_exact_image_match(&first, &second);
    }

    #[test]
    fn graphical_comparison_detects_different_lab_views() {
        let layout = lab_image("layout");
        let scrolling = lab_image("scrolling");
        let comparison = compare_images(&layout, &scrolling);

        assert!(
            comparison.differing_pixels > comparison.compared_pixels / 20,
            "expected visibly different lab views, got {comparison:?}"
        );
    }

    #[test]
    fn clicked_nav_view_matches_directly_seeded_view() {
        let mut clicked = lab_harness("layout");
        let rect = lab_rect("view-interaction");
        let interaction_nav_item = egui::pos2(
            rect.origin.x + rect.size.width / 2.0,
            rect.origin.y + rect.size.height / 2.0,
        );

        clicked.hover_at(interaction_nav_item);
        clicked.drag_at(interaction_nav_item);
        clicked.drop_at(interaction_nav_item);
        clicked.run();

        let clicked_image = render_harness(&mut clicked);
        let direct_image = lab_image("interaction");

        assert_exact_image_match(&clicked_image, &direct_image);
    }

    #[test]
    fn box_model_specimens_cover_size_inset_and_flow_contracts() {
        let output = lab_output("layout");

        assert_close(frame(&output, "box-auto-subject").rect.size.width, 12.0);
        assert_close(frame(&output, "box-auto-subject").rect.size.height, 12.0);
        assert_close(frame(&output, "box-px-subject").rect.size.width, 96.0);
        assert_close(frame(&output, "box-px-subject").rect.size.height, 44.0);
        assert_close(frame(&output, "box-min-subject").rect.size.width, 40.0);
        assert_close(frame(&output, "box-min-subject").rect.size.height, 40.0);

        assert_close(frame(&output, "box-fill-subject").rect.size.width, 292.0);
        assert_close(frame(&output, "box-percent-subject").rect.size.width, 146.0);
        assert_close(
            frame(&output, "box-height-fill-subject").rect.size.height,
            84.0,
        );

        let margin_subject = frame(&output, "box-margin-subject");
        let margin_frame = frame(&output, "box-margin-frame");
        assert_close(
            margin_subject.rect.origin.x - margin_frame.rect.origin.x,
            13.0,
        );
        assert_close(
            margin_subject.rect.origin.y - margin_frame.rect.origin.y,
            13.0,
        );

        assert_close(frame(&output, "box-padding-subject").rect.size.width, 36.0);
        assert_close(frame(&output, "box-padding-subject").rect.size.height, 36.0);
        assert_close(frame(&output, "box-border-subject").style.border_width, 5.0);
        assert_close(frame(&output, "box-border-subject").rect.size.width, 44.0);
        assert_close(frame(&output, "box-border-subject").rect.size.height, 44.0);

        assert_close(frame(&output, "box-row-gap-subject").rect.size.width, 56.0);
        assert_close(frame(&output, "box-row-gap-subject").rect.size.height, 12.0);
        let first_row_chip = frame(&output, "box-row-gap-chip-0");
        let second_row_chip = frame(&output, "box-row-gap-chip-1");
        assert_close(
            second_row_chip.rect.origin.x - first_row_chip.rect.origin.x,
            22.0,
        );

        assert_close(
            frame(&output, "box-column-gap-subject").rect.size.width,
            12.0,
        );
        assert_close(
            frame(&output, "box-column-gap-subject").rect.size.height,
            48.0,
        );
        let first_column_chip = frame(&output, "box-column-gap-chip-0");
        let second_column_chip = frame(&output, "box-column-gap-chip-1");
        assert_close(
            second_column_chip.rect.origin.y - first_column_chip.rect.origin.y,
            18.0,
        );

        let visible_overflow_child = frame(&output, "box-visible-overflow-overflow-child");
        let visible_overflow_subject = frame(&output, "box-visible-overflow-subject");
        assert!(
            visible_overflow_child.rect.bottom() > visible_overflow_subject.rect.bottom(),
            "visible overflow child should extend beyond its square subject"
        );

        assert_close(
            frame(&output, "box-nested-nine-subject").rect.size.width,
            74.0,
        );
        assert_close(
            frame(&output, "box-nested-nine-subject").rect.size.height,
            74.0,
        );
        assert_close(
            frame(&output, "box-nested-nine-inner").rect.size.width,
            52.0,
        );
        assert_close(
            frame(&output, "box-nested-nine-inner").rect.size.height,
            52.0,
        );
        assert_close(
            frame(&output, "box-nested-nine-cell-0-1").rect.origin.x
                - frame(&output, "box-nested-nine-cell-0-0").rect.origin.x,
            14.0,
        );
        assert_close(
            frame(&output, "box-nested-nine-cell-1-0").rect.origin.y
                - frame(&output, "box-nested-nine-cell-0-0").rect.origin.y,
            14.0,
        );

        assert_close(
            frame(&output, "box-inset-percent-child").rect.size.width,
            34.0,
        );
        assert_close(
            frame(&output, "box-inset-percent-child").rect.size.height,
            34.0,
        );

        assert!(
            output
                .scroll_chrome
                .iter()
                .any(|chrome| chrome.element_id.as_str() == "box-scroll-overflow-subject"),
            "scroll overflow specimen should emit scroll chrome"
        );
    }
}
