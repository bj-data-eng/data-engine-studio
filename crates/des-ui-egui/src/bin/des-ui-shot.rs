use des_app::AppCommand;
use des_ui_egui::NativeLaunchOptions;

fn main() {
    let title = std::env::var("DES_UI_HARNESS_TITLE")
        .unwrap_or_else(|_| "Data Engine Studio UI Harness".to_string());
    let width = env_f32("DES_UI_HARNESS_WIDTH").unwrap_or(1320.0);
    let height = env_f32("DES_UI_HARNESS_HEIGHT").unwrap_or(780.0);

    let options = NativeLaunchOptions {
        title,
        inner_size: [width, height],
        debug_overlay: env_bool("DES_UI_DEBUG_OVERLAY"),
        initial_scene_rect: env_scene_rect("DES_UI_SCENE_RECT"),
        startup_commands: startup_commands(),
    };

    if let Err(error) = des_ui_egui::run_native(options) {
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

fn env_scene_rect(name: &str) -> Option<[f32; 4]> {
    let value = std::env::var(name).ok()?;
    let parts: Vec<_> = value
        .split(',')
        .map(str::trim)
        .filter_map(|part| part.parse::<f32>().ok())
        .collect();
    match parts.as_slice() {
        [x, y, width, height] => Some([*x, *y, *width, *height]),
        _ => None,
    }
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
