//! Native window shell for DES UI.
//!
//! `des-ui-window` is the product-facing app/window layer that replaces the
//! useful shell responsibilities of `eframe` without taking on egui semantics.

pub mod demo;

use des_ui_document::{DocumentInput, DocumentOutput};
use des_ui_render::{DisplayList, plan_paint};
use des_ui_wgpu::{
    DisplayListRenderer, GpuRenderer, PhysicalRenderSize, RenderOptions, RenderPlan,
};
use des_ui_winit::{HostViewport, WindowSignal, WinitInputTranslator};
use std::{error, fmt, sync::Arc, time::Instant};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

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
pub enum NativeRunError {
    EventLoop(winit::error::EventLoopError),
    Window(winit::error::OsError),
    Renderer(des_ui_wgpu::RendererError),
}

impl fmt::Display for NativeRunError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EventLoop(error) => write!(f, "native event loop failed: {error}"),
            Self::Window(error) => write!(f, "native window failed: {error}"),
            Self::Renderer(error) => write!(f, "native renderer failed: {error}"),
        }
    }
}

impl error::Error for NativeRunError {}

impl From<winit::error::EventLoopError> for NativeRunError {
    fn from(error: winit::error::EventLoopError) -> Self {
        Self::EventLoop(error)
    }
}

impl From<winit::error::OsError> for NativeRunError {
    fn from(error: winit::error::OsError) -> Self {
        Self::Window(error)
    }
}

impl From<des_ui_wgpu::RendererError> for NativeRunError {
    fn from(error: des_ui_wgpu::RendererError) -> Self {
        Self::Renderer(error)
    }
}

pub fn run_native<A>(options: NativeOptions, app: A) -> Result<(), NativeRunError>
where
    A: WindowApp + 'static,
{
    let event_loop = EventLoop::new()?;
    let mut shell = NativeShell::new(options, app);
    event_loop.run_app(&mut shell)?;
    Ok(())
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
        let render_plan = DisplayListRenderer::new(render_options)
            .with_pixels_per_point(self.viewport.scale_factor as f32)
            .build_plan(&self.display_list);
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

struct NativeShell<A> {
    options: NativeOptions,
    app: A,
    start: Instant,
    window: Option<Arc<Window>>,
    renderer: Option<GpuRenderer<'static>>,
    input: WinitInputTranslator,
}

impl<A> NativeShell<A> {
    fn new(options: NativeOptions, app: A) -> Self {
        Self {
            options,
            app,
            start: Instant::now(),
            window: None,
            renderer: None,
            input: WinitInputTranslator::new(),
        }
    }

    fn create_window(&mut self, event_loop: &ActiveEventLoop) -> Result<(), NativeRunError> {
        if self.window.is_some() {
            return Ok(());
        }
        let window = Arc::new(
            event_loop.create_window(
                Window::default_attributes()
                    .with_title(self.options.title.clone())
                    .with_resizable(true)
                    .with_inner_size(PhysicalSize::new(
                        self.options.initial_width,
                        self.options.initial_height,
                    )),
            )?,
        );
        let size = window.inner_size();
        let viewport = HostViewport::new(size.width, size.height, window.scale_factor());
        self.input.set_viewport(viewport);
        let renderer = pollster::block_on(GpuRenderer::new(
            window.clone(),
            render_size_from_viewport(viewport),
            self.options.render_options,
        ))?;
        self.renderer = Some(renderer);
        self.window = Some(window);
        Ok(())
    }

    fn redraw(&mut self, event_loop: &ActiveEventLoop) -> Result<(), NativeRunError>
    where
        A: WindowApp,
    {
        let Some(window) = self.window.as_ref() else {
            return Ok(());
        };
        let Some(renderer) = self.renderer.as_mut() else {
            return Ok(());
        };
        let input = self.input.frame_input();
        let mut frame = AppFrame::new(self.input.viewport(), input);
        self.app.update(&mut frame);
        let output = frame.into_output(self.options.render_options);
        renderer.render_plan(&output.render_plan)?;
        if output.close_requested {
            event_loop.exit();
        } else if output.repaint_requested {
            window.request_redraw();
        }
        Ok(())
    }

    fn request_redraw(&self) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    fn time_seconds(&self) -> f64 {
        self.start.elapsed().as_secs_f64()
    }
}

impl<A> ApplicationHandler for NativeShell<A>
where
    A: WindowApp,
{
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let Err(error) = self.create_window(event_loop) {
            eprintln!("{error}");
            event_loop.exit();
            return;
        }
        self.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let signal = self.input.handle_window_event(&event, self.time_seconds());
        match signal {
            WindowSignal::CloseRequested => event_loop.exit(),
            WindowSignal::RedrawRequested => {
                if let Err(error) = self.redraw(event_loop) {
                    eprintln!("{error}");
                    event_loop.exit();
                }
            }
            WindowSignal::Resized(viewport) => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(render_size_from_viewport(viewport));
                }
                self.request_redraw();
            }
            WindowSignal::None => {
                self.request_redraw();
            }
        }
    }
}

fn render_size_from_viewport(viewport: HostViewport) -> PhysicalRenderSize {
    PhysicalRenderSize::new(
        viewport.physical_width,
        viewport.physical_height,
        viewport.scale_factor,
    )
}

#[cfg(test)]
mod tests {
    use des_ui_document::{
        Color, CornerRadii, Document, DocumentEngine, DocumentEventKind, DocumentInput, Element,
        ElementId, Point, Rect, Size, Style, StyleSelector, StyleSheet,
    };
    use des_ui_render::{FillRectPaint, PaintCommand, content_rect};
    use des_ui_wgpu::ClearColor;
    use des_ui_winit::HostViewport;

    use crate::{AppFrame, FrameOutput, NativeOptions, WindowApp};

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
        assert!(!output.render_plan.batches[0].mesh.indices.is_empty());
    }

    #[test]
    fn app_frame_output_uses_viewport_scale_for_shape_tessellation() {
        fn output_for_scale(scale_factor: f64) -> FrameOutput {
            let mut frame = AppFrame::new(
                HostViewport::new(800, 600, scale_factor),
                DocumentInput::default(),
            );
            frame
                .display_list_mut()
                .push(PaintCommand::FillRect(FillRectPaint {
                    element_id: ElementId::new("panel"),
                    rect: Rect::new(10.0, 20.0, 80.0, 40.0),
                    radius: CornerRadii::ZERO,
                    color: Color::rgb(20, 30, 40),
                }));
            frame.into_output(des_ui_wgpu::RenderOptions::default())
        }

        let low_density = output_for_scale(1.0);
        let high_density = output_for_scale(2.0);
        let min_x = |output: &FrameOutput| {
            output.render_plan.batches[0]
                .mesh
                .vertices
                .iter()
                .map(|vertex| vertex.position[0])
                .fold(f32::INFINITY, f32::min)
        };

        assert!(min_x(&high_density) > min_x(&low_density));
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

    #[test]
    fn render_size_uses_host_viewport_physical_extent_and_scale() {
        let size = super::render_size_from_viewport(HostViewport::new(1600, 900, 2.0));

        assert_eq!(size.width, 1600);
        assert_eq!(size.height, 900);
        assert_eq!(size.logical_width(), 800.0);
        assert_eq!(size.logical_height(), 450.0);
    }

    #[test]
    fn native_document_demo_renders_document_layout_into_frame() {
        let mut app = crate::demo::NativeDocumentDemo::new();
        let mut frame = AppFrame::new(HostViewport::new(980, 680, 1.0), DocumentInput::default());

        app.update(&mut frame);
        let document_output = app.last_output().unwrap().clone();
        let frame_output = frame.into_output(des_ui_wgpu::RenderOptions::default());
        let index_count = frame_output
            .render_plan
            .batches
            .iter()
            .map(|batch| batch.mesh.indices.len())
            .sum::<usize>();

        assert!(app.last_output().is_some());
        assert!(index_count > 48);
        assert!(frame_output.render_plan.text_batches.len() >= 4);
        assert!(
            frame_output
                .display_list
                .commands
                .iter()
                .any(|command| matches!(command, PaintCommand::ShadowRect(_))),
            "native demo should exercise the epaint blurred shadow path"
        );
        assert!(
            frame_output
                .render_plan
                .text_batches
                .iter()
                .any(|batch| batch.text.text.contains("π"))
        );
        assert_eq!(document_output.layout.rect.size, Size::new(980.0, 680.0));
    }

    #[test]
    fn native_document_demo_clips_overflow_text_in_render_plan() {
        let mut app = crate::demo::NativeDocumentDemo::new();
        let mut frame = AppFrame::new(HostViewport::new(980, 680, 1.0), DocumentInput::default());

        app.update(&mut frame);

        let output = app.last_output().unwrap();
        let clip_frame = output.layout.find("native-clip-window").unwrap();
        let content = content_rect(clip_frame);
        let expected_clip = Rect::new(
            clip_frame.rect.origin.x,
            content.origin.y,
            clip_frame.rect.size.width,
            content.size.height,
        );
        let frame_output = frame.into_output(des_ui_wgpu::RenderOptions::default());
        let clipped_text = frame_output
            .render_plan
            .text_batches
            .iter()
            .find(|batch| batch.text.element_id.as_str() == "native-clip-copy")
            .expect("native clipped text should be rendered from the real document");

        assert_eq!(clipped_text.clip, Some(expected_clip));
        assert!(
            clipped_text.text.rect.size.height > expected_clip.size.height,
            "the text item should overflow the container and rely on native scissor clipping"
        );
    }

    #[test]
    fn native_document_demo_clicks_flow_through_document_engine() {
        let mut app = crate::demo::NativeDocumentDemo::new();
        let mut frame = AppFrame::new(HostViewport::new(980, 680, 1.0), DocumentInput::default());
        app.update(&mut frame);
        let action_rect = app
            .last_output()
            .unwrap()
            .layout
            .find("native-action")
            .unwrap()
            .rect;
        let click_position = Point::new(
            action_rect.origin.x + action_rect.size.width * 0.5,
            action_rect.origin.y + action_rect.size.height * 0.5,
        );
        let mut click_frame = AppFrame::new(
            HostViewport::new(980, 680, 1.0),
            crate::demo::click_input_at(click_position, 0.10),
        );

        app.update(&mut click_frame);

        assert_eq!(app.clicks(), 1);
        assert!(click_frame.repaint_requested());
        assert!(
            click_frame
                .display_list()
                .commands
                .iter()
                .any(|command| matches!(
                    command,
                    PaintCommand::Text(text)
                        if text.element_id.as_str() == "native-action-label"
                            && text.text == "Clicks: 1"
                )),
            "the click frame should render the document state produced by the click"
        );
    }

    #[test]
    fn native_document_demo_reflows_with_viewport_size() {
        let mut app = crate::demo::NativeDocumentDemo::new();

        let mut wide_frame =
            AppFrame::new(HostViewport::new(980, 680, 1.0), DocumentInput::default());
        app.update(&mut wide_frame);
        let wide_output = app.last_output().unwrap().clone();

        let mut narrow_frame =
            AppFrame::new(HostViewport::new(520, 680, 1.0), DocumentInput::default());
        app.update(&mut narrow_frame);
        let narrow_output = app.last_output().unwrap();

        let wide_card = wide_output.layout.find("native-card").unwrap().rect;
        let narrow_card = narrow_output.layout.find("native-card").unwrap().rect;
        let wide_tile = wide_output.layout.find("tile-one").unwrap().rect;
        let narrow_tile = narrow_output.layout.find("tile-one").unwrap().rect;

        assert_eq!(wide_output.layout.rect.size, Size::new(980.0, 680.0));
        assert_eq!(narrow_output.layout.rect.size, Size::new(520.0, 680.0));
        assert!(
            narrow_card.size.width < wide_card.size.width,
            "the document shell should resize from the host viewport instead of freezing layout"
        );
        assert!(
            wide_tile.size.width < wide_card.size.width * 0.6,
            "wide viewport should keep the demo tiles in the two-column flex row"
        );
        assert!(
            narrow_tile.size.width > narrow_card.size.width * 0.8,
            "narrow viewport should apply the viewport breakpoint and stack tiles full width"
        );
    }

    #[test]
    fn native_document_demo_hover_animates_button_into_render_plan() {
        let mut app = crate::demo::NativeDocumentDemo::new();
        let mut frame = AppFrame::new(HostViewport::new(980, 680, 1.0), DocumentInput::default());
        app.update(&mut frame);
        let base_rect = app
            .last_output()
            .unwrap()
            .layout
            .find("native-action")
            .unwrap()
            .rect;
        let base_output = frame.into_output(des_ui_wgpu::RenderOptions::default());
        let base_color = fill_rect_color(&base_output, "native-action")
            .expect("native action should paint a base fill");
        let hover_position = Point::new(
            base_rect.origin.x + base_rect.size.width * 0.5,
            base_rect.origin.y + base_rect.size.height * 0.5,
        );
        let mut hover_frame = AppFrame::new(
            HostViewport::new(980, 680, 1.0),
            crate::demo::hover_input_at(hover_position, 0.20),
        );

        app.update(&mut hover_frame);

        let hover_output = app.last_output().unwrap();
        let hover_rect = hover_output.layout.find("native-action").unwrap().rect;
        assert!(hover_output.animating);
        assert!(hover_output.metrics.animation_changed_layout);
        assert!(hover_output.metrics.animation_changed_paint);
        assert!(hover_frame.repaint_requested());
        assert!(hover_rect.size.width > base_rect.size.width);

        let frame_output = hover_frame.into_output(des_ui_wgpu::RenderOptions::default());
        let hover_color = fill_rect_color(&frame_output, "native-action")
            .expect("native action should paint a hover fill");
        assert_ne!(
            hover_color, base_color,
            "hover paint animation should reach the native renderer display list"
        );
        assert!(!frame_output.render_plan.is_empty());
        assert!(!frame_output.render_plan.batches.is_empty());
        assert!(!frame_output.render_plan.text_batches.is_empty());
    }

    #[test]
    fn native_document_demo_press_animates_button_style() {
        let mut app = crate::demo::NativeDocumentDemo::new();
        let mut frame = AppFrame::new(HostViewport::new(980, 680, 1.0), DocumentInput::default());
        app.update(&mut frame);
        let base_rect = app
            .last_output()
            .unwrap()
            .layout
            .find("native-action")
            .unwrap()
            .rect;
        let base_output = frame.into_output(des_ui_wgpu::RenderOptions::default());
        let base_color = fill_rect_color(&base_output, "native-action")
            .expect("native action should paint a base fill");
        let press_position = Point::new(
            base_rect.origin.x + base_rect.size.width * 0.5,
            base_rect.origin.y + base_rect.size.height * 0.5,
        );
        let mut press_frame = AppFrame::new(
            HostViewport::new(980, 680, 1.0),
            crate::demo::press_input_at(press_position, 0.20),
        );

        app.update(&mut press_frame);

        let press_output = app.last_output().unwrap();
        let press_rect = press_output.layout.find("native-action").unwrap().rect;
        assert!(press_output.animating);
        assert!(press_output.metrics.animation_changed_layout);
        assert!(press_output.metrics.animation_changed_paint);
        assert!(press_frame.repaint_requested());
        assert!(press_rect.size.width < base_rect.size.width);
        assert!(press_output.events.iter().any(|event| {
            event.target.as_str() == "native-action" && event.kind == DocumentEventKind::Pressed
        }));

        let frame_output = press_frame.into_output(des_ui_wgpu::RenderOptions::default());
        let press_color = fill_rect_color(&frame_output, "native-action")
            .expect("native action should paint a pressed fill");
        assert_ne!(
            press_color, base_color,
            "press paint animation should reach the native renderer display list"
        );
    }

    fn fill_rect_color(output: &FrameOutput, element_id: &str) -> Option<Color> {
        output
            .display_list
            .commands
            .iter()
            .find_map(|command| match command {
                PaintCommand::FillRect(fill) if fill.element_id.as_str() == element_id => {
                    Some(fill.color)
                }
                _ => None,
            })
    }
}
