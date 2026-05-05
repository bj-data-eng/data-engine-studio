use crate::element::{Element, ElementId, ElementRole, ElementSpec};
use crate::geometry::{
    AlignItems, Direction, Insets, JustifyContent, Length, Overflow, Point, Position,
    Rect as DocumentRect, Size,
};
use crate::state::{ElementState, ResolvedElement};
use crate::style::{ChildPosition, ComputedStyle, StyleSheet, resolve_style_with_position};
use crate::table::{TableColumnId, TableSpec, TableTrackSize};
use crate::text::{FallbackTextMeasurer, TextLayoutRequest, TextMeasurer, TextWrapMode};
use layout_engine::prelude::{
    AlignItems as LayoutAlignItems, AvailableSpace, Dimension, Display, FlexDirection, FlexWrap,
    GridPlacement, GridTemplateComponent, JustifyContent as LayoutJustifyContent, LayoutTree,
    LengthPercentage, LengthPercentageAuto, NodeId, Position as LayoutPosition, Rect as LayoutRect,
    Size as LayoutSize, Style as LayoutStyle, fr, length, percent,
};
use layout_engine::style::Overflow as LayoutOverflow;
use std::collections::HashMap;

pub type SceneResult<T> = Result<T, SceneError>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SceneError {
    DuplicateElement(ElementId),
    MissingElement(ElementId),
    Layout(String),
}

impl std::fmt::Display for SceneError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SceneError::DuplicateElement(id) => write!(f, "Element {} already exists", id.as_str()),
            SceneError::MissingElement(id) => write!(f, "Element {} does not exist", id.as_str()),
            SceneError::Layout(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for SceneError {}

#[derive(Clone, Debug, PartialEq)]
pub struct SceneElement {
    pub id: ElementId,
    pub spec: ElementSpec,
    pub text: Option<String>,
    pub computed_style: ComputedStyle,
    scroll_offset: Point,
    layout_node: NodeId,
}

#[derive(Clone, Debug, PartialEq)]
struct SceneLayoutNode {
    id: ElementId,
    role: ElementRole,
    text: Option<String>,
}

pub struct DocumentScene {
    viewport: Size,
    layout: LayoutTree<SceneLayoutNode>,
    elements: HashMap<ElementId, SceneElement>,
    layout_to_element: HashMap<NodeId, ElementId>,
    root: ElementId,
}

impl DocumentScene {
    pub fn new(viewport: Size) -> Self {
        let mut layout = LayoutTree::new();
        let root = ElementId::new("root");
        let root_node = layout
            .new_leaf_with_context(
                root_layout_style(viewport),
                SceneLayoutNode {
                    id: root.clone(),
                    role: ElementRole::Root,
                    text: None,
                },
            )
            .expect("root layout node can be created");

        let mut elements = HashMap::new();
        elements.insert(
            root.clone(),
            SceneElement {
                id: root.clone(),
                spec: ElementSpec::new(ElementRole::Root),
                text: None,
                computed_style: root_sized_style(ComputedStyle::default(), viewport),
                scroll_offset: Point::ZERO,
                layout_node: root_node,
            },
        );

        let mut layout_to_element = HashMap::new();
        layout_to_element.insert(root_node, root.clone());

        Self {
            viewport,
            layout,
            elements,
            layout_to_element,
            root,
        }
    }

    pub fn viewport(&self) -> Size {
        self.viewport
    }

    pub fn root(&self) -> &ElementId {
        &self.root
    }

    pub fn element_ids(&self) -> Vec<ElementId> {
        let mut ids = self.elements.keys().cloned().collect::<Vec<_>>();
        ids.sort();
        ids
    }

    pub fn append_element(
        &mut self,
        parent: impl Into<ElementId>,
        id: impl Into<ElementId>,
        spec: ElementSpec,
    ) -> SceneResult<NodeId> {
        self.append_node(parent.into(), id.into(), spec, None)
    }

    pub fn append_text(
        &mut self,
        parent: impl Into<ElementId>,
        id: impl Into<ElementId>,
        spec: ElementSpec,
        text: impl Into<String>,
    ) -> SceneResult<NodeId> {
        self.append_node(parent.into(), id.into(), spec, Some(text.into()))
    }

    pub fn reparent(
        &mut self,
        id: impl Into<ElementId>,
        new_parent: impl Into<ElementId>,
    ) -> SceneResult<()> {
        let id = id.into();
        let new_parent = new_parent.into();
        let node = self.element(&id)?.layout_node;
        let parent_node = self.element(&new_parent)?.layout_node;
        self.layout
            .add_child(parent_node, node)
            .map_err(layout_error)?;
        Ok(())
    }

    pub fn remove(&mut self, id: impl Into<ElementId>) -> SceneResult<()> {
        let id = id.into();
        if id == self.root {
            return Err(SceneError::MissingElement(id));
        }
        self.remove_subtree(&id)
    }

    pub fn layout_node(&self, id: impl Into<ElementId>) -> Option<NodeId> {
        self.elements
            .get(&id.into())
            .map(|element| element.layout_node)
    }

    pub fn layout_style(&self, id: impl Into<ElementId>) -> SceneResult<&LayoutStyle> {
        let node = self.element(&id.into())?.layout_node;
        self.layout.style(node).map_err(layout_error)
    }

    pub fn apply_computed_style(
        &mut self,
        id: impl Into<ElementId>,
        style: &ComputedStyle,
    ) -> SceneResult<()> {
        let id = id.into();
        let node = self.element(&id)?.layout_node;
        self.layout
            .set_style(node, layout_style_from_computed(style))
            .map_err(layout_error)?;
        self.element_mut(&id)?.computed_style = style.clone();
        Ok(())
    }

    pub fn apply_stylesheet(
        &mut self,
        stylesheet: &StyleSheet,
        states: &HashMap<ElementId, ElementState>,
    ) -> SceneResult<()> {
        let mut positions = Vec::new();
        self.collect_positions(self.root.clone(), None, &mut positions)?;

        for (id, position) in positions {
            let element = self.snapshot_element(&id)?;
            let computed =
                resolve_style_with_position(&element, stylesheet, states.get(&id), position);
            let scroll_offset = states
                .get(&id)
                .map(|state| Point::new(state.scroll_x, state.scroll_y))
                .unwrap_or(Point::ZERO);
            self.element_mut(&id)?.scroll_offset = scroll_offset;
            if id == self.root {
                self.apply_computed_style(id.clone(), &root_sized_style(computed, self.viewport))?;
            } else {
                self.apply_computed_style(id.clone(), &computed)?;
            }
        }
        self.apply_table_grid_styles()?;

        Ok(())
    }

    pub fn compute_layout(&mut self) -> SceneResult<()> {
        let mut text_measurer = FallbackTextMeasurer;
        self.compute_layout_with_text_measurer(&mut text_measurer)
    }

    pub fn compute_layout_with_text_measurer(
        &mut self,
        text_measurer: &mut dyn TextMeasurer,
    ) -> SceneResult<()> {
        let root_node = self.element(&self.root)?.layout_node;
        let measure_inputs = self.measure_inputs();
        self.layout
            .compute_layout_with_measure(
                root_node,
                LayoutSize {
                    width: length::<_, AvailableSpace>(self.viewport.width),
                    height: length::<_, AvailableSpace>(self.viewport.height),
                },
                |known_dimensions, available_space, node_id, _, _| {
                    let Some(input) = measure_inputs.get(&node_id) else {
                        return LayoutSize::ZERO;
                    };
                    let measured = measure_text_content(
                        input.text.as_str(),
                        &input.style,
                        known_dimensions
                            .width
                            .or_else(|| available_space.width.into_option()),
                        text_measurer,
                    );
                    LayoutSize {
                        width: known_dimensions.width.unwrap_or(measured.width),
                        height: known_dimensions.height.unwrap_or(measured.height),
                    }
                },
            )
            .map_err(layout_error)
    }

    pub fn layout_rect(&self, id: impl Into<ElementId>) -> SceneResult<DocumentRect> {
        let node = self.element(&id.into())?.layout_node;
        let layout = self.layout.layout(node).map_err(layout_error)?;
        Ok(DocumentRect::new(
            layout.location.x,
            layout.location.y,
            layout.size.width,
            layout.size.height,
        ))
    }

    pub fn resolved_layout(&self) -> SceneResult<ResolvedElement> {
        let mut text_measurer = FallbackTextMeasurer;
        self.resolved_layout_with_text_measurer(&mut text_measurer)
    }

    pub fn resolved_layout_with_text_measurer(
        &self,
        text_measurer: &mut dyn TextMeasurer,
    ) -> SceneResult<ResolvedElement> {
        self.resolved_element(&self.root, Point::ZERO, Point::ZERO, text_measurer)
    }

    pub(crate) fn scroll_limits(&self) -> SceneResult<HashMap<ElementId, Size>> {
        let mut limits = HashMap::new();
        for id in self.element_ids() {
            let element = self.element(&id)?;
            let style = &element.computed_style;
            if style.overflow_x != Overflow::Scroll && style.overflow_y != Overflow::Scroll {
                continue;
            }

            let layout = self
                .layout
                .layout(element.layout_node)
                .map_err(layout_error)?;
            let max_scroll = Size::new(
                (layout.content_size.width - layout.size.width).max(0.0),
                (layout.content_size.height - layout.size.height).max(0.0),
            );
            if max_scroll.width > 0.0 || max_scroll.height > 0.0 {
                limits.insert(id, max_scroll);
            }
        }
        Ok(limits)
    }

    pub fn resolve_layout(
        &mut self,
        stylesheet: &StyleSheet,
        states: &HashMap<ElementId, ElementState>,
    ) -> SceneResult<ResolvedElement> {
        let mut text_measurer = FallbackTextMeasurer;
        self.resolve_layout_with_text_measurer(stylesheet, states, &mut text_measurer)
    }

    pub fn resolve_layout_with_text_measurer(
        &mut self,
        stylesheet: &StyleSheet,
        states: &HashMap<ElementId, ElementState>,
        text_measurer: &mut dyn TextMeasurer,
    ) -> SceneResult<ResolvedElement> {
        self.apply_stylesheet(stylesheet, states)?;
        self.compute_layout_with_text_measurer(text_measurer)?;
        self.resolved_layout_with_text_measurer(text_measurer)
    }

    pub fn parent(&self, id: impl Into<ElementId>) -> SceneResult<Option<ElementId>> {
        let node = self.element(&id.into())?.layout_node;
        Ok(self
            .layout
            .parent(node)
            .and_then(|parent| self.layout_to_element.get(&parent).cloned()))
    }

    pub fn children(&self, id: impl Into<ElementId>) -> SceneResult<Vec<ElementId>> {
        let node = self.element(&id.into())?.layout_node;
        self.layout
            .children(node)
            .map_err(layout_error)?
            .into_iter()
            .map(|child| {
                self.layout_to_element.get(&child).cloned().ok_or_else(|| {
                    SceneError::Layout(format!("Layout node {child:?} is not indexed"))
                })
            })
            .collect()
    }

    fn append_node(
        &mut self,
        parent: ElementId,
        id: ElementId,
        spec: ElementSpec,
        text: Option<String>,
    ) -> SceneResult<NodeId> {
        if self.elements.contains_key(&id) {
            return Err(SceneError::DuplicateElement(id));
        }

        let parent_node = self.element(&parent)?.layout_node;
        let node = self
            .layout
            .new_leaf_with_context(
                LayoutStyle::default(),
                SceneLayoutNode {
                    id: id.clone(),
                    role: spec.role,
                    text: text.clone(),
                },
            )
            .map_err(layout_error)?;
        self.layout
            .add_child(parent_node, node)
            .map_err(layout_error)?;
        self.layout_to_element.insert(node, id.clone());
        self.elements.insert(
            id.clone(),
            SceneElement {
                id,
                spec,
                text,
                computed_style: ComputedStyle::default(),
                scroll_offset: Point::ZERO,
                layout_node: node,
            },
        );

        Ok(node)
    }

    fn remove_subtree(&mut self, id: &ElementId) -> SceneResult<()> {
        let children = self.children(id.clone())?;
        for child in children {
            self.remove_subtree(&child)?;
        }

        let element = self
            .elements
            .remove(id)
            .ok_or_else(|| SceneError::MissingElement(id.clone()))?;
        self.layout_to_element.remove(&element.layout_node);
        self.layout
            .remove(element.layout_node)
            .map_err(layout_error)?;
        Ok(())
    }

    fn element(&self, id: &ElementId) -> SceneResult<&SceneElement> {
        self.elements
            .get(id)
            .ok_or_else(|| SceneError::MissingElement(id.clone()))
    }

    fn element_mut(&mut self, id: &ElementId) -> SceneResult<&mut SceneElement> {
        self.elements
            .get_mut(id)
            .ok_or_else(|| SceneError::MissingElement(id.clone()))
    }

    fn resolved_element(
        &self,
        id: &ElementId,
        parent_origin: Point,
        parent_scroll_offset: Point,
        text_measurer: &mut dyn TextMeasurer,
    ) -> SceneResult<ResolvedElement> {
        let element = self.element(id)?;
        let raw_rect = self.layout_rect(id.as_str())?;
        let rect = resolved_document_rect(
            raw_rect,
            &element.computed_style,
            self.viewport,
            parent_origin,
            parent_scroll_offset,
        );
        let text_layout = element
            .text
            .as_deref()
            .map(|text| measure_text(text, &element.computed_style, rect, text_measurer));
        let children = self
            .children(id.clone())?
            .into_iter()
            .map(|child| {
                self.resolved_element(&child, rect.origin, element.scroll_offset, text_measurer)
            })
            .collect::<SceneResult<Vec<_>>>()?;

        Ok(ResolvedElement {
            id: element.id.clone(),
            role: element.spec.role,
            classes: element.spec.classes.clone(),
            rect,
            style: element.computed_style.clone(),
            text: element.text.clone(),
            text_layout,
            selectable_text: element.spec.selectable_text && element.text.is_some(),
            copyable_text: element.spec.selectable_text
                && element.spec.copyable_text
                && element.text.is_some(),
            value: element.spec.value.clone(),
            glyph: element.spec.glyph,
            interactive: element.spec.interactive && !element.spec.disabled,
            children,
        })
    }

    fn collect_positions(
        &self,
        id: ElementId,
        position: Option<ChildPosition>,
        positions: &mut Vec<(ElementId, Option<ChildPosition>)>,
    ) -> SceneResult<()> {
        positions.push((id.clone(), position));

        let children = self.children(id)?;
        let sibling_count = children.len();
        for (index, child) in children.into_iter().enumerate() {
            self.collect_positions(
                child,
                Some(ChildPosition::new(index, sibling_count)),
                positions,
            )?;
        }

        Ok(())
    }

    fn snapshot_element(&self, id: &ElementId) -> SceneResult<Element> {
        let element = self.element(id)?;
        Ok(Element {
            id: element.id.clone(),
            spec: element.spec.clone(),
            text: element.text.clone(),
            children: Vec::new(),
        })
    }

    fn measure_inputs(&self) -> HashMap<NodeId, SceneMeasureInput> {
        self.elements
            .values()
            .filter_map(|element| {
                element.text.as_ref().map(|text| {
                    (
                        element.layout_node,
                        SceneMeasureInput {
                            text: text.clone(),
                            style: element.computed_style.clone(),
                        },
                    )
                })
            })
            .collect()
    }

    fn apply_table_grid_styles(&mut self) -> SceneResult<()> {
        let ids = self.element_ids();
        for id in ids {
            let element = self.element(&id)?;
            if element.spec.role != ElementRole::TableHeader
                && element.spec.role != ElementRole::TableRow
            {
                continue;
            }

            let Some(parent_id) = self.parent(id.clone())? else {
                continue;
            };
            let Some(table) = self.element(&parent_id)?.spec.table.clone() else {
                continue;
            };

            let node = element.layout_node;
            let mut style = self.layout.style(node).map_err(layout_error)?.clone();
            style.display = Display::Grid;
            style.grid_template_columns = table_grid_columns(&table);
            style.size.width = length(table_grid_width(&table));
            style.size.height = length(if element.spec.role == ElementRole::TableHeader {
                table.header_height
            } else {
                table.row_height
            });
            self.layout.set_style(node, style).map_err(layout_error)?;

            let row_children = self.children(id.clone())?;
            for child_id in row_children {
                let child = self.element(&child_id)?;
                let Some(cell) = &child.spec.table_cell else {
                    continue;
                };
                let Some(column_index) = table_column_index(&table, &cell.column_id) else {
                    continue;
                };
                let child_node = child.layout_node;
                let mut child_style = self.layout.style(child_node).map_err(layout_error)?.clone();
                child_style.grid_column = layout_engine::geometry::Line {
                    start: GridPlacement::Line((column_index + 1).into()),
                    end: GridPlacement::Line((column_index + 2).into()),
                };
                self.layout
                    .set_style(child_node, child_style)
                    .map_err(layout_error)?;
            }
        }

        Ok(())
    }
}

struct SceneMeasureInput {
    text: String,
    style: ComputedStyle,
}

fn root_layout_style(viewport: Size) -> LayoutStyle {
    LayoutStyle {
        display: Display::Flex,
        size: LayoutSize {
            width: length(viewport.width),
            height: length(viewport.height),
        },
        ..Default::default()
    }
}

fn root_sized_style(mut style: ComputedStyle, viewport: Size) -> ComputedStyle {
    style.width = Length::Px(viewport.width);
    style.height = Length::Px(viewport.height);
    style
}

fn table_grid_columns(table: &TableSpec) -> Vec<GridTemplateComponent<String>> {
    table
        .columns
        .iter()
        .map(|column| match column.width {
            TableTrackSize::Px(width) => length(clamp_table_column_width(
                width,
                column.min_width,
                column.max_width,
            )),
            TableTrackSize::Flex(weight) => fr(weight),
        })
        .collect()
}

fn table_grid_width(table: &TableSpec) -> f32 {
    table_column_widths(table, table_fixed_width(table))
        .into_iter()
        .sum()
}

fn table_fixed_width(table: &TableSpec) -> f32 {
    table
        .columns
        .iter()
        .map(|column| match column.width {
            TableTrackSize::Px(width) => {
                clamp_table_column_width(width, column.min_width, column.max_width)
            }
            TableTrackSize::Flex(_) => column.min_width,
        })
        .sum()
}

fn table_column_widths(table: &TableSpec, available_width: f32) -> Vec<f32> {
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

fn table_column_index(table: &TableSpec, column_id: &TableColumnId) -> Option<i16> {
    table
        .columns
        .iter()
        .position(|column| column.id == *column_id)
        .and_then(|index| i16::try_from(index).ok())
}

fn clamp_table_column_width(width: f32, min_width: f32, max_width: Option<f32>) -> f32 {
    let width = width.max(min_width);
    max_width.map_or(width, |max_width| width.min(max_width.max(min_width)))
}

fn resolved_document_rect(
    raw_rect: DocumentRect,
    style: &ComputedStyle,
    viewport: Size,
    parent_origin: Point,
    parent_scroll_offset: Point,
) -> DocumentRect {
    if style.position != Position::AbsoluteViewport {
        return DocumentRect::new(
            parent_origin.x + raw_rect.origin.x - parent_scroll_offset.x,
            parent_origin.y + raw_rect.origin.y - parent_scroll_offset.y,
            raw_rect.size.width,
            raw_rect.size.height,
        );
    }

    DocumentRect::new(
        viewport_axis_position(
            raw_rect.origin.x,
            raw_rect.size.width,
            viewport.width,
            style.inset.left,
            style.inset.right,
        ),
        viewport_axis_position(
            raw_rect.origin.y,
            raw_rect.size.height,
            viewport.height,
            style.inset.top,
            style.inset.bottom,
        ),
        raw_rect.size.width,
        raw_rect.size.height,
    )
}

fn viewport_axis_position(
    fallback: f32,
    size: f32,
    viewport_size: f32,
    start: Option<Length>,
    end: Option<Length>,
) -> f32 {
    if let Some(start) = start {
        return start.resolve(viewport_size, fallback);
    }
    if let Some(end) = end {
        return viewport_size - end.resolve(viewport_size, 0.0) - size;
    }
    fallback
}

fn measure_text(
    text: &str,
    style: &ComputedStyle,
    rect: DocumentRect,
    text_measurer: &mut dyn TextMeasurer,
) -> crate::text::TextLayoutResult {
    let content_rect = rect.inset(style.border_width).inset(style.padding);
    measure_text_with_wrap_width(text, style, content_rect.size.width, text_measurer)
}

fn measure_text_content(
    text: &str,
    style: &ComputedStyle,
    available_width: Option<f32>,
    text_measurer: &mut dyn TextMeasurer,
) -> Size {
    measure_text_with_wrap_width(
        text,
        style,
        available_width.unwrap_or(f32::INFINITY),
        text_measurer,
    )
    .size
}

fn measure_text_with_wrap_width(
    text: &str,
    style: &ComputedStyle,
    available_width: f32,
    text_measurer: &mut dyn TextMeasurer,
) -> crate::text::TextLayoutResult {
    let wrap_width = match style.text_wrap {
        TextWrapMode::Extend => f32::INFINITY,
        TextWrapMode::Wrap | TextWrapMode::Truncate => available_width,
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

fn layout_style_from_computed(style: &ComputedStyle) -> LayoutStyle {
    LayoutStyle {
        display: Display::Flex,
        overflow: layout_overflow(style.overflow_x, style.overflow_y),
        scrollbar_width: style.scrollbar_width,
        position: layout_position(style.position),
        inset: LayoutRect {
            left: style.inset.left.map_or_else(
                LengthPercentageAuto::auto,
                length_percentage_auto_from_document,
            ),
            right: style.inset.right.map_or_else(
                LengthPercentageAuto::auto,
                length_percentage_auto_from_document,
            ),
            top: style.inset.top.map_or_else(
                LengthPercentageAuto::auto,
                length_percentage_auto_from_document,
            ),
            bottom: style.inset.bottom.map_or_else(
                LengthPercentageAuto::auto,
                length_percentage_auto_from_document,
            ),
        },
        size: LayoutSize {
            width: dimension_from_document(style.width),
            height: dimension_from_document(style.height),
        },
        min_size: LayoutSize {
            width: layout_bound(style.min_size.width),
            height: layout_bound(style.min_size.height),
        },
        max_size: LayoutSize {
            width: layout_bound(style.max_size.width),
            height: layout_bound(style.max_size.height),
        },
        margin: layout_auto_rect(style.margin),
        padding: layout_rect(style.padding),
        border: layout_rect(style.border_width),
        align_items: Some(layout_align_items(style.align_items)),
        justify_content: Some(layout_justify_content(style.justify_content)),
        gap: LayoutSize::length(style.gap),
        flex_direction: layout_flex_direction(style.direction),
        flex_wrap: if style.wrap {
            FlexWrap::Wrap
        } else {
            FlexWrap::NoWrap
        },
        flex_shrink: 0.0,
        ..Default::default()
    }
}

fn dimension_from_document(length_value: Length) -> Dimension {
    match length_value {
        Length::Auto => Dimension::auto(),
        Length::Px(value) => length(value),
        Length::Fill => percent(1.0),
        Length::Percent(value) => percent(value),
    }
}

fn length_percentage_auto_from_document(length_value: Length) -> LengthPercentageAuto {
    match length_value {
        Length::Auto => LengthPercentageAuto::auto(),
        Length::Px(value) => length(value),
        Length::Fill => percent(1.0),
        Length::Percent(value) => percent(value),
    }
}

fn layout_bound(value: f32) -> Dimension {
    if value.is_finite() {
        length(value)
    } else {
        Dimension::auto()
    }
}

fn layout_auto_rect(insets: Insets) -> LayoutRect<LengthPercentageAuto> {
    LayoutRect {
        left: length(insets.left),
        right: length(insets.right),
        top: length(insets.top),
        bottom: length(insets.bottom),
    }
}

fn layout_rect(insets: Insets) -> LayoutRect<LengthPercentage> {
    LayoutRect {
        left: length(insets.left),
        right: length(insets.right),
        top: length(insets.top),
        bottom: length(insets.bottom),
    }
}

fn layout_overflow(x: Overflow, y: Overflow) -> layout_engine::geometry::Point<LayoutOverflow> {
    layout_engine::geometry::Point {
        x: match x {
            Overflow::Visible => LayoutOverflow::Visible,
            Overflow::Scroll => LayoutOverflow::Scroll,
        },
        y: match y {
            Overflow::Visible => LayoutOverflow::Visible,
            Overflow::Scroll => LayoutOverflow::Scroll,
        },
    }
}

fn layout_position(position: Position) -> LayoutPosition {
    match position {
        Position::Flow => LayoutPosition::Relative,
        Position::AbsoluteParent | Position::AbsoluteViewport => LayoutPosition::Absolute,
    }
}

fn layout_align_items(align_items: AlignItems) -> LayoutAlignItems {
    match align_items {
        AlignItems::Start => LayoutAlignItems::Start,
        AlignItems::Center => LayoutAlignItems::Center,
        AlignItems::End => LayoutAlignItems::End,
        AlignItems::Stretch => LayoutAlignItems::Stretch,
    }
}

fn layout_justify_content(justify_content: JustifyContent) -> LayoutJustifyContent {
    match justify_content {
        JustifyContent::Start => LayoutJustifyContent::Start,
        JustifyContent::Center => LayoutJustifyContent::Center,
        JustifyContent::End => LayoutJustifyContent::End,
        JustifyContent::SpaceBetween => LayoutJustifyContent::SpaceBetween,
    }
}

fn layout_flex_direction(direction: Direction) -> FlexDirection {
    match direction {
        Direction::Row => FlexDirection::Row,
        Direction::Column => FlexDirection::Column,
    }
}

fn layout_error(error: layout_engine::LayoutError) -> SceneError {
    SceneError::Layout(error.to_string())
}
