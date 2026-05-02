use des_app::{AppCommand, CanvasPoint, StudioAppState};

#[test]
fn new_state_selects_first_flow() {
    let state = StudioAppState::new();

    assert_eq!(state.selected_root_id(), Some("local-dev"));
    assert_eq!(state.selected_workspace_id(), Some("analytics"));
    assert_eq!(state.selected_project_id(), Some("customer-360"));
    assert_eq!(state.selected_group_id(), Some("customer-etl"));
    assert_eq!(state.selected_flow_id(), Some("customer-analytics"));
    assert_eq!(state.catalog().flows.len(), 9);
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
        .catalog
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
