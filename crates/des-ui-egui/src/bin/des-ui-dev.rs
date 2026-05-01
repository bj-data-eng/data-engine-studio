use des_core::StudioResult;
use des_ui_egui::NativeLaunchOptions;

fn main() -> StudioResult<()> {
    des_ui_egui::run_native(NativeLaunchOptions::default())
}
