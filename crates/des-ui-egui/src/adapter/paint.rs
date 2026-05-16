use super::text::{layout_job, paint_document_text_selection};
use des_ui_document::{
    BorderStyle, Color, CornerRadii, DocumentOutput, Glyph, Insets, Point, Rect, Shadow,
    TextLayoutRequest,
};
use des_ui_render::{
    BorderPaint, DisplayList, FloatingArrowPaint, GlyphPaint, PaintCommand, ScrollChromePaint,
    SurfacePaint, TextPaint, plan_paint,
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
            PaintCommand::Surface(surface) => {
                let painter = ui
                    .painter()
                    .with_clip_rect(*clip_stack.last().expect("clip stack is never empty"));
                paint_surface_command(&painter, origin, surface);
            }
            PaintCommand::Text(text) => {
                let painter = ui
                    .painter()
                    .with_clip_rect(*clip_stack.last().expect("clip stack is never empty"));
                paint_text_command(&painter, origin, text);
            }
            PaintCommand::Glyph(glyph) => {
                let painter = ui
                    .painter()
                    .with_clip_rect(*clip_stack.last().expect("clip stack is never empty"));
                paint_glyph_command(&painter, origin, glyph);
            }
            PaintCommand::ScrollChrome(chrome) => {
                let painter = ui
                    .painter()
                    .with_clip_rect(*clip_stack.last().expect("clip stack is never empty"));
                paint_scroll_chrome_command(&painter, origin, chrome);
            }
        }
    }
}

fn paint_surface_command(painter: &egui::Painter, origin: egui::Pos2, surface: &SurfacePaint) {
    let rect = document_rect_to_egui(origin, surface.rect);
    if let Some(arrow) = surface.floating_arrow {
        paint_floating_surface(painter, rect, surface, egui_arrow(arrow));
        return;
    }

    paint_shadows(painter, rect, surface.radius, &surface.shadows);
    if let Some(color) = surface.background {
        painter.rect_filled(rect, to_egui_radius(surface.radius), to_egui_color(color));
    }
    if let Some(border) = surface.border {
        paint_border(
            painter,
            rect,
            surface.radius,
            border.widths,
            border.style,
            border.color,
        );
    }
}

fn paint_text_command(painter: &egui::Painter, origin: egui::Pos2, text: &TextPaint) {
    let rect = document_rect_to_egui(origin, text.rect);
    let request = TextLayoutRequest {
        text: &text.text,
        font_size: text.font_size,
        wrap_width: text.wrap_width,
        wrap_mode: text.wrap_mode,
        max_lines: text.max_lines,
        line_height: text.line_height,
    };
    let color = to_egui_color(text.color);
    let mut galley = painter.layout_job(layout_job(request, color));
    if let Some(selection) = text.selection {
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

fn paint_glyph_command(painter: &egui::Painter, origin: egui::Pos2, glyph: &GlyphPaint) {
    paint_glyph(
        painter,
        document_rect_to_egui(origin, glyph.rect),
        glyph.glyph,
        glyph.color,
        glyph.size,
    );
}

fn paint_scroll_chrome_command(
    painter: &egui::Painter,
    origin: egui::Pos2,
    chrome: &ScrollChromePaint,
) {
    if !chrome.visible {
        return;
    }

    let track = document_rect_to_egui(origin, chrome.track_rect);
    let handle = document_rect_to_egui(origin, chrome.handle_rect);
    if let Some(track_color) = chrome.track_color {
        painter.rect_filled(track, chrome.radius, to_egui_color(track_color));
    }
    painter.rect_filled(handle, chrome.radius, to_egui_color(chrome.handle_color));
    if let Some(border_color) = chrome.handle_border_color
        && chrome.handle_border_width > 0.0
    {
        painter.rect_stroke(
            handle,
            chrome.radius,
            egui::Stroke::new(chrome.handle_border_width, to_egui_color(border_color)),
            egui::StrokeKind::Inside,
        );
    }
}

pub fn paint_surface(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    radius: CornerRadii,
    shadows: &[Shadow],
    background: Color,
    border: Option<Color>,
    border_width: Insets,
) {
    let painter = ui.painter();
    paint_shadows(painter, rect, radius, shadows);
    painter.rect_filled(rect, to_egui_radius(radius), to_egui_color(background));
    if let Some(border) = border {
        paint_border(
            painter,
            rect,
            radius,
            border_width,
            BorderStyle::Solid,
            border,
        );
    }
}

fn paint_glyph(painter: &egui::Painter, rect: egui::Rect, glyph: Glyph, color: Color, size: f32) {
    let color = to_egui_color(color);
    let stroke = egui::Stroke::new((size / 8.0).clamp(1.25, 2.5), color);
    let center = rect.center();
    let half = (size.min(rect.width()).min(rect.height()) / 2.0).max(1.0);
    match glyph {
        Glyph::Check => {
            let a = egui::pos2(center.x - half * 0.55, center.y - half * 0.05);
            let b = egui::pos2(center.x - half * 0.15, center.y + half * 0.38);
            let c = egui::pos2(center.x + half * 0.58, center.y - half * 0.42);
            painter.line_segment([a, b], stroke);
            painter.line_segment([b, c], stroke);
        }
        Glyph::ChevronDown => {
            let a = egui::pos2(center.x - half * 0.5, center.y - half * 0.2);
            let b = egui::pos2(center.x, center.y + half * 0.32);
            let c = egui::pos2(center.x + half * 0.5, center.y - half * 0.2);
            painter.line_segment([a, b], stroke);
            painter.line_segment([b, c], stroke);
        }
        Glyph::ChevronUp => {
            let a = egui::pos2(center.x - half * 0.5, center.y + half * 0.2);
            let b = egui::pos2(center.x, center.y - half * 0.32);
            let c = egui::pos2(center.x + half * 0.5, center.y + half * 0.2);
            painter.line_segment([a, b], stroke);
            painter.line_segment([b, c], stroke);
        }
        Glyph::DragHandle => {
            let radius = (size / 18.0).clamp(1.0, 1.7);
            let spacing_x = (size * 0.18).max(3.0);
            let spacing_y = (size * 0.24).max(4.0);
            for column in [-0.5_f32, 0.5] {
                for row in [-1.0_f32, 0.0, 1.0] {
                    painter.circle_filled(
                        egui::pos2(center.x + spacing_x * column, center.y + spacing_y * row),
                        radius,
                        color,
                    );
                }
            }
        }
    }
}

fn paint_border(
    painter: &egui::Painter,
    rect: egui::Rect,
    radius: CornerRadii,
    widths: Insets,
    style: BorderStyle,
    color: Color,
) {
    let color = to_egui_color(color);
    if style != BorderStyle::Solid {
        paint_segmented_border(painter, rect, widths, style, color);
        return;
    }
    if widths.is_uniform() {
        if widths.top > 0.0 {
            painter.rect_stroke(
                rect,
                to_egui_radius(radius),
                egui::Stroke::new(widths.top, color),
                egui::StrokeKind::Inside,
            );
        }
        return;
    }

    if widths.top > 0.0 {
        painter.rect_filled(
            egui::Rect::from_min_max(
                rect.left_top(),
                egui::pos2(rect.right(), rect.top() + widths.top),
            ),
            0.0,
            color,
        );
    }
    if widths.right > 0.0 {
        painter.rect_filled(
            egui::Rect::from_min_max(
                egui::pos2(rect.right() - widths.right, rect.top()),
                rect.right_bottom(),
            ),
            0.0,
            color,
        );
    }
    if widths.bottom > 0.0 {
        painter.rect_filled(
            egui::Rect::from_min_max(
                egui::pos2(rect.left(), rect.bottom() - widths.bottom),
                rect.right_bottom(),
            ),
            0.0,
            color,
        );
    }
    if widths.left > 0.0 {
        painter.rect_filled(
            egui::Rect::from_min_max(
                rect.left_top(),
                egui::pos2(rect.left() + widths.left, rect.bottom()),
            ),
            0.0,
            color,
        );
    }
}

fn paint_segmented_border(
    painter: &egui::Painter,
    rect: egui::Rect,
    widths: Insets,
    style: BorderStyle,
    color: egui::Color32,
) {
    let width = widths
        .top
        .max(widths.right)
        .max(widths.bottom)
        .max(widths.left);
    if width <= 0.0 {
        return;
    }
    let stroke = egui::Stroke::new(width, color);
    match style {
        BorderStyle::Solid => {}
        BorderStyle::Dashed => {
            let dash = (width * 3.0).max(6.0);
            let gap = (width * 2.0).max(4.0);
            paint_corner_preserved_dashed_border(painter, rect, dash, gap, stroke);
        }
        BorderStyle::Dotted => {
            let radius = (width * 0.5).max(1.0);
            let gap = (width * 3.0).max(6.0);
            paint_corner_preserved_dotted_border(painter, rect, radius, gap, color);
        }
    }
}

fn paint_corner_preserved_dashed_border(
    painter: &egui::Painter,
    rect: egui::Rect,
    dash: f32,
    gap: f32,
    stroke: egui::Stroke,
) {
    let corner_x = dash.min(rect.width() * 0.5);
    let corner_y = dash.min(rect.height() * 0.5);
    paint_dashed_corner_segments(painter, rect, corner_x, corner_y, stroke);
    paint_distributed_dashes(
        painter,
        rect.left_top() + egui::vec2(corner_x + gap, 0.0),
        rect.right_top() - egui::vec2(corner_x + gap, 0.0),
        dash,
        gap,
        stroke,
    );
    paint_distributed_dashes(
        painter,
        rect.right_top() + egui::vec2(0.0, corner_y + gap),
        rect.right_bottom() - egui::vec2(0.0, corner_y + gap),
        dash,
        gap,
        stroke,
    );
    paint_distributed_dashes(
        painter,
        rect.right_bottom() - egui::vec2(corner_x + gap, 0.0),
        rect.left_bottom() + egui::vec2(corner_x + gap, 0.0),
        dash,
        gap,
        stroke,
    );
    paint_distributed_dashes(
        painter,
        rect.left_bottom() - egui::vec2(0.0, corner_y + gap),
        rect.left_top() + egui::vec2(0.0, corner_y + gap),
        dash,
        gap,
        stroke,
    );
}

fn paint_dashed_corner_segments(
    painter: &egui::Painter,
    rect: egui::Rect,
    corner_x: f32,
    corner_y: f32,
    stroke: egui::Stroke,
) {
    for (corner, horizontal, vertical) in [
        (
            rect.left_top(),
            egui::vec2(corner_x, 0.0),
            egui::vec2(0.0, corner_y),
        ),
        (
            rect.right_top(),
            egui::vec2(-corner_x, 0.0),
            egui::vec2(0.0, corner_y),
        ),
        (
            rect.right_bottom(),
            egui::vec2(-corner_x, 0.0),
            egui::vec2(0.0, -corner_y),
        ),
        (
            rect.left_bottom(),
            egui::vec2(corner_x, 0.0),
            egui::vec2(0.0, -corner_y),
        ),
    ] {
        painter.add(egui::Shape::line(
            vec![corner + horizontal, corner, corner + vertical],
            stroke,
        ));
    }
}

fn paint_corner_preserved_dotted_border(
    painter: &egui::Painter,
    rect: egui::Rect,
    radius: f32,
    gap: f32,
    color: egui::Color32,
) {
    for corner in [
        rect.left_top(),
        rect.right_top(),
        rect.right_bottom(),
        rect.left_bottom(),
    ] {
        painter.circle_filled(corner, radius, color);
    }
    let corner_gap = gap.max(radius * 3.0);
    paint_dotted_segment(
        painter,
        rect.left_top() + egui::vec2(corner_gap, 0.0),
        rect.right_top() - egui::vec2(corner_gap, 0.0),
        radius,
        gap,
        color,
    );
    paint_dotted_segment(
        painter,
        rect.right_top() + egui::vec2(0.0, corner_gap),
        rect.right_bottom() - egui::vec2(0.0, corner_gap),
        radius,
        gap,
        color,
    );
    paint_dotted_segment(
        painter,
        rect.right_bottom() - egui::vec2(corner_gap, 0.0),
        rect.left_bottom() + egui::vec2(corner_gap, 0.0),
        radius,
        gap,
        color,
    );
    paint_dotted_segment(
        painter,
        rect.left_bottom() - egui::vec2(0.0, corner_gap),
        rect.left_top() + egui::vec2(0.0, corner_gap),
        radius,
        gap,
        color,
    );
}

fn paint_distributed_dashes(
    painter: &egui::Painter,
    start: egui::Pos2,
    end: egui::Pos2,
    preferred_dash: f32,
    preferred_gap: f32,
    stroke: egui::Stroke,
) {
    let vector = end - start;
    let length = vector.length();
    if length <= f32::EPSILON {
        return;
    }
    let pattern = distributed_dash_pattern(length, preferred_dash, preferred_gap);
    let direction = vector / length;
    let mut cursor = pattern.leading_gap;
    for _ in 0..pattern.count {
        painter.line_segment(
            [
                start + direction * cursor,
                start + direction * (cursor + pattern.dash),
            ],
            stroke,
        );
        cursor += pattern.dash + pattern.gap;
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct DistributedDashPattern {
    count: usize,
    dash: f32,
    gap: f32,
    leading_gap: f32,
}

fn distributed_dash_pattern(
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

fn paint_dotted_segment(
    painter: &egui::Painter,
    start: egui::Pos2,
    end: egui::Pos2,
    radius: f32,
    gap: f32,
    color: egui::Color32,
) {
    let vector = end - start;
    let length = vector.length();
    if length <= f32::EPSILON {
        return;
    }
    let direction = vector / length;
    let mut cursor = 0.0;
    while cursor <= length {
        painter.circle_filled(start + direction * cursor, radius, color);
        cursor += gap;
    }
}

fn paint_shadows(
    painter: &egui::Painter,
    rect: egui::Rect,
    radius: CornerRadii,
    shadows: &[Shadow],
) {
    for shadow in shadows.iter().rev().copied() {
        paint_shadow(painter, rect, radius, shadow);
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct EguiFloatingArrowPaint {
    points: [egui::Pos2; 3],
}

fn egui_arrow(arrow: FloatingArrowPaint) -> EguiFloatingArrowPaint {
    EguiFloatingArrowPaint {
        points: arrow.points.map(point_to_egui),
    }
}

fn paint_floating_surface(
    painter: &egui::Painter,
    rect: egui::Rect,
    surface: &SurfacePaint,
    arrow: EguiFloatingArrowPaint,
) {
    paint_shadows_with_arrow(painter, rect, surface.radius, &surface.shadows, arrow);
    if let Some(color) = surface.background {
        painter.rect_filled(rect, to_egui_radius(surface.radius), to_egui_color(color));
        painter.add(egui::Shape::convex_polygon(
            arrow.points.to_vec(),
            to_egui_color(color),
            egui::Stroke::NONE,
        ));
    }
    if let Some(border) = surface.border {
        paint_border(
            painter,
            rect,
            surface.radius,
            border.widths,
            border.style,
            border.color,
        );
        paint_arrow_border(painter, arrow, border);
    }
}

fn paint_shadows_with_arrow(
    painter: &egui::Painter,
    rect: egui::Rect,
    radius: CornerRadii,
    shadows: &[Shadow],
    arrow: EguiFloatingArrowPaint,
) {
    for shadow in shadows.iter().rev().copied() {
        paint_shadow(painter, rect, radius, shadow);
        paint_arrow_shadow(painter, arrow, shadow);
    }
}

fn paint_arrow_shadow(painter: &egui::Painter, arrow: EguiFloatingArrowPaint, shadow: Shadow) {
    if shadow.color.a == 0 {
        return;
    }
    let translated = translate_arrow(arrow, shadow.offset.x, shadow.offset.y);
    if shadow.blur <= 0.0 {
        painter.add(egui::Shape::convex_polygon(
            expanded_arrow(translated, shadow.spread).points.to_vec(),
            to_egui_color(shadow.color),
            egui::Stroke::NONE,
        ));
        return;
    }

    let sigma = shadow.blur * 0.5;
    let max_blur_extent = sigma * 3.0;
    let steps = max_blur_extent.ceil().clamp(10.0, 36.0) as usize;
    for step in (0..steps).rev() {
        let outer_distance = max_blur_extent * (step + 1) as f32 / steps as f32;
        let inner_distance = max_blur_extent * step as f32 / steps as f32;
        let outer_alpha = gaussian_alpha(outer_distance, sigma);
        let inner_alpha = gaussian_alpha(inner_distance, sigma);
        let alpha_portion = (inner_alpha - outer_alpha).max(0.0);
        let alpha = (shadow.color.a as f32 * alpha_portion * 0.86)
            .round()
            .clamp(0.0, 255.0) as u8;
        if alpha == 0 {
            continue;
        }
        let color = Color {
            a: alpha,
            ..shadow.color
        };
        painter.add(egui::Shape::convex_polygon(
            expanded_arrow(translated, shadow.spread + outer_distance)
                .points
                .to_vec(),
            to_egui_color(color),
            egui::Stroke::NONE,
        ));
    }
}

fn translate_arrow(arrow: EguiFloatingArrowPaint, x: f32, y: f32) -> EguiFloatingArrowPaint {
    EguiFloatingArrowPaint {
        points: arrow.points.map(|point| point + egui::vec2(x, y)),
    }
}

fn expanded_arrow(arrow: EguiFloatingArrowPaint, amount: f32) -> EguiFloatingArrowPaint {
    let center = egui::pos2(
        (arrow.points[0].x + arrow.points[1].x + arrow.points[2].x) / 3.0,
        (arrow.points[0].y + arrow.points[1].y + arrow.points[2].y) / 3.0,
    );
    EguiFloatingArrowPaint {
        points: arrow.points.map(|point| {
            let vector = point - center;
            let length = vector.length();
            if length <= f32::EPSILON {
                point
            } else {
                center + vector * ((length + amount).max(0.0) / length)
            }
        }),
    }
}

fn paint_arrow_border(painter: &egui::Painter, arrow: EguiFloatingArrowPaint, border: BorderPaint) {
    let width = border
        .widths
        .top
        .max(border.widths.right)
        .max(border.widths.bottom)
        .max(border.widths.left);
    if width <= 0.0 {
        return;
    }
    painter.add(egui::Shape::closed_line(
        arrow.points.to_vec(),
        egui::Stroke::new(width, to_egui_color(border.color)),
    ));
}

fn paint_shadow(painter: &egui::Painter, rect: egui::Rect, radius: CornerRadii, shadow: Shadow) {
    if shadow.color.a == 0 {
        return;
    }

    let base_rect = expand_rect_safely(
        rect.translate(egui::vec2(shadow.offset.x, shadow.offset.y)),
        shadow.spread,
    );
    if shadow.blur <= 0.0 {
        painter.rect_filled(
            base_rect,
            to_egui_radius(expand_radius(radius, shadow.spread)),
            to_egui_color(shadow.color),
        );
        return;
    }

    let sigma = shadow.blur * 0.5;
    let max_blur_extent = sigma * 3.0;
    let steps = max_blur_extent.ceil().clamp(10.0, 36.0) as usize;
    for step in (0..steps).rev() {
        let outer_distance = max_blur_extent * (step + 1) as f32 / steps as f32;
        let inner_distance = max_blur_extent * step as f32 / steps as f32;
        let outer_alpha = gaussian_alpha(outer_distance, sigma);
        let inner_alpha = gaussian_alpha(inner_distance, sigma);
        let alpha_portion = (inner_alpha - outer_alpha).max(0.0);
        let alpha = (shadow.color.a as f32 * alpha_portion * 0.86)
            .round()
            .clamp(0.0, 255.0) as u8;
        if alpha == 0 {
            continue;
        }
        let color = Color {
            a: alpha,
            ..shadow.color
        };
        let expansion = shadow.spread + outer_distance;
        painter.rect_filled(
            expand_rect_safely(
                rect.translate(egui::vec2(shadow.offset.x, shadow.offset.y)),
                expansion,
            ),
            to_egui_radius(expand_radius(radius, expansion)),
            to_egui_color(color),
        );
    }
}

fn expand_radius(radius: CornerRadii, amount: f32) -> CornerRadii {
    CornerRadii {
        top_left: (radius.top_left + amount).max(0.0),
        top_right: (radius.top_right + amount).max(0.0),
        bottom_right: (radius.bottom_right + amount).max(0.0),
        bottom_left: (radius.bottom_left + amount).max(0.0),
    }
}

fn expand_rect_safely(rect: egui::Rect, amount: f32) -> egui::Rect {
    if amount >= 0.0 {
        return rect.expand(amount);
    }
    let inset = (-amount).min(rect.width() * 0.5).min(rect.height() * 0.5);
    rect.shrink(inset)
}

fn gaussian_alpha(distance: f32, sigma: f32) -> f32 {
    if sigma <= 0.0 {
        return 1.0;
    }
    (-0.5 * (distance / sigma).powi(2)).exp()
}

#[cfg(test)]
mod tests {
    use super::*;
    use des_ui_document::FloatingPlacement;
    use des_ui_render::floating_arrow_points;

    #[test]
    fn floating_arrow_points_attach_to_opposite_side_of_placement() {
        let bottom = floating_arrow_points(
            Rect::new(20.0, 30.0, 80.0, 40.0),
            FloatingPlacement::Bottom,
            30.0,
            0.0,
            12.0,
            6.0,
        );
        assert_eq!(bottom[0], Point::new(50.0, 30.0));
        assert_eq!(bottom[1], Point::new(62.0, 30.0));
        assert_eq!(bottom[2], Point::new(56.0, 24.0));

        let right = floating_arrow_points(
            Rect::new(20.0, 30.0, 80.0, 40.0),
            FloatingPlacement::Right,
            0.0,
            10.0,
            6.0,
            12.0,
        );
        assert_eq!(right[0], Point::new(20.0, 52.0));
        assert_eq!(right[1], Point::new(20.0, 40.0));
        assert_eq!(right[2], Point::new(14.0, 46.0));
    }

    #[test]
    fn expanded_arrow_keeps_center_and_moves_points_outward() {
        let arrow = EguiFloatingArrowPaint {
            points: [
                egui::pos2(0.0, 0.0),
                egui::pos2(10.0, 0.0),
                egui::pos2(5.0, -5.0),
            ],
        };

        let expanded = expanded_arrow(arrow, 2.0);

        assert!(expanded.points[0].x < arrow.points[0].x);
        assert!(expanded.points[1].x > arrow.points[1].x);
        assert!(expanded.points[2].y < arrow.points[2].y);
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
    fn distributed_dash_pattern_centers_single_dash() {
        let pattern = distributed_dash_pattern(18.0, 12.0, 8.0);

        assert_eq!(
            pattern,
            DistributedDashPattern {
                count: 1,
                dash: 12.0,
                gap: 0.0,
                leading_gap: 3.0,
            }
        );
    }
}

fn document_rect_to_egui(origin: egui::Pos2, rect: Rect) -> egui::Rect {
    egui::Rect::from_min_size(
        egui::pos2(origin.x + rect.origin.x, origin.y + rect.origin.y),
        egui::vec2(rect.size.width, rect.size.height),
    )
}

fn point_to_egui(point: Point) -> egui::Pos2 {
    egui::pos2(point.x, point.y)
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
