use des_core::{AppInfo, Diagnostic};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceRootSummary {
    pub id: String,
    pub name: String,
    pub path: String,
    pub workspace_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceSummary {
    pub id: String,
    pub name: String,
    pub root_id: String,
    pub status: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectSummary {
    pub id: String,
    pub name: String,
    pub workspace_id: String,
    pub description: String,
    pub status: String,
    pub group_count: usize,
    pub flow_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FlowGroupSummary {
    pub id: String,
    pub name: String,
    pub project_id: String,
    pub description: String,
    pub kind: String,
    pub flow_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceNodeSummary {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub detail: String,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CanvasPoint {
    pub x: f32,
    pub y: f32,
}

impl CanvasPoint {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GraphPortSide {
    Input,
    Output,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GraphPortSummary {
    pub id: String,
    pub label: String,
    pub side: GraphPortSide,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GraphNodeSummary {
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub position: CanvasPoint,
    pub inputs: Vec<GraphPortSummary>,
    pub outputs: Vec<GraphPortSummary>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GraphEdgeSummary {
    pub id: String,
    pub from_node_id: String,
    pub from_port_id: String,
    pub to_node_id: String,
    pub to_port_id: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FlowGraphSummary {
    pub nodes: Vec<GraphNodeSummary>,
    pub edges: Vec<GraphEdgeSummary>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FlowSummary {
    pub id: String,
    pub name: String,
    pub project_id: String,
    pub group_id: String,
    pub description: String,
    pub node_count: usize,
    pub trigger: String,
    pub sources: Vec<SourceNodeSummary>,
    pub graph: FlowGraphSummary,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StudioCatalog {
    pub app_info: AppInfo,
    pub workspace_roots: Vec<WorkspaceRootSummary>,
    pub workspaces: Vec<WorkspaceSummary>,
    pub projects: Vec<ProjectSummary>,
    pub flow_groups: Vec<FlowGroupSummary>,
    pub flows: Vec<FlowSummary>,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AppSnapshot {
    pub catalog: StudioCatalog,
    pub selected_root_id: Option<String>,
    pub selected_workspace_id: Option<String>,
    pub selected_project_id: Option<String>,
    pub selected_group_id: Option<String>,
    pub selected_flow_id: Option<String>,
}
