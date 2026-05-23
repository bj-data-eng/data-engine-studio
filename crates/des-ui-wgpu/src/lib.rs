//! `wgpu` adapter for DES UI paint commands.
//!
//! This crate owns the GPU-facing representation of renderer-neutral
//! `des-ui-render` display lists. It intentionally starts below document
//! semantics: document/style/layout produce paint commands, and this crate
//! turns the supported commands into meshes and, later, `wgpu` draw calls.

use ab_glyph::{Font, FontArc, GlyphId, PxScale, ScaleFont, point};
use des_ui_document::{Color, Rect, TextWrapMode};
use des_ui_render::{
    DisplayList, PrimitiveCommand, PrimitiveList, RenderPrimitive, TextPaint,
    TriangleMeshPrimitive, plan_primitives,
};
use std::{error, fmt, mem};

const INTER_VARIABLE: &[u8] =
    include_bytes!("../../des-ui-text/assets/fonts/inter/InterVariable.ttf");

const SHADER: &str = r#"
struct Viewport {
    size: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> viewport: Viewport;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    let ndc = vec2<f32>(
        (input.position.x / viewport.size.x) * 2.0 - 1.0,
        1.0 - (input.position.y / viewport.size.y) * 2.0,
    );
    var output: VertexOutput;
    output.position = vec4<f32>(ndc, 0.0, 1.0);
    output.color = input.color;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return input.color;
}
"#;

const TEXT_SHADER: &str = r#"
struct Viewport {
    size: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> viewport: Viewport;
@group(0) @binding(1)
var text_texture: texture_2d<f32>;
@group(0) @binding(2)
var text_sampler: sampler;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    let ndc = vec2<f32>(
        (input.position.x / viewport.size.x) * 2.0 - 1.0,
        1.0 - (input.position.y / viewport.size.y) * 2.0,
    );
    var output: VertexOutput;
    output.position = vec4<f32>(ndc, 0.0, 1.0);
    output.uv = input.uv;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(text_texture, text_sampler, input.uv);
}
"#;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ClearColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl ClearColor {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn to_wgpu(self) -> wgpu::Color {
        wgpu::Color {
            r: self.r as f64 / 255.0,
            g: self.g as f64 / 255.0,
            b: self.b as f64 / 255.0,
            a: self.a as f64 / 255.0,
        }
    }
}

impl Default for ClearColor {
    fn default() -> Self {
        Self::rgb(255, 255, 255)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PresentMode {
    #[default]
    Vsync,
    Immediate,
}

impl PresentMode {
    pub fn to_wgpu(self) -> wgpu::PresentMode {
        match self {
            Self::Vsync => wgpu::PresentMode::Fifo,
            Self::Immediate => wgpu::PresentMode::Immediate,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RenderOptions {
    pub clear_color: ClearColor,
    pub present_mode: PresentMode,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            clear_color: ClearColor::default(),
            present_mode: PresentMode::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PhysicalRenderSize {
    pub width: u32,
    pub height: u32,
    pub scale_factor: f64,
}

impl PhysicalRenderSize {
    pub fn new(width: u32, height: u32, scale_factor: f64) -> Self {
        Self {
            width,
            height,
            scale_factor,
        }
    }

    pub fn logical_width(self) -> f32 {
        self.width as f32 / self.scale_factor as f32
    }

    pub fn logical_height(self) -> f32 {
        self.height as f32 / self.scale_factor as f32
    }

    pub fn is_empty(self) -> bool {
        self.width == 0 || self.height == 0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScissorRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug)]
pub enum RendererError {
    CreateSurface(wgpu::CreateSurfaceError),
    RequestAdapter(wgpu::RequestAdapterError),
    RequestDevice(wgpu::RequestDeviceError),
    UnsupportedSurface,
    SurfaceFrame(&'static str),
}

impl fmt::Display for RendererError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CreateSurface(error) => write!(f, "failed to create wgpu surface: {error}"),
            Self::RequestAdapter(error) => write!(f, "failed to request wgpu adapter: {error}"),
            Self::RequestDevice(error) => write!(f, "failed to request wgpu device: {error}"),
            Self::UnsupportedSurface => f.write_str("surface is not supported by the selected GPU"),
            Self::SurfaceFrame(error) => write!(f, "failed to render surface frame: {error}"),
        }
    }
}

impl error::Error for RendererError {}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RenderPlan {
    pub clear_color: ClearColor,
    pub items: Vec<RenderItem>,
    pub batches: Vec<MeshBatch>,
    pub text_batches: Vec<TextBatch>,
}

impl RenderPlan {
    pub fn is_empty(&self) -> bool {
        self.batches.iter().all(|batch| batch.mesh.is_empty()) && self.text_batches.is_empty()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum RenderItem {
    Mesh(MeshBatch),
    Text(TextBatch),
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MeshBatch {
    pub clip: Option<Rect>,
    pub mesh: Mesh,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextBatch {
    pub clip: Option<Rect>,
    pub text: TextPaint,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RasterizedText {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
    pub mesh: TextMesh,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextMesh {
    pub vertices: Vec<TextVertex>,
    pub indices: Vec<u32>,
}

impl TextMesh {
    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TextVertex {
    pub position: [f32; 2],
    pub uv: [f32; 2],
}

impl TextVertex {
    pub const ATTRIBUTES: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2];

    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

#[derive(Clone, Debug)]
pub struct TextRasterizer {
    font: FontArc,
}

impl TextRasterizer {
    pub fn new() -> Self {
        Self {
            font: FontArc::try_from_slice(INTER_VARIABLE)
                .expect("bundled Inter font must load for native text rendering"),
        }
    }

    pub fn rasterize(&self, text: &TextPaint, scale_factor: f32) -> RasterizedText {
        let scale_factor = scale_factor.max(0.000_001);
        let width = (text.rect.size.width * scale_factor).ceil().max(1.0) as u32;
        let height = (text.rect.size.height * scale_factor).ceil().max(1.0) as u32;
        let mut pixels = vec![0; width as usize * height as usize * 4];
        let layout = self.layout_text(text, scale_factor, width as f32);

        for glyph in layout.glyphs {
            let Some(outlined) = self.font.outline_glyph(glyph) else {
                continue;
            };
            let bounds = outlined.px_bounds();
            outlined.draw(|x, y, coverage| {
                let pixel_x = bounds.min.x.floor() as i32 + x as i32;
                let pixel_y = bounds.min.y.floor() as i32 + y as i32;
                if pixel_x < 0 || pixel_y < 0 || pixel_x >= width as i32 || pixel_y >= height as i32
                {
                    return;
                }
                let index = (pixel_y as u32 * width + pixel_x as u32) as usize * 4;
                let alpha = (text.color.a as f32 * coverage).round().clamp(0.0, 255.0) as u8;
                blend_pixel(
                    &mut pixels[index..index + 4],
                    [text.color.r, text.color.g, text.color.b, alpha],
                );
            });
        }

        RasterizedText {
            width,
            height,
            pixels,
            mesh: text_quad(text.rect),
        }
    }

    fn layout_text(
        &self,
        text: &TextPaint,
        scale_factor: f32,
        physical_wrap_width: f32,
    ) -> RasterTextLayout {
        let scale = PxScale::from(text.font_size * scale_factor);
        let scaled = self.font.as_scaled(scale);
        let line_height = text
            .line_height
            .unwrap_or_else(|| text.font_size * 1.2)
            .max(1.0)
            * scale_factor;
        let max_lines = text.max_lines.unwrap_or(usize::MAX);
        let wrap_width = match text.wrap_mode {
            TextWrapMode::Extend => f32::INFINITY,
            TextWrapMode::Wrap => (text.wrap_width * scale_factor)
                .min(physical_wrap_width)
                .max(1.0),
            TextWrapMode::Truncate => physical_wrap_width.max(1.0),
        };
        let mut glyphs = Vec::new();
        let mut line = 0usize;
        let mut x = 0.0;
        let mut baseline = scaled.ascent().ceil();
        let mut previous: Option<GlyphId> = None;

        for ch in text.text.chars() {
            if ch == '\n' {
                if !advance_line(&mut line, max_lines, &mut x, &mut baseline, line_height) {
                    break;
                }
                previous = None;
                continue;
            }
            let glyph_id = self.font.glyph_id(ch);
            let advance = scaled.h_advance(glyph_id);
            if x > 0.0 && x + advance > wrap_width {
                if !advance_line(&mut line, max_lines, &mut x, &mut baseline, line_height) {
                    break;
                }
                previous = None;
            }
            if let Some(previous) = previous {
                x += scaled.kern(previous, glyph_id);
            }
            let glyph = glyph_id.with_scale_and_position(scale, point(x, baseline));
            glyphs.push(glyph);
            x += advance;
            previous = Some(glyph_id);
        }

        RasterTextLayout { glyphs }
    }
}

impl Default for TextRasterizer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Default)]
struct RasterTextLayout {
    glyphs: Vec<ab_glyph::Glyph>,
}

fn advance_line(
    line: &mut usize,
    max_lines: usize,
    x: &mut f32,
    baseline: &mut f32,
    line_height: f32,
) -> bool {
    *line += 1;
    if *line >= max_lines {
        return false;
    }
    *x = 0.0;
    *baseline += line_height;
    true
}

fn text_quad(rect: Rect) -> TextMesh {
    let left = rect.origin.x;
    let top = rect.origin.y;
    let right = rect.right();
    let bottom = rect.bottom();
    TextMesh {
        vertices: vec![
            TextVertex {
                position: [left, top],
                uv: [0.0, 0.0],
            },
            TextVertex {
                position: [right, top],
                uv: [1.0, 0.0],
            },
            TextVertex {
                position: [right, bottom],
                uv: [1.0, 1.0],
            },
            TextVertex {
                position: [left, bottom],
                uv: [0.0, 1.0],
            },
        ],
        indices: vec![0, 1, 2, 0, 2, 3],
    }
}

fn blend_pixel(destination: &mut [u8], source: [u8; 4]) {
    let source_alpha = source[3] as f32 / 255.0;
    let destination_alpha = destination[3] as f32 / 255.0;
    let out_alpha = source_alpha + destination_alpha * (1.0 - source_alpha);
    if out_alpha <= f32::EPSILON {
        destination.copy_from_slice(&[0, 0, 0, 0]);
        return;
    }
    for channel in 0..3 {
        let source_value = source[channel] as f32 / 255.0;
        let destination_value = destination[channel] as f32 / 255.0;
        let out = (source_value * source_alpha
            + destination_value * destination_alpha * (1.0 - source_alpha))
            / out_alpha;
        destination[channel] = (out * 255.0).round().clamp(0.0, 255.0) as u8;
    }
    destination[3] = (out_alpha * 255.0).round().clamp(0.0, 255.0) as u8;
}

#[derive(Clone, Debug, Default)]
pub struct DisplayListRenderer {
    options: RenderOptions,
}

impl DisplayListRenderer {
    pub fn new(options: RenderOptions) -> Self {
        Self { options }
    }

    pub fn build_plan_for_output(&self, output: &des_ui_document::DocumentOutput) -> RenderPlan {
        self.build_plan(&des_ui_render::plan_paint(output))
    }

    pub fn build_plan(&self, display_list: &DisplayList) -> RenderPlan {
        let mut builder = RenderPlanBuilder::new(self.options);
        builder.push_primitives(&plan_primitives(display_list));
        builder.finish()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PackedColor([u8; 4]);

impl PackedColor {
    pub const fn to_array(self) -> [u8; 4] {
        self.0
    }
}

impl From<Color> for PackedColor {
    fn from(color: Color) -> Self {
        Self([color.r, color.g, color.b, color.a])
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 2],
    pub color: [u8; 4],
}

impl Vertex {
    pub const ATTRIBUTES: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Unorm8x4];

    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
struct ViewportUniform {
    size: [f32; 2],
    _pad: [f32; 2],
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Mesh {
    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }
}

#[derive(Clone, Debug, Default)]
pub struct MeshBuilder {
    mesh: Mesh,
}

impl MeshBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_display_list(&mut self, display_list: &DisplayList) {
        let primitives = plan_primitives(display_list);
        for command in &primitives.commands {
            self.push_command(command);
        }
    }

    pub fn push_command(&mut self, command: &PrimitiveCommand) {
        match command {
            PrimitiveCommand::Draw(RenderPrimitive::Triangles(mesh)) => self.push_triangles(mesh),
            PrimitiveCommand::Draw(RenderPrimitive::Text(_))
            | PrimitiveCommand::PushClip(_)
            | PrimitiveCommand::PopClip => {}
        }
    }

    pub fn finish(self) -> Mesh {
        self.mesh
    }

    fn push_triangles(&mut self, primitive: &TriangleMeshPrimitive) {
        let base = self.mesh.vertices.len() as u32;
        self.mesh
            .vertices
            .extend(primitive.vertices.iter().map(|vertex| Vertex {
                position: [vertex.position.x, vertex.position.y],
                color: PackedColor::from(vertex.color).to_array(),
            }));
        self.mesh
            .indices
            .extend(primitive.indices.iter().map(|index| base + *index));
    }
}

pub fn mesh_for_display_list(display_list: &DisplayList) -> Mesh {
    let mut builder = MeshBuilder::new();
    builder.push_display_list(display_list);
    builder.finish()
}

pub struct GpuRenderer<'window> {
    surface: wgpu::Surface<'window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    text_pipeline: wgpu::RenderPipeline,
    text_bind_group_layout: wgpu::BindGroupLayout,
    text_sampler: wgpu::Sampler,
    viewport_buffer: wgpu::Buffer,
    viewport_bind_group: wgpu::BindGroup,
    text_rasterizer: TextRasterizer,
    size: PhysicalRenderSize,
}

impl<'window> GpuRenderer<'window> {
    pub async fn new(
        target: impl Into<wgpu::SurfaceTarget<'window>>,
        size: PhysicalRenderSize,
        options: RenderOptions,
    ) -> Result<Self, RendererError> {
        let instance = wgpu::Instance::default();
        let surface = instance
            .create_surface(target)
            .map_err(RendererError::CreateSurface)?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .map_err(RendererError::RequestAdapter)?;
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("des-ui-wgpu device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_defaults(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
            })
            .await
            .map_err(RendererError::RequestDevice)?;
        let mut config = surface
            .get_default_config(&adapter, size.width.max(1), size.height.max(1))
            .ok_or(RendererError::UnsupportedSurface)?;
        config.present_mode = options.present_mode.to_wgpu();
        surface.configure(&device, &config);
        let viewport_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("des-ui viewport uniform"),
            size: mem::size_of::<ViewportUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let viewport_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("des-ui viewport bind group layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });
        let viewport_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("des-ui viewport bind group"),
            layout: &viewport_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: viewport_buffer.as_entire_binding(),
            }],
        });
        let text_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("des-ui text bind group layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });
        let text_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("des-ui text sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });
        let pipeline = create_pipeline(&device, config.format, &viewport_bind_group_layout);
        let text_pipeline = create_text_pipeline(&device, config.format, &text_bind_group_layout);
        let renderer = Self {
            surface,
            device,
            queue,
            config,
            pipeline,
            text_pipeline,
            text_bind_group_layout,
            text_sampler,
            viewport_buffer,
            viewport_bind_group,
            text_rasterizer: TextRasterizer::new(),
            size,
        };
        renderer.write_viewport_uniform();
        Ok(renderer)
    }

    pub fn resize(&mut self, size: PhysicalRenderSize) {
        self.size = size;
        if size.is_empty() {
            return;
        }
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
        self.write_viewport_uniform();
    }

    pub fn render_plan(&mut self, plan: &RenderPlan) -> Result<(), RendererError> {
        if self.size.is_empty() {
            return Ok(());
        }
        self.write_viewport_uniform();
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame)
            | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
            wgpu::CurrentSurfaceTexture::Lost | wgpu::CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&self.device, &self.config);
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                return Err(RendererError::SurfaceFrame("surface validation error"));
            }
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("des-ui render encoder"),
            });
        {
            let color_attachment = Some(wgpu::RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(plan.clear_color.to_wgpu()),
                    store: wgpu::StoreOp::Store,
                },
            });
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("des-ui render pass"),
                color_attachments: &[color_attachment],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.viewport_bind_group, &[]);
            for item in &plan.items {
                match item {
                    RenderItem::Mesh(batch) => {
                        pass.set_pipeline(&self.pipeline);
                        pass.set_bind_group(0, &self.viewport_bind_group, &[]);
                        self.draw_mesh_batch(&mut pass, batch);
                    }
                    RenderItem::Text(batch) => {
                        pass.set_pipeline(&self.text_pipeline);
                        self.draw_text_batch(&mut pass, batch);
                    }
                }
            }
        }
        self.queue.submit([encoder.finish()]);
        frame.present();
        Ok(())
    }

    fn draw_mesh_batch<'pass>(
        &'pass self,
        pass: &mut wgpu::RenderPass<'pass>,
        batch: &'pass MeshBatch,
    ) {
        if batch.mesh.is_empty() {
            return;
        }
        let Some(scissor) = clip_rect_to_scissor(batch.clip, self.size) else {
            return;
        };
        pass.set_scissor_rect(scissor.x, scissor.y, scissor.width, scissor.height);
        let vertex_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("des-ui mesh vertex buffer"),
            size: (batch.mesh.vertices.len() * mem::size_of::<Vertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let index_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("des-ui mesh index buffer"),
            size: (batch.mesh.indices.len() * mem::size_of::<u32>()) as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.queue.write_buffer(
            &vertex_buffer,
            0,
            bytemuck::cast_slice(&batch.mesh.vertices),
        );
        self.queue
            .write_buffer(&index_buffer, 0, bytemuck::cast_slice(&batch.mesh.indices));
        pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        pass.draw_indexed(0..batch.mesh.indices.len() as u32, 0, 0..1);
    }

    fn draw_text_batch<'pass>(
        &'pass self,
        pass: &mut wgpu::RenderPass<'pass>,
        batch: &'pass TextBatch,
    ) {
        let rasterized = self
            .text_rasterizer
            .rasterize(&batch.text, self.size.scale_factor as f32);
        if rasterized.mesh.is_empty() || rasterized.pixels.is_empty() {
            return;
        }
        let Some(scissor) = clip_rect_to_scissor(batch.clip, self.size) else {
            return;
        };
        pass.set_scissor_rect(scissor.x, scissor.y, scissor.width, scissor.height);

        let texture_size = wgpu::Extent3d {
            width: rasterized.width,
            height: rasterized.height,
            depth_or_array_layers: 1,
        };
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("des-ui text texture"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &rasterized.pixels,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(rasterized.width * 4),
                rows_per_image: Some(rasterized.height),
            },
            texture_size,
        );
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("des-ui text bind group"),
            layout: &self.text_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.viewport_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&self.text_sampler),
                },
            ],
        });
        let vertex_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("des-ui text vertex buffer"),
            size: (rasterized.mesh.vertices.len() * mem::size_of::<TextVertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let index_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("des-ui text index buffer"),
            size: (rasterized.mesh.indices.len() * mem::size_of::<u32>()) as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.queue.write_buffer(
            &vertex_buffer,
            0,
            bytemuck::cast_slice(&rasterized.mesh.vertices),
        );
        self.queue.write_buffer(
            &index_buffer,
            0,
            bytemuck::cast_slice(&rasterized.mesh.indices),
        );
        pass.set_bind_group(0, &bind_group, &[]);
        pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        pass.draw_indexed(0..rasterized.mesh.indices.len() as u32, 0, 0..1);
    }

    fn write_viewport_uniform(&self) {
        let uniform = ViewportUniform {
            size: [
                self.size.logical_width().max(1.0),
                self.size.logical_height().max(1.0),
            ],
            _pad: [0.0, 0.0],
        };
        self.queue
            .write_buffer(&self.viewport_buffer, 0, bytemuck::bytes_of(&uniform));
    }
}

fn create_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    viewport_bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("des-ui mesh shader"),
        source: wgpu::ShaderSource::Wgsl(SHADER.into()),
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("des-ui mesh pipeline layout"),
        bind_group_layouts: &[Some(viewport_bind_group_layout)],
        immediate_size: 0,
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("des-ui mesh pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[Vertex::layout()],
        },
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview_mask: None,
        cache: None,
    })
}

fn create_text_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    text_bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("des-ui text shader"),
        source: wgpu::ShaderSource::Wgsl(TEXT_SHADER.into()),
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("des-ui text pipeline layout"),
        bind_group_layouts: &[Some(text_bind_group_layout)],
        immediate_size: 0,
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("des-ui text pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[TextVertex::layout()],
        },
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview_mask: None,
        cache: None,
    })
}

pub fn clip_rect_to_scissor(clip: Option<Rect>, size: PhysicalRenderSize) -> Option<ScissorRect> {
    if size.is_empty() {
        return None;
    }
    let scale = size.scale_factor as f32;
    let (left, top, right, bottom) = if let Some(clip) = clip {
        (
            (clip.origin.x * scale).floor().max(0.0),
            (clip.origin.y * scale).floor().max(0.0),
            (clip.right() * scale).ceil().min(size.width as f32),
            (clip.bottom() * scale).ceil().min(size.height as f32),
        )
    } else {
        (0.0, 0.0, size.width as f32, size.height as f32)
    };
    if right <= left || bottom <= top {
        return None;
    }
    Some(ScissorRect {
        x: left as u32,
        y: top as u32,
        width: (right - left) as u32,
        height: (bottom - top) as u32,
    })
}

#[derive(Clone, Debug)]
struct RenderPlanBuilder {
    plan: RenderPlan,
    current_clip: Option<Rect>,
    current_mesh: MeshBuilder,
}

impl RenderPlanBuilder {
    fn new(options: RenderOptions) -> Self {
        Self {
            plan: RenderPlan {
                clear_color: options.clear_color,
                items: Vec::new(),
                batches: Vec::new(),
                text_batches: Vec::new(),
            },
            current_clip: None,
            current_mesh: MeshBuilder::new(),
        }
    }

    fn push_primitives(&mut self, primitives: &PrimitiveList) {
        let mut clip_stack: Vec<Rect> = Vec::new();
        for command in &primitives.commands {
            match command {
                PrimitiveCommand::PushClip(rect) => {
                    clip_stack.push(*rect);
                    self.set_clip(clip_stack.last().copied());
                }
                PrimitiveCommand::PopClip => {
                    clip_stack.pop();
                    self.set_clip(clip_stack.last().copied());
                }
                PrimitiveCommand::Draw(RenderPrimitive::Text(text)) => {
                    self.flush();
                    let batch = TextBatch {
                        clip: self.current_clip,
                        text: text.clone(),
                    };
                    self.plan.items.push(RenderItem::Text(batch.clone()));
                    self.plan.text_batches.push(batch);
                }
                _ => self.current_mesh.push_command(command),
            }
        }
    }

    fn set_clip(&mut self, clip: Option<Rect>) {
        if self.current_clip == clip {
            return;
        }
        self.flush();
        self.current_clip = clip;
    }

    fn flush(&mut self) {
        let mesh = std::mem::take(&mut self.current_mesh).finish();
        if !mesh.is_empty() {
            let batch = MeshBatch {
                clip: self.current_clip,
                mesh,
            };
            self.plan.items.push(RenderItem::Mesh(batch.clone()));
            self.plan.batches.push(batch);
        }
    }

    fn finish(mut self) -> RenderPlan {
        self.flush();
        self.plan
    }
}

#[cfg(test)]
mod tests {
    use des_ui_document::{
        Color, CornerRadii, Document, DocumentEngine, Element, ElementId, Rect, Size, Style,
        StyleSelector, StyleSheet, TextWrapMode,
    };
    use des_ui_render::{DisplayList, FillRectPaint, PaintCommand, TextPaint};

    use crate::{
        ClearColor, DisplayListRenderer, MeshBuilder, PackedColor, PhysicalRenderSize, RenderItem,
        RenderOptions, ScissorRect, TextRasterizer, clip_rect_to_scissor, mesh_for_display_list,
    };

    #[test]
    fn packed_color_preserves_rgba_channel_order() {
        assert_eq!(
            PackedColor::from(Color::rgba(10, 20, 30, 40)).to_array(),
            [10, 20, 30, 40]
        );
    }

    #[test]
    fn fill_rect_generates_antialiased_triangles_in_document_coordinates() {
        let mut display_list = DisplayList::new();
        display_list.push(PaintCommand::FillRect(FillRectPaint {
            element_id: ElementId::new("box"),
            rect: Rect::new(10.0, 20.0, 30.0, 40.0),
            radius: CornerRadii::ZERO,
            color: Color::rgba(1, 2, 3, 4),
        }));
        let mut builder = MeshBuilder::new();
        builder.push_display_list(&display_list);

        let mesh = builder.finish();
        assert_eq!(mesh.vertices.len(), 8);
        assert_eq!(&mesh.indices[0..6], [0, 1, 2, 0, 2, 3]);
        assert_eq!(mesh.vertices[0].position, [10.0, 20.0]);
        assert_eq!(mesh.vertices[1].position, [40.0, 20.0]);
        assert_eq!(mesh.vertices[2].position, [40.0, 60.0]);
        assert_eq!(mesh.vertices[3].position, [10.0, 60.0]);
        assert!(
            mesh.vertices
                .iter()
                .any(|vertex| vertex.color == [1, 2, 3, 4])
        );
        assert!(
            mesh.vertices
                .iter()
                .any(|vertex| vertex.color == [1, 2, 3, 0])
        );
    }

    #[test]
    fn render_plan_preserves_clear_color_and_clip_batches() {
        let mut display_list = DisplayList::new();
        display_list.push(PaintCommand::FillRect(FillRectPaint {
            element_id: ElementId::new("before"),
            rect: Rect::new(0.0, 0.0, 10.0, 10.0),
            radius: CornerRadii::ZERO,
            color: Color::rgb(1, 2, 3),
        }));
        display_list.push(PaintCommand::PushClip(Rect::new(4.0, 5.0, 6.0, 7.0)));
        display_list.push(PaintCommand::FillRect(FillRectPaint {
            element_id: ElementId::new("inside"),
            rect: Rect::new(10.0, 10.0, 10.0, 10.0),
            radius: CornerRadii::ZERO,
            color: Color::rgb(4, 5, 6),
        }));
        display_list.push(PaintCommand::PopClip);

        let plan = DisplayListRenderer::new(RenderOptions {
            clear_color: ClearColor::rgb(9, 8, 7),
            ..RenderOptions::default()
        })
        .build_plan(&display_list);

        assert_eq!(plan.clear_color, ClearColor::rgb(9, 8, 7));
        assert_eq!(plan.batches.len(), 2);
        assert_eq!(plan.batches[0].clip, None);
        assert_eq!(plan.batches[1].clip, Some(Rect::new(4.0, 5.0, 6.0, 7.0)));
        assert_eq!(plan.batches[0].mesh.indices.len(), 30);
        assert_eq!(plan.batches[1].mesh.indices.len(), 30);
    }

    #[test]
    fn render_plan_preserves_text_batches_with_active_clip() {
        let mut display_list = DisplayList::new();
        display_list.push(PaintCommand::PushClip(Rect::new(1.0, 2.0, 30.0, 40.0)));
        display_list.push(PaintCommand::Text(TextPaint {
            element_id: ElementId::new("label"),
            rect: Rect::new(4.0, 5.0, 20.0, 10.0),
            text: "Ready".into(),
            color: Color::rgb(1, 2, 3),
            font_size: 12.0,
            wrap_width: 20.0,
            wrap_mode: TextWrapMode::Extend,
            max_lines: None,
            line_height: None,
            selection: None,
        }));
        display_list.push(PaintCommand::PopClip);

        let plan = DisplayListRenderer::new(RenderOptions::default()).build_plan(&display_list);

        assert_eq!(plan.text_batches.len(), 1);
        assert_eq!(
            plan.text_batches[0].clip,
            Some(Rect::new(1.0, 2.0, 30.0, 40.0))
        );
        assert_eq!(plan.text_batches[0].text.text, "Ready");
        assert!(plan.batches.is_empty());
        assert!(!plan.is_empty());
    }

    #[test]
    fn text_rasterizer_draws_arbitrary_text_into_rgba_pixels() {
        let text = TextPaint {
            element_id: ElementId::new("label"),
            rect: Rect::new(12.0, 18.0, 220.0, 72.0),
            text: "Hello native text: π Σ data".into(),
            color: Color::rgb(18, 26, 38),
            font_size: 18.0,
            wrap_width: 220.0,
            wrap_mode: TextWrapMode::Wrap,
            max_lines: None,
            line_height: None,
            selection: None,
        };

        let rasterized = TextRasterizer::new().rasterize(&text, 2.0);

        assert!(rasterized.width > 0);
        assert!(rasterized.height > 0);
        assert_eq!(
            rasterized.pixels.len(),
            rasterized.width as usize * rasterized.height as usize * 4
        );
        assert!(rasterized.pixels.chunks_exact(4).any(|pixel| pixel[3] > 0));
        assert_eq!(rasterized.mesh.vertices[0].position, [12.0, 18.0]);
        assert_eq!(rasterized.mesh.indices, [0, 1, 2, 0, 2, 3]);
    }

    #[test]
    fn renderer_builds_plan_directly_from_document_output() {
        let mut document = Document::build(Size::new(200.0, 100.0), |ui| {
            ui.div("panel").children(|ui| {
                ui.text("label", "Document text");
            });
        });
        let stylesheet = StyleSheet::new()
            .rule(
                StyleSelector::Id("panel".into()),
                Style::default()
                    .size(100.0, 60.0)
                    .background(Color::rgb(20, 30, 40)),
            )
            .rule(
                StyleSelector::Element(Element::Text),
                Style::default()
                    .size(80.0, 20.0)
                    .text_color(Color::rgb(240, 241, 242)),
            );
        let output = DocumentEngine::default().update(&mut document, &stylesheet);

        let plan = DisplayListRenderer::default().build_plan_for_output(&output);

        assert!(!plan.batches.is_empty());
        assert_eq!(plan.text_batches.len(), 1);
        assert_eq!(plan.text_batches[0].text.text, "Document text");
    }

    #[test]
    fn render_plan_preserves_mixed_mesh_and_text_order() {
        let mut display_list = DisplayList::new();
        display_list.push(PaintCommand::FillRect(FillRectPaint {
            element_id: ElementId::new("background"),
            rect: Rect::new(0.0, 0.0, 100.0, 40.0),
            radius: CornerRadii::ZERO,
            color: Color::rgb(20, 30, 40),
        }));
        display_list.push(PaintCommand::Text(TextPaint {
            element_id: ElementId::new("label"),
            rect: Rect::new(4.0, 5.0, 20.0, 10.0),
            text: "Layered".into(),
            color: Color::rgb(1, 2, 3),
            font_size: 12.0,
            wrap_width: 20.0,
            wrap_mode: TextWrapMode::Extend,
            max_lines: None,
            line_height: None,
            selection: None,
        }));
        display_list.push(PaintCommand::FillRect(FillRectPaint {
            element_id: ElementId::new("overlay"),
            rect: Rect::new(8.0, 8.0, 20.0, 8.0),
            radius: CornerRadii::ZERO,
            color: Color::rgba(200, 0, 0, 128),
        }));

        let plan = DisplayListRenderer::default().build_plan(&display_list);

        assert!(matches!(plan.items[0], RenderItem::Mesh(_)));
        assert!(matches!(plan.items[1], RenderItem::Text(_)));
        assert!(matches!(plan.items[2], RenderItem::Mesh(_)));
        assert_eq!(plan.batches.len(), 2);
        assert_eq!(plan.text_batches.len(), 1);
    }

    #[test]
    fn clip_rect_converts_to_physical_scissor_inside_surface() {
        let scissor = clip_rect_to_scissor(
            Some(Rect::new(10.0, 20.5, 30.0, 40.0)),
            PhysicalRenderSize {
                width: 200,
                height: 200,
                scale_factor: 2.0,
            },
        );

        assert_eq!(
            scissor,
            Some(ScissorRect {
                x: 20,
                y: 41,
                width: 60,
                height: 80,
            })
        );
    }

    #[test]
    fn mesh_builder_consumes_backend_neutral_primitives() {
        let mut display_list = DisplayList::new();
        display_list.push(PaintCommand::FillCircle(des_ui_render::FillCirclePaint {
            element_id: ElementId::new("dot"),
            center: des_ui_document::Point::new(10.0, 10.0),
            radius: 4.0,
            color: Color::rgba(7, 8, 9, 10),
        }));

        let mesh = mesh_for_display_list(&display_list);

        assert!(mesh.vertices.len() > 4);
        assert!(mesh.indices.len() > 6);
        assert!(
            mesh.vertices
                .iter()
                .any(|vertex| vertex.color == [7, 8, 9, 10])
        );
        assert!(
            mesh.vertices
                .iter()
                .any(|vertex| vertex.color == [7, 8, 9, 0])
        );
    }
}
