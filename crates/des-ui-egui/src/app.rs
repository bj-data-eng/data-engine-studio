use crate::{graph_canvas::GraphCanvasState, shell, workspace_catalog::WorkspaceCatalogState};
use des_app::{AppCommand, StudioAppState};
use eframe::egui;

pub(crate) struct StudioEguiApp {
    state: StudioAppState,
    graph_canvas: GraphCanvasState,
    workspace_catalog: WorkspaceCatalogState,
    debug_overlay: bool,
}

pub(crate) struct StudioEguiAppOptions {
    pub(crate) debug_overlay: bool,
    pub(crate) initial_scene_rect: Option<egui::Rect>,
    pub(crate) startup_commands: Vec<AppCommand>,
}

impl StudioEguiApp {
    pub(crate) fn new(options: StudioEguiAppOptions) -> Self {
        let mut state = StudioAppState::default();
        for command in options.startup_commands {
            state.dispatch(command);
        }

        let mut graph_canvas = GraphCanvasState::default();
        if let Some(rect) = options.initial_scene_rect {
            graph_canvas.set_scene_rect(rect);
        }

        Self {
            state,
            graph_canvas,
            workspace_catalog: WorkspaceCatalogState::default(),
            debug_overlay: options.debug_overlay,
        }
    }
}

impl eframe::App for StudioEguiApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        shell::render(
            ui,
            &mut self.state,
            &mut self.graph_canvas,
            &mut self.workspace_catalog,
            self.debug_overlay,
        );
    }
}
