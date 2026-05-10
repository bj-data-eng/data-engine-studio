use crate::element::ElementId;
use crate::geometry::{Insets, Overflow, Point, Rect, ScrollAxis, Size};
use crate::state::{ElementState, ResolvedElement, ScrollChrome};
use layout_engine::geometry::{Point as LayoutPoint, Rect as LayoutInsets, Size as LayoutSize};
use layout_engine::scroll::{
    self as layout_scroll, ScrollAxis as LayoutScrollAxis, ScrollRect, ScrollbarGeometryInput,
};
use layout_engine::style::Overflow as LayoutOverflow;
use std::collections::HashMap;

pub(crate) fn scroll_chrome(
    frame: &ResolvedElement,
    states: &HashMap<ElementId, ElementState>,
    scroll_limits: &HashMap<ElementId, Size>,
) -> Vec<ScrollChrome> {
    let mut chrome = Vec::new();
    collect_scroll_chrome(frame, states, scroll_limits, None, &mut chrome);
    chrome
}

fn collect_scroll_chrome(
    frame: &ResolvedElement,
    states: &HashMap<ElementId, ElementState>,
    scroll_limits: &HashMap<ElementId, Size>,
    clip_rect: Option<Rect>,
    chrome: &mut Vec<ScrollChrome>,
) {
    if let Some(max_scroll) = scroll_limits.get(&frame.id).copied() {
        if frame.style.overflow_y == Overflow::Scroll && max_scroll.height > 0.0 {
            if let Some(scroll_chrome) = scroll_chrome_for_frame(
                frame,
                states,
                ScrollAxis::Vertical,
                max_scroll.height,
                clip_rect,
            ) {
                chrome.push(scroll_chrome);
            }
        }
        if frame.style.overflow_x == Overflow::Scroll && max_scroll.width > 0.0 {
            if let Some(scroll_chrome) = scroll_chrome_for_frame(
                frame,
                states,
                ScrollAxis::Horizontal,
                max_scroll.width,
                clip_rect,
            ) {
                chrome.push(scroll_chrome);
            }
        }
    }

    let child_clip = child_clip_rect(frame, clip_rect);
    for child in &frame.children {
        collect_scroll_chrome(child, states, scroll_limits, child_clip, chrome);
    }
}

fn scroll_chrome_for_frame(
    frame: &ResolvedElement,
    states: &HashMap<ElementId, ElementState>,
    axis: ScrollAxis,
    max_scroll: f32,
    clip_rect: Option<Rect>,
) -> Option<ScrollChrome> {
    const HIT_WIDTH: f32 = 12.0;
    const MIN_HANDLE_LENGTH: f32 = 18.0;

    let state = states.get(&frame.id);
    let container_hovered = state.is_some_and(|state| state.hovered);
    let scrollbar_hovered = state.is_some_and(|state| state.scrollbar_hovered_axis == Some(axis));
    let dragged = state.is_some_and(|state| state.scrollbar_dragged_axis == Some(axis));
    let visible =
        frame.style.scrollbar_visible || container_hovered || scrollbar_hovered || dragged;
    let expanded = scrollbar_hovered || dragged;
    let compact_visual_width = frame.style.scrollbar_width.max(0.0);
    let expanded_visual_width = frame.style.scrollbar_expanded_width.max(0.0);
    let target_visual_width = if expanded {
        expanded_visual_width
    } else {
        compact_visual_width
    };
    let visual_width = state
        .and_then(|state| match axis {
            ScrollAxis::Horizontal => state.scrollbar_visual_width_x,
            ScrollAxis::Vertical => state.scrollbar_visual_width_y,
        })
        .unwrap_or(target_visual_width)
        .max(0.0);
    let state_scroll = state
        .map(|state| match axis {
            ScrollAxis::Horizontal => state.scroll_x,
            ScrollAxis::Vertical => state.scroll_y,
        })
        .unwrap_or_default();
    let geometry = layout_scroll::scrollbar_geometry(ScrollbarGeometryInput {
        axis: layout_scroll_axis(axis),
        viewport_rect: layout_scroll::viewport_rect(
            layout_scroll_rect(frame.rect),
            layout_scroll_insets(frame.style.border_width),
            layout_scroll_insets(frame.style.padding),
        ),
        max_scroll,
        scroll_offset: state_scroll,
        visual_width,
        hit_width: HIT_WIDTH,
        min_handle_length: MIN_HANDLE_LENGTH,
        clip_rect: clip_rect.map(layout_scroll_rect),
    })?;
    let handle_color = if dragged {
        frame
            .style
            .scrollbar_pressed_handle_color
            .unwrap_or(frame.style.scrollbar_handle_color)
    } else if scrollbar_hovered {
        frame
            .style
            .scrollbar_hover_handle_color
            .unwrap_or(frame.style.scrollbar_handle_color)
    } else {
        frame.style.scrollbar_handle_color
    };
    let track_color = if dragged {
        frame
            .style
            .scrollbar_pressed_track_color
            .or(frame.style.scrollbar_track_color)
    } else if scrollbar_hovered {
        frame
            .style
            .scrollbar_hover_track_color
            .or(frame.style.scrollbar_track_color)
    } else {
        frame.style.scrollbar_track_color
    };
    let handle_border_color = if dragged {
        frame
            .style
            .scrollbar_pressed_handle_border_color
            .or(frame.style.scrollbar_handle_border_color)
    } else if scrollbar_hovered {
        frame
            .style
            .scrollbar_hover_handle_border_color
            .or(frame.style.scrollbar_handle_border_color)
    } else {
        frame.style.scrollbar_handle_border_color
    };
    let handle_border_width = if dragged {
        frame
            .style
            .scrollbar_pressed_handle_border_width
            .unwrap_or(frame.style.scrollbar_handle_border_width)
    } else if scrollbar_hovered {
        frame
            .style
            .scrollbar_hover_handle_border_width
            .unwrap_or(frame.style.scrollbar_handle_border_width)
    } else {
        frame.style.scrollbar_handle_border_width
    };

    Some(ScrollChrome {
        element_id: frame.id.clone(),
        axis,
        track_rect: document_rect(geometry.track_rect),
        hit_rect: document_rect(geometry.hit_rect),
        handle_rect: document_rect(geometry.handle_rect),
        handle_color,
        track_color,
        handle_border_color,
        handle_border_width,
        radius: frame.style.scrollbar_radius,
        max_scroll,
        visible,
        expanded,
        hovered: scrollbar_hovered,
        dragged,
        compact_visual_width,
        expanded_visual_width,
        transition: frame.style.transition,
    })
}

fn child_clip_rect(frame: &ResolvedElement, parent_clip: Option<Rect>) -> Option<Rect> {
    layout_scroll::child_clip_rect(
        layout_scroll_rect(frame.rect),
        layout_scroll_insets(frame.style.border_width),
        layout_scroll_insets(frame.style.padding),
        layout_overflow(frame.style.overflow_x),
        layout_overflow(frame.style.overflow_y),
        parent_clip.map(layout_scroll_rect),
    )
    .map(document_rect)
}

pub(crate) fn layout_scroll_axis(axis: ScrollAxis) -> LayoutScrollAxis {
    match axis {
        ScrollAxis::Horizontal => LayoutScrollAxis::Horizontal,
        ScrollAxis::Vertical => LayoutScrollAxis::Vertical,
    }
}

pub(crate) fn layout_scroll_point(point: Point) -> LayoutPoint<f32> {
    LayoutPoint {
        x: point.x,
        y: point.y,
    }
}

pub(crate) fn layout_scroll_rect(rect: Rect) -> ScrollRect {
    ScrollRect::new(
        layout_scroll_point(rect.origin),
        LayoutSize {
            width: rect.size.width,
            height: rect.size.height,
        },
    )
}

fn layout_scroll_insets(insets: Insets) -> LayoutInsets<f32> {
    LayoutInsets {
        left: insets.left,
        right: insets.right,
        top: insets.top,
        bottom: insets.bottom,
    }
}

fn layout_overflow(overflow: Overflow) -> LayoutOverflow {
    match overflow {
        Overflow::Visible => LayoutOverflow::Visible,
        Overflow::Scroll => LayoutOverflow::Scroll,
    }
}

pub(crate) fn document_rect(rect: ScrollRect) -> Rect {
    Rect::new(
        rect.origin.x,
        rect.origin.y,
        rect.size.width,
        rect.size.height,
    )
}
