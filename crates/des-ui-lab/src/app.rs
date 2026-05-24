use crate::ui_lab::{NativePointerMoveFilter, UiLabState};
use des_app::{AppCommand, StudioAppState};
use eframe::egui;
use std::sync::{Arc, Mutex};

pub(crate) struct StudioLabApp {
    _state: StudioAppState,
    ui_lab: UiLabState,
    debug_overlay: bool,
}

pub(crate) struct StudioLabAppOptions {
    pub(crate) debug_overlay: bool,
    pub(crate) initial_lab_view: Option<String>,
    pub(crate) initial_lab_scroll: Option<[f32; 2]>,
    pub(crate) startup_commands: Vec<AppCommand>,
    pub(crate) pointer_move_filter: Option<Arc<Mutex<NativePointerMoveFilter>>>,
}

impl StudioLabApp {
    pub(crate) fn new(options: StudioLabAppOptions) -> Self {
        let mut state = StudioAppState::default();
        for command in options.startup_commands {
            state.dispatch(command);
        }

        let mut ui_lab = match options.initial_lab_scroll {
            Some(scroll) => UiLabState::new_with_stage_scroll(
                options.initial_lab_view.as_deref(),
                Some(des_document::Point::new(scroll[0], scroll[1])),
            ),
            None => UiLabState::new(options.initial_lab_view.as_deref()),
        };
        if let Some(filter) = options.pointer_move_filter {
            ui_lab.set_pointer_move_filter(filter);
        }

        Self {
            _state: state,
            ui_lab,
            debug_overlay: options.debug_overlay,
        }
    }
}

impl eframe::App for StudioLabApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.ui_lab.render(ui, self.debug_overlay);
    }
}
