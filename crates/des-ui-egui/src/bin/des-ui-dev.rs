use des_core::StudioResult;
use des_ui_egui::NativeLaunchOptions;

fn main() -> StudioResult<()> {
    des_ui_egui::run_native(NativeLaunchOptions {
        debug_overlay: std::env::var("DES_UI_DEBUG_OVERLAY").is_ok_and(|value| value != "0"),
        initial_lab_view: std::env::var("DES_UI_LAB_VIEW").ok(),
        ..NativeLaunchOptions::default()
    })
}
