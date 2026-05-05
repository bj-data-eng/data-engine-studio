use crate::element::{ElementId, ElementRole, ElementSpec};
use crate::geometry::Size;
use layout_engine::prelude::{
    Display, LayoutTree, NodeId, Size as LayoutSize, Style as LayoutStyle, length,
};
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

fn layout_error(error: layout_engine::LayoutError) -> SceneError {
    SceneError::Layout(error.to_string())
}
