//! Floating element placement math.
//!
//! This module computes positions for UI that floats relative to another
//! rectangle, such as context menus, popovers, dropdowns, and tooltips.

use crate::geometry::{Point, Size};
use crate::util::sys::{f32_max, f32_min, Vec};

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

    /// Returns true when the point is inside the rectangle.
    #[must_use]
    pub fn contains_point(self, point: Point<f32>) -> bool {
        point.x >= self.left()
            && point.x <= self.right()
            && point.y >= self.top()
            && point.y <= self.bottom()
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

impl FloatingAlignment {
    /// Returns the opposite cross-axis alignment.
    #[must_use]
    pub const fn opposite(self) -> Self {
        match self {
            Self::Start => Self::End,
            Self::End => Self::Start,
        }
    }
}

/// A side plus optional alignment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum FloatingPlacement {
    /// Center inside the reference.
    Center,
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

/// A rect-aware floating offset value.
///
/// This mirrors Floating UI's function offset shape for layout-stable values
/// that can be serialized and styled: a pixel component plus linear terms
/// derived from the reference and floating rect sizes.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FloatingAxisOffset {
    /// Fixed pixel component.
    pub px: f32,
    /// Multiplier for the reference width.
    pub reference_width: f32,
    /// Multiplier for the reference height.
    pub reference_height: f32,
    /// Multiplier for the floating width.
    pub floating_width: f32,
    /// Multiplier for the floating height.
    pub floating_height: f32,
}

impl FloatingAxisOffset {
    /// Creates a fixed pixel offset.
    #[must_use]
    pub const fn px(px: f32) -> Self {
        Self {
            px,
            reference_width: 0.0,
            reference_height: 0.0,
            floating_width: 0.0,
            floating_height: 0.0,
        }
    }

    /// Creates an offset from the reference width.
    #[must_use]
    pub const fn reference_width(factor: f32) -> Self {
        Self {
            px: 0.0,
            reference_width: factor,
            reference_height: 0.0,
            floating_width: 0.0,
            floating_height: 0.0,
        }
    }

    /// Creates an offset from the reference height.
    #[must_use]
    pub const fn reference_height(factor: f32) -> Self {
        Self {
            px: 0.0,
            reference_width: 0.0,
            reference_height: factor,
            floating_width: 0.0,
            floating_height: 0.0,
        }
    }

    /// Creates an offset from the floating width.
    #[must_use]
    pub const fn floating_width(factor: f32) -> Self {
        Self {
            px: 0.0,
            reference_width: 0.0,
            reference_height: 0.0,
            floating_width: factor,
            floating_height: 0.0,
        }
    }

    /// Creates an offset from the floating height.
    #[must_use]
    pub const fn floating_height(factor: f32) -> Self {
        Self {
            px: 0.0,
            reference_width: 0.0,
            reference_height: 0.0,
            floating_width: 0.0,
            floating_height: factor,
        }
    }

    /// Adds another offset term.
    #[must_use]
    pub const fn plus(mut self, other: Self) -> Self {
        self.px += other.px;
        self.reference_width += other.reference_width;
        self.reference_height += other.reference_height;
        self.floating_width += other.floating_width;
        self.floating_height += other.floating_height;
        self
    }

    fn resolve(self, reference: FloatingRect, floating: Size<f32>) -> f32 {
        self.px
            + self.reference_width * reference.size.width
            + self.reference_height * reference.size.height
            + self.floating_width * floating.width
            + self.floating_height * floating.height
    }
}

impl FloatingPlacement {
    /// Returns the side component of the placement.
    #[must_use]
    pub const fn side(self) -> FloatingSide {
        match self {
            Self::Center => FloatingSide::Top,
            Self::Top | Self::TopStart | Self::TopEnd => FloatingSide::Top,
            Self::Right | Self::RightStart | Self::RightEnd => FloatingSide::Right,
            Self::Bottom | Self::BottomStart | Self::BottomEnd => FloatingSide::Bottom,
            Self::Left | Self::LeftStart | Self::LeftEnd => FloatingSide::Left,
        }
    }

    /// Returns true when placement is centered inside the reference.
    #[must_use]
    pub const fn is_center(self) -> bool {
        matches!(self, Self::Center)
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
        if self.is_center() {
            Self::Center
        } else {
            Self::from_side_alignment(self.side().opposite(), self.alignment())
        }
    }

    /// Returns the same side with the opposite start/end alignment.
    #[must_use]
    pub const fn opposite_alignment(self) -> Self {
        if self.is_center() {
            Self::Center
        } else {
            Self::from_side_alignment(
                self.side(),
                match self.alignment() {
                    Some(alignment) => Some(alignment.opposite()),
                    None => None,
                },
            )
        }
    }

    /// Returns the opposite side with the opposite start/end alignment.
    #[must_use]
    pub const fn opposite_side_and_alignment(self) -> Self {
        if self.is_center() {
            Self::Center
        } else {
            Self::from_side_alignment(
                self.side().opposite(),
                match self.alignment() {
                    Some(alignment) => Some(alignment.opposite()),
                    None => None,
                },
            )
        }
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
    pub main_axis: FloatingAxisOffset,
    /// Distance along the perpendicular alignment axis.
    pub cross_axis: FloatingAxisOffset,
    /// Distance along the alignment axis for start/end placements.
    ///
    /// When set, this overrides `cross_axis` for aligned placements and reverses
    /// direction for end alignment.
    pub alignment_axis: Option<FloatingAxisOffset>,
}

impl FloatingOffset {
    /// Creates a new offset.
    #[must_use]
    pub const fn new(main_axis: f32, cross_axis: f32) -> Self {
        Self {
            main_axis: FloatingAxisOffset::px(main_axis),
            cross_axis: FloatingAxisOffset::px(cross_axis),
            alignment_axis: None,
        }
    }

    /// Creates an offset from explicit rect-aware axis values.
    #[must_use]
    pub const fn from_axes(main_axis: FloatingAxisOffset, cross_axis: FloatingAxisOffset) -> Self {
        Self {
            main_axis,
            cross_axis,
            alignment_axis: None,
        }
    }

    /// Sets the aligned placement offset.
    #[must_use]
    pub const fn alignment_axis(mut self, alignment_axis: f32) -> Self {
        self.alignment_axis = Some(FloatingAxisOffset::px(alignment_axis));
        self
    }

    /// Sets a rect-aware aligned placement offset.
    #[must_use]
    pub const fn alignment_axis_offset(mut self, alignment_axis: FloatingAxisOffset) -> Self {
        self.alignment_axis = Some(alignment_axis);
        self
    }
}

/// Cross-axis overflow checking for flip.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum FloatingFlipCrossAxis {
    /// Ignore cross-axis overflow.
    None,
    /// Check cross-axis overflow only for aligned placements.
    Alignment,
    /// Check cross-axis overflow for all placements.
    All,
}

/// Perpendicular fallback side direction for flip.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum FloatingFallbackAxisSideDirection {
    /// Do not use perpendicular fallbacks.
    None,
    /// Use the logical start side.
    Start,
    /// Use the logical end side.
    End,
}

/// Fallback scoring strategy for flip.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum FloatingFallbackStrategy {
    /// Use the placement with the least overflow if none fit.
    BestFit,
    /// Keep the initial placement if none fit.
    InitialPlacement,
}

/// Flip behavior.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FloatingFlip {
    /// Whether to check overflow on the main placement side.
    pub main_axis: bool,
    /// Whether to check cross-axis overflow.
    pub cross_axis: FloatingFlipCrossAxis,
    /// Optional perpendicular fallback direction.
    pub fallback_axis_side_direction: FloatingFallbackAxisSideDirection,
    /// Whether to try opposite alignment for aligned placements.
    pub flip_alignment: bool,
    /// Explicit fallback placements.
    pub fallback_placements: Vec<FloatingPlacement>,
    /// Fallback strategy when no placement fits.
    pub fallback_strategy: FloatingFallbackStrategy,
    /// Overflow padding.
    pub padding: FloatingPadding,
}

impl FloatingFlip {
    /// Creates flip behavior matching Floating UI defaults.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            main_axis: true,
            cross_axis: FloatingFlipCrossAxis::All,
            fallback_axis_side_direction: FloatingFallbackAxisSideDirection::None,
            flip_alignment: true,
            fallback_placements: Vec::new(),
            fallback_strategy: FloatingFallbackStrategy::BestFit,
            padding: FloatingPadding {
                top: 0.0,
                right: 0.0,
                bottom: 0.0,
                left: 0.0,
            },
        }
    }

    /// Enables or disables main-axis checking.
    #[must_use]
    pub const fn main_axis(mut self, main_axis: bool) -> Self {
        self.main_axis = main_axis;
        self
    }

    /// Sets cross-axis checking.
    #[must_use]
    pub const fn cross_axis(mut self, cross_axis: FloatingFlipCrossAxis) -> Self {
        self.cross_axis = cross_axis;
        self
    }

    /// Sets perpendicular fallback direction.
    #[must_use]
    pub const fn fallback_axis_side_direction(
        mut self,
        direction: FloatingFallbackAxisSideDirection,
    ) -> Self {
        self.fallback_axis_side_direction = direction;
        self
    }

    /// Enables or disables opposite-alignment fallback.
    #[must_use]
    pub const fn flip_alignment(mut self, flip_alignment: bool) -> Self {
        self.flip_alignment = flip_alignment;
        self
    }

    /// Sets explicit fallback placements.
    #[must_use]
    pub fn fallback_placements(mut self, placements: impl Into<Vec<FloatingPlacement>>) -> Self {
        self.fallback_placements = placements.into();
        self
    }

    /// Sets the fallback strategy.
    #[must_use]
    pub const fn fallback_strategy(mut self, strategy: FloatingFallbackStrategy) -> Self {
        self.fallback_strategy = strategy;
        self
    }

    /// Sets overflow padding.
    #[must_use]
    pub const fn padding(mut self, padding: FloatingPadding) -> Self {
        self.padding = padding;
        self
    }
}

impl Default for FloatingFlip {
    fn default() -> Self {
        Self::new()
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
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FloatingShift {
    /// Shift along the main placement axis.
    pub main_axis: bool,
    /// Shift along the cross placement axis.
    pub cross_axis: bool,
    /// Maximum distance allowed when shifting on the main axis.
    pub main_axis_limit: Option<f32>,
    /// Maximum distance allowed when shifting on the cross axis.
    pub cross_axis_limit: Option<f32>,
    /// Optional rect-aware limiter.
    pub limiter: Option<FloatingShiftLimiter>,
    /// Overflow padding used for boundary shifting.
    pub padding: FloatingPadding,
}

impl FloatingShift {
    /// Creates a shift configuration.
    #[must_use]
    pub const fn new(main_axis: bool, cross_axis: bool) -> Self {
        Self {
            main_axis,
            cross_axis,
            main_axis_limit: None,
            cross_axis_limit: None,
            limiter: None,
            padding: FloatingPadding {
                top: 0.0,
                right: 0.0,
                bottom: 0.0,
                left: 0.0,
            },
        }
    }

    /// Enables both main-axis and cross-axis shifting.
    #[must_use]
    pub const fn main_and_cross_axis() -> Self {
        Self::new(true, true)
    }

    /// Limits main-axis shift distance.
    #[must_use]
    pub const fn limit_main_axis(mut self, limit: f32) -> Self {
        self.main_axis_limit = Some(limit);
        self
    }

    /// Limits cross-axis shift distance.
    #[must_use]
    pub const fn limit_cross_axis(mut self, limit: f32) -> Self {
        self.cross_axis_limit = Some(limit);
        self
    }

    /// Sets a rect-aware limiter.
    #[must_use]
    pub const fn limiter(mut self, limiter: FloatingShiftLimiter) -> Self {
        self.limiter = Some(limiter);
        self
    }

    /// Sets boundary padding for shift overflow detection.
    #[must_use]
    pub const fn padding(mut self, padding: FloatingPadding) -> Self {
        self.padding = padding;
        self
    }
}

/// Configures when shift limiting starts.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FloatingShiftLimiter {
    /// Whether to limit main-axis shift.
    pub main_axis: bool,
    /// Whether to limit cross-axis shift.
    pub cross_axis: bool,
    /// Rect-aware shift limit.
    pub offset: FloatingAxisOffset,
}

impl FloatingShiftLimiter {
    /// Creates a limiter that limits both axes at the reference edge.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            main_axis: true,
            cross_axis: true,
            offset: FloatingAxisOffset::px(0.0),
        }
    }

    /// Enables or disables main-axis limiting.
    #[must_use]
    pub const fn main_axis(mut self, main_axis: bool) -> Self {
        self.main_axis = main_axis;
        self
    }

    /// Enables or disables cross-axis limiting.
    #[must_use]
    pub const fn cross_axis(mut self, cross_axis: bool) -> Self {
        self.cross_axis = cross_axis;
        self
    }

    /// Sets the rect-aware limit offset.
    #[must_use]
    pub const fn offset(mut self, offset: FloatingAxisOffset) -> Self {
        self.offset = offset;
        self
    }
}

impl Default for FloatingShiftLimiter {
    fn default() -> Self {
        Self::new()
    }
}

/// Automatic placement selection.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FloatingAutoPlacement {
    /// Whether cross-axis overflow contributes to the score.
    pub cross_axis: bool,
    /// Optional alignment subset to consider.
    pub alignment: Option<FloatingAlignment>,
    /// Whether the opposite alignment can be selected.
    pub auto_alignment: bool,
    /// Explicit placement allow-list.
    pub allowed_placements: Vec<FloatingPlacement>,
    /// Overflow padding.
    pub padding: FloatingPadding,
}

impl FloatingAutoPlacement {
    /// Creates an automatic placement configuration.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            cross_axis: false,
            alignment: None,
            auto_alignment: true,
            allowed_placements: Vec::new(),
            padding: FloatingPadding {
                top: 0.0,
                right: 0.0,
                bottom: 0.0,
                left: 0.0,
            },
        }
    }

    /// Enables or disables cross-axis scoring.
    #[must_use]
    pub const fn cross_axis(mut self, cross_axis: bool) -> Self {
        self.cross_axis = cross_axis;
        self
    }

    /// Sets the aligned placement subset.
    #[must_use]
    pub const fn alignment(mut self, alignment: FloatingAlignment) -> Self {
        self.alignment = Some(alignment);
        self
    }

    /// Enables or disables opposite-alignment selection.
    #[must_use]
    pub const fn auto_alignment(mut self, auto_alignment: bool) -> Self {
        self.auto_alignment = auto_alignment;
        self
    }

    /// Sets explicit allowed placements.
    #[must_use]
    pub fn allowed_placements(mut self, placements: impl Into<Vec<FloatingPlacement>>) -> Self {
        self.allowed_placements = placements.into();
        self
    }

    /// Sets overflow padding.
    #[must_use]
    pub const fn padding(mut self, padding: FloatingPadding) -> Self {
        self.padding = padding;
        self
    }
}

impl Default for FloatingAutoPlacement {
    fn default() -> Self {
        Self::new()
    }
}

/// Floating size data and sizing behavior.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FloatingSize {
    /// Overflow padding.
    pub padding: FloatingPadding,
    /// Clamp width to the available width.
    pub max_width_to_available: bool,
    /// Clamp height to the available height.
    pub max_height_to_available: bool,
    /// Match the reference width.
    pub match_reference_width: bool,
    /// Match the reference height.
    pub match_reference_height: bool,
}

impl FloatingSize {
    /// Creates a size data configuration.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            padding: FloatingPadding {
                top: 0.0,
                right: 0.0,
                bottom: 0.0,
                left: 0.0,
            },
            max_width_to_available: false,
            max_height_to_available: false,
            match_reference_width: false,
            match_reference_height: false,
        }
    }

    /// Sets overflow padding.
    #[must_use]
    pub const fn padding(mut self, padding: FloatingPadding) -> Self {
        self.padding = padding;
        self
    }

    /// Clamps width to the available width.
    #[must_use]
    pub const fn max_width_to_available(mut self) -> Self {
        self.max_width_to_available = true;
        self
    }

    /// Clamps height to the available height.
    #[must_use]
    pub const fn max_height_to_available(mut self) -> Self {
        self.max_height_to_available = true;
        self
    }

    /// Matches the reference width.
    #[must_use]
    pub const fn match_reference_width(mut self) -> Self {
        self.match_reference_width = true;
        self
    }

    /// Matches the reference height.
    #[must_use]
    pub const fn match_reference_height(mut self) -> Self {
        self.match_reference_height = true;
        self
    }
}

impl Default for FloatingSize {
    fn default() -> Self {
        Self::new()
    }
}

/// Hide data strategy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum FloatingHideStrategy {
    /// Hide when the reference is clipped away.
    ReferenceHidden,
    /// Hide or de-emphasize when the floating element escapes.
    Escaped,
}

/// Hide strategy options.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FloatingHide {
    /// Hide strategy to report.
    pub strategy: FloatingHideStrategy,
    /// Overflow padding for this strategy.
    pub padding: FloatingPadding,
}

impl FloatingHide {
    /// Creates hide options for a strategy.
    #[must_use]
    pub const fn new(strategy: FloatingHideStrategy) -> Self {
        Self {
            strategy,
            padding: FloatingPadding {
                top: 0.0,
                right: 0.0,
                bottom: 0.0,
                left: 0.0,
            },
        }
    }

    /// Sets overflow padding.
    #[must_use]
    pub const fn padding(mut self, padding: FloatingPadding) -> Self {
        self.padding = padding;
        self
    }
}

/// Hide data reported by the floating computation.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FloatingHideData {
    /// Whether the reference is hidden.
    pub reference_hidden: bool,
    /// Reference clipping offsets.
    pub reference_hidden_offsets: Option<FloatingOverflow>,
    /// Whether the floating element escaped.
    pub escaped: bool,
    /// Floating clipping offsets.
    pub escaped_offsets: Option<FloatingOverflow>,
}

/// Multi-rect inline reference selection.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FloatingInline {
    /// Candidate reference fragments.
    pub rects: Vec<FloatingRect>,
    /// Optional point used to choose a fragment.
    pub point: Option<Point<f32>>,
    /// Padding used when choosing a disjoint fragment.
    pub padding: FloatingPadding,
}

impl FloatingInline {
    /// Creates inline options.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            rects: Vec::new(),
            point: None,
            padding: FloatingPadding {
                top: 2.0,
                right: 2.0,
                bottom: 2.0,
                left: 2.0,
            },
        }
    }

    /// Sets reference fragments.
    #[must_use]
    pub fn rects(mut self, rects: impl Into<Vec<FloatingRect>>) -> Self {
        self.rects = rects.into();
        self
    }

    /// Sets the point used to choose a fragment.
    #[must_use]
    pub const fn point(mut self, point: Point<f32>) -> Self {
        self.point = Some(point);
        self
    }

    /// Sets selection padding.
    #[must_use]
    pub const fn padding(mut self, padding: FloatingPadding) -> Self {
        self.padding = padding;
        self
    }
}

impl Default for FloatingInline {
    fn default() -> Self {
        Self::new()
    }
}

/// Arrow geometry for a floating element.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FloatingArrow {
    /// Size of the arrow box.
    pub size: Size<f32>,
    /// Minimum distance from each edge of the floating box.
    pub padding: FloatingPadding,
}

impl FloatingArrow {
    /// Creates an arrow with no edge padding.
    #[must_use]
    pub const fn new(size: Size<f32>) -> Self {
        Self {
            size,
            padding: FloatingPadding {
                top: 0.0,
                right: 0.0,
                bottom: 0.0,
                left: 0.0,
            },
        }
    }

    /// Sets arrow edge padding.
    #[must_use]
    pub const fn padding(mut self, padding: f32) -> Self {
        self.padding = FloatingPadding::all(padding);
        self
    }

    /// Sets side-aware arrow edge padding.
    #[must_use]
    pub const fn padding_sides(mut self, padding: FloatingPadding) -> Self {
        self.padding = padding;
        self
    }
}

/// Arrow positioning data.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FloatingArrowData {
    /// Arrow origin inside the floating rectangle.
    pub offset: Point<f32>,
    /// Distance between the clamped arrow and the ideal centered arrow.
    pub center_offset: f32,
}

impl FloatingArrowData {
    /// Creates arrow positioning data.
    #[must_use]
    pub const fn new(offset: Point<f32>, center_offset: f32) -> Self {
        Self {
            offset,
            center_offset,
        }
    }
}

/// Visibility state for the reference/floating relationship.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum FloatingVisibility {
    /// Reference and floating rectangles fit the available boundary.
    Visible,
    /// The reference rectangle is fully outside the boundary.
    ReferenceHidden,
    /// The floating rectangle overflows the boundary.
    FloatingEscaped,
}

/// Options for floating placement.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FloatingOptions {
    /// Preferred placement.
    pub placement: FloatingPlacement,
    /// Optional offset from the preferred placement.
    pub offset: FloatingOffset,
    /// Ordered fallback placements to try before the opposite side.
    pub fallbacks: Vec<FloatingPlacement>,
    /// Optional clipping boundary.
    pub boundary: Option<FloatingBoundary>,
    /// Whether to try the opposite side if the preferred side overflows.
    pub flip: bool,
    /// Flip behavior.
    pub flip_options: FloatingFlip,
    /// Optional automatic placement selection.
    pub auto_placement: Option<FloatingAutoPlacement>,
    /// Optional shift behavior inside the clipping boundary.
    pub shift: Option<FloatingShift>,
    /// Optional size data and constraints.
    pub size: Option<FloatingSize>,
    /// Optional arrow geometry.
    pub arrow: Option<FloatingArrow>,
    /// Hide data strategies to report.
    pub hide: Vec<FloatingHide>,
    /// Optional inline reference fragment selection.
    pub inline: Option<FloatingInline>,
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
                main_axis: FloatingAxisOffset::px(0.0),
                cross_axis: FloatingAxisOffset::px(0.0),
                alignment_axis: None,
            },
            fallbacks: Vec::new(),
            boundary: None,
            flip: false,
            flip_options: FloatingFlip::new(),
            auto_placement: None,
            shift: None,
            size: None,
            arrow: None,
            hide: Vec::new(),
            inline: None,
            rtl: false,
        }
    }

    /// Sets the placement offset.
    #[must_use]
    pub const fn offset(mut self, main_axis: f32, cross_axis: f32) -> Self {
        self.offset = FloatingOffset::new(main_axis, cross_axis);
        self
    }

    /// Sets the placement offset from rect-aware axis values.
    #[must_use]
    pub const fn offset_axes(
        mut self,
        main_axis: FloatingAxisOffset,
        cross_axis: FloatingAxisOffset,
    ) -> Self {
        self.offset = FloatingOffset::from_axes(main_axis, cross_axis);
        self
    }

    /// Sets an aligned placement offset.
    #[must_use]
    pub const fn alignment_axis(mut self, alignment_axis: f32) -> Self {
        self.offset.alignment_axis = Some(FloatingAxisOffset::px(alignment_axis));
        self
    }

    /// Sets a rect-aware aligned placement offset.
    #[must_use]
    pub const fn alignment_axis_offset(mut self, alignment_axis: FloatingAxisOffset) -> Self {
        self.offset.alignment_axis = Some(alignment_axis);
        self
    }

    /// Sets ordered fallback placements.
    #[must_use]
    pub fn fallbacks(mut self, fallbacks: impl Into<Vec<FloatingPlacement>>) -> Self {
        self.fallbacks = fallbacks.into();
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
    pub fn flip(mut self, flip: bool) -> Self {
        self.flip = flip;
        if flip {
            self.auto_placement = None;
        }
        self
    }

    /// Sets flip behavior and enables flipping.
    #[must_use]
    pub fn flip_options(mut self, flip_options: FloatingFlip) -> Self {
        self.flip = true;
        self.flip_options = flip_options;
        self.auto_placement = None;
        self
    }

    /// Sets automatic placement behavior.
    #[must_use]
    pub fn auto_placement(mut self, auto_placement: FloatingAutoPlacement) -> Self {
        self.flip = false;
        self.auto_placement = Some(auto_placement);
        self
    }

    /// Sets shift behavior.
    #[must_use]
    pub const fn shift(mut self, shift: FloatingShift) -> Self {
        self.shift = Some(shift);
        self
    }

    /// Sets size behavior.
    #[must_use]
    pub const fn size(mut self, size: FloatingSize) -> Self {
        self.size = Some(size);
        self
    }

    /// Sets arrow geometry.
    #[must_use]
    pub const fn arrow(mut self, arrow: FloatingArrow) -> Self {
        self.arrow = Some(arrow);
        self
    }

    /// Adds a hide data strategy.
    #[must_use]
    pub fn hide(mut self, strategy: FloatingHideStrategy) -> Self {
        self.hide.push(FloatingHide::new(strategy));
        self
    }

    /// Adds hide data options.
    #[must_use]
    pub fn hide_options(mut self, hide: FloatingHide) -> Self {
        self.hide.push(hide);
        self
    }

    /// Sets inline reference fragment behavior.
    #[must_use]
    pub fn inline(mut self, inline: FloatingInline) -> Self {
        self.inline = Some(inline);
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
    /// Reference rectangle used for placement.
    pub reference_rect: FloatingRect,
    /// Final floating size.
    pub size: Size<f32>,
    /// Final placement after optional flipping.
    pub placement: FloatingPlacement,
    /// Final rectangle after offset and optional shifting.
    pub rect: FloatingRect,
    /// Overflow after final placement if a boundary was provided.
    pub overflow: Option<FloatingOverflow>,
    /// Available size on the final placement side.
    pub available_size: Size<f32>,
    /// Arrow origin inside the floating rectangle.
    pub arrow_offset: Option<Point<f32>>,
    /// Arrow positioning data.
    pub arrow: Option<FloatingArrowData>,
    /// Shift delta applied to the base coordinates.
    pub shift_offset: Option<Point<f32>>,
    /// Hide data.
    pub hide: Option<FloatingHideData>,
    /// Visibility state relative to the boundary.
    pub visibility: FloatingVisibility,
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
    if placement.is_center() {
        return Point {
            x: common_x,
            y: common_y,
        };
    }
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
    let reference = select_inline_reference(reference, &options);
    let mut floating = floating;
    let mut placement = options.placement;
    if let (Some(boundary), Some(auto_placement)) =
        (options.boundary, options.auto_placement.as_ref())
    {
        placement = choose_auto_placement(reference, floating, &options, auto_placement, boundary);
    }
    let mut available_padding = options.size.map(|size| size.padding).unwrap_or_default();
    if let (Some(boundary), Some(size)) = (options.boundary, options.size) {
        let available =
            available_size_with_padding(reference, placement.side(), boundary, size.padding);
        floating = apply_size_options(reference, floating, available, size);
        available_padding = size.padding;
    }
    let mut origin =
        placed_origin_with_offset(reference, floating, placement, options.offset, options.rtl);

    if options.auto_placement.is_none() && !placement.is_center() {
        if let Some(boundary) = options.boundary {
            let fallback = choose_fallback_placement(reference, floating, &options, boundary);
            if fallback != placement {
                placement = fallback;
                origin = placed_origin_with_offset(
                    reference,
                    floating,
                    placement,
                    options.offset,
                    options.rtl,
                );
            }
        }
    }

    let mut shift_offset = None;
    if !placement.is_center() {
        if let (Some(boundary), Some(shift)) = (options.boundary, options.shift) {
            let shifted = shift_origin_into_boundary(
                origin,
                reference,
                floating,
                placement.side(),
                boundary,
                shift,
            );
            shift_offset = Some(Point {
                x: shifted.x - origin.x,
                y: shifted.y - origin.y,
            });
            origin = shifted;
        }
    }

    let rect = FloatingRect::new(origin, floating);
    let overflow = options
        .boundary
        .map(|boundary| detect_overflow(rect, boundary, FloatingPadding::default()));
    let available_size = options
        .boundary
        .map(|boundary| {
            if placement.is_center() {
                Size {
                    width: f32::INFINITY,
                    height: f32::INFINITY,
                }
            } else {
                available_size_with_padding(
                    reference,
                    placement.side(),
                    boundary,
                    available_padding,
                )
            }
        })
        .unwrap_or(Size {
            width: f32::INFINITY,
            height: f32::INFINITY,
        });
    let arrow = options.arrow.and_then(|arrow| {
        (!placement.is_center())
            .then(|| compute_arrow_data(reference, rect, placement.side(), arrow))
    });
    let arrow_offset = arrow.map(|arrow| arrow.offset);
    let visibility = options
        .boundary
        .map(|boundary| visibility_state(reference, rect, boundary))
        .unwrap_or(FloatingVisibility::Visible);
    let hide = options
        .boundary
        .and_then(|boundary| hide_data(reference, rect, boundary, &options.hide));
    FloatingPosition {
        origin,
        reference_rect: reference,
        size: floating,
        placement,
        rect,
        overflow,
        available_size,
        arrow_offset,
        arrow,
        shift_offset,
        hide,
        visibility,
    }
}

/// Applies a floating offset to an already placed origin.
#[must_use]
pub fn apply_offset(
    mut origin: Point<f32>,
    reference: FloatingRect,
    placement: FloatingPlacement,
    offset: FloatingOffset,
    floating: Size<f32>,
) -> Point<f32> {
    let main_axis = offset.main_axis.resolve(reference, floating);
    let cross_axis = offset.cross_axis.resolve(reference, floating);
    if placement.is_center() {
        origin.x += cross_axis;
        origin.y += main_axis;
        return origin;
    }
    let side = placement.side();
    let cross_axis = match (offset.alignment_axis, placement.alignment()) {
        (Some(alignment_axis), Some(FloatingAlignment::Start)) => {
            alignment_axis.resolve(reference, floating)
        }
        (Some(alignment_axis), Some(FloatingAlignment::End)) => {
            -alignment_axis.resolve(reference, floating)
        }
        _ => cross_axis,
    };
    match side {
        FloatingSide::Top => {
            origin.y -= main_axis;
            origin.x += cross_axis;
        }
        FloatingSide::Right => {
            origin.x += main_axis;
            origin.y += cross_axis;
        }
        FloatingSide::Bottom => {
            origin.y += main_axis;
            origin.x += cross_axis;
        }
        FloatingSide::Left => {
            origin.x -= main_axis;
            origin.y += cross_axis;
        }
    }
    origin
}

/// Shifts an origin inside a clipping boundary.
#[must_use]
pub fn shift_origin_into_boundary(
    origin: Point<f32>,
    reference: FloatingRect,
    floating: Size<f32>,
    side: FloatingSide,
    boundary: FloatingBoundary,
    shift: FloatingShift,
) -> Point<f32> {
    let min_x = boundary.rect.left() + boundary.padding.left + shift.padding.left;
    let max_x =
        boundary.rect.right() - boundary.padding.right - shift.padding.right - floating.width;
    let min_y = boundary.rect.top() + boundary.padding.top + shift.padding.top;
    let max_y =
        boundary.rect.bottom() - boundary.padding.bottom - shift.padding.bottom - floating.height;

    let clamp_x =
        (side.is_vertical() && shift.cross_axis) || (!side.is_vertical() && shift.main_axis);
    let clamp_y =
        (side.is_vertical() && shift.main_axis) || (!side.is_vertical() && shift.cross_axis);

    Point {
        x: if clamp_x {
            shift_axis_origin(
                origin.x,
                min_x,
                max_x,
                limit_shift_bounds_x(side, shift, reference, floating),
                shift_distance_limit_for_x(side, shift),
            )
        } else {
            origin.x
        },
        y: if clamp_y {
            shift_axis_origin(
                origin.y,
                min_y,
                max_y,
                limit_shift_bounds_y(side, shift, reference, floating),
                shift_distance_limit_for_y(side, shift),
            )
        } else {
            origin.y
        },
    }
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
        reference,
        placement,
        offset,
        floating,
    )
}

fn select_inline_reference(reference: FloatingRect, options: &FloatingOptions) -> FloatingRect {
    let Some(inline) = options.inline.as_ref() else {
        return reference;
    };
    if inline.rects.is_empty() {
        return reference;
    }
    if let Some(point) = inline.point {
        if let Some(rect) = inline
            .rects
            .iter()
            .copied()
            .find(|rect| padded_rect(*rect, inline.padding).contains_point(point))
        {
            return rect;
        }
    }
    let side = options.placement.side();
    match side {
        FloatingSide::Top => inline
            .rects
            .iter()
            .copied()
            .min_by(|a, b| a.top().total_cmp(&b.top()))
            .unwrap_or(reference),
        FloatingSide::Bottom => inline
            .rects
            .iter()
            .copied()
            .max_by(|a, b| a.bottom().total_cmp(&b.bottom()))
            .unwrap_or(reference),
        FloatingSide::Left => inline
            .rects
            .iter()
            .copied()
            .min_by(|a, b| a.left().total_cmp(&b.left()))
            .unwrap_or(reference),
        FloatingSide::Right => inline
            .rects
            .iter()
            .copied()
            .max_by(|a, b| a.right().total_cmp(&b.right()))
            .unwrap_or(reference),
    }
}

fn padded_rect(rect: FloatingRect, padding: FloatingPadding) -> FloatingRect {
    FloatingRect::from_xy_size(
        rect.left() - padding.left,
        rect.top() - padding.top,
        rect.size.width + padding.left + padding.right,
        rect.size.height + padding.top + padding.bottom,
    )
}

fn apply_size_options(
    reference: FloatingRect,
    mut floating: Size<f32>,
    available: Size<f32>,
    size: FloatingSize,
) -> Size<f32> {
    if size.match_reference_width {
        floating.width = reference.size.width;
    }
    if size.match_reference_height {
        floating.height = reference.size.height;
    }
    if size.max_width_to_available {
        floating.width = f32_min(floating.width, f32_max(available.width, 0.0));
    }
    if size.max_height_to_available {
        floating.height = f32_min(floating.height, f32_max(available.height, 0.0));
    }
    floating
}

fn choose_auto_placement(
    reference: FloatingRect,
    floating: Size<f32>,
    options: &FloatingOptions,
    auto_placement: &FloatingAutoPlacement,
    boundary: FloatingBoundary,
) -> FloatingPlacement {
    let candidates = auto_placement_candidates(auto_placement);
    let mut best = options.placement;
    let mut best_score = f32::NEG_INFINITY;
    for candidate in candidates {
        let origin =
            placed_origin_with_offset(reference, floating, candidate, options.offset, options.rtl);
        let overflow = detect_overflow(
            FloatingRect::new(origin, floating),
            boundary,
            auto_placement.padding,
        );
        let score = auto_placement_score(
            reference,
            candidate.side(),
            boundary,
            overflow,
            auto_placement,
        );
        if score > best_score {
            best = candidate;
            best_score = score;
        }
    }
    best
}

fn auto_placement_candidates(auto_placement: &FloatingAutoPlacement) -> Vec<FloatingPlacement> {
    if !auto_placement.allowed_placements.is_empty() {
        return auto_placement.allowed_placements.clone();
    }
    let sides = [
        FloatingSide::Top,
        FloatingSide::Right,
        FloatingSide::Bottom,
        FloatingSide::Left,
    ];
    let mut candidates = Vec::new();
    for side in sides {
        match auto_placement.alignment {
            Some(alignment) => {
                candidates.push(FloatingPlacement::from_side_alignment(
                    side,
                    Some(alignment),
                ));
                if auto_placement.auto_alignment {
                    candidates.push(FloatingPlacement::from_side_alignment(
                        side,
                        Some(alignment.opposite()),
                    ));
                }
            }
            None => candidates.push(FloatingPlacement::from_side_alignment(side, None)),
        }
    }
    candidates
}

fn auto_placement_score(
    reference: FloatingRect,
    side: FloatingSide,
    boundary: FloatingBoundary,
    overflow: FloatingOverflow,
    auto_placement: &FloatingAutoPlacement,
) -> f32 {
    let available = available_size_with_padding(reference, side, boundary, auto_placement.padding);
    let main = if side.is_vertical() {
        available.height
    } else {
        available.width
    };
    let cross = if side.is_vertical() {
        available.width
    } else {
        available.height
    };
    let overflow_penalty = if auto_placement.cross_axis || auto_placement.alignment.is_some() {
        overflow_score(overflow)
    } else {
        f32_max(overflow.side(side), 0.0)
    };
    main + if auto_placement.cross_axis {
        cross * 0.25
    } else {
        0.0
    } - overflow_penalty * 4.0
}

fn choose_fallback_placement(
    reference: FloatingRect,
    floating: Size<f32>,
    options: &FloatingOptions,
    boundary: FloatingBoundary,
) -> FloatingPlacement {
    let flip = &options.flip_options;
    let mut candidates = Vec::with_capacity(options.fallbacks.len() + 6);
    candidates.push(options.placement);
    let use_legacy_fallback_scoring = !options.flip && !options.fallbacks.is_empty();
    if !flip.fallback_placements.is_empty() {
        candidates.extend(flip.fallback_placements.iter().copied());
    } else {
        candidates.extend(options.fallbacks.iter().copied());
    }
    if options.flip && flip.fallback_placements.is_empty() && options.fallbacks.is_empty() {
        if flip.flip_alignment && options.placement.alignment().is_some() {
            candidates.push(options.placement.opposite_alignment());
        }
        candidates.push(options.placement.opposite());
        if flip.flip_alignment && options.placement.alignment().is_some() {
            candidates.push(options.placement.opposite_side_and_alignment());
        }
        candidates.extend(perpendicular_fallbacks(
            options.placement,
            flip.fallback_axis_side_direction,
            options.rtl,
        ));
    }

    if !options.flip && options.fallbacks.is_empty() && flip.fallback_placements.is_empty() {
        return options.placement;
    }
    let mut best = options.placement;
    let mut best_score = f32::INFINITY;
    for candidate in candidates {
        let origin =
            placed_origin_with_offset(reference, floating, candidate, options.offset, options.rtl);
        let overflow = detect_overflow(FloatingRect::new(origin, floating), boundary, flip.padding);
        let relevant_overflow = if use_legacy_fallback_scoring {
            overflow
        } else {
            flip_relevant_overflow(overflow, candidate, flip)
        };
        if !relevant_overflow.has_overflow() {
            return candidate;
        }
        let score = overflow_score(relevant_overflow);
        if score < best_score {
            best = candidate;
            best_score = score;
        }
    }
    match flip.fallback_strategy {
        FloatingFallbackStrategy::BestFit => best,
        FloatingFallbackStrategy::InitialPlacement => options.placement,
    }
}

fn perpendicular_fallbacks(
    placement: FloatingPlacement,
    direction: FloatingFallbackAxisSideDirection,
    rtl: bool,
) -> Vec<FloatingPlacement> {
    if matches!(direction, FloatingFallbackAxisSideDirection::None) {
        return Vec::new();
    }
    let side = placement.side();
    let alignment = placement.alignment();
    let (first, second) = match (side.is_vertical(), direction, rtl) {
        (true, FloatingFallbackAxisSideDirection::Start, false)
        | (true, FloatingFallbackAxisSideDirection::End, true) => {
            (FloatingSide::Left, FloatingSide::Right)
        }
        (true, FloatingFallbackAxisSideDirection::End, false)
        | (true, FloatingFallbackAxisSideDirection::Start, true) => {
            (FloatingSide::Right, FloatingSide::Left)
        }
        (false, FloatingFallbackAxisSideDirection::Start, _) => {
            (FloatingSide::Top, FloatingSide::Bottom)
        }
        (false, FloatingFallbackAxisSideDirection::End, _) => {
            (FloatingSide::Bottom, FloatingSide::Top)
        }
        (_, FloatingFallbackAxisSideDirection::None, _) => return Vec::new(),
    };
    vec![
        FloatingPlacement::from_side_alignment(first, alignment),
        FloatingPlacement::from_side_alignment(second, alignment),
    ]
}

fn flip_relevant_overflow(
    overflow: FloatingOverflow,
    placement: FloatingPlacement,
    flip: &FloatingFlip,
) -> FloatingOverflow {
    let side = placement.side();
    let mut relevant = FloatingOverflow::default();
    if flip.main_axis {
        match side {
            FloatingSide::Top => relevant.top = overflow.top,
            FloatingSide::Right => relevant.right = overflow.right,
            FloatingSide::Bottom => relevant.bottom = overflow.bottom,
            FloatingSide::Left => relevant.left = overflow.left,
        }
    }
    let check_cross_axis = match flip.cross_axis {
        FloatingFlipCrossAxis::None => false,
        FloatingFlipCrossAxis::Alignment => placement.alignment().is_some(),
        FloatingFlipCrossAxis::All => true,
    };
    if check_cross_axis {
        if side.is_vertical() {
            relevant.left = overflow.left;
            relevant.right = overflow.right;
        } else {
            relevant.top = overflow.top;
            relevant.bottom = overflow.bottom;
        }
    }
    relevant
}

fn overflow_score(overflow: FloatingOverflow) -> f32 {
    f32_max(overflow.top, 0.0)
        + f32_max(overflow.right, 0.0)
        + f32_max(overflow.bottom, 0.0)
        + f32_max(overflow.left, 0.0)
}

fn available_size_with_padding(
    reference: FloatingRect,
    side: FloatingSide,
    boundary: FloatingBoundary,
    padding: FloatingPadding,
) -> Size<f32> {
    let left = boundary.rect.left() + boundary.padding.left + padding.left;
    let right = boundary.rect.right() - boundary.padding.right - padding.right;
    let top = boundary.rect.top() + boundary.padding.top + padding.top;
    let bottom = boundary.rect.bottom() - boundary.padding.bottom - padding.bottom;
    match side {
        FloatingSide::Top => Size {
            width: right - left,
            height: reference.top() - top,
        },
        FloatingSide::Right => Size {
            width: right - reference.right(),
            height: bottom - top,
        },
        FloatingSide::Bottom => Size {
            width: right - left,
            height: bottom - reference.bottom(),
        },
        FloatingSide::Left => Size {
            width: reference.left() - left,
            height: bottom - top,
        },
    }
}

fn compute_arrow_data(
    reference: FloatingRect,
    floating: FloatingRect,
    side: FloatingSide,
    arrow: FloatingArrow,
) -> FloatingArrowData {
    if side.is_vertical() {
        let reference_center = reference.left() + reference.size.width / 2.0;
        let raw_x = reference_center - floating.left() - arrow.size.width / 2.0;
        let x = clamp(
            raw_x,
            arrow.padding.left,
            floating.size.width - arrow.size.width - arrow.padding.right,
        );
        FloatingArrowData {
            offset: Point { x, y: 0.0 },
            center_offset: raw_x - x,
        }
    } else {
        let reference_center = reference.top() + reference.size.height / 2.0;
        let raw_y = reference_center - floating.top() - arrow.size.height / 2.0;
        let y = clamp(
            raw_y,
            arrow.padding.top,
            floating.size.height - arrow.size.height - arrow.padding.bottom,
        );
        FloatingArrowData {
            offset: Point { x: 0.0, y },
            center_offset: raw_y - y,
        }
    }
}

fn visibility_state(
    reference: FloatingRect,
    floating: FloatingRect,
    boundary: FloatingBoundary,
) -> FloatingVisibility {
    if reference.right() <= boundary.rect.left()
        || reference.left() >= boundary.rect.right()
        || reference.bottom() <= boundary.rect.top()
        || reference.top() >= boundary.rect.bottom()
    {
        return FloatingVisibility::ReferenceHidden;
    }
    if detect_overflow(floating, boundary, FloatingPadding::default()).has_overflow() {
        return FloatingVisibility::FloatingEscaped;
    }
    FloatingVisibility::Visible
}

fn hide_data(
    reference: FloatingRect,
    floating: FloatingRect,
    boundary: FloatingBoundary,
    strategies: &[FloatingHide],
) -> Option<FloatingHideData> {
    if strategies.is_empty() {
        return None;
    }
    let mut data = FloatingHideData::default();
    for hide in strategies {
        match hide.strategy {
            FloatingHideStrategy::ReferenceHidden => {
                let offsets = detect_overflow(reference, boundary, hide.padding);
                data.reference_hidden = reference.right()
                    <= boundary.rect.left() + hide.padding.left
                    || reference.left() >= boundary.rect.right() - hide.padding.right
                    || reference.bottom() <= boundary.rect.top() + hide.padding.top
                    || reference.top() >= boundary.rect.bottom() - hide.padding.bottom;
                data.reference_hidden_offsets = Some(offsets);
            }
            FloatingHideStrategy::Escaped => {
                let offsets = detect_overflow(floating, boundary, hide.padding);
                data.escaped = offsets.has_overflow();
                data.escaped_offsets = Some(offsets);
            }
        }
    }
    Some(data)
}

fn limit_shift_bounds_x(
    side: FloatingSide,
    shift: FloatingShift,
    reference: FloatingRect,
    floating: Size<f32>,
) -> Option<(f32, f32)> {
    let Some(limiter) = shift.limiter else {
        return None;
    };
    let applies = if side.is_vertical() {
        limiter.cross_axis
    } else {
        limiter.main_axis
    };
    if !applies {
        return None;
    }
    let offset = limiter.offset.resolve(reference, floating);
    Some((
        reference.left() - floating.width + offset,
        reference.right() - offset,
    ))
}

fn limit_shift_bounds_y(
    side: FloatingSide,
    shift: FloatingShift,
    reference: FloatingRect,
    floating: Size<f32>,
) -> Option<(f32, f32)> {
    let Some(limiter) = shift.limiter else {
        return None;
    };
    let applies = if side.is_vertical() {
        limiter.main_axis
    } else {
        limiter.cross_axis
    };
    if !applies {
        return None;
    }
    let offset = limiter.offset.resolve(reference, floating);
    Some((
        reference.top() - floating.height + offset,
        reference.bottom() - offset,
    ))
}

fn shift_distance_limit_for_x(side: FloatingSide, shift: FloatingShift) -> Option<f32> {
    if side.is_vertical() {
        shift.cross_axis_limit
    } else {
        shift.main_axis_limit
    }
}

fn shift_distance_limit_for_y(side: FloatingSide, shift: FloatingShift) -> Option<f32> {
    if side.is_vertical() {
        shift.main_axis_limit
    } else {
        shift.cross_axis_limit
    }
}

fn shift_axis_origin(
    value: f32,
    min: f32,
    max: f32,
    limiter_bounds: Option<(f32, f32)>,
    distance_limit: Option<f32>,
) -> f32 {
    let mut clamped = clamp(value, min, max);
    if clamped == value {
        return value;
    }
    if let Some((limiter_min, limiter_max)) = limiter_bounds {
        clamped = clamp(clamped, limiter_min, limiter_max);
    }
    match distance_limit {
        Some(limit) => {
            let distance = clamped - value;
            value + clamp(distance, -limit, limit)
        }
        None => clamped,
    }
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
