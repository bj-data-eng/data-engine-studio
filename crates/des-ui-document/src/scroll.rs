use crate::element::ElementId;
use crate::geometry::{Overflow, Rect};
use crate::state::{ElementState, ResolvedElement, ScrollChrome};
use std::collections::HashMap;

pub(crate) fn scroll_chrome(
    frame: &ResolvedElement,
    states: &HashMap<ElementId, ElementState>,
    scroll_limits: &HashMap<ElementId, f32>,
) -> Vec<ScrollChrome> {
    let mut chrome = Vec::new();
    collect_scroll_chrome(frame, states, scroll_limits, &mut chrome);
    chrome
}

fn collect_scroll_chrome(
    frame: &ResolvedElement,
    states: &HashMap<ElementId, ElementState>,
    scroll_limits: &HashMap<ElementId, f32>,
    chrome: &mut Vec<ScrollChrome>,
) {
    if frame.style.overflow_y == Overflow::Scroll {
        let max_scroll = scroll_limits.get(&frame.id).copied().unwrap_or_default();
        if max_scroll > 0.0 {
            chrome.push(scroll_chrome_for_frame(frame, states, max_scroll));
        }
    }

    for child in &frame.children {
        collect_scroll_chrome(child, states, scroll_limits, chrome);
    }
}

fn scroll_chrome_for_frame(
    frame: &ResolvedElement,
    states: &HashMap<ElementId, ElementState>,
    max_scroll: f32,
) -> ScrollChrome {
    const BAR_WIDTH: f32 = 10.0;
    const IDLE_WIDTH: f32 = 2.0;
    const HIT_WIDTH: f32 = 12.0;
    const MIN_HANDLE_LENGTH: f32 = 18.0;

    let state = states.get(&frame.id);
    let container_hovered = state.is_some_and(|state| state.hovered);
    let scrollbar_hovered = state.is_some_and(|state| state.scrollbar_hovered);
    let dragged = state.is_some_and(|state| state.scrollbar_dragged);
    let visible = container_hovered || scrollbar_hovered || dragged;
    let expanded = scrollbar_hovered || dragged;
    let visual_width = if expanded { BAR_WIDTH } else { IDLE_WIDTH };
    let viewport_rect = frame
        .rect
        .inset(frame.style.border_width)
        .inset(frame.style.padding);
    let content_height = viewport_rect.size.height + max_scroll;
    let handle_height = if content_height <= f32::EPSILON {
        viewport_rect.size.height
    } else {
        (viewport_rect.size.height / content_height * viewport_rect.size.height)
            .max(MIN_HANDLE_LENGTH)
            .min(viewport_rect.size.height)
    };
    let state_scroll = state.map(|state| state.scroll_y).unwrap_or_default();
    let track_travel = (viewport_rect.size.height - handle_height).max(0.0);
    let handle_top = if max_scroll <= f32::EPSILON {
        viewport_rect.origin.y
    } else {
        viewport_rect.origin.y + (state_scroll / max_scroll).clamp(0.0, 1.0) * track_travel
    };
    let track_rect = Rect::new(
        viewport_rect.right() - visual_width,
        viewport_rect.origin.y,
        visual_width,
        viewport_rect.size.height,
    );
    let hit_rect = Rect::new(
        viewport_rect.right() - HIT_WIDTH,
        viewport_rect.origin.y,
        HIT_WIDTH,
        viewport_rect.size.height,
    );
    let handle_rect = Rect::new(
        viewport_rect.right() - visual_width,
        handle_top,
        visual_width,
        handle_height,
    );

    ScrollChrome {
        element_id: frame.id.clone(),
        track_rect,
        hit_rect,
        handle_rect,
        max_scroll,
        visible,
        expanded,
        hovered: scrollbar_hovered,
        dragged,
    }
}
