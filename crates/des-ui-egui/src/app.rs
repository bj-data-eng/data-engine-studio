use crate::ui_lab::UiLabState;
use des_app::{AppCommand, StudioAppState};
use eframe::egui;

pub(crate) struct StudioEguiApp {
    _state: StudioAppState,
    ui_lab: UiLabState,
    debug_overlay: bool,
}

pub(crate) struct StudioEguiAppOptions {
    pub(crate) debug_overlay: bool,
    pub(crate) startup_commands: Vec<AppCommand>,
}

impl StudioEguiApp {
    pub(crate) fn new(options: StudioEguiAppOptions) -> Self {
        let mut state = StudioAppState::default();
        for command in options.startup_commands {
            state.dispatch(command);
        }

        Self {
            _state: state,
            ui_lab: UiLabState::default(),
            debug_overlay: options.debug_overlay,
        }
    }
}

impl eframe::App for StudioEguiApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.ui_lab.render(ui, self.debug_overlay);
    }
}
