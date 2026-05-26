use super::*;
use des_document::Glyph;

pub(super) fn render_stage(ui: &mut des_document::DocumentBuilder, state: StageRenderState<'_>) {
    let StageRenderState {
        view,
        show_optional_card,
        dense_mode,
        checkbox_enabled,
        radio_choice,
        dropdown_open,
        dropdown_choice,
        drag,
        shadow_tune,
        shadow_hover_tune,
    } = state;
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
            LabView::Draggable => render_draggable_view(ui, drag),
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

#[derive(Clone, Copy, Debug)]
pub(super) struct StageRenderState<'a> {
    view: LabView,
    show_optional_card: bool,
    dense_mode: bool,
    checkbox_enabled: bool,
    radio_choice: usize,
    dropdown_open: bool,
    dropdown_choice: usize,
    drag: DragLabState<'a>,
    shadow_tune: ShadowTuneState,
    shadow_hover_tune: ShadowTuneState,
}

impl<'a> StageRenderState<'a> {
    pub(super) fn new(view: LabView) -> Self {
        Self {
            view,
            show_optional_card: false,
            dense_mode: false,
            checkbox_enabled: false,
            radio_choice: 0,
            dropdown_open: false,
            dropdown_choice: 0,
            drag: DragLabState::default(),
            shadow_tune: ShadowTuneState::default(),
            shadow_hover_tune: ShadowTuneState::default(),
        }
    }

    pub(super) fn optional_card(mut self, show_optional_card: bool) -> Self {
        self.show_optional_card = show_optional_card;
        self
    }

    pub(super) fn dense_mode(mut self, dense_mode: bool) -> Self {
        self.dense_mode = dense_mode;
        self
    }

    pub(super) fn controls(
        mut self,
        checkbox_enabled: bool,
        radio_choice: usize,
        dropdown_open: bool,
        dropdown_choice: usize,
    ) -> Self {
        self.checkbox_enabled = checkbox_enabled;
        self.radio_choice = radio_choice;
        self.dropdown_open = dropdown_open;
        self.dropdown_choice = dropdown_choice;
        self
    }

    pub(super) fn drag(mut self, drag: DragLabState<'a>) -> Self {
        self.drag = drag;
        self
    }

    pub(super) fn shadows(
        mut self,
        shadow_tune: ShadowTuneState,
        shadow_hover_tune: ShadowTuneState,
    ) -> Self {
        self.shadow_tune = shadow_tune;
        self.shadow_hover_tune = shadow_hover_tune;
        self
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct DragLabState<'a> {
    drag_item_cells: [usize; 3],
    drag_item_order: [usize; 3],
    scroll_list_item_order: [usize; 14],
    pressed_drag_source: Option<&'a str>,
    active_drag_item: Option<des_widgets::SortableItemId>,
    active_scroll_list_drag_item: Option<des_widgets::SortableItemId>,
    drag_pointer: Option<des_document::Point>,
    drag_drop_preview: Option<des_widgets::SortableDropPreview>,
    scroll_list_drop_preview: Option<des_widgets::SortableDropPreview>,
}

impl Default for DragLabState<'_> {
    fn default() -> Self {
        Self {
            drag_item_cells: [0; 3],
            drag_item_order: [0, 1, 2],
            scroll_list_item_order: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13],
            pressed_drag_source: None,
            active_drag_item: None,
            active_scroll_list_drag_item: None,
            drag_pointer: None,
            drag_drop_preview: None,
            scroll_list_drop_preview: None,
        }
    }
}

impl<'a> DragLabState<'a> {
    pub(super) fn new(
        drag_item_cells: [usize; 3],
        drag_item_order: [usize; 3],
        scroll_list_item_order: [usize; 14],
    ) -> Self {
        Self {
            drag_item_cells,
            drag_item_order,
            scroll_list_item_order,
            ..Default::default()
        }
    }

    pub(super) fn active(
        mut self,
        pressed_drag_source: Option<&'a str>,
        active_drag_item: Option<des_widgets::SortableItemId>,
        active_scroll_list_drag_item: Option<des_widgets::SortableItemId>,
        drag_pointer: Option<des_document::Point>,
    ) -> Self {
        self.pressed_drag_source = pressed_drag_source;
        self.active_drag_item = active_drag_item;
        self.active_scroll_list_drag_item = active_scroll_list_drag_item;
        self.drag_pointer = drag_pointer;
        self
    }

    pub(super) fn previews(
        mut self,
        drag_drop_preview: Option<des_widgets::SortableDropPreview>,
        scroll_list_drop_preview: Option<des_widgets::SortableDropPreview>,
    ) -> Self {
        self.drag_drop_preview = drag_drop_preview;
        self.scroll_list_drop_preview = scroll_list_drop_preview;
        self
    }
}

fn render_layout_view(
    ui: &mut des_document::DocumentBuilder,
    _show_optional_card: bool,
    _dense_mode: bool,
) {
    super::html::append_layout(ui);
}

fn render_interaction_view(
    ui: &mut des_document::DocumentBuilder,
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
    super::html::append_interaction_cards(ui);
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

fn render_draggable_view(ui: &mut des_document::DocumentBuilder, drag: DragLabState<'_>) {
    ui.child("draggable-heading", Element::Text)
        .class("heading")
        .text("Document Draggables");
    ui.child("draggable-copy", Element::Text)
        .class("muted")
        .text("Sortable drag/drop uses document events, visual subtree clones, optional handles, and style-owned overlays.");
    render_drag_drop_lab(ui, drag);
}

fn render_document_update_loop(ui: &mut des_document::DocumentBuilder) {
    super::html::append_interaction_loop(ui);
}

fn render_drag_drop_lab(ui: &mut des_document::DocumentBuilder, drag: DragLabState<'_>) {
    let DragLabState {
        drag_item_cells,
        drag_item_order,
        scroll_list_item_order,
        pressed_drag_source,
        active_drag_item,
        active_scroll_list_drag_item,
        drag_pointer: _,
        drag_drop_preview,
        scroll_list_drop_preview,
    } = drag;
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
                                if active_drag_item == Some(des_widgets::SortableItemId(item)) {
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
    ui: &mut des_document::DocumentBuilder,
    title: &'static str,
    scroll_list_item_order: [usize; 14],
    active_scroll_list_drag_item: Option<des_widgets::SortableItemId>,
    scroll_list_drop_preview: Option<des_widgets::SortableDropPreview>,
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
                            active_scroll_list_drag_item == Some(des_widgets::SortableItemId(item)),
                            pressed_drag_source,
                        );
                    }
                });
        });
}

pub(super) fn render_drag_overlay_layer(
    ui: &mut des_document::DocumentBuilder,
    drag_pointer: Option<des_document::Point>,
    drag_visual_clone: Option<&des_document::VisualElementClone>,
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

pub(super) fn render_debug_overlay_layer(ui: &mut des_document::DocumentBuilder, perf: UiLabPerf) {
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
                "debug-stylesheet-time",
                "stylesheet",
                format_duration(perf.stylesheet_time),
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
                "debug-text-cache",
                "text cache",
                format!(
                    "{} text / {} glyph / {} new / {} cached / {} pages",
                    perf.text_paint.glyphs_painted,
                    perf.text_paint.glyph_cache_hits,
                    perf.text_paint.rasterizations,
                    perf.text_paint.cached_glyphs,
                    perf.text_paint.atlas_pages
                ),
            );
            debug_metric_row(
                ui,
                "debug-text-layout-cache",
                "text layout",
                format!(
                    "{} hit / {} miss / {} cached",
                    perf.text_paint.layout_cache_hits,
                    perf.text_paint.layout_cache_misses,
                    perf.text_paint.layout_cache_entries
                ),
            );
            debug_metric_row(
                ui,
                "debug-text-paint-run-cache",
                "text paint runs",
                format!(
                    "{} hit / {} miss / {} cached",
                    perf.text_paint.paint_run_cache_hits,
                    perf.text_paint.paint_run_cache_misses,
                    perf.text_paint.paint_run_cache_entries
                ),
            );
            debug_metric_row(
                ui,
                "debug-text-mesh-cache",
                "text mesh cache",
                format!(
                    "{} hit / {} miss / {} cached",
                    perf.text_paint.glyph_mesh_cache_hits,
                    perf.text_paint.glyph_mesh_cache_misses,
                    perf.text_paint.glyph_mesh_cache_entries
                ),
            );
            debug_metric_row(
                ui,
                "debug-text-measure",
                "text measure",
                format!(
                    "{} / {}",
                    perf.text_paint.measure_requests,
                    format_duration(perf.text_paint.measure_time)
                ),
            );
            debug_metric_row(
                ui,
                "debug-text-glyph-run",
                "text glyph run",
                format!(
                    "{} / {}",
                    perf.text_paint.paint_text_requests,
                    format_duration(perf.text_paint.glyph_run_time)
                ),
            );
            debug_metric_row(
                ui,
                "debug-text-hit-test",
                "text hit test",
                format!(
                    "{} / {}",
                    perf.text_paint.hit_test_requests,
                    format_duration(perf.text_paint.hit_test_time)
                ),
            );
            debug_metric_row(
                ui,
                "debug-text-atlas-time",
                "text atlas",
                format_duration(perf.text_paint.glyph_atlas_time),
            );
            debug_metric_row(
                ui,
                "debug-text-glyph-image-time",
                "text glyph image",
                format_duration(perf.text_paint.glyph_image_time),
            );
            debug_metric_row(
                ui,
                "debug-text-upload-time",
                "text upload",
                format_duration(perf.text_paint.glyph_upload_time),
            );
            debug_metric_row(
                ui,
                "debug-text-glyph-paint-time",
                "text glyph paint",
                format_duration(perf.text_paint.glyph_paint_time),
            );
            debug_metric_row(
                ui,
                "debug-text-glyph-meshes",
                "text glyph meshes",
                perf.text_paint.glyph_meshes.to_string(),
            );
            debug_metric_row(
                ui,
                "debug-text-pixels",
                "text upload px",
                perf.text_paint.uploaded_pixels.to_string(),
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
    ui: &mut des_document::DocumentBuilder,
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
    ui: &mut des_document::DocumentBuilder,
    item: usize,
    drag_drop_preview: Option<des_widgets::SortableDropPreview>,
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
        && preview.nearest_item == Some(des_widgets::SortableItemId(item))
    {
        item_builder = item_builder.class(match preview.edge {
            des_widgets::DropEdge::Before => "drag-gap-before",
            des_widgets::DropEdge::After => "drag-gap-after",
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

fn drag_scroll_handle(ui: &mut des_document::DocumentBuilder, item: usize, origin_space: bool) {
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
    ui: &mut des_document::DocumentBuilder,
    item: usize,
    drag_drop_preview: Option<des_widgets::SortableDropPreview>,
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
        && preview.nearest_item == Some(des_widgets::SortableItemId(item))
    {
        item_builder = item_builder.class(match preview.edge {
            des_widgets::DropEdge::Before => "drag-gap-before",
            des_widgets::DropEdge::After => "drag-gap-after",
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

fn drag_handle(ui: &mut des_document::DocumentBuilder, item: usize, origin_space: bool) {
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
    ui: &mut des_document::DocumentBuilder,
    clone: &des_document::VisualElementClone,
) {
    ui.visual_clone(
        clone,
        des_document::VisualCloneOptions::new("drag-overlay", "drag-overlay/")
            .root_class("drag-overlay"),
    );
}

fn drag_overlay_placeholder(ui: &mut des_document::DocumentBuilder) {
    ui.div("drag-overlay")
        .class("drag-overlay")
        .class("drag-overlay-idle")
        .value("")
        .empty();
}

fn control_checkbox(ui: &mut des_document::DocumentBuilder, checked: bool) {
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
                    .on_click("control-checkbox")
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

fn control_radio_group(ui: &mut des_document::DocumentBuilder, choice: usize) {
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
                        .on_click(id)
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

fn control_dropdown(ui: &mut des_document::DocumentBuilder, open: bool, choice: usize) {
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
                    .on_click("control-dropdown"),
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
                                            .on_click(id)
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

fn control_text_inputs(ui: &mut des_document::DocumentBuilder) {
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
    ui: &mut des_document::DocumentBuilder,
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

fn render_shadow_specimens(ui: &mut des_document::DocumentBuilder) {
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
    ui: &mut des_document::DocumentBuilder,
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
        ElementSpec::new(Element::Text)
            .class("muted")
            .class("param-description"),
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
    ui: &mut des_document::DocumentBuilder,
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
    ui: &mut des_document::DocumentBuilder,
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
    ui: &mut des_document::DocumentBuilder,
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
    ui: &mut des_document::DocumentBuilder,
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
    ui: &mut des_document::DocumentBuilder,
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
    ui: &mut des_document::DocumentBuilder,
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
    ui: &mut des_document::DocumentBuilder,
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
    ui: &mut des_document::DocumentBuilder,
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

fn render_structural_selector_specimens(ui: &mut des_document::DocumentBuilder) {
    super::html::append_structural_selectors(ui);
}

fn render_animation_view(ui: &mut des_document::DocumentBuilder) {
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
                AnimationSpecimen {
                    id: "animation-hover-size",
                    title: "Hovered",
                    note: "width and height animate while the pointer is over the specimen",
                    rule_text:
                        "base: width Px(150); height Px(58) | hovered: width Px(220); height Px(84)",
                    box_class: "animation-box-hover-size",
                    ..Default::default()
                },
            );
            animation_margin_specimen(ui);
            animation_specimen(
                ui,
                AnimationSpecimen {
                    id: "animation-pressed-border",
                    title: "Pressed",
                    note: "border width and corner radius animate while primary pointer is down",
                    rule_text: "base: border width all sides 2; radius all corners 4 | pressed: border width all sides 10; radius all corners 22",
                    box_class: "animation-box-pressed-border",
                    ..Default::default()
                },
            );
            animation_specimen(
                ui,
                AnimationSpecimen {
                    id: "animation-selected-spacing",
                    title: "Selected",
                    note: "size, spacing, color, radius, and font size animate from selected state",
                    rule_text: "base: width 150; height 58; padding 8; margin 0; radius 4 | selected: width 210; height 92; padding 16; margin 10; radius 12",
                    box_class: "animation-box-selected-spacing",
                    selected: true,
                    ..Default::default()
                },
            );
            animation_specimen(
                ui,
                AnimationSpecimen {
                    id: "animation-disabled-color",
                    title: "Disabled",
                    note: "background, border color, and text color animate from disabled state",
                    rule_text: "base: background; border; text color | disabled: muted background; muted border; muted text color",
                    box_class: "animation-box-disabled-color",
                    disabled: true,
                    ..Default::default()
                },
            );
            animation_specimen(
                ui,
                AnimationSpecimen {
                    id: "animation-focused-min-size",
                    title: "Focused",
                    note: "size, border width, color, and radius animate from focused state",
                    rule_text: "base: width 150; height 58; border width 2; radius 4 | focused: width 226; height 88; border width 6; radius 16",
                    box_class: "animation-box-focused-min-size",
                    focused: true,
                    ..Default::default()
                },
            );
        },
    );
}

fn animation_margin_specimen(ui: &mut des_document::DocumentBuilder) {
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

#[derive(Clone, Copy, Debug, Default)]
struct AnimationSpecimen {
    id: &'static str,
    title: &'static str,
    note: &'static str,
    rule_text: &'static str,
    box_class: &'static str,
    selected: bool,
    disabled: bool,
    focused: bool,
}

fn animation_specimen(ui: &mut des_document::DocumentBuilder, specimen: AnimationSpecimen) {
    let AnimationSpecimen {
        id,
        title,
        note,
        rule_text,
        box_class,
        selected,
        disabled,
        focused,
    } = specimen;
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
                                    .disabled(disabled),
                                title,
                            );
                            ui.text_element(
                                format!("{id}-box-body"),
                                ElementSpec::new(Element::Text)
                                    .class("animation-box-body")
                                    .selected(selected)
                                    .disabled(disabled),
                                "state-driven transition",
                            );
                        },
                    );
                },
            );
        },
    );
}

fn render_scrolling_view(ui: &mut des_document::DocumentBuilder) {
    super::html::append_scrolling(ui);
}

fn render_table_view(ui: &mut des_document::DocumentBuilder) {
    super::html::append_table(ui);
}

fn render_floating_view(ui: &mut des_document::DocumentBuilder) {
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
            floating_offset_specimen(ui);
            floating_main_axis_specimen(ui);
            floating_cross_axis_specimen(ui);
            floating_alignment_axis_specimen(ui);
            floating_centered_axis_specimen(ui);
            floating_top_start_specimen(ui);
            floating_scroll_shift_specimen(ui);
            floating_scroll_attach_specimen(ui);
            floating_vertical_scroll_overlap_specimen(ui);
            floating_vertical_scroll_flip_specimen(ui);
            floating_edge_flip_specimen(ui);
        },
    );
}

fn floating_offset_specimen(ui: &mut des_document::DocumentBuilder) {
    ui.element(
        "floating-offset-specimen",
        ElementSpec::new(Element::Div).class("floating-specimen-box"),
        |ui| {
            ui.text_element(
                "floating-offset-specimen-title",
                ElementSpec::new(Element::Text).class("card-title"),
                "Floating offset",
            );
            ui.element(
                "floating-offset-row",
                ElementSpec::new(Element::Div).class("floating-offset-row"),
                |ui| {
                    floating_offset_reference(ui, "floating-offset-zero-reference", "0px");
                    floating_offset_reference(ui, "floating-offset-ten-reference", "10px");
                    floating_offset_popover(ui, "floating-offset-zero-popover");
                    floating_offset_popover(ui, "floating-offset-ten-popover");
                },
            );
        },
    );
}

fn floating_offset_reference(ui: &mut des_document::DocumentBuilder, id: &str, label: &str) {
    ui.element(
        id,
        ElementSpec::new(Element::Button)
            .class("floating-offset-reference")
            .interactive(),
        |ui| {
            ui.text_element(
                format!("{id}-label"),
                ElementSpec::new(Element::Text).class("floating-offset-reference-label"),
                label,
            );
        },
    );
}

fn floating_offset_popover(ui: &mut des_document::DocumentBuilder, id: &str) {
    ui.element(
        id,
        ElementSpec::new(Element::Button)
            .class("floating-offset-popover")
            .interactive(),
        |ui| {
            ui.text_element(
                format!("{id}-label"),
                ElementSpec::new(Element::Text).class("floating-offset-popover-label"),
                "Floating",
            );
        },
    );
}

fn floating_main_axis_specimen(ui: &mut des_document::DocumentBuilder) {
    ui.element(
        "floating-main-axis-specimen",
        ElementSpec::new(Element::Div)
            .class("floating-specimen-box")
            .class("floating-main-axis-specimen"),
        |ui| {
            ui.text_element(
                "floating-main-axis-specimen-title",
                ElementSpec::new(Element::Text).class("card-title"),
                "Main axis",
            );
            ui.element(
                "floating-main-axis-stack",
                ElementSpec::new(Element::Div).class("floating-main-axis-stack"),
                |ui| {
                    ui.element(
                        "floating-main-axis-row-vertical",
                        ElementSpec::new(Element::Div).class("floating-main-axis-row"),
                        |ui| {
                            floating_main_axis_reference(
                                ui,
                                "floating-main-axis-top-reference",
                                "top",
                            );
                            floating_main_axis_reference(
                                ui,
                                "floating-main-axis-bottom-reference",
                                "bottom",
                            );
                        },
                    );
                    ui.element(
                        "floating-main-axis-row-horizontal",
                        ElementSpec::new(Element::Div).class("floating-main-axis-row"),
                        |ui| {
                            floating_main_axis_reference(
                                ui,
                                "floating-main-axis-left-reference",
                                "left",
                            );
                            floating_main_axis_reference(
                                ui,
                                "floating-main-axis-right-reference",
                                "right",
                            );
                        },
                    );
                },
            );
            floating_offset_popover(ui, "floating-main-axis-top-popover");
            floating_offset_popover(ui, "floating-main-axis-bottom-popover");
            floating_offset_popover(ui, "floating-main-axis-left-popover");
            floating_offset_popover(ui, "floating-main-axis-right-popover");
        },
    );
}

fn floating_main_axis_reference(ui: &mut des_document::DocumentBuilder, id: &str, label: &str) {
    ui.element(
        id,
        ElementSpec::new(Element::Button)
            .class("floating-offset-reference")
            .class("floating-main-axis-reference")
            .interactive(),
        |ui| {
            ui.text_element(
                format!("{id}-label"),
                ElementSpec::new(Element::Text).class("floating-offset-reference-label"),
                label,
            );
        },
    );
}

fn floating_cross_axis_specimen(ui: &mut des_document::DocumentBuilder) {
    ui.element(
        "floating-cross-axis-specimen",
        ElementSpec::new(Element::Div)
            .class("floating-specimen-box")
            .class("floating-main-axis-specimen"),
        |ui| {
            ui.text_element(
                "floating-cross-axis-specimen-title",
                ElementSpec::new(Element::Text).class("card-title"),
                "Cross axis",
            );
            ui.element(
                "floating-cross-axis-stack",
                ElementSpec::new(Element::Div).class("floating-main-axis-stack"),
                |ui| {
                    ui.element(
                        "floating-cross-axis-row-vertical",
                        ElementSpec::new(Element::Div).class("floating-main-axis-row"),
                        |ui| {
                            floating_main_axis_reference(
                                ui,
                                "floating-cross-axis-top-reference",
                                "top",
                            );
                            floating_main_axis_reference(
                                ui,
                                "floating-cross-axis-bottom-reference",
                                "bottom",
                            );
                        },
                    );
                    ui.element(
                        "floating-cross-axis-row-horizontal",
                        ElementSpec::new(Element::Div).class("floating-main-axis-row"),
                        |ui| {
                            floating_main_axis_reference(
                                ui,
                                "floating-cross-axis-left-reference",
                                "left",
                            );
                            floating_main_axis_reference(
                                ui,
                                "floating-cross-axis-right-reference",
                                "right",
                            );
                        },
                    );
                },
            );
            floating_offset_popover(ui, "floating-cross-axis-top-popover");
            floating_offset_popover(ui, "floating-cross-axis-bottom-popover");
            floating_offset_popover(ui, "floating-cross-axis-left-popover");
            floating_offset_popover(ui, "floating-cross-axis-right-popover");
        },
    );
}

fn floating_alignment_axis_specimen(ui: &mut des_document::DocumentBuilder) {
    ui.element(
        "floating-alignment-axis-specimen",
        ElementSpec::new(Element::Div)
            .class("floating-specimen-box")
            .class("floating-main-axis-specimen"),
        |ui| {
            ui.text_element(
                "floating-alignment-axis-specimen-title",
                ElementSpec::new(Element::Text).class("card-title"),
                "Alignment axis",
            );
            ui.element(
                "floating-alignment-axis-stack",
                ElementSpec::new(Element::Div).class("floating-main-axis-stack"),
                |ui| {
                    ui.element(
                        "floating-alignment-axis-cross-row",
                        ElementSpec::new(Element::Div).class("floating-main-axis-row"),
                        |ui| {
                            floating_alignment_axis_reference(
                                ui,
                                "floating-alignment-axis-cross-start-reference",
                                "top-start",
                                "cross_axis",
                            );
                            floating_alignment_axis_reference(
                                ui,
                                "floating-alignment-axis-cross-end-reference",
                                "top-end",
                                "cross_axis",
                            );
                        },
                    );
                    ui.element(
                        "floating-alignment-axis-aligned-row",
                        ElementSpec::new(Element::Div).class("floating-main-axis-row"),
                        |ui| {
                            floating_alignment_axis_reference(
                                ui,
                                "floating-alignment-axis-start-reference",
                                "top-start",
                                "alignment_axis",
                            );
                            floating_alignment_axis_reference(
                                ui,
                                "floating-alignment-axis-end-reference",
                                "top-end",
                                "alignment_axis",
                            );
                        },
                    );
                },
            );
            floating_offset_popover(ui, "floating-alignment-axis-cross-start-popover");
            floating_offset_popover(ui, "floating-alignment-axis-cross-end-popover");
            floating_offset_popover(ui, "floating-alignment-axis-start-popover");
            floating_offset_popover(ui, "floating-alignment-axis-end-popover");
        },
    );
}

fn floating_alignment_axis_reference(
    ui: &mut des_document::DocumentBuilder,
    id: &str,
    placement: &str,
    axis: &str,
) {
    ui.element(
        id,
        ElementSpec::new(Element::Button)
            .class("floating-offset-reference")
            .class("floating-alignment-axis-reference")
            .interactive(),
        |ui| {
            ui.text_element(
                format!("{id}-placement"),
                ElementSpec::new(Element::Text).class("floating-alignment-axis-placement-label"),
                placement,
            );
            ui.text_element(
                format!("{id}-axis"),
                ElementSpec::new(Element::Text).class("floating-alignment-axis-axis-label"),
                axis,
            );
        },
    );
}

fn floating_centered_axis_specimen(ui: &mut des_document::DocumentBuilder) {
    ui.element(
        "floating-centered-axis-specimen",
        ElementSpec::new(Element::Div)
            .class("floating-specimen-box")
            .class("floating-centered-axis-specimen"),
        |ui| {
            ui.text_element(
                "floating-centered-axis-specimen-title",
                ElementSpec::new(Element::Text).class("card-title"),
                "Centered axes",
            );
            ui.element(
                "floating-centered-axis-row",
                ElementSpec::new(Element::Div).class("floating-centered-axis-row"),
                |ui| {
                    floating_offset_reference(ui, "floating-centered-axis-reference", "");
                    floating_offset_popover(ui, "floating-centered-axis-popover");
                },
            );
        },
    );
}

fn floating_top_start_specimen(ui: &mut des_document::DocumentBuilder) {
    ui.element(
        "floating-top-start-specimen",
        ElementSpec::new(Element::Div)
            .class("floating-specimen-box")
            .class("floating-centered-axis-specimen"),
        |ui| {
            ui.text_element(
                "floating-top-start-specimen-title",
                ElementSpec::new(Element::Text).class("card-title"),
                "Top start",
            );
            ui.element(
                "floating-top-start-row",
                ElementSpec::new(Element::Div).class("floating-centered-axis-row"),
                |ui| {
                    floating_offset_reference(ui, "floating-top-start-reference", "");
                    floating_offset_popover(ui, "floating-top-start-popover");
                },
            );
        },
    );
}

fn floating_scroll_shift_specimen(ui: &mut des_document::DocumentBuilder) {
    ui.element(
        "floating-scroll-shift-specimen",
        ElementSpec::new(Element::Div)
            .class("floating-specimen-box")
            .class("floating-centered-axis-specimen"),
        |ui| {
            ui.text_element(
                "floating-scroll-shift-specimen-title",
                ElementSpec::new(Element::Text).class("card-title"),
                "Horizontal scroll boundary",
            );
            ui.element(
                "floating-scroll-shift-panel",
                ElementSpec::new(Element::Div)
                    .class("floating-scroll-shift-panel")
                    .class("styled-scrollbar"),
                |ui| {
                    ui.element(
                        "floating-scroll-shift-track",
                        ElementSpec::new(Element::Div).class("floating-scroll-shift-track"),
                        |ui| {
                            floating_offset_reference(ui, "floating-scroll-shift-reference", "");
                        },
                    );
                    ui.element(
                        "floating-scroll-shift-popover",
                        ElementSpec::new(Element::Button)
                            .class("floating-offset-popover")
                            .class("floating-scroll-shift-popover")
                            .interactive(),
                        |ui| {
                            ui.text_element(
                                "floating-scroll-shift-popover-label",
                                ElementSpec::new(Element::Text)
                                    .class("floating-offset-popover-label"),
                                "A floating element that shifts along the x-axis",
                            );
                        },
                    );
                },
            );
        },
    );
}

fn floating_scroll_attach_specimen(ui: &mut des_document::DocumentBuilder) {
    ui.element(
        "floating-scroll-attach-specimen",
        ElementSpec::new(Element::Div)
            .class("floating-specimen-box")
            .class("floating-centered-axis-specimen"),
        |ui| {
            ui.text_element(
                "floating-scroll-attach-specimen-title",
                ElementSpec::new(Element::Text).class("card-title"),
                "Horizontal scroll attached",
            );
            ui.element(
                "floating-scroll-attach-panel",
                ElementSpec::new(Element::Div)
                    .class("floating-scroll-shift-panel")
                    .class("styled-scrollbar"),
                |ui| {
                    ui.element(
                        "floating-scroll-attach-track",
                        ElementSpec::new(Element::Div).class("floating-scroll-shift-track"),
                        |ui| {
                            floating_offset_reference(ui, "floating-scroll-attach-reference", "");
                        },
                    );
                    ui.element(
                        "floating-scroll-attach-popover",
                        ElementSpec::new(Element::Button)
                            .class("floating-offset-popover")
                            .class("floating-scroll-shift-popover")
                            .interactive(),
                        |ui| {
                            ui.text_element(
                                "floating-scroll-attach-popover-label",
                                ElementSpec::new(Element::Text)
                                    .class("floating-offset-popover-label"),
                                "A floating element that does not shift along the x-axis",
                            );
                        },
                    );
                },
            );
        },
    );
}

fn floating_vertical_scroll_overlap_specimen(ui: &mut des_document::DocumentBuilder) {
    ui.element(
        "floating-vertical-overlap-specimen",
        ElementSpec::new(Element::Div)
            .class("floating-specimen-box")
            .class("floating-centered-axis-specimen"),
        |ui| {
            ui.text_element(
                "floating-vertical-overlap-specimen-title",
                ElementSpec::new(Element::Text).class("card-title"),
                "Vertical scroll boundary",
            );
            ui.element(
                "floating-vertical-overlap-panel",
                ElementSpec::new(Element::Div)
                    .class("floating-vertical-overlap-panel")
                    .class("styled-scrollbar"),
                |ui| {
                    ui.element(
                        "floating-vertical-overlap-track",
                        ElementSpec::new(Element::Div).class("floating-vertical-overlap-track"),
                        |ui| {
                            floating_offset_reference(
                                ui,
                                "floating-vertical-overlap-reference",
                                "",
                            );
                        },
                    );
                    ui.element(
                        "floating-vertical-overlap-popover",
                        ElementSpec::new(Element::Button)
                            .class("floating-offset-popover")
                            .class("floating-vertical-overlap-popover")
                            .interactive(),
                        |ui| {
                            ui.text_element(
                                "floating-vertical-overlap-popover-label",
                                ElementSpec::new(Element::Text)
                                    .class("floating-offset-popover-label"),
                                "I can overlap my reference element",
                            );
                        },
                    );
                },
            );
        },
    );
}

fn floating_vertical_scroll_flip_specimen(ui: &mut des_document::DocumentBuilder) {
    ui.element(
        "floating-vertical-flip-specimen",
        ElementSpec::new(Element::Div)
            .class("floating-specimen-box")
            .class("floating-centered-axis-specimen"),
        |ui| {
            ui.text_element(
                "floating-vertical-flip-specimen-title",
                ElementSpec::new(Element::Text).class("card-title"),
                "Vertical scroll flip",
            );
            ui.element(
                "floating-vertical-flip-panel",
                ElementSpec::new(Element::Div)
                    .class("floating-vertical-flip-panel")
                    .class("styled-scrollbar"),
                |ui| {
                    ui.element(
                        "floating-vertical-flip-track",
                        ElementSpec::new(Element::Div).class("floating-vertical-flip-track"),
                        |ui| {
                            floating_offset_reference(ui, "floating-vertical-flip-reference", "");
                        },
                    );
                    floating_offset_popover(ui, "floating-vertical-flip-popover");
                },
            );
        },
    );
}

fn floating_edge_flip_specimen(ui: &mut des_document::DocumentBuilder) {
    ui.element(
        "floating-edge-flip-specimen",
        ElementSpec::new(Element::Div)
            .class("floating-specimen-box")
            .class("floating-centered-axis-specimen"),
        |ui| {
            ui.text_element(
                "floating-edge-flip-specimen-title",
                ElementSpec::new(Element::Text).class("card-title"),
                "Two-axis edge flip",
            );
            ui.element(
                "floating-edge-flip-panel",
                ElementSpec::new(Element::Div)
                    .class("floating-edge-flip-panel")
                    .class("styled-scrollbar"),
                |ui| {
                    ui.element(
                        "floating-edge-flip-track",
                        ElementSpec::new(Element::Div).class("floating-edge-flip-track"),
                        |ui| {
                            floating_offset_reference(ui, "floating-edge-flip-reference", "");
                        },
                    );
                    ui.element(
                        "floating-edge-flip-popover",
                        ElementSpec::new(Element::Button)
                            .class("floating-offset-popover")
                            .class("floating-edge-flip-popover")
                            .interactive(),
                        |ui| {
                            ui.text_element(
                                "floating-edge-flip-popover-label",
                                ElementSpec::new(Element::Text)
                                    .class("floating-offset-popover-label"),
                                "I will check cross axis overflow (default)",
                            );
                        },
                    );
                },
            );
        },
    );
}

fn render_text_view(ui: &mut des_document::DocumentBuilder) {
    super::html::append_text_specimens(ui);
}

fn render_nesting_view(ui: &mut des_document::DocumentBuilder) {
    super::html::append_nesting(ui);
}

fn render_graph_view(ui: &mut des_document::DocumentBuilder) {
    super::html::append_graph(ui);
}

fn interactive_labeled_row(
    ui: &mut des_document::DocumentBuilder,
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
