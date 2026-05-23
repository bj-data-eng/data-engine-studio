use super::text::{layout_job, paint_document_text_selection};
use des_ui_document::{Color, CornerRadii, DocumentOutput, Point, Rect, TextLayoutRequest};
use des_ui_render::{
    DashedLinePaint, DisplayList, DottedLinePaint, FillCirclePaint, FillPolygonPaint,
    FillRectPaint, PaintCommand, ShadowRectPaint, StrokeLinePaint, StrokePathPaint,
    StrokeRectPaint, TextPaint, plan_paint,
};
use eframe::egui;

pub fn paint_output(ui: &mut egui::Ui, origin: egui::Pos2, output: &DocumentOutput) {
    let display_list = plan_paint(output);
    paint_display_list(ui, origin, &display_list);
}

pub fn paint_display_list(ui: &mut egui::Ui, origin: egui::Pos2, display_list: &DisplayList) {
    let mut clip_stack = vec![ui.clip_rect()];
    for command in &display_list.commands {
        match command {
            PaintCommand::PushClip(rect) => {
                let clip = document_rect_to_egui(origin, *rect);
                let current = *clip_stack.last().expect("clip stack is never empty");
                clip_stack.push(current.intersect(clip));
            }
            PaintCommand::PopClip => {
                if clip_stack.len() > 1 {
                    clip_stack.pop();
                }
            }
            PaintCommand::FillRect(command) => {
                let painter = clipped_painter(ui, &clip_stack);
                paint_fill_rect(&painter, origin, command);
            }
            PaintCommand::ShadowRect(command) => {
                let painter = clipped_painter(ui, &clip_stack);
                paint_shadow_rect(&painter, origin, command);
            }
            PaintCommand::StrokeRect(command) => {
                let painter = clipped_painter(ui, &clip_stack);
                paint_stroke_rect(&painter, origin, command);
            }
            PaintCommand::StrokeLine(command) => {
                let painter = clipped_painter(ui, &clip_stack);
                paint_stroke_line(&painter, origin, command);
            }
            PaintCommand::DashedLine(command) => {
                let painter = clipped_painter(ui, &clip_stack);
                paint_dashed_line(&painter, origin, command);
            }
            PaintCommand::DottedLine(command) => {
                let painter = clipped_painter(ui, &clip_stack);
                paint_dotted_line(&painter, origin, command);
            }
            PaintCommand::StrokePath(command) => {
                let painter = clipped_painter(ui, &clip_stack);
                paint_stroke_path(&painter, origin, command);
            }
            PaintCommand::FillCircle(command) => {
                let painter = clipped_painter(ui, &clip_stack);
                paint_fill_circle(&painter, origin, command);
            }
            PaintCommand::FillPolygon(command) => {
                let painter = clipped_painter(ui, &clip_stack);
                paint_fill_polygon(&painter, origin, command);
            }
            PaintCommand::Text(command) => {
                let painter = clipped_painter(ui, &clip_stack);
                paint_text(&painter, origin, command);
            }
        }
    }
}

fn clipped_painter(ui: &egui::Ui, clip_stack: &[egui::Rect]) -> egui::Painter {
    ui.painter()
        .with_clip_rect(*clip_stack.last().expect("clip stack is never empty"))
}

fn paint_fill_rect(painter: &egui::Painter, origin: egui::Pos2, command: &FillRectPaint) {
    painter.rect_filled(
        document_rect_to_egui(origin, command.rect),
        to_egui_radius(command.radius),
        to_egui_color(command.color),
    );
}

fn paint_shadow_rect(painter: &egui::Painter, origin: egui::Pos2, command: &ShadowRectPaint) {
    if command.rect.size.width <= 0.0 || command.rect.size.height <= 0.0 || command.color.a == 0 {
        return;
    }
    painter.add(egui::Shape::Rect(
        egui::epaint::RectShape::filled(
            document_rect_to_egui(origin, command.rect),
            to_egui_radius(command.radius),
            to_egui_color(command.color),
        )
        .with_blur_width(command.blur_width.max(0.0)),
    ));
}

fn paint_stroke_rect(painter: &egui::Painter, origin: egui::Pos2, command: &StrokeRectPaint) {
    if command.width <= 0.0 {
        return;
    }
    painter.rect_stroke(
        document_rect_to_egui(origin, command.rect),
        to_egui_radius(command.radius),
        egui::Stroke::new(command.width, to_egui_color(command.color)),
        egui::StrokeKind::Inside,
    );
}

fn paint_stroke_line(painter: &egui::Painter, origin: egui::Pos2, command: &StrokeLinePaint) {
    if command.width <= 0.0 {
        return;
    }
    painter.line_segment(
        [
            document_point_to_egui(origin, command.from),
            document_point_to_egui(origin, command.to),
        ],
        egui::Stroke::new(command.width, to_egui_color(command.color)),
    );
}

fn paint_dashed_line(painter: &egui::Painter, origin: egui::Pos2, command: &DashedLinePaint) {
    if command.width <= 0.0 || command.dash <= 0.0 || command.gap < 0.0 {
        return;
    }
    let path = [
        document_point_to_egui(origin, command.from),
        document_point_to_egui(origin, command.to),
    ];
    let shapes = egui::Shape::dashed_line_with_offset(
        &path,
        egui::Stroke::new(command.width, to_egui_color(command.color)),
        &[command.dash],
        &[command.gap],
        command.offset.max(0.0),
    );
    painter.extend(shapes);
}

fn paint_dotted_line(painter: &egui::Painter, origin: egui::Pos2, command: &DottedLinePaint) {
    if command.radius <= 0.0 || command.spacing <= 0.0 {
        return;
    }
    let path = [
        document_point_to_egui(origin, command.from),
        document_point_to_egui(origin, command.to),
    ];
    painter.extend(egui::Shape::dotted_line(
        &path,
        to_egui_color(command.color),
        command.spacing,
        command.radius,
    ));
}

fn paint_stroke_path(painter: &egui::Painter, origin: egui::Pos2, command: &StrokePathPaint) {
    if command.width <= 0.0 || command.points.len() < 2 {
        return;
    }
    let points: Vec<_> = command
        .points
        .iter()
        .map(|point| document_point_to_egui(origin, *point))
        .collect();
    let stroke = egui::Stroke::new(command.width, to_egui_color(command.color));
    if command.closed {
        painter.add(egui::Shape::closed_line(points, stroke));
    } else {
        painter.add(egui::Shape::line(points, stroke));
    }
}

fn paint_fill_circle(painter: &egui::Painter, origin: egui::Pos2, command: &FillCirclePaint) {
    if command.radius <= 0.0 {
        return;
    }
    painter.circle_filled(
        document_point_to_egui(origin, command.center),
        command.radius,
        to_egui_color(command.color),
    );
}

fn paint_fill_polygon(painter: &egui::Painter, origin: egui::Pos2, command: &FillPolygonPaint) {
    if command.points.len() < 3 {
        return;
    }
    let points: Vec<_> = command
        .points
        .iter()
        .map(|point| document_point_to_egui(origin, *point))
        .collect();
    painter.add(egui::Shape::convex_polygon(
        points,
        to_egui_color(command.color),
        egui::Stroke::NONE,
    ));
}

fn paint_text(painter: &egui::Painter, origin: egui::Pos2, command: &TextPaint) {
    let rect = document_rect_to_egui(origin, command.rect);
    let request = TextLayoutRequest {
        text: &command.text,
        font_size: command.font_size,
        wrap_width: command.wrap_width,
        wrap_mode: command.wrap_mode,
        max_lines: command.max_lines,
        line_height: command.line_height,
    };
    let color = to_egui_color(command.color);
    let mut galley = painter.layout_job(layout_job(request, color));
    if let Some(selection) = command.selection {
        let cursor_range = egui::text_selection::CCursorRange::two(
            egui::text::CCursor::new(selection.anchor_index),
            egui::text::CCursor::new(selection.focus_index),
        );
        paint_document_text_selection(
            &mut galley,
            &cursor_range,
            selection.background,
            selection.color,
        );
    }
    painter.galley(rect.min, galley, color);
}

fn document_rect_to_egui(origin: egui::Pos2, rect: Rect) -> egui::Rect {
    egui::Rect::from_min_size(
        document_point_to_egui(origin, rect.origin),
        egui::vec2(rect.size.width, rect.size.height),
    )
}

fn document_point_to_egui(origin: egui::Pos2, point: Point) -> egui::Pos2 {
    egui::pos2(origin.x + point.x, origin.y + point.y)
}

fn to_egui_color(color: Color) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(color.r, color.g, color.b, color.a)
}

fn to_egui_radius(radius: CornerRadii) -> egui::CornerRadius {
    egui::CornerRadius {
        nw: radius.top_left.round().clamp(0.0, 255.0) as u8,
        ne: radius.top_right.round().clamp(0.0, 255.0) as u8,
        se: radius.bottom_right.round().clamp(0.0, 255.0) as u8,
        sw: radius.bottom_left.round().clamp(0.0, 255.0) as u8,
    }
}
