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
pub struct StudioHomeModel {
    pub app_info: AppInfo,
    pub workspace_roots: Vec<WorkspaceRootSummary>,
    pub workspaces: Vec<WorkspaceSummary>,
    pub projects: Vec<ProjectSummary>,
    pub flow_groups: Vec<FlowGroupSummary>,
    pub flows: Vec<FlowSummary>,
    pub diagnostics: Vec<Diagnostic>,
}

impl StudioHomeModel {
    pub fn sample() -> Self {
        Self {
            app_info: AppInfo::current(),
            workspace_roots: vec![
                WorkspaceRootSummary {
                    id: "local-dev".to_string(),
                    name: "Local Development".to_string(),
                    path: "C:\\DEV_PROJECT\\workspaces".to_string(),
                    workspace_count: 2,
                },
                WorkspaceRootSummary {
                    id: "team-share".to_string(),
                    name: "Team Share".to_string(),
                    path: "\\\\shared\\data-engine-studio".to_string(),
                    workspace_count: 4,
                },
            ],
            workspaces: vec![
                WorkspaceSummary {
                    id: "analytics".to_string(),
                    name: "Analytics".to_string(),
                    root_id: "local-dev".to_string(),
                    status: "Available".to_string(),
                },
                WorkspaceSummary {
                    id: "sandbox".to_string(),
                    name: "Sandbox".to_string(),
                    root_id: "local-dev".to_string(),
                    status: "Draft".to_string(),
                },
                WorkspaceSummary {
                    id: "finance".to_string(),
                    name: "Finance Ops".to_string(),
                    root_id: "team-share".to_string(),
                    status: "Leased by another workstation".to_string(),
                },
            ],
            projects: vec![
                ProjectSummary {
                    id: "customer-360".to_string(),
                    name: "Customer 360".to_string(),
                    workspace_id: "analytics".to_string(),
                    description: "Trusted customer, order, support, and revenue pipelines."
                        .to_string(),
                    status: "Active".to_string(),
                    group_count: 3,
                    flow_count: 6,
                },
                ProjectSummary {
                    id: "growth-lab".to_string(),
                    name: "Growth Lab".to_string(),
                    workspace_id: "analytics".to_string(),
                    description: "Experimental ingestion and campaign attribution work."
                        .to_string(),
                    status: "Design".to_string(),
                    group_count: 2,
                    flow_count: 3,
                },
                ProjectSummary {
                    id: "sandbox-drafts".to_string(),
                    name: "Sandbox Drafts".to_string(),
                    workspace_id: "sandbox".to_string(),
                    description: "Scratch flows and prototype transforms.".to_string(),
                    status: "Draft".to_string(),
                    group_count: 1,
                    flow_count: 1,
                },
                ProjectSummary {
                    id: "finance-recon".to_string(),
                    name: "Finance Reconciliation".to_string(),
                    workspace_id: "finance".to_string(),
                    description: "Shared ledger validation and close automation.".to_string(),
                    status: "Owned remotely".to_string(),
                    group_count: 2,
                    flow_count: 2,
                },
            ],
            flow_groups: vec![
                FlowGroupSummary {
                    id: "customer-etl".to_string(),
                    name: "ETL Flows".to_string(),
                    project_id: "customer-360".to_string(),
                    description: "Source-to-target data movement and transforms.".to_string(),
                    kind: "Pipelines".to_string(),
                    flow_count: 3,
                },
                FlowGroupSummary {
                    id: "customer-quality".to_string(),
                    name: "Quality Gates".to_string(),
                    project_id: "customer-360".to_string(),
                    description: "Assertions, drift checks, and publish blockers.".to_string(),
                    kind: "Validation".to_string(),
                    flow_count: 2,
                },
                FlowGroupSummary {
                    id: "customer-runtime".to_string(),
                    name: "Runtime Jobs".to_string(),
                    project_id: "customer-360".to_string(),
                    description: "Manual, scheduled, and polling trigger definitions.".to_string(),
                    kind: "Triggers".to_string(),
                    flow_count: 1,
                },
                FlowGroupSummary {
                    id: "growth-ingest".to_string(),
                    name: "Acquisition Ingest".to_string(),
                    project_id: "growth-lab".to_string(),
                    description: "Campaign and web event collection.".to_string(),
                    kind: "Pipelines".to_string(),
                    flow_count: 2,
                },
                FlowGroupSummary {
                    id: "growth-models".to_string(),
                    name: "Attribution Models".to_string(),
                    project_id: "growth-lab".to_string(),
                    description: "Semantic models and feature preparation.".to_string(),
                    kind: "Models".to_string(),
                    flow_count: 1,
                },
                FlowGroupSummary {
                    id: "sandbox-draft-flows".to_string(),
                    name: "Drafts".to_string(),
                    project_id: "sandbox-drafts".to_string(),
                    description: "Unpublished experiments.".to_string(),
                    kind: "Draft".to_string(),
                    flow_count: 1,
                },
                FlowGroupSummary {
                    id: "finance-ledger".to_string(),
                    name: "Ledger Checks".to_string(),
                    project_id: "finance-recon".to_string(),
                    description: "Daily ledger and settlement reconciliation.".to_string(),
                    kind: "Validation".to_string(),
                    flow_count: 2,
                },
            ],
            flows: vec![
                FlowSummary {
                    id: "customer-analytics".to_string(),
                    name: "Customer Analytics Pipeline".to_string(),
                    project_id: "customer-360".to_string(),
                    group_id: "customer-etl".to_string(),
                    description: "CSV, order, and rate sources into country revenue metrics."
                        .to_string(),
                    node_count: 9,
                    trigger: "Manual".to_string(),
                    sources: vec![
                        SourceNodeSummary {
                            id: "customers-csv".to_string(),
                            name: "customers.csv".to_string(),
                            kind: "File Source".to_string(),
                            detail: "12 columns / 1.2M rows".to_string(),
                        },
                        SourceNodeSummary {
                            id: "orders-db".to_string(),
                            name: "orders".to_string(),
                            kind: "Database Source".to_string(),
                            detail: "postgres://analytics".to_string(),
                        },
                        SourceNodeSummary {
                            id: "rates-api".to_string(),
                            name: "fx/rates/latest".to_string(),
                            kind: "API Source".to_string(),
                            detail: "4 columns / latest".to_string(),
                        },
                    ],
                    graph: FlowGraphSummary {
                        nodes: vec![
                            GraphNodeSummary {
                                id: "file-source".to_string(),
                                title: "File Source".to_string(),
                                subtitle: "customers.csv".to_string(),
                                position: CanvasPoint::new(760.0, 170.0),
                                inputs: Vec::new(),
                                outputs: vec![GraphPortSummary {
                                    id: "out".to_string(),
                                    label: "table".to_string(),
                                    side: GraphPortSide::Output,
                                }],
                            },
                            GraphNodeSummary {
                                id: "db-source".to_string(),
                                title: "Database Source".to_string(),
                                subtitle: "orders".to_string(),
                                position: CanvasPoint::new(760.0, 340.0),
                                inputs: Vec::new(),
                                outputs: vec![GraphPortSummary {
                                    id: "out".to_string(),
                                    label: "table".to_string(),
                                    side: GraphPortSide::Output,
                                }],
                            },
                            GraphNodeSummary {
                                id: "join".to_string(),
                                title: "Join".to_string(),
                                subtitle: "customer_id = id".to_string(),
                                position: CanvasPoint::new(1080.0, 255.0),
                                inputs: vec![
                                    GraphPortSummary {
                                        id: "left".to_string(),
                                        label: "left".to_string(),
                                        side: GraphPortSide::Input,
                                    },
                                    GraphPortSummary {
                                        id: "right".to_string(),
                                        label: "right".to_string(),
                                        side: GraphPortSide::Input,
                                    },
                                ],
                                outputs: vec![GraphPortSummary {
                                    id: "out".to_string(),
                                    label: "joined".to_string(),
                                    side: GraphPortSide::Output,
                                }],
                            },
                            GraphNodeSummary {
                                id: "aggregate".to_string(),
                                title: "Aggregate".to_string(),
                                subtitle: "group by country".to_string(),
                                position: CanvasPoint::new(1385.0, 255.0),
                                inputs: vec![GraphPortSummary {
                                    id: "in".to_string(),
                                    label: "input".to_string(),
                                    side: GraphPortSide::Input,
                                }],
                                outputs: vec![GraphPortSummary {
                                    id: "out".to_string(),
                                    label: "metrics".to_string(),
                                    side: GraphPortSide::Output,
                                }],
                            },
                            GraphNodeSummary {
                                id: "output".to_string(),
                                title: "Database Output".to_string(),
                                subtitle: "country_revenue".to_string(),
                                position: CanvasPoint::new(1660.0, 255.0),
                                inputs: vec![GraphPortSummary {
                                    id: "in".to_string(),
                                    label: "input".to_string(),
                                    side: GraphPortSide::Input,
                                }],
                                outputs: Vec::new(),
                            },
                        ],
                        edges: vec![
                            GraphEdgeSummary {
                                id: "file-to-join".to_string(),
                                from_node_id: "file-source".to_string(),
                                from_port_id: "out".to_string(),
                                to_node_id: "join".to_string(),
                                to_port_id: "left".to_string(),
                            },
                            GraphEdgeSummary {
                                id: "db-to-join".to_string(),
                                from_node_id: "db-source".to_string(),
                                from_port_id: "out".to_string(),
                                to_node_id: "join".to_string(),
                                to_port_id: "right".to_string(),
                            },
                            GraphEdgeSummary {
                                id: "join-to-aggregate".to_string(),
                                from_node_id: "join".to_string(),
                                from_port_id: "out".to_string(),
                                to_node_id: "aggregate".to_string(),
                                to_port_id: "in".to_string(),
                            },
                            GraphEdgeSummary {
                                id: "aggregate-to-output".to_string(),
                                from_node_id: "aggregate".to_string(),
                                from_port_id: "out".to_string(),
                                to_node_id: "output".to_string(),
                                to_port_id: "in".to_string(),
                            },
                        ],
                    },
                },
                FlowSummary {
                    id: "blank-flow".to_string(),
                    name: "Untitled Flow".to_string(),
                    project_id: "sandbox-drafts".to_string(),
                    group_id: "sandbox-draft-flows".to_string(),
                    description: "A new visual ETL flow ready for nodes.".to_string(),
                    node_count: 0,
                    trigger: "Manual".to_string(),
                    sources: Vec::new(),
                    graph: FlowGraphSummary {
                        nodes: vec![GraphNodeSummary {
                            id: "manual-trigger".to_string(),
                            title: "Manual Trigger".to_string(),
                            subtitle: "Click Run to start".to_string(),
                            position: CanvasPoint::new(760.0, 220.0),
                            inputs: Vec::new(),
                            outputs: vec![GraphPortSummary {
                                id: "run".to_string(),
                                label: "run".to_string(),
                                side: GraphPortSide::Output,
                            }],
                        }],
                        edges: Vec::new(),
                    },
                },
                sample_empty_flow(
                    "nightly-orders",
                    "Nightly Orders Refresh",
                    "customer-360",
                    "customer-etl",
                    "Warehouse orders and line items for dashboard refresh.",
                    7,
                    "Scheduled",
                ),
                sample_empty_flow(
                    "support-events",
                    "Support Event Import",
                    "customer-360",
                    "customer-etl",
                    "Ticket, chat, and CSAT events normalized by customer.",
                    6,
                    "Polling",
                ),
                sample_empty_flow(
                    "customer-contracts",
                    "Customer Contract Checks",
                    "customer-360",
                    "customer-quality",
                    "Contract lineage, duplicate keys, and missing terms checks.",
                    4,
                    "On publish",
                ),
                sample_empty_flow(
                    "revenue-drift",
                    "Revenue Drift Watch",
                    "customer-360",
                    "customer-quality",
                    "Threshold alerts for country revenue movement.",
                    5,
                    "Scheduled",
                ),
                sample_empty_flow(
                    "manual-replay",
                    "Manual Replay Job",
                    "customer-360",
                    "customer-runtime",
                    "Run-scoped replay for failed partitions.",
                    3,
                    "Manual",
                ),
                sample_empty_flow(
                    "campaign-ingest",
                    "Campaign Event Ingest",
                    "growth-lab",
                    "growth-ingest",
                    "Landing page and campaign events from API sources.",
                    5,
                    "Polling",
                ),
                sample_empty_flow(
                    "ledger-recon",
                    "Daily Ledger Reconciliation",
                    "finance-recon",
                    "finance-ledger",
                    "Compares source ledger extracts to settlement totals.",
                    8,
                    "Scheduled",
                ),
            ],
            diagnostics: vec![Diagnostic::info(
                "Milestone 1 shell is running through the Python native extension.",
            )],
        }
    }
}

fn sample_empty_flow(
    id: &str,
    name: &str,
    project_id: &str,
    group_id: &str,
    description: &str,
    node_count: usize,
    trigger: &str,
) -> FlowSummary {
    FlowSummary {
        id: id.to_string(),
        name: name.to_string(),
        project_id: project_id.to_string(),
        group_id: group_id.to_string(),
        description: description.to_string(),
        node_count,
        trigger: trigger.to_string(),
        sources: Vec::new(),
        graph: FlowGraphSummary {
            nodes: Vec::new(),
            edges: Vec::new(),
        },
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AppSnapshot {
    pub home: StudioHomeModel,
    pub selected_root_id: Option<String>,
    pub selected_workspace_id: Option<String>,
    pub selected_project_id: Option<String>,
    pub selected_group_id: Option<String>,
    pub selected_flow_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AppCommand {
    SelectWorkspaceRoot {
        root_id: String,
    },
    SelectWorkspace {
        workspace_id: String,
    },
    SelectProject {
        project_id: String,
    },
    SelectFlowGroup {
        group_id: String,
    },
    SelectFlow {
        flow_id: String,
    },
    MoveGraphNode {
        node_id: String,
        position: CanvasPoint,
    },
    MoveGraphNodeBy {
        node_id: String,
        dx: f32,
        dy: f32,
    },
}

#[derive(Clone, Debug)]
pub struct StudioAppState {
    home: StudioHomeModel,
    selected_root_id: Option<String>,
    selected_workspace_id: Option<String>,
    selected_project_id: Option<String>,
    selected_group_id: Option<String>,
    selected_flow_id: Option<String>,
}

impl StudioAppState {
    pub fn new() -> Self {
        let home = StudioHomeModel::sample();
        let selected_root_id = home.workspace_roots.first().map(|root| root.id.clone());
        let selected_workspace_id = home
            .workspaces
            .iter()
            .find(|workspace| Some(workspace.root_id.as_str()) == selected_root_id.as_deref())
            .map(|workspace| workspace.id.clone());
        let selected_project_id = home
            .projects
            .iter()
            .find(|project| Some(project.workspace_id.as_str()) == selected_workspace_id.as_deref())
            .map(|project| project.id.clone());
        let selected_group_id = home
            .flow_groups
            .iter()
            .find(|group| Some(group.project_id.as_str()) == selected_project_id.as_deref())
            .map(|group| group.id.clone());
        let selected_flow_id = home
            .flows
            .iter()
            .find(|flow| Some(flow.group_id.as_str()) == selected_group_id.as_deref())
            .map(|flow| flow.id.clone());
        Self {
            home,
            selected_root_id,
            selected_workspace_id,
            selected_project_id,
            selected_group_id,
            selected_flow_id,
        }
    }

    pub fn home(&self) -> &StudioHomeModel {
        &self.home
    }

    pub fn snapshot(&self) -> AppSnapshot {
        AppSnapshot {
            home: self.home.clone(),
            selected_root_id: self.selected_root_id.clone(),
            selected_workspace_id: self.selected_workspace_id.clone(),
            selected_project_id: self.selected_project_id.clone(),
            selected_group_id: self.selected_group_id.clone(),
            selected_flow_id: self.selected_flow_id.clone(),
        }
    }

    pub fn selected_root_id(&self) -> Option<&str> {
        self.selected_root_id.as_deref()
    }

    pub fn selected_workspace_id(&self) -> Option<&str> {
        self.selected_workspace_id.as_deref()
    }

    pub fn selected_project_id(&self) -> Option<&str> {
        self.selected_project_id.as_deref()
    }

    pub fn selected_group_id(&self) -> Option<&str> {
        self.selected_group_id.as_deref()
    }

    pub fn selected_flow_id(&self) -> Option<&str> {
        self.selected_flow_id.as_deref()
    }

    pub fn dispatch(&mut self, command: AppCommand) {
        match command {
            AppCommand::SelectWorkspaceRoot { root_id } => self.select_workspace_root(root_id),
            AppCommand::SelectWorkspace { workspace_id } => self.select_workspace(workspace_id),
            AppCommand::SelectProject { project_id } => self.select_project(project_id),
            AppCommand::SelectFlowGroup { group_id } => self.select_flow_group(group_id),
            AppCommand::SelectFlow { flow_id } => self.select_flow(flow_id),
            AppCommand::MoveGraphNode { node_id, position } => {
                self.move_graph_node(&node_id, position)
            }
            AppCommand::MoveGraphNodeBy { node_id, dx, dy } => {
                self.move_graph_node_by(&node_id, dx, dy)
            }
        }
    }

    fn select_workspace_root(&mut self, root_id: String) {
        self.selected_root_id = Some(root_id.clone());
        let workspace_id = self
            .home
            .workspaces
            .iter()
            .find(|workspace| workspace.root_id == root_id)
            .map(|workspace| workspace.id.clone());
        self.selected_workspace_id = workspace_id.clone();
        self.select_first_project_in_workspace(workspace_id.as_deref());
    }

    fn select_workspace(&mut self, workspace_id: String) {
        self.selected_workspace_id = Some(workspace_id.clone());
        if let Some(workspace) = self
            .home
            .workspaces
            .iter()
            .find(|workspace| workspace.id == workspace_id)
        {
            self.selected_root_id = Some(workspace.root_id.clone());
        }
        self.select_first_project_in_workspace(Some(&workspace_id));
    }

    fn select_project(&mut self, project_id: String) {
        self.selected_project_id = Some(project_id.clone());
        if let Some(project) = self
            .home
            .projects
            .iter()
            .find(|project| project.id == project_id)
        {
            self.selected_workspace_id = Some(project.workspace_id.clone());
            if let Some(workspace) = self
                .home
                .workspaces
                .iter()
                .find(|workspace| workspace.id == project.workspace_id)
            {
                self.selected_root_id = Some(workspace.root_id.clone());
            }
        }
        self.select_first_group_in_project(Some(&project_id));
    }

    fn select_flow_group(&mut self, group_id: String) {
        self.selected_group_id = Some(group_id.clone());
        if let Some(group) = self
            .home
            .flow_groups
            .iter()
            .find(|group| group.id == group_id)
        {
            self.select_project_without_cascade(group.project_id.clone());
        }
        self.select_first_flow_in_group(Some(&group_id));
    }

    fn select_flow(&mut self, flow_id: String) {
        self.selected_flow_id = Some(flow_id.clone());
        if let Some(flow) = self.home.flows.iter().find(|flow| flow.id == flow_id) {
            self.selected_group_id = Some(flow.group_id.clone());
            self.select_project_without_cascade(flow.project_id.clone());
        }
    }

    fn select_first_project_in_workspace(&mut self, workspace_id: Option<&str>) {
        let project_id = self
            .home
            .projects
            .iter()
            .find(|project| Some(project.workspace_id.as_str()) == workspace_id)
            .map(|project| project.id.clone());
        self.selected_project_id = project_id.clone();
        self.select_first_group_in_project(project_id.as_deref());
    }

    fn select_first_group_in_project(&mut self, project_id: Option<&str>) {
        let group_id = self
            .home
            .flow_groups
            .iter()
            .find(|group| Some(group.project_id.as_str()) == project_id)
            .map(|group| group.id.clone());
        self.selected_group_id = group_id.clone();
        self.select_first_flow_in_group(group_id.as_deref());
    }

    fn select_first_flow_in_group(&mut self, group_id: Option<&str>) {
        self.selected_flow_id = self
            .home
            .flows
            .iter()
            .find(|flow| Some(flow.group_id.as_str()) == group_id)
            .map(|flow| flow.id.clone());
    }

    fn select_project_without_cascade(&mut self, project_id: String) {
        self.selected_project_id = Some(project_id.clone());
        if let Some(project) = self
            .home
            .projects
            .iter()
            .find(|project| project.id == project_id)
        {
            self.selected_workspace_id = Some(project.workspace_id.clone());
            if let Some(workspace) = self
                .home
                .workspaces
                .iter()
                .find(|workspace| workspace.id == project.workspace_id)
            {
                self.selected_root_id = Some(workspace.root_id.clone());
            }
        }
    }

    fn move_graph_node(&mut self, node_id: &str, position: CanvasPoint) {
        let Some(flow_id) = self.selected_flow_id.as_deref() else {
            return;
        };
        let Some(flow) = self.home.flows.iter_mut().find(|flow| flow.id == flow_id) else {
            return;
        };
        let Some(node) = flow.graph.nodes.iter_mut().find(|node| node.id == node_id) else {
            return;
        };
        node.position = position;
    }

    fn move_graph_node_by(&mut self, node_id: &str, dx: f32, dy: f32) {
        let Some(flow_id) = self.selected_flow_id.as_deref() else {
            return;
        };
        let Some(flow) = self.home.flows.iter_mut().find(|flow| flow.id == flow_id) else {
            return;
        };
        let Some(node) = flow.graph.nodes.iter_mut().find(|node| node.id == node_id) else {
            return;
        };
        node.position = CanvasPoint::new(node.position.x + dx, node.position.y + dy);
    }
}

impl Default for StudioAppState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_state_selects_first_flow() {
        let state = StudioAppState::new();

        assert_eq!(state.selected_root_id(), Some("local-dev"));
        assert_eq!(state.selected_workspace_id(), Some("analytics"));
        assert_eq!(state.selected_project_id(), Some("customer-360"));
        assert_eq!(state.selected_group_id(), Some("customer-etl"));
        assert_eq!(state.selected_flow_id(), Some("customer-analytics"));
        assert_eq!(state.home().flows.len(), 9);
    }

    #[test]
    fn dispatch_select_flow_updates_current_selection() {
        let mut state = StudioAppState::new();

        state.dispatch(AppCommand::SelectFlow {
            flow_id: "blank-flow".to_string(),
        });

        assert_eq!(state.selected_workspace_id(), Some("sandbox"));
        assert_eq!(state.selected_project_id(), Some("sandbox-drafts"));
        assert_eq!(state.selected_group_id(), Some("sandbox-draft-flows"));
        assert_eq!(state.selected_flow_id(), Some("blank-flow"));
    }

    #[test]
    fn dispatch_select_root_updates_workspace_selection() {
        let mut state = StudioAppState::new();

        state.dispatch(AppCommand::SelectWorkspaceRoot {
            root_id: "team-share".to_string(),
        });

        assert_eq!(state.selected_root_id(), Some("team-share"));
        assert_eq!(state.selected_workspace_id(), Some("finance"));
        assert_eq!(state.selected_project_id(), Some("finance-recon"));
        assert_eq!(state.selected_group_id(), Some("finance-ledger"));
        assert_eq!(state.selected_flow_id(), Some("ledger-recon"));
    }

    #[test]
    fn dispatch_select_project_cascades_to_first_group_and_flow() {
        let mut state = StudioAppState::new();

        state.dispatch(AppCommand::SelectProject {
            project_id: "growth-lab".to_string(),
        });

        assert_eq!(state.selected_workspace_id(), Some("analytics"));
        assert_eq!(state.selected_project_id(), Some("growth-lab"));
        assert_eq!(state.selected_group_id(), Some("growth-ingest"));
        assert_eq!(state.selected_flow_id(), Some("campaign-ingest"));
    }

    #[test]
    fn dispatch_select_group_cascades_to_first_flow() {
        let mut state = StudioAppState::new();

        state.dispatch(AppCommand::SelectFlowGroup {
            group_id: "customer-quality".to_string(),
        });

        assert_eq!(state.selected_project_id(), Some("customer-360"));
        assert_eq!(state.selected_group_id(), Some("customer-quality"));
        assert_eq!(state.selected_flow_id(), Some("customer-contracts"));
    }

    #[test]
    fn dispatch_move_graph_node_updates_selected_flow_node_position() {
        let mut state = StudioAppState::new();

        state.dispatch(AppCommand::MoveGraphNode {
            node_id: "join".to_string(),
            position: CanvasPoint::new(410.0, 180.0),
        });

        let join = state
            .snapshot()
            .home
            .flows
            .into_iter()
            .find(|flow| flow.id == "customer-analytics")
            .unwrap()
            .graph
            .nodes
            .into_iter()
            .find(|node| node.id == "join")
            .unwrap();
        assert_eq!(join.position, CanvasPoint::new(410.0, 180.0));
    }
}
