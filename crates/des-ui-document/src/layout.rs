use crate::element::{Element, ElementId, ElementRole};
use crate::geometry::{
    AlignItems, Direction, JustifyContent, Length, Overflow, Point, Position, Rect, Size,
};
use crate::state::{ElementState, ResolvedElement};
use crate::style::{
    AnchorPlacement, ChildPosition, ComputedStyle, StyleSheet, resolve_style_with_position,
};
use crate::table::{TableSpec, TableTrackSize};
use crate::text::{TextLayoutRequest, TextLayoutResult, TextMeasurer};
use std::collections::HashMap;

pub(crate) fn layout_element(
    element: &Element,
    parent_rect: Rect,
    stylesheet: &StyleSheet,
    states: &HashMap<ElementId, ElementState>,
    scroll_limits: &mut HashMap<ElementId, Size>,
    text_measurer: &mut dyn TextMeasurer,
) -> ResolvedElement {
    let anchors = HashMap::new();
    layout_element_in_viewport(
        element,
        parent_rect,
        parent_rect,
        stylesheet,
        states,
        scroll_limits,
        text_measurer,
        &anchors,
        None,
    )
}

fn layout_element_in_viewport(
    element: &Element,
    parent_rect: Rect,
    viewport_rect: Rect,
    stylesheet: &StyleSheet,
    states: &HashMap<ElementId, ElementState>,
    scroll_limits: &mut HashMap<ElementId, Size>,
    text_measurer: &mut dyn TextMeasurer,
    anchors: &HashMap<ElementId, Rect>,
    position: Option<ChildPosition>,
) -> ResolvedElement {
    let style = computed_style_for(element, stylesheet, states, position);
    let rect = element_rect(
        element,
        &style,
        parent_rect,
        viewport_rect,
        stylesheet,
        states,
        anchors,
        text_measurer,
    );
    let inner_rect = rect.inset(style.border_width);
    let mut content_rect = inner_rect.inset(style.padding);
    let content_size = measure_children(
        element,
        &style,
        content_rect.size,
        stylesheet,
        states,
        text_measurer,
    );
    if style.overflow_x == Overflow::Scroll || style.overflow_y == Overflow::Scroll {
        scroll_limits.insert(
            element.id.clone(),
            Size::new(
                (content_size.width - content_rect.size.width).max(0.0),
                (content_size.height - content_rect.size.height).max(0.0),
            ),
        );
    }
    if let Some(state) = states.get(&element.id) {
        if style.overflow_x == Overflow::Scroll {
            content_rect.origin.x -= state.scroll_x;
        }
        if style.overflow_y == Overflow::Scroll {
            content_rect.origin.y -= state.scroll_y;
        }
    }
    let children = layout_children(
        element,
        &style,
        content_rect,
        viewport_rect,
        stylesheet,
        states,
        scroll_limits,
        text_measurer,
        anchors,
    );

    let text_layout = element
        .text
        .as_deref()
        .map(|text| measure_text(text, &style, content_rect.size, text_measurer));

    ResolvedElement {
        id: element.id.clone(),
        role: element.spec.role,
        classes: element.spec.classes.clone(),
        rect,
        style,
        text: element.text.clone(),
        text_layout,
        value: element.spec.value.clone(),
        glyph: element.spec.glyph,
        interactive: element.spec.interactive && !element.spec.disabled,
        children,
    }
}

fn computed_style_for(
    element: &Element,
    stylesheet: &StyleSheet,
    states: &HashMap<ElementId, ElementState>,
    position: Option<ChildPosition>,
) -> ComputedStyle {
    let style = resolve_style_with_position(element, stylesheet, states.get(&element.id), position);
    states
        .get(&element.id)
        .and_then(|state| state.rendered_style.clone())
        .unwrap_or(style)
}

fn child_position(index: usize, sibling_count: usize) -> Option<ChildPosition> {
    (sibling_count > 0).then_some(ChildPosition::new(index, sibling_count))
}

fn element_rect(
    element: &Element,
    style: &ComputedStyle,
    parent_rect: Rect,
    viewport_rect: Rect,
    stylesheet: &StyleSheet,
    states: &HashMap<ElementId, ElementState>,
    anchors: &HashMap<ElementId, Rect>,
    text_measurer: &mut dyn TextMeasurer,
) -> Rect {
    if element.spec.role == ElementRole::Root {
        return parent_rect;
    }

    let measured = measure_element(
        element,
        style,
        parent_rect.size,
        stylesheet,
        states,
        text_measurer,
    );
    if style.position != Position::Flow {
        if let Some(anchor) = &style.anchor {
            if let Some(anchor_rect) = anchors.get(&anchor.target) {
                return anchored_rect(style, *anchor_rect, measured);
            }
        }
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
    scroll_limits: &mut HashMap<ElementId, Size>,
    text_measurer: &mut dyn TextMeasurer,
    anchors: &HashMap<ElementId, Rect>,
) -> Vec<ResolvedElement> {
    if element.spec.role == ElementRole::Table {
        if let Some(table) = &element.spec.table {
            return layout_table_children(
                element,
                table,
                content_rect,
                viewport_rect,
                stylesheet,
                states,
                scroll_limits,
                text_measurer,
                anchors,
            );
        }
    }

    if style.direction == Direction::Row && style.wrap {
        return layout_wrapped_children(
            element,
            style,
            content_rect,
            viewport_rect,
            stylesheet,
            states,
            scroll_limits,
            text_measurer,
            anchors,
        );
    }

    let flow_metrics: Vec<_> = element
        .children
        .iter()
        .enumerate()
        .map(|(index, child)| {
            let child_style = computed_style_for(
                child,
                stylesheet,
                states,
                child_position(index, element.children.len()),
            );
            if child_style.position != Position::Flow {
                return None;
            }
            Some(measure_flow_child(
                child,
                child_style,
                content_rect.size,
                stylesheet,
                states,
                text_measurer,
            ))
        })
        .collect();
    let flow_child_count = flow_metrics
        .iter()
        .filter(|metrics| metrics.is_some())
        .count();
    let total_main = total_main_size(&flow_metrics, style);
    let available_main = main_axis_size(content_rect.size, style.direction);
    let free_main = (available_main - total_main).max(0.0);
    let (mut cursor_main, gap) = aligned_main_axis(
        style.justify_content,
        style.gap,
        free_main,
        flow_child_count,
    );
    let mut frames = vec![None; element.children.len()];
    let mut child_anchors = anchors.clone();

    for (index, (child, metrics)) in element.children.iter().zip(flow_metrics.iter()).enumerate() {
        let child_style = computed_style_for(
            child,
            stylesheet,
            states,
            child_position(index, element.children.len()),
        );
        if child_style.position != Position::Flow {
            continue;
        }

        let metrics = metrics.expect("flow child should have measured metrics");
        let outer_main = main_axis_size(metrics.outer, style.direction);
        let outer_cross = cross_axis_size(metrics.outer, style.direction);
        let available_cross = cross_axis_size(content_rect.size, style.direction);
        let cursor_cross = aligned_cross_axis(style.align_items, available_cross, outer_cross);
        let child_rect = flow_child_parent_rect(
            content_rect,
            style.direction,
            cursor_main,
            cursor_cross,
            metrics.available,
        );
        let frame = layout_element_in_viewport(
            child,
            child_rect,
            viewport_rect,
            stylesheet,
            states,
            scroll_limits,
            text_measurer,
            &child_anchors,
            child_position(index, element.children.len()),
        );
        collect_frame_rects(&frame, &mut child_anchors);
        frames[index] = Some(frame);

        cursor_main += outer_main + gap;
    }

    for (index, child) in element.children.iter().enumerate() {
        if frames[index].is_some() {
            continue;
        }
        let frame = layout_element_in_viewport(
            child,
            content_rect,
            viewport_rect,
            stylesheet,
            states,
            scroll_limits,
            text_measurer,
            &child_anchors,
            child_position(index, element.children.len()),
        );
        collect_frame_rects(&frame, &mut child_anchors);
        frames[index] = Some(frame);
    }

    frames.into_iter().flatten().collect()
}

fn layout_wrapped_children(
    element: &Element,
    style: &ComputedStyle,
    content_rect: Rect,
    viewport_rect: Rect,
    stylesheet: &StyleSheet,
    states: &HashMap<ElementId, ElementState>,
    scroll_limits: &mut HashMap<ElementId, Size>,
    text_measurer: &mut dyn TextMeasurer,
    anchors: &HashMap<ElementId, Rect>,
) -> Vec<ResolvedElement> {
    let mut cursor = content_rect.origin;
    let mut line_height: f32 = 0.0;
    let mut frames = vec![None; element.children.len()];
    let mut child_anchors = anchors.clone();

    for (index, child) in element.children.iter().enumerate() {
        let child_style = computed_style_for(
            child,
            stylesheet,
            states,
            child_position(index, element.children.len()),
        );
        if child_style.position != Position::Flow {
            continue;
        }

        let metrics = measure_flow_child(
            child,
            child_style,
            content_rect.size,
            stylesheet,
            states,
            text_measurer,
        );

        if cursor.x > content_rect.origin.x && cursor.x + metrics.outer.width > content_rect.right()
        {
            cursor.x = content_rect.origin.x;
            cursor.y += line_height + style.gap;
            line_height = 0.0;
        }

        let child_rect = Rect::new(
            cursor.x,
            cursor.y,
            metrics.available.width,
            metrics.available.height,
        );
        let frame = layout_element_in_viewport(
            child,
            child_rect,
            viewport_rect,
            stylesheet,
            states,
            scroll_limits,
            text_measurer,
            &child_anchors,
            child_position(index, element.children.len()),
        );
        collect_frame_rects(&frame, &mut child_anchors);
        frames[index] = Some(frame);

        cursor.x += metrics.outer.width + style.gap;
        line_height = line_height.max(metrics.outer.height);
    }

    for (index, child) in element.children.iter().enumerate() {
        if frames[index].is_some() {
            continue;
        }
        let frame = layout_element_in_viewport(
            child,
            content_rect,
            viewport_rect,
            stylesheet,
            states,
            scroll_limits,
            text_measurer,
            &child_anchors,
            child_position(index, element.children.len()),
        );
        collect_frame_rects(&frame, &mut child_anchors);
        frames[index] = Some(frame);
    }

    frames.into_iter().flatten().collect()
}

fn collect_frame_rects(frame: &ResolvedElement, anchors: &mut HashMap<ElementId, Rect>) {
    anchors.insert(frame.id.clone(), frame.rect);
    for child in &frame.children {
        collect_frame_rects(child, anchors);
    }
}

#[derive(Clone, Copy, Debug)]
struct FlowChildMetrics {
    available: Size,
    outer: Size,
}

fn measure_flow_child(
    child: &Element,
    child_style: ComputedStyle,
    parent_size: Size,
    stylesheet: &StyleSheet,
    states: &HashMap<ElementId, ElementState>,
    text_measurer: &mut dyn TextMeasurer,
) -> FlowChildMetrics {
    let available = child_available_size(parent_size, &child_style);
    let measured = measure_intrinsic_element(
        child,
        &child_style,
        available,
        stylesheet,
        states,
        text_measurer,
    );
    FlowChildMetrics {
        available,
        outer: Size::new(
            measured.width + child_style.margin.horizontal(),
            measured.height + child_style.margin.vertical(),
        ),
    }
}

fn child_available_size(parent_size: Size, child_style: &ComputedStyle) -> Size {
    Size::new(
        (parent_size.width - child_style.margin.horizontal()).max(0.0),
        (parent_size.height - child_style.margin.vertical()).max(0.0),
    )
}

fn total_main_size(children: &[Option<FlowChildMetrics>], style: &ComputedStyle) -> f32 {
    let mut total = 0.0;
    let mut count = 0;
    for metrics in children.iter().flatten() {
        total += main_axis_size(metrics.outer, style.direction);
        count += 1;
    }
    if count > 1 {
        total += style.gap * (count - 1) as f32;
    }
    total
}

fn aligned_main_axis(
    justify_content: JustifyContent,
    base_gap: f32,
    free_space: f32,
    child_count: usize,
) -> (f32, f32) {
    match justify_content {
        JustifyContent::Start => (0.0, base_gap),
        JustifyContent::Center => (free_space / 2.0, base_gap),
        JustifyContent::End => (free_space, base_gap),
        JustifyContent::SpaceBetween if child_count > 1 => {
            (0.0, base_gap + free_space / (child_count - 1) as f32)
        }
        JustifyContent::SpaceBetween => (0.0, base_gap),
    }
}

fn aligned_cross_axis(align_items: AlignItems, available: f32, outer: f32) -> f32 {
    let free_space = (available - outer).max(0.0);
    match align_items {
        AlignItems::Start | AlignItems::Stretch => 0.0,
        AlignItems::Center => free_space / 2.0,
        AlignItems::End => free_space,
    }
}

fn flow_child_parent_rect(
    content_rect: Rect,
    direction: Direction,
    cursor_main: f32,
    cursor_cross: f32,
    available: Size,
) -> Rect {
    match direction {
        Direction::Column => Rect::new(
            content_rect.origin.x + cursor_cross,
            content_rect.origin.y + cursor_main,
            available.width,
            available.height,
        ),
        Direction::Row => Rect::new(
            content_rect.origin.x + cursor_main,
            content_rect.origin.y + cursor_cross,
            available.width,
            available.height,
        ),
    }
}

fn main_axis_size(size: Size, direction: Direction) -> f32 {
    match direction {
        Direction::Column => size.height,
        Direction::Row => size.width,
    }
}

fn cross_axis_size(size: Size, direction: Direction) -> f32 {
    match direction {
        Direction::Column => size.width,
        Direction::Row => size.height,
    }
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

fn anchored_rect(style: &ComputedStyle, anchor_rect: Rect, measured: Size) -> Rect {
    let anchor = style
        .anchor
        .as_ref()
        .expect("anchored rect requires an anchor style");
    let (x, y) = match anchor.placement {
        AnchorPlacement::TopStart => (anchor_rect.origin.x, anchor_rect.origin.y - measured.height),
        AnchorPlacement::TopEnd => (
            anchor_rect.right() - measured.width,
            anchor_rect.origin.y - measured.height,
        ),
        AnchorPlacement::BottomStart => (anchor_rect.origin.x, anchor_rect.bottom()),
        AnchorPlacement::BottomEnd => (anchor_rect.right() - measured.width, anchor_rect.bottom()),
        AnchorPlacement::LeftStart => (anchor_rect.origin.x - measured.width, anchor_rect.origin.y),
        AnchorPlacement::LeftEnd => (
            anchor_rect.origin.x - measured.width,
            anchor_rect.bottom() - measured.height,
        ),
        AnchorPlacement::RightStart => (anchor_rect.right(), anchor_rect.origin.y),
        AnchorPlacement::RightEnd => (anchor_rect.right(), anchor_rect.bottom() - measured.height),
    };

    Rect::new(
        x + anchor.offset.x + style.margin.left,
        y + anchor.offset.y + style.margin.top,
        measured.width,
        measured.height,
    )
}

fn measure_element(
    element: &Element,
    style: &ComputedStyle,
    parent_size: Size,
    stylesheet: &StyleSheet,
    states: &HashMap<ElementId, ElementState>,
    text_measurer: &mut dyn TextMeasurer,
) -> Size {
    let auto_size = match element.spec.role {
        ElementRole::Text => {
            let content_parent_size = measurement_content_size(style, parent_size);
            measure_text(
                element.text.as_deref().unwrap_or_default(),
                style,
                content_parent_size,
                text_measurer,
            )
            .size
        }
        ElementRole::Icon => Size::new(
            style.font_size.max(style.min_size.width),
            style.font_size.max(style.min_size.height),
        ),
        ElementRole::Table => {
            let content_parent_size = measurement_content_size(style, parent_size);
            let content = element
                .spec
                .table
                .as_ref()
                .map(|table| measure_table_content(element, table, content_parent_size))
                .unwrap_or_default();
            Size::new(
                content.width + style.padding.horizontal() + style.border_width.horizontal(),
                content.height + style.padding.vertical() + style.border_width.vertical(),
            )
        }
        _ => {
            let content_parent_size = measurement_content_size(style, parent_size);
            let content = measure_children(
                element,
                style,
                content_parent_size,
                stylesheet,
                states,
                text_measurer,
            );
            Size::new(
                content.width + style.padding.horizontal() + style.border_width.horizontal(),
                content.height + style.padding.vertical() + style.border_width.vertical(),
            )
        }
    };

    clamp_size(
        Size::new(
            style
                .width
                .resolve(parent_size.width, auto_size.width)
                .max(style.min_size.width),
            style
                .height
                .resolve(parent_size.height, auto_size.height)
                .max(style.min_size.height),
        ),
        style,
    )
}

fn measure_intrinsic_element(
    element: &Element,
    style: &ComputedStyle,
    parent_size: Size,
    stylesheet: &StyleSheet,
    states: &HashMap<ElementId, ElementState>,
    text_measurer: &mut dyn TextMeasurer,
) -> Size {
    let auto_size = match element.spec.role {
        ElementRole::Text => {
            let content_parent_size = measurement_content_size(style, parent_size);
            measure_text(
                element.text.as_deref().unwrap_or_default(),
                style,
                content_parent_size,
                text_measurer,
            )
            .size
        }
        ElementRole::Icon => Size::new(
            style.font_size.max(style.min_size.width),
            style.font_size.max(style.min_size.height),
        ),
        ElementRole::Table => {
            let content_parent_size = measurement_content_size(style, parent_size);
            let content = element
                .spec
                .table
                .as_ref()
                .map(|table| measure_table_content(element, table, content_parent_size))
                .unwrap_or_default();
            Size::new(
                content.width + style.padding.horizontal() + style.border_width.horizontal(),
                content.height + style.padding.vertical() + style.border_width.vertical(),
            )
        }
        _ => {
            let content_parent_size = measurement_content_size(style, parent_size);
            let content = measure_children(
                element,
                style,
                content_parent_size,
                stylesheet,
                states,
                text_measurer,
            );
            Size::new(
                content.width + style.padding.horizontal() + style.border_width.horizontal(),
                content.height + style.padding.vertical() + style.border_width.vertical(),
            )
        }
    };

    clamp_size(
        Size::new(
            style
                .width
                .resolve_intrinsic(parent_size.width, auto_size.width)
                .max(style.min_size.width),
            style
                .height
                .resolve_intrinsic(parent_size.height, auto_size.height)
                .max(style.min_size.height),
        ),
        style,
    )
}

fn clamp_size(size: Size, style: &ComputedStyle) -> Size {
    Size::new(
        size.width
            .max(style.min_size.width)
            .min(style.max_size.width.max(style.min_size.width)),
        size.height
            .max(style.min_size.height)
            .min(style.max_size.height.max(style.min_size.height)),
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
    text_measurer: &mut dyn TextMeasurer,
) -> Size {
    if element.spec.role == ElementRole::Table {
        if let Some(table) = &element.spec.table {
            return measure_table_content(element, table, parent_size);
        }
    }

    if style.direction == Direction::Row && style.wrap {
        return measure_wrapped_children(
            element,
            style,
            parent_size,
            stylesheet,
            states,
            text_measurer,
        );
    }

    let mut width: f32 = 0.0;
    let mut height: f32 = 0.0;
    let mut flow_child_count = 0;

    for (index, child) in element.children.iter().enumerate() {
        let child_style = computed_style_for(
            child,
            stylesheet,
            states,
            child_position(index, element.children.len()),
        );
        if child_style.position != Position::Flow {
            continue;
        }
        flow_child_count += 1;

        let child_available = Size::new(
            (parent_size.width - child_style.margin.horizontal()).max(0.0),
            (parent_size.height - child_style.margin.vertical()).max(0.0),
        );
        let child_size = measure_intrinsic_element(
            child,
            &child_style,
            child_available,
            stylesheet,
            states,
            text_measurer,
        );
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

fn layout_table_children(
    element: &Element,
    table: &TableSpec,
    content_rect: Rect,
    viewport_rect: Rect,
    stylesheet: &StyleSheet,
    states: &HashMap<ElementId, ElementState>,
    scroll_limits: &mut HashMap<ElementId, Size>,
    text_measurer: &mut dyn TextMeasurer,
    anchors: &HashMap<ElementId, Rect>,
) -> Vec<ResolvedElement> {
    let widths = resolve_table_column_widths(table, content_rect.size.width);
    let table_width = widths.iter().sum::<f32>();
    let mut y = content_rect.origin.y;
    let mut frames = Vec::with_capacity(element.children.len());

    for (row_index, row) in element.children.iter().enumerate() {
        let row_style = computed_style_for(
            row,
            stylesheet,
            states,
            child_position(row_index, element.children.len()),
        );
        let row_height = if row.spec.role == ElementRole::TableHeader {
            table.header_height
        } else {
            table.row_height
        };
        let row_rect = Rect::new(content_rect.origin.x, y, table_width, row_height);
        let inner_rect = row_rect.inset(row_style.border_width);
        let row_content_rect = inner_rect.inset(row_style.padding);
        let children = layout_table_cells(
            row,
            table,
            &widths,
            row_content_rect,
            viewport_rect,
            stylesheet,
            states,
            scroll_limits,
            text_measurer,
            anchors,
        );
        let text_layout = row
            .text
            .as_deref()
            .map(|text| measure_text(text, &row_style, row_content_rect.size, text_measurer));
        frames.push(ResolvedElement {
            id: row.id.clone(),
            role: row.spec.role,
            classes: row.spec.classes.clone(),
            rect: row_rect,
            style: row_style,
            text: row.text.clone(),
            text_layout,
            value: row.spec.value.clone(),
            glyph: row.spec.glyph,
            interactive: row.spec.interactive && !row.spec.disabled,
            children,
        });
        y += row_height;
    }

    frames
}

fn layout_table_cells(
    row: &Element,
    table: &TableSpec,
    column_widths: &[f32],
    content_rect: Rect,
    viewport_rect: Rect,
    stylesheet: &StyleSheet,
    states: &HashMap<ElementId, ElementState>,
    scroll_limits: &mut HashMap<ElementId, Size>,
    text_measurer: &mut dyn TextMeasurer,
    anchors: &HashMap<ElementId, Rect>,
) -> Vec<ResolvedElement> {
    let mut x = content_rect.origin.x;
    let mut frames = Vec::with_capacity(row.children.len());

    for (column_index, column) in table.columns.iter().enumerate() {
        let Some((cell_index, cell)) = row.children.iter().enumerate().find(|(_, cell)| {
            cell.spec
                .table_cell
                .as_ref()
                .is_some_and(|cell| cell.column_id == column.id)
        }) else {
            x += column_widths[column_index];
            continue;
        };
        let cell_style = computed_style_for(
            cell,
            stylesheet,
            states,
            child_position(cell_index, row.children.len()),
        );
        let cell_rect = Rect::new(
            x,
            content_rect.origin.y,
            column_widths[column_index],
            content_rect.size.height,
        );
        let inner_rect = cell_rect.inset(cell_style.border_width);
        let cell_content_rect = inner_rect.inset(cell_style.padding);
        let children = layout_children(
            cell,
            &cell_style,
            cell_content_rect,
            viewport_rect,
            stylesheet,
            states,
            scroll_limits,
            text_measurer,
            anchors,
        );
        let text_layout = cell
            .text
            .as_deref()
            .map(|text| measure_text(text, &cell_style, cell_content_rect.size, text_measurer));
        frames.push(ResolvedElement {
            id: cell.id.clone(),
            role: cell.spec.role,
            classes: cell.spec.classes.clone(),
            rect: cell_rect,
            style: cell_style,
            text: cell.text.clone(),
            text_layout,
            value: cell.spec.value.clone(),
            glyph: cell.spec.glyph,
            interactive: cell.spec.interactive && !cell.spec.disabled,
            children,
        });
        x += column_widths[column_index];
    }

    frames
}

fn measure_table_content(element: &Element, table: &TableSpec, parent_size: Size) -> Size {
    let columns = resolve_table_column_widths(table, parent_size.width);
    let header_count = element
        .children
        .iter()
        .filter(|child| child.spec.role == ElementRole::TableHeader)
        .count();
    let body_count = element.children.len().saturating_sub(header_count);
    Size::new(
        columns.iter().sum(),
        table.header_height * header_count as f32 + table.row_height * body_count as f32,
    )
}

fn resolve_table_column_widths(table: &TableSpec, available_width: f32) -> Vec<f32> {
    let mut widths = Vec::with_capacity(table.columns.len());
    let mut fixed = 0.0;
    let mut flex_weight = 0.0;

    for column in &table.columns {
        match column.width {
            TableTrackSize::Px(width) => {
                let width = clamp_table_column_width(width, column.min_width, column.max_width);
                widths.push(width);
                fixed += width;
            }
            TableTrackSize::Flex(weight) => {
                widths.push(0.0);
                flex_weight += weight;
            }
        }
    }

    let remaining = (available_width - fixed).max(0.0);
    for (index, column) in table.columns.iter().enumerate() {
        if let TableTrackSize::Flex(weight) = column.width {
            let width = if flex_weight <= f32::EPSILON {
                column.min_width
            } else {
                remaining * (weight / flex_weight)
            };
            widths[index] = clamp_table_column_width(width, column.min_width, column.max_width);
        }
    }

    widths
}

fn clamp_table_column_width(width: f32, min_width: f32, max_width: Option<f32>) -> f32 {
    let width = width.max(min_width);
    max_width.map_or(width, |max_width| width.min(max_width.max(min_width)))
}

fn measure_wrapped_children(
    element: &Element,
    style: &ComputedStyle,
    parent_size: Size,
    stylesheet: &StyleSheet,
    states: &HashMap<ElementId, ElementState>,
    text_measurer: &mut dyn TextMeasurer,
) -> Size {
    let mut width: f32 = 0.0;
    let mut height: f32 = 0.0;
    let mut line_width: f32 = 0.0;
    let mut line_height: f32 = 0.0;
    let mut line_has_child = false;

    for (index, child) in element.children.iter().enumerate() {
        let child_style = computed_style_for(
            child,
            stylesheet,
            states,
            child_position(index, element.children.len()),
        );
        if child_style.position != Position::Flow {
            continue;
        }

        let child_available = Size::new(
            (parent_size.width - child_style.margin.horizontal()).max(0.0),
            (parent_size.height - child_style.margin.vertical()).max(0.0),
        );
        let child_size = measure_intrinsic_element(
            child,
            &child_style,
            child_available,
            stylesheet,
            states,
            text_measurer,
        );
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

fn measure_text(
    text: &str,
    style: &ComputedStyle,
    available_size: Size,
    text_measurer: &mut dyn TextMeasurer,
) -> TextLayoutResult {
    let wrap_width = match style.text_wrap {
        crate::TextWrapMode::Extend => f32::INFINITY,
        crate::TextWrapMode::Wrap | crate::TextWrapMode::Truncate => available_size.width,
    };
    text_measurer.measure_text(TextLayoutRequest {
        text,
        font_size: style.font_size,
        wrap_width,
        wrap_mode: style.text_wrap,
        max_lines: style.max_lines,
        line_height: style.line_height,
    })
}

pub(crate) fn hit_path(frame: &ResolvedElement, point: Point) -> Option<Vec<&ResolvedElement>> {
    let mut children: Vec<_> = frame.children.iter().collect();
    children.sort_by_key(|child| child.style.z_index);

    let clips_overflow =
        frame.style.overflow_x == Overflow::Scroll || frame.style.overflow_y == Overflow::Scroll;
    let may_hit_children = !clips_overflow || frame.rect.contains(point);
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
