use des_ui_document::{
    Color, DocumentInput, Overflow, Point, PointerInput, Rect, ResolvedElement, ScrollChrome,
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
            painter.rect_filled(rect, frame.style.radius, to_egui_color(color));
        }

        if let Some(color) = frame.style.border {
            painter.rect_stroke(
                rect,
                frame.style.radius,
                egui::Stroke::new(frame.style.border_width, to_egui_color(color)),
                egui::StrokeKind::Inside,
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
                rect.left() + frame.style.border_width + frame.style.padding.left,
                rect.top() + frame.style.border_width + frame.style.padding.top,
            ),
            egui::pos2(
                rect.right() - frame.style.border_width - frame.style.padding.right,
                rect.bottom() - frame.style.border_width - frame.style.padding.bottom,
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

pub(super) fn paint_scroll_chrome(ui: &mut egui::Ui, origin: egui::Pos2, chromes: &[ScrollChrome]) {
    let painter = ui.painter();
    for chrome in chromes {
        if !chrome.visible {
            continue;
        }

        let track = document_rect_to_egui(origin, chrome.track_rect);
        let handle = document_rect_to_egui(origin, chrome.handle_rect);
        let alpha = if chrome.dragged {
            235
        } else if chrome.hovered {
            220
        } else {
            118
        };
        let track_alpha = if chrome.dragged || chrome.hovered {
            84
        } else {
            0
        };
        if track_alpha > 0 {
            painter.rect_filled(
                track,
                6.0,
                egui::Color32::from_rgba_unmultiplied(2, 8, 12, track_alpha),
            );
        }
        painter.rect_filled(
            handle,
            6.0,
            egui::Color32::from_rgba_unmultiplied(232, 236, 240, alpha),
        );
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
