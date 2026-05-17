//! `wgpu` adapter for DES UI paint commands.
//!
//! This crate owns the GPU-facing representation of renderer-neutral
//! `des-ui-render` display lists. It intentionally starts below document
//! semantics: document/style/layout produce paint commands, and this crate
//! turns the supported commands into meshes and, later, `wgpu` draw calls.

use des_ui_document::{Color, Rect};
use des_ui_render::{DisplayList, FillRectPaint, PaintCommand};

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

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RenderPlan {
    pub clear_color: ClearColor,
    pub batches: Vec<MeshBatch>,
}

impl RenderPlan {
    pub fn is_empty(&self) -> bool {
        self.batches.iter().all(|batch| batch.mesh.is_empty())
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MeshBatch {
    pub clip: Option<Rect>,
    pub mesh: Mesh,
}

#[derive(Clone, Debug, Default)]
pub struct DisplayListRenderer {
    options: RenderOptions,
}

impl DisplayListRenderer {
    pub fn new(options: RenderOptions) -> Self {
        Self { options }
    }

    pub fn build_plan(&self, display_list: &DisplayList) -> RenderPlan {
        let mut builder = RenderPlanBuilder::new(self.options);
        builder.push_display_list(display_list);
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
        for command in &display_list.commands {
            self.push_command(command);
        }
    }

    pub fn push_command(&mut self, command: &PaintCommand) {
        match command {
            PaintCommand::FillRect(command) => self.push_fill_rect(command),
            PaintCommand::PushClip(_)
            | PaintCommand::PopClip
            | PaintCommand::StrokeRect(_)
            | PaintCommand::StrokeLine(_)
            | PaintCommand::StrokePath(_)
            | PaintCommand::FillCircle(_)
            | PaintCommand::FillPolygon(_)
            | PaintCommand::Text(_) => {}
        }
    }

    pub fn finish(self) -> Mesh {
        self.mesh
    }

    fn push_fill_rect(&mut self, command: &FillRectPaint) {
        if command.rect.size.width <= 0.0 || command.rect.size.height <= 0.0 {
            return;
        }
        self.push_solid_rect(command.rect, command.color);
    }

    fn push_solid_rect(&mut self, rect: Rect, color: Color) {
        let base = self.mesh.vertices.len() as u32;
        let color = PackedColor::from(color).to_array();
        let left = rect.origin.x;
        let top = rect.origin.y;
        let right = rect.right();
        let bottom = rect.bottom();
        self.mesh.vertices.extend([
            Vertex {
                position: [left, top],
                color,
            },
            Vertex {
                position: [right, top],
                color,
            },
            Vertex {
                position: [right, bottom],
                color,
            },
            Vertex {
                position: [left, bottom],
                color,
            },
        ]);
        self.mesh
            .indices
            .extend([base, base + 1, base + 2, base, base + 2, base + 3]);
    }
}

pub fn mesh_for_display_list(display_list: &DisplayList) -> Mesh {
    let mut builder = MeshBuilder::new();
    builder.push_display_list(display_list);
    builder.finish()
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
                batches: Vec::new(),
            },
            current_clip: None,
            current_mesh: MeshBuilder::new(),
        }
    }

    fn push_display_list(&mut self, display_list: &DisplayList) {
        let mut clip_stack: Vec<Rect> = Vec::new();
        for command in &display_list.commands {
            match command {
                PaintCommand::PushClip(rect) => {
                    clip_stack.push(*rect);
                    self.set_clip(clip_stack.last().copied());
                }
                PaintCommand::PopClip => {
                    clip_stack.pop();
                    self.set_clip(clip_stack.last().copied());
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
            self.plan.batches.push(MeshBatch {
                clip: self.current_clip,
                mesh,
            });
        }
    }

    fn finish(mut self) -> RenderPlan {
        self.flush();
        self.plan
    }
}

#[cfg(test)]
mod tests {
    use des_ui_document::{Color, CornerRadii, ElementId, Rect};
    use des_ui_render::{DisplayList, FillRectPaint, PaintCommand};

    use crate::{ClearColor, DisplayListRenderer, MeshBuilder, PackedColor, RenderOptions};

    #[test]
    fn packed_color_preserves_rgba_channel_order() {
        assert_eq!(
            PackedColor::from(Color::rgba(10, 20, 30, 40)).to_array(),
            [10, 20, 30, 40]
        );
    }

    #[test]
    fn fill_rect_generates_two_triangles_in_document_coordinates() {
        let mut builder = MeshBuilder::new();
        builder.push_command(&PaintCommand::FillRect(FillRectPaint {
            element_id: ElementId::new("box"),
            rect: Rect::new(10.0, 20.0, 30.0, 40.0),
            radius: CornerRadii::ZERO,
            color: Color::rgba(1, 2, 3, 4),
        }));

        let mesh = builder.finish();
        assert_eq!(mesh.vertices.len(), 4);
        assert_eq!(mesh.indices, [0, 1, 2, 0, 2, 3]);
        assert_eq!(mesh.vertices[0].position, [10.0, 20.0]);
        assert_eq!(mesh.vertices[1].position, [40.0, 20.0]);
        assert_eq!(mesh.vertices[2].position, [40.0, 60.0]);
        assert_eq!(mesh.vertices[3].position, [10.0, 60.0]);
        assert!(
            mesh.vertices
                .iter()
                .all(|vertex| vertex.color == [1, 2, 3, 4])
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
        assert_eq!(plan.batches[0].mesh.indices.len(), 6);
        assert_eq!(plan.batches[1].mesh.indices.len(), 6);
    }
}
