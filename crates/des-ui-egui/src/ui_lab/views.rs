use super::*;

pub(super) fn render_topbar(ui: &mut des_ui_document::DocumentBuilder, debug_overlay: bool) {
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
                    "document layout, style, input, and graph experiments / debug"
                } else {
                    "document layout, style, input, and graph experiments"
                },
            );
        },
    );
}

pub(super) fn render_nav(ui: &mut des_ui_document::DocumentBuilder, selected: LabView) {
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
                LabView::Animation,
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
        LabView::Animation => "state transitions and easing",
        LabView::Scrolling => "document scroll ownership",
        LabView::Nesting => "relative nested boxes",
        LabView::Graph => "canvas and bezier planning",
    }
}

pub(super) fn render_stage(
    ui: &mut des_ui_document::DocumentBuilder,
    view: LabView,
    show_optional_card: bool,
    dense_mode: bool,
    checkbox_enabled: bool,
    radio_choice: usize,
    dropdown_open: bool,
    dropdown_choice: usize,
) {
    ui.element(
        "stage",
        ElementSpec::new(ElementRole::Panel)
            .class("stage")
            .class("styled-scrollbar"),
        |ui| match view {
            LabView::Layout => render_layout_view(ui, show_optional_card, dense_mode),
            LabView::Interaction => render_interaction_view(
                ui,
                checkbox_enabled,
                radio_choice,
                dropdown_open,
                dropdown_choice,
            ),
            LabView::Styling => render_styling_view(ui, dense_mode),
            LabView::Animation => render_animation_view(ui),
            LabView::Scrolling => render_scrolling_view(ui),
            LabView::Nesting => render_nesting_view(ui),
            LabView::Graph => render_graph_view(ui),
        },
    );
}

fn render_layout_view(
    ui: &mut des_ui_document::DocumentBuilder,
    _show_optional_card: bool,
    _dense_mode: bool,
) {
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
                    "content-sized container",
                    "width: Auto; height: Auto",
                    "box-subject-auto",
                );
                box_model_case(
                    ui,
                    "box-px",
                    "Fixed size",
                    "width 96 pixels, height 44 pixels",
                    "width: Px(96); height: Px(44)",
                    "box-subject-px",
                );
                box_model_case(
                    ui,
                    "box-min",
                    "Min size",
                    "empty element expands to minimum size",
                    "width: Auto; height: Auto; minimum size: 40 by 40",
                    "box-subject-min",
                );
                box_model_case(
                    ui,
                    "box-max",
                    "Max size",
                    "auto content is clamped by maximum size",
                    "width: Auto; height: Auto; child size 88 by 48; maximum size 52 by 34",
                    "box-subject-max",
                );
            });
            box_model_row(ui, "box-row-parent-relative", |ui| {
                box_model_case(
                    ui,
                    "box-fill",
                    "Width fill",
                    "fills parent content width",
                    "width: Fill; height: Px(28)",
                    "box-subject-fill",
                );
                box_model_case(
                    ui,
                    "box-percent",
                    "Width 50%",
                    "resolves from parent content width",
                    "width: Percent(0.5); height: Px(28)",
                    "box-subject-percent",
                );
                box_model_case(
                    ui,
                    "box-height-fill",
                    "Height fill",
                    "fills parent content height",
                    "width: Px(64); height: Fill",
                    "box-subject-height-fill",
                );
            });
            box_model_row(ui, "box-row-insets", |ui| {
                box_model_case(
                    ui,
                    "box-margin",
                    "Margin",
                    "12 pixels outside the border box",
                    "size: 32 by 32; margin: all sides 12",
                    "box-subject-margin",
                );
                box_model_case(
                    ui,
                    "box-padding",
                    "Padding",
                    "12 pixels inside the border box",
                    "width: Auto; height: Auto; padding: all sides 12",
                    "box-subject-padding",
                );
                box_model_case(
                    ui,
                    "box-border",
                    "Border",
                    "5 pixels on every side",
                    "size: 44 by 44; border width: all sides 5",
                    "box-subject-border",
                );
            });
            box_model_row(ui, "box-row-flow", |ui| {
                box_model_case(
                    ui,
                    "box-row-gap",
                    "Row gap",
                    "3 children",
                    "direction: Row; width: Auto; height: Auto; gap: 10",
                    "box-subject-row-gap",
                );
                box_model_case(
                    ui,
                    "box-column-gap",
                    "Column gap",
                    "3 children",
                    "direction: Column; width: Auto; height: Auto; gap: 6",
                    "box-subject-column-gap",
                );
                box_model_case(
                    ui,
                    "box-visible-overflow",
                    "Overflow visible",
                    "unclipped child",
                    "size: 44 by 44; vertical overflow: Visible",
                    "box-subject-visible-overflow",
                );
            });
            box_model_row(ui, "box-row-alignment", |ui| {
                box_model_case(
                    ui,
                    "box-row-align",
                    "Row alignment",
                    "children centered on main axis and end-aligned on cross axis",
                    "direction: Row; size: 96 by 54; gap: 8; justify content: Center; align items: End",
                    "box-subject-row-align",
                );
                box_model_case(
                    ui,
                    "box-column-align",
                    "Column alignment",
                    "children spaced on main axis and centered on cross axis",
                    "direction: Column; size: 80 by 92; gap: 4; justify content: SpaceBetween; align items: Center",
                    "box-subject-column-align",
                );
            });
            box_model_row(ui, "box-row-overflow", |ui| {
                box_model_case(
                    ui,
                    "box-scroll-overflow",
                    "Overflow scroll",
                    "clipped content",
                    "size: 44 by 44; vertical overflow: Scroll",
                    "box-subject-scroll-overflow",
                );
                box_model_case(
                    ui,
                    "box-scroll-x-overflow",
                    "Overflow x scroll",
                    "horizontal clipped content",
                    "size: 44 by 44; horizontal overflow: Scroll; vertical overflow: Visible",
                    "box-subject-scroll-x-overflow",
                );
                box_model_case(
                    ui,
                    "box-scroll-xy-overflow",
                    "Overflow two-axis",
                    "horizontal and vertical clipped content",
                    "size: 44 by 44; horizontal overflow: Scroll; vertical overflow: Scroll",
                    "box-subject-scroll-xy-overflow",
                );
            });
            box_model_row(ui, "box-row-edges", |ui| {
                box_model_case(
                    ui,
                    "box-side-radius",
                    "Side + corner overrides",
                    "CSS-like layered edges",
                    "base: border width all sides 2; radius all corners 4 | override: left border width 8; bottom border width 5 | override: top-right radius 14; bottom-left radius 0",
                    "box-subject-side-radius",
                );
            });
            box_model_row(ui, "box-row-positioning", |ui| {
                box_model_case(
                    ui,
                    "box-absolute-parent",
                    "Absolute parent",
                    "child is anchored to parent content",
                    "parent: size 88 by 64; padding all sides 8; border width all sides 2 | child: position AbsoluteParent; top 8; left 14",
                    "box-subject-absolute-parent",
                );
                box_model_case(
                    ui,
                    "box-absolute-window",
                    "Absolute window",
                    "child is anchored to viewport coordinates",
                    "child: position AbsoluteViewport; top 140; left 420",
                    "box-subject-absolute-window",
                );
            });
            box_model_section_label(ui, "box-combo-title", "Nested Awareness");
            box_model_row(ui, "box-row-combinations-one", |ui| {
                box_model_case(
                    ui,
                    "box-nested-nine",
                    "Nested auto grid",
                    "outer margin + inner border",
                    "outer: Auto size; margin all sides 8; border width all sides 3 | inner: Auto size; padding all sides 5; border width all sides 2",
                    "box-subject-nested-nine",
                );
                box_model_case(
                    ui,
                    "box-inset-percent",
                    "Percent insets",
                    "child resolves from content rect",
                    "parent: size 88 by 88; padding all sides 8; border width all sides 2 | child: width Percent(0.5); height Percent(0.5)",
                    "box-subject-inset-percent",
                );
            });
        },
    );
}

fn box_model_section_label(
    ui: &mut des_ui_document::DocumentBuilder,
    id: &'static str,
    label: &'static str,
) {
    ui.text_element(
        id,
        ElementSpec::new(ElementRole::Text).class("box-section-label"),
        label,
    );
}

fn box_model_row(
    ui: &mut des_ui_document::DocumentBuilder,
    id: &'static str,
    add_contents: impl FnOnce(&mut des_ui_document::DocumentBuilder),
) {
    ui.element(
        id,
        ElementSpec::new(ElementRole::Panel).class("box-model-row"),
        add_contents,
    );
}

fn box_model_case(
    ui: &mut des_ui_document::DocumentBuilder,
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
            for (line_index, line) in rule_text.split(" | ").enumerate() {
                ui.text_element(
                    format!("{id}-rule-{line_index}"),
                    ElementSpec::new(ElementRole::Text).class("box-rule"),
                    line,
                );
            }
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
    ui: &mut des_ui_document::DocumentBuilder,
    case_id: &'static str,
    subject_class: &'static str,
) {
    let mut spec = ElementSpec::new(ElementRole::Panel)
        .class("box-subject")
        .class(subject_class);
    if matches!(
        subject_class,
        "box-subject-scroll-overflow"
            | "box-subject-scroll-x-overflow"
            | "box-subject-scroll-xy-overflow"
    ) {
        spec = spec.class("styled-scrollbar");
    }

    ui.element(
        format!("{case_id}-subject"),
        spec,
        |ui| match subject_class {
            "box-subject-auto" => {
                box_chip(ui, case_id, 0);
            }
            "box-subject-padding" => {
                box_chip(ui, case_id, 0);
            }
            "box-subject-max" => {
                ui.element(
                    format!("{case_id}-wide-child"),
                    ElementSpec::new(ElementRole::Panel).class("box-max-wide-child"),
                    |_| {},
                );
            }
            "box-subject-row-gap" | "box-subject-column-gap" => {
                box_chip(ui, case_id, 0);
                box_chip(ui, case_id, 1);
                box_chip(ui, case_id, 2);
            }
            "box-subject-row-align" => {
                box_chip(ui, case_id, 0);
                box_chip(ui, case_id, 1);
            }
            "box-subject-column-align" => {
                box_chip(ui, case_id, 0);
                box_chip(ui, case_id, 1);
                box_chip(ui, case_id, 2);
            }
            "box-subject-visible-overflow"
            | "box-subject-scroll-overflow"
            | "box-subject-scroll-x-overflow"
            | "box-subject-scroll-xy-overflow" => {
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
            "box-subject-absolute-parent" => {
                ui.element(
                    format!("{case_id}-parent"),
                    ElementSpec::new(ElementRole::Panel).class("box-absolute-parent-frame"),
                    |ui| {
                        ui.element(
                            format!("{case_id}-flow-child"),
                            ElementSpec::new(ElementRole::Panel).class("box-absolute-flow-child"),
                            |_| {},
                        );
                        ui.element(
                            format!("{case_id}-child"),
                            ElementSpec::new(ElementRole::Panel).class("box-absolute-parent-child"),
                            |_| {},
                        );
                    },
                );
            }
            "box-subject-absolute-window" => {
                ui.element(
                    format!("{case_id}-host"),
                    ElementSpec::new(ElementRole::Panel).class("box-absolute-window-host"),
                    |ui| {
                        ui.element(
                            format!("{case_id}-child"),
                            ElementSpec::new(ElementRole::Panel).class("box-absolute-window-child"),
                            |_| {},
                        );
                    },
                );
            }
            _ => {}
        },
    );
}

fn box_chip(ui: &mut des_ui_document::DocumentBuilder, case_id: &'static str, index: usize) {
    ui.element(
        format!("{case_id}-chip-{index}"),
        ElementSpec::new(ElementRole::Panel).class("box-chip"),
        |_| {},
    );
}

fn render_interaction_view(
    ui: &mut des_ui_document::DocumentBuilder,
    checkbox_enabled: bool,
    radio_choice: usize,
    dropdown_open: bool,
    dropdown_choice: usize,
) {
    ui.text_element(
        "interaction-heading",
        ElementSpec::new(ElementRole::Text).class("heading"),
        "Document Interaction",
    );
    ui.text_element(
        "interaction-copy",
        ElementSpec::new(ElementRole::Text).class("muted"),
        "Hover and click styles are resolved by document state. Inner text does not own clicks.",
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
    ui.text_element(
        "controls-title",
        ElementSpec::new(ElementRole::Text).class("section-title"),
        "Control roles",
    );
    ui.element(
        "controls-grid",
        ElementSpec::new(ElementRole::Panel).class("controls-grid"),
        |ui| {
            control_checkbox(ui, checkbox_enabled);
            control_radio_group(ui, radio_choice);
            control_dropdown(ui, dropdown_open, dropdown_choice);
            control_text_inputs(ui);
        },
    );
}

fn control_checkbox(ui: &mut des_ui_document::DocumentBuilder, checked: bool) {
    ui.element(
        "control-checkbox-card",
        ElementSpec::new(ElementRole::Card).class("control-card"),
        |ui| {
            ui.text_element(
                "control-checkbox-title",
                ElementSpec::new(ElementRole::Text).class("card-title"),
                "Checkbox",
            );
            ui.element(
                "control-checkbox",
                ElementSpec::new(ElementRole::Checkbox)
                    .class("control-row")
                    .interactive()
                    .selected(checked),
                |ui| {
                    ui.element(
                        "control-checkbox-mark",
                        ElementSpec::new(ElementRole::Panel)
                            .class("checkbox-mark")
                            .selected(checked),
                        |ui| {
                            if checked {
                                ui.text_element(
                                    "control-checkbox-glyph",
                                    ElementSpec::new(ElementRole::Text).class("check-glyph"),
                                    "x",
                                );
                            }
                        },
                    );
                    ui.text_element(
                        "control-checkbox-label",
                        ElementSpec::new(ElementRole::Text).class("control-label"),
                        "Profile this transform",
                    );
                },
            );
        },
    );
}

fn control_radio_group(ui: &mut des_ui_document::DocumentBuilder, choice: usize) {
    ui.element(
        "control-radio-card",
        ElementSpec::new(ElementRole::Card).class("control-card"),
        |ui| {
            ui.text_element(
                "control-radio-title",
                ElementSpec::new(ElementRole::Text).class("card-title"),
                "Radio group",
            );
            for (index, id, label) in [
                (0, "control-radio-local", "Local runtime"),
                (1, "control-radio-remote", "Remote worker"),
                (2, "control-radio-hybrid", "Hybrid"),
            ] {
                ui.element(
                    id,
                    ElementSpec::new(ElementRole::Radio)
                        .class("control-row")
                        .interactive()
                        .selected(choice == index),
                    |ui| {
                        ui.element(
                            format!("{id}-dot"),
                            ElementSpec::new(ElementRole::Panel)
                                .class("radio-dot")
                                .selected(choice == index),
                            |_| {},
                        );
                        ui.text_element(
                            format!("{id}-label"),
                            ElementSpec::new(ElementRole::Text).class("control-label"),
                            label,
                        );
                    },
                );
            }
        },
    );
}

fn control_dropdown(ui: &mut des_ui_document::DocumentBuilder, open: bool, choice: usize) {
    let selected = ["CSV source", "DuckDB table", "Python node"][choice];
    ui.element(
        "control-dropdown-card",
        ElementSpec::new(ElementRole::Card).class("control-card"),
        |ui| {
            ui.text_element(
                "control-dropdown-title",
                ElementSpec::new(ElementRole::Text).class("card-title"),
                "Dropdown",
            );
            ui.element(
                "control-dropdown",
                ElementSpec::new(ElementRole::Dropdown)
                    .class("dropdown-control")
                    .interactive()
                    .selected(open),
                |ui| {
                    ui.text_element(
                        "control-dropdown-label",
                        ElementSpec::new(ElementRole::Text).class("control-label"),
                        selected,
                    );
                    ui.text_element(
                        "control-dropdown-chevron",
                        ElementSpec::new(ElementRole::Text).class("muted"),
                        if open { "^" } else { "v" },
                    );
                },
            );
            if open {
                ui.element(
                    "control-dropdown-menu",
                    ElementSpec::new(ElementRole::Panel).class("dropdown-menu"),
                    |ui| {
                        for (index, id, label) in [
                            (0, "control-dropdown-option-csv", "CSV source"),
                            (1, "control-dropdown-option-duckdb", "DuckDB table"),
                            (2, "control-dropdown-option-python", "Python node"),
                        ] {
                            ui.element(
                                id,
                                ElementSpec::new(ElementRole::Control)
                                    .class("dropdown-option")
                                    .interactive()
                                    .selected(choice == index),
                                |ui| {
                                    ui.text_element(
                                        format!("{id}-label"),
                                        ElementSpec::new(ElementRole::Text)
                                            .class("control-label")
                                            .selected(choice == index),
                                        label,
                                    );
                                },
                            );
                        }
                    },
                );
            }
        },
    );
}

fn control_text_inputs(ui: &mut des_ui_document::DocumentBuilder) {
    ui.element(
        "control-input-card",
        ElementSpec::new(ElementRole::Card).class("control-card"),
        |ui| {
            ui.text_element(
                "control-input-title",
                ElementSpec::new(ElementRole::Text).class("card-title"),
                "Input fields",
            );
            for (id, label, focused, disabled) in [
                ("control-input-name", "pipeline_name", true, false),
                ("control-input-disabled", "read_only_id", false, true),
            ] {
                ui.element(
                    id,
                    ElementSpec::new(ElementRole::TextInput)
                        .class("input-field")
                        .interactive()
                        .focused(focused)
                        .disabled(disabled),
                    |ui| {
                        ui.text_element(
                            format!("{id}-label"),
                            ElementSpec::new(ElementRole::Text)
                                .class("control-label")
                                .focused(focused)
                                .disabled(disabled),
                            label,
                        );
                    },
                );
            }
        },
    );
}

fn render_styling_view(ui: &mut des_ui_document::DocumentBuilder, dense_mode: bool) {
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

fn render_animation_view(ui: &mut des_ui_document::DocumentBuilder) {
    ui.text_element(
        "animation-heading",
        ElementSpec::new(ElementRole::Text).class("heading"),
        "Animation Specimens",
    );
    ui.text_element(
        "animation-copy",
        ElementSpec::new(ElementRole::Text).class("muted"),
        "Each specimen isolates one state selector and the style properties it animates.",
    );
    ui.element(
        "animation-grid",
        ElementSpec::new(ElementRole::Panel).class("animation-grid"),
        |ui| {
            animation_specimen(
                ui,
                "animation-hover-size",
                "Hovered",
                "width and height animate while the pointer is over the specimen",
                "base: width Px(150); height Px(58) | hovered: width Px(220); height Px(84)",
                "animation-box-hover-size",
                false,
                false,
                false,
            );
            animation_margin_specimen(ui);
            animation_specimen(
                ui,
                "animation-pressed-border",
                "Pressed",
                "border width and corner radius animate while primary pointer is down",
                "base: border width all sides 2; radius all corners 4 | pressed: border width all sides 10; radius all corners 22",
                "animation-box-pressed-border",
                false,
                false,
                false,
            );
            animation_specimen(
                ui,
                "animation-selected-spacing",
                "Selected",
                "size, spacing, color, radius, and font size animate from selected state",
                "base: width 150; height 58; padding 8; margin 0; radius 4 | selected: width 210; height 92; padding 16; margin 10; radius 12",
                "animation-box-selected-spacing",
                true,
                false,
                false,
            );
            animation_specimen(
                ui,
                "animation-disabled-color",
                "Disabled",
                "background, border color, and text color animate from disabled state",
                "base: background; border; text color | disabled: muted background; muted border; muted text color",
                "animation-box-disabled-color",
                false,
                true,
                false,
            );
            animation_specimen(
                ui,
                "animation-focused-min-size",
                "Focused",
                "size, border width, color, and radius animate from focused state",
                "base: width 150; height 58; border width 2; radius 4 | focused: width 226; height 88; border width 6; radius 16",
                "animation-box-focused-min-size",
                false,
                false,
                true,
            );
        },
    );
}

fn animation_margin_specimen(ui: &mut des_ui_document::DocumentBuilder) {
    ui.element(
        "animation-hover-margin",
        ElementSpec::new(ElementRole::Panel).class("animation-specimen"),
        |ui| {
            ui.text_element(
                "animation-hover-margin-title",
                ElementSpec::new(ElementRole::Text).class("box-label"),
                "Hovered Margin",
            );
            ui.text_element(
                "animation-hover-margin-note",
                ElementSpec::new(ElementRole::Text).class("box-note"),
                "margin animates inside the parent and pushes neighboring boxes",
            );
            ui.text_element(
                "animation-hover-margin-rule-0",
                ElementSpec::new(ElementRole::Text).class("box-rule"),
                "base: margin all sides 0",
            );
            ui.text_element(
                "animation-hover-margin-rule-1",
                ElementSpec::new(ElementRole::Text).class("box-rule"),
                "hovered: margin all sides 18",
            );
            ui.element(
                "animation-hover-margin-surface",
                ElementSpec::new(ElementRole::Panel).class("animation-surface"),
                |ui| {
                    ui.element(
                        "animation-hover-margin-row",
                        ElementSpec::new(ElementRole::Panel).class("animation-margin-row"),
                        |ui| {
                            for id in [
                                "animation-hover-margin-before",
                                "animation-hover-margin-target",
                                "animation-hover-margin-after",
                            ] {
                                let spec = ElementSpec::new(ElementRole::Card)
                                    .class("animation-margin-chip")
                                    .class(match id {
                                        "animation-hover-margin-target" => {
                                            "animation-box-hover-margin"
                                        }
                                        _ => "animation-margin-reference",
                                    });

                                let spec = if id == "animation-hover-margin-target" {
                                    spec.interactive()
                                } else {
                                    spec
                                };

                                ui.element(id, spec, |_| {});
                            }
                        },
                    );
                },
            );
        },
    );
}

fn animation_specimen(
    ui: &mut des_ui_document::DocumentBuilder,
    id: &'static str,
    title: &'static str,
    note: &'static str,
    rule_text: &'static str,
    box_class: &'static str,
    selected: bool,
    disabled: bool,
    focused: bool,
) {
    ui.element(
        id,
        ElementSpec::new(ElementRole::Panel).class("animation-specimen"),
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
            for (line_index, line) in rule_text.split(" | ").enumerate() {
                ui.text_element(
                    format!("{id}-rule-{line_index}"),
                    ElementSpec::new(ElementRole::Text).class("box-rule"),
                    line,
                );
            }
            ui.element(
                format!("{id}-surface"),
                ElementSpec::new(ElementRole::Panel).class("animation-surface"),
                |ui| {
                    ui.element(
                        format!("{id}-box"),
                        ElementSpec::new(ElementRole::Card)
                            .class("animation-box")
                            .class(box_class)
                            .interactive()
                            .selected(selected)
                            .disabled(disabled)
                            .focused(focused),
                        |ui| {
                            ui.text_element(
                                format!("{id}-box-label"),
                                ElementSpec::new(ElementRole::Text)
                                    .class("animation-box-label")
                                    .selected(selected)
                                    .disabled(disabled)
                                    .focused(focused),
                                title,
                            );
                            ui.text_element(
                                format!("{id}-box-body"),
                                ElementSpec::new(ElementRole::Text)
                                    .class("animation-box-body")
                                    .selected(selected)
                                    .disabled(disabled)
                                    .focused(focused),
                                "state-driven transition",
                            );
                        },
                    );
                },
            );
        },
    );
}

fn render_scrolling_view(ui: &mut des_ui_document::DocumentBuilder) {
    ui.text_element(
        "scroll-heading",
        ElementSpec::new(ElementRole::Text).class("heading"),
        "Document Scrolling",
    );
    ui.text_element(
        "scroll-copy",
        ElementSpec::new(ElementRole::Text).class("muted"),
        "Use the wheel or touchpad over each panel. Scroll offsets live in des-ui-document.",
    );
    ui.text_element(
        "scroll-direct-title",
        ElementSpec::new(ElementRole::Text).class("section-title"),
        "Direct containers",
    );
    ui.element(
        "scroll-row",
        ElementSpec::new(ElementRole::Panel).class("card-row"),
        |ui| {
            scroll_panel(
                ui,
                "scroll-panel-a",
                "Vertical",
                12,
                "scroll-list",
                "scroll-row-card",
            );
            scroll_panel(
                ui,
                "scroll-panel-b",
                "Horizontal",
                8,
                "scroll-list-horizontal",
                "scroll-wide-row-card",
            );
            scroll_panel(
                ui,
                "scroll-panel-c",
                "Two-axis",
                12,
                "scroll-list-two-axis",
                "scroll-xy-row-card",
            );
        },
    );
    ui.text_element(
        "scroll-nested-title",
        ElementSpec::new(ElementRole::Text).class("section-title"),
        "Nested containers",
    );
    ui.element(
        "scroll-nested-row",
        ElementSpec::new(ElementRole::Panel).class("card-row"),
        |ui| {
            nested_scroll_panel(
                ui,
                "scroll-nested-vertical",
                "Nested vertical",
                12,
                "scroll-list",
                "scroll-row-card",
            );
            nested_scroll_panel(
                ui,
                "scroll-nested-horizontal",
                "Nested horizontal",
                8,
                "scroll-list-horizontal",
                "scroll-wide-row-card",
            );
            nested_scroll_panel(
                ui,
                "scroll-nested-two-axis",
                "Nested two-axis",
                12,
                "scroll-list-two-axis",
                "scroll-xy-row-card",
            );
        },
    );
}

fn render_nesting_view(ui: &mut des_ui_document::DocumentBuilder) {
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

fn render_graph_view(ui: &mut des_ui_document::DocumentBuilder) {
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
                "Next: document-managed canvas bounds with egui/epaint geometry inside.",
            );
        },
    );
}

fn interactive_labeled_row(
    ui: &mut des_ui_document::DocumentBuilder,
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
    ui: &mut des_ui_document::DocumentBuilder,
    id: &'static str,
    title: &'static str,
    row_count: usize,
    list_class: &'static str,
    row_class: &'static str,
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
                ElementSpec::new(ElementRole::Panel)
                    .class(list_class)
                    .class("styled-scrollbar"),
                |ui| {
                    scroll_rows(ui, id, row_count, row_class);
                },
            );
        },
    );
}

fn nested_scroll_panel(
    ui: &mut des_ui_document::DocumentBuilder,
    id: &'static str,
    title: &'static str,
    row_count: usize,
    list_class: &'static str,
    row_class: &'static str,
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
                format!("{id}-shell"),
                ElementSpec::new(ElementRole::Panel).class("scroll-nested-shell"),
                |ui| {
                    ui.element(
                        format!("{id}-list"),
                        ElementSpec::new(ElementRole::Panel)
                            .class(list_class)
                            .class("scroll-list-nested")
                            .class("styled-scrollbar"),
                        |ui| {
                            scroll_rows(ui, id, row_count, row_class);
                        },
                    );
                },
            );
        },
    );
}

fn scroll_rows(
    ui: &mut des_ui_document::DocumentBuilder,
    id: &'static str,
    row_count: usize,
    row_class: &'static str,
) {
    for index in 0..row_count {
        if row_class == "scroll-wide-row-card" {
            scroll_wide_card(ui, id, index);
            continue;
        }

        ui.element(
            format!("{id}-row-{index}"),
            ElementSpec::new(ElementRole::Card)
                .class(row_class)
                .interactive(),
            |ui| {
                ui.text_element(
                    format!("{id}-row-{index}-label"),
                    ElementSpec::new(ElementRole::Text).class("muted"),
                    format!("document-owned scroll row {:02}", index + 1),
                );
            },
        );
    }
}

fn scroll_wide_card(ui: &mut des_ui_document::DocumentBuilder, id: &'static str, index: usize) {
    ui.element(
        format!("{id}-row-{index}"),
        ElementSpec::new(ElementRole::Card)
            .class("scroll-wide-row-card")
            .interactive(),
        |ui| {
            ui.text_element(
                format!("{id}-row-{index}-label"),
                ElementSpec::new(ElementRole::Text).class("muted"),
                format!("horizontal card {:02}", index + 1),
            );
            ui.element(
                format!("{id}-row-{index}-mini-list"),
                ElementSpec::new(ElementRole::Panel)
                    .class("scroll-mini-list")
                    .class("styled-scrollbar"),
                |ui| {
                    for item_index in 0..8 {
                        ui.element(
                            format!("{id}-row-{index}-mini-row-{item_index}"),
                            ElementSpec::new(ElementRole::Card)
                                .class("scroll-mini-row")
                                .interactive(),
                            |ui| {
                                ui.text_element(
                                    format!("{id}-row-{index}-mini-row-{item_index}-label"),
                                    ElementSpec::new(ElementRole::Text).class("muted"),
                                    format!("nested item {:02}", item_index + 1),
                                );
                            },
                        );
                    }
                },
            );
        },
    );
}
