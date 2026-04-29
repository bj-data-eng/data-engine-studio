use crate::theme;
use des_app::{
    AppCommand, AppSnapshot, CanvasPoint, FlowGraphSummary, GraphNodeSummary, StudioAppState,
};
use des_graph_egui::{Graph, NodeId, View, edge::Edge, node::Node};
use eframe::egui;
use egui::{Color32, Pos2, Rect, RichText, Stroke, Vec2, vec2};
use std::collections::HashSet;

const ROOT_NODE_ID: NodeId = NodeId::from_u64(1);
const ROOT_NODE_SIZE: Vec2 = vec2(216.0, 420.0);
const NODE_SIZE: Vec2 = vec2(230.0, 112.0);
const INITIAL_SCENE_SIZE: Vec2 = vec2(1320.0, 780.0);

pub(crate) struct GraphCanvasState {
    view: View,
    selected_edges: HashSet<String>,
}

pub(crate) struct GraphCanvasDiagnostics {
    pub(crate) zoom: f32,
    pub(crate) scene_rect: Rect,
    pub(crate) node_count: usize,
    pub(crate) selected_edge_count: usize,
}

impl Default for GraphCanvasState {
    fn default() -> Self {
        Self {
            view: View {
                scene_rect: Rect::from_min_size(Pos2::ZERO, INITIAL_SCENE_SIZE),
                layout: Default::default(),
            },
            selected_edges: HashSet::new(),
        }
    }
}

impl GraphCanvasState {
    pub(crate) fn set_scene_rect(&mut self, rect: Rect) {
        if rect.is_finite() && rect.width() > 0.0 && rect.height() > 0.0 {
            self.view.scene_rect = rect;
        }
    }

    pub(crate) fn view_zoom(&self, screen_rect: Rect) -> f32 {
        if self.view.scene_rect.width() <= 0.0 || self.view.scene_rect.height() <= 0.0 {
            return 1.0;
        }
        (screen_rect.width() / self.view.scene_rect.width())
            .min(screen_rect.height() / self.view.scene_rect.height())
            .clamp(0.35, 2.25)
    }

    pub(crate) fn zoom_by(&mut self, factor: f32, anchor: Pos2, screen_rect: Rect) {
        let old_zoom = self.view_zoom(screen_rect);
        let next_zoom = (old_zoom * factor).clamp(0.35, 2.25);
        if (next_zoom - old_zoom).abs() < f32::EPSILON || screen_rect.size() == Vec2::ZERO {
            return;
        }
        let anchor_offset = anchor - screen_rect.min;
        let world_anchor = self.view.scene_rect.min + anchor_offset / old_zoom;
        let next_size = screen_rect.size() / next_zoom;
        let next_min = world_anchor - anchor_offset / next_zoom;
        self.view.scene_rect = Rect::from_min_size(next_min, next_size);
    }

    pub(crate) fn fit_world_rect(&mut self, world_rect: Rect, screen_rect: Rect, padding_px: f32) {
        if world_rect.width() <= 0.0 || world_rect.height() <= 0.0 {
            return;
        }
        let available = (screen_rect.size() - Vec2::splat(padding_px * 2.0)).max(Vec2::splat(1.0));
        let next_zoom = (available.x / world_rect.width())
            .min(available.y / world_rect.height())
            .clamp(0.35, 2.25);
        let next_size = screen_rect.size() / next_zoom;
        self.view.scene_rect = Rect::from_center_size(world_rect.center(), next_size);
    }

    pub(crate) fn diagnostics(
        &self,
        screen_rect: Rect,
        snapshot: &AppSnapshot,
    ) -> GraphCanvasDiagnostics {
        let node_count = selected_flow(snapshot)
            .map(|flow| flow.graph.nodes.len())
            .unwrap_or_default()
            + 1;

        GraphCanvasDiagnostics {
            zoom: self.view_zoom(screen_rect),
            scene_rect: self.view.scene_rect,
            node_count,
            selected_edge_count: self.selected_edges.len(),
        }
    }
}

pub(crate) fn render(
    ui: &mut egui::Ui,
    state: &mut StudioAppState,
    canvas_state: &mut GraphCanvasState,
    rect: Rect,
    snapshot: &AppSnapshot,
) {
    ensure_layout_positions(canvas_state, snapshot);
    let mut selected_edges = std::mem::take(&mut canvas_state.selected_edges);
    let content_scale = 1.0;

    ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
        ui.set_min_size(rect.size());
        let response = Graph::new("studio-workspace-graph")
            .zoom_range(egui::Rangef::new(0.35, 2.25))
            .max_inner_size(vec2(5000.0, 3000.0))
            .show(&mut canvas_state.view, ui, |ui, show| {
                show.nodes(ui, |nodes, ui| {
                    render_root_node(ui, state, snapshot, nodes, content_scale);
                    if let Some(flow) = selected_flow(snapshot) {
                        for node in &flow.graph.nodes {
                            render_flow_node(ui, nodes, node, content_scale);
                        }
                    }
                })
                .edges(ui, |edges, ui| {
                    if let Some(flow) = selected_flow(snapshot) {
                        render_root_edges(ui, &mut selected_edges, edges, &flow.graph);
                        render_flow_edges(ui, &mut selected_edges, edges, &flow.graph);
                    }
                });
            });

        if response.response.hovered() {
            ui.ctx().request_repaint();
        }
    });

    canvas_state.selected_edges = selected_edges;
    sync_node_positions(state, canvas_state, snapshot);
}

pub(crate) fn graph_world_bounds(snapshot: &AppSnapshot) -> Option<Rect> {
    let root = root_node_world_rect();
    let mut bounds = root;
    if let Some(flow) = selected_flow(snapshot)
        && let Some(graph_bounds) = flow_graph_world_bounds(&flow.graph)
    {
        bounds = bounds.union(graph_bounds);
    }
    Some(bounds)
}

fn ensure_layout_positions(canvas_state: &mut GraphCanvasState, snapshot: &AppSnapshot) {
    canvas_state
        .view
        .layout
        .entry(ROOT_NODE_ID)
        .or_insert(root_node_world_rect().min);

    if let Some(flow) = selected_flow(snapshot) {
        for node in &flow.graph.nodes {
            canvas_state
                .view
                .layout
                .entry(node_id(&node.id))
                .or_insert(Pos2::new(node.position.x, node.position.y));
        }
    }
}

fn sync_node_positions(
    state: &mut StudioAppState,
    canvas_state: &GraphCanvasState,
    snapshot: &AppSnapshot,
) {
    let Some(flow) = selected_flow(snapshot) else {
        return;
    };

    for node in &flow.graph.nodes {
        let Some(position) = canvas_state.view.layout.get(&node_id(&node.id)) else {
            continue;
        };
        if (position.x - node.position.x).abs() > 0.5 || (position.y - node.position.y).abs() > 0.5
        {
            state.dispatch(AppCommand::MoveGraphNode {
                node_id: node.id.clone(),
                position: CanvasPoint::new(position.x, position.y),
            });
        }
    }
}

fn render_root_node(
    ui: &mut egui::Ui,
    state: &mut StudioAppState,
    snapshot: &AppSnapshot,
    nodes: &mut des_graph_egui::NodesCtx<'_>,
    scale: f32,
) {
    let has_flow = selected_flow(snapshot).is_some();
    Node::from_id(ROOT_NODE_ID)
        .outputs(usize::from(has_flow))
        .max_width(ROOT_NODE_SIZE.x)
        .socket_color(theme::SOURCE_CONNECTOR)
        .socket_radius(6.0)
        .animation_time(0.0)
        .show(nodes, ui, |ctx| {
            let interaction = ctx.interaction();
            ctx.framed_with(root_frame(interaction), |ui, sockets| {
                ui.set_min_size(ROOT_NODE_SIZE);
                ui.set_width(ROOT_NODE_SIZE.x);
                ui.label(theme::graph_heading_at("Workspace Roots", scale));
                ui.label(theme::metadata_at("Catalog node", scale));
                ui.add_space(8.0);
                render_root_selector(ui, state, snapshot, scale);
                ui.add_space(10.0);
                render_workspace_cards(ui, state, snapshot, scale);
                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);
                render_flow_cards(ui, state, snapshot, sockets, scale);
            })
        });
}

fn render_root_selector(
    ui: &mut egui::Ui,
    state: &mut StudioAppState,
    snapshot: &AppSnapshot,
    scale: f32,
) {
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
        .width(ROOT_NODE_SIZE.x - 24.0)
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
        ui.label(theme::metadata_at(&root.path, scale));
    }
}

fn render_workspace_cards(
    ui: &mut egui::Ui,
    state: &mut StudioAppState,
    snapshot: &AppSnapshot,
    scale: f32,
) {
    ui.label(RichText::new("Workspaces").size(14.0 * scale).strong());
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
                egui::Label::new(RichText::new(&workspace.name).size(14.0 * scale).strong())
                    .wrap()
                    .selectable(false),
            );
            ui.add(
                egui::Label::new(theme::metadata_at(&workspace.status, scale))
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
    sockets: &mut des_graph_egui::SocketLayout,
    scale: f32,
) {
    ui.label(RichText::new("Grouped Flows").size(14.0 * scale).strong());
    let selected_id = snapshot.selected_flow_id.as_deref();
    for flow in &snapshot.home.flows {
        let selected = Some(flow.id.as_str()) == selected_id;
        let response = card_frame(selected).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.add(
                egui::Label::new(RichText::new(&flow.name).size(14.0 * scale).strong())
                    .wrap()
                    .selectable(false),
            );
            ui.add(
                egui::Label::new(theme::metadata_at(&flow.group, scale))
                    .wrap()
                    .selectable(false),
            );
            ui.horizontal(|ui| {
                ui.label(theme::metadata_at(
                    format!("{} nodes", flow.node_count),
                    scale,
                ));
                ui.separator();
                ui.label(theme::metadata_at(&flow.trigger, scale));
            });
        });
        if response.response.interact(egui::Sense::click()).clicked() {
            state.dispatch(AppCommand::SelectFlow {
                flow_id: flow.id.clone(),
            });
        }
        if selected {
            sockets.output(0, response.response.rect);
        }
        ui.add_space(6.0);
    }
}

fn render_flow_node(
    ui: &mut egui::Ui,
    nodes: &mut des_graph_egui::NodesCtx<'_>,
    node: &GraphNodeSummary,
    scale: f32,
) {
    let source_input = is_source_node(node);
    let input_count = node.inputs.len() + usize::from(source_input);
    Node::from_id(node_id(&node.id))
        .inputs(input_count)
        .outputs(node.outputs.len())
        .max_width(NODE_SIZE.x)
        .socket_color(node_socket_color(node))
        .socket_radius(6.0)
        .animation_time(0.0)
        .show(nodes, ui, |ctx| {
            let interaction = ctx.interaction();
            ctx.framed_with(flow_node_frame(interaction), |ui, sockets| {
                ui.set_min_size(NODE_SIZE);
                ui.set_width(NODE_SIZE.x);
                ui.label(theme::metadata_at(&node.subtitle, scale));
                ui.label(theme::node_title_at(&node.title, scale));
                ui.add_space(12.0);

                if source_input {
                    sockets.row(ui, Some(0), None, |ui| {
                        ui.label(theme::metadata_at("< flow", scale));
                    });
                }

                for (index, port) in node.inputs.iter().enumerate() {
                    let input_index = index + usize::from(source_input);
                    sockets.row(ui, Some(input_index), None, |ui| {
                        ui.label(theme::metadata_at(format!("< {}", port.label), scale));
                    });
                }
                for (index, port) in node.outputs.iter().enumerate() {
                    sockets.row(ui, None, Some(index), |ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(theme::metadata_at(format!("{} >", port.label), scale));
                        });
                    });
                }
            })
        });
}

fn render_root_edges(
    ui: &mut egui::Ui,
    selected_edges: &mut HashSet<String>,
    edges: &mut des_graph_egui::EdgesCtx,
    graph: &FlowGraphSummary,
) {
    for node in graph
        .nodes
        .iter()
        .filter(|node| is_source_node(node) && !node.outputs.is_empty())
    {
        let edge_id = format!("root-to-{}", node.id);
        let mut selected = selected_edges.contains(&edge_id);
        let response = Edge::new((ROOT_NODE_ID, 0), (node_id(&node.id), 0), &mut selected)
            .curvature_factor(0.75)
            .show(edges, ui);
        sync_edge_selection(selected_edges, edge_id, selected, response.changed());
    }
}

fn render_flow_edges(
    ui: &mut egui::Ui,
    selected_edges: &mut HashSet<String>,
    edges: &mut des_graph_egui::EdgesCtx,
    graph: &FlowGraphSummary,
) {
    for edge in &graph.edges {
        let Some(from_node) = graph.nodes.iter().find(|node| node.id == edge.from_node_id) else {
            continue;
        };
        let Some(to_node) = graph.nodes.iter().find(|node| node.id == edge.to_node_id) else {
            continue;
        };
        let Some(from_index) = output_index(from_node, &edge.from_port_id) else {
            continue;
        };
        let Some(to_index) = input_index(to_node, &edge.to_port_id) else {
            continue;
        };

        let mut selected = selected_edges.contains(&edge.id);
        let response = Edge::new(
            (node_id(&edge.from_node_id), from_index),
            (node_id(&edge.to_node_id), to_index),
            &mut selected,
        )
        .curvature_factor(0.75)
        .show(edges, ui);
        sync_edge_selection(
            selected_edges,
            edge.id.clone(),
            selected,
            response.changed(),
        );
    }
}

fn sync_edge_selection(
    selected_edges: &mut HashSet<String>,
    edge_id: String,
    selected: bool,
    changed: bool,
) {
    if !changed {
        return;
    }
    if selected {
        selected_edges.insert(edge_id);
    } else {
        selected_edges.remove(&edge_id);
    }
}

fn input_index(node: &GraphNodeSummary, port_id: &str) -> Option<usize> {
    node.inputs.iter().position(|port| port.id == port_id)
}

fn output_index(node: &GraphNodeSummary, port_id: &str) -> Option<usize> {
    node.outputs.iter().position(|port| port.id == port_id)
}

fn root_frame(interaction: des_graph_egui::NodeInteraction) -> egui::Frame {
    node_frame(theme::PANEL, interaction)
}

fn flow_node_frame(interaction: des_graph_egui::NodeInteraction) -> egui::Frame {
    let fill = if interaction.hovered || interaction.selected {
        Color32::from_rgb(36, 44, 50)
    } else {
        Color32::from_rgb(29, 35, 39)
    };
    node_frame(fill, interaction)
}

fn node_frame(fill: Color32, interaction: des_graph_egui::NodeInteraction) -> egui::Frame {
    egui::Frame::new()
        .fill(fill)
        .stroke(Stroke::new(
            if interaction.selected { 1.5 } else { 1.0 },
            if interaction.selected {
                theme::STROKE_SELECTED
            } else {
                theme::STROKE
            },
        ))
        .corner_radius(8.0)
        .inner_margin(12.0)
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

fn node_socket_color(node: &GraphNodeSummary) -> Color32 {
    if is_source_node(node) {
        theme::SOURCE_CONNECTOR
    } else if node.outputs.is_empty() {
        Color32::from_rgb(151, 93, 219)
    } else {
        theme::CONNECTOR
    }
}

fn is_source_node(node: &GraphNodeSummary) -> bool {
    node.inputs.is_empty() && !node.outputs.is_empty()
}

fn flow_graph_world_bounds(graph: &FlowGraphSummary) -> Option<Rect> {
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

fn selected_flow(snapshot: &AppSnapshot) -> Option<&des_app::FlowSummary> {
    snapshot
        .home
        .flows
        .iter()
        .find(|flow| Some(flow.id.as_str()) == snapshot.selected_flow_id.as_deref())
}

fn root_node_world_rect() -> Rect {
    Rect::from_min_size(Pos2::new(28.0, 78.0), ROOT_NODE_SIZE)
}

fn node_id(id: &str) -> NodeId {
    const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

    let mut hash = FNV_OFFSET;
    for byte in id.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    NodeId::from_u64(hash)
}
