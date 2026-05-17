//! Native window shell for DES UI.
//!
//! `des-ui-window` is the product-facing app/window layer that replaces the
//! useful shell responsibilities of `eframe` without taking on egui semantics.

use des_ui_document::{DocumentInput, DocumentOutput};
use des_ui_render::{DisplayList, plan_paint};
use des_ui_wgpu::{DisplayListRenderer, RenderOptions, RenderPlan};
use des_ui_winit::HostViewport;

#[derive(Clone, Debug, PartialEq)]
pub struct NativeOptions {
    pub title: String,
    pub initial_width: u32,
    pub initial_height: u32,
    pub vsync: bool,
    pub render_options: RenderOptions,
}

impl Default for NativeOptions {
    fn default() -> Self {
        Self {
            title: "Data Engine Studio".to_owned(),
            initial_width: 1280,
            initial_height: 800,
            vsync: true,
            render_options: RenderOptions::default(),
        }
    }
}

impl NativeOptions {
    pub fn initial_viewport(&self, scale_factor: f64) -> HostViewport {
        HostViewport::new(self.initial_width, self.initial_height, scale_factor)
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
    close_requested: bool,
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
            close_requested: false,
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

    pub fn set_document_output(&mut self, output: &DocumentOutput) {
        self.display_list = plan_paint(output);
    }

    pub fn set_display_list(&mut self, display_list: DisplayList) {
        self.display_list = display_list;
    }

    pub fn request_repaint(&mut self) {
        self.repaint_requested = true;
    }

    pub fn repaint_requested(&self) -> bool {
        self.repaint_requested
    }

    pub fn request_close(&mut self) {
        self.close_requested = true;
    }

    pub fn close_requested(&self) -> bool {
        self.close_requested
    }

    pub fn into_output(self, render_options: RenderOptions) -> FrameOutput {
        let render_plan = DisplayListRenderer::new(render_options).build_plan(&self.display_list);
        FrameOutput {
            display_list: self.display_list,
            render_plan,
            repaint_requested: self.repaint_requested,
            close_requested: self.close_requested,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FrameOutput {
    pub display_list: DisplayList,
    pub render_plan: RenderPlan,
    pub repaint_requested: bool,
    pub close_requested: bool,
}

#[cfg(test)]
mod tests {
    use des_ui_document::{
        Color, CornerRadii, Document, DocumentEngine, DocumentInput, Element, ElementId, Point,
        Rect, Size, Style, StyleSelector, StyleSheet,
    };
    use des_ui_render::{FillRectPaint, PaintCommand};
    use des_ui_wgpu::ClearColor;
    use des_ui_winit::HostViewport;

    use crate::{AppFrame, NativeOptions, WindowApp};

    #[test]
    fn native_options_have_desktop_friendly_defaults() {
        let options = NativeOptions::default();

        assert_eq!(options.title, "Data Engine Studio");
        assert_eq!(options.initial_width, 1280);
        assert_eq!(options.initial_height, 800);
        assert!(options.vsync);
        assert_eq!(
            options.initial_viewport(2.0),
            HostViewport::new(1280, 800, 2.0)
        );
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
    fn app_frame_output_builds_render_plan_from_display_list() {
        let input = DocumentInput::default();
        let mut frame = AppFrame::new(HostViewport::new(800, 600, 1.0), input);
        frame
            .display_list_mut()
            .push(PaintCommand::FillRect(FillRectPaint {
                element_id: ElementId::new("panel"),
                rect: Rect::new(0.0, 0.0, 80.0, 40.0),
                radius: CornerRadii::ZERO,
                color: Color::rgb(20, 30, 40),
            }));
        frame.request_repaint();

        let output = frame.into_output(des_ui_wgpu::RenderOptions {
            clear_color: ClearColor::rgb(250, 249, 247),
            ..des_ui_wgpu::RenderOptions::default()
        });

        assert!(output.repaint_requested);
        assert!(!output.close_requested);
        assert_eq!(output.display_list.commands.len(), 1);
        assert_eq!(
            output.render_plan.clear_color,
            ClearColor::rgb(250, 249, 247)
        );
        assert_eq!(output.render_plan.batches.len(), 1);
        assert_eq!(output.render_plan.batches[0].mesh.indices.len(), 6);
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

    #[test]
    fn app_frame_accepts_document_output_as_render_source() {
        let mut document = Document::build(Size::new(200.0, 120.0), |ui| {
            ui.div("panel").children(|ui| {
                ui.text("label", "Native document");
            });
        });
        let stylesheet = StyleSheet::new()
            .rule(
                StyleSelector::Id("panel".into()),
                Style::default()
                    .size(100.0, 50.0)
                    .background(Color::rgb(20, 30, 40)),
            )
            .rule(
                StyleSelector::Element(Element::Text),
                Style::default().size(80.0, 20.0),
            );
        let output = DocumentEngine::default().update(&mut document, &stylesheet);
        let mut frame = AppFrame::new(HostViewport::new(800, 600, 1.0), DocumentInput::default());

        frame.set_document_output(&output);
        let output = frame.into_output(des_ui_wgpu::RenderOptions::default());

        assert!(!output.render_plan.batches.is_empty());
        assert_eq!(output.render_plan.text_batches.len(), 1);
        assert_eq!(
            output.render_plan.text_batches[0].text.text,
            "Native document"
        );
    }
}
