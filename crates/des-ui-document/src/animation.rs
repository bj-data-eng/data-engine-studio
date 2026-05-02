use crate::element::{Color, Element, ElementId};
use crate::geometry::{CornerRadii, Insets, Length, Size};
use crate::state::ElementState;
use crate::style::{ComputedStyle, StyleSheet, Transition, resolve_style};
use std::collections::HashMap;

pub(crate) fn update_element_style_animation(
    element: &Element,
    stylesheet: &StyleSheet,
    states: &mut HashMap<ElementId, ElementState>,
    snap_epsilon: f32,
) -> bool {
    let target_style = resolve_style(element, stylesheet, states.get(&element.id));
    let mut animating = false;

    if let Some(state) = states.get_mut(&element.id) {
        let next_style = match &state.rendered_style {
            Some(current_style) => {
                if let Some(transition) = target_style.transition {
                    let (style, still_animating) =
                        eased_style(current_style, &target_style, transition, snap_epsilon);
                    animating |= still_animating;
                    style
                } else {
                    target_style
                }
            }
            None => target_style,
        };
        state.rendered_style = Some(next_style);
    }

    for child in &element.children {
        animating |= update_element_style_animation(child, stylesheet, states, snap_epsilon);
    }

    animating
}

fn eased_style(
    current: &ComputedStyle,
    target: &ComputedStyle,
    transition: Transition,
    snap_epsilon: f32,
) -> (ComputedStyle, bool) {
    let amount = transition.easing.sample(transition.step.clamp(0.0, 1.0));
    let mut next = target.clone();
    let mut animating = false;

    next.background =
        ease_optional_color(current.background, target.background, amount, snap_epsilon);
    animating |= next.background != target.background;

    next.border = ease_optional_color(current.border, target.border, amount, snap_epsilon);
    animating |= next.border != target.border;

    next.text_color = current.text_color.lerp(target.text_color, amount);
    if color_distance(next.text_color, target.text_color) <= snap_epsilon {
        next.text_color = target.text_color;
    }
    animating |= next.text_color != target.text_color;

    next.gap = ease_f32(current.gap, target.gap, amount, snap_epsilon);
    animating |= (next.gap - target.gap).abs() > snap_epsilon;

    next.margin = ease_insets(current.margin, target.margin, amount, snap_epsilon);
    animating |= next.margin != target.margin;

    next.padding = ease_insets(current.padding, target.padding, amount, snap_epsilon);
    animating |= next.padding != target.padding;

    next.width = ease_length(current.width, target.width, amount, snap_epsilon);
    animating |= next.width != target.width;

    next.height = ease_length(current.height, target.height, amount, snap_epsilon);
    animating |= next.height != target.height;

    next.min_size = ease_size(current.min_size, target.min_size, amount, snap_epsilon);
    animating |= next.min_size != target.min_size;

    next.max_size = ease_size(current.max_size, target.max_size, amount, snap_epsilon);
    animating |= next.max_size != target.max_size;

    next.border_width = ease_insets(
        current.border_width,
        target.border_width,
        amount,
        snap_epsilon,
    );
    animating |= next.border_width != target.border_width;

    next.radius = ease_corner_radii(current.radius, target.radius, amount, snap_epsilon);
    animating |= next.radius != target.radius;

    next.font_size = ease_f32(current.font_size, target.font_size, amount, snap_epsilon);
    animating |= (next.font_size - target.font_size).abs() > snap_epsilon;

    next.scrollbar_width = ease_f32(
        current.scrollbar_width,
        target.scrollbar_width,
        amount,
        snap_epsilon,
    );
    animating |= (next.scrollbar_width - target.scrollbar_width).abs() > snap_epsilon;

    next.scrollbar_handle_color = current
        .scrollbar_handle_color
        .lerp(target.scrollbar_handle_color, amount);
    if color_distance(next.scrollbar_handle_color, target.scrollbar_handle_color) <= snap_epsilon {
        next.scrollbar_handle_color = target.scrollbar_handle_color;
    }
    animating |= next.scrollbar_handle_color != target.scrollbar_handle_color;

    next.scrollbar_track_color = ease_optional_color(
        current.scrollbar_track_color,
        target.scrollbar_track_color,
        amount,
        snap_epsilon,
    );
    animating |= next.scrollbar_track_color != target.scrollbar_track_color;

    next.scrollbar_handle_border_color = ease_optional_color(
        current.scrollbar_handle_border_color,
        target.scrollbar_handle_border_color,
        amount,
        snap_epsilon,
    );
    animating |= next.scrollbar_handle_border_color != target.scrollbar_handle_border_color;

    next.scrollbar_handle_border_width = ease_f32(
        current.scrollbar_handle_border_width,
        target.scrollbar_handle_border_width,
        amount,
        snap_epsilon,
    );
    animating |= (next.scrollbar_handle_border_width - target.scrollbar_handle_border_width).abs()
        > snap_epsilon;

    next.scrollbar_radius = ease_f32(
        current.scrollbar_radius,
        target.scrollbar_radius,
        amount,
        snap_epsilon,
    );
    animating |= (next.scrollbar_radius - target.scrollbar_radius).abs() > snap_epsilon;

    (next, animating)
}

fn ease_optional_color(
    current: Option<Color>,
    target: Option<Color>,
    amount: f32,
    snap_epsilon: f32,
) -> Option<Color> {
    match (current, target) {
        (Some(current), Some(target)) => {
            let next = current.lerp(target, amount);
            if color_distance(next, target) <= snap_epsilon {
                Some(target)
            } else {
                Some(next)
            }
        }
        (None, Some(target)) => {
            let next = Color { a: 0, ..target }.lerp(target, amount);
            if color_distance(next, target) <= snap_epsilon {
                Some(target)
            } else {
                Some(next)
            }
        }
        (Some(current), None) => {
            let transparent = Color { a: 0, ..current };
            let next = current.lerp(transparent, amount);
            if color_distance(next, transparent) <= snap_epsilon {
                None
            } else {
                Some(next)
            }
        }
        (None, None) => None,
    }
}

fn color_distance(left: Color, right: Color) -> f32 {
    (left.r as f32 - right.r as f32).abs()
        + (left.g as f32 - right.g as f32).abs()
        + (left.b as f32 - right.b as f32).abs()
        + (left.a as f32 - right.a as f32).abs()
}

fn ease_f32(current: f32, target: f32, amount: f32, snap_epsilon: f32) -> f32 {
    if current == target {
        return target;
    }
    if !current.is_finite() || !target.is_finite() {
        return target;
    }

    let next = current + (target - current) * amount;
    if (next - target).abs() <= snap_epsilon {
        target
    } else {
        next
    }
}

fn ease_insets(current: Insets, target: Insets, amount: f32, snap_epsilon: f32) -> Insets {
    Insets {
        top: ease_f32(current.top, target.top, amount, snap_epsilon),
        right: ease_f32(current.right, target.right, amount, snap_epsilon),
        bottom: ease_f32(current.bottom, target.bottom, amount, snap_epsilon),
        left: ease_f32(current.left, target.left, amount, snap_epsilon),
    }
}

fn ease_size(current: Size, target: Size, amount: f32, snap_epsilon: f32) -> Size {
    Size {
        width: ease_f32(current.width, target.width, amount, snap_epsilon),
        height: ease_f32(current.height, target.height, amount, snap_epsilon),
    }
}

fn ease_length(current: Length, target: Length, amount: f32, snap_epsilon: f32) -> Length {
    match (current, target) {
        (Length::Px(current), Length::Px(target)) => {
            Length::Px(ease_f32(current, target, amount, snap_epsilon))
        }
        (Length::Percent(current), Length::Percent(target)) => {
            Length::Percent(ease_f32(current, target, amount, snap_epsilon))
        }
        _ => target,
    }
}

fn ease_corner_radii(
    current: CornerRadii,
    target: CornerRadii,
    amount: f32,
    snap_epsilon: f32,
) -> CornerRadii {
    CornerRadii {
        top_left: ease_f32(current.top_left, target.top_left, amount, snap_epsilon),
        top_right: ease_f32(current.top_right, target.top_right, amount, snap_epsilon),
        bottom_right: ease_f32(
            current.bottom_right,
            target.bottom_right,
            amount,
            snap_epsilon,
        ),
        bottom_left: ease_f32(
            current.bottom_left,
            target.bottom_left,
            amount,
            snap_epsilon,
        ),
    }
}
