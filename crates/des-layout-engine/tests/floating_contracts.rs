use des_layout_engine::floating::{
    compute_floating_position, detect_overflow, FloatingArrow, FloatingBoundary, FloatingOptions,
    FloatingPadding, FloatingPlacement, FloatingRect, FloatingShift, FloatingSide,
    FloatingVisibility,
};
use des_layout_engine::geometry::{Point, Size};

fn rect(x: f32, y: f32, width: f32, height: f32) -> FloatingRect {
    FloatingRect::new(Point { x, y }, Size { width, height })
}

#[test]
fn floating_position_matches_basic_placements() {
    let reference = rect(20.0, 30.0, 100.0, 80.0);
    let floating = Size {
        width: 50.0,
        height: 40.0,
    };

    assert_eq!(
        compute_floating_position(
            reference,
            floating,
            FloatingOptions::new(FloatingPlacement::Top)
        )
        .origin,
        Point { x: 45.0, y: -10.0 }
    );
    assert_eq!(
        compute_floating_position(
            reference,
            floating,
            FloatingOptions::new(FloatingPlacement::BottomStart)
        )
        .origin,
        Point { x: 20.0, y: 110.0 }
    );
    assert_eq!(
        compute_floating_position(
            reference,
            floating,
            FloatingOptions::new(FloatingPlacement::RightEnd)
        )
        .origin,
        Point { x: 120.0, y: 70.0 }
    );
}

#[test]
fn floating_position_applies_main_and_cross_axis_offsets() {
    let reference = rect(20.0, 30.0, 100.0, 80.0);
    let floating = Size {
        width: 50.0,
        height: 40.0,
    };

    let output = compute_floating_position(
        reference,
        floating,
        FloatingOptions::new(FloatingPlacement::BottomStart).offset(8.0, 3.0),
    );

    assert_eq!(output.origin, Point { x: 23.0, y: 118.0 });
}

#[test]
fn floating_position_applies_alignment_axis_to_aligned_placements() {
    let reference = rect(20.0, 30.0, 100.0, 80.0);
    let floating = Size {
        width: 50.0,
        height: 40.0,
    };

    let start = compute_floating_position(
        reference,
        floating,
        FloatingOptions::new(FloatingPlacement::TopStart).alignment_axis(12.0),
    );
    let end = compute_floating_position(
        reference,
        floating,
        FloatingOptions::new(FloatingPlacement::TopEnd).alignment_axis(12.0),
    );
    let centered = compute_floating_position(
        reference,
        floating,
        FloatingOptions::new(FloatingPlacement::Top).alignment_axis(12.0),
    );

    assert_eq!(start.origin, Point { x: 32.0, y: -10.0 });
    assert_eq!(end.origin, Point { x: 58.0, y: -10.0 });
    assert_eq!(centered.origin, Point { x: 45.0, y: -10.0 });
}

#[test]
fn floating_overflow_reports_signed_distance_from_boundary() {
    let boundary = FloatingBoundary::new(rect(0.0, 0.0, 100.0, 80.0));
    let overflow = detect_overflow(
        rect(70.0, -5.0, 40.0, 50.0),
        boundary,
        FloatingPadding::all(4.0),
    );

    assert_eq!(overflow.top, 9.0);
    assert_eq!(overflow.right, 14.0);
    assert_eq!(overflow.bottom, -31.0);
    assert_eq!(overflow.left, -66.0);
}

#[test]
fn floating_position_flips_when_preferred_side_overflows() {
    let reference = rect(40.0, 4.0, 20.0, 20.0);
    let floating = Size {
        width: 40.0,
        height: 30.0,
    };
    let boundary = FloatingBoundary::new(rect(0.0, 0.0, 100.0, 100.0));

    let output = compute_floating_position(
        reference,
        floating,
        FloatingOptions::new(FloatingPlacement::Top)
            .boundary(boundary)
            .flip(true),
    );

    assert_eq!(output.placement, FloatingPlacement::Bottom);
    assert_eq!(output.origin, Point { x: 30.0, y: 24.0 });
}

#[test]
fn floating_position_shifts_into_boundary_without_changing_side() {
    let reference = rect(102.0, 40.0, 12.0, 12.0);
    let floating = Size {
        width: 40.0,
        height: 20.0,
    };
    let boundary = FloatingBoundary::new(rect(0.0, 0.0, 100.0, 100.0));

    let output = compute_floating_position(
        reference,
        floating,
        FloatingOptions::new(FloatingPlacement::BottomEnd)
            .boundary(boundary)
            .shift(FloatingShift::main_and_cross_axis()),
    );

    assert_eq!(output.placement.side(), FloatingSide::Bottom);
    assert_eq!(output.origin, Point { x: 60.0, y: 52.0 });
}

#[test]
fn floating_position_uses_first_fallback_that_fits() {
    let reference = rect(84.0, 40.0, 12.0, 12.0);
    let floating = Size {
        width: 34.0,
        height: 24.0,
    };
    let boundary = FloatingBoundary::new(rect(0.0, 0.0, 100.0, 100.0));

    let output = compute_floating_position(
        reference,
        floating,
        FloatingOptions::new(FloatingPlacement::Right)
            .boundary(boundary)
            .fallbacks([FloatingPlacement::Bottom, FloatingPlacement::Left]),
    );

    assert_eq!(output.placement, FloatingPlacement::Left);
    assert_eq!(output.origin, Point { x: 50.0, y: 34.0 });
}

#[test]
fn floating_position_reports_available_size_for_final_placement() {
    let reference = rect(40.0, 40.0, 20.0, 20.0);
    let floating = Size {
        width: 40.0,
        height: 90.0,
    };
    let boundary = FloatingBoundary::new(rect(0.0, 0.0, 100.0, 100.0));

    let output = compute_floating_position(
        reference,
        floating,
        FloatingOptions::new(FloatingPlacement::Bottom).boundary(boundary),
    );

    assert_eq!(
        output.available_size,
        Size {
            width: 100.0,
            height: 40.0
        }
    );
}

#[test]
fn floating_position_reports_arrow_offset_clamped_to_floating_box() {
    let reference = rect(90.0, 40.0, 20.0, 20.0);
    let floating = Size {
        width: 60.0,
        height: 24.0,
    };
    let boundary = FloatingBoundary::new(rect(0.0, 0.0, 100.0, 100.0));

    let output = compute_floating_position(
        reference,
        floating,
        FloatingOptions::new(FloatingPlacement::Bottom)
            .boundary(boundary)
            .shift(FloatingShift::main_and_cross_axis())
            .arrow(FloatingArrow::new(Size {
                width: 10.0,
                height: 8.0,
            })),
    );

    assert_eq!(output.origin, Point { x: 40.0, y: 60.0 });
    assert_eq!(output.arrow_offset, Some(Point { x: 50.0, y: 0.0 }));
}

#[test]
fn floating_position_reports_visibility_state() {
    let boundary = FloatingBoundary::new(rect(0.0, 0.0, 100.0, 100.0));

    let hidden = compute_floating_position(
        rect(140.0, 10.0, 20.0, 20.0),
        Size {
            width: 40.0,
            height: 24.0,
        },
        FloatingOptions::new(FloatingPlacement::Bottom).boundary(boundary),
    );
    assert_eq!(hidden.visibility, FloatingVisibility::ReferenceHidden);

    let escaped = compute_floating_position(
        rect(20.0, 20.0, 20.0, 20.0),
        Size {
            width: 120.0,
            height: 24.0,
        },
        FloatingOptions::new(FloatingPlacement::BottomStart).boundary(boundary),
    );
    assert_eq!(escaped.visibility, FloatingVisibility::FloatingEscaped);
}

#[test]
fn floating_position_can_limit_cross_axis_shift() {
    let reference = rect(102.0, 40.0, 12.0, 12.0);
    let floating = Size {
        width: 40.0,
        height: 20.0,
    };
    let boundary = FloatingBoundary::new(rect(0.0, 0.0, 100.0, 100.0));

    let output = compute_floating_position(
        reference,
        floating,
        FloatingOptions::new(FloatingPlacement::BottomEnd)
            .boundary(boundary)
            .shift(FloatingShift::main_and_cross_axis().limit_cross_axis(10.0)),
    );

    assert_eq!(output.origin, Point { x: 64.0, y: 52.0 });
    assert_eq!(output.overflow.unwrap().right, 4.0);
}
