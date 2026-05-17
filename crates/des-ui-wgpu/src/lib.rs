//! `wgpu` adapter for DES UI paint commands.
//!
//! This crate owns the GPU-facing representation of renderer-neutral
//! `des-ui-render` display lists. It intentionally starts below document
//! semantics: document/style/layout produce paint commands, and this crate
//! turns the supported commands into meshes and, later, `wgpu` draw calls.

use des_ui_document::{Color, Rect};
use des_ui_render::{
    DisplayList, PrimitiveCommand, PrimitiveList, RenderPrimitive, TextPaint,
    TriangleMeshPrimitive, plan_primitives,
};

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
        ClearColor, DisplayListRenderer, MeshBuilder, PackedColor, RenderItem, RenderOptions,
        mesh_for_display_list,
    };

    #[test]
    fn packed_color_preserves_rgba_channel_order() {
        assert_eq!(
            PackedColor::from(Color::rgba(10, 20, 30, 40)).to_array(),
            [10, 20, 30, 40]
        );
    }

    #[test]
    fn fill_rect_generates_two_triangles_in_document_coordinates() {
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
                .all(|vertex| vertex.color == [7, 8, 9, 10])
        );
    }
}
