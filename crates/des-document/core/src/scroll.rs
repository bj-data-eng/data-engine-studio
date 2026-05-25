use crate::element::ElementId;
use crate::geometry::{ClipRect, Rect, ScrollAxis, Size};
use crate::layout::{from_scroll_rect, to_layout_insets, to_scroll_axis, to_scroll_rect};
use crate::state::{ElementState, ResolvedElement, ScrollChrome};
use des_layout::scroll::{self as layout_scroll, ScrollbarGeometryInput};
use std::collections::HashMap;

pub(crate) fn scroll_chrome(
    frame: &ResolvedElement,
    states: &HashMap<ElementId, ElementState>,
    scroll_limits: &HashMap<ElementId, Size>,
) -> Vec<ScrollChrome> {
    let mut chrome = Vec::new();
    collect_scroll_chrome(frame, states, scroll_limits, &mut chrome);
    chrome
}

fn collect_scroll_chrome(
    frame: &ResolvedElement,
    states: &HashMap<ElementId, ElementState>,
    scroll_limits: &HashMap<ElementId, Size>,
    chrome: &mut Vec<ScrollChrome>,
) {
    if let Some(max_scroll) = scroll_limits.get(&frame.id).copied() {
        let clip_rect = scroll_geometry_clip(frame.clip_rect);
        if frame.style.overflow_y.is_scrollable()
            && max_scroll.height > 0.0
            && let Some(scroll_chrome) = scroll_chrome_for_frame(
                frame,
                states,
                ScrollAxis::Vertical,
                max_scroll.height,
                clip_rect,
            )
        {
            chrome.push(scroll_chrome);
        }
        if frame.style.overflow_x.is_scrollable()
            && max_scroll.width > 0.0
            && let Some(scroll_chrome) = scroll_chrome_for_frame(
                frame,
                states,
                ScrollAxis::Horizontal,
                max_scroll.width,
                clip_rect,
            )
        {
            chrome.push(scroll_chrome);
        }
    }

    for child in &frame.children {
        collect_scroll_chrome(child, states, scroll_limits, chrome);
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
        axis: to_scroll_axis(axis),
        viewport_rect: layout_scroll::viewport_rect(
            to_scroll_rect(frame.rect),
            to_layout_insets(frame.style.border_width),
            to_layout_insets(frame.style.padding),
        ),
        max_scroll,
        scroll_offset: state_scroll,
        visual_width,
        hit_width: HIT_WIDTH,
        min_handle_length: MIN_HANDLE_LENGTH,
        clip_rect: clip_rect.map(to_scroll_rect),
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
        track_rect: from_scroll_rect(geometry.track_rect),
        hit_rect: from_scroll_rect(geometry.hit_rect),
        handle_rect: from_scroll_rect(geometry.handle_rect),
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

fn scroll_geometry_clip(clip: ClipRect) -> Option<Rect> {
    if clip == ClipRect::UNBOUNDED {
        return None;
    }

    let left = clip.left.unwrap_or(f32::NEG_INFINITY);
    let top = clip.top.unwrap_or(f32::NEG_INFINITY);
    let right = clip.right.unwrap_or(f32::INFINITY);
    let bottom = clip.bottom.unwrap_or(f32::INFINITY);
    Some(Rect::new(left, top, right - left, bottom - top))
}
