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
                    "box-side-radius",
                    "Side + corner overrides",
                    "CSS-like layered edges",
                    "base: border width all sides 2; radius all corners 4 | override: left border width 8; bottom border width 5 | override: top-right radius 14; bottom-left radius 0",
                    "box-subject-side-radius",
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

fn box_chip(ui: &mut des_ui_document::DocumentBuilder, case_id: &'static str, index: usize) {
    ui.element(
        format!("{case_id}-chip-{index}"),
        ElementSpec::new(ElementRole::Panel).class("box-chip"),
        |_| {},
    );
}

fn render_interaction_view(ui: &mut des_ui_document::DocumentBuilder) {
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

fn render_scrolling_view(ui: &mut des_ui_document::DocumentBuilder) {
    ui.text_element(
        "scroll-heading",
        ElementSpec::new(ElementRole::Text).class("heading"),
        "Document Scrolling",
    );
    ui.text_element(
        "scroll-copy",
        ElementSpec::new(ElementRole::Text).class("muted"),
        "Use the wheel or touchpad over either panel. The scroll offset lives in des-ui-document.",
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
                                    format!("document-owned scroll row {:02}", index + 1),
                                );
                            },
                        );
                    }
                },
            );
        },
    );
}
