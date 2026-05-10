use des_layout_engine::floating::{
    compute_floating_position, detect_overflow, FloatingAlignment, FloatingArrow,
    FloatingAutoPlacement, FloatingAxisOffset, FloatingBoundary, FloatingFallbackAxisSideDirection,
    FloatingFallbackStrategy, FloatingFlip, FloatingFlipCrossAxis, FloatingHide,
    FloatingHideStrategy, FloatingInline, FloatingOptions, FloatingPadding, FloatingPlacement,
    FloatingRect, FloatingShift, FloatingShiftLimiter, FloatingSide, FloatingSize,
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
fn floating_position_can_derive_offset_from_rects() {
    let reference = rect(20.0, 30.0, 100.0, 80.0);
    let floating = Size {
        width: 50.0,
        height: 40.0,
    };

    let start = compute_floating_position(
        reference,
        floating,
        FloatingOptions::new(FloatingPlacement::TopStart)
            .alignment_axis_offset(FloatingAxisOffset::floating_width(-1.0)),
    );
    let end = compute_floating_position(
        reference,
        floating,
        FloatingOptions::new(FloatingPlacement::TopEnd)
            .alignment_axis_offset(FloatingAxisOffset::floating_width(-1.0)),
    );
    let centered = compute_floating_position(
        reference,
        floating,
        FloatingOptions::new(FloatingPlacement::Bottom).offset_axes(
            FloatingAxisOffset::reference_height(-0.5)
                .plus(FloatingAxisOffset::floating_height(-0.5)),
            FloatingAxisOffset::px(0.0),
        ),
    );

    assert_eq!(start.origin, Point { x: -30.0, y: -10.0 });
    assert_eq!(end.origin, Point { x: 120.0, y: -10.0 });
    assert_eq!(centered.origin, Point { x: 45.0, y: 50.0 });
}

#[test]
fn floating_position_can_center_on_both_axes() {
    let reference = rect(20.0, 30.0, 100.0, 80.0);
    let floating = Size {
        width: 50.0,
        height: 40.0,
    };

    let output = compute_floating_position(
        reference,
        floating,
        FloatingOptions::new(FloatingPlacement::Center),
    );

    assert_eq!(output.origin, Point { x: 45.0, y: 50.0 });
    assert_eq!(output.placement.opposite(), FloatingPlacement::Center);
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
fn floating_position_flips_alignment_when_cross_axis_overflows() {
    let reference = rect(78.0, 36.0, 16.0, 16.0);
    let floating = Size {
        width: 44.0,
        height: 24.0,
    };
    let boundary = FloatingBoundary::new(rect(0.0, 0.0, 100.0, 100.0));

    let output = compute_floating_position(
        reference,
        floating,
        FloatingOptions::new(FloatingPlacement::BottomStart)
            .boundary(boundary)
            .flip(true),
    );

    assert_eq!(output.placement, FloatingPlacement::BottomEnd);
    assert_eq!(output.origin, Point { x: 50.0, y: 52.0 });
}

#[test]
fn floating_position_flips_side_and_alignment_near_corner() {
    let reference = rect(78.0, 72.0, 16.0, 16.0);
    let floating = Size {
        width: 44.0,
        height: 24.0,
    };
    let boundary = FloatingBoundary::new(rect(0.0, 0.0, 100.0, 100.0));

    let output = compute_floating_position(
        reference,
        floating,
        FloatingOptions::new(FloatingPlacement::BottomStart)
            .boundary(boundary)
            .flip(true),
    );

    assert_eq!(output.placement, FloatingPlacement::TopEnd);
    assert_eq!(output.origin, Point { x: 50.0, y: 48.0 });
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
    assert_eq!(output.arrow.unwrap().center_offset, 5.0);
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

#[test]
fn floating_auto_placement_chooses_allowed_placement_with_most_space() {
    let reference = rect(84.0, 42.0, 12.0, 12.0);
    let floating = Size {
        width: 40.0,
        height: 24.0,
    };
    let boundary = FloatingBoundary::new(rect(0.0, 0.0, 100.0, 100.0));

    let output = compute_floating_position(
        reference,
        floating,
        FloatingOptions::new(FloatingPlacement::Right)
            .boundary(boundary)
            .auto_placement(
                FloatingAutoPlacement::new()
                    .allowed_placements([FloatingPlacement::Right, FloatingPlacement::Bottom])
                    .cross_axis(true),
            ),
    );

    assert_eq!(output.placement, FloatingPlacement::Bottom);
    assert_eq!(output.origin, Point { x: 70.0, y: 54.0 });
}

#[test]
fn floating_auto_placement_can_include_aligned_placements() {
    let reference = rect(78.0, 42.0, 12.0, 12.0);
    let floating = Size {
        width: 40.0,
        height: 24.0,
    };
    let boundary = FloatingBoundary::new(rect(0.0, 0.0, 100.0, 100.0));

    let output = compute_floating_position(
        reference,
        floating,
        FloatingOptions::new(FloatingPlacement::BottomStart)
            .boundary(boundary)
            .auto_placement(
                FloatingAutoPlacement::new()
                    .allowed_placements([
                        FloatingPlacement::BottomStart,
                        FloatingPlacement::BottomEnd,
                    ])
                    .alignment(FloatingAlignment::Start)
                    .auto_alignment(true),
            ),
    );

    assert_eq!(output.placement, FloatingPlacement::BottomEnd);
    assert_eq!(output.origin, Point { x: 50.0, y: 54.0 });
}

#[test]
fn floating_size_reports_and_applies_available_constraints() {
    let reference = rect(20.0, 70.0, 50.0, 12.0);
    let floating = Size {
        width: 80.0,
        height: 80.0,
    };
    let boundary = FloatingBoundary::new(rect(0.0, 0.0, 100.0, 100.0));

    let output = compute_floating_position(
        reference,
        floating,
        FloatingOptions::new(FloatingPlacement::BottomStart)
            .boundary(boundary)
            .size(
                FloatingSize::new()
                    .padding(FloatingPadding::all(4.0))
                    .max_width_to_available()
                    .max_height_to_available()
                    .match_reference_width(),
            ),
    );

    assert_eq!(
        output.size,
        Size {
            width: 50.0,
            height: 14.0
        }
    );
    assert_eq!(
        output.available_size,
        Size {
            width: 92.0,
            height: 14.0
        }
    );
}

#[test]
fn floating_size_reports_signed_available_space_before_clamping() {
    let reference = rect(20.0, 96.0, 50.0, 16.0);
    let floating = Size {
        width: 80.0,
        height: 20.0,
    };
    let boundary = FloatingBoundary::new(rect(0.0, 0.0, 100.0, 100.0));

    let output = compute_floating_position(
        reference,
        floating,
        FloatingOptions::new(FloatingPlacement::Bottom)
            .boundary(boundary)
            .size(FloatingSize::new().max_height_to_available()),
    );

    assert_eq!(output.available_size.height, -12.0);
    assert_eq!(output.size.height, 0.0);
}

#[test]
fn floating_hide_data_reports_each_strategy_independently() {
    let boundary = FloatingBoundary::new(rect(0.0, 0.0, 100.0, 100.0));

    let output = compute_floating_position(
        rect(140.0, 10.0, 20.0, 20.0),
        Size {
            width: 40.0,
            height: 24.0,
        },
        FloatingOptions::new(FloatingPlacement::Bottom)
            .boundary(boundary)
            .hide(FloatingHideStrategy::ReferenceHidden)
            .hide(FloatingHideStrategy::Escaped),
    );

    let hide = output.hide.unwrap();
    assert!(hide.reference_hidden);
    assert!(hide.escaped);
    assert_eq!(output.visibility, FloatingVisibility::ReferenceHidden);
}

#[test]
fn floating_inline_uses_point_selected_reference_fragment() {
    let reference = rect(10.0, 10.0, 120.0, 44.0);
    let floating = Size {
        width: 30.0,
        height: 16.0,
    };

    let output = compute_floating_position(
        reference,
        floating,
        FloatingOptions::new(FloatingPlacement::Top).inline(
            FloatingInline::new()
                .rects([rect(10.0, 10.0, 70.0, 18.0), rect(10.0, 36.0, 36.0, 18.0)])
                .point(Point { x: 20.0, y: 40.0 }),
        ),
    );

    assert_eq!(output.reference_rect, rect(10.0, 36.0, 36.0, 18.0));
    assert_eq!(output.origin, Point { x: 13.0, y: 20.0 });
}

#[test]
fn floating_shift_limiter_can_derive_offset_from_rects() {
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
            .shift(
                FloatingShift::main_and_cross_axis().limiter(
                    FloatingShiftLimiter::new()
                        .cross_axis(true)
                        .offset(FloatingAxisOffset::reference_width(0.5)),
                ),
            ),
    );

    assert_eq!(output.origin, Point { x: 68.0, y: 52.0 });
    assert_eq!(output.shift_offset, Some(Point { x: -6.0, y: 0.0 }));
}

#[test]
fn floating_shift_limiter_stops_when_opposite_edges_align() {
    let reference = rect(20.0, 20.0, 20.0, 20.0);
    let floating = Size {
        width: 50.0,
        height: 20.0,
    };
    let boundary = FloatingBoundary::new(rect(50.0, 0.0, 150.0, 100.0));

    let unlimited = compute_floating_position(
        reference,
        floating,
        FloatingOptions::new(FloatingPlacement::Bottom)
            .boundary(boundary)
            .shift(FloatingShift::new(false, true)),
    );
    let limited = compute_floating_position(
        reference,
        floating,
        FloatingOptions::new(FloatingPlacement::Bottom)
            .boundary(boundary)
            .shift(FloatingShift::new(false, true).limiter(FloatingShiftLimiter::new())),
    );

    assert_eq!(unlimited.origin.x, 50.0);
    assert_eq!(limited.origin.x, 40.0);
}

#[test]
fn floating_shift_padding_uses_padded_boundary() {
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
            .shift(FloatingShift::main_and_cross_axis().padding(FloatingPadding::all(6.0))),
    );

    assert_eq!(output.origin.x, 34.0);
}

#[test]
fn floating_flip_can_use_perpendicular_axis_fallbacks() {
    let reference = rect(60.0, 42.0, 16.0, 16.0);
    let floating = Size {
        width: 44.0,
        height: 80.0,
    };
    let boundary = FloatingBoundary::new(rect(0.0, 0.0, 150.0, 100.0));

    let output = compute_floating_position(
        reference,
        floating,
        FloatingOptions::new(FloatingPlacement::Bottom)
            .boundary(boundary)
            .flip_options(
                FloatingFlip::new()
                    .fallback_axis_side_direction(FloatingFallbackAxisSideDirection::Start),
            ),
    );

    assert_eq!(output.placement, FloatingPlacement::Left);
}

#[test]
fn floating_flip_perpendicular_fallbacks_try_both_sides_in_order() {
    let reference = rect(20.0, 10.0, 10.0, 10.0);
    let floating = Size {
        width: 40.0,
        height: 30.0,
    };
    let boundary = FloatingBoundary::new(rect(0.0, 0.0, 50.0, 100.0));

    let output = compute_floating_position(
        reference,
        floating,
        FloatingOptions::new(FloatingPlacement::Right)
            .boundary(boundary)
            .flip_options(
                FloatingFlip::new()
                    .fallback_axis_side_direction(FloatingFallbackAxisSideDirection::Start),
            ),
    );

    assert_eq!(output.placement, FloatingPlacement::Bottom);
}

#[test]
fn floating_flip_default_checks_cross_axis_overflow() {
    let reference = rect(82.0, 44.0, 12.0, 12.0);
    let floating = Size {
        width: 40.0,
        height: 20.0,
    };
    let boundary = FloatingBoundary::new(rect(0.0, 0.0, 100.0, 100.0));

    let output = compute_floating_position(
        reference,
        floating,
        FloatingOptions::new(FloatingPlacement::BottomStart)
            .boundary(boundary)
            .flip(true),
    );

    assert_eq!(output.placement, FloatingPlacement::BottomEnd);
}

#[test]
fn floating_flip_can_use_initial_placement_strategy() {
    let reference = rect(48.0, 48.0, 8.0, 8.0);
    let floating = Size {
        width: 120.0,
        height: 120.0,
    };
    let boundary = FloatingBoundary::new(rect(0.0, 0.0, 100.0, 100.0));

    let output = compute_floating_position(
        reference,
        floating,
        FloatingOptions::new(FloatingPlacement::Bottom)
            .boundary(boundary)
            .flip_options(
                FloatingFlip::new().fallback_strategy(FloatingFallbackStrategy::InitialPlacement),
            ),
    );

    assert_eq!(output.placement, FloatingPlacement::Bottom);
}

#[test]
fn floating_flip_can_ignore_cross_axis_overflow() {
    let reference = rect(82.0, 44.0, 12.0, 12.0);
    let floating = Size {
        width: 40.0,
        height: 20.0,
    };
    let boundary = FloatingBoundary::new(rect(0.0, 0.0, 100.0, 100.0));

    let output = compute_floating_position(
        reference,
        floating,
        FloatingOptions::new(FloatingPlacement::BottomStart)
            .boundary(boundary)
            .flip_options(FloatingFlip::new().cross_axis(FloatingFlipCrossAxis::None)),
    );

    assert_eq!(output.placement, FloatingPlacement::BottomStart);
}

#[test]
fn floating_hide_strategy_can_use_padding() {
    let boundary = FloatingBoundary::new(rect(0.0, 0.0, 100.0, 100.0));

    let output = compute_floating_position(
        rect(-3.0, 10.0, 5.0, 20.0),
        Size {
            width: 40.0,
            height: 24.0,
        },
        FloatingOptions::new(FloatingPlacement::Bottom)
            .boundary(boundary)
            .hide_options(
                FloatingHide::new(FloatingHideStrategy::ReferenceHidden)
                    .padding(FloatingPadding::all(4.0)),
            ),
    );

    assert!(output.hide.unwrap().reference_hidden);
}

#[test]
fn floating_arrow_uses_side_aware_padding() {
    let reference = rect(10.0, 40.0, 10.0, 20.0);
    let floating = Size {
        width: 60.0,
        height: 24.0,
    };

    let output = compute_floating_position(
        reference,
        floating,
        FloatingOptions::new(FloatingPlacement::BottomStart).arrow(
            FloatingArrow::new(Size {
                width: 10.0,
                height: 8.0,
            })
            .padding_sides(FloatingPadding {
                top: 0.0,
                right: 0.0,
                bottom: 0.0,
                left: 20.0,
            }),
        ),
    );

    assert_eq!(output.arrow_offset, Some(Point { x: 20.0, y: 0.0 }));
}

#[test]
fn floating_collision_strategy_builders_are_mutually_exclusive() {
    let flip_first = FloatingOptions::new(FloatingPlacement::Bottom)
        .flip(true)
        .auto_placement(FloatingAutoPlacement::new());
    let auto_first = FloatingOptions::new(FloatingPlacement::Bottom)
        .auto_placement(FloatingAutoPlacement::new())
        .flip_options(FloatingFlip::new());

    assert!(!flip_first.flip);
    assert!(flip_first.auto_placement.is_some());
    assert!(auto_first.flip);
    assert!(auto_first.auto_placement.is_none());
}
