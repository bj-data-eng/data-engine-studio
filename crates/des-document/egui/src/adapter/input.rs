use des_document::{DocumentInput, Point, PointerInput};
use eframe::egui;
pub fn document_input(ui: &egui::Ui, origin: egui::Pos2) -> DocumentInput {
    ui.input(|input| DocumentInput {
        pointer: input
            .pointer
            .interact_pos()
            .or_else(|| input.pointer.hover_pos())
            .map(|position| PointerInput {
                position: Point::new(position.x - origin.x, position.y - origin.y),
                primary_delta: Point::new(input.pointer.delta().x, input.pointer.delta().y),
                primary_down: input.pointer.primary_down(),
                primary_pressed: input.pointer.primary_pressed(),
                primary_clicked: input.pointer.primary_clicked(),
                primary_click_count: if input
                    .pointer
                    .button_triple_clicked(egui::PointerButton::Primary)
                {
                    3
                } else if input
                    .pointer
                    .button_double_clicked(egui::PointerButton::Primary)
                {
                    2
                } else if input.pointer.primary_clicked() {
                    1
                } else {
                    0
                },
                secondary_clicked: input.pointer.secondary_clicked(),
                time_seconds: input.time,
            }),
        scroll_delta: Point::new(input.smooth_scroll_delta.x, input.smooth_scroll_delta.y),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn document_input_subtracts_origin_and_preserves_time() {
        let ctx = egui::Context::default();
        let raw = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(200.0, 120.0),
            )),
            time: Some(7.5),
            events: vec![egui::Event::PointerMoved(egui::pos2(42.0, 65.0))],
            ..Default::default()
        };
        let mut input = None;

        let _ = ctx.run_ui(raw, |ui| {
            input = Some(document_input(ui, egui::pos2(10.0, 20.0)));
        });

        let pointer = input.unwrap().pointer.unwrap();
        assert_eq!(pointer.position, Point::new(32.0, 45.0));
        assert_eq!(pointer.time_seconds, 7.5);
        assert!(!pointer.primary_down);
    }

    #[test]
    fn document_input_maps_smooth_scroll_delta() {
        let ctx = egui::Context::default();
        let raw = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(200.0, 120.0),
            )),
            events: vec![egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: egui::vec2(5.0, -7.0),
                phase: egui::TouchPhase::Move,
                modifiers: egui::Modifiers::default(),
            }],
            ..Default::default()
        };
        let mut input = None;

        let _ = ctx.run_ui(raw, |ui| {
            input = Some(document_input(ui, egui::Pos2::ZERO));
        });

        let scroll_delta = input.unwrap().scroll_delta;
        assert!(scroll_delta.x > 0.0);
        assert!(scroll_delta.y < 0.0);
    }
}
