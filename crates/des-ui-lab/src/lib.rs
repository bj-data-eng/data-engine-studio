mod app;
#[cfg(test)]
mod graphics_testing;
mod ui_lab;

use des_app::AppCommand;
use des_core::{StudioResult, identity};
use eframe::egui;
use std::sync::{Arc, Mutex};
use ui_lab::NativePointerMoveFilter;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
};

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
    pub initial_lab_scroll: Option<[f32; 2]>,
    pub startup_commands: Vec<AppCommand>,
}

impl Default for NativeLaunchOptions {
    fn default() -> Self {
        Self {
            title: identity::window_title(),
            inner_size: [DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT],
            debug_overlay: false,
            initial_lab_view: None,
            initial_lab_scroll: None,
            startup_commands: Vec::new(),
        }
    }
}

pub fn run_native(options: NativeLaunchOptions) -> StudioResult<()> {
    let pointer_move_filter = Arc::new(Mutex::new(NativePointerMoveFilter::default()));
    let app_options = app::StudioLabAppOptions {
        debug_overlay: options.debug_overlay,
        initial_lab_view: options.initial_lab_view.clone(),
        initial_lab_scroll: options.initial_lab_scroll,
        startup_commands: options.startup_commands.clone(),
        pointer_move_filter: Some(pointer_move_filter.clone()),
    };
    let native_options = native_options(options);
    let event_loop = EventLoop::<eframe::UserEvent>::with_user_event()
        .build()
        .map_err(|error| des_core::StudioError::new(error.to_string()))?;
    let mut app = FilteredEframeApp {
        inner: eframe::create_native(
            &native_options.title,
            native_options.options,
            Box::new(|creation_context| {
                des_egui::apply_default_host_configuration(&creation_context.egui_ctx);
                Ok(Box::new(app::StudioLabApp::new(app_options)))
            }),
            &event_loop,
        ),
        pointer_move_filter,
    };

    event_loop
        .run_app(&mut app)
        .map_err(|error| des_core::StudioError::new(error.to_string()))
}

struct FilteredEframeApp<'app> {
    inner: eframe::EframeWinitApplication<'app>,
    pointer_move_filter: Arc<Mutex<NativePointerMoveFilter>>,
}

impl ApplicationHandler<eframe::UserEvent> for FilteredEframeApp<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.inner.resumed(event_loop);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        if let WindowEvent::CursorMoved { position, .. } = &event
            && self
                .pointer_move_filter
                .lock()
                .expect("native pointer filter lock is healthy")
                .should_skip_cursor_moved(position.x, position.y)
        {
            return;
        }
        self.inner.window_event(event_loop, window_id, event);
    }

    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: winit::event::StartCause) {
        self.inner.new_events(event_loop, cause);
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: eframe::UserEvent) {
        self.inner.user_event(event_loop, event);
    }

    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        self.inner.device_event(event_loop, device_id, event);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.inner.about_to_wait(event_loop);
    }

    fn suspended(&mut self, event_loop: &ActiveEventLoop) {
        self.inner.suspended(event_loop);
    }

    fn exiting(&mut self, event_loop: &ActiveEventLoop) {
        self.inner.exiting(event_loop);
    }

    fn memory_warning(&mut self, event_loop: &ActiveEventLoop) {
        self.inner.memory_warning(event_loop);
    }
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
