use des_ui_wgpu::{ClearColor, RenderOptions};
use des_ui_window::{NativeOptions, demo::NativeDocumentDemo, run_native};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = NativeDocumentDemo::new();
    if let Some(frames) = exit_after_frames_from_env() {
        app = app.with_exit_after_frames(frames);
    }

    run_native(
        NativeOptions {
            title: "DES Native Document Demo".to_owned(),
            initial_width: 980,
            initial_height: 680,
            render_options: RenderOptions {
                clear_color: ClearColor::rgb(246, 241, 249),
                ..RenderOptions::default()
            },
            ..NativeOptions::default()
        },
        app,
    )?;
    Ok(())
}

fn exit_after_frames_from_env() -> Option<u32> {
    std::env::var("DES_NATIVE_DEMO_EXIT_AFTER_FRAMES")
        .ok()
        .and_then(|value| value.parse().ok())
}
