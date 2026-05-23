use des_ui_wgpu::{ClearColor, RenderAlphaFromCoverage, RenderOptions, RenderTextOptions};
use des_ui_window::{NativeOptions, run_native, text_rendering_demo::NativeTextRenderingDemo};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let text_coverage = text_coverage_from_env();
    let mut app = NativeTextRenderingDemo::new();
    if let Some(frames) = exit_after_frames_from_env() {
        app = app.with_exit_after_frames(frames);
    }

    run_native(
        NativeOptions {
            title: format!(
                "DES Text Rendering Specimen ({})",
                coverage_label(text_coverage)
            ),
            initial_width: 1180,
            initial_height: 820,
            render_options: RenderOptions {
                clear_color: ClearColor::rgb(246, 241, 249),
                text: RenderTextOptions {
                    alpha_from_coverage: text_coverage,
                    ..RenderTextOptions::default()
                },
                ..RenderOptions::default()
            },
            ..NativeOptions::default()
        },
        app,
    )?;
    Ok(())
}

fn text_coverage_from_env() -> RenderAlphaFromCoverage {
    match std::env::var("DES_TEXT_COVERAGE")
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "linear" | "light" => RenderAlphaFromCoverage::Linear,
        "gamma-075" | "gamma_075" | "gamma" => RenderAlphaFromCoverage::Gamma(0.75),
        "dark" | "default" | "" => RenderAlphaFromCoverage::default(),
        _ => RenderAlphaFromCoverage::default(),
    }
}

fn coverage_label(coverage: RenderAlphaFromCoverage) -> &'static str {
    match coverage {
        RenderAlphaFromCoverage::Linear => "linear coverage",
        RenderAlphaFromCoverage::Gamma(_) => "gamma coverage",
        RenderAlphaFromCoverage::TwoCoverageMinusCoverageSq => "default coverage",
    }
}

fn exit_after_frames_from_env() -> Option<u32> {
    std::env::var("DES_NATIVE_DEMO_EXIT_AFTER_FRAMES")
        .ok()
        .and_then(|value| value.parse().ok())
}
