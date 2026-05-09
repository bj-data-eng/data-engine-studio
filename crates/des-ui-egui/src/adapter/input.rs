use des_ui_document::{DocumentInput, Point, PointerInput};
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
