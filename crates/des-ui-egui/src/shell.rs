use crate::{debug_overlay, graph_canvas::GraphCanvasState, graph_view, theme};
use des_app::StudioAppState;
use eframe::egui;

pub(crate) fn render(
    ui: &mut egui::Ui,
    state: &mut StudioAppState,
    graph_canvas: &mut GraphCanvasState,
    debug_overlay: bool,
) {
    let full_rect = ui.max_rect();
    let menu_height = 30.0;
    let menu_rect =
        egui::Rect::from_min_size(full_rect.min, egui::vec2(full_rect.width(), menu_height));
    let graph_rect = egui::Rect::from_min_max(
        egui::pos2(full_rect.left(), full_rect.top() + menu_height),
        full_rect.right_bottom(),
    );

    ui.scope_builder(egui::UiBuilder::new().max_rect(menu_rect), |ui| {
        egui::Frame::new()
            .fill(theme::MENU_BAR)
            .inner_margin(egui::Margin::symmetric(8, 2))
            .show(ui, |ui| {
                ui.set_min_size(menu_rect.size());
                ui.set_max_size(menu_rect.size());
                ui.centered_and_justified(|ui| {
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        render_menu_bar(ui);
                    });
                });
            });
    });

    ui.scope_builder(egui::UiBuilder::new().max_rect(graph_rect), |ui| {
        ui.set_min_size(graph_rect.size());
        ui.set_max_size(graph_rect.size());
        graph_view::render(ui, state, graph_canvas);
    });

    if debug_overlay {
        debug_overlay::render(ui, graph_rect, graph_canvas, &state.snapshot());
    }
}

fn render_menu_bar(ui: &mut egui::Ui) {
    egui::MenuBar::new().ui(ui, |ui| {
        ui.menu_button("File", |ui| {
            menu_item(ui, "New Flow");
            menu_item(ui, "Open Workspace");
            menu_item(ui, "Save");
            ui.separator();
            if ui.button("Exit").clicked() {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            }
        });

        ui.menu_button("Edit", |ui| {
            menu_item(ui, "Undo");
            menu_item(ui, "Redo");
            ui.separator();
            menu_item(ui, "Cut");
            menu_item(ui, "Copy");
            menu_item(ui, "Paste");
        });

        ui.menu_button("View", |ui| {
            menu_item(ui, "Zoom In");
            menu_item(ui, "Zoom Out");
            menu_item(ui, "Fit Graph");
            ui.separator();
            menu_item(ui, "Inspector");
            menu_item(ui, "Runtime Panel");
        });

        ui.menu_button("Window", |ui| {
            menu_item(ui, "New Window");
            menu_item(ui, "Keep Process in Background");
            menu_item(ui, "Restore Layout");
        });

        ui.menu_button("Help", |ui| {
            menu_item(ui, "Documentation");
            menu_item(ui, "About Data Engine Studio");
        });
    });
}

fn menu_item(ui: &mut egui::Ui, label: &str) {
    if ui.button(label).clicked() {
        ui.close();
    }
}
