//! `winit` adapter for DES UI input and window state.
//!
//! This crate is the DES equivalent of `egui-winit`: it translates host window
//! state and raw `winit` input into the renderer-neutral document contracts.

use des_ui_document::{DocumentInput, Point, PointerInput, Size};
use winit::event::{ElementState, MouseButton as WinitMouseButton, MouseScrollDelta, WindowEvent};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HostViewport {
    pub physical_width: u32,
    pub physical_height: u32,
    pub scale_factor: f64,
}

impl HostViewport {
    pub fn new(physical_width: u32, physical_height: u32, scale_factor: f64) -> Self {
        Self {
            physical_width,
            physical_height,
            scale_factor: scale_factor.max(0.000_001),
        }
    }

    pub fn logical_size(self) -> Size {
        Size::new(
            self.physical_width as f32 / self.scale_factor as f32,
            self.physical_height as f32 / self.scale_factor as f32,
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PointerButton {
    Primary,
    Secondary,
    Middle,
    Other,
}

impl From<WinitMouseButton> for PointerButton {
    fn from(button: WinitMouseButton) -> Self {
        match button {
            WinitMouseButton::Left => Self::Primary,
            WinitMouseButton::Right => Self::Secondary,
            WinitMouseButton::Middle => Self::Middle,
            WinitMouseButton::Back | WinitMouseButton::Forward | WinitMouseButton::Other(_) => {
                Self::Other
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PointerPhase {
    Pressed,
    Released,
}

impl From<ElementState> for PointerPhase {
    fn from(state: ElementState) -> Self {
        match state {
            ElementState::Pressed => Self::Pressed,
            ElementState::Released => Self::Released,
        }
    }
}

#[derive(Clone, Debug)]
pub struct WinitInputTranslator {
    viewport: HostViewport,
    pointer_position: Option<Point>,
    previous_pointer_position: Option<Point>,
    primary_down: bool,
    primary_pressed: bool,
    primary_clicked: bool,
    primary_click_count: u8,
    secondary_clicked: bool,
    scroll_delta: Point,
    time_seconds: f64,
}

impl Default for WinitInputTranslator {
    fn default() -> Self {
        Self::new()
    }
}

impl WinitInputTranslator {
    pub fn new() -> Self {
        Self {
            viewport: HostViewport::new(0, 0, 1.0),
            pointer_position: None,
            previous_pointer_position: None,
            primary_down: false,
            primary_pressed: false,
            primary_clicked: false,
            primary_click_count: 0,
            secondary_clicked: false,
            scroll_delta: Point::ZERO,
            time_seconds: 0.0,
        }
    }

    pub fn viewport(&self) -> HostViewport {
        self.viewport
    }

    pub fn set_viewport(&mut self, viewport: HostViewport) {
        self.viewport = viewport;
    }

    pub fn handle_window_event(&mut self, event: &WindowEvent, time_seconds: f64) {
        self.time_seconds = time_seconds;
        match event {
            WindowEvent::Resized(size) => {
                self.viewport.physical_width = size.width;
                self.viewport.physical_height = size.height;
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.viewport.scale_factor = (*scale_factor).max(0.000_001);
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_moved(
                    Point::new(
                        position.x as f32 / self.viewport.scale_factor as f32,
                        position.y as f32 / self.viewport.scale_factor as f32,
                    ),
                    time_seconds,
                );
            }
            WindowEvent::CursorLeft { .. } => {
                self.previous_pointer_position = self.pointer_position;
                self.pointer_position = None;
            }
            WindowEvent::MouseInput { state, button, .. } => {
                self.pointer_button((*button).into(), (*state).into(), time_seconds);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                self.scroll(scroll_delta(*delta, self.viewport.scale_factor));
            }
            _ => {}
        }
    }

    pub fn cursor_moved(&mut self, position: Point, time_seconds: f64) {
        self.previous_pointer_position = self.pointer_position;
        self.pointer_position = Some(position);
        self.time_seconds = time_seconds;
    }

    pub fn pointer_button(
        &mut self,
        button: PointerButton,
        phase: PointerPhase,
        time_seconds: f64,
    ) {
        self.time_seconds = time_seconds;
        match (button, phase) {
            (PointerButton::Primary, PointerPhase::Pressed) => {
                self.primary_down = true;
                self.primary_pressed = true;
            }
            (PointerButton::Primary, PointerPhase::Released) => {
                self.primary_down = false;
                self.primary_clicked = true;
                self.primary_click_count = 1;
            }
            (PointerButton::Secondary, PointerPhase::Released) => {
                self.secondary_clicked = true;
            }
            _ => {}
        }
    }

    pub fn scroll(&mut self, delta: Point) {
        self.scroll_delta.x += delta.x;
        self.scroll_delta.y += delta.y;
    }

    pub fn frame_input(&mut self) -> DocumentInput {
        let pointer = self.pointer_position.map(|position| {
            let previous = self.previous_pointer_position.unwrap_or(position);
            PointerInput {
                position,
                primary_delta: Point::new(position.x - previous.x, position.y - previous.y),
                primary_down: self.primary_down,
                primary_pressed: self.primary_pressed,
                primary_clicked: self.primary_clicked,
                primary_click_count: self.primary_click_count,
                secondary_clicked: self.secondary_clicked,
                time_seconds: self.time_seconds,
            }
        });
        let input = DocumentInput {
            pointer,
            scroll_delta: self.scroll_delta,
        };
        self.primary_pressed = false;
        self.primary_clicked = false;
        self.primary_click_count = 0;
        self.secondary_clicked = false;
        self.scroll_delta = Point::ZERO;
        self.previous_pointer_position = self.pointer_position;
        input
    }
}

fn scroll_delta(delta: MouseScrollDelta, scale_factor: f64) -> Point {
    match delta {
        MouseScrollDelta::LineDelta(x, y) => Point::new(x * 40.0, y * 40.0),
        MouseScrollDelta::PixelDelta(position) => Point::new(
            position.x as f32 / scale_factor as f32,
            position.y as f32 / scale_factor as f32,
        ),
    }
}

#[cfg(test)]
mod tests {
    use des_ui_document::Point;

    use crate::{HostViewport, PointerButton, PointerPhase, WinitInputTranslator};

    #[test]
    fn viewport_reports_logical_size_from_physical_pixels() {
        let viewport = HostViewport::new(1600, 900, 2.0);

        assert_eq!(viewport.physical_width, 1600);
        assert_eq!(viewport.physical_height, 900);
        assert_eq!(viewport.scale_factor, 2.0);
        assert_eq!(viewport.logical_size().width, 800.0);
        assert_eq!(viewport.logical_size().height, 450.0);
    }

    #[test]
    fn pointer_events_accumulate_document_input_for_one_frame() {
        let mut input = WinitInputTranslator::new();
        input.cursor_moved(Point::new(10.0, 12.0), 0.10);
        input.pointer_button(PointerButton::Primary, PointerPhase::Pressed, 0.20);
        input.pointer_button(PointerButton::Primary, PointerPhase::Released, 0.25);
        input.scroll(Point::new(2.0, -8.0));

        let frame = input.frame_input();
        let pointer = frame.pointer.expect("pointer position should be known");
        assert_eq!(pointer.position, Point::new(10.0, 12.0));
        assert!(!pointer.primary_down);
        assert!(pointer.primary_pressed);
        assert!(pointer.primary_clicked);
        assert_eq!(pointer.primary_click_count, 1);
        assert_eq!(pointer.time_seconds, 0.25);
        assert_eq!(frame.scroll_delta, Point::new(2.0, -8.0));

        let next = input.frame_input();
        let pointer = next.pointer.expect("pointer position should be retained");
        assert!(!pointer.primary_pressed);
        assert!(!pointer.primary_clicked);
        assert_eq!(pointer.primary_click_count, 0);
        assert_eq!(next.scroll_delta, Point::ZERO);
    }
}
