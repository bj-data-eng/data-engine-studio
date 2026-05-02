use des_ui_document::{
    Color, CornerRadii, DocumentInput, Insets, Overflow, Point, PointerInput, Rect,
    ResolvedElement, ScrollChrome,
};
use eframe::egui;

pub(super) fn document_input(ui: &egui::Ui, origin: egui::Pos2) -> DocumentInput {
    ui.input(|input| DocumentInput {
        pointer: input.pointer.hover_pos().map(|position| PointerInput {
            position: Point::new(position.x - origin.x, position.y - origin.y),
            primary_delta: Point::new(input.pointer.delta().x, input.pointer.delta().y),
            primary_down: input.pointer.primary_down(),
            primary_clicked: input.pointer.primary_clicked(),
        }),
        scroll_delta: Point::new(input.smooth_scroll_delta.x, input.smooth_scroll_delta.y),
    })
}

pub(super) fn paint_frame(ui: &mut egui::Ui, origin: egui::Pos2, frame: &ResolvedElement) {
    paint_frame_clipped(ui, origin, frame, ui.clip_rect());
}

fn paint_frame_clipped(
    ui: &mut egui::Ui,
    origin: egui::Pos2,
    frame: &ResolvedElement,
    clip_rect: egui::Rect,
) {
    let painter = ui.painter().with_clip_rect(clip_rect);
    if frame.id.as_str() != "root" {
        let rect = egui::Rect::from_min_size(
            egui::pos2(
                origin.x + frame.rect.origin.x,
                origin.y + frame.rect.origin.y,
            ),
            egui::vec2(frame.rect.size.width, frame.rect.size.height),
        );

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
            painter.text(
                rect.min,
                egui::Align2::LEFT_TOP,
                text,
                egui::FontId::proportional(frame.style.font_size),
                to_egui_color(frame.style.text_color),
            );
        }
    }

    let mut children: Vec<_> = frame.children.iter().collect();
    children.sort_by_key(|child| child.style.z_index);
    let next_clip = if frame.style.overflow_y == Overflow::Scroll {
        let rect = egui::Rect::from_min_size(
            egui::pos2(
                origin.x + frame.rect.origin.x,
                origin.y + frame.rect.origin.y,
            ),
            egui::vec2(frame.rect.size.width, frame.rect.size.height),
        );
        let content_rect = egui::Rect::from_min_max(
            egui::pos2(
                rect.left() + frame.style.border_width.left + frame.style.padding.left,
                rect.top() + frame.style.border_width.top + frame.style.padding.top,
            ),
            egui::pos2(
                rect.right() - frame.style.border_width.right - frame.style.padding.right,
                rect.bottom() - frame.style.border_width.bottom - frame.style.padding.bottom,
            ),
        );
        clip_rect.intersect(content_rect)
    } else {
        clip_rect
    };
    for child in children {
        paint_frame_clipped(ui, origin, child, next_clip);
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

pub(super) fn paint_scroll_chrome(ui: &mut egui::Ui, origin: egui::Pos2, chromes: &[ScrollChrome]) {
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
