use crate::{graph_canvas::GraphCanvasState, graph_view};
use des_app::StudioAppState;
use eframe::egui;

pub(crate) fn render(
    ui: &mut egui::Ui,
    state: &mut StudioAppState,
    graph_canvas: &mut GraphCanvasState,
) {
    egui::CentralPanel::default()
        .frame(egui::Frame::new())
        .show_inside(ui, |ui| {
            graph_view::render(ui, state, graph_canvas);
        });
}
