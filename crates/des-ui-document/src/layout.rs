use crate::element::{Element, ElementId, ElementRole};
use crate::geometry::{Direction, Length, Overflow, Point, Position, Rect, Size};
use crate::state::{ElementState, ResolvedElement};
use crate::style::{ComputedStyle, StyleSheet, resolve_style};
use std::collections::HashMap;

pub(crate) fn layout_element(
    element: &Element,
    parent_rect: Rect,
    stylesheet: &StyleSheet,
    states: &HashMap<ElementId, ElementState>,
    scroll_limits: &mut HashMap<ElementId, f32>,
) -> ResolvedElement {
    layout_element_in_viewport(
        element,
        parent_rect,
        parent_rect,
        stylesheet,
        states,
        scroll_limits,
    )
}

fn layout_element_in_viewport(
    element: &Element,
    parent_rect: Rect,
    viewport_rect: Rect,
    stylesheet: &StyleSheet,
    states: &HashMap<ElementId, ElementState>,
    scroll_limits: &mut HashMap<ElementId, f32>,
) -> ResolvedElement {
    let style = computed_style_for(element, stylesheet, states);
    let rect = element_rect(
        element,
        &style,
        parent_rect,
        viewport_rect,
        stylesheet,
        states,
    );
    let inner_rect = rect.inset(style.border_width);
    let mut content_rect = inner_rect.inset(style.padding);
    let content_size = measure_children(element, &style, content_rect.size, stylesheet, states);
    if style.overflow_y == Overflow::Scroll {
        scroll_limits.insert(
            element.id.clone(),
            (content_size.height - content_rect.size.height).max(0.0),
        );
    }
    if style.overflow_y == Overflow::Scroll
        && let Some(state) = states.get(&element.id)
    {
        content_rect.origin.y -= state.scroll_y;
    }
    let children = layout_children(
        element,
        &style,
        content_rect,
        viewport_rect,
        stylesheet,
        states,
        scroll_limits,
    );

    ResolvedElement {
        id: element.id.clone(),
        role: element.spec.role,
        classes: element.spec.classes.clone(),
        rect,
        style,
        text: element.text.clone(),
        interactive: element.spec.interactive && !element.spec.disabled,
        children,
    }
}

fn computed_style_for(
    element: &Element,
    stylesheet: &StyleSheet,
    states: &HashMap<ElementId, ElementState>,
) -> ComputedStyle {
    let style = resolve_style(element, stylesheet, states.get(&element.id));
    states
        .get(&element.id)
        .and_then(|state| state.rendered_style.clone())
        .unwrap_or(style)
}

fn element_rect(
    element: &Element,
    style: &ComputedStyle,
    parent_rect: Rect,
    viewport_rect: Rect,
    stylesheet: &StyleSheet,
    states: &HashMap<ElementId, ElementState>,
) -> Rect {
    if element.spec.role == ElementRole::Root {
        return parent_rect;
    }

    let measured = measure_element(element, style, parent_rect.size, stylesheet, states);
    if style.position != Position::Flow {
        let containing_rect = match style.position {
            Position::Flow => parent_rect,
            Position::AbsoluteParent => parent_rect,
            Position::AbsoluteViewport => viewport_rect,
        };
        return positioned_rect(style, containing_rect, measured);
    }

    Rect::new(
        parent_rect.origin.x + style.margin.left,
        parent_rect.origin.y + style.margin.top,
        measured.width,
        measured.height,
    )
}

fn layout_children(
    element: &Element,
    style: &ComputedStyle,
    content_rect: Rect,
    viewport_rect: Rect,
    stylesheet: &StyleSheet,
    states: &HashMap<ElementId, ElementState>,
    scroll_limits: &mut HashMap<ElementId, f32>,
) -> Vec<ResolvedElement> {
    let mut cursor = content_rect.origin;
    let mut line_height: f32 = 0.0;
    let mut frames = Vec::with_capacity(element.children.len());

    for child in &element.children {
        let child_style = computed_style_for(child, stylesheet, states);
        if child_style.position != Position::Flow {
            frames.push(layout_element_in_viewport(
                child,
                content_rect,
                viewport_rect,
                stylesheet,
                states,
                scroll_limits,
            ));
            continue;
        }

        let child_available = Size::new(
            (content_rect.size.width - child_style.margin.horizontal()).max(0.0),
            (content_rect.size.height - child_style.margin.vertical()).max(0.0),
        );
        let measured =
            measure_intrinsic_element(child, &child_style, child_available, stylesheet, states);
        let outer_width = measured.width + child_style.margin.horizontal();
        let outer_height = measured.height + child_style.margin.vertical();

        if style.direction == Direction::Row
            && style.wrap
            && cursor.x > content_rect.origin.x
            && cursor.x + outer_width > content_rect.right()
        {
            cursor.x = content_rect.origin.x;
            cursor.y += line_height + style.gap;
            line_height = 0.0;
        }

        let child_rect = Rect::new(
            cursor.x,
            cursor.y,
            child_available.width,
            child_available.height,
        );
        frames.push(layout_element_in_viewport(
            child,
            child_rect,
            viewport_rect,
            stylesheet,
            states,
            scroll_limits,
        ));

        match style.direction {
            Direction::Column => cursor.y += outer_height + style.gap,
            Direction::Row => {
                cursor.x += outer_width + style.gap;
                line_height = line_height.max(outer_height);
            }
        }
    }

    frames
}

fn positioned_rect(style: &ComputedStyle, containing_rect: Rect, measured: Size) -> Rect {
    let available = containing_rect.size;
    let left = style
        .inset
        .left
        .map(|value| value.resolve(available.width, 0.0));
    let right = style
        .inset
        .right
        .map(|value| value.resolve(available.width, 0.0));
    let top = style
        .inset
        .top
        .map(|value| value.resolve(available.height, 0.0));
    let bottom = style
        .inset
        .bottom
        .map(|value| value.resolve(available.height, 0.0));

    let x = if let Some(left) = left {
        containing_rect.origin.x + left + style.margin.left
    } else if let Some(right) = right {
        containing_rect.right() - right - measured.width - style.margin.right
    } else {
        containing_rect.origin.x + style.margin.left
    };

    let y = if let Some(top) = top {
        containing_rect.origin.y + top + style.margin.top
    } else if let Some(bottom) = bottom {
        containing_rect.bottom() - bottom - measured.height - style.margin.bottom
    } else {
        containing_rect.origin.y + style.margin.top
    };

    Rect::new(x, y, measured.width, measured.height)
}

fn measure_element(
    element: &Element,
    style: &ComputedStyle,
    parent_size: Size,
    stylesheet: &StyleSheet,
    states: &HashMap<ElementId, ElementState>,
) -> Size {
    let auto_size = match element.spec.role {
        ElementRole::Text => {
            let width = element
                .text
                .as_ref()
                .map(|text| text.chars().count() as f32 * 7.5)
                .unwrap_or_default();
            Size::new(width.max(style.min_size.width), 18.0)
        }
        _ => {
            let content_parent_size = measurement_content_size(style, parent_size);
            let content = measure_children(element, style, content_parent_size, stylesheet, states);
            Size::new(
                content.width + style.padding.horizontal() + style.border_width.horizontal(),
                content.height + style.padding.vertical() + style.border_width.vertical(),
            )
        }
    };

    Size::new(
        style
            .width
            .resolve(parent_size.width, auto_size.width)
            .max(style.min_size.width),
        style
            .height
            .resolve(parent_size.height, auto_size.height)
            .max(style.min_size.height),
    )
}

fn measure_intrinsic_element(
    element: &Element,
    style: &ComputedStyle,
    parent_size: Size,
    stylesheet: &StyleSheet,
    states: &HashMap<ElementId, ElementState>,
) -> Size {
    let auto_size = match element.spec.role {
        ElementRole::Text => {
            let width = element
                .text
                .as_ref()
                .map(|text| text.chars().count() as f32 * 7.5)
                .unwrap_or_default();
            Size::new(width.max(style.min_size.width), 18.0)
        }
        _ => {
            let content_parent_size = measurement_content_size(style, parent_size);
            let content = measure_children(element, style, content_parent_size, stylesheet, states);
            Size::new(
                content.width + style.padding.horizontal() + style.border_width.horizontal(),
                content.height + style.padding.vertical() + style.border_width.vertical(),
            )
        }
    };

    Size::new(
        style
            .width
            .resolve_intrinsic(parent_size.width, auto_size.width)
            .max(style.min_size.width),
        style
            .height
            .resolve_intrinsic(parent_size.height, auto_size.height)
            .max(style.min_size.height),
    )
}

fn measurement_content_size(style: &ComputedStyle, parent_size: Size) -> Size {
    let width = match style.width {
        Length::Auto => parent_size.width,
        width => width
            .resolve_intrinsic(parent_size.width, parent_size.width)
            .max(style.min_size.width),
    };
    let height = match style.height {
        Length::Auto => parent_size.height,
        height => height
            .resolve_intrinsic(parent_size.height, parent_size.height)
            .max(style.min_size.height),
    };

    Size::new(
        (width - style.padding.horizontal() - style.border_width.horizontal()).max(0.0),
        (height - style.padding.vertical() - style.border_width.vertical()).max(0.0),
    )
}

fn measure_children(
    element: &Element,
    style: &ComputedStyle,
    parent_size: Size,
    stylesheet: &StyleSheet,
    states: &HashMap<ElementId, ElementState>,
) -> Size {
    if style.direction == Direction::Row && style.wrap {
        return measure_wrapped_children(element, style, parent_size, stylesheet, states);
    }

    let mut width: f32 = 0.0;
    let mut height: f32 = 0.0;
    let mut flow_child_count = 0;

    for child in &element.children {
        let child_style = computed_style_for(child, stylesheet, states);
        if child_style.position != Position::Flow {
            continue;
        }
        flow_child_count += 1;

        let child_available = Size::new(
            (parent_size.width - child_style.margin.horizontal()).max(0.0),
            (parent_size.height - child_style.margin.vertical()).max(0.0),
        );
        let child_size =
            measure_intrinsic_element(child, &child_style, child_available, stylesheet, states);
        let outer_width = child_size.width + child_style.margin.horizontal();
        let outer_height = child_size.height + child_style.margin.vertical();
        match style.direction {
            Direction::Column => {
                width = width.max(outer_width);
                height += outer_height;
            }
            Direction::Row => {
                width += outer_width;
                height = height.max(outer_height);
            }
        }
    }

    if flow_child_count > 1 {
        let gap = style.gap * (flow_child_count - 1) as f32;
        match style.direction {
            Direction::Column => height += gap,
            Direction::Row => width += gap,
        }
    }

    Size::new(width, height)
}

fn measure_wrapped_children(
    element: &Element,
    style: &ComputedStyle,
    parent_size: Size,
    stylesheet: &StyleSheet,
    states: &HashMap<ElementId, ElementState>,
) -> Size {
    let mut width: f32 = 0.0;
    let mut height: f32 = 0.0;
    let mut line_width: f32 = 0.0;
    let mut line_height: f32 = 0.0;
    let mut line_has_child = false;

    for child in &element.children {
        let child_style = computed_style_for(child, stylesheet, states);
        if child_style.position != Position::Flow {
            continue;
        }

        let child_available = Size::new(
            (parent_size.width - child_style.margin.horizontal()).max(0.0),
            (parent_size.height - child_style.margin.vertical()).max(0.0),
        );
        let child_size =
            measure_intrinsic_element(child, &child_style, child_available, stylesheet, states);
        let outer_width = child_size.width + child_style.margin.horizontal();
        let outer_height = child_size.height + child_style.margin.vertical();
        let next_width = if line_has_child {
            line_width + style.gap + outer_width
        } else {
            outer_width
        };

        if line_has_child && next_width > parent_size.width {
            width = width.max(line_width);
            height += line_height + style.gap;
            line_width = outer_width;
            line_height = outer_height;
        } else {
            line_width = next_width;
            line_height = line_height.max(outer_height);
            line_has_child = true;
        }
    }

    if line_has_child {
        width = width.max(line_width);
        height += line_height;
    }

    Size::new(width, height)
}

pub(crate) fn hit_path(frame: &ResolvedElement, point: Point) -> Option<Vec<&ResolvedElement>> {
    let mut children: Vec<_> = frame.children.iter().collect();
    children.sort_by_key(|child| child.style.z_index);

    let may_hit_children = frame.style.overflow_y != Overflow::Scroll || frame.rect.contains(point);
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
