use crate::element::ElementId;
use crate::geometry::{Overflow, Rect, ScrollAxis, Size};
use crate::state::{ElementState, ResolvedElement, ScrollChrome};
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
    let visible = container_hovered || scrollbar_hovered || dragged;
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
    let viewport_rect = frame
        .rect
        .inset(frame.style.border_width)
        .inset(frame.style.padding);
    let viewport_main = match axis {
        ScrollAxis::Horizontal => viewport_rect.size.width,
        ScrollAxis::Vertical => viewport_rect.size.height,
    };
    let content_main = viewport_main + max_scroll;
    let handle_length = if content_main <= f32::EPSILON {
        viewport_main
    } else {
        (viewport_main / content_main * viewport_main)
            .max(MIN_HANDLE_LENGTH)
            .min(viewport_main)
    };
    let state_scroll = state
        .map(|state| match axis {
            ScrollAxis::Horizontal => state.scroll_x,
            ScrollAxis::Vertical => state.scroll_y,
        })
        .unwrap_or_default();
    let track_travel = (viewport_main - handle_length).max(0.0);
    let handle_start = if max_scroll <= f32::EPSILON {
        match axis {
            ScrollAxis::Horizontal => viewport_rect.origin.x,
            ScrollAxis::Vertical => viewport_rect.origin.y,
        }
    } else {
        let origin = match axis {
            ScrollAxis::Horizontal => viewport_rect.origin.x,
            ScrollAxis::Vertical => viewport_rect.origin.y,
        };
        origin + (state_scroll / max_scroll).clamp(0.0, 1.0) * track_travel
    };
    let (track_rect, hit_rect, handle_rect) = match axis {
        ScrollAxis::Horizontal => (
            Rect::new(
                viewport_rect.origin.x,
                viewport_rect.bottom() - visual_width,
                viewport_rect.size.width,
                visual_width,
            ),
            Rect::new(
                viewport_rect.origin.x,
                viewport_rect.bottom() - HIT_WIDTH,
                viewport_rect.size.width,
                HIT_WIDTH,
            ),
            Rect::new(
                handle_start,
                viewport_rect.bottom() - visual_width,
                handle_length,
                visual_width,
            ),
        ),
        ScrollAxis::Vertical => (
            Rect::new(
                viewport_rect.right() - visual_width,
                viewport_rect.origin.y,
                visual_width,
                viewport_rect.size.height,
            ),
            Rect::new(
                viewport_rect.right() - HIT_WIDTH,
                viewport_rect.origin.y,
                HIT_WIDTH,
                viewport_rect.size.height,
            ),
            Rect::new(
                viewport_rect.right() - visual_width,
                handle_start,
                visual_width,
                handle_length,
            ),
        ),
    };
    let (track_rect, hit_rect, handle_rect) = if let Some(clip_rect) = clip_rect {
        (
            track_rect.intersect(clip_rect)?,
            hit_rect.intersect(clip_rect)?,
            handle_rect.intersect(clip_rect)?,
        )
    } else {
        (track_rect, hit_rect, handle_rect)
    };
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
        track_rect,
        hit_rect,
        handle_rect,
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
    if frame.style.overflow_x != Overflow::Scroll && frame.style.overflow_y != Overflow::Scroll {
        return parent_clip;
    }

    let viewport_rect = frame
        .rect
        .inset(frame.style.border_width)
        .inset(frame.style.padding);
    let left = if frame.style.overflow_x == Overflow::Scroll {
        viewport_rect.origin.x
    } else {
        parent_clip.map_or(f32::NEG_INFINITY, |clip| clip.origin.x)
    };
    let right = if frame.style.overflow_x == Overflow::Scroll {
        viewport_rect.right()
    } else {
        parent_clip.map_or(f32::INFINITY, Rect::right)
    };
    let top = if frame.style.overflow_y == Overflow::Scroll {
        viewport_rect.origin.y
    } else {
        parent_clip.map_or(f32::NEG_INFINITY, |clip| clip.origin.y)
    };
    let bottom = if frame.style.overflow_y == Overflow::Scroll {
        viewport_rect.bottom()
    } else {
        parent_clip.map_or(f32::INFINITY, Rect::bottom)
    };

    let scroll_clip = Rect::new(left, top, right - left, bottom - top);
    parent_clip
        .and_then(|clip| clip.intersect(scroll_clip))
        .or(Some(scroll_clip))
}
