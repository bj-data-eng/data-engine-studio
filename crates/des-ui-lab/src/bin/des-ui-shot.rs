use des_app::AppCommand;
use des_ui_lab::NativeLaunchOptions;

fn main() {
    let title = std::env::var("DES_UI_HARNESS_TITLE")
        .unwrap_or_else(|_| "Data Engine Studio UI Harness".to_string());
    let width = env_f32("DES_UI_HARNESS_WIDTH").unwrap_or(1320.0);
    let height = env_f32("DES_UI_HARNESS_HEIGHT").unwrap_or(780.0);

    let options = NativeLaunchOptions {
        title,
        inner_size: [width, height],
        debug_overlay: env_bool("DES_UI_DEBUG_OVERLAY"),
        initial_lab_view: std::env::var("DES_UI_LAB_VIEW").ok(),
        initial_lab_scroll: env_scroll_position(),
        startup_commands: startup_commands(),
    };

    if let Err(error) = des_ui_lab::run_native(options) {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn env_f32(name: &str) -> Option<f32> {
    std::env::var(name).ok()?.parse().ok()
}

fn env_bool(name: &str) -> bool {
    std::env::var(name)
        .ok()
        .is_some_and(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
}

fn env_scroll_position() -> Option<[f32; 2]> {
    let x = env_f32("DES_UI_LAB_SCROLL_X").unwrap_or(0.0);
    let y = env_f32("DES_UI_LAB_SCROLL_Y").unwrap_or(0.0);
    (x != 0.0 || y != 0.0).then_some([x, y])
}

fn startup_commands() -> Vec<AppCommand> {
    let mut commands = Vec::new();
    if let Ok(root_id) = std::env::var("DES_UI_SELECTED_ROOT") {
        commands.push(AppCommand::SelectWorkspaceRoot { root_id });
    }
    if let Ok(workspace_id) = std::env::var("DES_UI_SELECTED_WORKSPACE") {
        commands.push(AppCommand::SelectWorkspace { workspace_id });
    }
    if let Ok(project_id) = std::env::var("DES_UI_SELECTED_PROJECT") {
        commands.push(AppCommand::SelectProject { project_id });
    }
    if let Ok(group_id) = std::env::var("DES_UI_SELECTED_GROUP") {
        commands.push(AppCommand::SelectFlowGroup { group_id });
    }
    if let Ok(flow_id) = std::env::var("DES_UI_SELECTED_FLOW") {
        commands.push(AppCommand::SelectFlow { flow_id });
    }
    commands
}
