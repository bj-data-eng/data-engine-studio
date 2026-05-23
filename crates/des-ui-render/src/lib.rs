//! Renderer-neutral paint planning for document UI.
//!
//! `des-ui-render` turns resolved document output into a deterministic display
//! list. Backends such as `des-ui-egui` should translate these commands into
//! host-specific drawing calls.

use des_ui_document::{
    BorderStyle, Color, CornerRadii, DocumentOutput, ElementId, FloatingPlacement, Glyph, Insets,
    Overflow, Point, Rect, ResolvedElement, ScrollAxis, ScrollChrome, Shadow, TextWrapMode,
};

const DEFAULT_ANTIALIASING_FRINGE: f32 = 1.0;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DisplayList {
    pub commands: Vec<PaintCommand>,
}

impl DisplayList {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, command: PaintCommand) {
        self.commands.push(command);
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum PaintCommand {
    PushClip(Rect),
    PopClip,
    FillRect(FillRectPaint),
    StrokeRect(StrokeRectPaint),
    StrokeLine(StrokeLinePaint),
    StrokePath(StrokePathPaint),
    FillCircle(FillCirclePaint),
    FillPolygon(FillPolygonPaint),
    Text(TextPaint),
}

#[derive(Clone, Debug, PartialEq)]
pub struct FillRectPaint {
    pub element_id: ElementId,
    pub rect: Rect,
    pub radius: CornerRadii,
    pub color: Color,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StrokeRectPaint {
    pub element_id: ElementId,
    pub rect: Rect,
    pub radius: CornerRadii,
    pub width: f32,
    pub color: Color,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StrokeLinePaint {
    pub element_id: ElementId,
    pub from: Point,
    pub to: Point,
    pub width: f32,
    pub color: Color,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StrokePathPaint {
    pub element_id: ElementId,
    pub points: Vec<Point>,
    pub width: f32,
    pub color: Color,
    pub closed: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FillCirclePaint {
    pub element_id: ElementId,
    pub center: Point,
    pub radius: f32,
    pub color: Color,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FillPolygonPaint {
    pub element_id: ElementId,
    pub points: Vec<Point>,
    pub color: Color,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BorderPaint {
    pub color: Color,
    pub widths: Insets,
    pub style: BorderStyle,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FloatingArrowPaint {
    pub points: [Point; 3],
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextPaint {
    pub element_id: ElementId,
    pub rect: Rect,
    pub text: String,
    pub color: Color,
    pub font_size: f32,
    pub wrap_width: f32,
    pub wrap_mode: TextWrapMode,
    pub max_lines: Option<usize>,
    pub line_height: Option<f32>,
    pub selection: Option<TextSelectionPaint>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextSelectionPaint {
    pub anchor_index: usize,
    pub focus_index: usize,
    pub background: Color,
    pub color: Color,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PrimitiveList {
    pub commands: Vec<PrimitiveCommand>,
}

impl PrimitiveList {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, command: PrimitiveCommand) {
        self.commands.push(command);
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum PrimitiveCommand {
    PushClip(Rect),
    PopClip,
    Draw(RenderPrimitive),
}

#[derive(Clone, Debug, PartialEq)]
pub enum RenderPrimitive {
    Triangles(TriangleMeshPrimitive),
    Text(TextPaint),
}

#[derive(Clone, Debug, PartialEq)]
pub struct TriangleMeshPrimitive {
    pub element_id: ElementId,
    pub vertices: Vec<PrimitiveVertex>,
    pub indices: Vec<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PrimitiveVertex {
    pub position: Point,
    pub color: Color,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PrimitivePlanner;

impl PrimitivePlanner {
    pub fn new() -> Self {
        Self
    }

    pub fn plan_display_list(&self, display_list: &DisplayList) -> PrimitiveList {
        let mut primitives = PrimitiveList::new();
        for command in &display_list.commands {
            append_primitive_command(&mut primitives, command);
        }
        primitives
    }
}

pub fn plan_primitives(display_list: &DisplayList) -> PrimitiveList {
    PrimitivePlanner::new().plan_display_list(display_list)
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScrollChromePaint {
    pub element_id: ElementId,
    pub axis: ScrollAxis,
    pub track_rect: Rect,
    pub hit_rect: Rect,
    pub handle_rect: Rect,
    pub handle_color: Color,
    pub track_color: Option<Color>,
    pub handle_border_color: Option<Color>,
    pub handle_border_width: f32,
    pub radius: f32,
    pub visible: bool,
    pub expanded: bool,
    pub hovered: bool,
    pub dragged: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PaintPlanner;

impl PaintPlanner {
    pub fn new() -> Self {
        Self
    }

    pub fn plan_output(&self, output: &DocumentOutput) -> DisplayList {
        let mut list = DisplayList::new();
        self.plan_element(&mut list, &output.layout, None, output);
        for chrome in &output.scroll_chrome {
            append_scroll_chrome_commands(&mut list, &ScrollChromePaint::from(chrome));
        }
        list
    }

    pub fn plan_element(
        &self,
        list: &mut DisplayList,
        frame: &ResolvedElement,
        clip_rect: Option<Rect>,
        output: &DocumentOutput,
    ) {
        if frame.id.as_str() != "root" {
            append_surface_commands(list, frame);

            if let Some(text) = &frame.text {
                list.push(PaintCommand::Text(text_paint(frame, text, output)));
            }

            if let Some(glyph) = frame.glyph {
                append_glyph_commands(
                    list,
                    frame.id.clone(),
                    frame.rect,
                    glyph,
                    frame.style.text_color,
                    frame.style.font_size,
                );
            }
        }

        let next_clip = child_clip_rect(frame, clip_rect);
        let pushed_clip = next_clip != clip_rect;
        if let Some(next_clip) = next_clip
            && pushed_clip
        {
            list.push(PaintCommand::PushClip(next_clip));
        }

        let mut children: Vec<_> = frame.children.iter().collect();
        children.sort_by_key(|child| child.style.z_index);
        for child in children {
            self.plan_element(list, child, next_clip, output);
        }

        if pushed_clip {
            list.push(PaintCommand::PopClip);
        }
    }
}

pub fn plan_paint(output: &DocumentOutput) -> DisplayList {
    PaintPlanner::new().plan_output(output)
}

pub fn content_rect(frame: &ResolvedElement) -> Rect {
    frame.rect.inset(Insets {
        top: frame.style.border_width.top + frame.style.padding.top,
        right: frame.style.border_width.right + frame.style.padding.right,
        bottom: frame.style.border_width.bottom + frame.style.padding.bottom,
        left: frame.style.border_width.left + frame.style.padding.left,
    })
}

fn append_primitive_command(primitives: &mut PrimitiveList, command: &PaintCommand) {
    match command {
        PaintCommand::PushClip(rect) => primitives.push(PrimitiveCommand::PushClip(*rect)),
        PaintCommand::PopClip => primitives.push(PrimitiveCommand::PopClip),
        PaintCommand::FillRect(command) => {
            if let Some(mesh) = fill_rect_mesh(command) {
                primitives.push(PrimitiveCommand::Draw(RenderPrimitive::Triangles(mesh)));
            }
        }
        PaintCommand::StrokeRect(command) => {
            for mesh in stroke_rect_meshes(command) {
                primitives.push(PrimitiveCommand::Draw(RenderPrimitive::Triangles(mesh)));
            }
        }
        PaintCommand::StrokeLine(command) => {
            if let Some(mesh) = stroke_line_mesh(
                command.element_id.clone(),
                command.from,
                command.to,
                command.width,
                command.color,
            ) {
                primitives.push(PrimitiveCommand::Draw(RenderPrimitive::Triangles(mesh)));
            }
        }
        PaintCommand::StrokePath(command) => {
            for mesh in stroke_path_meshes(command) {
                primitives.push(PrimitiveCommand::Draw(RenderPrimitive::Triangles(mesh)));
            }
        }
        PaintCommand::FillCircle(command) => {
            if let Some(mesh) = fill_circle_mesh(command) {
                primitives.push(PrimitiveCommand::Draw(RenderPrimitive::Triangles(mesh)));
            }
        }
        PaintCommand::FillPolygon(command) => {
            if let Some(mesh) = fill_polygon_mesh(
                command.element_id.clone(),
                command.points.clone(),
                command.color,
            ) {
                primitives.push(PrimitiveCommand::Draw(RenderPrimitive::Triangles(mesh)));
            }
        }
        PaintCommand::Text(command) => {
            primitives.push(PrimitiveCommand::Draw(RenderPrimitive::Text(
                command.clone(),
            )));
        }
    }
}

fn fill_rect_mesh(command: &FillRectPaint) -> Option<TriangleMeshPrimitive> {
    if command.rect.size.width <= 0.0 || command.rect.size.height <= 0.0 || command.color.a == 0 {
        return None;
    }
    if command.radius == CornerRadii::ZERO {
        return fill_polygon_mesh(
            command.element_id.clone(),
            rect_points(command.rect),
            command.color,
        );
    }
    fill_polygon_mesh(
        command.element_id.clone(),
        rounded_rect_points(command.rect, command.radius),
        command.color,
    )
}

fn stroke_rect_meshes(command: &StrokeRectPaint) -> Vec<TriangleMeshPrimitive> {
    if command.width <= 0.0 || command.rect.size.width <= 0.0 || command.rect.size.height <= 0.0 {
        return Vec::new();
    }
    let points = if command.radius == CornerRadii::ZERO {
        rect_points(command.rect)
    } else {
        rounded_rect_points(command.rect, command.radius)
    };
    stroke_closed_polyline_meshes(
        command.element_id.clone(),
        &points,
        command.width,
        command.color,
    )
}

fn stroke_path_meshes(command: &StrokePathPaint) -> Vec<TriangleMeshPrimitive> {
    if command.width <= 0.0 || command.points.len() < 2 {
        return Vec::new();
    }
    if command.closed {
        return stroke_closed_polyline_meshes(
            command.element_id.clone(),
            &command.points,
            command.width,
            command.color,
        );
    }
    command
        .points
        .windows(2)
        .filter_map(|points| {
            stroke_line_mesh(
                command.element_id.clone(),
                points[0],
                points[1],
                command.width,
                command.color,
            )
        })
        .collect()
}

fn stroke_closed_polyline_meshes(
    element_id: ElementId,
    points: &[Point],
    width: f32,
    color: Color,
) -> Vec<TriangleMeshPrimitive> {
    if points.len() < 2 {
        return Vec::new();
    }
    let mut meshes: Vec<_> = points
        .windows(2)
        .filter_map(|points| {
            stroke_line_mesh(element_id.clone(), points[0], points[1], width, color)
        })
        .collect();
    if let Some(mesh) = stroke_line_mesh(
        element_id,
        *points.last().expect("checked length"),
        points[0],
        width,
        color,
    ) {
        meshes.push(mesh);
    }
    meshes
}

fn stroke_line_mesh(
    element_id: ElementId,
    from: Point,
    to: Point,
    width: f32,
    color: Color,
) -> Option<TriangleMeshPrimitive> {
    if width <= 0.0 || color.a == 0 {
        return None;
    }
    let vector = sub_points(to, from);
    let length = point_length(vector);
    if length <= f32::EPSILON {
        return None;
    }
    let normal = scale_point(Point::new(-vector.y, vector.x), width * 0.5 / length);
    fill_polygon_mesh(
        element_id,
        vec![
            add_points(from, normal),
            add_points(to, normal),
            sub_points(to, normal),
            sub_points(from, normal),
        ],
        color,
    )
}

fn fill_circle_mesh(command: &FillCirclePaint) -> Option<TriangleMeshPrimitive> {
    if command.radius <= 0.0 || command.color.a == 0 {
        return None;
    }
    let segments = circle_segments(command.radius);
    let mut points = Vec::with_capacity(segments);
    for index in 0..segments {
        let angle = std::f32::consts::TAU * index as f32 / segments as f32;
        points.push(Point::new(
            command.center.x + angle.cos() * command.radius,
            command.center.y + angle.sin() * command.radius,
        ));
    }
    fill_polygon_mesh(command.element_id.clone(), points, command.color)
}

fn fill_polygon_mesh(
    element_id: ElementId,
    points: Vec<Point>,
    color: Color,
) -> Option<TriangleMeshPrimitive> {
    if points.len() < 3 || color.a == 0 {
        return None;
    }
    fill_antialiased_convex_polygon_mesh(element_id, &points, color, DEFAULT_ANTIALIASING_FRINGE)
}

fn fill_antialiased_convex_polygon_mesh(
    element_id: ElementId,
    points: &[Point],
    color: Color,
    fringe_width: f32,
) -> Option<TriangleMeshPrimitive> {
    if points.len() < 3 || color.a == 0 {
        return None;
    }
    if fringe_width <= f32::EPSILON {
        return fill_solid_polygon_mesh(element_id, points, color);
    }
    let edge_normals = polygon_outward_edge_normals(points)?;
    let mut vertices = Vec::with_capacity(points.len() * 2);
    vertices.extend(
        points
            .iter()
            .copied()
            .map(|position| PrimitiveVertex { position, color }),
    );
    vertices.extend(points.iter().enumerate().map(|(index, point)| {
        let previous = edge_normals[(index + edge_normals.len() - 1) % edge_normals.len()];
        let next = edge_normals[index];
        let normal = normalized_or(next, add_points(previous, next));
        let dot = dot_points(normal, next).abs().max(0.25);
        let miter_length = (fringe_width / dot).min(fringe_width * 4.0);
        PrimitiveVertex {
            position: add_points(*point, scale_point(normal, miter_length)),
            color: Color { a: 0, ..color },
        }
    }));

    let count = points.len();
    let mut indices = Vec::with_capacity((count - 2) * 3 + count * 6);
    for index in 1..count - 1 {
        indices.extend([0, index as u32, index as u32 + 1]);
    }
    for index in 0..count {
        let next = (index + 1) % count;
        let inner_a = index as u32;
        let inner_b = next as u32;
        let outer_a = (count + index) as u32;
        let outer_b = (count + next) as u32;
        indices.extend([inner_a, inner_b, outer_b, inner_a, outer_b, outer_a]);
    }
    Some(TriangleMeshPrimitive {
        element_id,
        vertices,
        indices,
    })
}

fn fill_solid_polygon_mesh(
    element_id: ElementId,
    points: &[Point],
    color: Color,
) -> Option<TriangleMeshPrimitive> {
    if points.len() < 3 || color.a == 0 {
        return None;
    }
    let vertices = points
        .iter()
        .copied()
        .map(|position| PrimitiveVertex { position, color })
        .collect();
    let mut indices = Vec::with_capacity((points.len() - 2) * 3);
    for index in 1..points.len() - 1 {
        indices.extend([0, index as u32, index as u32 + 1]);
    }
    Some(TriangleMeshPrimitive {
        element_id,
        vertices,
        indices,
    })
}

fn polygon_outward_edge_normals(points: &[Point]) -> Option<Vec<Point>> {
    let area = signed_polygon_area(points);
    if area.abs() <= f32::EPSILON {
        return None;
    }
    let clockwise = area > 0.0;
    let mut normals = Vec::with_capacity(points.len());
    for index in 0..points.len() {
        let next = (index + 1) % points.len();
        let edge = sub_points(points[next], points[index]);
        let length = point_length(edge);
        if length <= f32::EPSILON {
            return None;
        }
        let normal = if clockwise {
            Point::new(edge.y / length, -edge.x / length)
        } else {
            Point::new(-edge.y / length, edge.x / length)
        };
        normals.push(normal);
    }
    Some(normals)
}

fn signed_polygon_area(points: &[Point]) -> f32 {
    let mut area = 0.0;
    for index in 0..points.len() {
        let next = (index + 1) % points.len();
        area += points[index].x * points[next].y - points[next].x * points[index].y;
    }
    area * 0.5
}

fn normalized_or(fallback: Point, point: Point) -> Point {
    let length = point_length(point);
    if length <= f32::EPSILON {
        fallback
    } else {
        scale_point(point, 1.0 / length)
    }
}

fn rect_points(rect: Rect) -> Vec<Point> {
    vec![
        rect.origin,
        Point::new(rect.right(), rect.origin.y),
        Point::new(rect.right(), rect.bottom()),
        Point::new(rect.origin.x, rect.bottom()),
    ]
}

fn rounded_rect_points(rect: Rect, radius: CornerRadii) -> Vec<Point> {
    let max_x = rect.size.width * 0.5;
    let max_y = rect.size.height * 0.5;
    let top_left = radius.top_left.min(max_x).min(max_y).max(0.0);
    let top_right = radius.top_right.min(max_x).min(max_y).max(0.0);
    let bottom_right = radius.bottom_right.min(max_x).min(max_y).max(0.0);
    let bottom_left = radius.bottom_left.min(max_x).min(max_y).max(0.0);
    let mut points = Vec::new();
    append_corner_arc(
        &mut points,
        Point::new(rect.origin.x + top_left, rect.origin.y + top_left),
        top_left,
        std::f32::consts::PI,
        std::f32::consts::PI * 1.5,
    );
    append_corner_arc(
        &mut points,
        Point::new(rect.right() - top_right, rect.origin.y + top_right),
        top_right,
        std::f32::consts::PI * 1.5,
        std::f32::consts::TAU,
    );
    append_corner_arc(
        &mut points,
        Point::new(rect.right() - bottom_right, rect.bottom() - bottom_right),
        bottom_right,
        0.0,
        std::f32::consts::FRAC_PI_2,
    );
    append_corner_arc(
        &mut points,
        Point::new(rect.origin.x + bottom_left, rect.bottom() - bottom_left),
        bottom_left,
        std::f32::consts::FRAC_PI_2,
        std::f32::consts::PI,
    );
    points
}

fn append_corner_arc(points: &mut Vec<Point>, center: Point, radius: f32, start: f32, end: f32) {
    if radius <= f32::EPSILON {
        points.push(center);
        return;
    }
    let segments = corner_segments(radius);
    for index in 0..=segments {
        let t = index as f32 / segments as f32;
        let angle = start + (end - start) * t;
        points.push(Point::new(
            center.x + angle.cos() * radius,
            center.y + angle.sin() * radius,
        ));
    }
}

fn corner_segments(radius: f32) -> usize {
    ((radius / 3.0).ceil() as usize).clamp(3, 10)
}

fn circle_segments(radius: f32) -> usize {
    ((radius * 1.5).ceil() as usize).clamp(12, 48)
}

fn append_surface_commands(list: &mut DisplayList, frame: &ResolvedElement) {
    let arrow = floating_arrow(frame);
    append_shadow_commands(
        list,
        frame.id.clone(),
        frame.rect,
        frame.style.radius,
        &frame.style.shadows,
        arrow,
    );
    if let Some(color) = frame.style.background {
        list.push(PaintCommand::FillRect(FillRectPaint {
            element_id: frame.id.clone(),
            rect: frame.rect,
            radius: frame.style.radius,
            color,
        }));
        if let Some(arrow) = arrow {
            list.push(PaintCommand::FillPolygon(FillPolygonPaint {
                element_id: frame.id.clone(),
                points: arrow.points.to_vec(),
                color,
            }));
        }
    }
    if let Some(color) = frame.style.border {
        let border = BorderPaint {
            color,
            widths: frame.style.border_width,
            style: frame.style.border_style,
        };
        append_border_commands(
            list,
            frame.id.clone(),
            frame.rect,
            frame.style.radius,
            border,
        );
        if let Some(arrow) = arrow {
            append_arrow_border_command(list, frame.id.clone(), arrow, border);
        }
    }
}

fn text_paint(frame: &ResolvedElement, text: &str, output: &DocumentOutput) -> TextPaint {
    let rect = content_rect(frame);
    let selection = output.text_selection.as_ref().and_then(|selection| {
        if frame.selectable_text && selection.target == frame.id {
            Some(TextSelectionPaint {
                anchor_index: selection.anchor_index,
                focus_index: selection.focus_index,
                background: frame.style.text_selection_background,
                color: frame.style.text_selection_color,
            })
        } else {
            None
        }
    });

    TextPaint {
        element_id: frame.id.clone(),
        rect,
        text: text.to_owned(),
        color: frame.style.text_color,
        font_size: frame.style.font_size,
        wrap_width: if frame.style.text_wrap == TextWrapMode::Extend {
            f32::INFINITY
        } else {
            rect.size.width
        },
        wrap_mode: frame.style.text_wrap,
        max_lines: frame.style.max_lines,
        line_height: frame.style.line_height,
        selection,
    }
}

fn child_clip_rect(frame: &ResolvedElement, current_clip: Option<Rect>) -> Option<Rect> {
    if frame.style.overflow_x != Overflow::Scroll && frame.style.overflow_y != Overflow::Scroll {
        return current_clip;
    }

    let content = content_rect(frame);
    let base = current_clip.unwrap_or(frame.rect);
    let clip = Rect::new(
        if frame.style.overflow_x == Overflow::Scroll {
            content.origin.x
        } else {
            base.origin.x
        },
        if frame.style.overflow_y == Overflow::Scroll {
            content.origin.y
        } else {
            base.origin.y
        },
        if frame.style.overflow_x == Overflow::Scroll {
            content.size.width
        } else {
            base.size.width
        },
        if frame.style.overflow_y == Overflow::Scroll {
            content.size.height
        } else {
            base.size.height
        },
    );
    base.intersect(clip)
}

fn floating_arrow(frame: &ResolvedElement) -> Option<FloatingArrowPaint> {
    let floating = frame.floating?;
    let offset = floating.arrow_offset?;
    let size = floating.arrow_size?;
    Some(FloatingArrowPaint {
        points: floating_arrow_points(
            frame.rect,
            floating.placement,
            offset.x,
            offset.y,
            size.width,
            size.height,
        ),
    })
}

pub fn floating_arrow_points(
    rect: Rect,
    placement: FloatingPlacement,
    offset_x: f32,
    offset_y: f32,
    width: f32,
    height: f32,
) -> [Point; 3] {
    match placement {
        FloatingPlacement::Center => {
            let center = Point::new(
                rect.origin.x + rect.size.width * 0.5,
                rect.origin.y + rect.size.height * 0.5,
            );
            [center, center, center]
        }
        FloatingPlacement::Bottom
        | FloatingPlacement::BottomStart
        | FloatingPlacement::BottomEnd => {
            let left = rect.origin.x + offset_x;
            let center = left + width * 0.5;
            [
                Point::new(left, rect.origin.y),
                Point::new(left + width, rect.origin.y),
                Point::new(center, rect.origin.y - height),
            ]
        }
        FloatingPlacement::Top | FloatingPlacement::TopStart | FloatingPlacement::TopEnd => {
            let left = rect.origin.x + offset_x;
            let center = left + width * 0.5;
            [
                Point::new(left + width, rect.bottom()),
                Point::new(left, rect.bottom()),
                Point::new(center, rect.bottom() + height),
            ]
        }
        FloatingPlacement::Right | FloatingPlacement::RightStart | FloatingPlacement::RightEnd => {
            let top = rect.origin.y + offset_y;
            let center = top + height * 0.5;
            [
                Point::new(rect.origin.x, top + height),
                Point::new(rect.origin.x, top),
                Point::new(rect.origin.x - width, center),
            ]
        }
        FloatingPlacement::Left | FloatingPlacement::LeftStart | FloatingPlacement::LeftEnd => {
            let top = rect.origin.y + offset_y;
            let center = top + height * 0.5;
            [
                Point::new(rect.right(), top),
                Point::new(rect.right(), top + height),
                Point::new(rect.right() + width, center),
            ]
        }
    }
}

fn append_border_commands(
    list: &mut DisplayList,
    element_id: ElementId,
    rect: Rect,
    radius: CornerRadii,
    border: BorderPaint,
) {
    let width = max_inset(border.widths);
    if width <= 0.0 {
        return;
    }
    if border.style != BorderStyle::Solid {
        append_segmented_border_commands(list, element_id, rect, border.style, width, border.color);
        return;
    }
    if border.widths.is_uniform() {
        list.push(PaintCommand::StrokeRect(StrokeRectPaint {
            element_id,
            rect,
            radius,
            width: border.widths.top,
            color: border.color,
        }));
        return;
    }
    let color = border.color;
    for fill in [
        (
            border.widths.top,
            Rect::new(
                rect.origin.x,
                rect.origin.y,
                rect.size.width,
                border.widths.top,
            ),
        ),
        (
            border.widths.right,
            Rect::new(
                rect.right() - border.widths.right,
                rect.origin.y,
                border.widths.right,
                rect.size.height,
            ),
        ),
        (
            border.widths.bottom,
            Rect::new(
                rect.origin.x,
                rect.bottom() - border.widths.bottom,
                rect.size.width,
                border.widths.bottom,
            ),
        ),
        (
            border.widths.left,
            Rect::new(
                rect.origin.x,
                rect.origin.y,
                border.widths.left,
                rect.size.height,
            ),
        ),
    ] {
        if fill.0 > 0.0 {
            list.push(PaintCommand::FillRect(FillRectPaint {
                element_id: element_id.clone(),
                rect: fill.1,
                radius: CornerRadii::ZERO,
                color,
            }));
        }
    }
}

fn append_segmented_border_commands(
    list: &mut DisplayList,
    element_id: ElementId,
    rect: Rect,
    style: BorderStyle,
    width: f32,
    color: Color,
) {
    match style {
        BorderStyle::Solid => {}
        BorderStyle::Dashed => {
            let dash = (width * 3.0).max(6.0);
            let gap = (width * 2.0).max(4.0);
            append_corner_preserved_dashed_border(list, element_id, rect, dash, gap, width, color);
        }
        BorderStyle::Dotted => {
            let radius = (width * 0.5).max(1.0);
            let gap = (width * 3.0).max(6.0);
            append_corner_preserved_dotted_border(list, element_id, rect, radius, gap, color);
        }
    }
}

fn append_corner_preserved_dashed_border(
    list: &mut DisplayList,
    element_id: ElementId,
    rect: Rect,
    dash: f32,
    gap: f32,
    width: f32,
    color: Color,
) {
    let corner_x = dash.min(rect.size.width * 0.5);
    let corner_y = dash.min(rect.size.height * 0.5);
    for (corner, horizontal, vertical) in [
        (
            rect.origin,
            Point::new(corner_x, 0.0),
            Point::new(0.0, corner_y),
        ),
        (
            Point::new(rect.right(), rect.origin.y),
            Point::new(-corner_x, 0.0),
            Point::new(0.0, corner_y),
        ),
        (
            Point::new(rect.right(), rect.bottom()),
            Point::new(-corner_x, 0.0),
            Point::new(0.0, -corner_y),
        ),
        (
            Point::new(rect.origin.x, rect.bottom()),
            Point::new(corner_x, 0.0),
            Point::new(0.0, -corner_y),
        ),
    ] {
        list.push(PaintCommand::StrokePath(StrokePathPaint {
            element_id: element_id.clone(),
            points: vec![
                add_points(corner, horizontal),
                corner,
                add_points(corner, vertical),
            ],
            width,
            color,
            closed: false,
        }));
    }
    append_distributed_dashes(
        list,
        element_id.clone(),
        Point::new(rect.origin.x + corner_x + gap, rect.origin.y),
        Point::new(rect.right() - corner_x - gap, rect.origin.y),
        dash,
        gap,
        width,
        color,
    );
    append_distributed_dashes(
        list,
        element_id.clone(),
        Point::new(rect.right(), rect.origin.y + corner_y + gap),
        Point::new(rect.right(), rect.bottom() - corner_y - gap),
        dash,
        gap,
        width,
        color,
    );
    append_distributed_dashes(
        list,
        element_id.clone(),
        Point::new(rect.right() - corner_x - gap, rect.bottom()),
        Point::new(rect.origin.x + corner_x + gap, rect.bottom()),
        dash,
        gap,
        width,
        color,
    );
    append_distributed_dashes(
        list,
        element_id,
        Point::new(rect.origin.x, rect.bottom() - corner_y - gap),
        Point::new(rect.origin.x, rect.origin.y + corner_y + gap),
        dash,
        gap,
        width,
        color,
    );
}

fn append_corner_preserved_dotted_border(
    list: &mut DisplayList,
    element_id: ElementId,
    rect: Rect,
    radius: f32,
    gap: f32,
    color: Color,
) {
    for center in [
        rect.origin,
        Point::new(rect.right(), rect.origin.y),
        Point::new(rect.right(), rect.bottom()),
        Point::new(rect.origin.x, rect.bottom()),
    ] {
        list.push(PaintCommand::FillCircle(FillCirclePaint {
            element_id: element_id.clone(),
            center,
            radius,
            color,
        }));
    }
    let corner_gap = gap.max(radius * 3.0);
    append_dotted_segment(
        list,
        element_id.clone(),
        Point::new(rect.origin.x + corner_gap, rect.origin.y),
        Point::new(rect.right() - corner_gap, rect.origin.y),
        radius,
        gap,
        color,
    );
    append_dotted_segment(
        list,
        element_id.clone(),
        Point::new(rect.right(), rect.origin.y + corner_gap),
        Point::new(rect.right(), rect.bottom() - corner_gap),
        radius,
        gap,
        color,
    );
    append_dotted_segment(
        list,
        element_id.clone(),
        Point::new(rect.right() - corner_gap, rect.bottom()),
        Point::new(rect.origin.x + corner_gap, rect.bottom()),
        radius,
        gap,
        color,
    );
    append_dotted_segment(
        list,
        element_id,
        Point::new(rect.origin.x, rect.bottom() - corner_gap),
        Point::new(rect.origin.x, rect.origin.y + corner_gap),
        radius,
        gap,
        color,
    );
}

fn append_distributed_dashes(
    list: &mut DisplayList,
    element_id: ElementId,
    start: Point,
    end: Point,
    preferred_dash: f32,
    preferred_gap: f32,
    width: f32,
    color: Color,
) {
    let vector = sub_points(end, start);
    let length = point_length(vector);
    if length <= f32::EPSILON {
        return;
    }
    let pattern = distributed_dash_pattern(length, preferred_dash, preferred_gap);
    let direction = scale_point(vector, 1.0 / length);
    let mut cursor = pattern.leading_gap;
    for _ in 0..pattern.count {
        list.push(PaintCommand::StrokeLine(StrokeLinePaint {
            element_id: element_id.clone(),
            from: add_points(start, scale_point(direction, cursor)),
            to: add_points(start, scale_point(direction, cursor + pattern.dash)),
            width,
            color,
        }));
        cursor += pattern.dash + pattern.gap;
    }
}

fn append_dotted_segment(
    list: &mut DisplayList,
    element_id: ElementId,
    start: Point,
    end: Point,
    radius: f32,
    gap: f32,
    color: Color,
) {
    let vector = sub_points(end, start);
    let length = point_length(vector);
    if length <= f32::EPSILON {
        return;
    }
    let direction = scale_point(vector, 1.0 / length);
    let mut cursor = 0.0;
    while cursor <= length {
        list.push(PaintCommand::FillCircle(FillCirclePaint {
            element_id: element_id.clone(),
            center: add_points(start, scale_point(direction, cursor)),
            radius,
            color,
        }));
        cursor += gap;
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DistributedDashPattern {
    pub count: usize,
    pub dash: f32,
    pub gap: f32,
    pub leading_gap: f32,
}

pub fn distributed_dash_pattern(
    length: f32,
    preferred_dash: f32,
    preferred_gap: f32,
) -> DistributedDashPattern {
    if length <= preferred_dash {
        return DistributedDashPattern {
            count: 1,
            dash: length.max(0.0),
            gap: 0.0,
            leading_gap: 0.0,
        };
    }
    let pattern = preferred_dash + preferred_gap;
    let count = ((length + preferred_gap) / pattern).floor().max(1.0) as usize;
    if count == 1 {
        return DistributedDashPattern {
            count,
            dash: preferred_dash.min(length),
            gap: 0.0,
            leading_gap: ((length - preferred_dash).max(0.0)) * 0.5,
        };
    }
    let used_dash = preferred_dash * count as f32;
    let remaining = (length - used_dash).max(0.0);
    DistributedDashPattern {
        count,
        dash: preferred_dash,
        gap: remaining / (count - 1) as f32,
        leading_gap: 0.0,
    }
}

fn append_shadow_commands(
    list: &mut DisplayList,
    element_id: ElementId,
    rect: Rect,
    radius: CornerRadii,
    shadows: &[Shadow],
    arrow: Option<FloatingArrowPaint>,
) {
    for shadow in shadows.iter().rev().copied() {
        append_shadow_command(list, element_id.clone(), rect, radius, shadow);
        if let Some(arrow) = arrow {
            append_arrow_shadow_command(list, element_id.clone(), arrow, shadow);
        }
    }
}

fn append_shadow_command(
    list: &mut DisplayList,
    element_id: ElementId,
    rect: Rect,
    radius: CornerRadii,
    shadow: Shadow,
) {
    if shadow.color.a == 0 {
        return;
    }
    let translated = translate_rect(rect, shadow.offset);
    if shadow.blur <= 0.0 {
        list.push(PaintCommand::FillRect(FillRectPaint {
            element_id,
            rect: expand_rect_safely(translated, shadow.spread),
            radius: expand_radius(radius, shadow.spread),
            color: shadow.color,
        }));
        return;
    }
    let sigma = shadow.blur * 0.5;
    let max_blur_extent = sigma * 3.0;
    let steps = max_blur_extent.ceil().clamp(10.0, 36.0) as usize;
    for step in (0..steps).rev() {
        let outer_distance = max_blur_extent * (step + 1) as f32 / steps as f32;
        let inner_distance = max_blur_extent * step as f32 / steps as f32;
        let alpha = shadow_alpha(shadow.color.a, outer_distance, inner_distance, sigma);
        if alpha == 0 {
            continue;
        }
        let expansion = shadow.spread + outer_distance;
        list.push(PaintCommand::FillRect(FillRectPaint {
            element_id: element_id.clone(),
            rect: expand_rect_safely(translated, expansion),
            radius: expand_radius(radius, expansion),
            color: Color {
                a: alpha,
                ..shadow.color
            },
        }));
    }
}

fn append_arrow_shadow_command(
    list: &mut DisplayList,
    element_id: ElementId,
    arrow: FloatingArrowPaint,
    shadow: Shadow,
) {
    if shadow.color.a == 0 {
        return;
    }
    let translated = translate_arrow(arrow, shadow.offset);
    if shadow.blur <= 0.0 {
        list.push(PaintCommand::FillPolygon(FillPolygonPaint {
            element_id,
            points: expanded_arrow(translated, shadow.spread).points.to_vec(),
            color: shadow.color,
        }));
        return;
    }
    let sigma = shadow.blur * 0.5;
    let max_blur_extent = sigma * 3.0;
    let steps = max_blur_extent.ceil().clamp(10.0, 36.0) as usize;
    for step in (0..steps).rev() {
        let outer_distance = max_blur_extent * (step + 1) as f32 / steps as f32;
        let inner_distance = max_blur_extent * step as f32 / steps as f32;
        let alpha = shadow_alpha(shadow.color.a, outer_distance, inner_distance, sigma);
        if alpha == 0 {
            continue;
        }
        list.push(PaintCommand::FillPolygon(FillPolygonPaint {
            element_id: element_id.clone(),
            points: expanded_arrow(translated, shadow.spread + outer_distance)
                .points
                .to_vec(),
            color: Color {
                a: alpha,
                ..shadow.color
            },
        }));
    }
}

fn append_arrow_border_command(
    list: &mut DisplayList,
    element_id: ElementId,
    arrow: FloatingArrowPaint,
    border: BorderPaint,
) {
    let width = max_inset(border.widths);
    if width <= 0.0 {
        return;
    }
    list.push(PaintCommand::StrokePath(StrokePathPaint {
        element_id,
        points: arrow.points.to_vec(),
        width,
        color: border.color,
        closed: true,
    }));
}

fn append_glyph_commands(
    list: &mut DisplayList,
    element_id: ElementId,
    rect: Rect,
    glyph: Glyph,
    color: Color,
    size: f32,
) {
    let center = Point::new(
        rect.origin.x + rect.size.width * 0.5,
        rect.origin.y + rect.size.height * 0.5,
    );
    let half = (size.min(rect.size.width).min(rect.size.height) / 2.0).max(1.0);
    let width = (size / 8.0).clamp(1.25, 2.5);
    match glyph {
        Glyph::Check => {
            append_glyph_line(
                list,
                element_id.clone(),
                Point::new(center.x - half * 0.55, center.y - half * 0.05),
                Point::new(center.x - half * 0.15, center.y + half * 0.38),
                width,
                color,
            );
            append_glyph_line(
                list,
                element_id,
                Point::new(center.x - half * 0.15, center.y + half * 0.38),
                Point::new(center.x + half * 0.58, center.y - half * 0.42),
                width,
                color,
            );
        }
        Glyph::ChevronDown => {
            append_glyph_line(
                list,
                element_id.clone(),
                Point::new(center.x - half * 0.5, center.y - half * 0.2),
                Point::new(center.x, center.y + half * 0.32),
                width,
                color,
            );
            append_glyph_line(
                list,
                element_id,
                Point::new(center.x, center.y + half * 0.32),
                Point::new(center.x + half * 0.5, center.y - half * 0.2),
                width,
                color,
            );
        }
        Glyph::ChevronUp => {
            append_glyph_line(
                list,
                element_id.clone(),
                Point::new(center.x - half * 0.5, center.y + half * 0.2),
                Point::new(center.x, center.y - half * 0.32),
                width,
                color,
            );
            append_glyph_line(
                list,
                element_id,
                Point::new(center.x, center.y - half * 0.32),
                Point::new(center.x + half * 0.5, center.y + half * 0.2),
                width,
                color,
            );
        }
        Glyph::DragHandle => {
            let radius = (size / 18.0).clamp(1.0, 1.7);
            let spacing_x = (size * 0.18).max(3.0);
            let spacing_y = (size * 0.24).max(4.0);
            for column in [-0.5_f32, 0.5] {
                for row in [-1.0_f32, 0.0, 1.0] {
                    list.push(PaintCommand::FillCircle(FillCirclePaint {
                        element_id: element_id.clone(),
                        center: Point::new(
                            center.x + spacing_x * column,
                            center.y + spacing_y * row,
                        ),
                        radius,
                        color,
                    }));
                }
            }
        }
    }
}

fn append_glyph_line(
    list: &mut DisplayList,
    element_id: ElementId,
    from: Point,
    to: Point,
    width: f32,
    color: Color,
) {
    list.push(PaintCommand::StrokeLine(StrokeLinePaint {
        element_id,
        from,
        to,
        width,
        color,
    }));
}

fn append_scroll_chrome_commands(list: &mut DisplayList, chrome: &ScrollChromePaint) {
    if !chrome.visible {
        return;
    }
    if let Some(track_color) = chrome.track_color {
        list.push(PaintCommand::FillRect(FillRectPaint {
            element_id: chrome.element_id.clone(),
            rect: chrome.track_rect,
            radius: CornerRadii::all(chrome.radius),
            color: track_color,
        }));
    }
    list.push(PaintCommand::FillRect(FillRectPaint {
        element_id: chrome.element_id.clone(),
        rect: chrome.handle_rect,
        radius: CornerRadii::all(chrome.radius),
        color: chrome.handle_color,
    }));
    if let Some(color) = chrome.handle_border_color
        && chrome.handle_border_width > 0.0
    {
        list.push(PaintCommand::StrokeRect(StrokeRectPaint {
            element_id: chrome.element_id.clone(),
            rect: chrome.handle_rect,
            radius: CornerRadii::all(chrome.radius),
            width: chrome.handle_border_width,
            color,
        }));
    }
}

fn translate_rect(rect: Rect, offset: Point) -> Rect {
    Rect::new(
        rect.origin.x + offset.x,
        rect.origin.y + offset.y,
        rect.size.width,
        rect.size.height,
    )
}

fn expand_rect_safely(rect: Rect, amount: f32) -> Rect {
    if amount >= 0.0 {
        return Rect::new(
            rect.origin.x - amount,
            rect.origin.y - amount,
            rect.size.width + amount * 2.0,
            rect.size.height + amount * 2.0,
        );
    }
    let inset = (-amount)
        .min(rect.size.width * 0.5)
        .min(rect.size.height * 0.5);
    Rect::new(
        rect.origin.x + inset,
        rect.origin.y + inset,
        rect.size.width - inset * 2.0,
        rect.size.height - inset * 2.0,
    )
}

fn expand_radius(radius: CornerRadii, amount: f32) -> CornerRadii {
    CornerRadii {
        top_left: (radius.top_left + amount).max(0.0),
        top_right: (radius.top_right + amount).max(0.0),
        bottom_right: (radius.bottom_right + amount).max(0.0),
        bottom_left: (radius.bottom_left + amount).max(0.0),
    }
}

fn translate_arrow(arrow: FloatingArrowPaint, offset: Point) -> FloatingArrowPaint {
    FloatingArrowPaint {
        points: arrow.points.map(|point| add_points(point, offset)),
    }
}

pub fn expanded_arrow(arrow: FloatingArrowPaint, amount: f32) -> FloatingArrowPaint {
    let center = Point::new(
        (arrow.points[0].x + arrow.points[1].x + arrow.points[2].x) / 3.0,
        (arrow.points[0].y + arrow.points[1].y + arrow.points[2].y) / 3.0,
    );
    FloatingArrowPaint {
        points: arrow.points.map(|point| {
            let vector = sub_points(point, center);
            let length = point_length(vector);
            if length <= f32::EPSILON {
                point
            } else {
                add_points(
                    center,
                    scale_point(vector, (length + amount).max(0.0) / length),
                )
            }
        }),
    }
}

fn shadow_alpha(base_alpha: u8, outer_distance: f32, inner_distance: f32, sigma: f32) -> u8 {
    let outer_alpha = gaussian_alpha(outer_distance, sigma);
    let inner_alpha = gaussian_alpha(inner_distance, sigma);
    let alpha_portion = (inner_alpha - outer_alpha).max(0.0);
    (base_alpha as f32 * alpha_portion * 0.86)
        .round()
        .clamp(0.0, 255.0) as u8
}

fn gaussian_alpha(distance: f32, sigma: f32) -> f32 {
    if sigma <= 0.0 {
        return 1.0;
    }
    (-0.5 * (distance / sigma).powi(2)).exp()
}

fn max_inset(insets: Insets) -> f32 {
    insets
        .top
        .max(insets.right)
        .max(insets.bottom)
        .max(insets.left)
}

fn add_points(lhs: Point, rhs: Point) -> Point {
    Point::new(lhs.x + rhs.x, lhs.y + rhs.y)
}

fn sub_points(lhs: Point, rhs: Point) -> Point {
    Point::new(lhs.x - rhs.x, lhs.y - rhs.y)
}

fn scale_point(point: Point, scale: f32) -> Point {
    Point::new(point.x * scale, point.y * scale)
}

fn dot_points(lhs: Point, rhs: Point) -> f32 {
    lhs.x * rhs.x + lhs.y * rhs.y
}

fn point_length(point: Point) -> f32 {
    (point.x * point.x + point.y * point.y).sqrt()
}

impl From<&ScrollChrome> for ScrollChromePaint {
    fn from(chrome: &ScrollChrome) -> Self {
        Self {
            element_id: chrome.element_id.clone(),
            axis: chrome.axis,
            track_rect: chrome.track_rect,
            hit_rect: chrome.hit_rect,
            handle_rect: chrome.handle_rect,
            handle_color: chrome.handle_color,
            track_color: chrome.track_color,
            handle_border_color: chrome.handle_border_color,
            handle_border_width: chrome.handle_border_width,
            radius: chrome.radius,
            visible: chrome.visible,
            expanded: chrome.expanded,
            hovered: chrome.hovered,
            dragged: chrome.dragged,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use des_ui_document::{
        Color, Document, DocumentEngine, Insets, Size, Style, StyleSelector, StyleSheet,
    };

    #[test]
    fn plans_surface_text_and_children_in_z_order() {
        let mut document = Document::build(Size::new(200.0, 120.0), |ui| {
            ui.div("lower").empty();
            ui.text("label", "Hello");
        });
        let stylesheet = StyleSheet::new()
            .rule(
                StyleSelector::Id("lower".into()),
                Style::default()
                    .size(40.0, 20.0)
                    .background(Color::rgb(10, 20, 30))
                    .z_index(5),
            )
            .rule(
                StyleSelector::Id("label".into()),
                Style::default().size(60.0, 20.0).z_index(1),
            );
        let output = DocumentEngine::default().update(&mut document, &stylesheet);

        let list = plan_paint(&output);
        let ids: Vec<_> = list
            .commands
            .iter()
            .filter_map(|command| match command {
                PaintCommand::FillRect(fill) => Some(fill.element_id.as_str().to_owned()),
                PaintCommand::Text(text) => Some(text.element_id.as_str().to_owned()),
                _ => None,
            })
            .collect();

        assert_eq!(ids, vec!["label", "lower"]);
    }

    #[test]
    fn computes_content_rect_from_border_and_padding() {
        let mut document = Document::build(Size::new(200.0, 120.0), |ui| {
            ui.div("box").empty();
        });
        let stylesheet = StyleSheet::new().rule(
            StyleSelector::Id("box".into()),
            Style::default()
                .size(100.0, 80.0)
                .border_widths(Insets::all(2.0))
                .padding(Insets::symmetric(10.0, 6.0)),
        );
        let output = DocumentEngine::default().update(&mut document, &stylesheet);
        let box_frame = output.layout.find("box").expect("box frame");

        assert_eq!(content_rect(box_frame), Rect::new(12.0, 8.0, 76.0, 64.0));
    }

    #[test]
    fn computes_floating_arrow_points_without_backend_types() {
        let points = floating_arrow_points(
            Rect::new(10.0, 20.0, 100.0, 50.0),
            FloatingPlacement::Bottom,
            30.0,
            0.0,
            12.0,
            8.0,
        );

        assert_eq!(
            points,
            [
                Point::new(40.0, 20.0),
                Point::new(52.0, 20.0),
                Point::new(46.0, 12.0),
            ]
        );
    }

    #[test]
    fn distributed_dash_pattern_avoids_cutoff_dash() {
        let pattern = distributed_dash_pattern(52.0, 8.0, 5.0);

        assert_eq!(pattern.count, 4);
        assert_eq!(pattern.dash, 8.0);
        assert_eq!(pattern.gap, (52.0 - 32.0) / 3.0);
        assert_eq!(
            pattern.dash * pattern.count as f32 + pattern.gap * (pattern.count - 1) as f32,
            52.0
        );
    }

    #[test]
    fn expanded_arrow_keeps_center_and_moves_points_outward() {
        let arrow = FloatingArrowPaint {
            points: [
                Point::new(0.0, 0.0),
                Point::new(10.0, 0.0),
                Point::new(5.0, -5.0),
            ],
        };

        let expanded = expanded_arrow(arrow, 2.0);

        assert!(expanded.points[0].x < arrow.points[0].x);
        assert!(expanded.points[1].x > arrow.points[1].x);
        assert!(expanded.points[2].y < arrow.points[2].y);
    }

    #[test]
    fn glyph_planning_emits_primitive_commands() {
        let mut list = DisplayList::new();
        append_glyph_commands(
            &mut list,
            "check".into(),
            Rect::new(0.0, 0.0, 20.0, 20.0),
            Glyph::Check,
            Color::rgb(1, 2, 3),
            12.0,
        );

        assert!(matches!(list.commands[0], PaintCommand::StrokeLine(_)));
        assert!(matches!(list.commands[1], PaintCommand::StrokeLine(_)));
        assert_eq!(list.commands.len(), 2);
    }

    #[test]
    fn primitive_planner_expands_shapes_and_preserves_text_and_clips() {
        let mut list = DisplayList::new();
        list.push(PaintCommand::PushClip(Rect::new(0.0, 0.0, 40.0, 30.0)));
        list.push(PaintCommand::FillRect(FillRectPaint {
            element_id: ElementId::new("surface"),
            rect: Rect::new(4.0, 5.0, 20.0, 10.0),
            radius: CornerRadii::ZERO,
            color: Color::rgb(10, 20, 30),
        }));
        list.push(PaintCommand::Text(TextPaint {
            element_id: ElementId::new("label"),
            rect: Rect::new(8.0, 9.0, 20.0, 10.0),
            text: "Hello".into(),
            color: Color::rgb(1, 2, 3),
            font_size: 12.0,
            wrap_width: 20.0,
            wrap_mode: TextWrapMode::Extend,
            max_lines: None,
            line_height: None,
            selection: None,
        }));
        list.push(PaintCommand::PopClip);

        let primitives = plan_primitives(&list);

        assert!(matches!(
            primitives.commands[0],
            PrimitiveCommand::PushClip(_)
        ));
        assert!(matches!(
            primitives.commands[1],
            PrimitiveCommand::Draw(RenderPrimitive::Triangles(_))
        ));
        assert!(matches!(
            primitives.commands[2],
            PrimitiveCommand::Draw(RenderPrimitive::Text(_))
        ));
        assert!(matches!(primitives.commands[3], PrimitiveCommand::PopClip));
    }

    #[test]
    fn rounded_rect_primitives_are_tessellated_beyond_a_plain_quad() {
        let mut list = DisplayList::new();
        list.push(PaintCommand::FillRect(FillRectPaint {
            element_id: ElementId::new("rounded"),
            rect: Rect::new(0.0, 0.0, 40.0, 24.0),
            radius: CornerRadii::all(6.0),
            color: Color::rgb(10, 20, 30),
        }));

        let primitives = plan_primitives(&list);
        let PrimitiveCommand::Draw(RenderPrimitive::Triangles(mesh)) = &primitives.commands[0]
        else {
            panic!("expected rounded rect mesh");
        };

        assert!(mesh.vertices.len() > 4);
        assert!(mesh.indices.len() > 6);
        assert_eq!(mesh.element_id.as_str(), "rounded");
    }

    #[test]
    fn filled_rect_primitives_include_antialiasing_fringe() {
        let mut list = DisplayList::new();
        list.push(PaintCommand::FillRect(FillRectPaint {
            element_id: ElementId::new("soft"),
            rect: Rect::new(10.0, 20.0, 30.0, 40.0),
            radius: CornerRadii::ZERO,
            color: Color::rgba(10, 20, 30, 220),
        }));

        let primitives = plan_primitives(&list);
        let PrimitiveCommand::Draw(RenderPrimitive::Triangles(mesh)) = &primitives.commands[0]
        else {
            panic!("expected antialiased rect mesh");
        };

        assert!(
            mesh.vertices.iter().any(|vertex| vertex.color.a == 0),
            "antialiasing requires transparent fringe vertices"
        );
        assert!(
            mesh.vertices.iter().any(|vertex| vertex.color.a == 220),
            "filled shape must retain opaque interior vertices"
        );
        assert!(
            mesh.vertices
                .iter()
                .any(|vertex| vertex.position.x < 10.0 || vertex.position.y < 20.0),
            "fringe should expand outside the shape on leading edges"
        );
        assert!(
            mesh.vertices
                .iter()
                .any(|vertex| vertex.position.x > 40.0 || vertex.position.y > 60.0),
            "fringe should expand outside the shape on trailing edges"
        );
    }

    #[test]
    fn rounded_rect_antialiasing_preserves_curved_edge_coverage() {
        let mut list = DisplayList::new();
        list.push(PaintCommand::FillRect(FillRectPaint {
            element_id: ElementId::new("soft-rounded"),
            rect: Rect::new(0.0, 0.0, 40.0, 24.0),
            radius: CornerRadii::all(6.0),
            color: Color::rgb(10, 20, 30),
        }));

        let primitives = plan_primitives(&list);
        let PrimitiveCommand::Draw(RenderPrimitive::Triangles(mesh)) = &primitives.commands[0]
        else {
            panic!("expected antialiased rounded rect mesh");
        };

        assert!(
            mesh.vertices.len() >= 2 * rounded_rect_points(Rect::new(0.0, 0.0, 40.0, 24.0), CornerRadii::all(6.0)).len(),
            "rounded rect should keep its curve samples and add an antialiasing fringe"
        );
        assert!(mesh.vertices.iter().any(|vertex| vertex.color.a == 0));
        assert!(mesh.vertices.iter().any(|vertex| vertex.color.a == 255));
    }
}
