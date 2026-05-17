//! Native window shell for DES UI.
//!
//! `des-ui-window` is the product-facing app/window layer that replaces the
//! useful shell responsibilities of `eframe` without taking on egui semantics.

use des_ui_document::DocumentInput;
use des_ui_render::DisplayList;
use des_ui_winit::HostViewport;

#[derive(Clone, Debug, PartialEq)]
pub struct NativeOptions {
    pub title: String,
    pub initial_width: u32,
    pub initial_height: u32,
    pub vsync: bool,
}

impl Default for NativeOptions {
    fn default() -> Self {
        Self {
            title: "Data Engine Studio".to_owned(),
            initial_width: 1280,
            initial_height: 800,
            vsync: true,
        }
    }
}

pub trait WindowApp {
    fn update(&mut self, frame: &mut AppFrame);
}

#[derive(Debug)]
pub struct AppFrame {
    viewport: HostViewport,
    input: DocumentInput,
    display_list: DisplayList,
    repaint_requested: bool,
}

impl AppFrame {
    pub fn new(viewport: HostViewport, input: DocumentInput) -> Self {
        Self::with_display_list(viewport, input, DisplayList::new())
    }

    pub fn with_display_list(
        viewport: HostViewport,
        input: DocumentInput,
        display_list: DisplayList,
    ) -> Self {
        Self {
            viewport,
            input,
            display_list,
            repaint_requested: false,
        }
    }

    pub fn viewport(&self) -> HostViewport {
        self.viewport
    }

    pub fn input(&self) -> &DocumentInput {
        &self.input
    }

    pub fn display_list(&self) -> &DisplayList {
        &self.display_list
    }

    pub fn display_list_mut(&mut self) -> &mut DisplayList {
        &mut self.display_list
    }

    pub fn request_repaint(&mut self) {
        self.repaint_requested = true;
    }

    pub fn repaint_requested(&self) -> bool {
        self.repaint_requested
    }
}

#[cfg(test)]
mod tests {
    use des_ui_document::{DocumentInput, Point};
    use des_ui_winit::HostViewport;

    use crate::{AppFrame, NativeOptions, WindowApp};

    #[test]
    fn native_options_have_desktop_friendly_defaults() {
        let options = NativeOptions::default();

        assert_eq!(options.title, "Data Engine Studio");
        assert_eq!(options.initial_width, 1280);
        assert_eq!(options.initial_height, 800);
        assert!(options.vsync);
    }

    #[test]
    fn app_frame_exposes_viewport_input_and_repaint_state() {
        let input = DocumentInput {
            pointer: None,
            scroll_delta: Point::new(0.0, -24.0),
        };
        let mut frame = AppFrame::new(HostViewport::new(1600, 1000, 2.0), input);

        assert_eq!(frame.viewport().logical_size().width, 800.0);
        assert_eq!(frame.input().scroll_delta, Point::new(0.0, -24.0));
        assert!(!frame.repaint_requested());

        frame.request_repaint();
        assert!(frame.repaint_requested());
    }

    #[test]
    fn window_app_trait_can_be_driven_without_a_native_window() {
        struct CounterApp {
            updates: usize,
        }

        impl WindowApp for CounterApp {
            fn update(&mut self, frame: &mut AppFrame) {
                self.updates += 1;
                frame.request_repaint();
            }
        }

        let input = DocumentInput::default();
        let mut frame = AppFrame::new(HostViewport::new(800, 600, 1.0), input);
        let mut app = CounterApp { updates: 0 };

        app.update(&mut frame);

        assert_eq!(app.updates, 1);
        assert!(frame.repaint_requested());
    }
}
