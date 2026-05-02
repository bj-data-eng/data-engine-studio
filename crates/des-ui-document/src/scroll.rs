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
        if frame.style.overflow_y == Overflow::Scroll && max_scroll.height > 0.0 {
            chrome.push(scroll_chrome_for_frame(
                frame,
                states,
                ScrollAxis::Vertical,
                max_scroll.height,
            ));
        }
        if frame.style.overflow_x == Overflow::Scroll && max_scroll.width > 0.0 {
            chrome.push(scroll_chrome_for_frame(
                frame,
                states,
                ScrollAxis::Horizontal,
                max_scroll.width,
            ));
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
) -> ScrollChrome {
    const HIT_WIDTH: f32 = 12.0;
    const MIN_HANDLE_LENGTH: f32 = 18.0;

    let state = states.get(&frame.id);
    let container_hovered = state.is_some_and(|state| state.hovered);
    let scrollbar_hovered = state.is_some_and(|state| state.scrollbar_hovered);
    let dragged = state.is_some_and(|state| state.scrollbar_dragged);
    let visible = container_hovered || scrollbar_hovered || dragged;
    let expanded = scrollbar_hovered || dragged;
    let visual_width = frame.style.scrollbar_width.max(0.0);
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

    ScrollChrome {
        element_id: frame.id.clone(),
        axis,
        track_rect,
        hit_rect,
        handle_rect,
        handle_color: frame.style.scrollbar_handle_color,
        track_color: frame.style.scrollbar_track_color,
        handle_border_color: frame.style.scrollbar_handle_border_color,
        handle_border_width: frame.style.scrollbar_handle_border_width,
        radius: frame.style.scrollbar_radius,
        max_scroll,
        visible,
        expanded,
        hovered: scrollbar_hovered,
        dragged,
    }
}
