mod app;
#[cfg(test)]
mod graphics_testing;
mod theme;
mod ui_lab;

use des_app::AppCommand;
use des_core::{StudioResult, identity};
use eframe::egui;

const MIN_WINDOW_WIDTH: f32 = 1080.0;
const MIN_WINDOW_HEIGHT: f32 = 680.0;
const DEFAULT_WINDOW_WIDTH: f32 = 1320.0;
const DEFAULT_WINDOW_HEIGHT: f32 = 780.0;

#[derive(Clone, Debug)]
pub struct NativeLaunchOptions {
    pub title: String,
    pub inner_size: [f32; 2],
    pub debug_overlay: bool,
    pub initial_lab_view: Option<String>,
    pub startup_commands: Vec<AppCommand>,
}

impl Default for NativeLaunchOptions {
    fn default() -> Self {
        Self {
            title: identity::window_title(),
            inner_size: [DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT],
            debug_overlay: false,
            initial_lab_view: None,
            startup_commands: Vec::new(),
        }
    }
}

pub fn run_native(options: NativeLaunchOptions) -> StudioResult<()> {
    let app_options = app::StudioEguiAppOptions {
        debug_overlay: options.debug_overlay,
        initial_lab_view: options.initial_lab_view.clone(),
        startup_commands: options.startup_commands.clone(),
    };
    let native_options = native_options(options);

    eframe::run_native(
        &native_options.title,
        native_options.options,
        Box::new(|creation_context| {
            theme::apply_theme(&creation_context.egui_ctx);
            Ok(Box::new(app::StudioEguiApp::new(app_options)))
        }),
    )
    .map_err(|error| des_core::StudioError::new(error.to_string()))
}

pub fn apply_default_theme(context: &egui::Context) {
    theme::apply_theme(context);
}

struct BuiltNativeOptions {
    title: String,
    options: eframe::NativeOptions,
}

fn native_options(options: NativeLaunchOptions) -> BuiltNativeOptions {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(options.title.clone())
            .with_app_id(identity::APP_INTERNAL_ID)
            .with_inner_size(options.inner_size)
            .with_min_inner_size([MIN_WINDOW_WIDTH, MIN_WINDOW_HEIGHT]),
        persist_window: false,
        centered: true,
        ..Default::default()
    };

    with_screenshot_renderer(BuiltNativeOptions {
        title: options.title,
        options: native_options,
    })
}

#[cfg(feature = "ui-screenshot")]
fn with_screenshot_renderer(mut built: BuiltNativeOptions) -> BuiltNativeOptions {
    built.options.renderer = eframe::Renderer::Glow;
    built
}

#[cfg(not(feature = "ui-screenshot"))]
fn with_screenshot_renderer(built: BuiltNativeOptions) -> BuiltNativeOptions {
    built
}
