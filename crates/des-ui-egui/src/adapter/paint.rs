use super::text::{layout_job, paint_document_text_selection};
use des_ui_document::{
    Color, CornerRadii, DocumentTextSelection, Glyph, Insets, Overflow, Rect, ResolvedElement,
    ScrollChrome, Shadow, TextLayoutRequest, TextWrapMode,
};
use eframe::egui;
pub fn paint_frame(
    ui: &mut egui::Ui,
    origin: egui::Pos2,
    frame: &ResolvedElement,
    text_selection: Option<&DocumentTextSelection>,
) {
    paint_frame_clipped(ui, origin, frame, ui.clip_rect(), text_selection);
}

fn paint_frame_clipped(
    ui: &mut egui::Ui,
    origin: egui::Pos2,
    frame: &ResolvedElement,
    clip_rect: egui::Rect,
    text_selection: Option<&DocumentTextSelection>,
) {
    let painter = ui.painter().with_clip_rect(clip_rect);
    if frame.id.as_str() != "root" {
        let rect = frame_rect(origin, frame);

        paint_shadows(&painter, rect, frame.style.radius, &frame.style.shadows);

        if let Some(color) = frame.style.background {
            painter.rect_filled(
                rect,
                to_egui_radius(frame.style.radius),
                to_egui_color(color),
            );
        }

        if let Some(color) = frame.style.border {
            paint_border(
                &painter,
                rect,
                frame.style.radius,
                frame.style.border_width,
                color,
            );
        }

        if let Some(text) = &frame.text {
            let text_rect = frame_content_rect(rect, frame);
            let request = TextLayoutRequest {
                text,
                font_size: frame.style.font_size,
                wrap_width: match frame.style.text_wrap {
                    TextWrapMode::Extend => f32::INFINITY,
                    TextWrapMode::Wrap | TextWrapMode::Truncate => text_rect.width(),
                },
                wrap_mode: frame.style.text_wrap,
                max_lines: frame.style.max_lines,
                line_height: frame.style.line_height,
            };
            let color = to_egui_color(frame.style.text_color);
            let mut galley = painter.layout_job(layout_job(request, color));
            if frame.selectable_text
                && let Some(selection) = text_selection
                && selection.target == frame.id
            {
                let cursor_range = egui::text_selection::CCursorRange::two(
                    egui::text::CCursor::new(selection.anchor_index),
                    egui::text::CCursor::new(selection.focus_index),
                );
                paint_document_text_selection(
                    &mut galley,
                    &cursor_range,
                    frame.style.text_selection_background,
                    frame.style.text_selection_color,
                );
            }
            painter.galley(text_rect.min, galley, color);
        }

        if let Some(glyph) = frame.glyph {
            paint_glyph(
                &painter,
                rect,
                glyph,
                frame.style.text_color,
                frame.style.font_size,
            );
        }
    }

    let mut children: Vec<_> = frame.children.iter().collect();
    children.sort_by_key(|child| child.style.z_index);
    let next_clip = if frame.style.overflow_x == Overflow::Scroll
        || frame.style.overflow_y == Overflow::Scroll
    {
        let rect = frame_rect(origin, frame);
        let content_rect = frame_content_rect(rect, frame);
        let min = egui::pos2(
            if frame.style.overflow_x == Overflow::Scroll {
                content_rect.left()
            } else {
                clip_rect.left()
            },
            if frame.style.overflow_y == Overflow::Scroll {
                content_rect.top()
            } else {
                clip_rect.top()
            },
        );
        let max = egui::pos2(
            if frame.style.overflow_x == Overflow::Scroll {
                content_rect.right()
            } else {
                clip_rect.right()
            },
            if frame.style.overflow_y == Overflow::Scroll {
                content_rect.bottom()
            } else {
                clip_rect.bottom()
            },
        );
        clip_rect.intersect(egui::Rect::from_min_max(min, max))
    } else {
        clip_rect
    };
    for child in children {
        paint_frame_clipped(ui, origin, child, next_clip, text_selection);
    }
}

fn frame_rect(origin: egui::Pos2, frame: &ResolvedElement) -> egui::Rect {
    document_rect_to_egui(origin, frame.rect)
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
        paint_border(painter, rect, radius, border_width, border);
    }
}

fn frame_content_rect(rect: egui::Rect, frame: &ResolvedElement) -> egui::Rect {
    let min = egui::pos2(
        rect.left() + frame.style.border_width.left + frame.style.padding.left,
        rect.top() + frame.style.border_width.top + frame.style.padding.top,
    );
    let max = egui::pos2(
        (rect.right() - frame.style.border_width.right - frame.style.padding.right).max(min.x),
        (rect.bottom() - frame.style.border_width.bottom - frame.style.padding.bottom).max(min.y),
    );
    egui::Rect::from_min_max(min, max)
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
    color: Color,
) {
    let color = to_egui_color(color);
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

pub fn paint_scroll_chrome(ui: &mut egui::Ui, origin: egui::Pos2, chromes: &[ScrollChrome]) {
    let painter = ui.painter();
    for chrome in chromes {
        if !chrome.visible {
            continue;
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
}

fn document_rect_to_egui(origin: egui::Pos2, rect: Rect) -> egui::Rect {
    egui::Rect::from_min_size(
        egui::pos2(origin.x + rect.origin.x, origin.y + rect.origin.y),
        egui::vec2(rect.size.width, rect.size.height),
    )
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
