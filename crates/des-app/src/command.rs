use crate::catalog::CanvasPoint;

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
