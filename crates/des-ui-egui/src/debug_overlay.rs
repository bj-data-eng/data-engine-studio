use crate::{graph_canvas::GraphCanvasState, theme};
use des_app::AppSnapshot;
use eframe::egui;
use egui::{Align2, Color32, FontFamily, FontId, Pos2, Rect, Stroke, StrokeKind, vec2};

const WIDTH: f32 = 330.0;
const HEIGHT: f32 = 178.0;
const PADDING: f32 = 10.0;
const ROW_HEIGHT: f32 = 18.0;

pub(crate) fn render(
    ui: &mut egui::Ui,
    graph_rect: Rect,
    graph_canvas: &GraphCanvasState,
    snapshot: &AppSnapshot,
) {
    let diagnostics = graph_canvas.diagnostics(graph_rect, snapshot);
    let pointer = ui.input(|input| input.pointer.latest_pos());
    let scroll_delta = ui.input(|input| input.smooth_scroll_delta());
    let zoom_delta = ui.input(|input| input.zoom_delta());
    let overlay_rect = Rect::from_min_size(
        graph_rect.right_bottom() + vec2(-WIDTH - 18.0, -HEIGHT - 18.0),
        vec2(WIDTH, HEIGHT),
    );
    let painter = ui.ctx().layer_painter(egui::LayerId::new(
        egui::Order::Tooltip,
        egui::Id::new("debug_overlay_layer"),
    ));

    painter.rect(
        overlay_rect,
        6.0,
        Color32::from_rgba_premultiplied(16, 18, 21, 232),
        Stroke::new(1.0, theme::STROKE),
        StrokeKind::Inside,
    );

    let mut y = overlay_rect.top() + PADDING;
    text(
        &painter,
        overlay_rect.left() + PADDING,
        y,
        "UI Harness",
        Color32::from_rgb(230, 235, 240),
        true,
    );
    y += ROW_HEIGHT + 4.0;

    row(
        &painter,
        overlay_rect,
        y,
        "zoom",
        format!("{:.2}", diagnostics.zoom),
    );
    y += ROW_HEIGHT;
    row(
        &painter,
        overlay_rect,
        y,
        "scene",
        format!(
            "{:.0}, {:.0}, {:.0} x {:.0}",
            diagnostics.scene_rect.min.x,
            diagnostics.scene_rect.min.y,
            diagnostics.scene_rect.width(),
            diagnostics.scene_rect.height()
        ),
    );
    y += ROW_HEIGHT;
    row(
        &painter,
        overlay_rect,
        y,
        "nodes",
        diagnostics.node_count.to_string(),
    );
    y += ROW_HEIGHT;
    row(
        &painter,
        overlay_rect,
        y,
        "selected edges",
        diagnostics.selected_edge_count.to_string(),
    );
    y += ROW_HEIGHT;
    row(
        &painter,
        overlay_rect,
        y,
        "flow",
        selected_flow_label(snapshot),
    );
    y += ROW_HEIGHT;
    row(&painter, overlay_rect, y, "pointer", pointer_label(pointer));
    y += ROW_HEIGHT;
    row(
        &painter,
        overlay_rect,
        y,
        "scroll",
        format!("{:.1}, {:.1}", scroll_delta.x, scroll_delta.y),
    );
    y += ROW_HEIGHT;
    row(
        &painter,
        overlay_rect,
        y,
        "zoom delta",
        format!("{:.3}", zoom_delta),
    );
}

fn row(painter: &egui::Painter, rect: Rect, y: f32, label: &str, value: impl Into<String>) {
    text(
        painter,
        rect.left() + PADDING,
        y,
        label,
        theme::TEXT_MUTED,
        false,
    );
    painter.text(
        Pos2::new(rect.right() - PADDING, y),
        Align2::RIGHT_TOP,
        value.into(),
        FontId::new(11.0, FontFamily::Monospace),
        Color32::from_rgb(212, 220, 228),
    );
}

fn text(
    painter: &egui::Painter,
    x: f32,
    y: f32,
    value: impl Into<String>,
    color: Color32,
    strong: bool,
) {
    let size = if strong { 13.0 } else { 11.0 };
    painter.text(
        Pos2::new(x, y),
        Align2::LEFT_TOP,
        value.into(),
        FontId::new(size, FontFamily::Proportional),
        color,
    );
}

fn selected_flow_label(snapshot: &AppSnapshot) -> String {
    snapshot
        .home
        .flows
        .iter()
        .find(|flow| Some(flow.id.as_str()) == snapshot.selected_flow_id.as_deref())
        .map(|flow| flow.name.clone())
        .unwrap_or_else(|| "(none)".to_string())
}

fn pointer_label(pointer: Option<egui::Pos2>) -> String {
    pointer
        .map(|pos| format!("{:.0}, {:.0}", pos.x, pos.y))
        .unwrap_or_else(|| "(none)".to_string())
}
