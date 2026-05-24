use crate::element::{Color, DocumentNode, ElementId};
use crate::geometry::{CornerRadii, Insets, Length, Point, Size};
use crate::state::ElementState;
use crate::style::{
    ChildPosition, ComputedStyle, Shadow, StyleInvalidation, StyleMatchContext, StyleSheet,
    Transition, classify_computed_style_change, resolve_style_with_position,
};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct AnimationUpdate {
    pub paint_changed: bool,
    pub layout_changed: bool,
    pub animating: bool,
}

impl AnimationUpdate {
    pub(crate) fn changed(self) -> bool {
        self.paint_changed || self.layout_changed
    }
}

pub(crate) fn update_element_style_animation(
    element: &DocumentNode,
    stylesheet: &StyleSheet,
    states: &mut HashMap<ElementId, ElementState>,
    snap_epsilon: f32,
    viewport: Size,
    container_size: &dyn Fn(&ElementId) -> Option<Size>,
) -> AnimationUpdate {
    update_element_style_animation_at(
        element,
        stylesheet,
        states,
        snap_epsilon,
        None,
        &mut Vec::new(),
        viewport,
        container_size,
    )
}

fn update_element_style_animation_at<'a>(
    element: &'a DocumentNode,
    stylesheet: &StyleSheet,
    states: &mut HashMap<ElementId, ElementState>,
    snap_epsilon: f32,
    position: Option<ChildPosition>,
    ancestors: &mut Vec<(&'a DocumentNode, Option<ChildPosition>)>,
    viewport: Size,
    container_size: &dyn Fn(&ElementId) -> Option<Size>,
) -> AnimationUpdate {
    let target_style = {
        let ancestor_contexts = ancestors
            .iter()
            .map(|(element, position)| StyleMatchContext {
                element,
                state: states.get(&element.id),
                position: *position,
            })
            .collect::<Vec<_>>();
        resolve_style_with_position(
            element,
            stylesheet,
            states.get(&element.id),
            position,
            &ancestor_contexts,
            viewport,
            container_size(&element.id),
        )
    };
    let mut update = AnimationUpdate::default();

    if let Some(state) = states.get_mut(&element.id) {
        let previous = state.rendered_style.clone();
        let next_style = match (state.rendered_style.as_ref(), target_style.transition) {
            (Some(current_style), Some(_)) if current_style == &target_style => Some(target_style),
            (Some(current_style), Some(transition)) => {
                let (style, still_animating) =
                    eased_style(current_style, &target_style, transition, snap_epsilon);
                update.animating |= still_animating;
                Some(style)
            }
            (None, Some(_)) => Some(target_style),
            (_, None) => None,
        };
        state.rendered_style = next_style;
        update += classify_computed_style_change(previous.as_ref(), state.rendered_style.as_ref());
    }

    ancestors.push((element, position));
    for (index, child) in element.children.iter().enumerate() {
        update += update_element_style_animation_at(
            child,
            stylesheet,
            states,
            snap_epsilon,
            Some(ChildPosition::new(index, element.children.len())),
            ancestors,
            viewport,
            container_size,
        );
    }
    ancestors.pop();

    update
}

impl std::ops::AddAssign for AnimationUpdate {
    fn add_assign(&mut self, rhs: Self) {
        self.paint_changed |= rhs.paint_changed;
        self.layout_changed |= rhs.layout_changed;
        self.animating |= rhs.animating;
    }
}

impl std::ops::AddAssign<StyleInvalidation> for AnimationUpdate {
    fn add_assign(&mut self, rhs: StyleInvalidation) {
        self.paint_changed |= rhs.paint_changed;
        self.layout_changed |= rhs.layout_changed;
    }
}

fn eased_style(
    current: &ComputedStyle,
    target: &ComputedStyle,
    transition: Transition,
    snap_epsilon: f32,
) -> (ComputedStyle, bool) {
    if current == target {
        return (target.clone(), false);
    }

    let amount = transition.easing.sample(transition.step.clamp(0.0, 1.0));
    let mut next = target.clone();
    let mut animating = false;

    if target.animate_paint {
        next.background =
            ease_optional_color(current.background, target.background, amount, snap_epsilon);
        animating |= next.background != target.background;

        next.border = ease_optional_color(current.border, target.border, amount, snap_epsilon);
        animating |= next.border != target.border;

        if target.animate_shadows {
            next.shadows = ease_shadows(&current.shadows, &target.shadows, amount, snap_epsilon);
            animating |= next.shadows != target.shadows;
        } else {
            next.shadows = target.shadows.clone();
        }

        next.text_color = ease_color(current.text_color, target.text_color, amount, snap_epsilon);
        animating |= next.text_color != target.text_color;

        next.text_selection_background = ease_color(
            current.text_selection_background,
            target.text_selection_background,
            amount,
            snap_epsilon,
        );
        animating |= next.text_selection_background != target.text_selection_background;

        next.text_selection_color = ease_color(
            current.text_selection_color,
            target.text_selection_color,
            amount,
            snap_epsilon,
        );
        animating |= next.text_selection_color != target.text_selection_color;
    } else {
        next.background = target.background;
        next.border = target.border;
        next.shadows = target.shadows.clone();
        next.text_color = target.text_color;
        next.text_selection_background = target.text_selection_background;
        next.text_selection_color = target.text_selection_color;
    }

    next.gap = ease_length(current.gap, target.gap, amount, snap_epsilon);
    animating |= next.gap != target.gap;

    next.row_gap = ease_length(current.row_gap, target.row_gap, amount, snap_epsilon);
    animating |= next.row_gap != target.row_gap;

    next.column_gap = ease_length(current.column_gap, target.column_gap, amount, snap_epsilon);
    animating |= next.column_gap != target.column_gap;

    next.flex_basis = ease_length(current.flex_basis, target.flex_basis, amount, snap_epsilon);
    animating |= next.flex_basis != target.flex_basis;

    next.flex_grow = ease_f32(current.flex_grow, target.flex_grow, amount, snap_epsilon);
    animating |= (next.flex_grow - target.flex_grow).abs() > snap_epsilon;

    next.flex_shrink = ease_f32(
        current.flex_shrink,
        target.flex_shrink,
        amount,
        snap_epsilon,
    );
    animating |= (next.flex_shrink - target.flex_shrink).abs() > snap_epsilon;

    next.margin = ease_insets(current.margin, target.margin, amount, snap_epsilon);
    animating |= next.margin != target.margin;

    next.padding = ease_insets(current.padding, target.padding, amount, snap_epsilon);
    animating |= next.padding != target.padding;

    if target.animate_size {
        next.width = ease_length(current.width, target.width, amount, snap_epsilon);
        animating |= next.width != target.width;

        next.height = ease_length(current.height, target.height, amount, snap_epsilon);
        animating |= next.height != target.height;

        next.min_size = ease_size(current.min_size, target.min_size, amount, snap_epsilon);
        animating |= next.min_size != target.min_size;

        next.max_size = ease_size(current.max_size, target.max_size, amount, snap_epsilon);
        animating |= next.max_size != target.max_size;
    } else {
        next.width = target.width;
        next.height = target.height;
        next.min_size = target.min_size;
        next.max_size = target.max_size;
    }

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

    next.scrollbar_expanded_width = ease_f32(
        current.scrollbar_expanded_width,
        target.scrollbar_expanded_width,
        amount,
        snap_epsilon,
    );
    animating |=
        (next.scrollbar_expanded_width - target.scrollbar_expanded_width).abs() > snap_epsilon;

    next.scrollbar_handle_color = ease_color(
        current.scrollbar_handle_color,
        target.scrollbar_handle_color,
        amount,
        snap_epsilon,
    );
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

    next.scrollbar_hover_handle_color = ease_optional_color(
        current.scrollbar_hover_handle_color,
        target.scrollbar_hover_handle_color,
        amount,
        snap_epsilon,
    );
    animating |= next.scrollbar_hover_handle_color != target.scrollbar_hover_handle_color;

    next.scrollbar_hover_track_color = ease_optional_color(
        current.scrollbar_hover_track_color,
        target.scrollbar_hover_track_color,
        amount,
        snap_epsilon,
    );
    animating |= next.scrollbar_hover_track_color != target.scrollbar_hover_track_color;

    next.scrollbar_hover_handle_border_color = ease_optional_color(
        current.scrollbar_hover_handle_border_color,
        target.scrollbar_hover_handle_border_color,
        amount,
        snap_epsilon,
    );
    animating |=
        next.scrollbar_hover_handle_border_color != target.scrollbar_hover_handle_border_color;

    next.scrollbar_hover_handle_border_width = ease_optional_f32(
        current.scrollbar_hover_handle_border_width,
        target.scrollbar_hover_handle_border_width,
        amount,
        snap_epsilon,
    );
    animating |=
        next.scrollbar_hover_handle_border_width != target.scrollbar_hover_handle_border_width;

    next.scrollbar_pressed_handle_color = ease_optional_color(
        current.scrollbar_pressed_handle_color,
        target.scrollbar_pressed_handle_color,
        amount,
        snap_epsilon,
    );
    animating |= next.scrollbar_pressed_handle_color != target.scrollbar_pressed_handle_color;

    next.scrollbar_pressed_track_color = ease_optional_color(
        current.scrollbar_pressed_track_color,
        target.scrollbar_pressed_track_color,
        amount,
        snap_epsilon,
    );
    animating |= next.scrollbar_pressed_track_color != target.scrollbar_pressed_track_color;

    next.scrollbar_pressed_handle_border_color = ease_optional_color(
        current.scrollbar_pressed_handle_border_color,
        target.scrollbar_pressed_handle_border_color,
        amount,
        snap_epsilon,
    );
    animating |=
        next.scrollbar_pressed_handle_border_color != target.scrollbar_pressed_handle_border_color;

    next.scrollbar_pressed_handle_border_width = ease_optional_f32(
        current.scrollbar_pressed_handle_border_width,
        target.scrollbar_pressed_handle_border_width,
        amount,
        snap_epsilon,
    );
    animating |=
        next.scrollbar_pressed_handle_border_width != target.scrollbar_pressed_handle_border_width;

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
            let next = ease_color(current, target, amount, snap_epsilon);
            if next == target {
                Some(target)
            } else {
                Some(next)
            }
        }
        (None, Some(target)) => {
            let next = ease_color(Color { a: 0, ..target }, target, amount, snap_epsilon);
            if next == target {
                Some(target)
            } else {
                Some(next)
            }
        }
        (Some(current), None) => {
            let transparent = Color { a: 0, ..current };
            let next = ease_color(current, transparent, amount, snap_epsilon);
            if next == transparent {
                None
            } else {
                Some(next)
            }
        }
        (None, None) => None,
    }
}

fn ease_shadows(
    current: &[Shadow],
    target: &[Shadow],
    amount: f32,
    snap_epsilon: f32,
) -> Vec<Shadow> {
    let count = current.len().max(target.len());
    let mut shadows = Vec::with_capacity(count);
    for index in 0..count {
        match (current.get(index).copied(), target.get(index).copied()) {
            (Some(current), Some(target)) => {
                shadows.push(ease_shadow(current, target, amount, snap_epsilon));
            }
            (None, Some(target)) => {
                shadows.push(ease_shadow(
                    Shadow {
                        color: Color {
                            a: 0,
                            ..target.color
                        },
                        ..target
                    },
                    target,
                    amount,
                    snap_epsilon,
                ));
            }
            (Some(current), None) => {
                let transparent = Shadow {
                    color: Color {
                        a: 0,
                        ..current.color
                    },
                    ..current
                };
                let next = ease_shadow(current, transparent, amount, snap_epsilon);
                if next != transparent {
                    shadows.push(next);
                }
            }
            (None, None) => {}
        }
    }
    shadows
}

fn ease_shadow(current: Shadow, target: Shadow, amount: f32, snap_epsilon: f32) -> Shadow {
    Shadow {
        offset: Point::new(
            ease_f32(current.offset.x, target.offset.x, amount, snap_epsilon),
            ease_f32(current.offset.y, target.offset.y, amount, snap_epsilon),
        ),
        blur: ease_f32(current.blur, target.blur, amount, snap_epsilon),
        spread: ease_f32(current.spread, target.spread, amount, snap_epsilon),
        color: ease_color(current.color, target.color, amount, snap_epsilon),
    }
}

fn ease_color(current: Color, target: Color, amount: f32, snap_epsilon: f32) -> Color {
    if current == target {
        return target;
    }
    if color_distance(current, target) <= snap_epsilon.max(1.0) {
        return target;
    }

    let next = current.lerp(target, amount);
    if next == current { target } else { next }
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

fn ease_optional_f32(
    current: Option<f32>,
    target: Option<f32>,
    amount: f32,
    snap_epsilon: f32,
) -> Option<f32> {
    match (current, target) {
        (Some(current), Some(target)) => Some(ease_f32(current, target, amount, snap_epsilon)),
        (None, Some(target)) => Some(ease_f32(0.0, target, amount, snap_epsilon)),
        (Some(current), None) => {
            let next = ease_f32(current, 0.0, amount, snap_epsilon);
            if next <= snap_epsilon {
                None
            } else {
                Some(next)
            }
        }
        (None, None) => None,
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
        (
            Length::Calc {
                percent: current_percent,
                px: current_px,
            },
            Length::Calc {
                percent: target_percent,
                px: target_px,
            },
        ) => Length::Calc {
            percent: ease_f32(current_percent, target_percent, amount, snap_epsilon),
            px: ease_f32(current_px, target_px, amount, snap_epsilon),
        },
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
