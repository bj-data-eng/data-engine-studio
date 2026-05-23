//! `wgpu` adapter for DES UI paint commands.
//!
//! This crate owns the GPU-facing representation of renderer-neutral
//! `des-ui-render` display lists. It intentionally starts below document
//! semantics: document/style/layout produce paint commands, and this crate
//! turns the supported commands into meshes and, later, `wgpu` draw calls.

use des_ui_document::{Color, Rect, TextWrapMode};
use des_ui_render::{
    DisplayList, EpaintMeshPrimitive, PrimitiveCommand, PrimitiveList, PrimitivePlanner,
    RenderPrimitive, RenderTessellationOptions, TextPaint,
};
use std::{cell::RefCell, collections::HashMap, error, fmt, mem, ops::Range};

const TEXTURED_SHADER: &str = r#"
struct Viewport {
    size: vec2<f32>,
    dithering: u32,
    predictable_texture_filtering: u32,
};

@group(0) @binding(0)
var<uniform> viewport: Viewport;
@group(1) @binding(0)
var paint_texture: texture_2d<f32>;
@group(1) @binding(1)
var paint_sampler: sampler;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: u32,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
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
    output.color = unpack_color(input.color);
    return output;
}

@fragment
fn fs_main_gamma_framebuffer(input: VertexOutput) -> @location(0) vec4<f32> {
    var color = sample_texture(input) * input.color;
    if viewport.dithering == 1u {
        color = vec4<f32>(dither_interleaved(color.rgb, 256.0, input.position), color.a);
    }
    return color;
}

fn linear_from_gamma_rgb(srgb: vec3<f32>) -> vec3<f32> {
    let cutoff = srgb < vec3<f32>(0.04045);
    let lower = srgb / vec3<f32>(12.92);
    let higher = pow((srgb + vec3<f32>(0.055)) / vec3<f32>(1.055), vec3<f32>(2.4));
    return select(higher, lower, cutoff);
}

fn interleaved_gradient_noise(n: vec2<f32>) -> f32 {
    let f = 0.06711056 * n.x + 0.00583715 * n.y;
    return fract(52.9829189 * fract(f));
}

fn dither_interleaved(rgb: vec3<f32>, levels: f32, frag_coord: vec4<f32>) -> vec3<f32> {
    var noise = interleaved_gradient_noise(frag_coord.xy);
    noise = (noise - 0.5) * 0.95;
    return rgb + noise / (levels - 1.0);
}

fn sample_texture(input: VertexOutput) -> vec4<f32> {
    if viewport.predictable_texture_filtering == 0u {
        return textureSample(paint_texture, paint_sampler, input.uv);
    }

    let texture_size = vec2<i32>(textureDimensions(paint_texture, 0));
    let texture_size_f = vec2<f32>(texture_size);
    let pixel_coord = input.uv * texture_size_f - 0.5;
    let pixel_fract = fract(pixel_coord);
    let pixel_floor = vec2<i32>(floor(pixel_coord));
    let max_coord = texture_size - vec2<i32>(1, 1);
    let p00 = clamp(pixel_floor + vec2<i32>(0, 0), vec2<i32>(0, 0), max_coord);
    let p10 = clamp(pixel_floor + vec2<i32>(1, 0), vec2<i32>(0, 0), max_coord);
    let p01 = clamp(pixel_floor + vec2<i32>(0, 1), vec2<i32>(0, 0), max_coord);
    let p11 = clamp(pixel_floor + vec2<i32>(1, 1), vec2<i32>(0, 0), max_coord);
    let top = mix(textureLoad(paint_texture, p00, 0), textureLoad(paint_texture, p10, 0), pixel_fract.x);
    let bottom = mix(textureLoad(paint_texture, p01, 0), textureLoad(paint_texture, p11, 0), pixel_fract.x);
    return mix(top, bottom, pixel_fract.y);
}

fn unpack_color(color: u32) -> vec4<f32> {
    return vec4<f32>(
        f32(color & 255u),
        f32((color >> 8u) & 255u),
        f32((color >> 16u) & 255u),
        f32((color >> 24u) & 255u),
    ) / 255.0;
}

@fragment
fn fs_main_linear_framebuffer(input: VertexOutput) -> @location(0) vec4<f32> {
    var color = sample_texture(input) * input.color;
    if viewport.dithering == 1u {
        color = vec4<f32>(dither_interleaved(color.rgb, 256.0, input.position), color.a);
    }
    return vec4<f32>(linear_from_gamma_rgb(color.rgb), color.a);
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RenderAlphaFromCoverage {
    Linear,
    Gamma(f32),
    TwoCoverageMinusCoverageSq,
}

impl Default for RenderAlphaFromCoverage {
    fn default() -> Self {
        epaint::AlphaFromCoverage::default().into()
    }
}

impl From<epaint::AlphaFromCoverage> for RenderAlphaFromCoverage {
    fn from(value: epaint::AlphaFromCoverage) -> Self {
        match value {
            epaint::AlphaFromCoverage::Linear => Self::Linear,
            epaint::AlphaFromCoverage::Gamma(gamma) => Self::Gamma(gamma),
            epaint::AlphaFromCoverage::TwoCoverageMinusCoverageSq => {
                Self::TwoCoverageMinusCoverageSq
            }
        }
    }
}

impl From<RenderAlphaFromCoverage> for epaint::AlphaFromCoverage {
    fn from(value: RenderAlphaFromCoverage) -> Self {
        match value {
            RenderAlphaFromCoverage::Linear => Self::Linear,
            RenderAlphaFromCoverage::Gamma(gamma) => Self::Gamma(gamma),
            RenderAlphaFromCoverage::TwoCoverageMinusCoverageSq => Self::TwoCoverageMinusCoverageSq,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum TextureFilter {
    Nearest,
    #[default]
    Linear,
}

impl From<epaint::textures::TextureFilter> for TextureFilter {
    fn from(value: epaint::textures::TextureFilter) -> Self {
        match value {
            epaint::textures::TextureFilter::Nearest => Self::Nearest,
            epaint::textures::TextureFilter::Linear => Self::Linear,
        }
    }
}

impl From<TextureFilter> for wgpu::FilterMode {
    fn from(value: TextureFilter) -> Self {
        match value {
            TextureFilter::Nearest => Self::Nearest,
            TextureFilter::Linear => Self::Linear,
        }
    }
}

impl From<TextureFilter> for wgpu::MipmapFilterMode {
    fn from(value: TextureFilter) -> Self {
        match value {
            TextureFilter::Nearest => Self::Nearest,
            TextureFilter::Linear => Self::Linear,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum TextureWrapMode {
    #[default]
    ClampToEdge,
    Repeat,
    MirroredRepeat,
}

impl From<epaint::textures::TextureWrapMode> for TextureWrapMode {
    fn from(value: epaint::textures::TextureWrapMode) -> Self {
        match value {
            epaint::textures::TextureWrapMode::ClampToEdge => Self::ClampToEdge,
            epaint::textures::TextureWrapMode::Repeat => Self::Repeat,
            epaint::textures::TextureWrapMode::MirroredRepeat => Self::MirroredRepeat,
        }
    }
}

impl From<TextureWrapMode> for wgpu::AddressMode {
    fn from(value: TextureWrapMode) -> Self {
        match value {
            TextureWrapMode::ClampToEdge => Self::ClampToEdge,
            TextureWrapMode::Repeat => Self::Repeat,
            TextureWrapMode::MirroredRepeat => Self::MirrorRepeat,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TextureOptions {
    pub magnification: TextureFilter,
    pub minification: TextureFilter,
    pub wrap_mode: TextureWrapMode,
    pub mipmap_mode: Option<TextureFilter>,
}

impl Default for TextureOptions {
    fn default() -> Self {
        epaint::textures::TextureOptions::default().into()
    }
}

impl From<epaint::textures::TextureOptions> for TextureOptions {
    fn from(value: epaint::textures::TextureOptions) -> Self {
        Self {
            magnification: value.magnification.into(),
            minification: value.minification.into(),
            wrap_mode: value.wrap_mode.into(),
            mipmap_mode: value.mipmap_mode.map(Into::into),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderTextOptions {
    pub max_texture_side: usize,
    pub alpha_from_coverage: RenderAlphaFromCoverage,
    pub font_hinting: bool,
}

impl Default for RenderTextOptions {
    fn default() -> Self {
        epaint::TextOptions::default().into()
    }
}

impl From<epaint::TextOptions> for RenderTextOptions {
    fn from(value: epaint::TextOptions) -> Self {
        Self {
            max_texture_side: value.max_texture_side,
            alpha_from_coverage: value.alpha_from_coverage.into(),
            font_hinting: value.font_hinting,
        }
    }
}

impl From<RenderTextOptions> for epaint::TextOptions {
    fn from(value: RenderTextOptions) -> Self {
        Self {
            max_texture_side: value.max_texture_side,
            alpha_from_coverage: value.alpha_from_coverage.into(),
            font_hinting: value.font_hinting,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderOptions {
    pub clear_color: ClearColor,
    pub present_mode: PresentMode,
    pub text: RenderTextOptions,
    pub tessellation: RenderTessellationOptions,
    pub dithering: bool,
    pub predictable_texture_filtering: bool,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            clear_color: ClearColor::default(),
            present_mode: PresentMode::default(),
            text: RenderTextOptions::default(),
            tessellation: RenderTessellationOptions::default(),
            dithering: true,
            predictable_texture_filtering: false,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegisteredTexture {
    pub width: u32,
    pub height: u32,
    pub options: TextureOptions,
    pub pixels: Vec<u8>,
}

impl RegisteredTexture {
    pub fn rgba(
        width: u32,
        height: u32,
        pixels: impl Into<Vec<u8>>,
        options: TextureOptions,
    ) -> Self {
        Self {
            width,
            height,
            options,
            pixels: pixels.into(),
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
    PartialTextureUpdateMissingTexture(epaint::TextureId),
    TextMeshCountMismatch { expected: usize, actual: usize },
    SurfaceFrame(&'static str),
}

impl fmt::Display for RendererError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CreateSurface(error) => write!(f, "failed to create wgpu surface: {error}"),
            Self::RequestAdapter(error) => write!(f, "failed to request wgpu adapter: {error}"),
            Self::RequestDevice(error) => write!(f, "failed to request wgpu device: {error}"),
            Self::UnsupportedSurface => f.write_str("surface is not supported by the selected GPU"),
            Self::PartialTextureUpdateMissingTexture(texture_id) => write!(
                f,
                "cannot apply partial native texture update before full allocation: {texture_id:?}"
            ),
            Self::TextMeshCountMismatch { expected, actual } => write!(
                f,
                "native text mesh count mismatch: expected {expected}, got {actual}"
            ),
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
    pub mesh: Mesh,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RasterizedTextFrame {
    pub width: u32,
    pub height: u32,
    pub atlas_delta: Option<TextAtlasDelta>,
    pub batches: Vec<Mesh>,
}

impl RasterizedTextFrame {
    pub fn is_empty(&self) -> bool {
        self.batches.iter().all(Mesh::is_empty)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextAtlasDelta {
    pub pos: Option<[u32; 2]>,
    pub width: u32,
    pub height: u32,
    pub options: TextureOptions,
    pub pixels: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextureDelta {
    pub pos: Option<[u32; 2]>,
    pub width: u32,
    pub height: u32,
    pub options: TextureOptions,
    pub pixels: Vec<u8>,
}

pub struct TextRasterizer {
    fonts: epaint::Fonts,
}

impl TextRasterizer {
    pub fn new() -> Self {
        Self::with_text_options(RenderTextOptions::default())
    }

    pub fn with_text_options(text_options: RenderTextOptions) -> Self {
        Self {
            fonts: epaint::Fonts::new(
                epaint::TextOptions::from(text_options),
                epaint::text::FontDefinitions::default(),
            ),
        }
    }

    pub fn rasterize(&mut self, text: &TextPaint, scale_factor: f32) -> RasterizedText {
        self.rasterize_with_options(
            text,
            scale_factor,
            RenderTextOptions::default(),
            RenderTessellationOptions::default(),
        )
    }

    pub fn rasterize_with_options(
        &mut self,
        text: &TextPaint,
        scale_factor: f32,
        text_options: RenderTextOptions,
        tessellation: RenderTessellationOptions,
    ) -> RasterizedText {
        let frame = self.rasterize_frame_with_options(
            std::slice::from_ref(text),
            scale_factor,
            text_options,
            tessellation,
        );
        RasterizedText {
            width: frame.width,
            height: frame.height,
            pixels: frame
                .atlas_delta
                .as_ref()
                .map_or_else(Vec::new, |delta| delta.pixels.clone()),
            mesh: frame.batches.into_iter().next().unwrap_or_default(),
        }
    }

    pub fn rasterize_frame(
        &mut self,
        text_batches: &[TextPaint],
        scale_factor: f32,
    ) -> RasterizedTextFrame {
        self.rasterize_frame_with_options(
            text_batches,
            scale_factor,
            RenderTextOptions::default(),
            RenderTessellationOptions::default(),
        )
    }

    pub fn rasterize_frame_with_options(
        &mut self,
        text_batches: &[TextPaint],
        scale_factor: f32,
        text_options: RenderTextOptions,
        tessellation: RenderTessellationOptions,
    ) -> RasterizedTextFrame {
        let scale_factor = scale_factor.max(0.000_001);
        self.fonts
            .begin_pass(epaint::TextOptions::from(text_options));
        let galleys = {
            let mut view = self.fonts.with_pixels_per_point(scale_factor);
            text_batches
                .iter()
                .map(|text| view.layout_job(epaint_layout_job(text)))
                .collect::<Vec<_>>()
        };
        let font_image_size = self.fonts.font_image_size();
        let prepared_discs = self.fonts.texture_atlas().prepared_discs();
        let mut tessellator = epaint::Tessellator::new(
            scale_factor,
            epaint::TessellationOptions::from(tessellation),
            font_image_size,
            prepared_discs,
        );
        let batches = text_batches
            .iter()
            .zip(galleys)
            .map(|(text, galley)| {
                let text_shape = epaint_text_shape(text, galley);
                let mesh = tessellate_text_shape(&mut tessellator, text_shape);
                Mesh::from_epaint_mesh(&mesh)
            })
            .collect();
        let [width, height] = self.fonts.font_image_size();
        let atlas_delta = self
            .fonts
            .font_image_delta()
            .map(text_atlas_delta_from_epaint);

        RasterizedTextFrame {
            width: width as u32,
            height: height as u32,
            atlas_delta,
            batches,
        }
    }

    pub fn shape_tessellator(
        &self,
        scale_factor: f32,
        tessellation: RenderTessellationOptions,
    ) -> epaint::Tessellator {
        epaint::Tessellator::new(
            scale_factor.max(0.000_001),
            epaint::TessellationOptions::from(tessellation),
            self.fonts.font_image_size(),
            self.fonts.texture_atlas().prepared_discs(),
        )
    }
}

impl Default for TextRasterizer {
    fn default() -> Self {
        Self::new()
    }
}

fn epaint_text_shape(
    text: &TextPaint,
    galley: std::sync::Arc<epaint::Galley>,
) -> epaint::TextShape {
    let mut shape = epaint::TextShape::new(
        epaint::pos2(text.rect.origin.x, text.rect.origin.y),
        galley,
        to_epaint_color(text.color),
    );
    shape.override_text_color = text.override_text_color.map(to_epaint_color);
    shape.opacity_factor = text.opacity_factor;
    shape.angle = text.angle;
    if let Some(underline) = text.underline {
        shape.underline =
            epaint::Stroke::new(underline.width.max(0.0), to_epaint_color(underline.color));
    }
    shape
}

fn tessellate_text_shape(
    tessellator: &mut epaint::Tessellator,
    text_shape: epaint::TextShape,
) -> epaint::Mesh {
    let primitives = tessellator.tessellate_shapes(vec![epaint::ClippedShape {
        clip_rect: epaint::Rect::EVERYTHING,
        shape: epaint::Shape::Text(text_shape),
    }]);
    let mut mesh = epaint::Mesh::default();
    for primitive in primitives {
        let epaint::Primitive::Mesh(primitive_mesh) = primitive.primitive else {
            continue;
        };
        if mesh.is_empty() {
            mesh.texture_id = primitive_mesh.texture_id;
        }
        mesh.append(primitive_mesh);
    }
    mesh
}

fn epaint_layout_job(text: &TextPaint) -> epaint::text::LayoutJob {
    let mut job = if let Some(selection) = text.selection {
        epaint_selection_layout_job(text, selection)
    } else {
        epaint::text::LayoutJob::single_section(
            text.text.clone(),
            epaint_text_format(text, text.color, None),
        )
    };
    job.wrap.max_width = match text.wrap_mode {
        TextWrapMode::Extend => f32::INFINITY,
        TextWrapMode::Wrap => text.wrap_width.max(1.0),
        TextWrapMode::Truncate => text.wrap_width.max(1.0),
    };
    job.wrap.max_rows = match text.wrap_mode {
        TextWrapMode::Truncate => text.max_lines.unwrap_or(1).max(1),
        _ => text.max_lines.unwrap_or(usize::MAX),
    };
    job.wrap.break_anywhere = text.wrap_mode == TextWrapMode::Truncate;
    job
}

fn epaint_selection_layout_job(
    text: &TextPaint,
    selection: des_ui_render::TextSelectionPaint,
) -> epaint::text::LayoutJob {
    let mut range_start = selection.anchor_index.min(selection.focus_index);
    let mut range_end = selection.anchor_index.max(selection.focus_index);
    let char_len = text.text.chars().count();
    range_start = range_start.min(char_len);
    range_end = range_end.min(char_len);
    if range_start == range_end {
        return epaint::text::LayoutJob::single_section(
            text.text.clone(),
            epaint_text_format(text, text.color, None),
        );
    }

    let byte_start = char_index_to_byte_index(&text.text, range_start);
    let byte_end = char_index_to_byte_index(&text.text, range_end);
    let mut sections = Vec::new();
    if byte_start > 0 {
        sections.push(epaint::text::LayoutSection {
            leading_space: 0.0,
            byte_range: 0..byte_start,
            format: epaint_text_format(text, text.color, None),
        });
    }
    sections.push(epaint::text::LayoutSection {
        leading_space: 0.0,
        byte_range: byte_start..byte_end,
        format: epaint_text_format(text, selection.color, Some(selection.background)),
    });
    if byte_end < text.text.len() {
        sections.push(epaint::text::LayoutSection {
            leading_space: 0.0,
            byte_range: byte_end..text.text.len(),
            format: epaint_text_format(text, text.color, None),
        });
    }

    epaint::text::LayoutJob {
        text: text.text.clone(),
        sections,
        break_on_newline: true,
        ..Default::default()
    }
}

fn epaint_text_format(
    text: &TextPaint,
    color: Color,
    background: Option<Color>,
) -> epaint::text::TextFormat {
    let mut format = epaint::text::TextFormat::simple(
        epaint::FontId::new(text.font_size, epaint::FontFamily::Proportional),
        to_epaint_color(color),
    );
    format.line_height = text.line_height;
    format.background = background.map_or(epaint::Color32::TRANSPARENT, to_epaint_color);
    format
}

fn char_index_to_byte_index(text: &str, char_index: usize) -> usize {
    text.char_indices()
        .map(|(byte_index, _)| byte_index)
        .nth(char_index)
        .unwrap_or(text.len())
}

fn to_epaint_color(color: Color) -> epaint::Color32 {
    epaint::Color32::from_rgba_unmultiplied(color.r, color.g, color.b, color.a)
}

fn texture_delta_from_epaint(delta: epaint::ImageDelta) -> TextureDelta {
    let width = delta.image.width() as u32;
    let height = delta.image.height() as u32;
    let pixels = match delta.image {
        epaint::ImageData::Color(image) => image
            .pixels
            .iter()
            .flat_map(|color| color.to_array())
            .collect(),
    };
    TextureDelta {
        pos: delta.pos.map(|[x, y]| [x as u32, y as u32]),
        width,
        height,
        options: delta.options.into(),
        pixels,
    }
}

fn text_atlas_delta_from_epaint(delta: epaint::ImageDelta) -> TextAtlasDelta {
    let delta = texture_delta_from_epaint(delta);
    TextAtlasDelta {
        pos: delta.pos,
        width: delta.width,
        height: delta.height,
        options: delta.options,
        pixels: delta.pixels,
    }
}

#[derive(Clone, Debug)]
pub struct DisplayListRenderer {
    options: RenderOptions,
    pixels_per_point: f32,
}

impl DisplayListRenderer {
    pub fn new(options: RenderOptions) -> Self {
        Self {
            options,
            pixels_per_point: 1.0,
        }
    }

    pub fn with_pixels_per_point(mut self, pixels_per_point: f32) -> Self {
        self.pixels_per_point = pixels_per_point.max(0.000_001);
        self
    }

    pub fn build_plan_for_output(&self, output: &des_ui_document::DocumentOutput) -> RenderPlan {
        self.build_plan(&des_ui_render::plan_paint(output))
    }

    pub fn build_plan(&self, display_list: &DisplayList) -> RenderPlan {
        let mut builder = RenderPlanBuilder::new(self.options);
        let text_rasterizer = TextRasterizer::with_text_options(self.options.text);
        let tessellator =
            text_rasterizer.shape_tessellator(self.pixels_per_point, self.options.tessellation);
        let primitives = PrimitivePlanner::new()
            .with_pixels_per_point(self.pixels_per_point)
            .with_tessellation_options(self.options.tessellation)
            .plan_display_list_with_tessellator(display_list, tessellator);
        builder.push_primitives(&primitives);
        builder.finish()
    }
}

impl Default for DisplayListRenderer {
    fn default() -> Self {
        Self::new(RenderOptions::default())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PackedColor([u8; 4]);

impl PackedColor {
    pub const fn to_array(self) -> [u8; 4] {
        self.0
    }

    pub const fn to_epaint_u32(self) -> u32 {
        let [r, g, b, a] = self.0;
        r as u32 | ((g as u32) << 8) | ((b as u32) << 16) | ((a as u32) << 24)
    }
}

impl From<Color> for PackedColor {
    fn from(color: Color) -> Self {
        Self(epaint::Color32::from_rgba_unmultiplied(color.r, color.g, color.b, color.a).to_array())
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 2],
    pub uv: [f32; 2],
    pub color: u32,
}

impl Vertex {
    pub const ATTRIBUTES: [wgpu::VertexAttribute; 3] =
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Uint32];

    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }

    pub fn from_epaint(vertex: epaint::Vertex) -> Self {
        Self {
            position: [vertex.pos.x, vertex.pos.y],
            uv: [vertex.uv.x, vertex.uv.y],
            color: PackedColor(vertex.color.to_array()).to_epaint_u32(),
        }
    }

    pub fn color_array(self) -> [u8; 4] {
        [
            (self.color & 255) as u8,
            ((self.color >> 8) & 255) as u8,
            ((self.color >> 16) & 255) as u8,
            ((self.color >> 24) & 255) as u8,
        ]
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
struct ViewportUniform {
    size: [f32; 2],
    dithering: u32,
    predictable_texture_filtering: u32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Mesh {
    pub texture_id: Option<epaint::TextureId>,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Mesh {
    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }

    pub fn from_epaint_mesh(mesh: &epaint::Mesh) -> Self {
        Self {
            texture_id: Some(mesh.texture_id),
            vertices: mesh
                .vertices
                .iter()
                .copied()
                .map(Vertex::from_epaint)
                .collect(),
            indices: mesh.indices.clone(),
        }
    }
}

#[derive(Clone, Debug, Default)]
struct MeshBuilder {
    mesh: Mesh,
}

impl MeshBuilder {
    fn new() -> Self {
        Self::default()
    }

    #[cfg(test)]
    fn push_display_list(&mut self, display_list: &DisplayList) {
        let primitives = des_ui_render::plan_primitives(display_list);
        for command in &primitives.commands {
            self.push_command(command);
        }
    }

    fn push_command(&mut self, command: &PrimitiveCommand) {
        match command {
            PrimitiveCommand::Draw(RenderPrimitive::Mesh(mesh)) => self.push_mesh(mesh),
            PrimitiveCommand::Draw(RenderPrimitive::Text(_))
            | PrimitiveCommand::PushClip(_)
            | PrimitiveCommand::PopClip => {}
        }
    }

    fn finish(self) -> Mesh {
        self.mesh
    }

    fn texture_id(&self) -> Option<epaint::TextureId> {
        self.mesh.texture_id
    }

    fn push_mesh(&mut self, primitive: &EpaintMeshPrimitive) {
        match self.mesh.texture_id {
            Some(texture_id) => assert_eq!(
                texture_id, primitive.mesh.texture_id,
                "single mesh batches cannot merge different epaint texture ids"
            ),
            None => {
                self.mesh.texture_id = Some(primitive.mesh.texture_id);
            }
        }
        let base = self.mesh.vertices.len() as u32;
        let mesh = Mesh::from_epaint_mesh(&primitive.mesh);
        self.mesh.vertices.extend(mesh.vertices);
        self.mesh
            .indices
            .extend(primitive.mesh.indices.iter().map(|index| base + *index));
    }
}

#[cfg(test)]
fn mesh_for_display_list(display_list: &DisplayList) -> Mesh {
    let mut builder = MeshBuilder::new();
    builder.push_display_list(display_list);
    builder.finish()
}

pub struct GpuRenderer<'window> {
    surface: wgpu::Surface<'window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    options: RenderOptions,
    pipeline: wgpu::RenderPipeline,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    viewport_buffer: wgpu::Buffer,
    viewport_bind_group: wgpu::BindGroup,
    previous_viewport_uniform: Option<ViewportUniform>,
    solid_texture: GpuTexture,
    text_rasterizer: RefCell<TextRasterizer>,
    text_atlas: Option<GpuTextAtlas>,
    frame_buffers: GpuFrameBuffers,
    textures: HashMap<epaint::TextureId, GpuRegisteredTexture>,
    samplers: HashMap<TextureOptions, wgpu::Sampler>,
    size: PhysicalRenderSize,
}

struct GpuTextAtlas {
    descriptor: TextureDescriptor,
    gpu: GpuTexture,
}

struct GpuRegisteredTexture {
    descriptor: TextureDescriptor,
    gpu: GpuTexture,
}

#[derive(Clone)]
struct GpuTexture {
    texture: wgpu::Texture,
    bind_group: wgpu::BindGroup,
}

impl GpuTexture {
    fn destroy(self) {
        self.texture.destroy();
    }
}

struct TextureUpload<'a> {
    width: u32,
    height: u32,
    pixels: &'a [u8],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct TextureDescriptor {
    width: u32,
    height: u32,
    options: TextureOptions,
}

impl TextureDescriptor {
    fn from_frame(frame: &RasterizedTextFrame, current: Option<Self>) -> Option<Self> {
        if frame.is_empty() && frame.atlas_delta.is_none() {
            return None;
        }
        let options = frame
            .atlas_delta
            .as_ref()
            .map(|delta| delta.options)
            .or_else(|| current.map(|descriptor| descriptor.options))
            .unwrap_or_default();
        Some(Self {
            width: frame.width,
            height: frame.height,
            options,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TextAtlasUpload {
    Skip,
    Unchanged(TextureDescriptor),
    Reuse(TextureDescriptor),
    Recreate(TextureDescriptor),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TextureDeltaUpload {
    Allocate(TextureDescriptor),
    Patch,
    PatchAndRebind(TextureDescriptor),
}

#[derive(Default)]
struct GpuFrameBuffers {
    vertex: Option<GpuBuffer>,
    index: Option<GpuBuffer>,
}

struct GpuBuffer {
    buffer: wgpu::Buffer,
    capacity: u64,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct UploadedFrame {
    draws: Vec<UploadedDraw>,
}

#[derive(Clone, Debug, PartialEq)]
struct UploadedDraw {
    clip: Option<Rect>,
    texture: DrawTexture,
    vertex_range: Range<u64>,
    index_range: Range<u64>,
    index_count: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DrawTexture {
    Solid,
    TextAtlas,
    Registered(epaint::TextureId),
}

fn draw_texture_for_mesh(mesh: &Mesh) -> DrawTexture {
    match mesh.texture_id {
        None | Some(epaint::TextureId::Managed(0)) if mesh_uses_only_white_uv(mesh) => {
            DrawTexture::Solid
        }
        None | Some(epaint::TextureId::Managed(0)) => DrawTexture::TextAtlas,
        Some(texture_id) => DrawTexture::Registered(texture_id),
    }
}

fn mesh_uses_only_white_uv(mesh: &Mesh) -> bool {
    let white = epaint::WHITE_UV;
    mesh.vertices
        .iter()
        .all(|vertex| vertex.uv == [white.x, white.y])
}

fn mesh_uses_paint_atlas(mesh: &Mesh) -> bool {
    matches!(mesh.texture_id, None | Some(epaint::TextureId::Managed(0)))
        && !mesh_uses_only_white_uv(mesh)
}

fn render_plan_needs_text_atlas(plan: &RenderPlan) -> bool {
    plan.items.iter().any(|item| match item {
        RenderItem::Text(_) => true,
        RenderItem::Mesh(batch) => mesh_uses_paint_atlas(&batch.mesh),
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BufferUpload {
    Skip,
    Reuse(u64),
    Recreate(u64),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FrameBufferSlot {
    Vertex,
    Index,
}

fn text_atlas_upload(
    current: Option<TextureDescriptor>,
    frame: &RasterizedTextFrame,
) -> TextAtlasUpload {
    let Some(next) = TextureDescriptor::from_frame(frame, current) else {
        return TextAtlasUpload::Skip;
    };
    if current == Some(next) {
        if frame.atlas_delta.is_some() {
            TextAtlasUpload::Reuse(next)
        } else {
            TextAtlasUpload::Unchanged(next)
        }
    } else {
        TextAtlasUpload::Recreate(next)
    }
}

fn texture_delta_upload(
    current: Option<TextureDescriptor>,
    delta: &TextureDelta,
    texture_id: epaint::TextureId,
) -> Result<TextureDeltaUpload, RendererError> {
    let next = TextureDescriptor {
        width: delta.width,
        height: delta.height,
        options: delta.options,
    };
    if delta.pos.is_none() {
        return Ok(TextureDeltaUpload::Allocate(next));
    }
    let Some(current) = current else {
        return Err(RendererError::PartialTextureUpdateMissingTexture(
            texture_id,
        ));
    };
    if current.options == delta.options {
        Ok(TextureDeltaUpload::Patch)
    } else {
        Ok(TextureDeltaUpload::PatchAndRebind(TextureDescriptor {
            options: delta.options,
            ..current
        }))
    }
}

fn buffer_upload(current_capacity: Option<u64>, required_size: u64) -> BufferUpload {
    if required_size == 0 {
        return BufferUpload::Skip;
    }
    let Some(current_capacity) = current_capacity else {
        return BufferUpload::Recreate(required_size);
    };
    if current_capacity >= required_size {
        BufferUpload::Reuse(current_capacity)
    } else {
        BufferUpload::Recreate((current_capacity * 2).max(required_size))
    }
}

fn append_mesh_draw<V: Copy>(
    vertices: &mut Vec<V>,
    indices: &mut Vec<u32>,
    clip: Option<Rect>,
    texture: DrawTexture,
    next_vertices: &[V],
    next_indices: &[u32],
) -> UploadedDraw {
    let vertex_start = (vertices.len() * mem::size_of::<V>()) as u64;
    let index_start = (indices.len() * mem::size_of::<u32>()) as u64;
    vertices.extend_from_slice(next_vertices);
    indices.extend_from_slice(next_indices);
    UploadedDraw {
        clip,
        texture,
        vertex_range: vertex_start..(vertices.len() * mem::size_of::<V>()) as u64,
        index_range: index_start..(indices.len() * mem::size_of::<u32>()) as u64,
        index_count: next_indices.len() as u32,
    }
}

fn ensure_frame_buffer(
    device: &wgpu::Device,
    buffers: &mut GpuFrameBuffers,
    slot: FrameBufferSlot,
    label: &'static str,
    usage: wgpu::BufferUsages,
    required_size: u64,
) {
    let current_capacity = match slot {
        FrameBufferSlot::Vertex => buffers.vertex.as_ref().map(|buffer| buffer.capacity),
        FrameBufferSlot::Index => buffers.index.as_ref().map(|buffer| buffer.capacity),
    };
    let capacity = match buffer_upload(current_capacity, required_size) {
        BufferUpload::Skip | BufferUpload::Reuse(_) => return,
        BufferUpload::Recreate(capacity) => capacity,
    };
    let buffer = GpuBuffer {
        buffer: device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: capacity,
            usage,
            mapped_at_creation: false,
        }),
        capacity,
    };
    match slot {
        FrameBufferSlot::Vertex => buffers.vertex = Some(buffer),
        FrameBufferSlot::Index => buffers.index = Some(buffer),
    }
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
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
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
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("des-ui texture bind group layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });
        let mut samplers = HashMap::new();
        let texture_sampler = cached_sampler(
            &device,
            &mut samplers,
            "des-ui texture sampler",
            TextureOptions::default(),
        );
        let pipeline = create_pipeline(
            &device,
            config.format,
            &viewport_bind_group_layout,
            &texture_bind_group_layout,
        );
        let solid_texture = create_rgba_texture(
            &device,
            &queue,
            &texture_bind_group_layout,
            &texture_sampler,
            "des-ui solid white texture",
            TextureUpload {
                width: 1,
                height: 1,
                pixels: &[255, 255, 255, 255],
            },
        );
        let mut renderer = Self {
            surface,
            device,
            queue,
            config,
            options,
            pipeline,
            texture_bind_group_layout,
            viewport_buffer,
            viewport_bind_group,
            previous_viewport_uniform: None,
            solid_texture,
            text_rasterizer: RefCell::new(TextRasterizer::with_text_options(options.text)),
            text_atlas: None,
            frame_buffers: GpuFrameBuffers::default(),
            textures: HashMap::new(),
            samplers,
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

    pub fn set_texture(&mut self, texture_id: epaint::TextureId, texture: RegisteredTexture) {
        let delta = TextureDelta {
            pos: None,
            width: texture.width,
            height: texture.height,
            options: texture.options,
            pixels: texture.pixels,
        };
        self.apply_texture_delta(texture_id, &delta)
            .expect("full texture uploads should always allocate");
    }

    pub fn apply_textures_delta(
        &mut self,
        delta: epaint::textures::TexturesDelta,
    ) -> Result<(), RendererError> {
        for (texture_id, image_delta) in delta.set {
            self.apply_epaint_texture_delta(texture_id, image_delta)?;
        }
        for texture_id in delta.free {
            self.remove_texture(texture_id);
        }
        Ok(())
    }

    pub fn apply_epaint_texture_delta(
        &mut self,
        texture_id: epaint::TextureId,
        delta: epaint::ImageDelta,
    ) -> Result<(), RendererError> {
        let delta = texture_delta_from_epaint(delta);
        self.apply_texture_delta(texture_id, &delta)
    }

    pub fn remove_texture(&mut self, texture_id: epaint::TextureId) {
        if let Some(texture) = self.textures.remove(&texture_id) {
            texture.gpu.destroy();
        }
    }

    fn apply_texture_delta(
        &mut self,
        texture_id: epaint::TextureId,
        delta: &TextureDelta,
    ) -> Result<(), RendererError> {
        let current = self
            .textures
            .get(&texture_id)
            .map(|texture| texture.descriptor);
        match texture_delta_upload(current, delta, texture_id)? {
            TextureDeltaUpload::Patch => {
                let texture = self.textures.get_mut(&texture_id).ok_or(
                    RendererError::PartialTextureUpdateMissingTexture(texture_id),
                )?;
                write_rgba_texture_delta(&self.queue, &texture.gpu.texture, delta);
            }
            TextureDeltaUpload::PatchAndRebind(descriptor) => {
                let sampler = cached_sampler(
                    &self.device,
                    &mut self.samplers,
                    "des-ui registered texture sampler",
                    descriptor.options,
                );
                let texture = self.textures.get_mut(&texture_id).ok_or(
                    RendererError::PartialTextureUpdateMissingTexture(texture_id),
                )?;
                texture.gpu.bind_group = create_texture_bind_group(
                    &self.device,
                    &self.texture_bind_group_layout,
                    &texture.gpu.texture,
                    &sampler,
                    "des-ui registered texture",
                );
                texture.descriptor = descriptor;
                write_rgba_texture_delta(&self.queue, &texture.gpu.texture, delta);
            }
            TextureDeltaUpload::Allocate(descriptor) => {
                let sampler = cached_sampler(
                    &self.device,
                    &mut self.samplers,
                    "des-ui registered texture sampler",
                    descriptor.options,
                );
                let gpu = create_rgba_texture(
                    &self.device,
                    &self.queue,
                    &self.texture_bind_group_layout,
                    &sampler,
                    "des-ui registered texture",
                    TextureUpload {
                        width: delta.width,
                        height: delta.height,
                        pixels: &delta.pixels,
                    },
                );
                if let Some(old) = self
                    .textures
                    .insert(texture_id, GpuRegisteredTexture { descriptor, gpu })
                {
                    old.gpu.destroy();
                }
            }
        }
        Ok(())
    }

    pub fn render_plan(&mut self, plan: &RenderPlan) -> Result<(), RendererError> {
        if self.size.is_empty() {
            return Ok(());
        }
        self.write_viewport_uniform();
        let text_paints = plan
            .text_batches
            .iter()
            .map(|batch| batch.text.clone())
            .collect::<Vec<_>>();
        let text_frame = if render_plan_needs_text_atlas(plan) {
            self.text_rasterizer
                .borrow_mut()
                .rasterize_frame_with_options(
                    &text_paints,
                    self.size.scale_factor as f32,
                    self.options.text,
                    self.options.tessellation,
                )
        } else {
            RasterizedTextFrame::default()
        };
        let uploaded_frame = self.upload_render_frame(plan, &text_frame)?;
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
        let text_bind_group = self.upload_text_frame(&text_frame);
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
            for draw in &uploaded_frame.draws {
                match draw.texture {
                    DrawTexture::Solid => {
                        self.draw_batch(&mut pass, draw, &self.solid_texture.bind_group);
                    }
                    DrawTexture::TextAtlas => {
                        if let Some(bind_group) = text_bind_group.as_ref() {
                            self.draw_batch(&mut pass, draw, bind_group);
                        }
                    }
                    DrawTexture::Registered(texture_id) => {
                        if should_draw_registered_texture(self.textures.contains_key(&texture_id)) {
                            let texture = &self.textures[&texture_id];
                            self.draw_batch(&mut pass, draw, &texture.gpu.bind_group);
                        }
                    }
                }
            }
        }
        self.queue.submit([encoder.finish()]);
        frame.present();
        Ok(())
    }
    fn upload_render_frame(
        &mut self,
        plan: &RenderPlan,
        frame: &RasterizedTextFrame,
    ) -> Result<UploadedFrame, RendererError> {
        let (uploaded_frame, vertices, indices) = build_uploaded_frame(plan, frame)?;
        self.upload_frame_buffers(
            "des-ui frame vertex buffer",
            "des-ui frame index buffer",
            &vertices,
            &indices,
        );
        Ok(uploaded_frame)
    }

    fn upload_frame_buffers<V: bytemuck::Pod>(
        &mut self,
        vertex_label: &'static str,
        index_label: &'static str,
        vertices: &[V],
        indices: &[u32],
    ) {
        let required_vertex_size = mem::size_of_val(vertices) as u64;
        let required_index_size = mem::size_of_val(indices) as u64;
        let device = self.device.clone();
        ensure_frame_buffer(
            &device,
            &mut self.frame_buffers,
            FrameBufferSlot::Vertex,
            vertex_label,
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            required_vertex_size,
        );
        ensure_frame_buffer(
            &device,
            &mut self.frame_buffers,
            FrameBufferSlot::Index,
            index_label,
            wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            required_index_size,
        );
        if required_vertex_size > 0
            && let Some(buffer) = self.frame_buffers.vertex.as_ref()
        {
            self.queue
                .write_buffer(&buffer.buffer, 0, bytemuck::cast_slice(vertices));
        }
        if required_index_size > 0
            && let Some(buffer) = self.frame_buffers.index.as_ref()
        {
            self.queue
                .write_buffer(&buffer.buffer, 0, bytemuck::cast_slice(indices));
        }
    }

    fn draw_batch<'pass>(
        &'pass self,
        pass: &mut wgpu::RenderPass<'pass>,
        draw: &'pass UploadedDraw,
        bind_group: &'pass wgpu::BindGroup,
    ) {
        if draw.index_count == 0 {
            return;
        }
        let Some(scissor) = clip_rect_to_scissor(draw.clip, self.size) else {
            return;
        };
        let Some(vertex_buffer) = self.frame_buffers.vertex.as_ref() else {
            return;
        };
        let Some(index_buffer) = self.frame_buffers.index.as_ref() else {
            return;
        };
        pass.set_scissor_rect(scissor.x, scissor.y, scissor.width, scissor.height);
        pass.set_bind_group(1, bind_group, &[]);
        pass.set_vertex_buffer(0, vertex_buffer.buffer.slice(draw.vertex_range.clone()));
        pass.set_index_buffer(
            index_buffer.buffer.slice(draw.index_range.clone()),
            wgpu::IndexFormat::Uint32,
        );
        pass.draw_indexed(0..draw.index_count, 0, 0..1);
    }

    fn upload_text_frame(&mut self, frame: &RasterizedTextFrame) -> Option<wgpu::BindGroup> {
        let current = self.text_atlas.as_ref().map(|atlas| atlas.descriptor);
        match text_atlas_upload(current, frame) {
            TextAtlasUpload::Skip => return None,
            TextAtlasUpload::Unchanged(_) => {}
            TextAtlasUpload::Reuse(_) => {}
            TextAtlasUpload::Recreate(descriptor) => {
                if let Some(atlas) = self.text_atlas.take() {
                    atlas.gpu.destroy();
                }
                self.text_atlas = Some(self.create_text_atlas(descriptor));
            }
        }
        if let Some(delta) = frame.atlas_delta.as_ref() {
            let atlas = self.text_atlas.as_ref()?;
            write_text_atlas_delta(&self.queue, &atlas.gpu.texture, delta);
        }
        Some(self.text_atlas.as_ref()?.gpu.bind_group.clone())
    }

    fn create_text_atlas(&mut self, descriptor: TextureDescriptor) -> GpuTextAtlas {
        let sampler = cached_sampler(
            &self.device,
            &mut self.samplers,
            "des-ui text atlas sampler",
            descriptor.options,
        );
        let gpu = create_rgba_texture(
            &self.device,
            &self.queue,
            &self.texture_bind_group_layout,
            &sampler,
            "des-ui text atlas texture",
            TextureUpload {
                width: descriptor.width,
                height: descriptor.height,
                pixels: &[],
            },
        );
        GpuTextAtlas { descriptor, gpu }
    }

    fn write_viewport_uniform(&mut self) {
        let uniform = viewport_uniform(self.size, self.options);
        if !should_write_viewport_uniform(self.previous_viewport_uniform, uniform) {
            return;
        }
        self.queue
            .write_buffer(&self.viewport_buffer, 0, bytemuck::bytes_of(&uniform));
        self.previous_viewport_uniform = Some(uniform);
    }
}

fn build_uploaded_frame(
    plan: &RenderPlan,
    frame: &RasterizedTextFrame,
) -> Result<(UploadedFrame, Vec<Vertex>, Vec<u32>), RendererError> {
    let expected_text_meshes = plan
        .items
        .iter()
        .filter(|item| matches!(item, RenderItem::Text(_)))
        .count();
    if frame.batches.len() != expected_text_meshes {
        return Err(RendererError::TextMeshCountMismatch {
            expected: expected_text_meshes,
            actual: frame.batches.len(),
        });
    }
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut draws = Vec::new();
    let mut text_meshes = frame.batches.iter();
    for item in &plan.items {
        match item {
            RenderItem::Mesh(batch) => {
                let texture = draw_texture_for_mesh(&batch.mesh);
                draws.push(append_mesh_draw(
                    &mut vertices,
                    &mut indices,
                    batch.clip,
                    texture,
                    &batch.mesh.vertices,
                    &batch.mesh.indices,
                ));
            }
            RenderItem::Text(batch) => {
                if let Some(mesh) = text_meshes.next() {
                    draws.push(append_mesh_draw(
                        &mut vertices,
                        &mut indices,
                        batch.clip,
                        DrawTexture::TextAtlas,
                        &mesh.vertices,
                        &mesh.indices,
                    ));
                }
            }
        }
    }
    Ok((UploadedFrame { draws }, vertices, indices))
}

fn viewport_uniform(size: PhysicalRenderSize, options: RenderOptions) -> ViewportUniform {
    ViewportUniform {
        size: [
            size.logical_width().max(1.0),
            size.logical_height().max(1.0),
        ],
        dithering: u32::from(options.dithering),
        predictable_texture_filtering: u32::from(options.predictable_texture_filtering),
    }
}

fn should_write_viewport_uniform(previous: Option<ViewportUniform>, next: ViewportUniform) -> bool {
    previous != Some(next)
}

fn should_draw_registered_texture(is_registered: bool) -> bool {
    is_registered
}

fn write_text_atlas_delta(queue: &wgpu::Queue, texture: &wgpu::Texture, delta: &TextAtlasDelta) {
    write_rgba_texture_patch(
        queue,
        texture,
        delta.pos,
        delta.width,
        delta.height,
        &delta.pixels,
    );
}

fn write_rgba_texture_delta(queue: &wgpu::Queue, texture: &wgpu::Texture, delta: &TextureDelta) {
    write_rgba_texture_patch(
        queue,
        texture,
        delta.pos,
        delta.width,
        delta.height,
        &delta.pixels,
    );
}

fn write_rgba_texture_patch(
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    pos: Option<[u32; 2]>,
    width: u32,
    height: u32,
    pixels: &[u8],
) {
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d {
                x: pos.map_or(0, |pos| pos[0]),
                y: pos.map_or(0, |pos| pos[1]),
                z: 0,
            },
            aspect: wgpu::TextureAspect::All,
        },
        pixels,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(width * 4),
            rows_per_image: Some(height),
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
}

fn create_texture_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    texture: &wgpu::Texture,
    sampler: &wgpu::Sampler,
    label: &'static str,
) -> wgpu::BindGroup {
    let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(label),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
        ],
    })
}

fn cached_sampler(
    device: &wgpu::Device,
    samplers: &mut HashMap<TextureOptions, wgpu::Sampler>,
    label: &'static str,
    options: TextureOptions,
) -> wgpu::Sampler {
    samplers
        .entry(options)
        .or_insert_with(|| device.create_sampler(&texture_sampler_descriptor(label, options)))
        .clone()
}

fn texture_sampler_descriptor(
    label: &'static str,
    options: TextureOptions,
) -> wgpu::SamplerDescriptor<'static> {
    wgpu::SamplerDescriptor {
        label: Some(label),
        address_mode_u: options.wrap_mode.into(),
        address_mode_v: options.wrap_mode.into(),
        address_mode_w: options.wrap_mode.into(),
        mag_filter: options.magnification.into(),
        min_filter: options.minification.into(),
        mipmap_filter: options
            .mipmap_mode
            .map_or(wgpu::MipmapFilterMode::Nearest, Into::into),
        ..Default::default()
    }
}

fn create_rgba_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
    sampler: &wgpu::Sampler,
    label: &'static str,
    upload: TextureUpload<'_>,
) -> GpuTexture {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width: upload.width,
            height: upload.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    if !upload.pixels.is_empty() {
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            upload.pixels,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(upload.width * 4),
                rows_per_image: Some(upload.height),
            },
            wgpu::Extent3d {
                width: upload.width,
                height: upload.height,
                depth_or_array_layers: 1,
            },
        );
    }
    let bind_group = create_texture_bind_group(device, layout, &texture, sampler, label);
    GpuTexture {
        texture,
        bind_group,
    }
}

fn create_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    viewport_bind_group_layout: &wgpu::BindGroupLayout,
    texture_bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("des-ui textured mesh shader"),
        source: wgpu::ShaderSource::Wgsl(TEXTURED_SHADER.into()),
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("des-ui textured mesh pipeline layout"),
        bind_group_layouts: &[
            Some(viewport_bind_group_layout),
            Some(texture_bind_group_layout),
        ],
        immediate_size: 0,
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("des-ui textured mesh pipeline"),
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
            entry_point: Some(fragment_entry_point(format)),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(premultiplied_alpha_blend()),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview_mask: None,
        cache: None,
    })
}

fn premultiplied_alpha_blend() -> wgpu::BlendState {
    wgpu::BlendState {
        color: wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::One,
            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
            operation: wgpu::BlendOperation::Add,
        },
        alpha: wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::OneMinusDstAlpha,
            dst_factor: wgpu::BlendFactor::One,
            operation: wgpu::BlendOperation::Add,
        },
    }
}

fn fragment_entry_point(format: wgpu::TextureFormat) -> &'static str {
    if format.is_srgb() {
        "fs_main_linear_framebuffer"
    } else {
        "fs_main_gamma_framebuffer"
    }
}

pub fn clip_rect_to_scissor(clip: Option<Rect>, size: PhysicalRenderSize) -> Option<ScissorRect> {
    if size.is_empty() {
        return None;
    }
    let scale = size.scale_factor as f32;
    let (left, top, right, bottom) = if let Some(clip) = clip {
        let screen_width = size.width as i32;
        let screen_height = size.height as i32;
        let left = (clip.origin.x * scale).round() as i32;
        let top = (clip.origin.y * scale).round() as i32;
        let right = (clip.right() * scale).round() as i32;
        let bottom = (clip.bottom() * scale).round() as i32;
        let left = left.clamp(0, screen_width);
        let right = right.clamp(left, screen_width);
        let top = top.clamp(0, screen_height);
        let bottom = bottom.clamp(top, screen_height);
        (left, top, right, bottom)
    } else {
        (0, 0, size.width as i32, size.height as i32)
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
    current_clip_empty: bool,
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
            current_clip_empty: false,
            current_mesh: MeshBuilder::new(),
        }
    }

    fn push_primitives(&mut self, primitives: &PrimitiveList) {
        let mut clip_stack: Vec<Rect> = Vec::new();
        for command in &primitives.commands {
            match command {
                PrimitiveCommand::PushClip(rect) => {
                    clip_stack.push(*rect);
                    self.set_clip(active_clip(&clip_stack));
                }
                PrimitiveCommand::PopClip => {
                    clip_stack.pop();
                    self.set_clip(active_clip(&clip_stack));
                }
                PrimitiveCommand::Draw(RenderPrimitive::Text(text)) => {
                    self.set_clip(active_clip(&clip_stack));
                    self.flush();
                    if self.current_clip_empty {
                        continue;
                    }
                    let batch = TextBatch {
                        clip: self.current_clip,
                        text: text.clone(),
                    };
                    self.plan.items.push(RenderItem::Text(batch.clone()));
                    self.plan.text_batches.push(batch);
                }
                PrimitiveCommand::Draw(RenderPrimitive::Mesh(mesh)) => {
                    if !self.current_clip_empty {
                        self.set_clip(match mesh.clip {
                            Some(rect) => ClipState::Clipped(rect),
                            None => ClipState::Unclipped,
                        });
                        if self
                            .current_mesh
                            .texture_id()
                            .is_some_and(|texture_id| texture_id != mesh.mesh.texture_id)
                        {
                            self.flush();
                        }
                        self.current_mesh.push_command(command);
                    }
                }
            }
        }
    }

    fn set_clip(&mut self, clip: ClipState) {
        let (next_clip, next_empty) = match clip {
            ClipState::Unclipped => (None, false),
            ClipState::Clipped(rect) => (Some(rect), false),
            ClipState::Empty => (None, true),
        };
        if self.current_clip == next_clip && self.current_clip_empty == next_empty {
            return;
        }
        self.flush();
        self.current_clip = next_clip;
        self.current_clip_empty = next_empty;
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

#[derive(Clone, Copy, Debug, PartialEq)]
enum ClipState {
    Unclipped,
    Clipped(Rect),
    Empty,
}

fn active_clip(clip_stack: &[Rect]) -> ClipState {
    let mut clips = clip_stack.iter().copied();
    let Some(first) = clips.next() else {
        return ClipState::Unclipped;
    };
    match clips.try_fold(first, Rect::intersect) {
        Some(rect) => ClipState::Clipped(rect),
        None => ClipState::Empty,
    }
}

#[cfg(test)]
mod tests {
    use std::mem;

    use des_ui_document::{
        Color, CornerRadii, Document, DocumentEngine, Element, ElementId, Point, Rect, Size, Style,
        StyleSelector, StyleSheet, TextWrapMode,
    };
    use des_ui_render::{
        DisplayList, FillCirclePaint, FillRectPaint, PaintCommand, RenderTessellationOptions,
        TextPaint, TextSelectionPaint, TextUnderlinePaint,
    };

    use crate::{
        ClearColor, DisplayListRenderer, Mesh, MeshBuilder, PackedColor, PhysicalRenderSize,
        RenderAlphaFromCoverage, RenderItem, RenderOptions, RenderTextOptions, RendererError,
        ScissorRect, TextRasterizer, TextureFilter, TextureOptions, TextureWrapMode, Vertex,
        build_uploaded_frame, clip_rect_to_scissor, epaint_layout_job, mesh_for_display_list,
        mesh_uses_paint_atlas, render_plan_needs_text_atlas, text_atlas_upload,
    };

    #[test]
    fn packed_color_preserves_rgba_channel_order() {
        let color = PackedColor::from(Color::rgba(10, 20, 30, 40));
        let expected = epaint::Color32::from_rgba_unmultiplied(10, 20, 30, 40).to_array();

        assert_eq!(color.to_array(), expected);
        assert_eq!(
            color.to_epaint_u32(),
            u32::from(expected[0])
                | (u32::from(expected[1]) << 8)
                | (u32::from(expected[2]) << 16)
                | (u32::from(expected[3]) << 24)
        );
    }

    #[test]
    fn packed_color_from_document_color_uses_epaint_premultiplied_alpha() {
        let color = PackedColor::from(Color::rgba(120, 80, 40, 128));

        assert_eq!(
            color.to_array(),
            epaint::Color32::from_rgba_unmultiplied(120, 80, 40, 128).to_array()
        );
        assert_ne!(
            color.to_array(),
            [120, 80, 40, 128],
            "epaint vertices store premultiplied color, not straight RGBA"
        );
    }

    #[test]
    fn blend_state_matches_epaint_premultiplied_alpha_contract() {
        let blend = crate::premultiplied_alpha_blend();

        assert_eq!(blend.color.src_factor, wgpu::BlendFactor::One);
        assert_eq!(blend.color.dst_factor, wgpu::BlendFactor::OneMinusSrcAlpha);
        assert_eq!(blend.alpha.src_factor, wgpu::BlendFactor::OneMinusDstAlpha);
        assert_eq!(blend.alpha.dst_factor, wgpu::BlendFactor::One);
    }

    #[test]
    fn fragment_entry_point_matches_epaint_framebuffer_color_contract() {
        assert_eq!(
            crate::fragment_entry_point(wgpu::TextureFormat::Bgra8UnormSrgb),
            "fs_main_linear_framebuffer"
        );
        assert_eq!(
            crate::fragment_entry_point(wgpu::TextureFormat::Bgra8Unorm),
            "fs_main_gamma_framebuffer"
        );
    }

    #[test]
    fn render_options_default_to_epaint_style_dithering() {
        assert!(RenderOptions::default().dithering);
        assert!(!RenderOptions::default().predictable_texture_filtering);
        assert_eq!(
            RenderOptions::default().tessellation,
            RenderTessellationOptions::default()
        );
        assert_eq!(RenderOptions::default().text, RenderTextOptions::default());
    }

    #[test]
    fn text_options_default_to_epaint_contract() {
        let epaint_defaults = epaint::TextOptions::default();

        assert_eq!(
            RenderTextOptions::default(),
            RenderTextOptions {
                max_texture_side: epaint_defaults.max_texture_side,
                alpha_from_coverage: RenderAlphaFromCoverage::from(
                    epaint_defaults.alpha_from_coverage
                ),
                font_hinting: epaint_defaults.font_hinting,
            }
        );
    }

    #[test]
    fn text_options_convert_to_epaint_text_contract() {
        let options = RenderTextOptions {
            max_texture_side: 4096,
            alpha_from_coverage: RenderAlphaFromCoverage::Gamma(0.75),
            font_hinting: false,
        };
        let epaint_options = epaint::TextOptions::from(options);

        assert_eq!(epaint_options.max_texture_side, 4096);
        assert_eq!(
            epaint_options.alpha_from_coverage,
            epaint::AlphaFromCoverage::Gamma(0.75)
        );
        assert!(!epaint_options.font_hinting);
    }

    #[test]
    fn viewport_uniform_carries_dithering_to_fragment_shader() {
        let size = PhysicalRenderSize::new(200, 100, 2.0);
        let enabled = crate::viewport_uniform(size, RenderOptions::default());
        let disabled = crate::viewport_uniform(
            size,
            RenderOptions {
                dithering: false,
                ..RenderOptions::default()
            },
        );

        assert_eq!(enabled.size, [100.0, 50.0]);
        assert_eq!(enabled.dithering, 1);
        assert_eq!(enabled.predictable_texture_filtering, 0);
        assert_eq!(disabled.dithering, 0);
    }

    #[test]
    fn viewport_uniform_carries_predictable_texture_filtering_to_fragment_shader() {
        let uniform = crate::viewport_uniform(
            PhysicalRenderSize::new(320, 180, 1.0),
            RenderOptions {
                predictable_texture_filtering: true,
                ..RenderOptions::default()
            },
        );

        assert_eq!(uniform.predictable_texture_filtering, 1);
    }

    #[test]
    fn viewport_uniform_upload_skips_when_epaint_viewport_contract_is_unchanged() {
        let first = crate::viewport_uniform(
            PhysicalRenderSize::new(320, 180, 2.0),
            RenderOptions::default(),
        );
        let same = crate::viewport_uniform(
            PhysicalRenderSize::new(320, 180, 2.0),
            RenderOptions::default(),
        );
        let resized = crate::viewport_uniform(
            PhysicalRenderSize::new(640, 180, 2.0),
            RenderOptions::default(),
        );
        let changed_filtering = crate::viewport_uniform(
            PhysicalRenderSize::new(320, 180, 2.0),
            RenderOptions {
                predictable_texture_filtering: true,
                ..RenderOptions::default()
            },
        );

        assert!(crate::should_write_viewport_uniform(None, first));
        assert!(!crate::should_write_viewport_uniform(Some(first), same));
        assert!(crate::should_write_viewport_uniform(Some(first), resized));
        assert!(crate::should_write_viewport_uniform(
            Some(first),
            changed_filtering
        ));
    }

    #[test]
    fn shader_contains_epaint_style_predictable_texture_filtering_paths() {
        assert!(crate::TEXTURED_SHADER.contains("textureSample"));
        assert!(crate::TEXTURED_SHADER.contains("textureLoad"));
        assert!(crate::TEXTURED_SHADER.contains("predictable_texture_filtering == 0u"));
    }

    #[test]
    fn shader_uses_epaint_wgpu_style_uniform_and_texture_groups() {
        assert!(crate::TEXTURED_SHADER.contains("@group(0) @binding(0)\nvar<uniform> viewport"));
        assert!(crate::TEXTURED_SHADER.contains("@group(1) @binding(0)\nvar paint_texture"));
        assert!(crate::TEXTURED_SHADER.contains("@group(1) @binding(1)\nvar paint_sampler"));
    }

    #[test]
    fn fill_rect_uses_epaint_tessellation_in_document_coordinates() {
        let mut display_list = DisplayList::new();
        display_list.push(PaintCommand::FillRect(FillRectPaint {
            element_id: ElementId::new("box"),
            rect: Rect::new(10.0, 20.0, 30.0, 40.0),
            radius: CornerRadii::ZERO,
            color: Color::rgb(1, 2, 3),
        }));
        let mut builder = MeshBuilder::new();
        builder.push_display_list(&display_list);

        let mesh = builder.finish();
        assert!(mesh.vertices.len() >= 8);
        assert!(mesh.indices.len() >= 30);
        assert!(
            mesh.vertices
                .iter()
                .any(|vertex| vertex.color_array() == [1, 2, 3, 255])
        );
        assert!(
            mesh.vertices
                .iter()
                .any(|vertex| vertex.color_array()[3] == 0)
        );
        assert!(
            mesh.vertices
                .iter()
                .any(|vertex| vertex.position[0] > 10.0 && vertex.position[1] > 20.0),
            "inner fill vertices should inset by half the antialiasing fringe"
        );
        assert!(
            mesh.vertices
                .iter()
                .any(|vertex| vertex.position[0] < 10.0 && vertex.position[1] < 20.0),
            "outer fringe vertices should expand outside the filled edge"
        );
    }

    #[test]
    fn mesh_builder_preserves_epaint_vertex_payload() {
        let mut epaint_mesh = epaint::Mesh::default();
        let color = epaint::Color32::from_rgba_unmultiplied(10, 20, 30, 40);
        epaint_mesh.vertices.push(epaint::Vertex {
            pos: epaint::pos2(4.0, 8.0),
            uv: epaint::pos2(0.25, 0.75),
            color,
        });
        epaint_mesh.indices.push(0);

        let mut builder = MeshBuilder::new();
        builder.push_command(&des_ui_render::PrimitiveCommand::Draw(
            des_ui_render::RenderPrimitive::Mesh(des_ui_render::EpaintMeshPrimitive {
                element_id: ElementId::new("payload"),
                clip: None,
                mesh: epaint_mesh,
            }),
        ));

        let mesh = builder.finish();
        assert_eq!(mesh.texture_id, Some(epaint::TextureId::Managed(0)));
        assert_eq!(mesh.vertices[0].position, [4.0, 8.0]);
        assert_eq!(mesh.vertices[0].uv, [0.25, 0.75]);
        assert_eq!(mesh.vertices[0].color_array(), color.to_array());
    }

    #[test]
    fn render_plan_splits_epaint_meshes_by_texture_id() {
        fn primitive(texture_id: epaint::TextureId) -> des_ui_render::PrimitiveCommand {
            let mut mesh = epaint::Mesh {
                texture_id,
                ..epaint::Mesh::default()
            };
            mesh.vertices.push(epaint::Vertex {
                pos: epaint::pos2(0.0, 0.0),
                uv: epaint::pos2(0.0, 0.0),
                color: epaint::Color32::WHITE,
            });
            mesh.indices.push(0);
            des_ui_render::PrimitiveCommand::Draw(des_ui_render::RenderPrimitive::Mesh(
                des_ui_render::EpaintMeshPrimitive {
                    element_id: ElementId::new("textured"),
                    clip: None,
                    mesh,
                },
            ))
        }

        let mut primitives = des_ui_render::PrimitiveList::new();
        primitives.push(primitive(epaint::TextureId::Managed(0)));
        primitives.push(primitive(epaint::TextureId::Managed(1)));
        let mut builder = crate::RenderPlanBuilder::new(RenderOptions::default());
        builder.push_primitives(&primitives);

        let plan = builder.finish();
        assert_eq!(plan.batches.len(), 2);
        assert_eq!(
            plan.batches[0].mesh.texture_id,
            Some(epaint::TextureId::Managed(0))
        );
        assert_eq!(
            plan.batches[1].mesh.texture_id,
            Some(epaint::TextureId::Managed(1))
        );
    }

    #[test]
    fn render_plan_uses_epaint_mesh_clip_metadata() {
        let mut mesh = epaint::Mesh::default();
        mesh.vertices.push(epaint::Vertex {
            pos: epaint::pos2(0.0, 0.0),
            uv: epaint::pos2(0.0, 0.0),
            color: epaint::Color32::WHITE,
        });
        mesh.indices.push(0);

        let mut primitives = des_ui_render::PrimitiveList::new();
        primitives.push(des_ui_render::PrimitiveCommand::Draw(
            des_ui_render::RenderPrimitive::Mesh(des_ui_render::EpaintMeshPrimitive {
                element_id: ElementId::new("clipped"),
                clip: Some(Rect::new(3.0, 4.0, 50.0, 60.0)),
                mesh,
            }),
        ));
        let mut builder = crate::RenderPlanBuilder::new(RenderOptions::default());
        builder.push_primitives(&primitives);

        let plan = builder.finish();
        assert_eq!(plan.batches.len(), 1);
        assert_eq!(plan.batches[0].clip, Some(Rect::new(3.0, 4.0, 50.0, 60.0)));
    }

    #[test]
    fn render_plan_text_uses_document_clip_after_mesh_clip_metadata() {
        let mut mesh = epaint::Mesh::default();
        mesh.vertices.push(epaint::Vertex {
            pos: epaint::pos2(0.0, 0.0),
            uv: epaint::pos2(0.0, 0.0),
            color: epaint::Color32::WHITE,
        });
        mesh.indices.push(0);

        let document_clip = Rect::new(1.0, 2.0, 100.0, 80.0);
        let mesh_clip = Rect::new(10.0, 20.0, 30.0, 40.0);
        let mut primitives = des_ui_render::PrimitiveList::new();
        primitives.push(des_ui_render::PrimitiveCommand::PushClip(document_clip));
        primitives.push(des_ui_render::PrimitiveCommand::Draw(
            des_ui_render::RenderPrimitive::Mesh(des_ui_render::EpaintMeshPrimitive {
                element_id: ElementId::new("mesh"),
                clip: Some(mesh_clip),
                mesh,
            }),
        ));
        primitives.push(des_ui_render::PrimitiveCommand::Draw(
            des_ui_render::RenderPrimitive::Text(TextPaint {
                element_id: ElementId::new("label"),
                rect: Rect::new(4.0, 5.0, 20.0, 10.0),
                text: "Still clipped by document".into(),
                color: Color::rgb(1, 2, 3),
                override_text_color: None,
                font_size: 12.0,
                wrap_width: 20.0,
                wrap_mode: TextWrapMode::Extend,
                max_lines: None,
                line_height: None,
                selection: None,
                underline: None,
                opacity_factor: 1.0,
                angle: 0.0,
            }),
        ));
        primitives.push(des_ui_render::PrimitiveCommand::PopClip);

        let mut builder = crate::RenderPlanBuilder::new(RenderOptions::default());
        builder.push_primitives(&primitives);

        let plan = builder.finish();
        assert_eq!(plan.batches[0].clip, Some(mesh_clip));
        assert_eq!(plan.text_batches[0].clip, Some(document_clip));
    }

    #[test]
    fn mesh_builder_rejects_mixed_epaint_texture_ids() {
        fn primitive(texture_id: epaint::TextureId) -> des_ui_render::PrimitiveCommand {
            let mut mesh = epaint::Mesh {
                texture_id,
                ..epaint::Mesh::default()
            };
            mesh.vertices.push(epaint::Vertex {
                pos: epaint::pos2(0.0, 0.0),
                uv: epaint::pos2(0.0, 0.0),
                color: epaint::Color32::WHITE,
            });
            mesh.indices.push(0);
            des_ui_render::PrimitiveCommand::Draw(des_ui_render::RenderPrimitive::Mesh(
                des_ui_render::EpaintMeshPrimitive {
                    element_id: ElementId::new("textured"),
                    clip: None,
                    mesh,
                },
            ))
        }

        let mut builder = MeshBuilder::new();
        builder.push_command(&primitive(epaint::TextureId::Managed(0)));
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            builder.push_command(&primitive(epaint::TextureId::Managed(1)));
        }));

        assert!(
            result.is_err(),
            "single mesh batches must preserve epaint texture identity"
        );
    }

    #[test]
    fn draw_texture_for_mesh_routes_epaint_texture_ids() {
        let mut solid = crate::Mesh::default();
        assert_eq!(
            crate::draw_texture_for_mesh(&solid),
            crate::DrawTexture::Solid
        );

        solid.texture_id = Some(epaint::TextureId::Managed(0));
        assert_eq!(
            crate::draw_texture_for_mesh(&solid),
            crate::DrawTexture::Solid
        );

        let mut registered = crate::Mesh {
            texture_id: Some(epaint::TextureId::Managed(1)),
            ..crate::Mesh::default()
        };
        assert_eq!(
            crate::draw_texture_for_mesh(&registered),
            crate::DrawTexture::Registered(epaint::TextureId::Managed(1))
        );

        registered.texture_id = Some(epaint::TextureId::User(7));
        assert_eq!(
            crate::draw_texture_for_mesh(&registered),
            crate::DrawTexture::Registered(epaint::TextureId::User(7))
        );
    }

    #[test]
    fn managed_zero_mesh_with_atlas_uv_uses_text_atlas_texture() {
        let mesh = crate::Mesh {
            texture_id: Some(epaint::TextureId::Managed(0)),
            vertices: vec![crate::Vertex {
                position: [0.0, 0.0],
                uv: [0.25, 0.5],
                color: PackedColor::from(Color::rgb(255, 255, 255)).to_epaint_u32(),
            }],
            indices: vec![0],
        };

        assert!(matches!(
            crate::draw_texture_for_mesh(&mesh),
            crate::DrawTexture::TextAtlas
        ));
    }

    #[test]
    fn uploaded_frame_routes_registered_epaint_textures() {
        let registered_mesh = Mesh {
            texture_id: Some(epaint::TextureId::User(42)),
            ..Mesh::default()
        };
        let plan = crate::RenderPlan {
            items: vec![RenderItem::Mesh(crate::MeshBatch {
                clip: None,
                mesh: registered_mesh,
            })],
            ..crate::RenderPlan::default()
        };

        let (uploaded, _, _) = build_uploaded_frame(&plan, &crate::RasterizedTextFrame::default())
            .expect("registered epaint texture ids should survive upload planning");

        assert_eq!(
            uploaded.draws[0].texture,
            crate::DrawTexture::Registered(epaint::TextureId::User(42))
        );
    }

    #[test]
    fn missing_registered_textures_are_skipped_like_egui_wgpu() {
        assert!(crate::should_draw_registered_texture(true));
        assert!(!crate::should_draw_registered_texture(false));
    }

    #[test]
    fn uploaded_frame_routes_managed_zero_paint_atlas_meshes() {
        let atlas_mesh = Mesh {
            texture_id: Some(epaint::TextureId::Managed(0)),
            vertices: vec![crate::Vertex {
                position: [0.0, 0.0],
                uv: [0.125, 0.25],
                color: PackedColor::from(Color::rgb(255, 255, 255)).to_epaint_u32(),
            }],
            indices: vec![0],
        };
        let plan = crate::RenderPlan {
            items: vec![RenderItem::Mesh(crate::MeshBatch {
                clip: None,
                mesh: atlas_mesh,
            })],
            ..crate::RenderPlan::default()
        };

        let (uploaded, _, _) = build_uploaded_frame(&plan, &crate::RasterizedTextFrame::default())
            .expect("paint atlas UV meshes should route to the shared epaint atlas texture");

        assert_eq!(uploaded.draws[0].texture, crate::DrawTexture::TextAtlas);
    }

    #[test]
    fn uploaded_frame_requires_one_text_mesh_per_text_item() {
        let plan = crate::RenderPlan {
            items: vec![RenderItem::Text(crate::TextBatch {
                clip: None,
                text: TextPaint {
                    element_id: ElementId::new("label"),
                    rect: Rect::new(0.0, 0.0, 100.0, 24.0),
                    text: "Missing mesh".into(),
                    color: Color::rgb(1, 2, 3),
                    override_text_color: None,
                    font_size: 12.0,
                    wrap_width: 100.0,
                    wrap_mode: TextWrapMode::Extend,
                    max_lines: None,
                    line_height: None,
                    selection: None,
                    underline: None,
                    opacity_factor: 1.0,
                    angle: 0.0,
                },
            })],
            ..crate::RenderPlan::default()
        };

        let error = build_uploaded_frame(&plan, &crate::RasterizedTextFrame::default())
            .expect_err("text draw items must have matching epaint text meshes");

        assert!(matches!(
            error,
            RendererError::TextMeshCountMismatch {
                expected: 1,
                actual: 0
            }
        ));
    }

    #[test]
    fn display_list_renderer_tessellates_shapes_for_pixels_per_point() {
        let mut display_list = DisplayList::new();
        display_list.push(PaintCommand::FillRect(FillRectPaint {
            element_id: ElementId::new("box"),
            rect: Rect::new(10.0, 20.0, 30.0, 40.0),
            radius: CornerRadii::ZERO,
            color: Color::rgb(1, 2, 3),
        }));

        let low_density = DisplayListRenderer::default().build_plan(&display_list);
        let high_density = DisplayListRenderer::default()
            .with_pixels_per_point(2.0)
            .build_plan(&display_list);
        let min_x = |plan: &crate::RenderPlan| {
            plan.batches[0]
                .mesh
                .vertices
                .iter()
                .map(|vertex| vertex.position[0])
                .fold(f32::INFINITY, f32::min)
        };

        assert!(
            min_x(&high_density) > min_x(&low_density),
            "higher pixel density should shrink the antialiasing fringe in logical coordinates"
        );
    }

    #[test]
    fn display_list_renderer_uses_epaint_prepared_disc_atlas_for_circles() {
        let mut display_list = DisplayList::new();
        display_list.push(PaintCommand::FillCircle(FillCirclePaint {
            element_id: ElementId::new("disc"),
            center: Point::new(12.0, 12.0),
            radius: 4.0,
            color: Color::rgb(120, 90, 180),
        }));

        let plan = DisplayListRenderer::default().build_plan(&display_list);

        assert!(render_plan_needs_text_atlas(&plan));
        assert!(
            plan.items.iter().any(|item| matches!(
                item,
                RenderItem::Mesh(batch) if mesh_uses_paint_atlas(&batch.mesh)
            )),
            "native shape planning should seed epaint with prepared atlas discs instead of falling back to hand-tessellated circle fans"
        );
    }

    #[test]
    fn display_list_renderer_passes_tessellation_options_to_epaint() {
        let mut display_list = DisplayList::new();
        display_list.push(PaintCommand::FillRect(FillRectPaint {
            element_id: ElementId::new("box"),
            rect: Rect::new(10.0, 20.0, 30.0, 40.0),
            radius: CornerRadii::ZERO,
            color: Color::rgba(1, 2, 3, 220),
        }));

        let plan = DisplayListRenderer::new(RenderOptions {
            tessellation: RenderTessellationOptions {
                feathering: false,
                ..RenderTessellationOptions::default()
            },
            ..RenderOptions::default()
        })
        .build_plan(&display_list);

        assert!(
            plan.batches[0]
                .mesh
                .vertices
                .iter()
                .all(|vertex| vertex.color_array()[3] == 220),
            "native renderer options should preserve the caller's epaint tessellation settings"
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
        assert!(!plan.batches[0].mesh.indices.is_empty());
        assert!(!plan.batches[1].mesh.indices.is_empty());
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
            override_text_color: None,
            font_size: 12.0,
            wrap_width: 20.0,
            wrap_mode: TextWrapMode::Extend,
            max_lines: None,
            line_height: None,
            selection: None,
            underline: None,
            opacity_factor: 1.0,
            angle: 0.0,
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
    fn render_plan_intersects_nested_clips_without_expanding_parent_clip() {
        let mut display_list = DisplayList::new();
        display_list.push(PaintCommand::PushClip(Rect::new(10.0, 10.0, 40.0, 40.0)));
        display_list.push(PaintCommand::PushClip(Rect::new(0.0, 0.0, 80.0, 80.0)));
        display_list.push(PaintCommand::FillRect(FillRectPaint {
            element_id: ElementId::new("inside"),
            rect: Rect::new(0.0, 0.0, 20.0, 20.0),
            radius: CornerRadii::ZERO,
            color: Color::rgb(1, 2, 3),
        }));
        display_list.push(PaintCommand::PopClip);
        display_list.push(PaintCommand::FillRect(FillRectPaint {
            element_id: ElementId::new("after-inner"),
            rect: Rect::new(20.0, 20.0, 20.0, 20.0),
            radius: CornerRadii::ZERO,
            color: Color::rgb(4, 5, 6),
        }));
        display_list.push(PaintCommand::PopClip);

        let plan = DisplayListRenderer::default().build_plan(&display_list);

        assert_eq!(plan.batches.len(), 1);
        assert_eq!(
            plan.batches[0].clip,
            Some(Rect::new(10.0, 10.0, 40.0, 40.0))
        );
        assert_eq!(plan.items.len(), 1);
    }

    #[test]
    fn render_plan_drops_draws_inside_empty_nested_clip() {
        let mut display_list = DisplayList::new();
        display_list.push(PaintCommand::PushClip(Rect::new(0.0, 0.0, 10.0, 10.0)));
        display_list.push(PaintCommand::PushClip(Rect::new(20.0, 20.0, 10.0, 10.0)));
        display_list.push(PaintCommand::FillRect(FillRectPaint {
            element_id: ElementId::new("hidden"),
            rect: Rect::new(0.0, 0.0, 20.0, 20.0),
            radius: CornerRadii::ZERO,
            color: Color::rgb(1, 2, 3),
        }));

        let plan = DisplayListRenderer::default().build_plan(&display_list);

        assert!(plan.batches.is_empty());
        assert!(plan.items.is_empty());
        assert!(plan.is_empty());
    }

    #[test]
    fn text_rasterizer_draws_arbitrary_text_into_rgba_pixels() {
        let text = TextPaint {
            element_id: ElementId::new("label"),
            rect: Rect::new(12.0, 18.0, 220.0, 72.0),
            text: "Hello native text: π Σ data".into(),
            color: Color::rgb(18, 26, 38),
            override_text_color: None,
            font_size: 18.0,
            wrap_width: 220.0,
            wrap_mode: TextWrapMode::Wrap,
            max_lines: None,
            line_height: None,
            selection: None,
            underline: None,
            opacity_factor: 1.0,
            angle: 0.0,
        };

        let rasterized = TextRasterizer::new().rasterize(&text, 2.0);

        assert!(rasterized.width > 0);
        assert!(rasterized.height > 0);
        assert_eq!(
            rasterized.pixels.len(),
            rasterized.width as usize * rasterized.height as usize * 4
        );
        assert!(rasterized.pixels.chunks_exact(4).any(|pixel| pixel[3] > 0));
        assert!(!rasterized.mesh.vertices.is_empty());
        assert!(!rasterized.mesh.indices.is_empty());
        assert_eq!(
            rasterized.mesh.texture_id,
            Some(epaint::TextureId::Managed(0))
        );
        assert!(
            rasterized
                .mesh
                .vertices
                .iter()
                .any(|vertex| vertex.color_array()[3] > 0)
        );
        assert!(
            rasterized
                .mesh
                .vertices
                .iter()
                .any(|vertex| vertex.position[0] >= 12.0 && vertex.position[1] >= 18.0)
        );
    }

    #[test]
    fn text_rasterizer_tessellates_selection_background_with_epaint_text() {
        let text = TextPaint {
            element_id: ElementId::new("selected"),
            rect: Rect::new(12.0, 18.0, 220.0, 72.0),
            text: "alpha beta gamma".into(),
            color: Color::rgb(18, 26, 38),
            override_text_color: None,
            font_size: 18.0,
            wrap_width: 220.0,
            wrap_mode: TextWrapMode::Wrap,
            max_lines: None,
            line_height: None,
            selection: Some(TextSelectionPaint {
                anchor_index: 6,
                focus_index: 10,
                background: Color::rgb(234, 221, 255),
                color: Color::rgb(29, 27, 32),
            }),
            underline: None,
            opacity_factor: 1.0,
            angle: 0.0,
        };

        let rasterized = TextRasterizer::new().rasterize(&text, 2.0);
        let selection_color = epaint::Color32::from_rgb(234, 221, 255).to_array();
        let selected_text_color = epaint::Color32::from_rgb(29, 27, 32).to_array();

        assert!(
            rasterized
                .mesh
                .vertices
                .iter()
                .any(|vertex| vertex.color_array() == selection_color),
            "epaint should tessellate text selection background into the text mesh"
        );
        assert!(
            rasterized
                .mesh
                .vertices
                .iter()
                .any(|vertex| vertex.color_array() == selected_text_color),
            "selected glyphs should use the selected text color"
        );
    }

    #[test]
    fn text_rasterizer_preserves_epaint_text_shape_color_and_underline_controls() {
        let text = TextPaint {
            element_id: ElementId::new("effects"),
            rect: Rect::new(12.0, 18.0, 220.0, 72.0),
            text: "shape effects".into(),
            color: Color::rgb(18, 26, 38),
            override_text_color: Some(Color::rgb(220, 40, 80)),
            font_size: 18.0,
            wrap_width: 220.0,
            wrap_mode: TextWrapMode::Wrap,
            max_lines: None,
            line_height: None,
            selection: None,
            underline: Some(TextUnderlinePaint {
                width: 2.0,
                color: Color::rgb(20, 140, 90),
            }),
            opacity_factor: 1.0,
            angle: 0.0,
        };

        let rasterized = TextRasterizer::new().rasterize(&text, 2.0);
        let override_color = epaint::Color32::from_rgb(220, 40, 80).to_array();
        let underline_color = epaint::Color32::from_rgb(20, 140, 90).to_array();

        assert!(
            rasterized
                .mesh
                .vertices
                .iter()
                .any(|vertex| vertex.color_array() == override_color),
            "native text should pass epaint override_text_color through to glyph vertices"
        );
        assert!(
            rasterized
                .mesh
                .vertices
                .iter()
                .any(|vertex| vertex.color_array() == underline_color),
            "native text should pass epaint underline stroke through to tessellation"
        );
    }

    #[test]
    fn text_rasterizer_preserves_epaint_text_shape_opacity_and_rotation_controls() {
        let base = TextPaint {
            element_id: ElementId::new("rotated"),
            rect: Rect::new(12.0, 18.0, 220.0, 72.0),
            text: "rotation".into(),
            color: Color::rgba(18, 26, 38, 240),
            override_text_color: None,
            font_size: 28.0,
            wrap_width: 220.0,
            wrap_mode: TextWrapMode::Extend,
            max_lines: None,
            line_height: None,
            selection: None,
            underline: None,
            opacity_factor: 1.0,
            angle: 0.0,
        };
        let mut shaped = base.clone();
        shaped.opacity_factor = 0.5;
        shaped.angle = std::f32::consts::FRAC_PI_2;

        let mut rasterizer = TextRasterizer::new();
        let normal = rasterizer.rasterize(&base, 1.0);
        let rotated = rasterizer.rasterize(&shaped, 1.0);
        let normal_bounds = mesh_bounds(&normal.mesh).expect("normal text mesh");
        let rotated_bounds = mesh_bounds(&rotated.mesh).expect("rotated text mesh");

        assert!(
            rotated
                .mesh
                .vertices
                .iter()
                .any(|vertex| vertex.color_array()[3] < base.color.a),
            "native text should pass epaint opacity_factor through to tessellated vertices"
        );
        assert!(
            rotated_bounds.max_y - rotated_bounds.min_y > normal_bounds.max_y - normal_bounds.min_y,
            "native text should pass epaint text angle through to tessellation"
        );
    }

    #[test]
    fn text_rasterizer_uses_epaint_shape_path_for_debug_text_rects() {
        let text = TextPaint {
            element_id: ElementId::new("debug-text-rect"),
            rect: Rect::new(12.0, 18.0, 220.0, 72.0),
            text: "debug rect".into(),
            color: Color::rgb(18, 26, 38),
            override_text_color: None,
            font_size: 18.0,
            wrap_width: 220.0,
            wrap_mode: TextWrapMode::Wrap,
            max_lines: None,
            line_height: None,
            selection: None,
            underline: None,
            opacity_factor: 1.0,
            angle: 0.0,
        };
        let mut rasterizer = TextRasterizer::new();
        let plain = rasterizer.rasterize_with_options(
            &text,
            1.0,
            RenderTextOptions::default(),
            RenderTessellationOptions::default(),
        );
        let debug = rasterizer.rasterize_with_options(
            &text,
            1.0,
            RenderTextOptions::default(),
            RenderTessellationOptions {
                debug_paint_text_rects: true,
                ..RenderTessellationOptions::default()
            },
        );
        assert!(
            debug.mesh.vertices.len() > plain.mesh.vertices.len(),
            "debug_paint_text_rects should add epaint's text rect stroke vertices"
        );
        assert!(
            debug.mesh.vertices.iter().any(|vertex| {
                let [r, g, b, _] = vertex.color_array();
                g > 100 && r < 100 && b < 100
            }),
            "native text should enter epaint through Shape::Text so debug text rects render"
        );
    }

    #[test]
    fn text_rasterizer_honors_epaint_truncation_layout_bounds() {
        let text = TextPaint {
            element_id: ElementId::new("truncated"),
            rect: Rect::new(24.0, 30.0, 72.0, 24.0),
            text: "this label should not extend forever".into(),
            color: Color::rgb(18, 26, 38),
            override_text_color: None,
            font_size: 18.0,
            wrap_width: 72.0,
            wrap_mode: TextWrapMode::Truncate,
            max_lines: None,
            line_height: None,
            selection: None,
            underline: None,
            opacity_factor: 1.0,
            angle: 0.0,
        };

        let rasterized = TextRasterizer::new().rasterize(&text, 1.0);
        let max_x = rasterized
            .mesh
            .vertices
            .iter()
            .map(|vertex| vertex.position[0])
            .fold(f32::NEG_INFINITY, f32::max);

        assert!(
            max_x <= text.rect.right() + 1.0,
            "epaint truncation should keep tessellated glyphs inside the wrap width"
        );
    }

    #[test]
    fn text_rasterizer_truncates_newline_text_to_one_row_like_epaint() {
        let text = TextPaint {
            element_id: ElementId::new("newline-truncated"),
            rect: Rect::new(24.0, 30.0, 96.0, 24.0),
            text: "first line\nsecond line should not render".into(),
            color: Color::rgb(18, 26, 38),
            override_text_color: None,
            font_size: 18.0,
            wrap_width: 96.0,
            wrap_mode: TextWrapMode::Truncate,
            max_lines: None,
            line_height: Some(22.0),
            selection: None,
            underline: None,
            opacity_factor: 1.0,
            angle: 0.0,
        };

        let rasterized = TextRasterizer::new().rasterize(&text, 1.0);
        let bounds = mesh_bounds(&rasterized.mesh).expect("truncated text should produce glyphs");

        assert!(
            bounds.max_y <= text.rect.origin.y + 24.0,
            "epaint newline truncation should keep native text on one visible row"
        );
        assert!(
            bounds.max_x <= text.rect.right() + 1.0,
            "epaint newline truncation should also honor the configured wrap width"
        );
    }

    #[test]
    fn text_rasterizer_handles_zero_wrap_width_through_epaint_layout() {
        let text = TextPaint {
            element_id: ElementId::new("zero-width"),
            rect: Rect::new(8.0, 12.0, 0.0, 24.0),
            text: "a".into(),
            color: Color::rgb(18, 26, 38),
            override_text_color: None,
            font_size: 18.0,
            wrap_width: 0.0,
            wrap_mode: TextWrapMode::Wrap,
            max_lines: None,
            line_height: None,
            selection: None,
            underline: None,
            opacity_factor: 1.0,
            angle: 0.0,
        };

        let job = epaint_layout_job(&text);
        assert_eq!(
            job.wrap.max_width, 1.0,
            "DES clamps zero document text width before handing layout to epaint"
        );

        let rasterized = TextRasterizer::new().rasterize(&text, 1.0);
        assert!(
            !rasterized.mesh.vertices.is_empty(),
            "epaint should still produce a stable glyph mesh for zero-width document text"
        );
    }

    #[test]
    fn text_rasterizer_passes_round_text_option_to_epaint() {
        let text = TextPaint {
            element_id: ElementId::new("fractional-text"),
            rect: Rect::new(12.35, 18.25, 220.0, 72.0),
            text: "Fractional text".into(),
            color: Color::rgb(18, 26, 38),
            override_text_color: None,
            font_size: 18.0,
            wrap_width: 220.0,
            wrap_mode: TextWrapMode::Wrap,
            max_lines: None,
            line_height: None,
            selection: None,
            underline: None,
            opacity_factor: 1.0,
            angle: 0.0,
        };
        let mut rasterizer = TextRasterizer::new();
        let rounded = rasterizer.rasterize_with_options(
            &text,
            1.0,
            RenderTextOptions::default(),
            RenderTessellationOptions {
                round_text_to_pixels: true,
                ..RenderTessellationOptions::default()
            },
        );
        let unrounded = rasterizer.rasterize_with_options(
            &text,
            1.0,
            RenderTextOptions::default(),
            RenderTessellationOptions {
                round_text_to_pixels: false,
                ..RenderTessellationOptions::default()
            },
        );
        let rounded_bounds = mesh_bounds(&rounded.mesh).expect("rounded text mesh");
        let unrounded_bounds = mesh_bounds(&unrounded.mesh).expect("unrounded text mesh");

        assert!(
            (rounded_bounds.min_x - unrounded_bounds.min_x).abs() > 0.1,
            "native text rasterization should pass round_text_to_pixels through to epaint"
        );
    }

    #[test]
    fn text_rasterizer_passes_alpha_coverage_option_to_epaint() {
        let text = TextPaint {
            element_id: ElementId::new("coverage"),
            rect: Rect::new(8.0, 12.0, 240.0, 64.0),
            text: "coverage".into(),
            color: Color::rgb(18, 26, 38),
            override_text_color: None,
            font_size: 28.0,
            wrap_width: 240.0,
            wrap_mode: TextWrapMode::Wrap,
            max_lines: None,
            line_height: None,
            selection: None,
            underline: None,
            opacity_factor: 1.0,
            angle: 0.0,
        };
        let mut linear_rasterizer = TextRasterizer::with_text_options(RenderTextOptions {
            alpha_from_coverage: RenderAlphaFromCoverage::Linear,
            ..RenderTextOptions::default()
        });
        let mut dark_rasterizer = TextRasterizer::with_text_options(RenderTextOptions {
            alpha_from_coverage: RenderAlphaFromCoverage::TwoCoverageMinusCoverageSq,
            ..RenderTextOptions::default()
        });

        let linear = linear_rasterizer.rasterize_with_options(
            &text,
            2.0,
            RenderTextOptions {
                alpha_from_coverage: RenderAlphaFromCoverage::Linear,
                ..RenderTextOptions::default()
            },
            RenderTessellationOptions::default(),
        );
        let dark = dark_rasterizer.rasterize_with_options(
            &text,
            2.0,
            RenderTextOptions {
                alpha_from_coverage: RenderAlphaFromCoverage::TwoCoverageMinusCoverageSq,
                ..RenderTextOptions::default()
            },
            RenderTessellationOptions::default(),
        );

        assert_ne!(
            linear.pixels, dark.pixels,
            "native text rasterization should pass coverage-to-alpha policy through to epaint"
        );
    }

    #[test]
    fn text_rasterizer_lays_out_frame_text_against_one_atlas() {
        let first = TextPaint {
            element_id: ElementId::new("first"),
            rect: Rect::new(12.0, 18.0, 220.0, 72.0),
            text: "First label".into(),
            color: Color::rgb(18, 26, 38),
            override_text_color: None,
            font_size: 18.0,
            wrap_width: 220.0,
            wrap_mode: TextWrapMode::Wrap,
            max_lines: None,
            line_height: None,
            selection: None,
            underline: None,
            opacity_factor: 1.0,
            angle: 0.0,
        };
        let second = TextPaint {
            element_id: ElementId::new("second"),
            rect: Rect::new(12.0, 54.0, 220.0, 72.0),
            text: "Second label".into(),
            color: Color::rgb(96, 72, 154),
            override_text_color: None,
            font_size: 18.0,
            wrap_width: 220.0,
            wrap_mode: TextWrapMode::Wrap,
            max_lines: None,
            line_height: None,
            selection: None,
            underline: None,
            opacity_factor: 1.0,
            angle: 0.0,
        };

        let frame = TextRasterizer::new().rasterize_frame(&[first, second], 2.0);

        assert_eq!(frame.batches.len(), 2);
        assert!(frame.width > 0);
        assert!(frame.height > 0);
        let delta = frame
            .atlas_delta
            .as_ref()
            .expect("new glyphs should produce an epaint font atlas delta");
        assert_eq!(
            delta.pixels.len(),
            delta.width as usize * delta.height as usize * 4
        );
        assert!(!frame.batches[0].is_empty());
        assert!(!frame.batches[1].is_empty());
        assert!(
            frame.batches[1]
                .vertices
                .iter()
                .any(|vertex| vertex.position[1] >= 54.0)
        );
    }

    #[test]
    fn text_rasterizer_can_upload_initial_atlas_without_text() {
        let frame = TextRasterizer::new().rasterize_frame(&[], 1.0);

        assert_eq!(frame.batches.len(), 0);
        assert!(
            frame.atlas_delta.is_some(),
            "epaint's initial atlas contains the white pixel and prepared discs used by shape meshes"
        );
    }

    #[test]
    fn text_atlas_upload_reuses_existing_gpu_texture_until_size_changes() {
        let empty = crate::RasterizedTextFrame::default();
        assert_eq!(
            text_atlas_upload(None, &empty),
            crate::TextAtlasUpload::Skip
        );

        let frame_with_delta = crate::RasterizedTextFrame {
            width: 64,
            height: 128,
            atlas_delta: Some(crate::TextAtlasDelta {
                pos: None,
                width: 64,
                height: 128,
                options: TextureOptions::default(),
                pixels: vec![255; 64 * 128 * 4],
            }),
            batches: vec![crate::Mesh {
                texture_id: Some(epaint::TextureId::Managed(0)),
                vertices: vec![crate::Vertex {
                    position: [0.0, 0.0],
                    uv: [0.0, 0.0],
                    color: PackedColor::from(Color::rgb(255, 255, 255)).to_epaint_u32(),
                }],
                indices: vec![0],
            }],
        };
        let frame_without_delta = crate::RasterizedTextFrame {
            atlas_delta: None,
            ..frame_with_delta.clone()
        };
        let descriptor = crate::TextureDescriptor {
            width: 64,
            height: 128,
            options: TextureOptions::default(),
        };

        assert_eq!(
            text_atlas_upload(None, &frame_with_delta),
            crate::TextAtlasUpload::Recreate(descriptor)
        );
        assert_eq!(
            text_atlas_upload(Some(descriptor), &frame_without_delta),
            crate::TextAtlasUpload::Unchanged(descriptor)
        );
        assert_eq!(
            text_atlas_upload(Some(descriptor), &frame_with_delta),
            crate::TextAtlasUpload::Reuse(descriptor)
        );
        assert_eq!(
            text_atlas_upload(
                Some(crate::TextureDescriptor {
                    width: 32,
                    height: 128,
                    options: TextureOptions::default(),
                }),
                &frame_with_delta
            ),
            crate::TextAtlasUpload::Recreate(descriptor)
        );
    }

    #[test]
    fn text_atlas_upload_keeps_initial_atlas_delta_without_text_meshes() {
        let frame = crate::RasterizedTextFrame {
            width: 64,
            height: 32,
            atlas_delta: Some(crate::TextAtlasDelta {
                pos: None,
                width: 64,
                height: 32,
                options: TextureOptions::default(),
                pixels: vec![255; 64 * 32 * 4],
            }),
            batches: Vec::new(),
        };
        let descriptor = crate::TextureDescriptor {
            width: 64,
            height: 32,
            options: TextureOptions::default(),
        };

        assert_eq!(
            text_atlas_upload(None, &frame),
            crate::TextAtlasUpload::Recreate(descriptor)
        );
    }

    #[test]
    fn render_plan_needs_text_atlas_for_text_or_atlas_uv_meshes() {
        assert!(!crate::render_plan_needs_text_atlas(&crate::RenderPlan {
            items: vec![RenderItem::Mesh(crate::MeshBatch {
                clip: None,
                mesh: crate::Mesh::default(),
            })],
            ..crate::RenderPlan::default()
        }));

        assert!(crate::render_plan_needs_text_atlas(&crate::RenderPlan {
            items: vec![RenderItem::Mesh(crate::MeshBatch {
                clip: None,
                mesh: crate::Mesh {
                    texture_id: Some(epaint::TextureId::Managed(0)),
                    vertices: vec![crate::Vertex {
                        position: [0.0, 0.0],
                        uv: [0.5, 0.5],
                        color: PackedColor::from(Color::rgb(255, 255, 255)).to_epaint_u32(),
                    }],
                    indices: vec![0],
                },
            })],
            ..crate::RenderPlan::default()
        }));
    }

    #[test]
    fn text_atlas_upload_recreates_when_sampling_contract_changes() {
        let nearest = TextureOptions {
            magnification: TextureFilter::Nearest,
            minification: TextureFilter::Nearest,
            wrap_mode: TextureWrapMode::ClampToEdge,
            mipmap_mode: None,
        };
        let current = crate::TextureDescriptor {
            width: 64,
            height: 64,
            options: TextureOptions::default(),
        };
        let frame = crate::RasterizedTextFrame {
            width: 64,
            height: 64,
            atlas_delta: Some(crate::TextAtlasDelta {
                pos: None,
                width: 64,
                height: 64,
                options: nearest,
                pixels: vec![255; 64 * 64 * 4],
            }),
            batches: vec![crate::Mesh {
                texture_id: Some(epaint::TextureId::Managed(0)),
                vertices: vec![crate::Vertex {
                    position: [0.0, 0.0],
                    uv: [0.0, 0.0],
                    color: PackedColor::from(Color::rgb(255, 255, 255)).to_epaint_u32(),
                }],
                indices: vec![0],
            }],
        };
        let next = crate::TextureDescriptor {
            options: nearest,
            ..current
        };

        assert_eq!(
            text_atlas_upload(Some(current), &frame),
            crate::TextAtlasUpload::Recreate(next)
        );
    }

    #[test]
    fn texture_delta_upload_matches_epaint_allocation_patch_and_rebind_contracts() {
        let nearest = TextureOptions {
            magnification: TextureFilter::Nearest,
            minification: TextureFilter::Nearest,
            wrap_mode: TextureWrapMode::ClampToEdge,
            mipmap_mode: None,
        };
        let current = crate::TextureDescriptor {
            width: 64,
            height: 64,
            options: TextureOptions::default(),
        };
        let full = crate::TextureDelta {
            pos: None,
            width: 16,
            height: 32,
            options: nearest,
            pixels: vec![255; 16 * 32 * 4],
        };
        let patch = crate::TextureDelta {
            pos: Some([4, 8]),
            width: 8,
            height: 8,
            options: TextureOptions::default(),
            pixels: vec![128; 8 * 8 * 4],
        };
        let patch_with_new_options = crate::TextureDelta {
            options: nearest,
            ..patch.clone()
        };

        assert_eq!(
            crate::texture_delta_upload(None, &full, epaint::TextureId::User(1)).unwrap(),
            crate::TextureDeltaUpload::Allocate(crate::TextureDescriptor {
                width: 16,
                height: 32,
                options: nearest,
            })
        );
        assert_eq!(
            crate::texture_delta_upload(Some(current), &patch, epaint::TextureId::User(1)).unwrap(),
            crate::TextureDeltaUpload::Patch
        );
        assert_eq!(
            crate::texture_delta_upload(
                Some(current),
                &patch_with_new_options,
                epaint::TextureId::User(1)
            )
            .unwrap(),
            crate::TextureDeltaUpload::PatchAndRebind(crate::TextureDescriptor {
                options: nearest,
                ..current
            })
        );
        assert!(matches!(
            crate::texture_delta_upload(None, &patch, epaint::TextureId::User(7)),
            Err(RendererError::PartialTextureUpdateMissingTexture(
                epaint::TextureId::User(7)
            ))
        ));
    }

    #[test]
    fn text_atlas_delta_preserves_epaint_partial_patch_contract() {
        let image = epaint::ColorImage::new(
            [2, 1],
            vec![
                epaint::Color32::from_rgba_premultiplied(10, 20, 30, 40),
                epaint::Color32::from_rgba_premultiplied(50, 60, 70, 80),
            ],
        );
        let delta = epaint::ImageDelta::partial(
            [12, 34],
            image,
            epaint::textures::TextureOptions::NEAREST_REPEAT,
        );

        let atlas_delta = crate::text_atlas_delta_from_epaint(delta);

        assert_eq!(atlas_delta.pos, Some([12, 34]));
        assert_eq!(atlas_delta.width, 2);
        assert_eq!(atlas_delta.height, 1);
        assert_eq!(
            atlas_delta.options,
            TextureOptions {
                magnification: TextureFilter::Nearest,
                minification: TextureFilter::Nearest,
                wrap_mode: TextureWrapMode::Repeat,
                mipmap_mode: None,
            }
        );
        assert_eq!(atlas_delta.pixels, vec![10, 20, 30, 40, 50, 60, 70, 80]);
    }

    #[test]
    fn texture_delta_preserves_epaint_full_and_partial_contracts() {
        let full = crate::texture_delta_from_epaint(epaint::ImageDelta::full(
            epaint::ColorImage::new(
                [1, 2],
                vec![
                    epaint::Color32::from_rgba_premultiplied(1, 2, 3, 4),
                    epaint::Color32::from_rgba_premultiplied(5, 6, 7, 8),
                ],
            ),
            epaint::textures::TextureOptions::LINEAR,
        ));
        assert_eq!(full.pos, None);
        assert_eq!(full.width, 1);
        assert_eq!(full.height, 2);
        assert_eq!(full.options, TextureOptions::default());
        assert_eq!(full.pixels, vec![1, 2, 3, 4, 5, 6, 7, 8]);

        let partial = crate::texture_delta_from_epaint(epaint::ImageDelta::partial(
            [3, 5],
            epaint::ColorImage::new(
                [2, 1],
                vec![
                    epaint::Color32::from_rgba_premultiplied(9, 10, 11, 12),
                    epaint::Color32::from_rgba_premultiplied(13, 14, 15, 16),
                ],
            ),
            epaint::textures::TextureOptions::NEAREST_REPEAT,
        ));
        assert_eq!(partial.pos, Some([3, 5]));
        assert_eq!(partial.width, 2);
        assert_eq!(partial.height, 1);
        assert_eq!(
            partial.options,
            TextureOptions {
                magnification: TextureFilter::Nearest,
                minification: TextureFilter::Nearest,
                wrap_mode: TextureWrapMode::Repeat,
                mipmap_mode: None,
            }
        );
        assert_eq!(partial.pixels, vec![9, 10, 11, 12, 13, 14, 15, 16]);
    }

    #[test]
    fn texture_sampler_descriptor_maps_epaint_texture_options_to_wgpu() {
        let descriptor = crate::texture_sampler_descriptor(
            "texture options",
            epaint::textures::TextureOptions::LINEAR_MIRRORED_REPEAT
                .with_mipmap_mode(Some(epaint::textures::TextureFilter::Linear))
                .into(),
        );

        assert_eq!(descriptor.label, Some("texture options"));
        assert_eq!(descriptor.address_mode_u, wgpu::AddressMode::MirrorRepeat);
        assert_eq!(descriptor.address_mode_v, wgpu::AddressMode::MirrorRepeat);
        assert_eq!(descriptor.address_mode_w, wgpu::AddressMode::MirrorRepeat);
        assert_eq!(descriptor.mag_filter, wgpu::FilterMode::Linear);
        assert_eq!(descriptor.min_filter, wgpu::FilterMode::Linear);
        assert_eq!(descriptor.mipmap_filter, wgpu::MipmapFilterMode::Linear);
    }

    #[test]
    fn texture_options_are_stable_sampler_cache_keys() {
        let linear = TextureOptions::default();
        let repeated_linear = epaint::textures::TextureOptions::LINEAR.into();
        let nearest_repeat = epaint::textures::TextureOptions::NEAREST_REPEAT.into();

        let mut keys = std::collections::HashSet::new();
        keys.insert(linear);
        keys.insert(repeated_linear);
        keys.insert(nearest_repeat);

        assert_eq!(
            keys.len(),
            2,
            "sampler caching should share equivalent epaint texture options"
        );
    }

    #[test]
    fn frame_buffer_upload_reuses_capacity_and_doubles_when_growing() {
        assert_eq!(crate::buffer_upload(None, 0), crate::BufferUpload::Skip);
        assert_eq!(
            crate::buffer_upload(None, 128),
            crate::BufferUpload::Recreate(128)
        );
        assert_eq!(
            crate::buffer_upload(Some(256), 128),
            crate::BufferUpload::Reuse(256)
        );
        assert_eq!(
            crate::buffer_upload(Some(256), 300),
            crate::BufferUpload::Recreate(512)
        );
        assert_eq!(
            crate::buffer_upload(Some(256), 900),
            crate::BufferUpload::Recreate(900)
        );
    }

    #[test]
    fn uploaded_draw_ranges_track_frame_buffer_slices() {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let first_clip = Some(Rect::new(1.0, 2.0, 3.0, 4.0));
        let first = crate::append_mesh_draw(
            &mut vertices,
            &mut indices,
            first_clip,
            crate::DrawTexture::Solid,
            &[
                Vertex {
                    position: [0.0, 0.0],
                    uv: [0.0, 0.0],
                    color: PackedColor::from(Color::rgb(255, 0, 0)).to_epaint_u32(),
                },
                Vertex {
                    position: [1.0, 0.0],
                    uv: [1.0, 0.0],
                    color: PackedColor::from(Color::rgb(255, 0, 0)).to_epaint_u32(),
                },
            ],
            &[0, 1],
        );
        let second = crate::append_mesh_draw(
            &mut vertices,
            &mut indices,
            None,
            crate::DrawTexture::TextAtlas,
            &[Vertex {
                position: [2.0, 0.0],
                uv: [0.0, 1.0],
                color: PackedColor::from(Color::rgb(0, 0, 255)).to_epaint_u32(),
            }],
            &[0],
        );

        assert_eq!(first.clip, first_clip);
        assert_eq!(first.texture, crate::DrawTexture::Solid);
        assert_eq!(first.vertex_range, 0..(2 * mem::size_of::<Vertex>()) as u64);
        assert_eq!(first.index_range, 0..(2 * mem::size_of::<u32>()) as u64);
        assert_eq!(first.index_count, 2);
        assert_eq!(
            second.vertex_range,
            (2 * mem::size_of::<Vertex>()) as u64..(3 * mem::size_of::<Vertex>()) as u64
        );
        assert_eq!(second.texture, crate::DrawTexture::TextAtlas);
        assert_eq!(
            second.index_range,
            (2 * mem::size_of::<u32>()) as u64..(3 * mem::size_of::<u32>()) as u64
        );
        assert_eq!(vertices.len(), 3);
        assert_eq!(indices.len(), 3);
    }

    #[test]
    fn text_layout_job_maps_document_wrap_modes_to_epaint() {
        let mut wrap = TextPaint {
            element_id: ElementId::new("wrap"),
            rect: Rect::new(0.0, 0.0, 100.0, 40.0),
            text: "alpha beta gamma".into(),
            color: Color::rgb(1, 2, 3),
            override_text_color: None,
            font_size: 14.0,
            wrap_width: 80.0,
            wrap_mode: TextWrapMode::Wrap,
            max_lines: Some(3),
            line_height: Some(18.0),
            selection: None,
            underline: None,
            opacity_factor: 1.0,
            angle: 0.0,
        };

        let wrap_job = epaint_layout_job(&wrap);
        assert_eq!(wrap_job.wrap.max_width, 80.0);
        assert_eq!(wrap_job.wrap.max_rows, 3);
        assert!(!wrap_job.wrap.break_anywhere);
        assert_eq!(wrap_job.sections[0].format.line_height, Some(18.0));

        wrap.wrap_mode = TextWrapMode::Truncate;
        wrap.max_lines = None;
        let truncate_job = epaint_layout_job(&wrap);
        assert_eq!(truncate_job.wrap.max_rows, 1);
        assert!(truncate_job.wrap.break_anywhere);

        wrap.wrap_mode = TextWrapMode::Extend;
        let extend_job = epaint_layout_job(&wrap);
        assert_eq!(extend_job.wrap.max_width, f32::INFINITY);
    }

    #[test]
    fn text_layout_job_maps_document_selection_to_epaint_sections() {
        let text = TextPaint {
            element_id: ElementId::new("selected"),
            rect: Rect::new(0.0, 0.0, 160.0, 40.0),
            text: "alpha π beta".into(),
            color: Color::rgb(20, 21, 22),
            override_text_color: None,
            font_size: 14.0,
            wrap_width: 160.0,
            wrap_mode: TextWrapMode::Wrap,
            max_lines: None,
            line_height: None,
            selection: Some(TextSelectionPaint {
                anchor_index: 6,
                focus_index: 7,
                background: Color::rgb(234, 221, 255),
                color: Color::rgb(29, 27, 32),
            }),
            underline: None,
            opacity_factor: 1.0,
            angle: 0.0,
        };

        let job = epaint_layout_job(&text);

        assert_eq!(job.sections.len(), 3);
        assert_eq!(&job.text[job.sections[1].byte_range.clone()], "π");
        assert_eq!(
            job.sections[1].format.background,
            epaint::Color32::from_rgb(234, 221, 255)
        );
        assert_eq!(
            job.sections[1].format.color,
            epaint::Color32::from_rgb(29, 27, 32)
        );
        assert_eq!(
            job.sections[0].format.background,
            epaint::Color32::TRANSPARENT
        );
        assert_eq!(
            job.sections[2].format.background,
            epaint::Color32::TRANSPARENT
        );
    }

    #[derive(Clone, Copy, Debug, PartialEq)]
    struct MeshBounds {
        min_x: f32,
        max_x: f32,
        min_y: f32,
        max_y: f32,
    }

    fn mesh_bounds(mesh: &Mesh) -> Option<MeshBounds> {
        let mut vertices = mesh.vertices.iter();
        let first = vertices.next()?;
        let mut min_x = first.position[0];
        let mut max_x = first.position[0];
        let mut min_y = first.position[1];
        let mut max_y = first.position[1];
        for vertex in vertices {
            min_x = min_x.min(vertex.position[0]);
            max_x = max_x.max(vertex.position[0]);
            min_y = min_y.min(vertex.position[1]);
            max_y = max_y.max(vertex.position[1]);
        }
        Some(MeshBounds {
            min_x,
            max_x,
            min_y,
            max_y,
        })
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
            override_text_color: None,
            font_size: 12.0,
            wrap_width: 20.0,
            wrap_mode: TextWrapMode::Extend,
            max_lines: None,
            line_height: None,
            selection: None,
            underline: None,
            opacity_factor: 1.0,
            angle: 0.0,
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
    fn clip_rect_rounds_to_physical_pixels_like_epaint_viewport() {
        let scissor = clip_rect_to_scissor(
            Some(Rect::new(10.25, 20.25, 30.25, 40.25)),
            PhysicalRenderSize {
                width: 200,
                height: 200,
                scale_factor: 2.0,
            },
        );

        assert_eq!(
            scissor,
            Some(ScissorRect {
                x: 21,
                y: 41,
                width: 60,
                height: 80,
            })
        );
    }

    #[test]
    fn clip_rect_rounds_and_clamps_to_surface_bounds() {
        let scissor = clip_rect_to_scissor(
            Some(Rect::new(-1.4, -1.4, 5.6, 5.6)),
            PhysicalRenderSize {
                width: 8,
                height: 8,
                scale_factor: 1.0,
            },
        );

        assert_eq!(
            scissor,
            Some(ScissorRect {
                x: 0,
                y: 0,
                width: 4,
                height: 4,
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
                .any(|vertex| vertex.color_array()[3] == 10)
        );
        assert!(
            mesh.vertices
                .iter()
                .any(|vertex| vertex.color_array()[3] == 0)
        );
    }
}
