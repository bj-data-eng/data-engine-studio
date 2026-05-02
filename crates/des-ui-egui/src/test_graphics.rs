use egui_kittest::Harness;
use image::RgbaImage;

pub(crate) const TEST_WIDTH: f32 = 1320.0;
pub(crate) const TEST_HEIGHT: f32 = 780.0;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct ImageStats {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) total_pixels: u64,
    pub(crate) non_transparent_pixels: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct ImageComparison {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) compared_pixels: u64,
    pub(crate) differing_pixels: u64,
    pub(crate) max_channel_delta: u8,
    pub(crate) mean_channel_delta: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct ImageTolerance {
    pub(crate) max_differing_pixels: u64,
    pub(crate) max_channel_delta: u8,
    pub(crate) max_mean_channel_delta: f32,
}

impl ImageTolerance {
    pub(crate) const EXACT: Self = Self {
        max_differing_pixels: 0,
        max_channel_delta: 0,
        max_mean_channel_delta: 0.0,
    };
}

pub(crate) fn test_harness<State>(
    state: State,
    render: impl FnMut(&mut egui::Ui, &mut State) + 'static,
) -> Harness<'static, State>
where
    State: 'static,
{
    Harness::builder()
        .with_size(egui::vec2(TEST_WIDTH, TEST_HEIGHT))
        .with_pixels_per_point(1.0)
        .with_step_dt(1.0 / 60.0)
        .with_max_steps(24)
        .wgpu()
        .build_ui_state(render, state)
}

pub(crate) fn render_harness<State>(harness: &mut Harness<'_, State>) -> RgbaImage {
    harness.run();
    harness.render().expect("render egui scene through kittest")
}

pub(crate) fn image_stats(image: &RgbaImage) -> ImageStats {
    ImageStats {
        width: image.width(),
        height: image.height(),
        total_pixels: u64::from(image.width()) * u64::from(image.height()),
        non_transparent_pixels: image
            .pixels()
            .filter(|pixel| pixel.0[3] > 0)
            .count()
            .try_into()
            .expect("pixel count fits u64"),
    }
}

pub(crate) fn compare_images(actual: &RgbaImage, expected: &RgbaImage) -> ImageComparison {
    assert_eq!(
        actual.dimensions(),
        expected.dimensions(),
        "image comparison requires matching dimensions"
    );

    let mut differing_pixels = 0_u64;
    let mut max_channel_delta = 0_u8;
    let mut channel_delta_sum = 0_u64;
    let mut channel_count = 0_u64;

    for (actual, expected) in actual.pixels().zip(expected.pixels()) {
        let mut pixel_differs = false;
        for channel_index in 0..4 {
            let delta = actual.0[channel_index].abs_diff(expected.0[channel_index]);
            pixel_differs |= delta > 0;
            max_channel_delta = max_channel_delta.max(delta);
            channel_delta_sum += u64::from(delta);
            channel_count += 1;
        }
        if pixel_differs {
            differing_pixels += 1;
        }
    }

    ImageComparison {
        width: actual.width(),
        height: actual.height(),
        compared_pixels: u64::from(actual.width()) * u64::from(actual.height()),
        differing_pixels,
        max_channel_delta,
        mean_channel_delta: channel_delta_sum as f32 / channel_count.max(1) as f32,
    }
}

pub(crate) fn assert_exact_image_match(actual: &RgbaImage, expected: &RgbaImage) {
    assert_image_within(actual, expected, ImageTolerance::EXACT);
}

pub(crate) fn assert_image_within(
    actual: &RgbaImage,
    expected: &RgbaImage,
    tolerance: ImageTolerance,
) {
    let comparison = compare_images(actual, expected);
    assert!(
        comparison.differing_pixels <= tolerance.max_differing_pixels
            && comparison.max_channel_delta <= tolerance.max_channel_delta
            && comparison.mean_channel_delta <= tolerance.max_mean_channel_delta,
        "image comparison exceeded tolerance: comparison={comparison:?}, tolerance={tolerance:?}"
    );
}
