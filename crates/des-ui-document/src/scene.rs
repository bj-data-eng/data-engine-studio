use crate::element::{Element, ElementId, ElementRole, ElementSpec};
use crate::geometry::{
    AlignItems, Direction, Insets, JustifyContent, Length, Overflow, Position,
    Rect as DocumentRect, Size,
};
use crate::state::{ElementState, ResolvedElement};
use crate::style::{ChildPosition, ComputedStyle, StyleSheet, resolve_style_with_position};
use layout_engine::prelude::{
    AlignItems as LayoutAlignItems, AvailableSpace, Dimension, Display, FlexDirection, FlexWrap,
    JustifyContent as LayoutJustifyContent, LayoutTree, LengthPercentage, LengthPercentageAuto,
    NodeId, Position as LayoutPosition, Rect as LayoutRect, Size as LayoutSize,
    Style as LayoutStyle, length, percent,
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
            if id == self.root {
                self.apply_computed_style(id.clone(), &root_sized_style(computed, self.viewport))?;
            } else {
                self.apply_computed_style(id.clone(), &computed)?;
            }
        }

        Ok(())
    }

    pub fn compute_layout(&mut self) -> SceneResult<()> {
        let root_node = self.element(&self.root)?.layout_node;
        self.layout
            .compute_layout(
                root_node,
                LayoutSize {
                    width: length::<_, AvailableSpace>(self.viewport.width),
                    height: length::<_, AvailableSpace>(self.viewport.height),
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
        self.resolved_element(&self.root)
    }

    pub fn resolve_layout(
        &mut self,
        stylesheet: &StyleSheet,
        states: &HashMap<ElementId, ElementState>,
    ) -> SceneResult<ResolvedElement> {
        self.apply_stylesheet(stylesheet, states)?;
        self.compute_layout()?;
        self.resolved_layout()
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

    fn resolved_element(&self, id: &ElementId) -> SceneResult<ResolvedElement> {
        let element = self.element(id)?;
        let children = self
            .children(id.clone())?
            .into_iter()
            .map(|child| self.resolved_element(&child))
            .collect::<SceneResult<Vec<_>>>()?;

        Ok(ResolvedElement {
            id: element.id.clone(),
            role: element.spec.role,
            classes: element.spec.classes.clone(),
            rect: self.layout_rect(id.as_str())?,
            style: element.computed_style.clone(),
            text: element.text.clone(),
            text_layout: None,
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
