use crate::element::{Color, Element, ElementId};
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

    next.border_width = ease_f32(
        current.border_width,
        target.border_width,
        amount,
        snap_epsilon,
    );
    animating |= (next.border_width - target.border_width).abs() > snap_epsilon;

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
    let next = current + (target - current) * amount;
    if (next - target).abs() <= snap_epsilon {
        target
    } else {
        next
    }
}
