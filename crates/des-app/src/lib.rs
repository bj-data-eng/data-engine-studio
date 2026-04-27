use des_core::{AppInfo, Diagnostic};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FlowSummary {
    pub id: String,
    pub name: String,
    pub description: String,
    pub node_count: usize,
    pub trigger: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StudioHomeModel {
    pub app_info: AppInfo,
    pub flows: Vec<FlowSummary>,
    pub diagnostics: Vec<Diagnostic>,
}

impl StudioHomeModel {
    pub fn sample() -> Self {
        Self {
            app_info: AppInfo::current(),
            flows: vec![
                FlowSummary {
                    id: "customer-analytics".to_string(),
                    name: "Customer Analytics Pipeline".to_string(),
                    description: "CSV, order, and rate sources into country revenue metrics."
                        .to_string(),
                    node_count: 9,
                    trigger: "Manual".to_string(),
                },
                FlowSummary {
                    id: "blank-flow".to_string(),
                    name: "Untitled Flow".to_string(),
                    description: "A new visual ETL flow ready for nodes.".to_string(),
                    node_count: 0,
                    trigger: "Manual".to_string(),
                },
            ],
            diagnostics: vec![Diagnostic::info(
                "Milestone 1 shell is running through the Python native extension.",
            )],
        }
    }
}

#[derive(Clone, Debug)]
pub struct StudioAppState {
    home: StudioHomeModel,
    selected_flow_id: Option<String>,
}

impl StudioAppState {
    pub fn new() -> Self {
        let home = StudioHomeModel::sample();
        let selected_flow_id = home.flows.first().map(|flow| flow.id.clone());
        Self {
            home,
            selected_flow_id,
        }
    }

    pub fn home(&self) -> &StudioHomeModel {
        &self.home
    }

    pub fn selected_flow_id(&self) -> Option<&str> {
        self.selected_flow_id.as_deref()
    }

    pub fn select_flow(&mut self, flow_id: impl Into<String>) {
        self.selected_flow_id = Some(flow_id.into());
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

        assert_eq!(state.selected_flow_id(), Some("customer-analytics"));
        assert_eq!(state.home().flows.len(), 2);
    }

    #[test]
    fn select_flow_updates_current_selection() {
        let mut state = StudioAppState::new();

        state.select_flow("blank-flow");

        assert_eq!(state.selected_flow_id(), Some("blank-flow"));
    }
}
