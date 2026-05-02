use crate::element::{Element, ElementId, ElementRole, ElementStateSelector};
use crate::geometry::{Direction, Insets, Overflow, Point, Rect, Size};
use crate::state::{ElementState, LayoutFrame};
use crate::style::{ComputedStyle, StyleSelector, StyleSheet};
use std::collections::HashMap;

pub(crate) fn layout_element(
    element: &Element,
    parent_rect: Rect,
    stylesheet: &StyleSheet,
    states: &HashMap<ElementId, ElementState>,
    scroll_limits: &mut HashMap<ElementId, f32>,
) -> LayoutFrame {
    let style = resolve_style(element, stylesheet, states.get(&element.id));
    let style = states
        .get(&element.id)
        .and_then(|state| state.rendered_style.clone())
        .unwrap_or(style);
    let rect = element_rect(element, &style, parent_rect, stylesheet, states);
    let inner_rect = rect.inset(Insets::all(style.border_width));
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
        stylesheet,
        states,
        scroll_limits,
    );

    LayoutFrame {
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

pub(crate) fn resolve_style(
    element: &Element,
    stylesheet: &StyleSheet,
    state: Option<&ElementState>,
) -> ComputedStyle {
    let mut style = ComputedStyle::default();

    for rule in &stylesheet.rules {
        if selector_matches(rule.selector, element, state) {
            style.apply(&rule.patch);
        }
    }

    style
}

fn selector_matches(
    selector: StyleSelector,
    element: &Element,
    state: Option<&ElementState>,
) -> bool {
    match selector {
        StyleSelector::Role(role) => element.spec.role == role,
        StyleSelector::Class(class) => element
            .spec
            .classes
            .iter()
            .any(|element_class| element_class.as_str() == class),
        StyleSelector::Id(id) => element.id.as_str() == id,
        StyleSelector::State(selector) => match selector {
            ElementStateSelector::Hovered => state.is_some_and(|state| state.hovered),
            ElementStateSelector::Pressed => state.is_some_and(|state| state.pressed),
            ElementStateSelector::Focused => state.is_some_and(|state| state.focused),
            ElementStateSelector::Selected => element.spec.selected,
            ElementStateSelector::Disabled => element.spec.disabled,
        },
        StyleSelector::ClassState(class, selector) => {
            element
                .spec
                .classes
                .iter()
                .any(|element_class| element_class.as_str() == class)
                && selector_matches(StyleSelector::State(selector), element, state)
        }
        StyleSelector::IdState(id, selector) => {
            element.id.as_str() == id
                && selector_matches(StyleSelector::State(selector), element, state)
        }
    }
}

fn element_rect(
    element: &Element,
    style: &ComputedStyle,
    parent_rect: Rect,
    stylesheet: &StyleSheet,
    states: &HashMap<ElementId, ElementState>,
) -> Rect {
    if element.spec.role == ElementRole::Root {
        return parent_rect;
    }

    let measured = measure_element(element, style, parent_rect.size, stylesheet, states);
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
    stylesheet: &StyleSheet,
    states: &HashMap<ElementId, ElementState>,
    scroll_limits: &mut HashMap<ElementId, f32>,
) -> Vec<LayoutFrame> {
    let mut cursor = content_rect.origin;
    let mut frames = Vec::with_capacity(element.children.len());

    for child in &element.children {
        let child_style = resolve_style(child, stylesheet, states.get(&child.id));
        let child_available = Size::new(
            (content_rect.size.width - child_style.margin.horizontal()).max(0.0),
            (content_rect.size.height - child_style.margin.vertical()).max(0.0),
        );
        let measured = measure_element(child, &child_style, child_available, stylesheet, states);
        let child_rect = Rect::new(
            cursor.x,
            cursor.y,
            child_available.width,
            child_available.height,
        );
        frames.push(layout_element(
            child,
            child_rect,
            stylesheet,
            states,
            scroll_limits,
        ));

        match style.direction {
            Direction::Column => {
                cursor.y += measured.height + child_style.margin.vertical() + style.gap
            }
            Direction::Row => {
                cursor.x += measured.width + child_style.margin.horizontal() + style.gap
            }
        }
    }

    frames
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
            let content = measure_children(element, style, parent_size, stylesheet, states);
            Size::new(
                content.width + style.padding.horizontal() + style.border_width * 2.0,
                content.height + style.padding.vertical() + style.border_width * 2.0,
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

fn measure_children(
    element: &Element,
    style: &ComputedStyle,
    parent_size: Size,
    stylesheet: &StyleSheet,
    states: &HashMap<ElementId, ElementState>,
) -> Size {
    let mut width: f32 = 0.0;
    let mut height: f32 = 0.0;
    let child_count = element.children.len();

    for child in &element.children {
        let child_style = resolve_style(child, stylesheet, states.get(&child.id));
        let child_available = Size::new(
            (parent_size.width - child_style.margin.horizontal()).max(0.0),
            (parent_size.height - child_style.margin.vertical()).max(0.0),
        );
        let child_size = measure_element(child, &child_style, child_available, stylesheet, states);
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

    if child_count > 1 {
        let gap = style.gap * (child_count - 1) as f32;
        match style.direction {
            Direction::Column => height += gap,
            Direction::Row => width += gap,
        }
    }

    Size::new(width.min(parent_size.width), height)
}

pub(crate) fn hit_path(frame: &LayoutFrame, point: Point) -> Option<Vec<&LayoutFrame>> {
    if !frame.rect.contains(point) {
        return None;
    }

    let mut children: Vec<_> = frame.children.iter().collect();
    children.sort_by_key(|child| child.style.z_index);

    let mut path = vec![frame];
    if let Some(mut child_path) = children
        .into_iter()
        .rev()
        .find_map(|child| hit_path(child, point))
    {
        path.append(&mut child_path);
    }

    Some(path)
}
