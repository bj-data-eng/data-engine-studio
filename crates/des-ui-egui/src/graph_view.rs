use crate::{graph_canvas, graph_canvas::GraphCanvasState, theme};
use des_app::{AppCommand, AppSnapshot, FlowSummary, StudioAppState};
use des_core::identity;
use eframe::egui;
use egui::{Align2, Color32, FontId, Pos2, Rect, RichText, Stroke, Vec2, vec2};

const ROOT_NODE_SIZE: Vec2 = vec2(216.0, 420.0);

pub(crate) fn render(
    ui: &mut egui::Ui,
    state: &mut StudioAppState,
    graph_canvas: &mut GraphCanvasState,
) {
    let snapshot = state.snapshot();
    let rect = ui.max_rect();
    handle_viewport_input(ui, rect, graph_canvas);
    let painter = ui.painter_at(rect);

    painter.rect_filled(rect, 0.0, theme::BACKGROUND);
    paint_grid(&painter, rect, graph_canvas);
    paint_title(&painter, rect, &snapshot);
    let root_world_rect = Rect::from_min_size(Pos2::new(28.0, 78.0), ROOT_NODE_SIZE);
    let selected_flow_card = render_root_node(ui, state, graph_canvas, rect, &snapshot);
    render_expanded_flow(ui, state, graph_canvas, rect, &snapshot);
    paint_connectors(&painter, rect, &snapshot, selected_flow_card, graph_canvas);
    render_view_controls(ui, rect, graph_canvas, root_world_rect, &snapshot);
}

fn handle_viewport_input(ui: &mut egui::Ui, rect: Rect, graph_canvas: &mut GraphCanvasState) {
    let Some(pointer_pos) = ui.input(|input| input.pointer.hover_pos()) else {
        return;
    };
    if !rect.contains(pointer_pos) {
        return;
    }

    ui.input(|input| {
        let pointer_delta = input.pointer.delta();
        let pan_drag = input.pointer.middle_down()
            || input.pointer.secondary_down()
            || (input.key_down(egui::Key::Space) && input.pointer.primary_down());
        if pan_drag && pointer_delta != Vec2::ZERO {
            graph_canvas.pan(pointer_delta);
        }

        let scroll = input.smooth_scroll_delta;
        if scroll != Vec2::ZERO {
            if input.modifiers.ctrl {
                let factor = (scroll.y * 0.0015).exp();
                graph_canvas.zoom_by(factor, pointer_pos, rect);
            } else {
                graph_canvas.pan(scroll);
            }
        }

        let pinch_zoom = input.zoom_delta();
        if (pinch_zoom - 1.0).abs() > 0.001 {
            graph_canvas.zoom_by(pinch_zoom, pointer_pos, rect);
        }
    });
}

fn paint_grid(painter: &egui::Painter, rect: Rect, graph_canvas: &GraphCanvasState) {
    let spacing = 24.0;
    let minor = Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 10));
    let major = Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 18));

    let scaled_spacing = spacing * graph_canvas.zoom().max(0.35);
    let origin = graph_canvas.world_to_screen(Pos2::ZERO, rect);

    let mut x = origin.x - ((origin.x - rect.left()) / scaled_spacing).ceil() * scaled_spacing;
    let mut index = 0;
    while x <= rect.right() {
        painter.line_segment(
            [Pos2::new(x, rect.top()), Pos2::new(x, rect.bottom())],
            if index % 4 == 0 { major } else { minor },
        );
        x += scaled_spacing;
        index += 1;
    }

    let mut y = origin.y - ((origin.y - rect.top()) / scaled_spacing).ceil() * scaled_spacing;
    index = 0;
    while y <= rect.bottom() {
        painter.line_segment(
            [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
            if index % 4 == 0 { major } else { minor },
        );
        y += scaled_spacing;
        index += 1;
    }
}

fn paint_title(painter: &egui::Painter, rect: Rect, snapshot: &AppSnapshot) {
    let title_pos = rect.left_top() + vec2(24.0, 18.0);
    painter.text(
        title_pos,
        Align2::LEFT_TOP,
        identity::APP_DISPLAY_NAME,
        FontId::proportional(20.0),
        Color32::from_rgb(230, 235, 240),
    );
    painter.text(
        title_pos + vec2(0.0, 26.0),
        Align2::LEFT_TOP,
        format!("v{} / workspace graph", snapshot.home.app_info.version),
        FontId::proportional(12.0),
        theme::TEXT_MUTED,
    );
}

fn paint_connectors(
    painter: &egui::Painter,
    rect: Rect,
    snapshot: &AppSnapshot,
    selected_flow_card: Option<Rect>,
    graph_canvas: &GraphCanvasState,
) {
    let Some(flow) = selected_flow(snapshot) else {
        return;
    };
    let Some(card_rect) = selected_flow_card else {
        return;
    };
    let from = Pos2::new(card_rect.right(), card_rect.center().y);
    for target in graph_canvas::source_entry_points_with_view(rect, &flow.graph, graph_canvas) {
        draw_curve(painter, from, target, theme::SOURCE_CONNECTOR);
    }
}

fn draw_curve(painter: &egui::Painter, from: Pos2, to: Pos2, color: Color32) {
    let midpoint_x = (from.x + to.x) * 0.5;
    let points = vec![
        from,
        Pos2::new(midpoint_x, from.y),
        Pos2::new(midpoint_x, to.y),
        to,
    ];
    painter.add(egui::Shape::line(points, Stroke::new(2.0, color)));
}

fn render_root_node(
    ui: &mut egui::Ui,
    state: &mut StudioAppState,
    graph_canvas: &GraphCanvasState,
    rect: Rect,
    snapshot: &AppSnapshot,
) -> Option<Rect> {
    let node_rect = root_node_rect(rect, graph_canvas);
    egui::Area::new("workspace_root_node".into())
        .fixed_pos(node_rect.min)
        .show(ui.ctx(), |ui| {
            egui::Frame::new()
                .fill(theme::PANEL)
                .stroke(Stroke::new(1.0, theme::STROKE))
                .corner_radius(8.0)
                .inner_margin(12.0)
                .show(ui, |ui| {
                    ui.set_width(node_rect.width());
                    ui.set_min_width(node_rect.width());
                    ui.set_max_width(node_rect.width());
                    ui.set_min_size(node_rect.size());
                    ui.heading("Workspace Roots");
                    ui.label(
                        RichText::new("Catalog node")
                            .small()
                            .color(theme::TEXT_MUTED),
                    );
                    ui.add_space(8.0);
                    render_root_selector(ui, state, snapshot);
                    ui.add_space(10.0);
                    render_workspace_cards(ui, state, snapshot);
                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);
                    render_flow_cards(ui, state, snapshot)
                })
                .inner
        })
        .inner
}

fn render_root_selector(ui: &mut egui::Ui, state: &mut StudioAppState, snapshot: &AppSnapshot) {
    let selected_root = snapshot
        .home
        .workspace_roots
        .iter()
        .find(|root| Some(root.id.as_str()) == snapshot.selected_root_id.as_deref());
    let selected_label = selected_root
        .map(|root| root.name.as_str())
        .unwrap_or("No root selected");

    egui::ComboBox::from_id_salt("workspace_root_selector")
        .selected_text(selected_label)
        .width(ROOT_NODE_SIZE.x)
        .show_ui(ui, |ui| {
            for root in &snapshot.home.workspace_roots {
                let selected = Some(root.id.as_str()) == snapshot.selected_root_id.as_deref();
                if ui.selectable_label(selected, &root.name).clicked() {
                    state.dispatch(AppCommand::SelectWorkspaceRoot {
                        root_id: root.id.clone(),
                    });
                }
            }
        });

    if let Some(root) = selected_root {
        ui.label(RichText::new(&root.path).small().color(theme::TEXT_MUTED));
    }
}

fn render_workspace_cards(ui: &mut egui::Ui, state: &mut StudioAppState, snapshot: &AppSnapshot) {
    ui.label(RichText::new("Workspaces").strong());
    let workspaces: Vec<_> = snapshot
        .home
        .workspaces
        .iter()
        .filter(|workspace| {
            Some(workspace.root_id.as_str()) == snapshot.selected_root_id.as_deref()
        })
        .collect();

    for workspace in workspaces {
        let selected = Some(workspace.id.as_str()) == snapshot.selected_workspace_id.as_deref();
        let response = card_frame(selected).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.add(
                egui::Label::new(RichText::new(&workspace.name).strong())
                    .wrap()
                    .selectable(false),
            );
            ui.add(
                egui::Label::new(
                    RichText::new(&workspace.status)
                        .small()
                        .color(theme::TEXT_MUTED),
                )
                .wrap()
                .selectable(false),
            );
        });
        if response.response.interact(egui::Sense::click()).clicked() {
            state.dispatch(AppCommand::SelectWorkspace {
                workspace_id: workspace.id.clone(),
            });
        }
        ui.add_space(6.0);
    }
}

fn render_flow_cards(
    ui: &mut egui::Ui,
    state: &mut StudioAppState,
    snapshot: &AppSnapshot,
) -> Option<Rect> {
    ui.label(RichText::new("Grouped Flows").strong());
    let selected_id = snapshot.selected_flow_id.as_deref();
    let mut selected_rect = None;
    for flow in &snapshot.home.flows {
        let selected = Some(flow.id.as_str()) == selected_id;
        let response = card_frame(selected).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.add(
                egui::Label::new(RichText::new(&flow.name).strong())
                    .wrap()
                    .selectable(false),
            );
            ui.add(
                egui::Label::new(RichText::new(&flow.group).small().color(theme::TEXT_MUTED))
                    .wrap()
                    .selectable(false),
            );
            ui.horizontal(|ui| {
                ui.label(RichText::new(format!("{} nodes", flow.node_count)).small());
                ui.separator();
                ui.label(RichText::new(&flow.trigger).small());
            });
        });
        if response.response.interact(egui::Sense::click()).clicked() {
            state.dispatch(AppCommand::SelectFlow {
                flow_id: flow.id.clone(),
            });
        }
        if selected {
            selected_rect = Some(response.response.rect);
        }
        ui.add_space(6.0);
    }
    selected_rect
}

fn render_expanded_flow(
    ui: &mut egui::Ui,
    state: &mut StudioAppState,
    graph_canvas_state: &mut GraphCanvasState,
    rect: Rect,
    snapshot: &AppSnapshot,
) {
    let Some(flow) = selected_flow(snapshot) else {
        return;
    };

    graph_canvas::render(ui, state, graph_canvas_state, rect, &flow.graph);
}

fn render_view_controls(
    ui: &mut egui::Ui,
    rect: Rect,
    graph_canvas: &mut GraphCanvasState,
    root_world_rect: Rect,
    snapshot: &AppSnapshot,
) {
    egui::Area::new("graph_view_controls".into())
        .fixed_pos(rect.right_top() + vec2(-168.0, 18.0))
        .show(ui.ctx(), |ui| {
            egui::Frame::new()
                .fill(Color32::from_rgb(24, 28, 32))
                .stroke(Stroke::new(1.0, theme::STROKE))
                .corner_radius(6.0)
                .inner_margin(6.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("-").clicked() {
                            graph_canvas.zoom_by(0.9, rect.center(), rect);
                        }
                        ui.label(format!("{:.0}%", graph_canvas.zoom() * 100.0));
                        if ui.button("+").clicked() {
                            graph_canvas.zoom_by(1.1, rect.center(), rect);
                        }
                        if ui.button("Fit").clicked() {
                            if let Some(bounds) = fit_world_bounds(root_world_rect, snapshot) {
                                graph_canvas.fit_world_rect(bounds, rect, 48.0);
                            }
                        }
                    });
                });
        });
}

fn fit_world_bounds(root_world_rect: Rect, snapshot: &AppSnapshot) -> Option<Rect> {
    let mut bounds = root_world_rect;
    if let Some(flow) = selected_flow(snapshot) {
        if let Some(graph_bounds) = graph_canvas::graph_world_bounds(&flow.graph) {
            bounds = bounds.union(graph_bounds);
        }
    }
    Some(bounds)
}

fn card_frame(selected: bool) -> egui::Frame {
    egui::Frame::new()
        .fill(if selected {
            theme::PANEL_SELECTED
        } else {
            Color32::from_rgb(32, 37, 42)
        })
        .stroke(Stroke::new(
            1.0,
            if selected {
                theme::STROKE_SELECTED
            } else {
                theme::STROKE
            },
        ))
        .corner_radius(6.0)
        .inner_margin(8.0)
}

fn selected_flow(snapshot: &AppSnapshot) -> Option<&FlowSummary> {
    snapshot
        .home
        .flows
        .iter()
        .find(|flow| Some(flow.id.as_str()) == snapshot.selected_flow_id.as_deref())
}

fn root_node_rect(rect: Rect, graph_canvas: &GraphCanvasState) -> Rect {
    graph_canvas.world_rect(Pos2::new(28.0, 78.0), ROOT_NODE_SIZE, rect)
}
