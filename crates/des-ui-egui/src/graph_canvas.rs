use crate::theme;
use des_app::{
    AppCommand, FlowGraphSummary, GraphEdgeSummary, GraphNodeSummary, GraphPortSide, StudioAppState,
};
use eframe::egui;
use egui::{Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, Vec2, vec2};

const NODE_SIZE: Vec2 = vec2(230.0, 112.0);
const PORT_RADIUS: f32 = 6.0;
const PORT_TOP_OFFSET: f32 = 46.0;
const PORT_SPACING: f32 = 24.0;

pub(crate) struct GraphCanvasState {
    dragging_edge: Option<PortRef>,
    pan: Vec2,
    zoom: f32,
}

impl Default for GraphCanvasState {
    fn default() -> Self {
        Self {
            dragging_edge: None,
            pan: Vec2::ZERO,
            zoom: 1.0,
        }
    }
}

impl GraphCanvasState {
    pub(crate) fn zoom(&self) -> f32 {
        self.zoom
    }

    pub(crate) fn pan(&mut self, delta: Vec2) {
        self.pan += delta;
    }

    pub(crate) fn zoom_by(&mut self, factor: f32, anchor: Pos2, rect: Rect) {
        let old_zoom = self.zoom;
        let next_zoom = (self.zoom * factor).clamp(0.35, 2.25);
        if (next_zoom - old_zoom).abs() < f32::EPSILON {
            return;
        }
        let world_anchor = self.screen_to_world(anchor, rect);
        self.zoom = next_zoom;
        let next_anchor = self.world_to_screen(world_anchor, rect);
        self.pan += anchor - next_anchor;
    }

    pub(crate) fn world_to_screen(&self, world: Pos2, rect: Rect) -> Pos2 {
        rect.left_top() + self.pan + world.to_vec2() * self.zoom
    }

    pub(crate) fn screen_to_world(&self, screen: Pos2, rect: Rect) -> Pos2 {
        let local = (screen - rect.left_top() - self.pan) / self.zoom;
        Pos2::new(local.x, local.y)
    }

    pub(crate) fn world_rect(&self, min: Pos2, size: Vec2, rect: Rect) -> Rect {
        Rect::from_min_size(self.world_to_screen(min, rect), size * self.zoom)
    }

    pub(crate) fn fit_world_rect(&mut self, world_rect: Rect, screen_rect: Rect, padding: f32) {
        if world_rect.width() <= 0.0 || world_rect.height() <= 0.0 {
            return;
        }
        let available = (screen_rect.size() - Vec2::splat(padding * 2.0)).max(Vec2::splat(1.0));
        let next_zoom = (available.x / world_rect.width())
            .min(available.y / world_rect.height())
            .clamp(0.35, 2.25);
        self.zoom = next_zoom;
        let content_size = world_rect.size() * self.zoom;
        let target_min = screen_rect.min + (screen_rect.size() - content_size) * 0.5;
        self.pan = target_min - screen_rect.min - world_rect.min.to_vec2() * self.zoom;
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PortRef {
    node_id: String,
    port_id: String,
    side: GraphPortSide,
}

pub(crate) fn render(
    ui: &mut egui::Ui,
    state: &mut StudioAppState,
    canvas_state: &mut GraphCanvasState,
    rect: Rect,
    graph: &FlowGraphSummary,
) {
    let painter = ui.painter_at(rect);
    paint_edges(&painter, rect, graph, canvas_state);

    if !ui.input(|input| input.pointer.primary_down()) {
        canvas_state.dragging_edge = None;
    }

    if let Some(from_port) = &canvas_state.dragging_edge {
        if let Some(from_pos) = port_position(
            rect,
            graph,
            &from_port.node_id,
            &from_port.port_id,
            from_port.side,
            canvas_state,
        ) {
            let pointer_pos = ui
                .input(|input| input.pointer.hover_pos())
                .unwrap_or(from_pos);
            paint_curve(
                &painter,
                from_pos,
                pointer_pos,
                Color32::from_rgb(235, 185, 82),
                2.0,
            );
        }
    }

    for node in &graph.nodes {
        render_node(ui, state, canvas_state, rect, node);
    }
}

fn render_node(
    ui: &mut egui::Ui,
    state: &mut StudioAppState,
    canvas_state: &mut GraphCanvasState,
    canvas_rect: Rect,
    node: &GraphNodeSummary,
) {
    let rect = node_rect(canvas_rect, node, canvas_state);
    let painter = ui.painter_at(canvas_rect);
    paint_node_body(ui, &painter, state, canvas_state, rect, node);
    paint_ports(ui, &painter, canvas_state, canvas_rect, node);
}

pub(crate) fn source_entry_points_with_view(
    canvas_rect: Rect,
    graph: &FlowGraphSummary,
    canvas_state: &GraphCanvasState,
) -> Vec<Pos2> {
    graph
        .nodes
        .iter()
        .filter(|node| node.inputs.is_empty() && !node.outputs.is_empty())
        .map(|node| {
            let rect = node_rect(canvas_rect, node, canvas_state);
            Pos2::new(rect.left(), rect.center().y)
        })
        .collect()
}

pub(crate) fn graph_world_bounds(graph: &FlowGraphSummary) -> Option<Rect> {
    let mut bounds: Option<Rect> = None;
    for node in &graph.nodes {
        let rect = Rect::from_min_size(Pos2::new(node.position.x, node.position.y), NODE_SIZE);
        bounds = Some(match bounds {
            Some(existing) => existing.union(rect),
            None => rect,
        });
    }
    bounds
}

fn paint_node_body(
    ui: &mut egui::Ui,
    painter: &egui::Painter,
    state: &mut StudioAppState,
    canvas_state: &GraphCanvasState,
    rect: Rect,
    node: &GraphNodeSummary,
) {
    let response = ui.interact(
        rect,
        egui::Id::new(("graph-node", &node.id)),
        Sense::click_and_drag(),
    );
    if response.dragged() {
        let delta = ui.input(|input| input.pointer.delta());
        if delta != Vec2::ZERO {
            state.dispatch(AppCommand::MoveGraphNodeBy {
                node_id: node.id.clone(),
                dx: delta.x / canvas_state.zoom(),
                dy: delta.y / canvas_state.zoom(),
            });
        }
    }

    let scale = canvas_state.zoom().clamp(0.75, 1.35);
    let fill = if response.hovered() || response.dragged() {
        Color32::from_rgb(36, 44, 50)
    } else {
        Color32::from_rgb(29, 35, 39)
    };
    painter.rect_filled(rect, 8.0, fill);
    painter.rect_stroke(
        rect,
        8.0,
        Stroke::new(1.0, theme::STROKE),
        egui::StrokeKind::Inside,
    );

    let text_left = rect.left() + 16.0 * scale;
    painter.text(
        Pos2::new(text_left, rect.top() + 14.0 * scale),
        Align2::LEFT_TOP,
        &node.subtitle,
        FontId::proportional(12.0 * scale),
        theme::TEXT_MUTED,
    );
    painter.text(
        Pos2::new(text_left, rect.top() + 34.0 * scale),
        Align2::LEFT_TOP,
        &node.title,
        FontId::proportional(18.0 * scale),
        Color32::from_rgb(222, 228, 235),
    );

    for (index, port) in node.inputs.iter().enumerate() {
        painter.text(
            Pos2::new(
                rect.left() + 16.0 * scale,
                rect.top() + (PORT_TOP_OFFSET + index as f32 * PORT_SPACING - 8.0) * scale,
            ),
            Align2::LEFT_TOP,
            format!("< {}", port.label),
            FontId::proportional(11.0 * scale),
            theme::TEXT_MUTED,
        );
    }
    for (index, port) in node.outputs.iter().enumerate() {
        painter.text(
            Pos2::new(
                rect.right() - 16.0 * scale,
                rect.top() + (PORT_TOP_OFFSET + index as f32 * PORT_SPACING - 8.0) * scale,
            ),
            Align2::RIGHT_TOP,
            format!("{} >", port.label),
            FontId::proportional(11.0 * scale),
            theme::TEXT_MUTED,
        );
    }
}

fn paint_ports(
    ui: &mut egui::Ui,
    painter: &egui::Painter,
    canvas_state: &mut GraphCanvasState,
    canvas_rect: Rect,
    node: &GraphNodeSummary,
) {
    for port in node.inputs.iter().chain(node.outputs.iter()) {
        let center = port_position(
            canvas_rect,
            &FlowGraphSummary {
                nodes: vec![node.clone()],
                edges: Vec::new(),
            },
            &node.id,
            &port.id,
            port.side,
            canvas_state,
        )
        .unwrap_or_else(|| node_rect(canvas_rect, node, canvas_state).center());
        let color = match port.side {
            GraphPortSide::Input => Color32::from_rgb(95, 204, 140),
            GraphPortSide::Output => Color32::from_rgb(94, 162, 230),
        };
        let radius = PORT_RADIUS * canvas_state.zoom().clamp(0.75, 1.35);
        painter.circle_filled(center, radius, color);
        painter.circle_stroke(
            center,
            radius,
            Stroke::new(1.0, Color32::from_rgb(220, 230, 240)),
        );

        let hit_rect = Rect::from_center_size(center, vec2(22.0, 22.0));
        let response = ui.interact(
            hit_rect,
            egui::Id::new(("graph-port", &node.id, &port.id, format!("{:?}", port.side))),
            Sense::drag(),
        );
        if response.drag_started() {
            canvas_state.dragging_edge = Some(PortRef {
                node_id: node.id.clone(),
                port_id: port.id.clone(),
                side: port.side,
            });
        }
    }
}

fn paint_edges(
    painter: &egui::Painter,
    canvas_rect: Rect,
    graph: &FlowGraphSummary,
    canvas_state: &GraphCanvasState,
) {
    for edge in &graph.edges {
        paint_edge(painter, canvas_rect, graph, edge, canvas_state);
    }
}

fn paint_edge(
    painter: &egui::Painter,
    canvas_rect: Rect,
    graph: &FlowGraphSummary,
    edge: &GraphEdgeSummary,
    canvas_state: &GraphCanvasState,
) {
    let Some(from) = port_position(
        canvas_rect,
        graph,
        &edge.from_node_id,
        &edge.from_port_id,
        GraphPortSide::Output,
        canvas_state,
    ) else {
        return;
    };
    let Some(to) = port_position(
        canvas_rect,
        graph,
        &edge.to_node_id,
        &edge.to_port_id,
        GraphPortSide::Input,
        canvas_state,
    ) else {
        return;
    };
    paint_curve(painter, from, to, theme::CONNECTOR, 2.25);
}

fn paint_curve(painter: &egui::Painter, from: Pos2, to: Pos2, color: Color32, width: f32) {
    let distance = (to.x - from.x).abs().max(80.0);
    let control_offset = distance * 0.45;
    let points = cubic_points(
        from,
        Pos2::new(from.x + control_offset, from.y),
        Pos2::new(to.x - control_offset, to.y),
        to,
    );
    painter.add(egui::Shape::line(points, Stroke::new(width, color)));
}

fn cubic_points(p0: Pos2, p1: Pos2, p2: Pos2, p3: Pos2) -> Vec<Pos2> {
    let mut points = Vec::with_capacity(24);
    for index in 0..24 {
        let t = index as f32 / 23.0;
        let nt = 1.0 - t;
        let x = nt.powi(3) * p0.x
            + 3.0 * nt.powi(2) * t * p1.x
            + 3.0 * nt * t.powi(2) * p2.x
            + t.powi(3) * p3.x;
        let y = nt.powi(3) * p0.y
            + 3.0 * nt.powi(2) * t * p1.y
            + 3.0 * nt * t.powi(2) * p2.y
            + t.powi(3) * p3.y;
        points.push(Pos2::new(x, y));
    }
    points
}

fn port_position(
    canvas_rect: Rect,
    graph: &FlowGraphSummary,
    node_id: &str,
    port_id: &str,
    side: GraphPortSide,
    canvas_state: &GraphCanvasState,
) -> Option<Pos2> {
    let node = graph.nodes.iter().find(|node| node.id == node_id)?;
    let rect = node_rect(canvas_rect, node, canvas_state);
    let ports = match side {
        GraphPortSide::Input => &node.inputs,
        GraphPortSide::Output => &node.outputs,
    };
    let index = ports.iter().position(|port| port.id == port_id)?;
    let y = rect.top() + (PORT_TOP_OFFSET + index as f32 * PORT_SPACING) * canvas_state.zoom();
    let x = match side {
        GraphPortSide::Input => rect.left(),
        GraphPortSide::Output => rect.right(),
    };
    Some(Pos2::new(x, y))
}

fn node_rect(canvas_rect: Rect, node: &GraphNodeSummary, canvas_state: &GraphCanvasState) -> Rect {
    canvas_state.world_rect(
        Pos2::new(node.position.x, node.position.y),
        NODE_SIZE,
        canvas_rect,
    )
}
