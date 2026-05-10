use super::*;
use des_ui_document::Glyph;

pub(super) fn render_topbar(ui: &mut des_ui_document::DocumentBuilder, debug_overlay: bool) {
    ui.element(
        "topbar",
        ElementSpec::new(Element::Div).class("topbar"),
        |ui| {
            ui.text_element(
                "title",
                ElementSpec::new(Element::Text).class("title"),
                "Data Engine Studio UI Lab",
            );
            ui.text_element(
                "subtitle",
                ElementSpec::new(Element::Text).class("muted"),
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
        ElementSpec::new(Element::Div)
            .class("nav")
            .class("styled-scrollbar"),
        |ui| {
            ui.text_element(
                "nav-title",
                ElementSpec::new(Element::Text).class("section-title"),
                "Feature Views",
            );
            for view in [
                LabView::Layout,
                LabView::Interaction,
                LabView::Draggable,
                LabView::Styling,
                LabView::Animation,
                LabView::Scrolling,
                LabView::Floating,
                LabView::Table,
                LabView::Text,
                LabView::Nesting,
                LabView::Graph,
            ] {
                ui.element(
                    view.id(),
                    ElementSpec::new(Element::Div)
                        .class("nav-item")
                        .interactive()
                        .selected(view == selected),
                    |ui| {
                        ui.text_element(
                            format!("{}-label", view.id()),
                            ElementSpec::new(Element::Text).class("card-title"),
                            view.label(),
                        );
                        ui.text_element(
                            format!("{}-hint", view.id()),
                            ElementSpec::new(Element::Text).class("muted"),
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
        LabView::Draggable => "clone layers, handles, sorting",
        LabView::Styling => "elements, classes, states, ids",
        LabView::Animation => "state transitions and easing",
        LabView::Scrolling => "document scroll ownership",
        LabView::Floating => "anchors, fallbacks, arrows",
        LabView::Table => "columns, rows, headers, cells",
        LabView::Text => "wrap, truncate, and measured lines",
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
    drag_item_cells: [usize; 3],
    drag_item_order: [usize; 3],
    scroll_list_item_order: [usize; 14],
    pressed_drag_source: Option<&str>,
    active_drag_item: Option<des_ui_widgets::SortableItemId>,
    active_scroll_list_drag_item: Option<des_ui_widgets::SortableItemId>,
    drag_pointer: Option<des_ui_document::Point>,
    drag_drop_preview: Option<des_ui_widgets::SortableDropPreview>,
    scroll_list_drop_preview: Option<des_ui_widgets::SortableDropPreview>,
    shadow_tune: ShadowTuneState,
    shadow_hover_tune: ShadowTuneState,
) {
    ui.element(
        "stage",
        ElementSpec::new(Element::Div)
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
            LabView::Draggable => render_draggable_view(
                ui,
                drag_item_cells,
                drag_item_order,
                scroll_list_item_order,
                pressed_drag_source,
                active_drag_item,
                active_scroll_list_drag_item,
                drag_pointer,
                drag_drop_preview,
                scroll_list_drop_preview,
            ),
            LabView::Styling => render_styling_view(ui, dense_mode, shadow_tune, shadow_hover_tune),
            LabView::Animation => render_animation_view(ui),
            LabView::Scrolling => render_scrolling_view(ui),
            LabView::Floating => render_floating_view(ui),
            LabView::Table => render_table_view(ui),
            LabView::Text => render_text_view(ui),
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
        ElementSpec::new(Element::Text).class("heading"),
        "Box Model Specimens",
    );
    ui.text_element(
        "layout-copy",
        ElementSpec::new(Element::Text).class("muted"),
        "Each subject isolates one layout contract. Selector rules are printed above the specimen.",
    );
    ui.element(
        "box-model-grid",
        ElementSpec::new(Element::Div).class("box-model-grid"),
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
                    "flex_direction: Row; width: Auto; height: Auto; gap: 10",
                    "box-subject-row-gap",
                );
                box_model_case(
                    ui,
                    "box-column-gap",
                    "Column gap",
                    "3 children",
                    "flex_direction: Column; width: Auto; height: Auto; gap: 6",
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
                    "flex_direction: Row; size: 96 by 54; gap: 8; justify content: Center; align items: End",
                    "box-subject-row-align",
                );
                box_model_case(
                    ui,
                    "box-column-align",
                    "Column alignment",
                    "children spaced on main axis and centered on cross axis",
                    "flex_direction: Column; size: 80 by 92; gap: 4; justify content: SpaceBetween; align items: Center",
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
        ElementSpec::new(Element::Text).class("box-section-label"),
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
        ElementSpec::new(Element::Div).class("box-model-row"),
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
        ElementSpec::new(Element::Div).class("box-model-case"),
        |ui| {
            ui.text_element(
                format!("{id}-title"),
                ElementSpec::new(Element::Text).class("box-label"),
                title,
            );
            ui.text_element(
                format!("{id}-note"),
                ElementSpec::new(Element::Text).class("box-note"),
                note,
            );
            for (line_index, line) in rule_text.split(" | ").enumerate() {
                ui.text_element(
                    format!("{id}-rule-{line_index}"),
                    ElementSpec::new(Element::Text).class("box-rule"),
                    line,
                );
            }
            ui.element(
                format!("{id}-frame"),
                ElementSpec::new(Element::Div).class("box-subject-frame"),
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
    let mut spec = ElementSpec::new(Element::Div)
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
                    ElementSpec::new(Element::Div).class("box-max-wide-child"),
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
                    ElementSpec::new(Element::Div).class("box-overflow-child"),
                    |_| {},
                );
            }
            "box-subject-nested-nine" => {
                ui.element(
                    format!("{case_id}-outer"),
                    ElementSpec::new(Element::Div).class("box-nested-outer"),
                    |ui| {
                        ui.element(
                            format!("{case_id}-inner"),
                            ElementSpec::new(Element::Div).class("box-nested-inner"),
                            |ui| {
                                for row in 0..3 {
                                    ui.element(
                                        format!("{case_id}-row-{row}"),
                                        ElementSpec::new(Element::Div).class("box-nested-row"),
                                        |ui| {
                                            for column in 0..3 {
                                                ui.element(
                                                    format!("{case_id}-cell-{row}-{column}"),
                                                    ElementSpec::new(Element::Div)
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
                    ElementSpec::new(Element::Div).class("box-inset-percent-parent"),
                    |ui| {
                        ui.element(
                            format!("{case_id}-child"),
                            ElementSpec::new(Element::Div).class("box-inset-percent-child"),
                            |_| {},
                        );
                    },
                );
            }
            "box-subject-absolute-parent" => {
                ui.element(
                    format!("{case_id}-parent"),
                    ElementSpec::new(Element::Div).class("box-absolute-parent-frame"),
                    |ui| {
                        ui.element(
                            format!("{case_id}-flow-child"),
                            ElementSpec::new(Element::Div).class("box-absolute-flow-child"),
                            |_| {},
                        );
                        ui.element(
                            format!("{case_id}-child"),
                            ElementSpec::new(Element::Div).class("box-absolute-parent-child"),
                            |_| {},
                        );
                    },
                );
            }
            "box-subject-absolute-window" => {
                ui.element(
                    format!("{case_id}-host"),
                    ElementSpec::new(Element::Div).class("box-absolute-window-host"),
                    |ui| {
                        ui.element(
                            format!("{case_id}-child"),
                            ElementSpec::new(Element::Div).class("box-absolute-window-child"),
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
        ElementSpec::new(Element::Div).class("box-chip"),
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
        ElementSpec::new(Element::Text).class("heading"),
        "Document Interaction",
    );
    ui.text_element(
        "interaction-copy",
        ElementSpec::new(Element::Text).class("muted"),
        "Hover and click styles are resolved by document state. Inner text does not own clicks.",
    );
    ui.element(
        "interaction-row",
        ElementSpec::new(Element::Div).class("card-row"),
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
                    ElementSpec::new(Element::Div)
                        .class("feature-card")
                        .interactive(),
                    |ui| {
                        ui.text_element(
                            format!("{id}-title"),
                            ElementSpec::new(Element::Text).class("card-title"),
                            title,
                        );
                        ui.text_element(
                            format!("{id}-body"),
                            ElementSpec::new(Element::Text).class("muted"),
                            body,
                        );
                    },
                );
            }
        },
    );
    ui.text_element(
        "controls-title",
        ElementSpec::new(Element::Text).class("section-title"),
        "Control elements",
    );
    ui.element(
        "controls-grid",
        ElementSpec::new(Element::Div).class("controls-grid"),
        |ui| {
            control_checkbox(ui, checkbox_enabled);
            control_radio_group(ui, radio_choice);
            control_dropdown(ui, dropdown_open, dropdown_choice);
            control_text_inputs(ui);
        },
    );
    render_document_update_loop(ui);
}

fn render_draggable_view(
    ui: &mut des_ui_document::DocumentBuilder,
    drag_item_cells: [usize; 3],
    drag_item_order: [usize; 3],
    scroll_list_item_order: [usize; 14],
    pressed_drag_source: Option<&str>,
    active_drag_item: Option<des_ui_widgets::SortableItemId>,
    active_scroll_list_drag_item: Option<des_ui_widgets::SortableItemId>,
    drag_pointer: Option<des_ui_document::Point>,
    drag_drop_preview: Option<des_ui_widgets::SortableDropPreview>,
    scroll_list_drop_preview: Option<des_ui_widgets::SortableDropPreview>,
) {
    ui.child("draggable-heading", Element::Text)
        .class("heading")
        .text("Document Draggables");
    ui.child("draggable-copy", Element::Text)
        .class("muted")
        .text("Sortable drag/drop uses document events, visual subtree clones, optional handles, and style-owned overlays.");
    render_drag_drop_lab(
        ui,
        drag_item_cells,
        drag_item_order,
        scroll_list_item_order,
        pressed_drag_source,
        active_drag_item,
        active_scroll_list_drag_item,
        drag_pointer,
        drag_drop_preview,
        scroll_list_drop_preview,
    );
}

fn render_document_update_loop(ui: &mut des_ui_document::DocumentBuilder) {
    ui.text_element(
        "loop-title",
        ElementSpec::new(Element::Text).class("section-title"),
        "Event to document update loop",
    );
    ui.element(
        "loop-grid",
        ElementSpec::new(Element::Div).class("loop-grid"),
        |ui| {
            ui.element(
                "loop-action-card",
                ElementSpec::new(Element::Div).class("loop-control-card"),
                |ui| {
                    ui.text_element(
                        "loop-action-title",
                        ElementSpec::new(Element::Text).class("card-title"),
                        "Button event",
                    );
                    ui.element(
                        "loop-action-button",
                        ElementSpec::new(Element::Button)
                            .class("loop-button")
                            .interactive(),
                        |ui| {
                            ui.text_element(
                                "loop-action-button-label",
                                ElementSpec::new(Element::Text).class("control-label"),
                                "Send update",
                            );
                        },
                    );
                },
            );
            loop_result_card(
                ui,
                "loop-button-result-box",
                "loop-button-result",
                "Button target",
                "waiting for button event",
            );
            loop_result_card(
                ui,
                "loop-checkbox-result-box",
                "loop-checkbox-result",
                "Checkbox target",
                "profiling state pending",
            );
            loop_result_card(
                ui,
                "loop-radio-result-box",
                "loop-radio-result",
                "Radio target",
                "runtime state pending",
            );
            loop_result_card(
                ui,
                "loop-dropdown-result-box",
                "loop-dropdown-result",
                "Dropdown target",
                "source adapter pending",
            );
            loop_result_card(
                ui,
                "loop-summary-result-box",
                "loop-summary-result",
                "Combined target",
                "waiting for updates",
            );
        },
    );
}

fn loop_result_card(
    ui: &mut des_ui_document::DocumentBuilder,
    box_id: &'static str,
    text_id: &'static str,
    title: &'static str,
    fallback: &'static str,
) {
    ui.element(
        box_id,
        ElementSpec::new(Element::Div).class("loop-result-card"),
        |ui| {
            ui.text_element(
                format!("{text_id}-title"),
                ElementSpec::new(Element::Text).class("muted"),
                title,
            );
            ui.text_element(
                text_id,
                ElementSpec::new(Element::Text).class("control-label"),
                fallback,
            );
        },
    );
}

fn render_drag_drop_lab(
    ui: &mut des_ui_document::DocumentBuilder,
    drag_item_cells: [usize; 3],
    drag_item_order: [usize; 3],
    scroll_list_item_order: [usize; 14],
    pressed_drag_source: Option<&str>,
    active_drag_item: Option<des_ui_widgets::SortableItemId>,
    active_scroll_list_drag_item: Option<des_ui_widgets::SortableItemId>,
    _drag_pointer: Option<des_ui_document::Point>,
    drag_drop_preview: Option<des_ui_widgets::SortableDropPreview>,
    scroll_list_drop_preview: Option<des_ui_widgets::SortableDropPreview>,
) {
    ui.child("drag-title", Element::Text)
        .class("section-title")
        .text("Drag and drop grid");
    ui.div("drag-workbench")
        .class("drag-workbench")
        .children(|ui| {
            render_elevated_scrollable_drag_list(
                ui,
                "Scrollable list target",
                scroll_list_item_order,
                active_scroll_list_drag_item,
                scroll_list_drop_preview,
                pressed_drag_source,
            );
            ui.div("drag-grid").class("drag-grid").children(|ui| {
                for cell in 0..6 {
                    let column = if cell % 2 == 0 { "Left" } else { "Right" };
                    let row = cell / 2 + 1;
                    ui.div(format!("drag-cell-{cell}"))
                        .class("drag-cell")
                        .children(|ui| {
                            ui.child(format!("drag-cell-{cell}-label"), Element::Text)
                                .class("muted")
                                .text(format!("{column} row {row}"));
                            let mut cell_items: Vec<_> = drag_item_cells
                                .iter()
                                .enumerate()
                                .filter_map(|(item, item_cell)| {
                                    (*item_cell == cell).then_some(item)
                                })
                                .collect();
                            cell_items.sort_by_key(|item| drag_item_order[*item]);
                            for item in cell_items {
                                if active_drag_item == Some(des_ui_widgets::SortableItemId(item)) {
                                    drag_item(
                                        ui,
                                        item,
                                        drag_drop_preview,
                                        true,
                                        pressed_drag_source,
                                    );
                                } else {
                                    drag_item(
                                        ui,
                                        item,
                                        drag_drop_preview,
                                        false,
                                        pressed_drag_source,
                                    );
                                }
                            }
                        });
                }
            });
        });
}

fn render_elevated_scrollable_drag_list(
    ui: &mut des_ui_document::DocumentBuilder,
    title: &'static str,
    scroll_list_item_order: [usize; 14],
    active_scroll_list_drag_item: Option<des_ui_widgets::SortableItemId>,
    scroll_list_drop_preview: Option<des_ui_widgets::SortableDropPreview>,
    pressed_drag_source: Option<&str>,
) {
    ui.div("drag-scroll-list-card")
        .class("drag-scroll-list-card")
        .children(|ui| {
            ui.child("drag-scroll-list-title", Element::Text)
                .class("section-subtitle")
                .text(title);
            ui.div("drag-scroll-list-0")
                .class("drag-scroll-list")
                .children(|ui| {
                    let mut list_items: Vec<_> = (0..scroll_list_item_order.len()).collect();
                    list_items.sort_by_key(|item| scroll_list_item_order[*item]);
                    for item in list_items {
                        drag_scroll_item(
                            ui,
                            item,
                            scroll_list_drop_preview,
                            active_scroll_list_drag_item
                                == Some(des_ui_widgets::SortableItemId(item)),
                            pressed_drag_source,
                        );
                    }
                });
        });
}

pub(super) fn render_drag_overlay_layer(
    ui: &mut des_ui_document::DocumentBuilder,
    drag_pointer: Option<des_ui_document::Point>,
    drag_visual_clone: Option<&des_ui_document::VisualElementClone>,
) {
    if drag_pointer.is_none() {
        drag_overlay_placeholder(ui);
        return;
    }
    if let Some(clone) = drag_visual_clone {
        drag_visual_overlay(ui, clone);
    } else {
        drag_overlay_placeholder(ui);
    }
}

pub(super) fn render_debug_overlay_layer(
    ui: &mut des_ui_document::DocumentBuilder,
    perf: UiLabPerf,
) {
    ui.element(
        "debug-overlay",
        ElementSpec::new(Element::Div).class("debug-overlay"),
        |ui| {
            ui.text_element(
                "debug-overlay-title",
                ElementSpec::new(Element::Text).class("debug-overlay-title"),
                "UI Lab Runtime",
            );
            debug_metric_row(
                ui,
                "debug-document-time",
                "document",
                format_duration(perf.document_time),
            );
            debug_metric_row(
                ui,
                "debug-engine-time",
                "engine",
                format_duration(perf.engine_time),
            );
            debug_metric_row(
                ui,
                "debug-paint-time",
                "paint",
                format_duration(perf.paint_time),
            );
            debug_metric_row(
                ui,
                "debug-elements",
                "elements",
                perf.metrics.element_count.to_string(),
            );
            debug_metric_row(
                ui,
                "debug-scrollbars",
                "scrollbars",
                perf.metrics.scroll_chrome_count.to_string(),
            );
            debug_metric_row(
                ui,
                "debug-input-cache",
                "input cache hit",
                perf.metrics.reused_cached_layout.to_string(),
            );
            debug_metric_row(
                ui,
                "debug-final-layout",
                "final relayout skipped",
                perf.metrics.reused_input_layout.to_string(),
            );
            debug_metric_row(
                ui,
                "debug-input-changed",
                "input changed",
                perf.metrics.input_changed_state.to_string(),
            );
            debug_metric_row(
                ui,
                "debug-style-changed",
                "style changed",
                perf.metrics.animation_changed_style.to_string(),
            );
            debug_metric_row(
                ui,
                "debug-layout-changed",
                "layout changed",
                perf.metrics.animation_changed_layout.to_string(),
            );
            debug_metric_row(
                ui,
                "debug-paint-changed",
                "paint changed",
                perf.metrics.animation_changed_paint.to_string(),
            );
        },
    );
}

fn debug_metric_row(
    ui: &mut des_ui_document::DocumentBuilder,
    id: &'static str,
    label: &'static str,
    value: String,
) {
    ui.element(
        id,
        ElementSpec::new(Element::Div).class("debug-row"),
        |ui| {
            ui.text_element(
                format!("{id}-label"),
                ElementSpec::new(Element::Text).class("debug-label"),
                label,
            );
            ui.text_element(
                format!("{id}-value"),
                ElementSpec::new(Element::Text).class("debug-value"),
                value,
            );
        },
    );
}

fn format_duration(duration: std::time::Duration) -> String {
    format!("{:.2} ms", duration.as_secs_f64() * 1000.0)
}

fn drag_scroll_item(
    ui: &mut des_ui_document::DocumentBuilder,
    item: usize,
    drag_drop_preview: Option<des_ui_widgets::SortableDropPreview>,
    origin_space: bool,
    pressed_drag_source: Option<&str>,
) {
    let label = format!("auto-scroll row {:02}", item + 1);
    let mut item_builder = ui
        .div(format!("drag-scroll-item-{item}"))
        .class("drag-item")
        .class("drag-scroll-item")
        .value(label.clone());
    if pressed_drag_source == Some(format!("drag-scroll-item-{item}").as_str()) {
        item_builder = item_builder.class("drag-handle-pressed");
    }
    if origin_space {
        item_builder = item_builder.class("drag-origin-space");
        if drag_drop_preview.is_some() {
            item_builder = item_builder.class("drag-origin-collapsed");
        }
    }
    if let Some(preview) = drag_drop_preview
        && preview.nearest_item == Some(des_ui_widgets::SortableItemId(item))
    {
        item_builder = item_builder.class(match preview.edge {
            des_ui_widgets::DropEdge::Before => "drag-gap-before",
            des_ui_widgets::DropEdge::After => "drag-gap-after",
        });
    }
    item_builder.children(|ui| {
        let mut label_builder = ui
            .child(format!("drag-scroll-item-{item}-label"), Element::Text)
            .class("control-label");
        if origin_space {
            label_builder = label_builder.class("drag-origin-content");
        }
        label_builder.text(label);
        drag_scroll_handle(ui, item, origin_space);
    });
}

fn drag_scroll_handle(ui: &mut des_ui_document::DocumentBuilder, item: usize, origin_space: bool) {
    let mut handle_builder = ui
        .button(format!("drag-scroll-handle-{item}"))
        .class("drag-handle")
        .class("drag-scroll-handle")
        .interactive()
        .value(format!("drag-scroll-item-{item}"));
    if origin_space {
        handle_builder = handle_builder.class("drag-origin-content");
    }
    handle_builder.children(|ui| {
        let mut glyph_builder = ui
            .icon(format!("drag-scroll-handle-{item}-glyph"))
            .class("drag-handle-glyph");
        if origin_space {
            glyph_builder = glyph_builder.class("drag-origin-content");
        }
        glyph_builder.glyph(Glyph::DragHandle).empty();
    });
}

fn drag_item(
    ui: &mut des_ui_document::DocumentBuilder,
    item: usize,
    drag_drop_preview: Option<des_ui_widgets::SortableDropPreview>,
    origin_space: bool,
    pressed_drag_source: Option<&str>,
) {
    let label = ["Customers", "Orders", "Rates"][item];
    let mut item_builder = ui
        .div(format!("drag-item-{item}"))
        .class("drag-item")
        .value(label);
    if pressed_drag_source == Some(format!("drag-item-{item}").as_str()) {
        item_builder = item_builder.class("drag-handle-pressed");
    }
    if origin_space {
        item_builder = item_builder.class("drag-origin-space");
        if drag_drop_preview.is_some() {
            item_builder = item_builder.class("drag-origin-collapsed");
        }
    }
    if let Some(preview) = drag_drop_preview
        && preview.nearest_item == Some(des_ui_widgets::SortableItemId(item))
    {
        item_builder = item_builder.class(match preview.edge {
            des_ui_widgets::DropEdge::Before => "drag-gap-before",
            des_ui_widgets::DropEdge::After => "drag-gap-after",
        });
    }
    item_builder.children(|ui| {
        let mut label_builder = ui
            .child(format!("drag-item-{item}-label"), Element::Text)
            .class("control-label");
        if origin_space {
            label_builder = label_builder.class("drag-origin-content");
        }
        label_builder.text(label);
        drag_handle(ui, item, origin_space);
    });
}

fn drag_handle(ui: &mut des_ui_document::DocumentBuilder, item: usize, origin_space: bool) {
    let mut handle_builder = ui
        .button(format!("drag-handle-{item}"))
        .class("drag-handle")
        .interactive()
        .value(format!("drag-item-{item}"));
    if origin_space {
        handle_builder = handle_builder.class("drag-origin-content");
    }
    handle_builder.children(|ui| {
        let mut glyph_builder = ui
            .icon(format!("drag-handle-{item}-glyph"))
            .class("drag-handle-glyph");
        if origin_space {
            glyph_builder = glyph_builder.class("drag-origin-content");
        }
        glyph_builder.glyph(Glyph::DragHandle).empty();
    });
}

fn drag_visual_overlay(
    ui: &mut des_ui_document::DocumentBuilder,
    clone: &des_ui_document::VisualElementClone,
) {
    ui.visual_clone(
        clone,
        des_ui_document::VisualCloneOptions::new("drag-overlay", "drag-overlay/")
            .root_class("drag-overlay"),
    );
}

fn drag_overlay_placeholder(ui: &mut des_ui_document::DocumentBuilder) {
    ui.div("drag-overlay")
        .class("drag-overlay")
        .class("drag-overlay-idle")
        .value("")
        .empty();
}

fn control_checkbox(ui: &mut des_ui_document::DocumentBuilder, checked: bool) {
    ui.element(
        "control-checkbox-card",
        ElementSpec::new(Element::Div).class("control-card"),
        |ui| {
            ui.text_element(
                "control-checkbox-title",
                ElementSpec::new(Element::Text).class("card-title"),
                "Checkbox",
            );
            ui.element(
                "control-checkbox",
                ElementSpec::new(Element::Checkbox)
                    .class("control-row")
                    .interactive()
                    .selected(checked),
                |ui| {
                    ui.element(
                        "control-checkbox-mark",
                        ElementSpec::new(Element::Div)
                            .class("checkbox-mark")
                            .selected(checked),
                        |ui| {
                            if checked {
                                ui.element(
                                    "control-checkbox-glyph",
                                    ElementSpec::new(Element::Icon)
                                        .class("check-glyph")
                                        .glyph(Glyph::Check),
                                    |_| {},
                                );
                            }
                        },
                    );
                    ui.text_element(
                        "control-checkbox-label",
                        ElementSpec::new(Element::Text).class("control-label"),
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
        ElementSpec::new(Element::Div).class("control-card"),
        |ui| {
            ui.text_element(
                "control-radio-title",
                ElementSpec::new(Element::Text).class("card-title"),
                "Radio group",
            );
            for (index, id, label) in [
                (0, "control-radio-local", "Local runtime"),
                (1, "control-radio-remote", "Remote worker"),
                (2, "control-radio-hybrid", "Hybrid"),
            ] {
                ui.element(
                    id,
                    ElementSpec::new(Element::Radio)
                        .class("control-row")
                        .interactive()
                        .selected(choice == index),
                    |ui| {
                        ui.element(
                            format!("{id}-dot"),
                            ElementSpec::new(Element::Div)
                                .class("radio-dot")
                                .selected(choice == index),
                            |ui| {
                                if choice == index {
                                    ui.element(
                                        format!("{id}-dot-fill"),
                                        ElementSpec::new(Element::Div).class("radio-dot-fill"),
                                        |_| {},
                                    );
                                }
                            },
                        );
                        ui.text_element(
                            format!("{id}-label"),
                            ElementSpec::new(Element::Text).class("control-label"),
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
        ElementSpec::new(Element::Div).class("control-card"),
        |ui| {
            ui.text_element(
                "control-dropdown-title",
                ElementSpec::new(Element::Text).class("card-title"),
                "Dropdown",
            );
            ui.element(
                "control-dropdown",
                ElementSpec::new(Element::Div)
                    .class("dropdown-field")
                    .interactive(),
                |ui| {
                    ui.element(
                        "control-dropdown-trigger",
                        ElementSpec::new(Element::Select)
                            .class("dropdown-control")
                            .selected(open),
                        |ui| {
                            ui.text_element(
                                "control-dropdown-label",
                                ElementSpec::new(Element::Text).class("control-label"),
                                selected,
                            );
                            ui.element(
                                "control-dropdown-chevron",
                                ElementSpec::new(Element::Icon)
                                    .class("dropdown-chevron")
                                    .glyph(if open {
                                        Glyph::ChevronUp
                                    } else {
                                        Glyph::ChevronDown
                                    }),
                                |_| {},
                            );
                        },
                    );
                    if open {
                        ui.element(
                            "control-dropdown-menu",
                            ElementSpec::new(Element::Div).class("dropdown-menu"),
                            |ui| {
                                for (index, id, label) in [
                                    (0, "control-dropdown-option-csv", "CSV source"),
                                    (1, "control-dropdown-option-duckdb", "DuckDB table"),
                                    (2, "control-dropdown-option-python", "Python node"),
                                ] {
                                    ui.element(
                                        id,
                                        ElementSpec::new(Element::Button)
                                            .class("dropdown-option")
                                            .interactive()
                                            .selected(choice == index),
                                        |ui| {
                                            ui.text_element(
                                                format!("{id}-label"),
                                                ElementSpec::new(Element::Text)
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
        },
    );
}

fn control_text_inputs(ui: &mut des_ui_document::DocumentBuilder) {
    ui.element(
        "control-input-card",
        ElementSpec::new(Element::Div).class("control-card"),
        |ui| {
            ui.text_element(
                "control-input-title",
                ElementSpec::new(Element::Text).class("card-title"),
                "Input fields",
            );
            for (id, label, focused, disabled) in [
                ("control-input-name", "pipeline_name", true, false),
                ("control-input-disabled", "read_only_id", false, true),
            ] {
                ui.element(
                    id,
                    ElementSpec::new(Element::Input)
                        .class("input-field")
                        .interactive()
                        .focused(focused)
                        .disabled(disabled),
                    |ui| {
                        ui.text_element(
                            format!("{id}-label"),
                            ElementSpec::new(Element::Text)
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

fn render_styling_view(
    ui: &mut des_ui_document::DocumentBuilder,
    dense_mode: bool,
    shadow_tune: ShadowTuneState,
    shadow_hover_tune: ShadowTuneState,
) {
    ui.text_element(
        "styling-heading",
        ElementSpec::new(Element::Text).class("heading"),
        "Deterministic Styling",
    );
    ui.text_element(
        "styling-copy",
        ElementSpec::new(Element::Text).class("muted"),
        "Style order is element, class, state, id. No CSS specificity maze.",
    );
    ui.element(
        "style-stack",
        ElementSpec::new(Element::Div).class("stack"),
        |ui| {
            interactive_labeled_row(
                ui,
                "style-row-element",
                "Element",
                "Element::Div stays structural; classes define surfaces.",
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
    render_shadow_specimens(ui);
    render_shadow_tuner(ui, shadow_tune, shadow_hover_tune);
    render_structural_selector_specimens(ui);
}

fn render_shadow_specimens(ui: &mut des_ui_document::DocumentBuilder) {
    ui.text_element(
        "shadow-specimen-title",
        ElementSpec::new(Element::Text).class("section-title"),
        "Shadow Styling",
    );
    ui.text_element(
        "shadow-specimen-copy",
        ElementSpec::new(Element::Text).class("muted"),
        "Single soft shadows are paint-only; spread can contract or expand the source shape.",
    );
    ui.element(
        "shadow-specimen-grid",
        ElementSpec::new(Element::Div).class("shadow-grid"),
        |ui| {
            shadow_item(
                ui,
                "shadow-single",
                "Elevation level 2",
                "single soft layer",
            );
            shadow_item(
                ui,
                "shadow-layered",
                "Elevation level 3",
                "menu/card emphasis",
            );
            shadow_item(
                ui,
                "shadow-negative-spread",
                "Elevation level 5",
                "dragged surface",
            );
        },
    );
    ui.element(
        "shadow-light-stage",
        ElementSpec::new(Element::Div).class("shadow-light-stage"),
        |ui| {
            light_shadow_card(ui, "shadow-light-top", "48", true);
            light_shadow_card(ui, "shadow-light-bottom", "30", false);
        },
    );
    ui.element(
        "shadow-web-stage",
        ElementSpec::new(Element::Div).class("shadow-web-stage"),
        |ui| {
            web_shadow_card(ui, "shadow-web-top", "46", true);
            web_shadow_card(ui, "shadow-web-bottom", "48", false);
        },
    );
}

fn render_shadow_tuner(
    ui: &mut des_ui_document::DocumentBuilder,
    shadow_tune: ShadowTuneState,
    shadow_hover_tune: ShadowTuneState,
) {
    ui.text_element(
        "shadow-tune-title",
        ElementSpec::new(Element::Text).class("section-title"),
        "Shadow Tuner",
    );
    ui.text_element(
        "shadow-tune-copy",
        ElementSpec::new(Element::Text).class("muted"),
        "Tune base and hover shadows by eye, then copy the numbers into the elevation recipe.",
    );
    ui.element(
        "shadow-tune-panel",
        ElementSpec::new(Element::Div).class("shadow-tune-panel"),
        |ui| {
            ui.element(
                "shadow-tune-preview",
                ElementSpec::new(Element::Div).class("shadow-tune-preview"),
                |ui| {
                    shadow_tune_preview_card(ui, "shadow-tune-preview-card-1", "base row 01");
                    shadow_tune_preview_card(ui, "shadow-tune-preview-card-2", "hover row 02");
                    shadow_tune_preview_card(ui, "shadow-tune-preview-card-3", "hover row 03");
                },
            );
            ui.element(
                "shadow-tune-controls",
                ElementSpec::new(Element::Div).class("shadow-tune-controls"),
                |ui| {
                    shadow_tune_group(ui, ShadowTuneTarget::Base, shadow_tune);
                    shadow_tune_group(ui, ShadowTuneTarget::Hover, shadow_hover_tune);
                    ui.text_element(
                        "shadow-tune-output",
                        ElementSpec::new(Element::Text)
                            .class("shadow-tune-output")
                            .selectable_text(),
                        shadow_tune_output(shadow_tune, shadow_hover_tune),
                    );
                },
            );
        },
    );
}

fn shadow_tune_preview_card(
    ui: &mut des_ui_document::DocumentBuilder,
    id: &'static str,
    label: &'static str,
) {
    ui.element(
        id,
        ElementSpec::new(Element::Div)
            .class("shadow-tune-preview-card")
            .interactive(),
        |ui| {
            ui.text_element(
                format!("{id}-label"),
                ElementSpec::new(Element::Text).class("shadow-web-label"),
                label,
            );
            ui.element(
                format!("{id}-handle"),
                ElementSpec::new(Element::Icon)
                    .class("shadow-web-handle")
                    .glyph(Glyph::DragHandle),
                |_| {},
            );
        },
    );
}

fn shadow_tune_group(
    ui: &mut des_ui_document::DocumentBuilder,
    target: ShadowTuneTarget,
    state: ShadowTuneState,
) {
    ui.element(
        format!("shadow-tune-{}-group", target.id_prefix()),
        ElementSpec::new(Element::Div).class("shadow-tune-group"),
        |ui| {
            ui.text_element(
                format!("shadow-tune-{}-group-title", target.id_prefix()),
                ElementSpec::new(Element::Text).class("section-title"),
                format!("{} shadow", target.label()),
            );
            shadow_tune_layer(ui, target, 0, state.layers[0]);
            shadow_tune_layer(ui, target, 1, state.layers[1]);
        },
    );
}

fn shadow_tune_layer(
    ui: &mut des_ui_document::DocumentBuilder,
    target: ShadowTuneTarget,
    layer_index: usize,
    layer: ShadowTuneLayer,
) {
    let target_id = target.id_prefix();
    let layer_id = format!("shadow-tune-{target_id}-layer-{layer_index}");
    ui.element(
        layer_id,
        ElementSpec::new(Element::Div).class("shadow-tune-layer"),
        |ui| {
            ui.element(
                format!("shadow-tune-{target_id}-layer-{layer_index}-header"),
                ElementSpec::new(Element::Div).class("shadow-tune-header"),
                |ui| {
                    ui.text_element(
                        format!("shadow-tune-{target_id}-layer-{layer_index}-title"),
                        ElementSpec::new(Element::Text).class("card-title"),
                        format!(
                            "Layer {} ({})",
                            layer_index + 1,
                            if layer.enabled { "on" } else { "off" }
                        ),
                    );
                    ui.element(
                        format!("shadow-tune-{target_id}-layer-{layer_index}-toggle"),
                        ElementSpec::new(Element::Button)
                            .class("shadow-tune-toggle")
                            .interactive(),
                        |ui| {
                            ui.text_element(
                                format!("shadow-tune-{target_id}-layer-{layer_index}-toggle-label"),
                                ElementSpec::new(Element::Text).class("button-label"),
                                if layer.enabled { "Disable" } else { "Enable" },
                            );
                        },
                    );
                },
            );
            shadow_tune_control(ui, target, layer_index, "x", "x", format!("{:.0}", layer.x));
            shadow_tune_control(ui, target, layer_index, "y", "y", format!("{:.0}", layer.y));
            shadow_tune_control(
                ui,
                target,
                layer_index,
                "blur",
                "blur",
                format!("{:.0}", layer.blur),
            );
            shadow_tune_control(
                ui,
                target,
                layer_index,
                "spread",
                "spread",
                format!("{:.0}", layer.spread),
            );
            shadow_tune_control(
                ui,
                target,
                layer_index,
                "alpha",
                "alpha",
                layer.alpha.to_string(),
            );
        },
    );
}

fn shadow_tune_control(
    ui: &mut des_ui_document::DocumentBuilder,
    target: ShadowTuneTarget,
    layer_index: usize,
    field: &'static str,
    label: &'static str,
    value: String,
) {
    let target_id = target.id_prefix();
    let row_id = format!("shadow-tune-{target_id}-l{layer_index}-{field}-row");
    ui.element(
        row_id,
        ElementSpec::new(Element::Div).class("shadow-tune-row"),
        |ui| {
            ui.text_element(
                format!("shadow-tune-{target_id}-l{layer_index}-{field}-label"),
                ElementSpec::new(Element::Text).class("shadow-tune-label"),
                label,
            );
            shadow_tune_button(ui, target, layer_index, field, "dec", "-");
            ui.text_element(
                format!("shadow-tune-{target_id}-l{layer_index}-{field}-value"),
                ElementSpec::new(Element::Text).class("shadow-tune-value"),
                value,
            );
            shadow_tune_button(ui, target, layer_index, field, "inc", "+");
        },
    );
}

fn shadow_tune_button(
    ui: &mut des_ui_document::DocumentBuilder,
    target: ShadowTuneTarget,
    layer_index: usize,
    field: &'static str,
    direction: &'static str,
    label: &'static str,
) {
    let button_id = format!(
        "shadow-tune-{}-l{layer_index}-{field}-{direction}",
        target.id_prefix()
    );
    ui.element(
        button_id.clone(),
        ElementSpec::new(Element::Button)
            .class("shadow-tune-button")
            .interactive(),
        |ui| {
            ui.text_element(
                format!("{button_id}-label"),
                ElementSpec::new(Element::Text).class("button-label"),
                label,
            );
        },
    );
}

fn shadow_tune_output(base: ShadowTuneState, hover: ShadowTuneState) -> String {
    let layer = |state: ShadowTuneState, index: usize| {
        let layer = state.layers[index];
        format!(
            "{}: x {:.0}, y {:.0}, blur {:.0}, spread {:.0}, alpha {}",
            if layer.enabled { "on" } else { "off" },
            layer.x,
            layer.y,
            layer.blur,
            layer.spread,
            layer.alpha
        )
    };
    format!(
        "base L1 {}; base L2 {}; hover L1 {}; hover L2 {}",
        layer(base, 0),
        layer(base, 1),
        layer(hover, 0),
        layer(hover, 1)
    )
}

fn shadow_item(
    ui: &mut des_ui_document::DocumentBuilder,
    id: &'static str,
    label: &'static str,
    body: &'static str,
) {
    ui.element(
        id,
        ElementSpec::new(Element::Div)
            .class("shadow-card")
            .class(id),
        |ui| {
            ui.text_element(
                format!("{id}-label"),
                ElementSpec::new(Element::Text).class("card-title"),
                label,
            );
            ui.text_element(
                format!("{id}-body"),
                ElementSpec::new(Element::Text).class("muted"),
                body,
            );
        },
    );
}

fn light_shadow_card(
    ui: &mut des_ui_document::DocumentBuilder,
    id: &'static str,
    label: &'static str,
    raised: bool,
) {
    let card_spec = if raised {
        ElementSpec::new(Element::Div)
            .class("shadow-light-card")
            .class("shadow-light-card-raised")
    } else {
        ElementSpec::new(Element::Div).class("shadow-light-card")
    };
    ui.element(id, card_spec, |ui| {
        ui.text_element(
            format!("{id}-label"),
            ElementSpec::new(Element::Text).class("shadow-light-label"),
            label,
        );
        ui.element(
            format!("{id}-handle"),
            ElementSpec::new(Element::Icon)
                .class("shadow-light-handle")
                .glyph(Glyph::DragHandle),
            |_| {},
        );
    });
}

fn web_shadow_card(
    ui: &mut des_ui_document::DocumentBuilder,
    id: &'static str,
    label: &'static str,
    raised: bool,
) {
    let card_spec = if raised {
        ElementSpec::new(Element::Div)
            .class("shadow-web-card")
            .class("shadow-web-card-raised")
    } else {
        ElementSpec::new(Element::Div).class("shadow-web-card")
    };
    ui.element(id, card_spec, |ui| {
        ui.text_element(
            format!("{id}-label"),
            ElementSpec::new(Element::Text).class("shadow-web-label"),
            label,
        );
        ui.element(
            format!("{id}-handle"),
            ElementSpec::new(Element::Icon)
                .class("shadow-web-handle")
                .glyph(Glyph::DragHandle),
            |_| {},
        );
    });
}

fn render_structural_selector_specimens(ui: &mut des_ui_document::DocumentBuilder) {
    ui.text_element(
        "structural-selector-title",
        ElementSpec::new(Element::Text).class("section-title"),
        "Structural Selectors",
    );
    ui.text_element(
        "structural-selector-copy",
        ElementSpec::new(Element::Text).class("muted"),
        "first-child, last-child, and nth-child are resolved from document nesting.",
    );
    ui.element(
        "structural-selector-grid",
        ElementSpec::new(Element::Div).class("structural-grid"),
        |ui| {
            ui.element(
                "structural-main-list",
                ElementSpec::new(Element::Div).class("structural-list"),
                |ui| {
                    structural_item(
                        ui,
                        "structural-main-one",
                        "first-child",
                        ".structural-item:first-child -> green surface",
                    );
                    structural_item(
                        ui,
                        "structural-main-two",
                        "nth-child(2)",
                        ".structural-item:nth-child(2) -> accent surface",
                    );
                    structural_item(
                        ui,
                        "structural-main-three",
                        "nth-child(3)",
                        ".structural-item:nth-child(3) -> purple left rail",
                    );
                    structural_item(
                        ui,
                        "structural-main-four",
                        "last-child",
                        ".structural-item:last-child -> purple border",
                    );
                },
            );
            ui.element(
                "structural-nested-shell",
                ElementSpec::new(Element::Div).class("structural-nested-shell"),
                |ui| {
                    for (list, label) in [("a", "Project A"), ("b", "Project B")] {
                        ui.element(
                            format!("structural-nested-list-{list}"),
                            ElementSpec::new(Element::Div).class("structural-list"),
                            |ui| {
                                structural_nested_item(
                                    ui,
                                    format!("structural-nested-{list}-one"),
                                    label,
                                    "first child",
                                );
                                structural_nested_item(
                                    ui,
                                    format!("structural-nested-{list}-two"),
                                    "Pipeline",
                                    "last child",
                                );
                            },
                        );
                    }
                },
            );
        },
    );
}

fn structural_item(
    ui: &mut des_ui_document::DocumentBuilder,
    id: &'static str,
    label: &'static str,
    body: &'static str,
) {
    ui.element(
        id,
        ElementSpec::new(Element::Div).class("structural-item"),
        |ui| {
            ui.text_element(
                format!("{id}-label"),
                ElementSpec::new(Element::Text).class("card-title"),
                label,
            );
            ui.text_element(
                format!("{id}-body"),
                ElementSpec::new(Element::Text).class("muted"),
                body,
            );
        },
    );
}

fn structural_nested_item(
    ui: &mut des_ui_document::DocumentBuilder,
    id: String,
    label: &'static str,
    body: &'static str,
) {
    ui.element(
        id.clone(),
        ElementSpec::new(Element::Div).class("structural-item"),
        |ui| {
            ui.text_element(
                format!("{id}-label"),
                ElementSpec::new(Element::Text).class("card-title"),
                label,
            );
            ui.text_element(
                format!("{id}-body"),
                ElementSpec::new(Element::Text).class("muted"),
                body,
            );
        },
    );
}

fn render_animation_view(ui: &mut des_ui_document::DocumentBuilder) {
    ui.text_element(
        "animation-heading",
        ElementSpec::new(Element::Text).class("heading"),
        "Animation Specimens",
    );
    ui.text_element(
        "animation-copy",
        ElementSpec::new(Element::Text).class("muted"),
        "Each specimen isolates one state selector and the style properties it animates.",
    );
    ui.element(
        "animation-grid",
        ElementSpec::new(Element::Div).class("animation-grid"),
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
        ElementSpec::new(Element::Div).class("animation-specimen"),
        |ui| {
            ui.text_element(
                "animation-hover-margin-title",
                ElementSpec::new(Element::Text).class("box-label"),
                "Hovered Margin",
            );
            ui.text_element(
                "animation-hover-margin-note",
                ElementSpec::new(Element::Text).class("box-note"),
                "margin animates inside the parent and pushes neighboring boxes",
            );
            ui.text_element(
                "animation-hover-margin-rule-0",
                ElementSpec::new(Element::Text).class("box-rule"),
                "base: margin all sides 0",
            );
            ui.text_element(
                "animation-hover-margin-rule-1",
                ElementSpec::new(Element::Text).class("box-rule"),
                "hovered: margin all sides 18",
            );
            ui.element(
                "animation-hover-margin-surface",
                ElementSpec::new(Element::Div).class("animation-surface"),
                |ui| {
                    ui.element(
                        "animation-hover-margin-row",
                        ElementSpec::new(Element::Div).class("animation-margin-row"),
                        |ui| {
                            for id in [
                                "animation-hover-margin-before",
                                "animation-hover-margin-target",
                                "animation-hover-margin-after",
                            ] {
                                let spec = ElementSpec::new(Element::Div)
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
        ElementSpec::new(Element::Div).class("animation-specimen"),
        |ui| {
            ui.text_element(
                format!("{id}-title"),
                ElementSpec::new(Element::Text).class("box-label"),
                title,
            );
            ui.text_element(
                format!("{id}-note"),
                ElementSpec::new(Element::Text).class("box-note"),
                note,
            );
            for (line_index, line) in rule_text.split(" | ").enumerate() {
                ui.text_element(
                    format!("{id}-rule-{line_index}"),
                    ElementSpec::new(Element::Text).class("box-rule"),
                    line,
                );
            }
            ui.element(
                format!("{id}-surface"),
                ElementSpec::new(Element::Div).class("animation-surface"),
                |ui| {
                    ui.element(
                        format!("{id}-box"),
                        ElementSpec::new(Element::Div)
                            .class("animation-box")
                            .class(box_class)
                            .interactive()
                            .selected(selected)
                            .disabled(disabled)
                            .focused(focused),
                        |ui| {
                            ui.text_element(
                                format!("{id}-box-label"),
                                ElementSpec::new(Element::Text)
                                    .class("animation-box-label")
                                    .selected(selected)
                                    .disabled(disabled)
                                    .focused(focused),
                                title,
                            );
                            ui.text_element(
                                format!("{id}-box-body"),
                                ElementSpec::new(Element::Text)
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
        ElementSpec::new(Element::Text).class("heading"),
        "Document Scrolling",
    );
    ui.text_element(
        "scroll-copy",
        ElementSpec::new(Element::Text).class("muted"),
        "Use the wheel or touchpad over each panel. Scroll offsets live in des-ui-document.",
    );
    ui.text_element(
        "scroll-direct-title",
        ElementSpec::new(Element::Text).class("section-title"),
        "Direct containers",
    );
    ui.element(
        "scroll-row",
        ElementSpec::new(Element::Div).class("card-row"),
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
        ElementSpec::new(Element::Text).class("section-title"),
        "Nested containers",
    );
    ui.element(
        "scroll-nested-row",
        ElementSpec::new(Element::Div).class("card-row"),
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

fn render_table_view(ui: &mut des_ui_document::DocumentBuilder) {
    ui.text_element(
        "table-heading",
        ElementSpec::new(Element::Text).class("heading"),
        "Document Table",
    );
    ui.text_element(
        "table-copy",
        ElementSpec::new(Element::Text).class("muted"),
        "Table layout resolves column tracks once and applies them to headers and body cells.",
    );
    ui.element(
        "table-specimen-card",
        ElementSpec::new(Element::Div).class("specimen-card"),
        |ui| {
            ui.text_element(
                "table-specimen-title",
                ElementSpec::new(Element::Text).class("card-title"),
                "Data-driven columns",
            );
            ui.element(
                "customer-preview-table",
                ElementSpec::new(Element::Table)
                    .class("data-table")
                    .class("styled-scrollbar")
                    .table(sample_table_spec()),
                |ui| {
                    table_header(ui);
                    for (index, row) in sample_table_rows().iter().enumerate() {
                        table_row(ui, index, row);
                    }
                },
            );
        },
    );
}

fn render_floating_view(ui: &mut des_ui_document::DocumentBuilder) {
    ui.text_element(
        "floating-heading",
        ElementSpec::new(Element::Text).class("heading"),
        "Floating Layout",
    );
    ui.text_element(
        "floating-copy",
        ElementSpec::new(Element::Text).class("muted"),
        "Anchored surfaces use document styles for placement, fallback, shift, and optional arrow geometry.",
    );
    ui.element(
        "floating-playground",
        ElementSpec::new(Element::Div).class("floating-playground"),
        |ui| {
            floating_anchor(ui, "floating-anchor-edge", "Edge anchor");
            floating_anchor(ui, "floating-anchor-arrow", "Arrow anchor");
            floating_anchor(ui, "floating-anchor-plain", "Plain anchor");

            floating_popover(
                ui,
                "floating-shift-popover",
                "Fallback + shift",
                "Prefers right, falls back left near the viewport edge.",
                "floating-popover",
            );
            floating_popover(
                ui,
                "floating-arrow-popover",
                "Arrow surface",
                "Arrow is opt-in and painted with the surface shadow.",
                "floating-popover",
            );
            floating_popover(
                ui,
                "floating-plain-popover",
                "No arrow",
                "Same floating contract with rectangular painting.",
                "floating-popover",
            );
        },
    );
}

fn floating_anchor(ui: &mut des_ui_document::DocumentBuilder, id: &str, label: &str) {
    ui.element(
        id,
        ElementSpec::new(Element::Button)
            .class("floating-anchor")
            .interactive(),
        |ui| {
            ui.text_element(
                format!("{id}-label"),
                ElementSpec::new(Element::Text).class("card-title"),
                label,
            );
        },
    );
}

fn floating_popover(
    ui: &mut des_ui_document::DocumentBuilder,
    id: &str,
    title: &str,
    copy: &str,
    class: &str,
) {
    ui.element(id, ElementSpec::new(Element::Div).class(class), |ui| {
        ui.text_element(
            format!("{id}-title"),
            ElementSpec::new(Element::Text).class("card-title"),
            title,
        );
        ui.text_element(
            format!("{id}-copy"),
            ElementSpec::new(Element::Text).class("muted"),
            copy,
        );
    });
}

fn render_text_view(ui: &mut des_ui_document::DocumentBuilder) {
    ui.text_element(
        "text-heading",
        ElementSpec::new(Element::Text).class("heading"),
        "Text Measurement",
    );
    ui.text_element(
        "text-copy",
        ElementSpec::new(Element::Text)
            .class("muted")
            .class("text-copy"),
        "These specimens use the document text contract with egui/epaint providing the real measurement and galley painting.",
    );
    ui.element(
        "text-specimen-grid",
        ElementSpec::new(Element::Div).class("text-specimen-grid"),
        |ui| {
            text_specimen(
                ui,
                "text-extend",
                "Extend",
                "width: Px(220); text-wrap: Extend",
                "Long labels stay on one measured line and can extend past a narrow box.",
                "text-box-extend",
            );
            text_specimen(
                ui,
                "text-wrap",
                "Wrap",
                "width: Px(220); text-wrap: Wrap",
                "Long labels wrap naturally inside the fixed content width using epaint line breaking.",
                "text-box-wrap",
            );
            text_specimen(
                ui,
                "text-truncate",
                "Truncate",
                "width: Px(220); text-wrap: Truncate",
                "A compact field title should elide when the value is too wide for its container.",
                "text-box-truncate",
            );
            text_specimen(
                ui,
                "text-max-lines",
                "Max lines",
                "width: Px(220); text-wrap: Wrap; max-lines: 2",
                "Preview descriptions can wrap for two lines and then stop cleanly when they still have more content.",
                "text-box-max-lines",
            );
        },
    );
}

fn text_specimen(
    ui: &mut des_ui_document::DocumentBuilder,
    id: &str,
    title: &str,
    rule: &str,
    body: &str,
    text_class: &str,
) {
    ui.element(
        id,
        ElementSpec::new(Element::Div).class("text-specimen-card"),
        |ui| {
            ui.text_element(
                format!("{id}-title"),
                ElementSpec::new(Element::Text).class("card-title"),
                title,
            );
            ui.text_element(
                format!("{id}-rule"),
                ElementSpec::new(Element::Text)
                    .class("muted")
                    .class("text-rule"),
                rule,
            );
            ui.text_element(
                format!("{id}-body"),
                ElementSpec::new(Element::Text)
                    .class("text-box")
                    .class(text_class)
                    .selectable_text(),
                body,
            );
        },
    );
}

fn sample_table_spec() -> TableSpec {
    TableSpec::new(vec![
        TableColumnSpec::new("customer", "Customer")
            .width(TableTrackSize::px(170.0))
            .min_width(120.0),
        TableColumnSpec::new("country", "Country")
            .width(TableTrackSize::px(110.0))
            .min_width(80.0),
        TableColumnSpec::new("orders", "Orders")
            .width(TableTrackSize::px(82.0))
            .min_width(64.0),
        TableColumnSpec::new("revenue", "Revenue")
            .width(TableTrackSize::px(112.0))
            .min_width(90.0),
        TableColumnSpec::new("status", "Status")
            .width(TableTrackSize::flex(1.0))
            .min_width(120.0),
    ])
    .header_height(34.0)
    .row_height(32.0)
}

fn table_header(ui: &mut des_ui_document::DocumentBuilder) {
    ui.element(
        "customer-preview-header",
        ElementSpec::new(Element::Thead).class("table-header-row"),
        |ui| {
            for column in sample_table_spec().columns {
                ui.text_element(
                    format!("customer-preview-header-{}", column.id.as_str()),
                    ElementSpec::new(Element::Td)
                        .class("table-header-cell")
                        .table_cell(TableCellSpec::new(column.id)),
                    column.title,
                );
            }
        },
    );
}

fn table_row(ui: &mut des_ui_document::DocumentBuilder, index: usize, row: &[&str; 5]) {
    ui.element(
        format!("customer-preview-row-{index}"),
        ElementSpec::new(Element::Tr).class("table-row"),
        |ui| {
            for (column, value) in sample_table_spec().columns.iter().zip(row.iter()) {
                ui.text_element(
                    format!("customer-preview-row-{index}-{}", column.id.as_str()),
                    ElementSpec::new(Element::Td)
                        .class("table-cell")
                        .table_cell(TableCellSpec::new(column.id.clone())),
                    *value,
                );
            }
        },
    );
}

fn sample_table_rows() -> [[&'static str; 5]; 6] {
    [
        ["Acme Logistics", "US", "182", "$42,880", "Active"],
        ["Northwind", "CA", "94", "$18,250", "Review"],
        ["Globex Retail", "UK", "211", "$51,040", "Active"],
        ["Initech", "US", "33", "$7,920", "Draft"],
        ["Umbrella", "DE", "76", "$14,600", "Paused"],
        ["Stark Data", "FR", "128", "$29,440", "Active"],
    ]
}

fn render_nesting_view(ui: &mut des_ui_document::DocumentBuilder) {
    ui.text_element(
        "nesting-heading",
        ElementSpec::new(Element::Text).class("heading"),
        "Nested Relative Boxes",
    );
    ui.text_element(
        "nesting-copy",
        ElementSpec::new(Element::Text).class("muted"),
        "Each child is positioned relative to its parent content rect. Absolute positioning comes next.",
    );
    ui.element(
        "nest-outer",
        ElementSpec::new(Element::Div).class("nest-outer"),
        |ui| {
            ui.text_element(
                "nest-outer-title",
                ElementSpec::new(Element::Text).class("card-title"),
                "Outer panel",
            );
            ui.element(
                "nest-middle",
                ElementSpec::new(Element::Div).class("nest-middle"),
                |ui| {
                    ui.text_element(
                        "nest-middle-title",
                        ElementSpec::new(Element::Text).class("card-title"),
                        "Middle card",
                    );
                    ui.element(
                        "nest-inner",
                        ElementSpec::new(Element::Div)
                            .class("nest-inner")
                            .interactive(),
                        |ui| {
                            ui.text_element(
                                "nest-inner-title",
                                ElementSpec::new(Element::Text).class("card-title"),
                                "Inner interactive box",
                            );
                            ui.text_element(
                                "nest-inner-body",
                                ElementSpec::new(Element::Text).class("muted"),
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
        ElementSpec::new(Element::Text).class("heading"),
        "Graph Surface Plan",
    );
    ui.text_element(
        "graph-copy",
        ElementSpec::new(Element::Text).class("muted"),
        "This placeholder reserves the lab view for canvas, layers, custom geometry, and bezier hit testing.",
    );
    ui.element(
        "graph-canvas-placeholder",
        ElementSpec::new(Element::Canvas).class("canvas-placeholder"),
        |ui| {
            ui.text_element(
                "graph-canvas-title",
                ElementSpec::new(Element::Text).class("card-title"),
                "Canvas adapter target",
            );
            ui.text_element(
                "graph-canvas-body",
                ElementSpec::new(Element::Text).class("muted"),
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
        ElementSpec::new(Element::Div)
            .class("list-row")
            .class("specificity-proof")
            .interactive(),
        |ui| {
            ui.text_element(
                format!("{id}-label"),
                ElementSpec::new(Element::Text).class("card-title"),
                label,
            );
            ui.text_element(
                format!("{id}-body"),
                ElementSpec::new(Element::Text).class("muted"),
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
        ElementSpec::new(Element::Div).class("scroll-panel"),
        |ui| {
            ui.text_element(
                format!("{id}-title"),
                ElementSpec::new(Element::Text).class("card-title"),
                title,
            );
            ui.element(
                format!("{id}-list"),
                ElementSpec::new(Element::Div)
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
        ElementSpec::new(Element::Div).class("scroll-panel"),
        |ui| {
            ui.text_element(
                format!("{id}-title"),
                ElementSpec::new(Element::Text).class("card-title"),
                title,
            );
            ui.element(
                format!("{id}-shell"),
                ElementSpec::new(Element::Div).class("scroll-nested-shell"),
                |ui| {
                    ui.element(
                        format!("{id}-list"),
                        ElementSpec::new(Element::Div)
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
            ElementSpec::new(Element::Div)
                .class(row_class)
                .interactive(),
            |ui| {
                ui.text_element(
                    format!("{id}-row-{index}-label"),
                    ElementSpec::new(Element::Text).class("muted"),
                    format!("document-owned scroll row {:02}", index + 1),
                );
            },
        );
    }
}

fn scroll_wide_card(ui: &mut des_ui_document::DocumentBuilder, id: &'static str, index: usize) {
    ui.element(
        format!("{id}-row-{index}"),
        ElementSpec::new(Element::Div)
            .class("scroll-wide-row-card")
            .interactive(),
        |ui| {
            ui.text_element(
                format!("{id}-row-{index}-label"),
                ElementSpec::new(Element::Text).class("muted"),
                format!("horizontal card {:02}", index + 1),
            );
            ui.element(
                format!("{id}-row-{index}-mini-list"),
                ElementSpec::new(Element::Div)
                    .class("scroll-mini-list")
                    .class("styled-scrollbar"),
                |ui| {
                    for item_index in 0..8 {
                        ui.element(
                            format!("{id}-row-{index}-mini-row-{item_index}"),
                            ElementSpec::new(Element::Div)
                                .class("scroll-mini-row")
                                .interactive(),
                            |ui| {
                                ui.text_element(
                                    format!("{id}-row-{index}-mini-row-{item_index}-label"),
                                    ElementSpec::new(Element::Text).class("muted"),
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
