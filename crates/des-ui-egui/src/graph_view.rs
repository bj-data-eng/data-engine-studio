use crate::{graph_canvas, graph_canvas::GraphCanvasState, theme};
use des_app::{AppSnapshot, StudioAppState};
use des_core::identity;
use eframe::egui;
use egui::{Align2, Color32, Rect, Stroke, vec2};

pub(crate) fn render(
    ui: &mut egui::Ui,
    state: &mut StudioAppState,
    graph_canvas: &mut GraphCanvasState,
) {
    let snapshot = state.snapshot();
    let rect = ui.max_rect();
    let painter = ui.painter_at(rect);

    painter.rect_filled(rect, 0.0, theme::BACKGROUND);
    graph_canvas::render(ui, state, graph_canvas, rect, &snapshot);
    paint_title(&painter, rect, &snapshot);
    render_view_controls(ui, rect, graph_canvas, &snapshot);
}

fn paint_title(painter: &egui::Painter, rect: Rect, snapshot: &AppSnapshot) {
    let title_pos = rect.left_top() + vec2(24.0, 18.0);
    painter.text(
        title_pos,
        Align2::LEFT_TOP,
        identity::APP_DISPLAY_NAME,
        theme::app_title_font(),
        Color32::from_rgb(230, 235, 240),
    );
    painter.text(
        title_pos + vec2(0.0, 26.0),
        Align2::LEFT_TOP,
        format!("v{} / workspace graph", snapshot.home.app_info.version),
        theme::app_subtitle_font(),
        theme::TEXT_MUTED,
    );
}

fn render_view_controls(
    ui: &mut egui::Ui,
    rect: Rect,
    graph_canvas: &mut GraphCanvasState,
    snapshot: &AppSnapshot,
) {
    egui::Area::new("graph_view_controls".into())
        .fixed_pos(rect.right_top() + vec2(-168.0, 18.0))
        .show(ui.ctx(), |ui| {
            egui::Frame::new()
                .fill(Color32::from_rgb(24, 28, 32))
                .stroke(Stroke::new(1.0, theme::STROKE))
                .corner_radius(6.0)
                .inner_margin(6.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if ui
                            .button(theme::icon("zoom-out", 15.0))
                            .on_hover_text("Zoom out")
                            .clicked()
                        {
                            graph_canvas.zoom_by(0.9, rect.center(), rect);
                        }
                        ui.label(format!("{:.0}%", graph_canvas.view_zoom(rect) * 100.0));
                        if ui
                            .button(theme::icon("zoom-in", 15.0))
                            .on_hover_text("Zoom in")
                            .clicked()
                        {
                            graph_canvas.zoom_by(1.1, rect.center(), rect);
                        }
                        if ui
                            .button(theme::icon("scan", 15.0))
                            .on_hover_text("Fit graph to view")
                            .clicked()
                            && let Some(bounds) = graph_canvas::graph_world_bounds(snapshot)
                        {
                            graph_canvas.fit_world_rect(bounds, rect, 48.0);
                        }
                    });
                });
        });
}
