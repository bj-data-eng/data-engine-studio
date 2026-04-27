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
    pub description: String,
    pub node_count: usize,
    pub trigger: String,
    pub group: String,
    pub sources: Vec<SourceNodeSummary>,
    pub graph: FlowGraphSummary,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StudioHomeModel {
    pub app_info: AppInfo,
    pub workspace_roots: Vec<WorkspaceRootSummary>,
    pub workspaces: Vec<WorkspaceSummary>,
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
            flows: vec![
                FlowSummary {
                    id: "customer-analytics".to_string(),
                    name: "Customer Analytics Pipeline".to_string(),
                    description: "CSV, order, and rate sources into country revenue metrics."
                        .to_string(),
                    node_count: 9,
                    trigger: "Manual".to_string(),
                    group: "Customer Analytics".to_string(),
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
                    description: "A new visual ETL flow ready for nodes.".to_string(),
                    node_count: 0,
                    trigger: "Manual".to_string(),
                    group: "Drafts".to_string(),
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
            ],
            diagnostics: vec![Diagnostic::info(
                "Milestone 1 shell is running through the Python native extension.",
            )],
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AppSnapshot {
    pub home: StudioHomeModel,
    pub selected_root_id: Option<String>,
    pub selected_workspace_id: Option<String>,
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
        let selected_flow_id = home.flows.first().map(|flow| flow.id.clone());
        Self {
            home,
            selected_root_id,
            selected_workspace_id,
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
            selected_flow_id: self.selected_flow_id.clone(),
        }
    }

    pub fn selected_root_id(&self) -> Option<&str> {
        self.selected_root_id.as_deref()
    }

    pub fn selected_workspace_id(&self) -> Option<&str> {
        self.selected_workspace_id.as_deref()
    }

    pub fn selected_flow_id(&self) -> Option<&str> {
        self.selected_flow_id.as_deref()
    }

    pub fn dispatch(&mut self, command: AppCommand) {
        match command {
            AppCommand::SelectWorkspaceRoot { root_id } => self.select_workspace_root(root_id),
            AppCommand::SelectWorkspace { workspace_id } => {
                self.selected_workspace_id = Some(workspace_id)
            }
            AppCommand::SelectFlow { flow_id } => self.selected_flow_id = Some(flow_id),
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
        self.selected_workspace_id = self
            .home
            .workspaces
            .iter()
            .find(|workspace| workspace.root_id == root_id)
            .map(|workspace| workspace.id.clone());
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
        assert_eq!(state.selected_flow_id(), Some("customer-analytics"));
        assert_eq!(state.home().flows.len(), 2);
    }

    #[test]
    fn dispatch_select_flow_updates_current_selection() {
        let mut state = StudioAppState::new();

        state.dispatch(AppCommand::SelectFlow {
            flow_id: "blank-flow".to_string(),
        });

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
