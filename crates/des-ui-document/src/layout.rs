use crate::geometry::{Insets, Overflow, Point, Rect, ScrollAxis, Size};
use crate::state::ResolvedElement;
use layout_engine::geometry::{Point as LayoutPoint, Rect as LayoutInsets, Size as LayoutSize};
use layout_engine::scroll::{ScrollAxis as LayoutScrollAxis, ScrollRect};
use layout_engine::style::Overflow as LayoutOverflow;

pub(crate) fn hit_path(frame: &ResolvedElement, point: Point) -> Option<Vec<&ResolvedElement>> {
    let mut children: Vec<_> = frame.children.iter().collect();
    children.sort_by_key(|child| child.style.z_index);

    let clips_overflow =
        frame.style.overflow_x == Overflow::Scroll || frame.style.overflow_y == Overflow::Scroll;
    let may_hit_children = !clips_overflow || frame.rect.contains(point);
    if may_hit_children
        && let Some(mut child_path) = children
            .into_iter()
            .rev()
            .find_map(|child| hit_path(child, point))
    {
        let mut path = vec![frame];
        path.append(&mut child_path);
        return Some(path);
    }

    if frame.rect.contains(point) {
        return Some(vec![frame]);
    }

    None
}

pub(crate) fn to_layout_point(point: Point) -> LayoutPoint<f32> {
    LayoutPoint {
        x: point.x,
        y: point.y,
    }
}

pub(crate) fn to_layout_size(size: Size) -> LayoutSize<f32> {
    LayoutSize {
        width: size.width,
        height: size.height,
    }
}

pub(crate) fn to_layout_insets(insets: Insets) -> LayoutInsets<f32> {
    LayoutInsets {
        left: insets.left,
        right: insets.right,
        top: insets.top,
        bottom: insets.bottom,
    }
}

pub(crate) fn to_layout_overflow(overflow: Overflow) -> LayoutOverflow {
    match overflow {
        Overflow::Visible => LayoutOverflow::Visible,
        Overflow::Scroll => LayoutOverflow::Scroll,
    }
}

pub(crate) fn to_scroll_axis(axis: ScrollAxis) -> LayoutScrollAxis {
    match axis {
        ScrollAxis::Horizontal => LayoutScrollAxis::Horizontal,
        ScrollAxis::Vertical => LayoutScrollAxis::Vertical,
    }
}

pub(crate) fn to_scroll_rect(rect: Rect) -> ScrollRect {
    ScrollRect::new(to_layout_point(rect.origin), to_layout_size(rect.size))
}

pub(crate) fn from_scroll_rect(rect: ScrollRect) -> Rect {
    Rect::new(
        rect.origin.x,
        rect.origin.y,
        rect.size.width,
        rect.size.height,
    )
}
