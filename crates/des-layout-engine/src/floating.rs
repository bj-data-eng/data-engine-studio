//! Floating element placement math.
//!
//! This module computes positions for UI that floats relative to another
//! rectangle, such as context menus, popovers, dropdowns, and tooltips.

use crate::geometry::{Point, Size};
use crate::util::sys::{f32_max, f32_min};

/// A rectangle used by floating placement.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FloatingRect {
    /// The top-left origin of the rectangle.
    pub origin: Point<f32>,
    /// The width and height of the rectangle.
    pub size: Size<f32>,
}

impl FloatingRect {
    /// Creates a floating rectangle from an origin and size.
    #[must_use]
    pub const fn new(origin: Point<f32>, size: Size<f32>) -> Self {
        Self { origin, size }
    }

    /// Creates a floating rectangle from scalar components.
    #[must_use]
    pub const fn from_xy_size(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            origin: Point { x, y },
            size: Size { width, height },
        }
    }

    /// The x coordinate of the left edge.
    #[must_use]
    pub fn left(self) -> f32 {
        self.origin.x
    }

    /// The x coordinate of the right edge.
    #[must_use]
    pub fn right(self) -> f32 {
        self.origin.x + self.size.width
    }

    /// The y coordinate of the top edge.
    #[must_use]
    pub fn top(self) -> f32 {
        self.origin.y
    }

    /// The y coordinate of the bottom edge.
    #[must_use]
    pub fn bottom(self) -> f32 {
        self.origin.y + self.size.height
    }
}

/// The side of a reference rectangle used by a floating element.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum FloatingSide {
    /// Position above the reference rectangle.
    Top,
    /// Position to the right of the reference rectangle.
    Right,
    /// Position below the reference rectangle.
    Bottom,
    /// Position to the left of the reference rectangle.
    Left,
}

impl FloatingSide {
    /// Returns the opposite side.
    #[must_use]
    pub const fn opposite(self) -> Self {
        match self {
            Self::Top => Self::Bottom,
            Self::Right => Self::Left,
            Self::Bottom => Self::Top,
            Self::Left => Self::Right,
        }
    }

    /// Returns true when this is a vertical side.
    #[must_use]
    pub const fn is_vertical(self) -> bool {
        matches!(self, Self::Top | Self::Bottom)
    }
}

/// Alignment along the axis perpendicular to the floating side.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum FloatingAlignment {
    /// Align starting edges.
    Start,
    /// Align ending edges.
    End,
}

/// A side plus optional alignment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum FloatingPlacement {
    /// Center above the reference.
    Top,
    /// Start-aligned above the reference.
    TopStart,
    /// End-aligned above the reference.
    TopEnd,
    /// Center to the right of the reference.
    Right,
    /// Start-aligned to the right of the reference.
    RightStart,
    /// End-aligned to the right of the reference.
    RightEnd,
    /// Center below the reference.
    Bottom,
    /// Start-aligned below the reference.
    BottomStart,
    /// End-aligned below the reference.
    BottomEnd,
    /// Center to the left of the reference.
    Left,
    /// Start-aligned to the left of the reference.
    LeftStart,
    /// End-aligned to the left of the reference.
    LeftEnd,
}

impl FloatingPlacement {
    /// Returns the side component of the placement.
    #[must_use]
    pub const fn side(self) -> FloatingSide {
        match self {
            Self::Top | Self::TopStart | Self::TopEnd => FloatingSide::Top,
            Self::Right | Self::RightStart | Self::RightEnd => FloatingSide::Right,
            Self::Bottom | Self::BottomStart | Self::BottomEnd => FloatingSide::Bottom,
            Self::Left | Self::LeftStart | Self::LeftEnd => FloatingSide::Left,
        }
    }

    /// Returns the alignment component of the placement.
    #[must_use]
    pub const fn alignment(self) -> Option<FloatingAlignment> {
        match self {
            Self::TopStart | Self::RightStart | Self::BottomStart | Self::LeftStart => {
                Some(FloatingAlignment::Start)
            }
            Self::TopEnd | Self::RightEnd | Self::BottomEnd | Self::LeftEnd => {
                Some(FloatingAlignment::End)
            }
            _ => None,
        }
    }

    /// Returns the same alignment on the opposite side.
    #[must_use]
    pub const fn opposite(self) -> Self {
        Self::from_side_alignment(self.side().opposite(), self.alignment())
    }

    /// Creates a placement from side and optional alignment.
    #[must_use]
    pub const fn from_side_alignment(
        side: FloatingSide,
        alignment: Option<FloatingAlignment>,
    ) -> Self {
        match (side, alignment) {
            (FloatingSide::Top, None) => Self::Top,
            (FloatingSide::Top, Some(FloatingAlignment::Start)) => Self::TopStart,
            (FloatingSide::Top, Some(FloatingAlignment::End)) => Self::TopEnd,
            (FloatingSide::Right, None) => Self::Right,
            (FloatingSide::Right, Some(FloatingAlignment::Start)) => Self::RightStart,
            (FloatingSide::Right, Some(FloatingAlignment::End)) => Self::RightEnd,
            (FloatingSide::Bottom, None) => Self::Bottom,
            (FloatingSide::Bottom, Some(FloatingAlignment::Start)) => Self::BottomStart,
            (FloatingSide::Bottom, Some(FloatingAlignment::End)) => Self::BottomEnd,
            (FloatingSide::Left, None) => Self::Left,
            (FloatingSide::Left, Some(FloatingAlignment::Start)) => Self::LeftStart,
            (FloatingSide::Left, Some(FloatingAlignment::End)) => Self::LeftEnd,
        }
    }
}

/// Extra distance applied after the base floating placement.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FloatingOffset {
    /// Distance away from the placement side.
    pub main_axis: f32,
    /// Distance along the perpendicular alignment axis.
    pub cross_axis: f32,
}

impl FloatingOffset {
    /// Creates a new offset.
    #[must_use]
    pub const fn new(main_axis: f32, cross_axis: f32) -> Self {
        Self {
            main_axis,
            cross_axis,
        }
    }
}

/// Boundary padding for overflow detection.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FloatingPadding {
    /// Top padding.
    pub top: f32,
    /// Right padding.
    pub right: f32,
    /// Bottom padding.
    pub bottom: f32,
    /// Left padding.
    pub left: f32,
}

impl FloatingPadding {
    /// Creates equal padding on every side.
    #[must_use]
    pub const fn all(value: f32) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }
}

/// A clipping boundary for floating placement.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FloatingBoundary {
    /// Boundary rectangle.
    pub rect: FloatingRect,
    /// Padding applied inside the boundary.
    pub padding: FloatingPadding,
}

impl FloatingBoundary {
    /// Creates a boundary with no padding.
    #[must_use]
    pub const fn new(rect: FloatingRect) -> Self {
        Self {
            rect,
            padding: FloatingPadding {
                top: 0.0,
                right: 0.0,
                bottom: 0.0,
                left: 0.0,
            },
        }
    }

    /// Sets boundary padding.
    #[must_use]
    pub const fn padding(mut self, padding: FloatingPadding) -> Self {
        self.padding = padding;
        self
    }
}

/// Signed overflow distances from a boundary.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FloatingOverflow {
    /// Positive when the top edge overflows the boundary.
    pub top: f32,
    /// Positive when the right edge overflows the boundary.
    pub right: f32,
    /// Positive when the bottom edge overflows the boundary.
    pub bottom: f32,
    /// Positive when the left edge overflows the boundary.
    pub left: f32,
}

impl FloatingOverflow {
    /// Returns true if any side overflows.
    #[must_use]
    pub fn has_overflow(self) -> bool {
        self.top > 0.0 || self.right > 0.0 || self.bottom > 0.0 || self.left > 0.0
    }

    /// Returns the overflow amount for the provided side.
    #[must_use]
    pub const fn side(self, side: FloatingSide) -> f32 {
        match side {
            FloatingSide::Top => self.top,
            FloatingSide::Right => self.right,
            FloatingSide::Bottom => self.bottom,
            FloatingSide::Left => self.left,
        }
    }
}

/// Shift behavior after placement is computed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FloatingShift {
    /// Shift along the main placement axis.
    pub main_axis: bool,
    /// Shift along the cross placement axis.
    pub cross_axis: bool,
}

impl FloatingShift {
    /// Creates a shift configuration.
    #[must_use]
    pub const fn new(main_axis: bool, cross_axis: bool) -> Self {
        Self {
            main_axis,
            cross_axis,
        }
    }

    /// Enables both main-axis and cross-axis shifting.
    #[must_use]
    pub const fn main_and_cross_axis() -> Self {
        Self::new(true, true)
    }
}

/// Options for floating placement.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FloatingOptions {
    /// Preferred placement.
    pub placement: FloatingPlacement,
    /// Optional offset from the preferred placement.
    pub offset: FloatingOffset,
    /// Optional clipping boundary.
    pub boundary: Option<FloatingBoundary>,
    /// Whether to try the opposite side if the preferred side overflows.
    pub flip: bool,
    /// Optional shift behavior inside the clipping boundary.
    pub shift: Option<FloatingShift>,
    /// Whether start/end alignment should invert in right-to-left layout for vertical sides.
    pub rtl: bool,
}

impl FloatingOptions {
    /// Creates options with a preferred placement.
    #[must_use]
    pub const fn new(placement: FloatingPlacement) -> Self {
        Self {
            placement,
            offset: FloatingOffset {
                main_axis: 0.0,
                cross_axis: 0.0,
            },
            boundary: None,
            flip: false,
            shift: None,
            rtl: false,
        }
    }

    /// Sets the placement offset.
    #[must_use]
    pub const fn offset(mut self, main_axis: f32, cross_axis: f32) -> Self {
        self.offset = FloatingOffset::new(main_axis, cross_axis);
        self
    }

    /// Sets the clipping boundary.
    #[must_use]
    pub const fn boundary(mut self, boundary: FloatingBoundary) -> Self {
        self.boundary = Some(boundary);
        self
    }

    /// Enables or disables flipping.
    #[must_use]
    pub const fn flip(mut self, flip: bool) -> Self {
        self.flip = flip;
        self
    }

    /// Sets shift behavior.
    #[must_use]
    pub const fn shift(mut self, shift: FloatingShift) -> Self {
        self.shift = Some(shift);
        self
    }

    /// Sets right-to-left alignment behavior.
    #[must_use]
    pub const fn rtl(mut self, rtl: bool) -> Self {
        self.rtl = rtl;
        self
    }
}

/// Result of floating placement.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FloatingPosition {
    /// Resolved origin of the floating rectangle.
    pub origin: Point<f32>,
    /// Final placement after optional flipping.
    pub placement: FloatingPlacement,
    /// Final rectangle after offset and optional shifting.
    pub rect: FloatingRect,
    /// Overflow after final placement if a boundary was provided.
    pub overflow: Option<FloatingOverflow>,
}

/// Computes the base coordinates for a floating element.
#[must_use]
pub fn compute_coords_from_placement(
    reference: FloatingRect,
    floating: Size<f32>,
    placement: FloatingPlacement,
    rtl: bool,
) -> Point<f32> {
    let common_x = reference.left() + reference.size.width / 2.0 - floating.width / 2.0;
    let common_y = reference.top() + reference.size.height / 2.0 - floating.height / 2.0;
    let common_align = if placement.side().is_vertical() {
        reference.size.width / 2.0 - floating.width / 2.0
    } else {
        reference.size.height / 2.0 - floating.height / 2.0
    };

    let mut origin = match placement.side() {
        FloatingSide::Top => Point {
            x: common_x,
            y: reference.top() - floating.height,
        },
        FloatingSide::Right => Point {
            x: reference.right(),
            y: common_y,
        },
        FloatingSide::Bottom => Point {
            x: common_x,
            y: reference.bottom(),
        },
        FloatingSide::Left => Point {
            x: reference.left() - floating.width,
            y: common_y,
        },
    };

    let rtl_multiplier = if rtl && placement.side().is_vertical() {
        -1.0
    } else {
        1.0
    };
    match placement.alignment() {
        Some(FloatingAlignment::Start) => update_alignment_axis(
            &mut origin,
            placement.side(),
            -common_align * rtl_multiplier,
        ),
        Some(FloatingAlignment::End) => {
            update_alignment_axis(&mut origin, placement.side(), common_align * rtl_multiplier)
        }
        None => {}
    }

    origin
}

/// Detects signed overflow from a boundary.
#[must_use]
pub fn detect_overflow(
    floating: FloatingRect,
    boundary: FloatingBoundary,
    padding: FloatingPadding,
) -> FloatingOverflow {
    let padding = FloatingPadding {
        top: boundary.padding.top + padding.top,
        right: boundary.padding.right + padding.right,
        bottom: boundary.padding.bottom + padding.bottom,
        left: boundary.padding.left + padding.left,
    };
    FloatingOverflow {
        top: boundary.rect.top() - floating.top() + padding.top,
        right: floating.right() - boundary.rect.right() + padding.right,
        bottom: floating.bottom() - boundary.rect.bottom() + padding.bottom,
        left: boundary.rect.left() - floating.left() + padding.left,
    }
}

/// Computes final floating coordinates from reference geometry and options.
#[must_use]
pub fn compute_floating_position(
    reference: FloatingRect,
    floating: Size<f32>,
    options: FloatingOptions,
) -> FloatingPosition {
    let mut placement = options.placement;
    let mut origin =
        placed_origin_with_offset(reference, floating, placement, options.offset, options.rtl);

    if options.flip {
        if let Some(boundary) = options.boundary {
            let overflow = detect_overflow(
                FloatingRect::new(origin, floating),
                boundary,
                FloatingPadding::default(),
            );
            if overflow.side(placement.side()) > 0.0 {
                let opposite = placement.opposite();
                let opposite_origin = placed_origin_with_offset(
                    reference,
                    floating,
                    opposite,
                    options.offset,
                    options.rtl,
                );
                let opposite_overflow = detect_overflow(
                    FloatingRect::new(opposite_origin, floating),
                    boundary,
                    FloatingPadding::default(),
                );
                if opposite_overflow.side(opposite.side()) <= overflow.side(placement.side()) {
                    placement = opposite;
                    origin = opposite_origin;
                }
            }
        }
    }

    if let (Some(boundary), Some(shift)) = (options.boundary, options.shift) {
        origin = shift_origin_into_boundary(origin, floating, placement.side(), boundary, shift);
    }

    let rect = FloatingRect::new(origin, floating);
    let overflow = options
        .boundary
        .map(|boundary| detect_overflow(rect, boundary, FloatingPadding::default()));
    FloatingPosition {
        origin,
        placement,
        rect,
        overflow,
    }
}

/// Applies a floating offset to an already placed origin.
#[must_use]
pub fn apply_offset(
    mut origin: Point<f32>,
    side: FloatingSide,
    offset: FloatingOffset,
) -> Point<f32> {
    match side {
        FloatingSide::Top => {
            origin.y -= offset.main_axis;
            origin.x += offset.cross_axis;
        }
        FloatingSide::Right => {
            origin.x += offset.main_axis;
            origin.y += offset.cross_axis;
        }
        FloatingSide::Bottom => {
            origin.y += offset.main_axis;
            origin.x += offset.cross_axis;
        }
        FloatingSide::Left => {
            origin.x -= offset.main_axis;
            origin.y += offset.cross_axis;
        }
    }
    origin
}

/// Shifts an origin inside a clipping boundary.
#[must_use]
pub fn shift_origin_into_boundary(
    mut origin: Point<f32>,
    floating: Size<f32>,
    side: FloatingSide,
    boundary: FloatingBoundary,
    shift: FloatingShift,
) -> Point<f32> {
    let min_x = boundary.rect.left() + boundary.padding.left;
    let max_x = boundary.rect.right() - boundary.padding.right - floating.width;
    let min_y = boundary.rect.top() + boundary.padding.top;
    let max_y = boundary.rect.bottom() - boundary.padding.bottom - floating.height;

    let clamp_x =
        (side.is_vertical() && shift.cross_axis) || (!side.is_vertical() && shift.main_axis);
    let clamp_y =
        (side.is_vertical() && shift.main_axis) || (!side.is_vertical() && shift.cross_axis);

    if clamp_x {
        origin.x = clamp(origin.x, min_x, max_x);
    }
    if clamp_y {
        origin.y = clamp(origin.y, min_y, max_y);
    }
    origin
}

fn placed_origin_with_offset(
    reference: FloatingRect,
    floating: Size<f32>,
    placement: FloatingPlacement,
    offset: FloatingOffset,
    rtl: bool,
) -> Point<f32> {
    apply_offset(
        compute_coords_from_placement(reference, floating, placement, rtl),
        placement.side(),
        offset,
    )
}

fn update_alignment_axis(origin: &mut Point<f32>, side: FloatingSide, delta: f32) {
    if side.is_vertical() {
        origin.x += delta;
    } else {
        origin.y += delta;
    }
}

fn clamp(value: f32, min: f32, max: f32) -> f32 {
    if min > max {
        return min;
    }
    f32_min(f32_max(value, min), max)
}
