use crate::geometry::{ClipRect, Insets, Overflow, Point, Rect, ScrollAxis, Size};
use crate::state::ResolvedElement;
use des_layout::geometry::{Point as LayoutPoint, Rect as LayoutInsets, Size as LayoutSize};
use des_layout::scroll::{ScrollAxis as LayoutScrollAxis, ScrollRect};

pub(crate) fn hit_path(frame: &ResolvedElement, point: Point) -> Option<Vec<&ResolvedElement>> {
    if !frame.clip_rect.contains(point) || frame.clip_rect.is_empty() {
        return None;
    }

    let mut children: Vec<_> = frame.children.iter().collect();
    children.sort_by_key(|child| child.style.z_index);

    if let Some(mut child_path) = children
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

pub(crate) fn child_clip_rect(
    frame_rect: Rect,
    style: &crate::ComputedStyle,
    parent_clip: ClipRect,
) -> ClipRect {
    let viewport = frame_rect.inset(style.border_width).inset(style.padding);
    let mut clip = parent_clip;
    if clips_overflow(style.overflow_x) {
        clip = clip.constrain_x(viewport.origin.x, viewport.right());
    }
    if clips_overflow(style.overflow_y) {
        clip = clip.constrain_y(viewport.origin.y, viewport.bottom());
    }
    clip
}

pub(crate) fn clips_overflow(overflow: Overflow) -> bool {
    overflow.clips_contents()
}
