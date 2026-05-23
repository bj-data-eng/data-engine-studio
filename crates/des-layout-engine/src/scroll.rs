//! Scroll container and scrollbar geometry math.
//!
//! This module keeps scroll range, viewport, clipping, and scrollbar handle
//! positioning independent from any particular UI adapter or document model.

use crate::geometry::{Point, Rect, Size};
use crate::style::Overflow;
use crate::util::sys::{f32_max, f32_min};

/// The absolute axis a scroll operation applies to.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ScrollAxis {
    /// Horizontal scrolling along the x-axis.
    Horizontal,
    /// Vertical scrolling along the y-axis.
    Vertical,
}

impl ScrollAxis {
    /// Returns the position component for this axis.
    #[must_use]
    pub fn position(self, point: Point<f32>) -> f32 {
        match self {
            Self::Horizontal => point.x,
            Self::Vertical => point.y,
        }
    }

    /// Returns the size component for this axis.
    #[must_use]
    pub fn size(self, size: Size<f32>) -> f32 {
        match self {
            Self::Horizontal => size.width,
            Self::Vertical => size.height,
        }
    }

    /// Returns the origin component for this axis.
    #[must_use]
    pub fn rect_origin(self, rect: ScrollRect) -> f32 {
        self.position(rect.origin)
    }

    /// Returns the length component for this axis.
    #[must_use]
    pub fn rect_length(self, rect: ScrollRect) -> f32 {
        self.size(rect.size)
    }
}

/// A rectangle used by scroll geometry.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ScrollRect {
    /// The top-left origin of the rectangle.
    pub origin: Point<f32>,
    /// The width and height of the rectangle.
    pub size: Size<f32>,
}

impl ScrollRect {
    /// Creates a scroll rectangle from an origin and size.
    #[must_use]
    pub const fn new(origin: Point<f32>, size: Size<f32>) -> Self {
        Self { origin, size }
    }

    /// Creates a scroll rectangle from scalar components.
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

    /// Returns a rectangle inset by the given side values.
    #[must_use]
    pub fn inset(self, insets: Rect<f32>) -> Self {
        Self::from_xy_size(
            self.origin.x + insets.left,
            self.origin.y + insets.top,
            f32_max(0.0, self.size.width - insets.left - insets.right),
            f32_max(0.0, self.size.height - insets.top - insets.bottom),
        )
    }

    /// Returns true when the point is inside the rectangle.
    #[must_use]
    pub fn contains(self, point: Point<f32>) -> bool {
        point.x >= self.left()
            && point.x <= self.right()
            && point.y >= self.top()
            && point.y <= self.bottom()
    }

    /// Returns the intersection of two rectangles.
    #[must_use]
    pub fn intersect(self, other: Self) -> Option<Self> {
        let left = f32_max(self.left(), other.left());
        let top = f32_max(self.top(), other.top());
        let right = f32_min(self.right(), other.right());
        let bottom = f32_min(self.bottom(), other.bottom());
        if right <= left || bottom <= top {
            return None;
        }

        Some(Self::from_xy_size(left, top, right - left, bottom - top))
    }
}

/// Inputs required to compute scrollbar geometry for one axis.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ScrollbarGeometryInput {
    /// The axis the scrollbar controls.
    pub axis: ScrollAxis,
    /// The scroll container viewport rectangle.
    pub viewport_rect: ScrollRect,
    /// The maximum scroll offset along `axis`.
    pub max_scroll: f32,
    /// The current scroll offset along `axis`.
    pub scroll_offset: f32,
    /// The visual width or height of the scrollbar track and handle.
    pub visual_width: f32,
    /// The hit-test width or height of the scrollbar interaction strip.
    pub hit_width: f32,
    /// The minimum handle length.
    pub min_handle_length: f32,
    /// Optional clipping rectangle inherited from ancestor scroll containers.
    pub clip_rect: Option<ScrollRect>,
}

/// Computed scrollbar geometry for one axis.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ScrollbarGeometry {
    /// The visible scrollbar track rectangle.
    pub track_rect: ScrollRect,
    /// The scrollbar hit-test rectangle.
    pub hit_rect: ScrollRect,
    /// The visible scrollbar handle rectangle.
    pub handle_rect: ScrollRect,
    /// The maximum scroll offset along the scrollbar axis.
    pub max_scroll: f32,
}

/// Returns the content viewport rectangle for a scroll container.
#[must_use]
pub fn viewport_rect(border_box: ScrollRect, border: Rect<f32>, padding: Rect<f32>) -> ScrollRect {
    border_box.inset(border).inset(padding)
}

/// Returns the maximum scroll range for both axes.
#[must_use]
pub fn scroll_limits(
    content_size: Size<f32>,
    outer_size: Size<f32>,
    border: Rect<f32>,
) -> Size<f32> {
    Size {
        width: scroll_limit_for_axis(content_size.width, outer_size.width, border.right),
        height: scroll_limit_for_axis(content_size.height, outer_size.height, border.bottom),
    }
}

/// Returns the maximum scroll range for a single axis.
#[must_use]
pub fn scroll_limit_for_axis(
    content_size: f32,
    outer_size: f32,
    trailing_border_width: f32,
) -> f32 {
    let overflow = content_size - outer_size;
    if overflow > 0.0 {
        overflow + trailing_border_width
    } else {
        0.0
    }
}

/// Clamps a scroll offset to a maximum scroll range.
#[must_use]
pub fn clamp_scroll_offset(offset: Point<f32>, max_scroll: Size<f32>) -> Point<f32> {
    Point {
        x: clamp_scroll_value(offset.x, max_scroll.width),
        y: clamp_scroll_value(offset.y, max_scroll.height),
    }
}

/// Clamps a single scroll offset to its maximum scroll range.
#[must_use]
pub fn clamp_scroll_value(offset: f32, max_scroll: f32) -> f32 {
    offset.clamp(0.0, max_scroll)
}

/// Computes the child clip rectangle inherited from a scroll container.
#[must_use]
pub fn child_clip_rect(
    border_box: ScrollRect,
    border: Rect<f32>,
    padding: Rect<f32>,
    overflow_x: Overflow,
    overflow_y: Overflow,
    parent_clip: Option<ScrollRect>,
) -> Option<ScrollRect> {
    if !overflow_x.is_scroll_container()
        && overflow_x != Overflow::Clip
        && !overflow_y.is_scroll_container()
        && overflow_y != Overflow::Clip
    {
        return parent_clip;
    }

    let viewport = viewport_rect(border_box, border, padding);
    let left = if overflow_x.is_scroll_container() || overflow_x == Overflow::Clip {
        viewport.left()
    } else {
        parent_clip.map_or(border_box.left(), ScrollRect::left)
    };
    let right = if overflow_x.is_scroll_container() || overflow_x == Overflow::Clip {
        viewport.right()
    } else {
        parent_clip.map_or(border_box.right(), ScrollRect::right)
    };
    let top = if overflow_y.is_scroll_container() || overflow_y == Overflow::Clip {
        viewport.top()
    } else {
        parent_clip.map_or(border_box.top(), ScrollRect::top)
    };
    let bottom = if overflow_y.is_scroll_container() || overflow_y == Overflow::Clip {
        viewport.bottom()
    } else {
        parent_clip.map_or(border_box.bottom(), ScrollRect::bottom)
    };

    let scroll_clip = ScrollRect::from_xy_size(left, top, right - left, bottom - top);
    parent_clip
        .and_then(|clip| clip.intersect(scroll_clip))
        .or(Some(scroll_clip))
}

/// Computes scrollbar geometry for one axis.
#[must_use]
pub fn scrollbar_geometry(input: ScrollbarGeometryInput) -> Option<ScrollbarGeometry> {
    let viewport_main = input.axis.rect_length(input.viewport_rect);
    let content_main = viewport_main + input.max_scroll;
    let handle_length = if content_main <= f32::EPSILON {
        viewport_main
    } else {
        (viewport_main / content_main * viewport_main)
            .max(input.min_handle_length)
            .min(viewport_main)
    };
    let track_travel = f32_max(0.0, viewport_main - handle_length);
    let handle_start = if input.max_scroll <= f32::EPSILON {
        input.axis.rect_origin(input.viewport_rect)
    } else {
        input.axis.rect_origin(input.viewport_rect)
            + (input.scroll_offset / input.max_scroll).clamp(0.0, 1.0) * track_travel
    };
    let visual_width = f32_max(0.0, input.visual_width);
    let hit_width = f32_max(0.0, input.hit_width);

    let (track_rect, hit_rect, handle_rect) = match input.axis {
        ScrollAxis::Horizontal => (
            ScrollRect::from_xy_size(
                input.viewport_rect.left(),
                input.viewport_rect.bottom() - visual_width,
                input.viewport_rect.size.width,
                visual_width,
            ),
            ScrollRect::from_xy_size(
                input.viewport_rect.left(),
                input.viewport_rect.bottom() - hit_width,
                input.viewport_rect.size.width,
                hit_width,
            ),
            ScrollRect::from_xy_size(
                handle_start,
                input.viewport_rect.bottom() - visual_width,
                handle_length,
                visual_width,
            ),
        ),
        ScrollAxis::Vertical => (
            ScrollRect::from_xy_size(
                input.viewport_rect.right() - visual_width,
                input.viewport_rect.top(),
                visual_width,
                input.viewport_rect.size.height,
            ),
            ScrollRect::from_xy_size(
                input.viewport_rect.right() - hit_width,
                input.viewport_rect.top(),
                hit_width,
                input.viewport_rect.size.height,
            ),
            ScrollRect::from_xy_size(
                input.viewport_rect.right() - visual_width,
                handle_start,
                visual_width,
                handle_length,
            ),
        ),
    };

    let (track_rect, hit_rect, handle_rect) = if let Some(clip_rect) = input.clip_rect {
        (
            track_rect.intersect(clip_rect)?,
            hit_rect.intersect(clip_rect)?,
            handle_rect.intersect(clip_rect)?,
        )
    } else {
        (track_rect, hit_rect, handle_rect)
    };

    Some(ScrollbarGeometry {
        track_rect,
        hit_rect,
        handle_rect,
        max_scroll: input.max_scroll,
    })
}

/// Converts a scrollbar drag pointer position into a scroll offset.
#[must_use]
pub fn scroll_offset_from_handle_drag(
    axis: ScrollAxis,
    track_rect: ScrollRect,
    handle_rect: ScrollRect,
    pointer_position: Point<f32>,
    pointer_offset_from_handle_start: f32,
    max_scroll: f32,
) -> f32 {
    let track_travel = f32_max(
        0.0,
        axis.rect_length(track_rect) - axis.rect_length(handle_rect),
    );
    let handle_start = axis.position(pointer_position) - pointer_offset_from_handle_start;
    if track_travel <= f32::EPSILON {
        0.0
    } else {
        ((handle_start - axis.rect_origin(track_rect)) / track_travel).clamp(0.0, 1.0) * max_scroll
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(x: f32, y: f32, width: f32, height: f32) -> ScrollRect {
        ScrollRect::from_xy_size(x, y, width, height)
    }

    #[test]
    fn scroll_limits_include_trailing_border_when_content_overflows() {
        let limits = scroll_limits(
            Size {
                width: 260.0,
                height: 180.0,
            },
            Size {
                width: 200.0,
                height: 160.0,
            },
            Rect {
                left: 2.0,
                right: 4.0,
                top: 3.0,
                bottom: 5.0,
            },
        );

        assert_eq!(
            limits,
            Size {
                width: 64.0,
                height: 25.0,
            }
        );
    }

    #[test]
    fn scrollbar_geometry_maps_scroll_offset_to_handle_position() {
        let geometry = scrollbar_geometry(ScrollbarGeometryInput {
            axis: ScrollAxis::Vertical,
            viewport_rect: rect(10.0, 20.0, 100.0, 200.0),
            max_scroll: 200.0,
            scroll_offset: 100.0,
            visual_width: 6.0,
            hit_width: 12.0,
            min_handle_length: 18.0,
            clip_rect: None,
        })
        .expect("scrollbar should be visible");

        assert_eq!(geometry.track_rect, rect(104.0, 20.0, 6.0, 200.0));
        assert_eq!(geometry.hit_rect, rect(98.0, 20.0, 12.0, 200.0));
        assert_eq!(geometry.handle_rect, rect(104.0, 70.0, 6.0, 100.0));
    }

    #[test]
    fn handle_drag_maps_pointer_position_to_scroll_offset() {
        let offset = scroll_offset_from_handle_drag(
            ScrollAxis::Horizontal,
            rect(10.0, 20.0, 100.0, 8.0),
            rect(35.0, 20.0, 50.0, 8.0),
            Point { x: 70.0, y: 22.0 },
            10.0,
            200.0,
        );

        assert_eq!(offset, 200.0);
    }

    #[test]
    fn child_clip_rect_uses_finite_border_box_on_unclipped_axis_without_parent_clip() {
        let clip = child_clip_rect(
            rect(10.0, 20.0, 120.0, 80.0),
            Rect {
                left: 2.0,
                right: 4.0,
                top: 3.0,
                bottom: 5.0,
            },
            Rect {
                left: 7.0,
                right: 11.0,
                top: 13.0,
                bottom: 17.0,
            },
            Overflow::Visible,
            Overflow::Scroll,
            None,
        )
        .expect("scrolling one axis should still produce a finite clip rectangle");

        assert_eq!(clip, rect(10.0, 36.0, 120.0, 42.0));
    }

    #[test]
    fn child_clip_rect_clips_hidden_and_clip_overflow_without_scroll_chrome_semantics() {
        let hidden = child_clip_rect(
            rect(10.0, 20.0, 120.0, 80.0),
            Rect::length(2.0),
            Rect::length(8.0),
            Overflow::Hidden,
            Overflow::Clip,
            None,
        )
        .expect("hidden and clip overflow should constrain descendants");

        assert_eq!(hidden, rect(20.0, 30.0, 100.0, 60.0));
    }
}
