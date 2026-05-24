use crate::catalog::{AppSnapshot, CanvasPoint, StudioCatalog};
use crate::command::AppCommand;

#[derive(Clone, Debug)]
pub struct StudioAppState {
    catalog: StudioCatalog,
    selected_root_id: Option<String>,
    selected_workspace_id: Option<String>,
    selected_project_id: Option<String>,
    selected_group_id: Option<String>,
    selected_flow_id: Option<String>,
}

impl StudioAppState {
    pub fn new() -> Self {
        let catalog = StudioCatalog::sample();
        let selected_root_id = catalog.workspace_roots.first().map(|root| root.id.clone());
        let selected_workspace_id = catalog
            .workspaces
            .iter()
            .find(|workspace| Some(workspace.root_id.as_str()) == selected_root_id.as_deref())
            .map(|workspace| workspace.id.clone());
        let selected_project_id = catalog
            .projects
            .iter()
            .find(|project| Some(project.workspace_id.as_str()) == selected_workspace_id.as_deref())
            .map(|project| project.id.clone());
        let selected_group_id = catalog
            .flow_groups
            .iter()
            .find(|group| Some(group.project_id.as_str()) == selected_project_id.as_deref())
            .map(|group| group.id.clone());
        let selected_flow_id = catalog
            .flows
            .iter()
            .find(|flow| Some(flow.group_id.as_str()) == selected_group_id.as_deref())
            .map(|flow| flow.id.clone());
        Self {
            catalog,
            selected_root_id,
            selected_workspace_id,
            selected_project_id,
            selected_group_id,
            selected_flow_id,
        }
    }

    pub fn catalog(&self) -> &StudioCatalog {
        &self.catalog
    }

    pub fn snapshot(&self) -> AppSnapshot {
        AppSnapshot {
            catalog: self.catalog.clone(),
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
        if !self
            .catalog
            .workspace_roots
            .iter()
            .any(|root| root.id == root_id)
        {
            return;
        }
        self.selected_root_id = Some(root_id.clone());
        let workspace_id = self
            .catalog
            .workspaces
            .iter()
            .find(|workspace| workspace.root_id == root_id)
            .map(|workspace| workspace.id.clone());
        self.selected_workspace_id = workspace_id.clone();
        self.select_first_project_in_workspace(workspace_id.as_deref());
    }

    fn select_workspace(&mut self, workspace_id: String) {
        let Some(workspace) = self
            .catalog
            .workspaces
            .iter()
            .find(|workspace| workspace.id == workspace_id)
        else {
            return;
        };
        self.selected_workspace_id = Some(workspace_id.clone());
        self.selected_root_id = Some(workspace.root_id.clone());
        self.select_first_project_in_workspace(Some(&workspace_id));
    }

    fn select_project(&mut self, project_id: String) {
        let Some(project) = self
            .catalog
            .projects
            .iter()
            .find(|project| project.id == project_id)
        else {
            return;
        };
        self.selected_project_id = Some(project_id.clone());
        self.selected_workspace_id = Some(project.workspace_id.clone());
        if let Some(workspace) = self
            .catalog
            .workspaces
            .iter()
            .find(|workspace| workspace.id == project.workspace_id)
        {
            self.selected_root_id = Some(workspace.root_id.clone());
        }
        self.select_first_group_in_project(Some(&project_id));
    }

    fn select_flow_group(&mut self, group_id: String) {
        let Some(group) = self
            .catalog
            .flow_groups
            .iter()
            .find(|group| group.id == group_id)
        else {
            return;
        };
        self.selected_group_id = Some(group_id.clone());
        self.select_project_without_cascade(group.project_id.clone());
        self.select_first_flow_in_group(Some(&group_id));
    }

    fn select_flow(&mut self, flow_id: String) {
        let Some(flow) = self.catalog.flows.iter().find(|flow| flow.id == flow_id) else {
            return;
        };
        self.selected_flow_id = Some(flow_id.clone());
        self.selected_group_id = Some(flow.group_id.clone());
        self.select_project_without_cascade(flow.project_id.clone());
    }

    fn select_first_project_in_workspace(&mut self, workspace_id: Option<&str>) {
        let project_id = self
            .catalog
            .projects
            .iter()
            .find(|project| Some(project.workspace_id.as_str()) == workspace_id)
            .map(|project| project.id.clone());
        self.selected_project_id = project_id.clone();
        self.select_first_group_in_project(project_id.as_deref());
    }

    fn select_first_group_in_project(&mut self, project_id: Option<&str>) {
        let group_id = self
            .catalog
            .flow_groups
            .iter()
            .find(|group| Some(group.project_id.as_str()) == project_id)
            .map(|group| group.id.clone());
        self.selected_group_id = group_id.clone();
        self.select_first_flow_in_group(group_id.as_deref());
    }

    fn select_first_flow_in_group(&mut self, group_id: Option<&str>) {
        self.selected_flow_id = self
            .catalog
            .flows
            .iter()
            .find(|flow| Some(flow.group_id.as_str()) == group_id)
            .map(|flow| flow.id.clone());
    }

    fn select_project_without_cascade(&mut self, project_id: String) {
        self.selected_project_id = Some(project_id.clone());
        if let Some(project) = self
            .catalog
            .projects
            .iter()
            .find(|project| project.id == project_id)
        {
            self.selected_workspace_id = Some(project.workspace_id.clone());
            if let Some(workspace) = self
                .catalog
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
        let Some(flow) = self
            .catalog
            .flows
            .iter_mut()
            .find(|flow| flow.id == flow_id)
        else {
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
        let Some(flow) = self
            .catalog
            .flows
            .iter_mut()
            .find(|flow| flow.id == flow_id)
        else {
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
