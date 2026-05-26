use super::*;
use des_document::Glyph;

pub(super) fn render_stage(ui: &mut des_document::DocumentBuilder, state: StageRenderState<'_>) {
    let StageRenderState {
        view,
        show_optional_card,
        dense_mode,
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
            LabView::Interaction => render_interaction_view(ui),
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

fn render_interaction_view(ui: &mut des_document::DocumentBuilder) {
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
    super::html::append_interaction_controls(ui);
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

fn render_styling_view(
    ui: &mut des_document::DocumentBuilder,
    _dense_mode: bool,
    shadow_tune: ShadowTuneState,
    shadow_hover_tune: ShadowTuneState,
) {
    super::html::append_styling_overview(ui);
    render_shadow_specimens(ui);
    render_shadow_tuner(ui, shadow_tune, shadow_hover_tune);
    render_structural_selector_specimens(ui);
}

fn render_shadow_specimens(ui: &mut des_document::DocumentBuilder) {
    super::html::append_shadow_specimens(ui);
}

fn render_shadow_tuner(
    ui: &mut des_document::DocumentBuilder,
    shadow_tune: ShadowTuneState,
    shadow_hover_tune: ShadowTuneState,
) {
    ShadowStyler::new(shadow_tune, shadow_hover_tune).render(ui);
}

fn render_structural_selector_specimens(ui: &mut des_document::DocumentBuilder) {
    super::html::append_structural_selectors(ui);
}

fn render_animation_view(ui: &mut des_document::DocumentBuilder) {
    super::html::append_animation(ui);
}

fn render_scrolling_view(ui: &mut des_document::DocumentBuilder) {
    super::html::append_scrolling(ui);
}

fn render_table_view(ui: &mut des_document::DocumentBuilder) {
    super::html::append_table(ui);
}

fn render_floating_view(ui: &mut des_document::DocumentBuilder) {
    super::html::append_floating(ui);
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
