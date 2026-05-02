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

fn render_topbar(ui: &mut des_ui_runtime::Ui, debug_overlay: bool) {
    ui.element(
        "topbar",
        ElementSpec::new(ElementRole::Panel).class("topbar"),
        |ui| {
            ui.text_element(
                "title",
                ElementSpec::new(ElementRole::Text).class("title"),
                "Data Engine Studio UI Lab",
            );
            ui.text_element(
                "subtitle",
                ElementSpec::new(ElementRole::Text).class("muted"),
                if debug_overlay {
                    "runtime layout, style, input, and graph experiments / debug"
                } else {
                    "runtime layout, style, input, and graph experiments"
                },
            );
        },
    );
}

fn render_nav(ui: &mut des_ui_runtime::Ui, selected: LabView) {
    ui.element(
        "nav",
        ElementSpec::new(ElementRole::Panel).class("nav"),
        |ui| {
            ui.text_element(
                "nav-title",
                ElementSpec::new(ElementRole::Text).class("section-title"),
                "Feature Views",
            );
            for view in [
                LabView::Layout,
                LabView::Interaction,
                LabView::Styling,
                LabView::Scrolling,
                LabView::Nesting,
                LabView::Graph,
            ] {
                ui.element(
                    view.id(),
                    ElementSpec::new(ElementRole::Card)
                        .class("nav-item")
                        .interactive()
                        .selected(view == selected),
                    |ui| {
                        ui.text_element(
                            format!("{}-label", view.id()),
                            ElementSpec::new(ElementRole::Text).class("card-title"),
                            view.label(),
                        );
                        ui.text_element(
                            format!("{}-hint", view.id()),
                            ElementSpec::new(ElementRole::Text).class("muted"),
                            view_hint(view),
                        );
                    },
                );
            }
        },
    );
}

fn view_hint(view: LabView) -> &'static str {
    match view {
        LabView::Layout => "nesting, margins, rows, columns",
        LabView::Interaction => "hover, press, click ownership",
        LabView::Styling => "roles, classes, states, ids",
        LabView::Scrolling => "runtime scroll ownership",
        LabView::Nesting => "relative nested boxes",
        LabView::Graph => "canvas and bezier planning",
    }
}

fn render_stage(
    ui: &mut des_ui_runtime::Ui,
    view: LabView,
    show_optional_card: bool,
    dense_mode: bool,
) {
    ui.element(
        "stage",
        ElementSpec::new(ElementRole::Panel).class("stage"),
        |ui| match view {
            LabView::Layout => render_layout_view(ui, show_optional_card, dense_mode),
            LabView::Interaction => render_interaction_view(ui),
            LabView::Styling => render_styling_view(ui, dense_mode),
            LabView::Scrolling => render_scrolling_view(ui),
            LabView::Nesting => render_nesting_view(ui),
            LabView::Graph => render_graph_view(ui),
        },
    );
}

fn render_layout_view(ui: &mut des_ui_runtime::Ui, _show_optional_card: bool, _dense_mode: bool) {
    ui.text_element(
        "layout-heading",
        ElementSpec::new(ElementRole::Text).class("heading"),
        "Box Model Specimens",
    );
    ui.text_element(
        "layout-copy",
        ElementSpec::new(ElementRole::Text).class("muted"),
        "Each subject isolates one layout contract. Selector rules are printed above the specimen.",
    );
    ui.element(
        "box-model-grid",
        ElementSpec::new(ElementRole::Panel).class("box-model-grid"),
        |ui| {
            box_model_row(ui, "box-row-size", |ui| {
                box_model_case(
                    ui,
                    "box-auto",
                    "Auto",
                    "content-sized",
                    ".box-subject-auto { width:auto; height:auto }",
                    "box-subject-auto",
                );
                box_model_case(
                    ui,
                    "box-px",
                    "Px size",
                    "96 x 44",
                    ".box-subject-px { width:96px; height:44px }",
                    "box-subject-px",
                );
                box_model_case(
                    ui,
                    "box-min",
                    "Min size",
                    "empty -> min",
                    ".box-subject-min { auto; min-size:40px }",
                    "box-subject-min",
                );
            });
            box_model_row(ui, "box-row-parent-relative", |ui| {
                box_model_case(
                    ui,
                    "box-fill",
                    "Width fill",
                    "parent content",
                    ".box-subject-fill { width:fill; height:28px }",
                    "box-subject-fill",
                );
                box_model_case(
                    ui,
                    "box-percent",
                    "Width 50%",
                    "parent content",
                    ".box-subject-percent { width:50%; height:28px }",
                    "box-subject-percent",
                );
                box_model_case(
                    ui,
                    "box-height-fill",
                    "Height fill",
                    "parent content",
                    ".box-subject-height-fill { width:64px; height:fill }",
                    "box-subject-height-fill",
                );
            });
            box_model_row(ui, "box-row-insets", |ui| {
                box_model_case(
                    ui,
                    "box-margin",
                    "Margin",
                    "12px outside",
                    ".box-subject-margin { size:32px; margin:12px }",
                    "box-subject-margin",
                );
                box_model_case(
                    ui,
                    "box-padding",
                    "Padding",
                    "12px inside",
                    ".box-subject-padding { auto; padding:12px }",
                    "box-subject-padding",
                );
                box_model_case(
                    ui,
                    "box-border",
                    "Border",
                    "5px inside",
                    ".box-subject-border { size:44px; border:5px }",
                    "box-subject-border",
                );
            });
            box_model_row(ui, "box-row-flow", |ui| {
                box_model_case(
                    ui,
                    "box-row-gap",
                    "Row gap",
                    "3 children",
                    ".box-subject-row-gap { row; auto; gap:10px }",
                    "box-subject-row-gap",
                );
                box_model_case(
                    ui,
                    "box-column-gap",
                    "Column gap",
                    "3 children",
                    ".box-subject-column-gap { column; auto; gap:6px }",
                    "box-subject-column-gap",
                );
                box_model_case(
                    ui,
                    "box-visible-overflow",
                    "Overflow visible",
                    "unclipped child",
                    ".box-subject-visible-overflow { size:44px; overflow:visible }",
                    "box-subject-visible-overflow",
                );
            });
            box_model_row(ui, "box-row-overflow", |ui| {
                box_model_case(
                    ui,
                    "box-scroll-overflow",
                    "Overflow scroll",
                    "clipped content",
                    ".box-subject-scroll-overflow { size:44px; overflow:scroll }",
                    "box-subject-scroll-overflow",
                );
            });
            box_model_section_label(ui, "box-combo-title", "Nested Awareness");
            box_model_row(ui, "box-row-combinations-one", |ui| {
                box_model_case(
                    ui,
                    "box-nested-nine",
                    "Nested auto grid",
                    "outer margin + inner border",
                    ".outer:auto margin8 border3; .inner:auto padding5 border2",
                    "box-subject-nested-nine",
                );
                box_model_case(
                    ui,
                    "box-inset-percent",
                    "Percent insets",
                    "child resolves from content rect",
                    ".parent:88px padding8 border2; .child:50%",
                    "box-subject-inset-percent",
                );
            });
        },
    );
}

fn box_model_section_label(ui: &mut des_ui_runtime::Ui, id: &'static str, label: &'static str) {
    ui.text_element(
        id,
        ElementSpec::new(ElementRole::Text).class("box-section-label"),
        label,
    );
}

fn box_model_row(
    ui: &mut des_ui_runtime::Ui,
    id: &'static str,
    add_contents: impl FnOnce(&mut des_ui_runtime::Ui),
) {
    ui.element(
        id,
        ElementSpec::new(ElementRole::Panel).class("box-model-row"),
        add_contents,
    );
}

fn box_model_case(
    ui: &mut des_ui_runtime::Ui,
    id: &'static str,
    title: &'static str,
    note: &'static str,
    rule_text: &'static str,
    subject_class: &'static str,
) {
    ui.element(
        id,
        ElementSpec::new(ElementRole::Panel).class("box-model-case"),
        |ui| {
            ui.text_element(
                format!("{id}-title"),
                ElementSpec::new(ElementRole::Text).class("box-label"),
                title,
            );
            ui.text_element(
                format!("{id}-note"),
                ElementSpec::new(ElementRole::Text).class("box-note"),
                note,
            );
            ui.text_element(
                format!("{id}-rule"),
                ElementSpec::new(ElementRole::Text).class("box-rule"),
                rule_text,
            );
            ui.element(
                format!("{id}-frame"),
                ElementSpec::new(ElementRole::Panel).class("box-subject-frame"),
                |ui| {
                    box_model_subject(ui, id, subject_class);
                },
            );
        },
    );
}

fn box_model_subject(
    ui: &mut des_ui_runtime::Ui,
    case_id: &'static str,
    subject_class: &'static str,
) {
    ui.element(
        format!("{case_id}-subject"),
        ElementSpec::new(ElementRole::Panel)
            .class("box-subject")
            .class(subject_class),
        |ui| match subject_class {
            "box-subject-auto" => {
                box_chip(ui, case_id, 0);
            }
            "box-subject-padding" => {
                box_chip(ui, case_id, 0);
            }
            "box-subject-row-gap" | "box-subject-column-gap" => {
                box_chip(ui, case_id, 0);
                box_chip(ui, case_id, 1);
                box_chip(ui, case_id, 2);
            }
            "box-subject-visible-overflow" | "box-subject-scroll-overflow" => {
                ui.element(
                    format!("{case_id}-overflow-child"),
                    ElementSpec::new(ElementRole::Panel).class("box-overflow-child"),
                    |_| {},
                );
            }
            "box-subject-nested-nine" => {
                ui.element(
                    format!("{case_id}-outer"),
                    ElementSpec::new(ElementRole::Panel).class("box-nested-outer"),
                    |ui| {
                        ui.element(
                            format!("{case_id}-inner"),
                            ElementSpec::new(ElementRole::Panel).class("box-nested-inner"),
                            |ui| {
                                for row in 0..3 {
                                    ui.element(
                                        format!("{case_id}-row-{row}"),
                                        ElementSpec::new(ElementRole::Panel)
                                            .class("box-nested-row"),
                                        |ui| {
                                            for column in 0..3 {
                                                ui.element(
                                                    format!("{case_id}-cell-{row}-{column}"),
                                                    ElementSpec::new(ElementRole::Panel)
                                                        .class("box-nested-cell"),
                                                    |_| {},
                                                );
                                            }
                                        },
                                    );
                                }
                            },
                        );
                    },
                );
            }
            "box-subject-inset-percent" => {
                ui.element(
                    format!("{case_id}-parent"),
                    ElementSpec::new(ElementRole::Panel).class("box-inset-percent-parent"),
                    |ui| {
                        ui.element(
                            format!("{case_id}-child"),
                            ElementSpec::new(ElementRole::Panel).class("box-inset-percent-child"),
                            |_| {},
                        );
                    },
                );
            }
            _ => {}
        },
    );
}

fn box_chip(ui: &mut des_ui_runtime::Ui, case_id: &'static str, index: usize) {
    ui.element(
        format!("{case_id}-chip-{index}"),
        ElementSpec::new(ElementRole::Panel).class("box-chip"),
        |_| {},
    );
}

fn render_interaction_view(ui: &mut des_ui_runtime::Ui) {
    ui.text_element(
        "interaction-heading",
        ElementSpec::new(ElementRole::Text).class("heading"),
        "Interaction Runtime",
    );
    ui.text_element(
        "interaction-copy",
        ElementSpec::new(ElementRole::Text).class("muted"),
        "Hover and click styles are resolved by runtime state. Inner text does not own clicks.",
    );
    ui.element(
        "interaction-row",
        ElementSpec::new(ElementRole::Panel).class("card-row"),
        |ui| {
            for (id, title, body) in [
                (
                    "interaction-card-one",
                    "Hover Target",
                    "background comes from class:hover",
                ),
                (
                    "interaction-card-two",
                    "Click Target",
                    "interactive owner is the card",
                ),
                (
                    "interaction-card-three",
                    "Pressed Target",
                    "press state resolves before paint",
                ),
            ] {
                ui.element(
                    id,
                    ElementSpec::new(ElementRole::Card)
                        .class("feature-card")
                        .interactive(),
                    |ui| {
                        ui.text_element(
                            format!("{id}-title"),
                            ElementSpec::new(ElementRole::Text).class("card-title"),
                            title,
                        );
                        ui.text_element(
                            format!("{id}-body"),
                            ElementSpec::new(ElementRole::Text).class("muted"),
                            body,
                        );
                    },
                );
            }
        },
    );
}

fn render_styling_view(ui: &mut des_ui_runtime::Ui, dense_mode: bool) {
    ui.text_element(
        "styling-heading",
        ElementSpec::new(ElementRole::Text).class("heading"),
        "Deterministic Styling",
    );
    ui.text_element(
        "styling-copy",
        ElementSpec::new(ElementRole::Text).class("muted"),
        "Style order is role, class, state, id. No CSS specificity maze.",
    );
    ui.element(
        "style-stack",
        ElementSpec::new(ElementRole::Panel).class("stack"),
        |ui| {
            interactive_labeled_row(
                ui,
                "style-row-role",
                "Role",
                "ElementRole::Card sets base surface behavior.",
            );
            interactive_labeled_row(
                ui,
                "style-row-class",
                "Class",
                ".feature-card changes color, radius, and size.",
            );
            interactive_labeled_row(
                ui,
                "style-row-state",
                "State",
                ".feature-card:hover and :pressed adjust paint.",
            );
            interactive_labeled_row(
                ui,
                "style-row-density",
                "App State",
                if dense_mode {
                    "Dense mode is active from the layout view toggle."
                } else {
                    "Dense mode is inactive from the layout view toggle."
                },
            );
        },
    );
}

fn render_scrolling_view(ui: &mut des_ui_runtime::Ui) {
    ui.text_element(
        "scroll-heading",
        ElementSpec::new(ElementRole::Text).class("heading"),
        "Runtime Scrolling",
    );
    ui.text_element(
        "scroll-copy",
        ElementSpec::new(ElementRole::Text).class("muted"),
        "Use the wheel or touchpad over either panel. The scroll offset lives in des-ui-runtime.",
    );
    ui.element(
        "scroll-row",
        ElementSpec::new(ElementRole::Panel).class("card-row"),
        |ui| {
            scroll_panel(ui, "scroll-panel-a", "Project List", 12);
            scroll_panel(ui, "scroll-panel-b", "Preview Rows", 18);
        },
    );
}

fn render_nesting_view(ui: &mut des_ui_runtime::Ui) {
    ui.text_element(
        "nesting-heading",
        ElementSpec::new(ElementRole::Text).class("heading"),
        "Nested Relative Boxes",
    );
    ui.text_element(
        "nesting-copy",
        ElementSpec::new(ElementRole::Text).class("muted"),
        "Each child is positioned relative to its parent content rect. Absolute positioning comes next.",
    );
    ui.element(
        "nest-outer",
        ElementSpec::new(ElementRole::Panel).class("nest-outer"),
        |ui| {
            ui.text_element(
                "nest-outer-title",
                ElementSpec::new(ElementRole::Text).class("card-title"),
                "Outer panel",
            );
            ui.element(
                "nest-middle",
                ElementSpec::new(ElementRole::Card).class("nest-middle"),
                |ui| {
                    ui.text_element(
                        "nest-middle-title",
                        ElementSpec::new(ElementRole::Text).class("card-title"),
                        "Middle card",
                    );
                    ui.element(
                        "nest-inner",
                        ElementSpec::new(ElementRole::Card)
                            .class("nest-inner")
                            .interactive(),
                        |ui| {
                            ui.text_element(
                                "nest-inner-title",
                                ElementSpec::new(ElementRole::Text).class("card-title"),
                                "Inner interactive box",
                            );
                            ui.text_element(
                                "nest-inner-body",
                                ElementSpec::new(ElementRole::Text).class("muted"),
                                "Hover proves hit testing through nested relative frames.",
                            );
                        },
                    );
                },
            );
        },
    );
}

fn render_graph_view(ui: &mut des_ui_runtime::Ui) {
    ui.text_element(
        "graph-heading",
        ElementSpec::new(ElementRole::Text).class("heading"),
        "Graph Surface Plan",
    );
    ui.text_element(
        "graph-copy",
        ElementSpec::new(ElementRole::Text).class("muted"),
        "This placeholder reserves the lab view for canvas, layers, custom geometry, and bezier hit testing.",
    );
    ui.element(
        "graph-canvas-placeholder",
        ElementSpec::new(ElementRole::Canvas).class("canvas-placeholder"),
        |ui| {
            ui.text_element(
                "graph-canvas-title",
                ElementSpec::new(ElementRole::Text).class("card-title"),
                "Canvas adapter target",
            );
            ui.text_element(
                "graph-canvas-body",
                ElementSpec::new(ElementRole::Text).class("muted"),
                "Next: runtime-managed canvas bounds with egui/epaint geometry inside.",
            );
        },
    );
}

fn interactive_labeled_row(
    ui: &mut des_ui_runtime::Ui,
    id: &'static str,
    label: &'static str,
    body: &'static str,
) {
    ui.element(
        id,
        ElementSpec::new(ElementRole::Card)
            .class("list-row")
            .class("specificity-proof")
            .interactive(),
        |ui| {
            ui.text_element(
                format!("{id}-label"),
                ElementSpec::new(ElementRole::Text).class("card-title"),
                label,
            );
            ui.text_element(
                format!("{id}-body"),
                ElementSpec::new(ElementRole::Text).class("muted"),
                body,
            );
        },
    );
}

fn scroll_panel(
    ui: &mut des_ui_runtime::Ui,
    id: &'static str,
    title: &'static str,
    row_count: usize,
) {
    ui.element(
        id,
        ElementSpec::new(ElementRole::Panel).class("scroll-panel"),
        |ui| {
            ui.text_element(
                format!("{id}-title"),
                ElementSpec::new(ElementRole::Text).class("card-title"),
                title,
            );
            ui.element(
                format!("{id}-list"),
                ElementSpec::new(ElementRole::Panel).class("scroll-list"),
                |ui| {
                    for index in 0..row_count {
                        ui.element(
                            format!("{id}-row-{index}"),
                            ElementSpec::new(ElementRole::Card)
                                .class("scroll-row-card")
                                .interactive(),
                            |ui| {
                                ui.text_element(
                                    format!("{id}-row-{index}-label"),
                                    ElementSpec::new(ElementRole::Text).class("muted"),
                                    format!("runtime-owned scroll row {:02}", index + 1),
                                );
                            },
                        );
                    }
                },
            );
        },
    );
}

fn stylesheet() -> StyleSheet {
    StyleSheet::new()
        .rule(
            StyleSelector::Role(ElementRole::Root),
            StylePatch::default()
                .direction(Direction::Column)
                .background(BACKGROUND),
        )
        .rule(
            StyleSelector::Role(ElementRole::Panel),
            StylePatch::default()
                .direction(Direction::Column)
                .background(PANEL),
        )
        .rule(
            StyleSelector::Role(ElementRole::Card),
            StylePatch::default()
                .direction(Direction::Column)
                .padding(Insets::all(12.0))
                .gap(5.0)
                .background(CARD)
                .border(STROKE)
                .radius(7.0),
        )
        .rule(
            StyleSelector::Role(ElementRole::Control),
            StylePatch::default()
                .padding(Insets::symmetric(12.0, 7.0))
                .background(CARD)
                .border(STROKE)
                .radius(5.0),
        )
        .rule(
            StyleSelector::Role(ElementRole::Text),
            StylePatch::default().font_size(13.0).text_color(TEXT),
        )
        .rule(
            StyleSelector::Class("lab-root"),
            StylePatch::default()
                .size(1320.0, 780.0)
                .background(BACKGROUND)
                .gap(0.0),
        )
        .rule(
            StyleSelector::Class("topbar"),
            StylePatch::default()
                .size(1320.0, 58.0)
                .padding(Insets::symmetric(18.0, 10.0))
                .gap(3.0)
                .background(Color::rgb(22, 26, 30)),
        )
        .rule(
            StyleSelector::Class("lab-body"),
            StylePatch::default()
                .direction(Direction::Row)
                .size(1320.0, 722.0)
                .padding(Insets::all(14.0))
                .gap(14.0)
                .background(BACKGROUND),
        )
        .rule(
            StyleSelector::Class("nav"),
            StylePatch::default()
                .size(242.0, 690.0)
                .padding(Insets::all(12.0))
                .gap(8.0)
                .background(PANEL)
                .border(STROKE)
                .radius(8.0)
                .overflow_y(Overflow::Scroll)
                .z_index(10),
        )
        .rule(
            StyleSelector::Class("stage"),
            StylePatch::default()
                .size(1036.0, 690.0)
                .padding(Insets::all(18.0))
                .gap(12.0)
                .background(PANEL_ALT)
                .border(STROKE)
                .radius(8.0)
                .overflow_y(Overflow::Scroll),
        )
        .rule(
            StyleSelector::Class("box-model-grid"),
            StylePatch::default()
                .width_fill()
                .gap(10.0)
                .background(PANEL_ALT),
        )
        .rule(
            StyleSelector::Class("box-model-row"),
            StylePatch::default()
                .direction(Direction::Row)
                .width_fill()
                .height(Length::Px(172.0))
                .gap(10.0)
                .background(PANEL_ALT),
        )
        .rule(
            StyleSelector::Class("box-model-case"),
            StylePatch::default()
                .size(318.0, 172.0)
                .padding(Insets::all(8.0))
                .gap(3.0)
                .background(Color::rgb(20, 24, 28))
                .border(Color::rgb(45, 54, 62))
                .radius(5.0),
        )
        .rule(
            StyleSelector::Class("box-section-label"),
            StylePatch::default()
                .font_size(14.0)
                .text_color(TEXT_ACCENT)
                .height(Length::Px(18.0)),
        )
        .rule(
            StyleSelector::Class("box-label"),
            StylePatch::default()
                .font_size(13.0)
                .text_color(TEXT)
                .height(Length::Px(16.0)),
        )
        .rule(
            StyleSelector::Class("box-note"),
            StylePatch::default()
                .font_size(11.0)
                .text_color(TEXT_MUTED)
                .height(Length::Px(14.0)),
        )
        .rule(
            StyleSelector::Class("box-rule"),
            StylePatch::default()
                .font_size(10.0)
                .text_color(TEXT_ACCENT)
                .height(Length::Px(24.0)),
        )
        .rule(
            StyleSelector::Class("box-subject-frame"),
            StylePatch::default()
                .size(294.0, 86.0)
                .background(Color::rgb(13, 16, 19))
                .border(Color::rgb(39, 48, 56))
                .overflow_y(Overflow::Visible),
        )
        .rule(
            StyleSelector::Class("box-subject"),
            StylePatch::default()
                .size(32.0, 32.0)
                .gap(0.0)
                .padding(Insets::ZERO)
                .background(Color::rgb(65, 121, 164)),
        )
        .rule(
            StyleSelector::Class("box-chip"),
            StylePatch::default()
                .size(12.0, 12.0)
                .background(Color::rgb(141, 207, 164)),
        )
        .rule(
            StyleSelector::Class("box-overflow-child"),
            StylePatch::default()
                .size(112.0, 112.0)
                .background(Color::rgb(218, 151, 77)),
        )
        .rule(
            StyleSelector::Class("box-subject-auto"),
            StylePatch::default()
                .width(Length::Auto)
                .height(Length::Auto),
        )
        .rule(
            StyleSelector::Class("box-subject-px"),
            StylePatch::default().size(96.0, 44.0),
        )
        .rule(
            StyleSelector::Class("box-subject-min"),
            StylePatch::default()
                .width(Length::Auto)
                .height(Length::Auto)
                .min_size(40.0, 40.0),
        )
        .rule(
            StyleSelector::Class("box-subject-fill"),
            StylePatch::default().width_fill().height(Length::Px(28.0)),
        )
        .rule(
            StyleSelector::Class("box-subject-percent"),
            StylePatch::default()
                .width_percent(0.5)
                .height(Length::Px(28.0)),
        )
        .rule(
            StyleSelector::Class("box-subject-height-fill"),
            StylePatch::default().width(Length::Px(64.0)).height_fill(),
        )
        .rule(
            StyleSelector::Class("box-subject-margin"),
            StylePatch::default()
                .size(32.0, 32.0)
                .margin(Insets::all(12.0)),
        )
        .rule(
            StyleSelector::Class("box-subject-padding"),
            StylePatch::default()
                .width(Length::Auto)
                .height(Length::Auto)
                .padding(Insets::all(12.0)),
        )
        .rule(
            StyleSelector::Class("box-subject-border"),
            StylePatch::default()
                .size(44.0, 44.0)
                .border(PURPLE)
                .border_width(5.0),
        )
        .rule(
            StyleSelector::Class("box-subject-row-gap"),
            StylePatch::default()
                .direction(Direction::Row)
                .width(Length::Auto)
                .height(Length::Auto)
                .gap(10.0),
        )
        .rule(
            StyleSelector::Class("box-subject-column-gap"),
            StylePatch::default()
                .direction(Direction::Column)
                .width(Length::Auto)
                .height(Length::Auto)
                .gap(6.0),
        )
        .rule(
            StyleSelector::Class("box-subject-visible-overflow"),
            StylePatch::default()
                .size(44.0, 44.0)
                .overflow_y(Overflow::Visible),
        )
        .rule(
            StyleSelector::Class("box-subject-scroll-overflow"),
            StylePatch::default()
                .size(44.0, 44.0)
                .overflow_y(Overflow::Scroll),
        )
        .rule(
            StyleSelector::Class("box-subject-nested-nine"),
            StylePatch::default()
                .width(Length::Auto)
                .height(Length::Auto),
        )
        .rule(
            StyleSelector::Class("box-nested-outer"),
            StylePatch::default()
                .width(Length::Auto)
                .height(Length::Auto)
                .margin(Insets::all(8.0))
                .border(PURPLE)
                .border_width(3.0)
                .background(Color::rgb(41, 58, 73)),
        )
        .rule(
            StyleSelector::Class("box-nested-inner"),
            StylePatch::default()
                .width(Length::Auto)
                .height(Length::Auto)
                .padding(Insets::all(5.0))
                .gap(4.0)
                .border(GREEN)
                .border_width(2.0)
                .background(Color::rgb(52, 72, 88)),
        )
        .rule(
            StyleSelector::Class("box-nested-row"),
            StylePatch::default()
                .direction(Direction::Row)
                .width(Length::Auto)
                .height(Length::Auto)
                .gap(4.0)
                .background(Color::rgb(65, 121, 164)),
        )
        .rule(
            StyleSelector::Class("box-nested-cell"),
            StylePatch::default()
                .size(10.0, 10.0)
                .background(Color::rgb(141, 207, 164)),
        )
        .rule(
            StyleSelector::Class("box-subject-inset-percent"),
            StylePatch::default()
                .width(Length::Auto)
                .height(Length::Auto),
        )
        .rule(
            StyleSelector::Class("box-inset-percent-parent"),
            StylePatch::default()
                .size(88.0, 88.0)
                .padding(Insets::all(8.0))
                .border(PURPLE)
                .border_width(2.0)
                .background(Color::rgb(65, 121, 164)),
        )
        .rule(
            StyleSelector::Class("box-inset-percent-child"),
            StylePatch::default()
                .width_percent(0.5)
                .height_percent(0.5)
                .background(Color::rgb(141, 207, 164)),
        )
        .rule(
            StyleSelector::Class("nav-item"),
            StylePatch::default()
                .width_fill()
                .height(des_ui_runtime::Length::Px(64.0))
                .background(CARD)
                .border(STROKE),
        )
        .rule(
            StyleSelector::ClassState("nav-item", ElementStateSelector::Selected),
            StylePatch::default()
                .background(CARD_SELECTED)
                .border(STROKE_SELECTED),
        )
        .rule(
            StyleSelector::ClassState("nav-item", ElementStateSelector::Hovered),
            StylePatch::default()
                .background(CARD_HOVER)
                .border(STROKE_SELECTED),
        )
        .rule(
            StyleSelector::Class("toolbar-row"),
            StylePatch::default()
                .direction(Direction::Row)
                .gap(8.0)
                .background(PANEL_ALT),
        )
        .rule(
            StyleSelector::Class("button"),
            StylePatch::default()
                .size(156.0, 36.0)
                .background(Color::rgb(38, 43, 48))
                .border(STROKE),
        )
        .rule(
            StyleSelector::ClassState("button", ElementStateSelector::Selected),
            StylePatch::default()
                .background(CARD_SELECTED)
                .border(STROKE_SELECTED),
        )
        .rule(
            StyleSelector::ClassState("button", ElementStateSelector::Hovered),
            StylePatch::default()
                .background(CARD_HOVER)
                .border(STROKE_SELECTED),
        )
        .rule(
            StyleSelector::ClassState("button", ElementStateSelector::Pressed),
            StylePatch::default().background(CARD_PRESSED),
        )
        .rule(
            StyleSelector::Class("button-label"),
            StylePatch::default().font_size(13.0).text_color(TEXT),
        )
        .rule(
            StyleSelector::Class("card-row"),
            StylePatch::default()
                .direction(Direction::Row)
                .gap(12.0)
                .background(PANEL_ALT),
        )
        .rule(
            StyleSelector::Class("card-row-dense"),
            StylePatch::default()
                .direction(Direction::Row)
                .gap(6.0)
                .background(PANEL_ALT),
        )
        .rule(
            StyleSelector::Class("feature-card"),
            StylePatch::default()
                .size(250.0, 98.0)
                .background(CARD)
                .border(STROKE),
        )
        .rule(
            StyleSelector::ClassState("feature-card", ElementStateSelector::Hovered),
            StylePatch::default()
                .background(CARD_HOVER)
                .border(STROKE_SELECTED),
        )
        .rule(
            StyleSelector::ClassState("feature-card", ElementStateSelector::Pressed),
            StylePatch::default().background(CARD_PRESSED),
        )
        .rule(
            StyleSelector::Class("stack"),
            StylePatch::default()
                .size(620.0, 320.0)
                .padding(Insets::all(10.0))
                .gap(8.0)
                .background(PANEL)
                .border(STROKE)
                .radius(7.0),
        )
        .rule(
            StyleSelector::Class("list-row"),
            StylePatch::default()
                .size(600.0, 58.0)
                .background(Color::rgb(25, 30, 34))
                .border(STROKE)
                .radius(5.0),
        )
        .rule(
            StyleSelector::Class("specificity-proof"),
            StylePatch::default()
                .background(Color::rgb(30, 37, 43))
                .border(Color::rgb(80, 91, 103)),
        )
        .rule(
            StyleSelector::ClassState("specificity-proof", ElementStateSelector::Hovered),
            StylePatch::default()
                .background(Color::rgb(38, 55, 64))
                .border(GREEN),
        )
        .rule(
            StyleSelector::Id("style-row-state"),
            StylePatch::default().border(PURPLE),
        )
        .rule(
            StyleSelector::IdState("style-row-state", ElementStateSelector::Hovered),
            StylePatch::default()
                .background(Color::rgb(50, 41, 68))
                .border(TEXT_ACCENT),
        )
        .rule(
            StyleSelector::Class("scroll-panel"),
            StylePatch::default()
                .size(318.0, 420.0)
                .padding(Insets::all(10.0))
                .gap(7.0)
                .background(Color::rgb(20, 24, 28))
                .border(STROKE)
                .radius(7.0),
        )
        .rule(
            StyleSelector::Class("scroll-list"),
            StylePatch::default()
                .width_fill()
                .height(des_ui_runtime::Length::Px(370.0))
                .padding(Insets::symmetric(4.0, 4.0))
                .gap(7.0)
                .overflow_y(Overflow::Scroll),
        )
        .rule(
            StyleSelector::ClassState("scroll-panel", ElementStateSelector::Hovered),
            StylePatch::default().border(STROKE_SELECTED),
        )
        .rule(
            StyleSelector::Class("scroll-row-card"),
            StylePatch::default()
                .width_fill()
                .height(des_ui_runtime::Length::Px(34.0))
                .padding(Insets::symmetric(9.0, 7.0))
                .background(Color::rgb(29, 34, 39))
                .border(Color::rgb(48, 57, 65))
                .radius(4.0),
        )
        .rule(
            StyleSelector::ClassState("scroll-row-card", ElementStateSelector::Hovered),
            StylePatch::default()
                .background(Color::rgb(38, 47, 54))
                .border(STROKE_SELECTED),
        )
        .rule(
            StyleSelector::Class("nest-outer"),
            StylePatch::default()
                .size(650.0, 430.0)
                .padding(Insets::all(28.0))
                .gap(16.0)
                .background(Color::rgb(20, 24, 29))
                .border(STROKE)
                .radius(8.0),
        )
        .rule(
            StyleSelector::Class("nest-middle"),
            StylePatch::default()
                .size(500.0, 270.0)
                .padding(Insets::all(24.0))
                .gap(14.0)
                .background(Color::rgb(31, 43, 52))
                .border(STROKE_SELECTED)
                .radius(7.0),
        )
        .rule(
            StyleSelector::Class("nest-inner"),
            StylePatch::default()
                .size(360.0, 130.0)
                .padding(Insets::all(18.0))
                .gap(6.0)
                .background(Color::rgb(42, 37, 57))
                .border(PURPLE)
                .radius(7.0),
        )
        .rule(
            StyleSelector::ClassState("nest-inner", ElementStateSelector::Hovered),
            StylePatch::default()
                .background(Color::rgb(55, 50, 78))
                .border(TEXT_ACCENT),
        )
        .rule(
            StyleSelector::Class("canvas-placeholder"),
            StylePatch::default()
                .size(720.0, 360.0)
                .padding(Insets::all(18.0))
                .gap(8.0)
                .background(Color::rgb(15, 18, 21))
                .border(Color::rgb(72, 82, 92))
                .radius(7.0),
        )
        .rule(
            StyleSelector::Class("title"),
            StylePatch::default().font_size(21.0).text_color(TEXT),
        )
        .rule(
            StyleSelector::Class("heading"),
            StylePatch::default().font_size(24.0).text_color(TEXT),
        )
        .rule(
            StyleSelector::Class("section-title"),
            StylePatch::default()
                .font_size(13.0)
                .text_color(TEXT_ACCENT),
        )
        .rule(
            StyleSelector::Class("card-title"),
            StylePatch::default().font_size(16.0).text_color(TEXT),
        )
        .rule(
            StyleSelector::Class("muted"),
            StylePatch::default().font_size(12.5).text_color(TEXT_MUTED),
        )
        .rule(
            StyleSelector::IdState("interaction-card-two", ElementStateSelector::Hovered),
            StylePatch::default()
                .border(GREEN)
                .transition(Transition::ease_out(0.24)),
        )
        .rule(
            StyleSelector::Id("interaction-card-three"),
            StylePatch::default().transition(Transition::ease_out(0.06)),
        )
        .rule(
            StyleSelector::IdState("interaction-card-three", ElementStateSelector::Pressed),
            StylePatch::default()
                .background(Color::rgb(53, 38, 70))
                .border(PURPLE),
        )
}

fn runtime_input(ui: &egui::Ui, origin: egui::Pos2) -> RuntimeInput {
    ui.input(|input| RuntimeInput {
        pointer: input.pointer.hover_pos().map(|position| PointerInput {
            position: Point::new(position.x - origin.x, position.y - origin.y),
            primary_delta: Point::new(input.pointer.delta().x, input.pointer.delta().y),
            primary_down: input.pointer.primary_down(),
            primary_clicked: input.pointer.primary_clicked(),
        }),
        scroll_delta: Point::new(input.smooth_scroll_delta.x, input.smooth_scroll_delta.y),
    })
}

fn paint_frame(ui: &mut egui::Ui, origin: egui::Pos2, frame: &LayoutFrame) {
    paint_frame_clipped(ui, origin, frame, ui.clip_rect());
}

fn paint_frame_clipped(
    ui: &mut egui::Ui,
    origin: egui::Pos2,
    frame: &LayoutFrame,
    clip_rect: egui::Rect,
) {
    let painter = ui.painter().with_clip_rect(clip_rect);
    if frame.id.as_str() != "root" {
        let rect = egui::Rect::from_min_size(
            egui::pos2(
                origin.x + frame.rect.origin.x,
                origin.y + frame.rect.origin.y,
            ),
            egui::vec2(frame.rect.size.width, frame.rect.size.height),
        );

        if let Some(color) = frame.style.background {
            painter.rect_filled(rect, frame.style.radius, to_egui_color(color));
        }

        if let Some(color) = frame.style.border {
            painter.rect_stroke(
                rect,
                frame.style.radius,
                egui::Stroke::new(frame.style.border_width, to_egui_color(color)),
                egui::StrokeKind::Inside,
            );
        }

        if let Some(text) = &frame.text {
            painter.text(
                rect.min,
                egui::Align2::LEFT_TOP,
                text,
                egui::FontId::proportional(frame.style.font_size),
                to_egui_color(frame.style.text_color),
            );
        }
    }

    let mut children: Vec<_> = frame.children.iter().collect();
    children.sort_by_key(|child| child.style.z_index);
    let next_clip = if frame.style.overflow_y == Overflow::Scroll {
        let rect = egui::Rect::from_min_size(
            egui::pos2(
                origin.x + frame.rect.origin.x,
                origin.y + frame.rect.origin.y,
            ),
            egui::vec2(frame.rect.size.width, frame.rect.size.height),
        );
        let content_rect = egui::Rect::from_min_max(
            egui::pos2(
                rect.left() + frame.style.border_width + frame.style.padding.left,
                rect.top() + frame.style.border_width + frame.style.padding.top,
            ),
            egui::pos2(
                rect.right() - frame.style.border_width - frame.style.padding.right,
                rect.bottom() - frame.style.border_width - frame.style.padding.bottom,
            ),
        );
        clip_rect.intersect(content_rect)
    } else {
        clip_rect
    };
    for child in children {
        paint_frame_clipped(ui, origin, child, next_clip);
    }
}

fn paint_scroll_chrome(
    ui: &mut egui::Ui,
    origin: egui::Pos2,
    chromes: &[des_ui_runtime::ScrollChrome],
) {
    let painter = ui.painter();
    for chrome in chromes {
        if !chrome.visible {
            continue;
        }

        let track = runtime_rect_to_egui(origin, chrome.track_rect);
        let handle = runtime_rect_to_egui(origin, chrome.handle_rect);
        let alpha = if chrome.dragged {
            235
        } else if chrome.hovered {
            220
        } else {
            118
        };
        let track_alpha = if chrome.dragged || chrome.hovered {
            84
        } else {
            0
        };
        if track_alpha > 0 {
            painter.rect_filled(
                track,
                6.0,
                egui::Color32::from_rgba_unmultiplied(2, 8, 12, track_alpha),
            );
        }
        painter.rect_filled(
            handle,
            6.0,
            egui::Color32::from_rgba_unmultiplied(232, 236, 240, alpha),
        );
    }
}

fn runtime_rect_to_egui(origin: egui::Pos2, rect: des_ui_runtime::Rect) -> egui::Rect {
    egui::Rect::from_min_size(
        egui::pos2(origin.x + rect.origin.x, origin.y + rect.origin.y),
        egui::vec2(rect.size.width, rect.size.height),
    )
}

fn to_egui_color(color: Color) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(color.r, color.g, color.b, color.a)
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
