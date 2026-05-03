use des_core::StudioResult;
use des_ui_egui::NativeLaunchOptions;

fn main() -> StudioResult<()> {
    des_ui_egui::run_native(NativeLaunchOptions {
        debug_overlay: std::env::var("DES_UI_DEBUG_OVERLAY").is_ok_and(|value| value != "0"),
        initial_lab_view: std::env::var("DES_UI_LAB_VIEW").ok(),
        initial_lab_scroll: env_scroll_position(),
        ..NativeLaunchOptions::default()
    })
}

fn env_scroll_position() -> Option<[f32; 2]> {
    let x = env_f32("DES_UI_LAB_SCROLL_X").unwrap_or(0.0);
    let y = env_f32("DES_UI_LAB_SCROLL_Y").unwrap_or(0.0);
    (x != 0.0 || y != 0.0).then_some([x, y])
}

fn env_f32(name: &str) -> Option<f32> {
    std::env::var(name).ok()?.parse().ok()
}
