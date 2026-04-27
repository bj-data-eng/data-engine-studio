mod app;
mod graph_canvas;
mod graph_view;
mod shell;
mod theme;

use des_core::{StudioResult, identity};
use eframe::egui;

const MIN_WINDOW_WIDTH: f32 = 1080.0;
const MIN_WINDOW_HEIGHT: f32 = 680.0;

#[derive(Clone, Debug)]
pub struct NativeLaunchOptions {
    pub title: String,
}

impl Default for NativeLaunchOptions {
    fn default() -> Self {
        Self {
            title: identity::window_title(),
        }
    }
}

pub fn run_native(options: NativeLaunchOptions) -> StudioResult<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(options.title.clone())
            .with_app_id(identity::APP_INTERNAL_ID)
            .with_inner_size([1320.0, 780.0])
            .with_min_inner_size([MIN_WINDOW_WIDTH, MIN_WINDOW_HEIGHT]),
        persist_window: false,
        centered: true,
        ..Default::default()
    };

    eframe::run_native(
        &options.title,
        native_options,
        Box::new(|creation_context| {
            theme::apply_visuals(&creation_context.egui_ctx);
            Ok(Box::<app::StudioEguiApp>::default())
        }),
    )
    .map_err(|error| des_core::StudioError::new(error.to_string()))
}
