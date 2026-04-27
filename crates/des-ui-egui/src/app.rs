use crate::{graph_canvas::GraphCanvasState, shell};
use des_app::StudioAppState;
use eframe::egui;

#[derive(Default)]
pub(crate) struct StudioEguiApp {
    state: StudioAppState,
    graph_canvas: GraphCanvasState,
}

impl eframe::App for StudioEguiApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        shell::render(ui, &mut self.state, &mut self.graph_canvas);
    }
}
