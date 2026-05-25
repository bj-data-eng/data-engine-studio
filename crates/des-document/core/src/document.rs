use crate::element::{
    ClassName, DocumentNode, Element, ElementBehaviorEvent, ElementBehaviorHook, ElementId,
    ElementSpec, Glyph, VisualCloneOptions, VisualElementClone,
};
use crate::geometry::{
    AlignContent, AlignItems, ClipRect, FlexDirection, FlexWrap, Insets, JustifyContent, Length,
    Overflow, Point, Position, Rect as DocumentRect, Size,
};
use crate::layout::{child_clip_rect, to_layout_insets, to_layout_size};
use crate::projection::{DocumentProjection, DocumentProjectionReport, ElementProjectionPatch};
use crate::state::{
    DocumentCommandAction, DocumentCommandBinding, DocumentCommandDispatchReport,
    DocumentCommandRegistry, DocumentInput, ElementState, ResolvedElement, ResolvedFloating,
};
use crate::style::{
    ChildPosition, ComputedStyle, StyleMatchContext, StyleResolutionContext, StyleSheet,
    classify_computed_style_change, resolve_style_with_position,
};
use crate::table::{TableColumnId, TableSpec, TableTrackSize};
#[cfg(test)]
use crate::text::FallbackTextMeasurer;
use crate::text::{NormalizedText, TextContent, TextLayoutRequest, TextMeasurer};
use des_layout::floating::{FloatingBoundary, FloatingRect, compute_floating_position};
use des_layout::geometry::{Point as LayoutPoint, Size as FloatingSize};
use des_layout::prelude::{
    AlignContent as LayoutAlignContent, AlignItems as LayoutAlignItems, AvailableSpace, Dimension,
    Display, FlexDirection as LayoutFlexDirection, FlexWrap as LayoutFlexWrap, GridPlacement,
    GridTemplateComponent, JustifyContent as LayoutJustifyContent, LayoutTree, LengthPercentage,
    LengthPercentageAuto, NodeId, Position as LayoutPosition, Rect as LayoutRect,
    Size as LayoutSize, Style as LayoutStyle, fr, length, percent,
};
use des_layout::scroll as layout_scroll;
use des_layout::style::Overflow as LayoutOverflow;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

pub type DocumentResult<T> = Result<T, DocumentError>;
pub type DocumentAuthoringResult<T> = Result<T, DocumentAuthoringError>;

static NEXT_DOCUMENT_INSTANCE_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct StyleResolutionReport {
    pub visited: usize,
    pub paint_changed: bool,
    pub layout_changed: bool,
}

impl StyleResolutionReport {
    pub fn changed(self) -> bool {
        self.paint_changed || self.layout_changed
    }
}

#[derive(Clone, Debug)]
struct OwnedStyleMatchContext {
    element: DocumentNode,
    state: Option<ElementState>,
    position: Option<ChildPosition>,
}

struct ApplyStylesheetContext<'a> {
    stylesheet: &'a StyleSheet,
    states: &'a HashMap<ElementId, ElementState>,
    resolve_container_queries: bool,
    report: &'a mut StyleResolutionReport,
}

struct ApplyStylesheetTraversal<'a> {
    position: Option<ChildPosition>,
    ancestors: &'a mut Vec<OwnedStyleMatchContext>,
    previous_siblings: &'a [OwnedStyleMatchContext],
}

#[derive(Clone, Copy, Debug)]
struct ResolveParentFrame {
    origin: Point,
    scroll_offset: Point,
    clip: ClipRect,
}

struct ResolveTreeContext<'a> {
    text_measurer: &'a mut dyn TextMeasurer,
    anchors: &'a mut HashMap<ElementId, DocumentRect>,
    boundaries: &'a mut HashMap<ElementId, DocumentRect>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DocumentError {
    DuplicateElement(ElementId),
    MissingElement(ElementId),
    Layout(String),
}

impl std::fmt::Display for DocumentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DocumentError::DuplicateElement(id) => {
                write!(f, "DocumentNode {} already exists", id.as_str())
            }
            DocumentError::MissingElement(id) => {
                write!(f, "DocumentNode {} does not exist", id.as_str())
            }
            DocumentError::Layout(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for DocumentError {}

/// Error returned by app-facing authoring helpers that combine CSS parsing
/// with document/widget contract validation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DocumentAuthoringError {
    Css(crate::CssParseError),
    Document(DocumentError),
}

impl std::fmt::Display for DocumentAuthoringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Css(error) => write!(f, "CSS authoring error: {error}"),
            Self::Document(error) => write!(f, "document authoring error: {error}"),
        }
    }
}

impl std::error::Error for DocumentAuthoringError {}

impl From<crate::CssParseError> for DocumentAuthoringError {
    fn from(error: crate::CssParseError) -> Self {
        Self::Css(error)
    }
}

impl From<DocumentError> for DocumentAuthoringError {
    fn from(error: DocumentError) -> Self {
        Self::Document(error)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct DocumentElement {
    pub id: ElementId,
    pub spec: ElementSpec,
    pub text: Option<TextContent>,
    pub computed_style: ComputedStyle,
    scroll_offset: Point,
    layout_node: NodeId,
}

#[derive(Clone, Debug, PartialEq)]
struct DocumentLayoutNode {
    id: ElementId,
    element: Element,
    text: Option<TextContent>,
}

pub struct Document {
    instance_id: u64,
    viewport: Size,
    revision: u64,
    layout: LayoutTree<DocumentLayoutNode>,
    calc_lengths: HashMap<CalcLengthKey, Box<LayoutCalcLength>>,
    elements: HashMap<ElementId, DocumentElement>,
    layout_to_element: HashMap<NodeId, ElementId>,
    root: ElementId,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct CalcLengthKey {
    percent: u32,
    px: u32,
}

impl CalcLengthKey {
    fn new(percent: f32, px: f32) -> Self {
        Self {
            percent: percent.to_bits(),
            px: px.to_bits(),
        }
    }
}

#[derive(Debug)]
struct LayoutCalcLength {
    percent: f32,
    px: f32,
}

fn resolve_layout_calc_length(value: *const (), basis: f32) -> f32 {
    if value.is_null() {
        return 0.0;
    }
    // SAFETY: calc handles are interned `LayoutCalcLength` boxes owned by the document for
    // at least as long as the layout tree can resolve them.
    let calc = unsafe { &*(value.cast::<LayoutCalcLength>()) };
    basis * calc.percent + calc.px
}

impl Document {
    pub fn build(viewport: Size, add_contents: impl FnOnce(&mut DocumentBuilder)) -> Self {
        let mut builder = DocumentBuilder::default();
        add_contents(&mut builder);
        let mut document = Self::new(viewport);
        for child in builder.children {
            document
                .append_element_tree("root", child)
                .expect("document builder produces a valid element tree");
        }
        document
    }

    pub fn new(viewport: Size) -> Self {
        let mut layout = LayoutTree::new();
        layout.set_calc_resolver(resolve_layout_calc_length);
        let root = ElementId::new("root");
        let root_node = layout
            .new_leaf_with_context(
                root_layout_style(viewport),
                DocumentLayoutNode {
                    id: root.clone(),
                    element: Element::Root,
                    text: None,
                },
            )
            .expect("root layout node can be created");

        let mut elements = HashMap::new();
        elements.insert(
            root.clone(),
            DocumentElement {
                id: root.clone(),
                spec: ElementSpec::new(Element::Root),
                text: None,
                computed_style: root_sized_style(ComputedStyle::default(), viewport),
                scroll_offset: Point::ZERO,
                layout_node: root_node,
            },
        );

        let mut layout_to_element = HashMap::new();
        layout_to_element.insert(root_node, root.clone());

        Self {
            instance_id: NEXT_DOCUMENT_INSTANCE_ID.fetch_add(1, Ordering::Relaxed),
            viewport,
            revision: 0,
            layout,
            calc_lengths: HashMap::new(),
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

    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub(crate) fn instance_id(&self) -> u64 {
        self.instance_id
    }

    pub fn element_ids(&self) -> Vec<ElementId> {
        let mut ids = self.elements.keys().cloned().collect::<Vec<_>>();
        ids.sort();
        ids
    }

    pub(crate) fn element_spec(&self, id: &ElementId) -> DocumentResult<&ElementSpec> {
        Ok(&self.element(id)?.spec)
    }

    pub fn append_element(
        &mut self,
        parent: impl Into<ElementId>,
        id: impl Into<ElementId>,
        spec: ElementSpec,
    ) -> DocumentResult<()> {
        self.append_node(parent.into(), id.into(), spec, None)?;
        Ok(())
    }

    pub fn append_text(
        &mut self,
        parent: impl Into<ElementId>,
        id: impl Into<ElementId>,
        spec: ElementSpec,
        text: impl Into<TextContent>,
    ) -> DocumentResult<()> {
        self.append_node(parent.into(), id.into(), spec, Some(text.into()))?;
        Ok(())
    }

    pub(crate) fn append_element_tree(
        &mut self,
        parent: impl Into<ElementId>,
        element: DocumentNode,
    ) -> DocumentResult<NodeId> {
        let parent = parent.into();
        let id = element.id.clone();
        let node = self.append_node(parent, id.clone(), element.spec, element.text)?;
        for child in element.children {
            self.append_element_tree(id.clone(), child)?;
        }
        Ok(node)
    }

    pub fn set_text(
        &mut self,
        id: impl Into<ElementId>,
        text: impl Into<TextContent>,
    ) -> DocumentResult<bool> {
        let id = id.into();
        let text = Some(text.into());
        let node = self.element(&id)?.layout_node;
        if self.element(&id)?.text == text {
            return Ok(false);
        }
        self.element_mut(&id)?.text = text.clone();
        if let Some(context) = self.layout.get_node_context_mut(node) {
            context.text = text;
        }
        self.layout.mark_dirty(node).map_err(layout_error)?;
        self.revision = self.revision.wrapping_add(1);
        Ok(true)
    }

    pub fn set_value(
        &mut self,
        id: impl Into<ElementId>,
        value: impl Into<String>,
    ) -> DocumentResult<bool> {
        let id = id.into();
        let value = Some(value.into());
        let element = self.element_mut(&id)?;
        if element.spec.value == value {
            return Ok(false);
        }
        element.spec.value = value;
        self.revision = self.revision.wrapping_add(1);
        Ok(true)
    }

    pub fn set_attribute(
        &mut self,
        id: impl Into<ElementId>,
        name: impl Into<String>,
        value: impl Into<String>,
    ) -> DocumentResult<bool> {
        let id = id.into();
        let name = name.into();
        let value = value.into();
        let element = self.element_mut(&id)?;
        if element
            .spec
            .attributes
            .get(&name)
            .is_some_and(|existing| existing == &value)
        {
            return Ok(false);
        }
        element.spec.attributes.insert(name, value);
        self.revision = self.revision.wrapping_add(1);
        Ok(true)
    }

    pub fn set_attributes<I, K, V>(
        &mut self,
        id: impl Into<ElementId>,
        attributes: I,
    ) -> DocumentResult<usize>
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        let id = id.into();
        let mut changed = 0;
        for (name, value) in attributes {
            changed += usize::from(self.set_attribute(id.clone(), name, value)?);
        }
        Ok(changed)
    }

    pub fn set_data(
        &mut self,
        id: impl Into<ElementId>,
        name: impl AsRef<str>,
        value: impl Into<String>,
    ) -> DocumentResult<bool> {
        self.set_attribute(id, prefixed_attribute_name("data-", name), value)
    }

    pub fn set_aria(
        &mut self,
        id: impl Into<ElementId>,
        name: impl AsRef<str>,
        value: impl Into<String>,
    ) -> DocumentResult<bool> {
        self.set_attribute(id, prefixed_attribute_name("aria-", name), value)
    }

    pub fn remove_attribute(
        &mut self,
        id: impl Into<ElementId>,
        name: impl Into<String>,
    ) -> DocumentResult<bool> {
        let id = id.into();
        let name = name.into();
        let element = self.element_mut(&id)?;
        if element.spec.attributes.remove(&name).is_none() {
            return Ok(false);
        }
        self.revision = self.revision.wrapping_add(1);
        Ok(true)
    }

    pub fn remove_attributes<I, K>(
        &mut self,
        id: impl Into<ElementId>,
        names: I,
    ) -> DocumentResult<usize>
    where
        I: IntoIterator<Item = K>,
        K: Into<String>,
    {
        let id = id.into();
        let mut changed = 0;
        for name in names {
            changed += usize::from(self.remove_attribute(id.clone(), name)?);
        }
        Ok(changed)
    }

    pub fn remove_data(
        &mut self,
        id: impl Into<ElementId>,
        name: impl AsRef<str>,
    ) -> DocumentResult<bool> {
        self.remove_attribute(id, prefixed_attribute_name("data-", name))
    }

    pub fn remove_aria(
        &mut self,
        id: impl Into<ElementId>,
        name: impl AsRef<str>,
    ) -> DocumentResult<bool> {
        self.remove_attribute(id, prefixed_attribute_name("aria-", name))
    }

    pub fn add_class(
        &mut self,
        id: impl Into<ElementId>,
        class: impl Into<ClassName>,
    ) -> DocumentResult<bool> {
        let id = id.into();
        let class = class.into();
        let element = self.element_mut(&id)?;
        if element.spec.classes.contains(&class) {
            return Ok(false);
        }
        element.spec.classes.push(class);
        self.revision = self.revision.wrapping_add(1);
        Ok(true)
    }

    pub fn add_classes<I, C>(
        &mut self,
        id: impl Into<ElementId>,
        classes: I,
    ) -> DocumentResult<usize>
    where
        I: IntoIterator<Item = C>,
        C: Into<ClassName>,
    {
        let id = id.into();
        let mut changed = 0;
        for class in classes {
            changed += usize::from(self.add_class(id.clone(), class)?);
        }
        Ok(changed)
    }

    pub fn remove_class(
        &mut self,
        id: impl Into<ElementId>,
        class: impl Into<ClassName>,
    ) -> DocumentResult<bool> {
        let id = id.into();
        let class = class.into();
        let element = self.element_mut(&id)?;
        let previous_len = element.spec.classes.len();
        element.spec.classes.retain(|existing| existing != &class);
        if element.spec.classes.len() == previous_len {
            return Ok(false);
        }
        self.revision = self.revision.wrapping_add(1);
        Ok(true)
    }

    pub fn remove_classes<I, C>(
        &mut self,
        id: impl Into<ElementId>,
        classes: I,
    ) -> DocumentResult<usize>
    where
        I: IntoIterator<Item = C>,
        C: Into<ClassName>,
    {
        let id = id.into();
        let mut changed = 0;
        for class in classes {
            changed += usize::from(self.remove_class(id.clone(), class)?);
        }
        Ok(changed)
    }

    pub fn toggle_class(
        &mut self,
        id: impl Into<ElementId>,
        class: impl Into<ClassName>,
    ) -> DocumentResult<bool> {
        let id = id.into();
        let class = class.into();
        if self.element(&id)?.spec.classes.contains(&class) {
            self.remove_class(id, class)
        } else {
            self.add_class(id, class)
        }
    }

    pub fn set_selected(
        &mut self,
        id: impl Into<ElementId>,
        selected: bool,
    ) -> DocumentResult<bool> {
        let id = id.into();
        let element = self.element_mut(&id)?;
        if element.spec.selected == selected {
            return Ok(false);
        }
        element.spec.selected = selected;
        self.revision = self.revision.wrapping_add(1);
        Ok(true)
    }

    pub fn set_checked(&mut self, id: impl Into<ElementId>, checked: bool) -> DocumentResult<bool> {
        self.set_selected(id, checked)
    }

    pub fn check(&mut self, id: impl Into<ElementId>) -> DocumentResult<bool> {
        self.set_checked(id, true)
    }

    pub fn uncheck(&mut self, id: impl Into<ElementId>) -> DocumentResult<bool> {
        self.set_checked(id, false)
    }

    pub fn select(&mut self, id: impl Into<ElementId>) -> DocumentResult<bool> {
        self.set_selected(id, true)
    }

    pub fn deselect(&mut self, id: impl Into<ElementId>) -> DocumentResult<bool> {
        self.set_selected(id, false)
    }

    pub fn set_disabled(
        &mut self,
        id: impl Into<ElementId>,
        disabled: bool,
    ) -> DocumentResult<bool> {
        let id = id.into();
        let element = self.element_mut(&id)?;
        if element.spec.disabled == disabled {
            return Ok(false);
        }
        element.spec.disabled = disabled;
        self.revision = self.revision.wrapping_add(1);
        Ok(true)
    }

    pub fn set_enabled(&mut self, id: impl Into<ElementId>, enabled: bool) -> DocumentResult<bool> {
        self.set_disabled(id, !enabled)
    }

    pub fn disable(&mut self, id: impl Into<ElementId>) -> DocumentResult<bool> {
        self.set_disabled(id, true)
    }

    pub fn enable(&mut self, id: impl Into<ElementId>) -> DocumentResult<bool> {
        self.set_disabled(id, false)
    }

    pub fn set_focused(&mut self, id: impl Into<ElementId>, focused: bool) -> DocumentResult<bool> {
        let id = id.into();
        let element = self.element_mut(&id)?;
        if element.spec.focused == focused {
            return Ok(false);
        }
        element.spec.focused = focused;
        self.revision = self.revision.wrapping_add(1);
        Ok(true)
    }

    pub fn focus(&mut self, id: impl Into<ElementId>) -> DocumentResult<bool> {
        self.set_focused(id, true)
    }

    pub fn blur(&mut self, id: impl Into<ElementId>) -> DocumentResult<bool> {
        self.set_focused(id, false)
    }

    pub fn reparent(
        &mut self,
        id: impl Into<ElementId>,
        new_parent: impl Into<ElementId>,
    ) -> DocumentResult<()> {
        let id = id.into();
        let new_parent = new_parent.into();
        let node = self.element(&id)?.layout_node;
        let parent_node = self.element(&new_parent)?.layout_node;
        self.layout
            .add_child(parent_node, node)
            .map_err(layout_error)?;
        self.revision = self.revision.wrapping_add(1);
        Ok(())
    }

    pub fn remove(&mut self, id: impl Into<ElementId>) -> DocumentResult<()> {
        let id = id.into();
        if id == self.root {
            return Err(DocumentError::MissingElement(id));
        }
        self.remove_subtree(&id)
    }

    #[cfg(test)]
    pub(crate) fn layout_node(&self, id: impl Into<ElementId>) -> Option<NodeId> {
        self.elements
            .get(&id.into())
            .map(|element| element.layout_node)
    }

    #[cfg(test)]
    pub(crate) fn layout_style(&self, id: impl Into<ElementId>) -> DocumentResult<&LayoutStyle> {
        let node = self.element(&id.into())?.layout_node;
        self.layout.style(node).map_err(layout_error)
    }

    pub(crate) fn layout_dirty(&self, id: impl Into<ElementId>) -> DocumentResult<bool> {
        let node = self.element(&id.into())?.layout_node;
        self.layout.dirty(node).map_err(layout_error)
    }

    pub(crate) fn mark_layout_dirty(&mut self) -> DocumentResult<()> {
        let nodes = self
            .elements
            .values()
            .map(|element| element.layout_node)
            .collect::<Vec<_>>();
        for node in nodes {
            self.layout.mark_dirty(node).map_err(layout_error)?;
        }
        Ok(())
    }

    pub(crate) fn apply_computed_style(
        &mut self,
        id: impl Into<ElementId>,
        style: &ComputedStyle,
    ) -> DocumentResult<bool> {
        let id = id.into();
        let mut style = style.clone();
        style.normalize_overflow_axes();
        let node = self.element(&id)?.layout_node;
        let layout_style = self.layout_style_from_computed(&style);
        let layout_changed = self.set_layout_style_if_changed(node, layout_style)?;
        self.element_mut(&id)?.computed_style = style;
        Ok(layout_changed)
    }

    pub(crate) fn apply_stylesheet(
        &mut self,
        stylesheet: &StyleSheet,
        states: &HashMap<ElementId, ElementState>,
    ) -> DocumentResult<StyleResolutionReport> {
        self.apply_stylesheet_with_container_queries(stylesheet, states, true)
    }

    pub(crate) fn apply_stylesheet_without_container_queries(
        &mut self,
        stylesheet: &StyleSheet,
        states: &HashMap<ElementId, ElementState>,
    ) -> DocumentResult<StyleResolutionReport> {
        self.apply_stylesheet_with_container_queries(stylesheet, states, false)
    }

    fn apply_stylesheet_with_container_queries(
        &mut self,
        stylesheet: &StyleSheet,
        states: &HashMap<ElementId, ElementState>,
        resolve_container_queries: bool,
    ) -> DocumentResult<StyleResolutionReport> {
        let mut report = StyleResolutionReport::default();
        let mut ancestors = Vec::new();
        self.apply_stylesheet_subtree(
            self.root.clone(),
            ApplyStylesheetContext {
                stylesheet,
                states,
                resolve_container_queries,
                report: &mut report,
            },
            ApplyStylesheetTraversal {
                position: None,
                ancestors: &mut ancestors,
                previous_siblings: &[],
            },
        )?;
        report.layout_changed |= self.apply_table_grid_styles()?;

        Ok(report)
    }

    fn apply_stylesheet_subtree(
        &mut self,
        id: ElementId,
        context: ApplyStylesheetContext<'_>,
        traversal: ApplyStylesheetTraversal<'_>,
    ) -> DocumentResult<OwnedStyleMatchContext> {
        let ApplyStylesheetContext {
            stylesheet,
            states,
            resolve_container_queries,
            report,
        } = context;
        let ApplyStylesheetTraversal {
            position,
            ancestors,
            previous_siblings,
        } = traversal;
        report.visited += 1;
        let element = self.snapshot_element(&id)?;
        let ancestor_contexts = ancestors
            .iter()
            .map(|context| StyleMatchContext {
                element: &context.element,
                state: context.state.as_ref(),
                position: context.position,
            })
            .collect::<Vec<_>>();
        let previous_sibling_contexts = previous_siblings
            .iter()
            .map(|context| StyleMatchContext {
                element: &context.element,
                state: context.state.as_ref(),
                position: context.position,
            })
            .collect::<Vec<_>>();
        let computed = resolve_style_with_position(
            stylesheet,
            StyleResolutionContext {
                element: &element,
                state: states.get(&id),
                position,
                ancestors: &ancestor_contexts,
                previous_siblings: &previous_sibling_contexts,
                viewport: self.viewport,
                container: if resolve_container_queries {
                    self.parent_container_size(&id)?
                } else {
                    None
                },
            },
        );
        let computed = if id == self.root {
            root_sized_style(computed, self.viewport)
        } else {
            computed
        };
        let rendered = states
            .get(&id)
            .and_then(|state| state.rendered_style.clone())
            .map(|style| {
                if id == self.root {
                    root_sized_style(style, self.viewport)
                } else {
                    style
                }
            })
            .unwrap_or(computed);
        let invalidation = classify_computed_style_change(
            Some(&self.element(&id)?.computed_style),
            Some(&rendered),
        );
        report.paint_changed |= invalidation.paint_changed;
        let scroll_offset = states
            .get(&id)
            .map(|state| Point::new(state.scroll_x, state.scroll_y))
            .unwrap_or(Point::ZERO);
        self.element_mut(&id)?.scroll_offset = scroll_offset;
        report.layout_changed |= self.apply_computed_style(id.clone(), &rendered)?;

        let context = OwnedStyleMatchContext {
            element,
            state: states.get(&id).cloned(),
            position,
        };
        let children = self.children(id)?;
        let sibling_count = children.len();
        ancestors.push(context.clone());
        let mut previous_siblings = Vec::new();
        for (index, child) in children.into_iter().enumerate() {
            let child_position = Some(ChildPosition::new(index, sibling_count));
            let child_context = self.apply_stylesheet_subtree(
                child,
                ApplyStylesheetContext {
                    stylesheet,
                    states,
                    resolve_container_queries,
                    report,
                },
                ApplyStylesheetTraversal {
                    position: child_position,
                    ancestors,
                    previous_siblings: &previous_siblings,
                },
            )?;
            previous_siblings.push(child_context);
        }
        ancestors.pop();

        Ok(context)
    }

    pub(crate) fn parent_container_size(&self, id: &ElementId) -> DocumentResult<Option<Size>> {
        let Some(parent_id) = self.parent(id.clone())? else {
            return Ok(None);
        };
        let parent = self.element(&parent_id)?;
        let layout = self
            .layout
            .layout(parent.layout_node)
            .map_err(layout_error)?;
        Ok(Some(Size::new(layout.size.width, layout.size.height)))
    }

    pub(crate) fn apply_scroll_offsets(&mut self, states: &HashMap<ElementId, ElementState>) {
        for (id, element) in &mut self.elements {
            element.scroll_offset = states
                .get(id)
                .map(|state| Point::new(state.scroll_x, state.scroll_y))
                .unwrap_or(Point::ZERO);
        }
    }

    #[cfg(test)]
    pub(crate) fn compute_layout(&mut self) -> DocumentResult<()> {
        let mut text_measurer = FallbackTextMeasurer;
        self.compute_layout_with_text_measurer(&mut text_measurer)
    }

    pub(crate) fn compute_layout_with_text_measurer(
        &mut self,
        text_measurer: &mut dyn TextMeasurer,
    ) -> DocumentResult<()> {
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
                        &input.text,
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

    pub(crate) fn layout_rect(&self, id: impl Into<ElementId>) -> DocumentResult<DocumentRect> {
        let node = self.element(&id.into())?.layout_node;
        let layout = self.layout.layout(node).map_err(layout_error)?;
        Ok(DocumentRect::new(
            layout.location.x,
            layout.location.y,
            layout.size.width,
            layout.size.height,
        ))
    }

    #[cfg(test)]
    pub(crate) fn resolved_layout(&self) -> DocumentResult<ResolvedElement> {
        let mut text_measurer = FallbackTextMeasurer;
        self.resolved_layout_with_text_measurer(&mut text_measurer)
    }

    pub(crate) fn resolved_layout_with_text_measurer(
        &self,
        text_measurer: &mut dyn TextMeasurer,
    ) -> DocumentResult<ResolvedElement> {
        let mut anchors = HashMap::new();
        let mut boundaries = HashMap::new();
        self.resolved_element(
            &self.root,
            ResolveParentFrame {
                origin: Point::ZERO,
                scroll_offset: Point::ZERO,
                clip: ClipRect::from_rect(DocumentRect::new(
                    0.0,
                    0.0,
                    self.viewport.width,
                    self.viewport.height,
                )),
            },
            &mut ResolveTreeContext {
                text_measurer,
                anchors: &mut anchors,
                boundaries: &mut boundaries,
            },
        )
    }

    pub(crate) fn scroll_limits(&self) -> DocumentResult<HashMap<ElementId, Size>> {
        let mut limits = HashMap::new();
        for id in self.element_ids() {
            let element = self.element(&id)?;
            let style = &element.computed_style;
            if !style.overflow_x.is_scrollable() && !style.overflow_y.is_scrollable() {
                continue;
            }

            let layout = self
                .layout
                .layout(element.layout_node)
                .map_err(layout_error)?;
            let content_size = self.scroll_content_size(element)?;
            let max_scroll = layout_scroll::scroll_limits(
                to_layout_size(content_size),
                des_layout::geometry::Size {
                    width: layout.size.width,
                    height: layout.size.height,
                },
                to_layout_insets(style.border_width),
            );
            let max_scroll = Size::new(max_scroll.width, max_scroll.height);
            if max_scroll.width > 0.0 || max_scroll.height > 0.0 {
                limits.insert(id, max_scroll);
            }
        }
        Ok(limits)
    }

    fn scroll_content_size(&self, element: &DocumentElement) -> DocumentResult<Size> {
        let layout = self
            .layout
            .layout(element.layout_node)
            .map_err(layout_error)?;
        let mut content_size = Size::new(layout.content_size.width, layout.content_size.height);
        for child_node in self
            .layout
            .children(element.layout_node)
            .map_err(layout_error)?
        {
            let Some(child_id) = self.layout_to_element.get(&child_node) else {
                continue;
            };
            let child = self.element(child_id)?;
            let child_layout = self.layout.layout(child_node).map_err(layout_error)?;
            content_size.width = content_size.width.max(
                child_layout.location.x
                    + child_layout.size.width
                    + child.computed_style.margin.right,
            );
            content_size.height = content_size.height.max(
                child_layout.location.y
                    + child_layout.size.height
                    + child.computed_style.margin.bottom,
            );
        }
        Ok(content_size)
    }

    #[cfg(test)]
    pub(crate) fn resolve_layout(
        &mut self,
        stylesheet: &StyleSheet,
        states: &HashMap<ElementId, ElementState>,
    ) -> DocumentResult<ResolvedElement> {
        let mut text_measurer = FallbackTextMeasurer;
        self.resolve_layout_with_text_measurer(stylesheet, states, &mut text_measurer)
    }

    #[cfg(test)]
    pub(crate) fn resolve_layout_with_text_measurer(
        &mut self,
        stylesheet: &StyleSheet,
        states: &HashMap<ElementId, ElementState>,
        text_measurer: &mut dyn TextMeasurer,
    ) -> DocumentResult<ResolvedElement> {
        let report = self.apply_stylesheet(stylesheet, states)?;
        if report.layout_changed || self.layout_dirty(self.root.clone())? {
            self.compute_layout_with_text_measurer(text_measurer)?;
        }
        self.resolved_layout_with_text_measurer(text_measurer)
    }

    pub fn parent(&self, id: impl Into<ElementId>) -> DocumentResult<Option<ElementId>> {
        let node = self.element(&id.into())?.layout_node;
        Ok(self
            .layout
            .parent(node)
            .and_then(|parent| self.layout_to_element.get(&parent).cloned()))
    }

    pub fn children(&self, id: impl Into<ElementId>) -> DocumentResult<Vec<ElementId>> {
        let node = self.element(&id.into())?.layout_node;
        self.layout
            .children(node)
            .map_err(layout_error)?
            .into_iter()
            .map(|child| {
                self.layout_to_element.get(&child).cloned().ok_or_else(|| {
                    DocumentError::Layout(format!("Layout node {child:?} is not indexed"))
                })
            })
            .collect()
    }

    fn append_node(
        &mut self,
        parent: ElementId,
        id: ElementId,
        spec: ElementSpec,
        text: Option<TextContent>,
    ) -> DocumentResult<NodeId> {
        if self.elements.contains_key(&id) {
            return Err(DocumentError::DuplicateElement(id));
        }

        let parent_node = self.element(&parent)?.layout_node;
        let node = self
            .layout
            .new_leaf_with_context(
                LayoutStyle::default(),
                DocumentLayoutNode {
                    id: id.clone(),
                    element: spec.element,
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
            DocumentElement {
                id,
                spec,
                text,
                computed_style: ComputedStyle::default(),
                scroll_offset: Point::ZERO,
                layout_node: node,
            },
        );
        self.revision = self.revision.wrapping_add(1);

        Ok(node)
    }

    fn remove_subtree(&mut self, id: &ElementId) -> DocumentResult<()> {
        let children = self.children(id.clone())?;
        for child in children {
            self.remove_subtree(&child)?;
        }

        let element = self
            .elements
            .remove(id)
            .ok_or_else(|| DocumentError::MissingElement(id.clone()))?;
        self.layout_to_element.remove(&element.layout_node);
        self.layout
            .remove(element.layout_node)
            .map_err(layout_error)?;
        self.revision = self.revision.wrapping_add(1);
        Ok(())
    }

    fn element(&self, id: &ElementId) -> DocumentResult<&DocumentElement> {
        self.elements
            .get(id)
            .ok_or_else(|| DocumentError::MissingElement(id.clone()))
    }

    fn element_mut(&mut self, id: &ElementId) -> DocumentResult<&mut DocumentElement> {
        self.elements
            .get_mut(id)
            .ok_or_else(|| DocumentError::MissingElement(id.clone()))
    }

    fn resolved_element(
        &self,
        id: &ElementId,
        parent: ResolveParentFrame,
        context: &mut ResolveTreeContext<'_>,
    ) -> DocumentResult<ResolvedElement> {
        let element = self.element(id)?;
        let raw_rect = self.layout_rect(id.as_str())?;
        let (rect, floating) = resolved_document_rect(
            raw_rect,
            &element.computed_style,
            self.viewport,
            parent.origin,
            parent.scroll_offset,
            context.anchors,
            context.boundaries,
        );
        let clip_rect = if element.computed_style.position == Position::AbsoluteViewport {
            ClipRect::from_rect(DocumentRect::new(
                0.0,
                0.0,
                self.viewport.width,
                self.viewport.height,
            ))
        } else {
            parent.clip
        };
        let normalized_text = element
            .text
            .as_ref()
            .map(|text| NormalizedText::from_content(text, element.computed_style.text_layout));
        let text_layout = normalized_text.as_ref().map(|text| {
            let content_width = rect
                .inset(element.computed_style.border_width)
                .inset(element.computed_style.padding)
                .size
                .width;
            measure_normalized_text(
                text,
                &element.computed_style,
                content_width,
                context.text_measurer,
            )
        });
        context.anchors.insert(element.id.clone(), rect);
        context.boundaries.insert(
            element.id.clone(),
            rect.inset(element.computed_style.border_width),
        );
        let children = self.resolved_children(
            id,
            rect,
            element.scroll_offset,
            child_clip_rect(rect, &element.computed_style, clip_rect),
            context,
        )?;

        Ok(ResolvedElement {
            id: element.id.clone(),
            element: element.spec.element,
            classes: element.spec.classes.clone(),
            role: element.spec.role.clone(),
            attributes: element.spec.attributes.clone(),
            behavior_hooks: element.spec.behavior_hooks.clone(),
            rect,
            clip_rect,
            style: element.computed_style.clone(),
            text: element.text.clone(),
            normalized_text,
            text_layout,
            selectable_text: element.spec.selectable_text && element.text.is_some(),
            copyable_text: element.spec.selectable_text
                && element.spec.copyable_text
                && element.text.is_some(),
            value: element.spec.value.clone(),
            glyph: element.spec.glyph,
            interactive: element.spec.interactive && !element.spec.disabled,
            selected: element.spec.selected,
            disabled: element.spec.disabled,
            focused: element.spec.focused,
            floating,
            children,
        })
    }

    fn resolved_children(
        &self,
        id: &ElementId,
        parent_rect: DocumentRect,
        parent_scroll_offset: Point,
        child_clip: ClipRect,
        context: &mut ResolveTreeContext<'_>,
    ) -> DocumentResult<Vec<ResolvedElement>> {
        let children = self.children(id.clone())?;
        let mut resolved = vec![None; children.len()];

        for (index, child) in children.iter().enumerate() {
            if self.element(child)?.computed_style.position != Position::Flow {
                continue;
            }
            resolved[index] = Some(self.resolved_element(
                child,
                ResolveParentFrame {
                    origin: child_parent_origin(
                        parent_rect,
                        &self.element(id)?.computed_style,
                        &self.element(child)?.computed_style,
                    ),
                    scroll_offset: parent_scroll_offset,
                    clip: child_clip,
                },
                context,
            )?);
        }

        for (index, child) in children.iter().enumerate() {
            if resolved[index].is_some() {
                continue;
            }
            resolved[index] = Some(self.resolved_element(
                child,
                ResolveParentFrame {
                    origin: child_parent_origin(
                        parent_rect,
                        &self.element(id)?.computed_style,
                        &self.element(child)?.computed_style,
                    ),
                    scroll_offset: parent_scroll_offset,
                    clip: child_clip,
                },
                context,
            )?);
        }

        Ok(resolved.into_iter().flatten().collect())
    }

    fn snapshot_element(&self, id: &ElementId) -> DocumentResult<DocumentNode> {
        let element = self.element(id)?;
        Ok(DocumentNode {
            id: element.id.clone(),
            spec: element.spec.clone(),
            text: element.text.clone(),
            children: Vec::new(),
        })
    }

    pub(crate) fn element_tree(&self) -> DocumentResult<DocumentNode> {
        self.element_subtree(&self.root)
    }

    fn element_subtree(&self, id: &ElementId) -> DocumentResult<DocumentNode> {
        let element = self.element(id)?;
        let children = self
            .children(id.clone())?
            .into_iter()
            .map(|child| self.element_subtree(&child))
            .collect::<DocumentResult<Vec<_>>>()?;

        Ok(DocumentNode {
            id: element.id.clone(),
            spec: element.spec.clone(),
            text: element.text.clone(),
            children,
        })
    }

    fn measure_inputs(&self) -> HashMap<NodeId, DocumentMeasureInput> {
        self.elements
            .values()
            .filter_map(|element| {
                element.text.as_ref().map(|text| {
                    (
                        element.layout_node,
                        DocumentMeasureInput {
                            text: text.clone(),
                            style: element.computed_style.clone(),
                        },
                    )
                })
            })
            .collect()
    }

    fn set_layout_style_if_changed(
        &mut self,
        node: NodeId,
        style: LayoutStyle,
    ) -> DocumentResult<bool> {
        if self.layout.style(node).map_err(layout_error)? == &style {
            return Ok(false);
        }

        self.layout.set_style(node, style).map_err(layout_error)?;
        Ok(true)
    }

    fn apply_table_grid_styles(&mut self) -> DocumentResult<bool> {
        let ids = self.element_ids();
        let mut layout_changed = false;
        for id in ids {
            let element = self.element(&id)?;
            if element.spec.element != Element::Thead && element.spec.element != Element::Tr {
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
            style.size.height = length(if element.spec.element == Element::Thead {
                table.header_height
            } else {
                table.row_height
            });
            layout_changed |= self.set_layout_style_if_changed(node, style)?;

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
                child_style.grid_column = des_layout::geometry::Line {
                    start: GridPlacement::Line((column_index + 1).into()),
                    end: GridPlacement::Line((column_index + 2).into()),
                };
                layout_changed |= self.set_layout_style_if_changed(child_node, child_style)?;
            }
        }

        Ok(layout_changed)
    }
}

#[derive(Default)]
pub struct DocumentBuilder {
    children: Vec<DocumentNode>,
}

/// Reusable egui-free widget behavior over the document contract.
///
/// A `DocumentWidget` owns the structure it contributes, optional style rules,
/// and optional retained-state projection. It does not own renderer state or app
/// state; hosts compose widgets into a [`crate::DocumentView`] and project fresh
/// app state through the widget on each update as needed.
pub trait DocumentWidget {
    /// Renders this widget's retained document structure.
    fn render(&self, ui: &mut DocumentBuilder);

    /// Appends the style rules required by this widget.
    fn push_styles(&self, _stylesheet: &mut StyleSheet) {}

    /// Returns a single-element projection patch for simple state projection.
    fn projection_patch(&self) -> Option<(ElementId, ElementProjectionPatch)> {
        None
    }

    /// Returns reusable projection patches for widgets that own several elements.
    ///
    /// The default preserves the simpler [`DocumentWidget::projection_patch`]
    /// convention. Multi-element widgets can override this method instead of
    /// hand-writing [`DocumentWidget::push_projection`].
    fn projection_patches(&self) -> Vec<(ElementId, ElementProjectionPatch)> {
        self.projection_patch().into_iter().collect()
    }

    /// Pushes this widget's retained-state projection into a caller-owned batch.
    fn push_projection(&self, projection: &mut DocumentProjection) {
        projection.patches(self.projection_patches());
    }

    /// Builds the stylesheet declared by this widget.
    fn stylesheet(&self) -> StyleSheet {
        let mut stylesheet = StyleSheet::new();
        self.push_styles(&mut stylesheet);
        stylesheet
    }

    /// Builds the retained-state projection declared by this widget.
    fn projection(&self) -> DocumentProjection {
        let mut projection = DocumentProjection::new();
        self.push_projection(&mut projection);
        projection
    }

    /// Builds the retained document structure declared by this widget.
    ///
    /// This is the lightest egui-free contract surface for widget tests: it
    /// proves the widget's element ids, roles, classes, hooks, and authored
    /// structure without resolving layout or creating a view.
    fn document(&self, viewport: Size) -> Document {
        Document::build(viewport, |ui| {
            ui.widget(self);
        })
    }

    /// Builds this widget's retained document structure and applies its projection.
    fn projected_document(
        &self,
        viewport: Size,
    ) -> DocumentResult<(DocumentProjectionReport, Document)> {
        let mut document = self.document(viewport);
        let report = self.projection().apply_to(&mut document)?;
        Ok((report, document))
    }

    /// Creates a ready-to-update document view containing this widget.
    fn view(&self, viewport: Size) -> crate::DocumentView {
        self.view_with_stylesheet(viewport, StyleSheet::new())
    }

    /// Creates a view containing this widget, returning projection errors explicitly.
    fn try_view(&self, viewport: Size) -> DocumentResult<crate::DocumentView> {
        self.try_view_with_stylesheet(viewport, StyleSheet::new())
    }

    /// Creates a document view containing this widget and an external stylesheet.
    fn view_with_stylesheet(&self, viewport: Size, stylesheet: StyleSheet) -> crate::DocumentView {
        crate::DocumentView::build_widget(viewport, stylesheet, self)
    }

    /// Creates a document view containing this widget and strict CSS stylesheet rules.
    fn view_with_css(
        &self,
        viewport: Size,
        css: &str,
    ) -> Result<crate::DocumentView, crate::CssParseError>
    where
        Self: Sized,
    {
        let stylesheet = StyleSheet::from_css(css)?;
        Ok(self.view_with_stylesheet(viewport, stylesheet))
    }

    /// Creates a CSS-backed view, returning CSS and widget projection errors explicitly.
    fn try_view_with_css(
        &self,
        viewport: Size,
        css: &str,
    ) -> DocumentAuthoringResult<crate::DocumentView>
    where
        Self: Sized,
    {
        let stylesheet = StyleSheet::from_css(css)?;
        Ok(self.try_view_with_stylesheet(viewport, stylesheet)?)
    }

    /// Creates a document view containing this widget and browser-forgiving CSS rules.
    fn view_with_css_forgiving(
        &self,
        viewport: Size,
        css: &str,
    ) -> Result<crate::DocumentView, crate::CssParseError>
    where
        Self: Sized,
    {
        let stylesheet = StyleSheet::from_css_forgiving(css)?;
        Ok(self.view_with_stylesheet(viewport, stylesheet))
    }

    /// Creates a forgiving CSS-backed view, returning all authoring errors explicitly.
    fn try_view_with_css_forgiving(
        &self,
        viewport: Size,
        css: &str,
    ) -> DocumentAuthoringResult<crate::DocumentView>
    where
        Self: Sized,
    {
        let stylesheet = StyleSheet::from_css_forgiving(css)?;
        Ok(self.try_view_with_stylesheet(viewport, stylesheet)?)
    }

    /// Creates a styled view containing this widget, returning projection errors explicitly.
    fn try_view_with_stylesheet(
        &self,
        viewport: Size,
        stylesheet: StyleSheet,
    ) -> DocumentResult<crate::DocumentView> {
        crate::DocumentView::try_build_widget(viewport, stylesheet, self)
    }

    /// Builds this widget, resolves it, and returns document output.
    fn update(&self, viewport: Size) -> crate::DocumentOutput
    where
        Self: Sized,
    {
        self.view(viewport).update()
    }

    /// Builds this widget, resolves it, and returns projection errors explicitly.
    fn try_update(&self, viewport: Size) -> DocumentResult<crate::DocumentOutput>
    where
        Self: Sized,
    {
        Ok(self.try_view(viewport)?.update())
    }
}

/// Widget convention for declaring typed app actions alongside document hooks.
///
/// The widget still renders through [`DocumentWidget`]. This extension lets it
/// pair authored behavior hooks such as clicks, keyboard input, or context menus
/// with typed Rust actions without coupling the widget to an egui adapter.
pub trait DocumentActionWidget<Action>: DocumentWidget {
    /// Returns a single command binding for simple action widgets.
    fn command_binding(&self) -> Option<DocumentCommandBinding<Action>> {
        None
    }

    /// Returns command bindings declared by this widget.
    ///
    /// The default preserves the simpler [`DocumentActionWidget::command_binding`]
    /// convention. Widgets with several behavior hooks can override this method
    /// instead of hand-writing [`DocumentActionWidget::push_commands`].
    fn command_bindings(&self) -> Vec<DocumentCommandBinding<Action>> {
        self.command_binding().into_iter().collect()
    }

    /// Pushes this widget's command-to-action bindings into a caller-owned registry.
    fn push_commands(&self, registry: &mut DocumentCommandRegistry<Action>) {
        registry.push_bindings(self.command_bindings());
    }

    /// Builds the command registry declared by this widget.
    fn commands(&self) -> DocumentCommandRegistry<Action> {
        let mut registry = DocumentCommandRegistry::new();
        self.push_commands(&mut registry);
        registry
    }

    /// Alias for [`DocumentActionWidget::commands`] for call sites that prefer registry language.
    fn command_registry(&self) -> DocumentCommandRegistry<Action>
    where
        Self: Sized,
    {
        self.commands()
    }

    /// Builds this widget, resolves it, and collects typed app actions.
    fn update_actions(&self, viewport: Size) -> crate::DocumentActionFrame<Action>
    where
        Self: Sized,
        Action: Clone,
    {
        self.action_surface(viewport).update_actions()
    }

    /// Builds this widget, resolves it, and returns projection errors explicitly.
    fn try_update_actions(
        &self,
        viewport: Size,
    ) -> DocumentResult<crate::DocumentActionFrame<Action>>
    where
        Self: Sized,
        Action: Clone,
    {
        Ok(self.try_action_surface(viewport)?.update_actions())
    }

    /// Builds this widget, routes input, and collects typed app actions.
    fn update_with_input_actions(
        &self,
        viewport: Size,
        input: DocumentInput,
    ) -> crate::DocumentActionFrame<Action>
    where
        Self: Sized,
        Action: Clone,
    {
        self.action_surface(viewport)
            .update_with_input_actions(input)
    }

    /// Builds this widget, routes input, and returns projection errors explicitly.
    fn try_update_with_input_actions(
        &self,
        viewport: Size,
        input: DocumentInput,
    ) -> DocumentResult<crate::DocumentActionFrame<Action>>
    where
        Self: Sized,
        Action: Clone,
    {
        Ok(self
            .try_action_surface(viewport)?
            .update_with_input_actions(input))
    }

    /// Builds this widget, resolves it, collects actions, and dispatches them.
    fn update_and_dispatch(
        &self,
        viewport: Size,
        handler: impl for<'frame> FnMut(&'frame DocumentCommandAction<Action>),
    ) -> (
        crate::DocumentActionFrame<Action>,
        DocumentCommandDispatchReport,
    )
    where
        Self: Sized,
        Action: Clone,
    {
        self.action_surface(viewport).update_and_dispatch(handler)
    }

    /// Builds this widget, resolves it, and dispatches only typed app action values.
    fn update_and_dispatch_action_values(
        &self,
        viewport: Size,
        handler: impl for<'frame> FnMut(&'frame Action),
    ) -> (
        crate::DocumentActionFrame<Action>,
        DocumentCommandDispatchReport,
    )
    where
        Self: Sized,
        Action: Clone,
    {
        self.action_surface(viewport)
            .update_and_dispatch_action_values(handler)
    }

    /// Builds this widget, resolves it, dispatches action values, and returns projection errors.
    fn try_update_and_dispatch_action_values(
        &self,
        viewport: Size,
        handler: impl for<'frame> FnMut(&'frame Action),
    ) -> DocumentResult<(
        crate::DocumentActionFrame<Action>,
        DocumentCommandDispatchReport,
    )>
    where
        Self: Sized,
        Action: Clone,
    {
        Ok(self
            .try_action_surface(viewport)?
            .update_and_dispatch_action_values(handler))
    }

    /// Builds this widget, resolves it, and returns projection errors explicitly.
    fn try_update_and_dispatch(
        &self,
        viewport: Size,
        handler: impl for<'frame> FnMut(&'frame DocumentCommandAction<Action>),
    ) -> DocumentResult<(
        crate::DocumentActionFrame<Action>,
        DocumentCommandDispatchReport,
    )>
    where
        Self: Sized,
        Action: Clone,
    {
        Ok(self
            .try_action_surface(viewport)?
            .update_and_dispatch(handler))
    }

    /// Builds this widget, routes input, collects actions, and dispatches them.
    fn update_with_input_and_dispatch(
        &self,
        viewport: Size,
        input: DocumentInput,
        handler: impl for<'frame> FnMut(&'frame DocumentCommandAction<Action>),
    ) -> (
        crate::DocumentActionFrame<Action>,
        DocumentCommandDispatchReport,
    )
    where
        Self: Sized,
        Action: Clone,
    {
        self.action_surface(viewport)
            .update_with_input_and_dispatch(input, handler)
    }

    /// Builds this widget, routes input, and dispatches only typed app action values.
    fn update_with_input_and_dispatch_action_values(
        &self,
        viewport: Size,
        input: DocumentInput,
        handler: impl for<'frame> FnMut(&'frame Action),
    ) -> (
        crate::DocumentActionFrame<Action>,
        DocumentCommandDispatchReport,
    )
    where
        Self: Sized,
        Action: Clone,
    {
        self.action_surface(viewport)
            .update_with_input_and_dispatch_action_values(input, handler)
    }

    /// Builds this widget, routes input, dispatches action values, and returns projection errors.
    fn try_update_with_input_and_dispatch_action_values(
        &self,
        viewport: Size,
        input: DocumentInput,
        handler: impl for<'frame> FnMut(&'frame Action),
    ) -> DocumentResult<(
        crate::DocumentActionFrame<Action>,
        DocumentCommandDispatchReport,
    )>
    where
        Self: Sized,
        Action: Clone,
    {
        Ok(self
            .try_action_surface(viewport)?
            .update_with_input_and_dispatch_action_values(input, handler))
    }

    /// Builds this widget, routes input, and returns projection errors explicitly.
    fn try_update_with_input_and_dispatch(
        &self,
        viewport: Size,
        input: DocumentInput,
        handler: impl for<'frame> FnMut(&'frame DocumentCommandAction<Action>),
    ) -> DocumentResult<(
        crate::DocumentActionFrame<Action>,
        DocumentCommandDispatchReport,
    )>
    where
        Self: Sized,
        Action: Clone,
    {
        Ok(self
            .try_action_surface(viewport)?
            .update_with_input_and_dispatch(input, handler))
    }

    /// Builds this widget with strict CSS, resolves it, collects actions, and dispatches them.
    fn update_and_dispatch_with_css(
        &self,
        viewport: Size,
        css: &str,
        handler: impl for<'frame> FnMut(&'frame DocumentCommandAction<Action>),
    ) -> Result<
        (
            crate::DocumentActionFrame<Action>,
            DocumentCommandDispatchReport,
        ),
        crate::CssParseError,
    >
    where
        Self: Sized,
        Action: Clone,
    {
        Ok(self
            .action_surface_with_css(viewport, css)?
            .update_and_dispatch(handler))
    }

    /// Builds this widget with strict CSS, resolves it, and dispatches only action values.
    fn update_and_dispatch_action_values_with_css(
        &self,
        viewport: Size,
        css: &str,
        handler: impl for<'frame> FnMut(&'frame Action),
    ) -> Result<
        (
            crate::DocumentActionFrame<Action>,
            DocumentCommandDispatchReport,
        ),
        crate::CssParseError,
    >
    where
        Self: Sized,
        Action: Clone,
    {
        Ok(self
            .action_surface_with_css(viewport, css)?
            .update_and_dispatch_action_values(handler))
    }

    /// Builds this widget with strict CSS, resolves it, and returns authoring errors explicitly.
    fn try_update_and_dispatch_with_css(
        &self,
        viewport: Size,
        css: &str,
        handler: impl for<'frame> FnMut(&'frame DocumentCommandAction<Action>),
    ) -> DocumentAuthoringResult<(
        crate::DocumentActionFrame<Action>,
        DocumentCommandDispatchReport,
    )>
    where
        Self: Sized,
        Action: Clone,
    {
        Ok(self
            .try_action_surface_with_css(viewport, css)?
            .update_and_dispatch(handler))
    }

    /// Builds this widget with strict CSS, resolves it, and dispatches action values explicitly.
    fn try_update_and_dispatch_action_values_with_css(
        &self,
        viewport: Size,
        css: &str,
        handler: impl for<'frame> FnMut(&'frame Action),
    ) -> DocumentAuthoringResult<(
        crate::DocumentActionFrame<Action>,
        DocumentCommandDispatchReport,
    )>
    where
        Self: Sized,
        Action: Clone,
    {
        Ok(self
            .try_action_surface_with_css(viewport, css)?
            .update_and_dispatch_action_values(handler))
    }

    /// Builds this widget with forgiving CSS, resolves it, collects actions, and dispatches them.
    fn update_and_dispatch_with_css_forgiving(
        &self,
        viewport: Size,
        css: &str,
        handler: impl for<'frame> FnMut(&'frame DocumentCommandAction<Action>),
    ) -> Result<
        (
            crate::DocumentActionFrame<Action>,
            DocumentCommandDispatchReport,
        ),
        crate::CssParseError,
    >
    where
        Self: Sized,
        Action: Clone,
    {
        Ok(self
            .action_surface_with_css_forgiving(viewport, css)?
            .update_and_dispatch(handler))
    }

    /// Builds this widget with forgiving CSS, resolves it, and dispatches only action values.
    fn update_and_dispatch_action_values_with_css_forgiving(
        &self,
        viewport: Size,
        css: &str,
        handler: impl for<'frame> FnMut(&'frame Action),
    ) -> Result<
        (
            crate::DocumentActionFrame<Action>,
            DocumentCommandDispatchReport,
        ),
        crate::CssParseError,
    >
    where
        Self: Sized,
        Action: Clone,
    {
        Ok(self
            .action_surface_with_css_forgiving(viewport, css)?
            .update_and_dispatch_action_values(handler))
    }

    /// Builds this widget with forgiving CSS, resolves it, and returns authoring errors explicitly.
    fn try_update_and_dispatch_with_css_forgiving(
        &self,
        viewport: Size,
        css: &str,
        handler: impl for<'frame> FnMut(&'frame DocumentCommandAction<Action>),
    ) -> DocumentAuthoringResult<(
        crate::DocumentActionFrame<Action>,
        DocumentCommandDispatchReport,
    )>
    where
        Self: Sized,
        Action: Clone,
    {
        Ok(self
            .try_action_surface_with_css_forgiving(viewport, css)?
            .update_and_dispatch(handler))
    }

    /// Builds this widget with forgiving CSS, resolves it, and dispatches action values explicitly.
    fn try_update_and_dispatch_action_values_with_css_forgiving(
        &self,
        viewport: Size,
        css: &str,
        handler: impl for<'frame> FnMut(&'frame Action),
    ) -> DocumentAuthoringResult<(
        crate::DocumentActionFrame<Action>,
        DocumentCommandDispatchReport,
    )>
    where
        Self: Sized,
        Action: Clone,
    {
        Ok(self
            .try_action_surface_with_css_forgiving(viewport, css)?
            .update_and_dispatch_action_values(handler))
    }

    /// Builds this widget with strict CSS, routes input, collects actions, and dispatches them.
    fn update_with_input_and_css_and_dispatch(
        &self,
        viewport: Size,
        input: DocumentInput,
        css: &str,
        handler: impl for<'frame> FnMut(&'frame DocumentCommandAction<Action>),
    ) -> Result<
        (
            crate::DocumentActionFrame<Action>,
            DocumentCommandDispatchReport,
        ),
        crate::CssParseError,
    >
    where
        Self: Sized,
        Action: Clone,
    {
        Ok(self
            .action_surface_with_css(viewport, css)?
            .update_with_input_and_dispatch(input, handler))
    }

    /// Builds this widget with strict CSS, routes input, and dispatches only action values.
    fn update_with_input_and_css_and_dispatch_action_values(
        &self,
        viewport: Size,
        input: DocumentInput,
        css: &str,
        handler: impl for<'frame> FnMut(&'frame Action),
    ) -> Result<
        (
            crate::DocumentActionFrame<Action>,
            DocumentCommandDispatchReport,
        ),
        crate::CssParseError,
    >
    where
        Self: Sized,
        Action: Clone,
    {
        Ok(self
            .action_surface_with_css(viewport, css)?
            .update_with_input_and_dispatch_action_values(input, handler))
    }

    /// Builds this widget with strict CSS, routes input, and returns authoring errors explicitly.
    fn try_update_with_input_and_css_and_dispatch(
        &self,
        viewport: Size,
        input: DocumentInput,
        css: &str,
        handler: impl for<'frame> FnMut(&'frame DocumentCommandAction<Action>),
    ) -> DocumentAuthoringResult<(
        crate::DocumentActionFrame<Action>,
        DocumentCommandDispatchReport,
    )>
    where
        Self: Sized,
        Action: Clone,
    {
        Ok(self
            .try_action_surface_with_css(viewport, css)?
            .update_with_input_and_dispatch(input, handler))
    }

    /// Builds this widget with strict CSS, routes input, and dispatches action values explicitly.
    fn try_update_with_input_and_css_and_dispatch_action_values(
        &self,
        viewport: Size,
        input: DocumentInput,
        css: &str,
        handler: impl for<'frame> FnMut(&'frame Action),
    ) -> DocumentAuthoringResult<(
        crate::DocumentActionFrame<Action>,
        DocumentCommandDispatchReport,
    )>
    where
        Self: Sized,
        Action: Clone,
    {
        Ok(self
            .try_action_surface_with_css(viewport, css)?
            .update_with_input_and_dispatch_action_values(input, handler))
    }

    /// Builds this widget with forgiving CSS, routes input, collects actions, and dispatches them.
    fn update_with_input_and_css_forgiving_and_dispatch(
        &self,
        viewport: Size,
        input: DocumentInput,
        css: &str,
        handler: impl for<'frame> FnMut(&'frame DocumentCommandAction<Action>),
    ) -> Result<
        (
            crate::DocumentActionFrame<Action>,
            DocumentCommandDispatchReport,
        ),
        crate::CssParseError,
    >
    where
        Self: Sized,
        Action: Clone,
    {
        Ok(self
            .action_surface_with_css_forgiving(viewport, css)?
            .update_with_input_and_dispatch(input, handler))
    }

    /// Builds this widget with forgiving CSS, routes input, and dispatches only action values.
    fn update_with_input_and_css_forgiving_and_dispatch_action_values(
        &self,
        viewport: Size,
        input: DocumentInput,
        css: &str,
        handler: impl for<'frame> FnMut(&'frame Action),
    ) -> Result<
        (
            crate::DocumentActionFrame<Action>,
            DocumentCommandDispatchReport,
        ),
        crate::CssParseError,
    >
    where
        Self: Sized,
        Action: Clone,
    {
        Ok(self
            .action_surface_with_css_forgiving(viewport, css)?
            .update_with_input_and_dispatch_action_values(input, handler))
    }

    /// Builds this widget with forgiving CSS, routes input, and returns authoring errors explicitly.
    fn try_update_with_input_and_css_forgiving_and_dispatch(
        &self,
        viewport: Size,
        input: DocumentInput,
        css: &str,
        handler: impl for<'frame> FnMut(&'frame DocumentCommandAction<Action>),
    ) -> DocumentAuthoringResult<(
        crate::DocumentActionFrame<Action>,
        DocumentCommandDispatchReport,
    )>
    where
        Self: Sized,
        Action: Clone,
    {
        Ok(self
            .try_action_surface_with_css_forgiving(viewport, css)?
            .update_with_input_and_dispatch(input, handler))
    }

    /// Builds this widget with forgiving CSS, routes input, and dispatches action values explicitly.
    fn try_update_with_input_and_css_forgiving_and_dispatch_action_values(
        &self,
        viewport: Size,
        input: DocumentInput,
        css: &str,
        handler: impl for<'frame> FnMut(&'frame Action),
    ) -> DocumentAuthoringResult<(
        crate::DocumentActionFrame<Action>,
        DocumentCommandDispatchReport,
    )>
    where
        Self: Sized,
        Action: Clone,
    {
        Ok(self
            .try_action_surface_with_css_forgiving(viewport, css)?
            .update_with_input_and_dispatch_action_values(input, handler))
    }

    /// Creates a ready-to-update action surface containing this widget.
    fn action_surface(&self, viewport: Size) -> crate::DocumentActionSurface<Action>
    where
        Self: Sized,
    {
        self.action_surface_with_stylesheet(viewport, StyleSheet::new())
    }

    /// Creates an action surface, returning projection errors explicitly.
    fn try_action_surface(
        &self,
        viewport: Size,
    ) -> DocumentResult<crate::DocumentActionSurface<Action>>
    where
        Self: Sized,
    {
        self.try_action_surface_with_stylesheet(viewport, StyleSheet::new())
    }

    /// Creates an action surface containing this widget and an external stylesheet.
    fn action_surface_with_stylesheet(
        &self,
        viewport: Size,
        stylesheet: StyleSheet,
    ) -> crate::DocumentActionSurface<Action>
    where
        Self: Sized,
    {
        crate::DocumentView::compose(viewport)
            .stylesheet(stylesheet)
            .action_widget(self)
    }

    /// Creates an action surface containing this widget and strict CSS stylesheet rules.
    fn action_surface_with_css(
        &self,
        viewport: Size,
        css: &str,
    ) -> Result<crate::DocumentActionSurface<Action>, crate::CssParseError>
    where
        Self: Sized,
    {
        let stylesheet = StyleSheet::from_css(css)?;
        Ok(self.action_surface_with_stylesheet(viewport, stylesheet))
    }

    /// Creates a CSS-backed action surface, returning all authoring errors explicitly.
    fn try_action_surface_with_css(
        &self,
        viewport: Size,
        css: &str,
    ) -> DocumentAuthoringResult<crate::DocumentActionSurface<Action>>
    where
        Self: Sized,
    {
        let stylesheet = StyleSheet::from_css(css)?;
        Ok(self.try_action_surface_with_stylesheet(viewport, stylesheet)?)
    }

    /// Creates an action surface containing this widget and browser-forgiving CSS rules.
    fn action_surface_with_css_forgiving(
        &self,
        viewport: Size,
        css: &str,
    ) -> Result<crate::DocumentActionSurface<Action>, crate::CssParseError>
    where
        Self: Sized,
    {
        let stylesheet = StyleSheet::from_css_forgiving(css)?;
        Ok(self.action_surface_with_stylesheet(viewport, stylesheet))
    }

    /// Creates a forgiving CSS-backed action surface, returning all authoring errors explicitly.
    fn try_action_surface_with_css_forgiving(
        &self,
        viewport: Size,
        css: &str,
    ) -> DocumentAuthoringResult<crate::DocumentActionSurface<Action>>
    where
        Self: Sized,
    {
        let stylesheet = StyleSheet::from_css_forgiving(css)?;
        Ok(self.try_action_surface_with_stylesheet(viewport, stylesheet)?)
    }

    /// Creates a styled action surface, returning projection errors explicitly.
    fn try_action_surface_with_stylesheet(
        &self,
        viewport: Size,
        stylesheet: StyleSheet,
    ) -> DocumentResult<crate::DocumentActionSurface<Action>>
    where
        Self: Sized,
    {
        crate::DocumentView::compose(viewport)
            .stylesheet(stylesheet)
            .try_action_widget(self)
    }
}

impl<Action> DocumentCommandRegistry<Action> {
    pub fn bind_widget(mut self, widget: &(impl DocumentActionWidget<Action> + ?Sized)) -> Self {
        self.push_widget_commands(widget);
        self
    }

    pub fn bind_widget_if(
        mut self,
        widget: &(impl DocumentActionWidget<Action> + ?Sized),
        present: bool,
    ) -> Self {
        self.push_widget_commands_if(widget, present);
        self
    }

    pub fn bind_widgets<'a, W>(mut self, widgets: impl IntoIterator<Item = &'a W>) -> Self
    where
        W: DocumentActionWidget<Action> + ?Sized + 'a,
    {
        self.push_widget_commands_many(widgets);
        self
    }

    pub fn bind_widgets_if<'a, W>(
        mut self,
        widgets: impl IntoIterator<Item = &'a W>,
        present: bool,
    ) -> Self
    where
        W: DocumentActionWidget<Action> + ?Sized + 'a,
    {
        self.push_widget_commands_many_if(widgets, present);
        self
    }

    pub fn push_widget_commands(&mut self, widget: &(impl DocumentActionWidget<Action> + ?Sized)) {
        widget.push_commands(self);
    }

    pub fn push_widget_commands_if(
        &mut self,
        widget: &(impl DocumentActionWidget<Action> + ?Sized),
        present: bool,
    ) {
        if present {
            self.push_widget_commands(widget);
        }
    }

    pub fn push_widget_commands_many<'a, W>(&mut self, widgets: impl IntoIterator<Item = &'a W>)
    where
        W: DocumentActionWidget<Action> + ?Sized + 'a,
    {
        for widget in widgets {
            self.push_widget_commands(widget);
        }
    }

    pub fn push_widget_commands_many_if<'a, W>(
        &mut self,
        widgets: impl IntoIterator<Item = &'a W>,
        present: bool,
    ) where
        W: DocumentActionWidget<Action> + ?Sized + 'a,
    {
        if present {
            self.push_widget_commands_many(widgets);
        }
    }
}

pub struct ElementBuilder<'a> {
    parent: &'a mut DocumentBuilder,
    id: ElementId,
    spec: ElementSpec,
}

macro_rules! element_builder_methods {
    ($($name:ident => $element:expr),+ $(,)?) => {
        $(
            pub fn $name(&mut self, id: impl Into<ElementId>) -> ElementBuilder<'_> {
                self.child(id, $element)
            }
        )+
    };
}

impl DocumentBuilder {
    pub fn child(&mut self, id: impl Into<ElementId>, element: Element) -> ElementBuilder<'_> {
        ElementBuilder {
            parent: self,
            id: id.into(),
            spec: ElementSpec::new(element),
        }
    }

    pub fn child_with(
        &mut self,
        id: impl Into<ElementId>,
        element: Element,
        build: impl FnOnce(ElementBuilder<'_>),
    ) -> &mut Self {
        build(self.child(id, element));
        self
    }

    pub fn child_with_if(
        &mut self,
        id: impl Into<ElementId>,
        element: Element,
        present: bool,
        build: impl FnOnce(ElementBuilder<'_>),
    ) -> &mut Self {
        if present {
            self.child_with(id, element, build);
        }
        self
    }

    pub fn when(&mut self, present: bool, build: impl FnOnce(&mut DocumentBuilder)) -> &mut Self {
        if present {
            build(self);
        }
        self
    }

    pub fn widget(&mut self, widget: &(impl DocumentWidget + ?Sized)) -> &mut Self {
        widget.render(self);
        self
    }

    pub fn widget_if(
        &mut self,
        widget: &(impl DocumentWidget + ?Sized),
        present: bool,
    ) -> &mut Self {
        if present {
            widget.render(self);
        }
        self
    }

    pub fn widgets<'a, W>(&mut self, widgets: impl IntoIterator<Item = &'a W>) -> &mut Self
    where
        W: DocumentWidget + ?Sized + 'a,
    {
        for widget in widgets {
            widget.render(self);
        }
        self
    }

    pub fn widgets_if<'a, W>(
        &mut self,
        widgets: impl IntoIterator<Item = &'a W>,
        present: bool,
    ) -> &mut Self
    where
        W: DocumentWidget + ?Sized + 'a,
    {
        if present {
            self.widgets(widgets);
        }
        self
    }

    /// Renders a collection of app items through one immediate-style builder hook.
    ///
    /// This is the structure-authoring companion to `DocumentProjection::items`:
    /// app code can render the retained elements for each item here, then project
    /// changing item state into those stable elements later.
    pub fn items<I>(
        &mut self,
        items: I,
        mut build: impl FnMut(&mut DocumentBuilder, I::Item),
    ) -> &mut Self
    where
        I: IntoIterator,
    {
        for item in items {
            build(self, item);
        }
        self
    }

    /// Conditionally renders a collection of app items.
    pub fn items_if<I>(
        &mut self,
        items: I,
        present: bool,
        build: impl FnMut(&mut DocumentBuilder, I::Item),
    ) -> &mut Self
    where
        I: IntoIterator,
    {
        if present {
            self.items(items, build);
        }
        self
    }

    element_builder_methods! {
        div => Element::Div,
        span => Element::Span,
        main => Element::Main,
        section => Element::Section,
        article => Element::Article,
        header => Element::Header,
        footer => Element::Footer,
        nav => Element::Nav,
        aside => Element::Aside,
        p => Element::P,
        h1 => Element::H1,
        h2 => Element::H2,
        h3 => Element::H3,
        h4 => Element::H4,
        h5 => Element::H5,
        h6 => Element::H6,
        button => Element::Button,
        input => Element::Input,
        checkbox => Element::Checkbox,
        radio => Element::Radio,
        select => Element::Select,
        option => Element::Option,
        textarea => Element::Textarea,
        label => Element::Label,
        canvas => Element::Canvas,
        icon => Element::Icon,
        table => Element::Table,
        thead => Element::Thead,
        tbody => Element::Tbody,
        tr => Element::Tr,
        th => Element::Th,
        td => Element::Td,
    }

    pub fn element(
        &mut self,
        id: impl Into<ElementId>,
        spec: ElementSpec,
        add_contents: impl FnOnce(&mut DocumentBuilder),
    ) -> &mut Self {
        let mut child_builder = DocumentBuilder::default();
        add_contents(&mut child_builder);
        self.children.push(DocumentNode {
            id: id.into(),
            spec,
            text: None,
            children: child_builder.children,
        });
        self
    }

    pub fn element_if(
        &mut self,
        id: impl Into<ElementId>,
        spec: ElementSpec,
        present: bool,
        add_contents: impl FnOnce(&mut DocumentBuilder),
    ) -> &mut Self {
        if present {
            self.element(id, spec, add_contents);
        }
        self
    }

    pub fn text(&mut self, id: impl Into<ElementId>, text: impl Into<TextContent>) -> &mut Self {
        self.text_element(id, ElementSpec::new(Element::Text), text)
    }

    pub fn text_if(
        &mut self,
        id: impl Into<ElementId>,
        text: impl Into<TextContent>,
        present: bool,
    ) -> &mut Self {
        if present {
            self.text(id, text);
        }
        self
    }

    pub fn text_node(&mut self, id: impl Into<ElementId>) -> ElementBuilder<'_> {
        self.child(id, Element::Text)
    }

    pub fn text_node_with(
        &mut self,
        id: impl Into<ElementId>,
        build: impl FnOnce(ElementBuilder<'_>),
    ) -> &mut Self {
        build(self.text_node(id));
        self
    }

    pub fn text_node_with_if(
        &mut self,
        id: impl Into<ElementId>,
        present: bool,
        build: impl FnOnce(ElementBuilder<'_>),
    ) -> &mut Self {
        if present {
            self.text_node_with(id, build);
        }
        self
    }

    pub fn text_element(
        &mut self,
        id: impl Into<ElementId>,
        spec: ElementSpec,
        text: impl Into<TextContent>,
    ) -> &mut Self {
        self.children.push(DocumentNode {
            id: id.into(),
            spec,
            text: Some(text.into()),
            children: Vec::new(),
        });
        self
    }

    pub fn text_element_if(
        &mut self,
        id: impl Into<ElementId>,
        spec: ElementSpec,
        text: impl Into<TextContent>,
        present: bool,
    ) -> &mut Self {
        if present {
            self.text_element(id, spec, text);
        }
        self
    }

    pub fn visual_clone(
        &mut self,
        clone: &VisualElementClone,
        options: VisualCloneOptions,
    ) -> &mut Self {
        self.children.push(clone.to_element(&options, true));
        self
    }

    pub fn visual_clone_if(
        &mut self,
        clone: &VisualElementClone,
        options: VisualCloneOptions,
        present: bool,
    ) -> &mut Self {
        if present {
            self.visual_clone(clone, options);
        }
        self
    }
}

impl<'a> ElementBuilder<'a> {
    pub fn class(mut self, class: impl Into<ClassName>) -> Self {
        self.spec = self.spec.class(class);
        self
    }

    pub fn classes<I, C>(mut self, classes: I) -> Self
    where
        I: IntoIterator<Item = C>,
        C: Into<ClassName>,
    {
        self.spec = self.spec.classes(classes);
        self
    }

    pub fn class_if(mut self, class: impl Into<ClassName>, present: bool) -> Self {
        self.spec = self.spec.class_if(class, present);
        self
    }

    pub fn classes_if<I, C>(mut self, classes: I, present: bool) -> Self
    where
        I: IntoIterator<Item = C>,
        C: Into<ClassName>,
    {
        self.spec = self.spec.classes_if(classes, present);
        self
    }

    pub fn role(mut self, role: impl Into<String>) -> Self {
        self.spec.role = Some(role.into());
        self
    }

    pub fn attribute(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.spec = self.spec.attribute(name, value);
        self
    }

    pub fn attribute_if(
        mut self,
        name: impl Into<String>,
        value: impl Into<String>,
        present: bool,
    ) -> Self {
        self.spec = self.spec.attribute_if(name, value, present);
        self
    }

    pub fn attributes<I, K, V>(mut self, attributes: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        self.spec = self.spec.attributes(attributes);
        self
    }

    pub fn data(mut self, name: impl AsRef<str>, value: impl Into<String>) -> Self {
        self.spec = self.spec.data(name, value);
        self
    }

    pub fn data_if(
        mut self,
        name: impl AsRef<str>,
        value: impl Into<String>,
        present: bool,
    ) -> Self {
        self.spec = self.spec.data_if(name, value, present);
        self
    }

    pub fn aria(mut self, name: impl AsRef<str>, value: impl Into<String>) -> Self {
        self.spec = self.spec.aria(name, value);
        self
    }

    pub fn aria_if(
        mut self,
        name: impl AsRef<str>,
        value: impl Into<String>,
        present: bool,
    ) -> Self {
        self.spec = self.spec.aria_if(name, value, present);
        self
    }

    pub fn behavior_hook(mut self, event: impl Into<String>, command: impl Into<String>) -> Self {
        self.spec = self.spec.behavior_hook(event, command);
        self
    }

    pub fn behavior_hooks<I, H>(mut self, hooks: I) -> Self
    where
        I: IntoIterator<Item = H>,
        H: Into<ElementBehaviorHook>,
    {
        self.spec = self.spec.behavior_hooks(hooks);
        self
    }

    pub fn on(mut self, event: ElementBehaviorEvent, command: impl Into<String>) -> Self {
        self.spec = self.spec.on(event, command);
        self
    }

    pub fn on_if(
        mut self,
        event: ElementBehaviorEvent,
        command: impl Into<String>,
        present: bool,
    ) -> Self {
        self.spec = self.spec.on_if(event, command, present);
        self
    }

    pub fn command(self, command: impl Into<String>) -> Self {
        self.on_click(command)
    }

    pub fn command_if(self, command: impl Into<String>, present: bool) -> Self {
        self.on_click_if(command, present)
    }

    pub fn command_on(self, event: ElementBehaviorEvent, command: impl Into<String>) -> Self {
        self.on(event, command)
    }

    pub fn command_on_if(
        self,
        event: ElementBehaviorEvent,
        command: impl Into<String>,
        present: bool,
    ) -> Self {
        self.on_if(event, command, present)
    }

    pub fn on_events<I, C>(mut self, events: I) -> Self
    where
        I: IntoIterator<Item = (ElementBehaviorEvent, C)>,
        C: Into<String>,
    {
        self.spec = self.spec.on_events(events);
        self
    }

    pub fn on_click(self, command: impl Into<String>) -> Self {
        self.on(ElementBehaviorEvent::Click, command)
    }

    pub fn on_click_if(self, command: impl Into<String>, present: bool) -> Self {
        self.on_if(ElementBehaviorEvent::Click, command, present)
    }

    pub fn on_context_menu(self, command: impl Into<String>) -> Self {
        self.on(ElementBehaviorEvent::ContextMenu, command)
    }

    pub fn on_context_menu_if(self, command: impl Into<String>, present: bool) -> Self {
        self.on_if(ElementBehaviorEvent::ContextMenu, command, present)
    }

    pub fn on_pointer_enter(self, command: impl Into<String>) -> Self {
        self.on(ElementBehaviorEvent::PointerEnter, command)
    }

    pub fn on_pointer_enter_if(self, command: impl Into<String>, present: bool) -> Self {
        self.on_if(ElementBehaviorEvent::PointerEnter, command, present)
    }

    pub fn on_pointer_leave(self, command: impl Into<String>) -> Self {
        self.on(ElementBehaviorEvent::PointerLeave, command)
    }

    pub fn on_pointer_leave_if(self, command: impl Into<String>, present: bool) -> Self {
        self.on_if(ElementBehaviorEvent::PointerLeave, command, present)
    }

    pub fn on_pointer_down(self, command: impl Into<String>) -> Self {
        self.on(ElementBehaviorEvent::PointerDown, command)
    }

    pub fn on_pointer_down_if(self, command: impl Into<String>, present: bool) -> Self {
        self.on_if(ElementBehaviorEvent::PointerDown, command, present)
    }

    pub fn on_pointer_up(self, command: impl Into<String>) -> Self {
        self.on(ElementBehaviorEvent::PointerUp, command)
    }

    pub fn on_pointer_up_if(self, command: impl Into<String>, present: bool) -> Self {
        self.on_if(ElementBehaviorEvent::PointerUp, command, present)
    }

    pub fn on_drag_start(self, command: impl Into<String>) -> Self {
        self.on(ElementBehaviorEvent::DragStart, command)
    }

    pub fn on_drag_start_if(self, command: impl Into<String>, present: bool) -> Self {
        self.on_if(ElementBehaviorEvent::DragStart, command, present)
    }

    pub fn on_drag(self, command: impl Into<String>) -> Self {
        self.on(ElementBehaviorEvent::Drag, command)
    }

    pub fn on_drag_if(self, command: impl Into<String>, present: bool) -> Self {
        self.on_if(ElementBehaviorEvent::Drag, command, present)
    }

    pub fn on_drag_end(self, command: impl Into<String>) -> Self {
        self.on(ElementBehaviorEvent::DragEnd, command)
    }

    pub fn on_drag_end_if(self, command: impl Into<String>, present: bool) -> Self {
        self.on_if(ElementBehaviorEvent::DragEnd, command, present)
    }

    pub fn on_scroll(self, command: impl Into<String>) -> Self {
        self.on(ElementBehaviorEvent::Scroll, command)
    }

    pub fn on_scroll_if(self, command: impl Into<String>, present: bool) -> Self {
        self.on_if(ElementBehaviorEvent::Scroll, command, present)
    }

    pub fn on_key_down(self, command: impl Into<String>) -> Self {
        self.on(ElementBehaviorEvent::KeyDown, command)
    }

    pub fn on_key_down_if(self, command: impl Into<String>, present: bool) -> Self {
        self.on_if(ElementBehaviorEvent::KeyDown, command, present)
    }

    pub fn on_key_up(self, command: impl Into<String>) -> Self {
        self.on(ElementBehaviorEvent::KeyUp, command)
    }

    pub fn on_key_up_if(self, command: impl Into<String>, present: bool) -> Self {
        self.on_if(ElementBehaviorEvent::KeyUp, command, present)
    }

    pub fn interactive(mut self) -> Self {
        self.spec.interactive = true;
        self
    }

    pub fn interactive_if(mut self, interactive: bool) -> Self {
        self.spec = self.spec.interactive_if(interactive);
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.spec.selected = selected;
        self
    }

    pub fn checked(mut self, checked: bool) -> Self {
        self.spec = self.spec.checked(checked);
        self
    }

    pub fn check(mut self) -> Self {
        self.spec = self.spec.check();
        self
    }

    pub fn check_if(mut self, present: bool) -> Self {
        self.spec = self.spec.check_if(present);
        self
    }

    pub fn uncheck(mut self) -> Self {
        self.spec = self.spec.uncheck();
        self
    }

    pub fn uncheck_if(mut self, present: bool) -> Self {
        self.spec = self.spec.uncheck_if(present);
        self
    }

    pub fn checked_if(mut self, checked: bool, present: bool) -> Self {
        self.spec = self.spec.checked_if(checked, present);
        self
    }

    pub fn select(mut self) -> Self {
        self.spec.selected = true;
        self
    }

    pub fn deselect(mut self) -> Self {
        self.spec = self.spec.deselect();
        self
    }

    pub fn select_if(mut self, present: bool) -> Self {
        self.spec = self.spec.selected_if(present);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.spec.disabled = disabled;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.spec = self.spec.enabled(enabled);
        self
    }

    pub fn enabled_if(mut self, enabled: bool, present: bool) -> Self {
        self.spec = self.spec.enabled_if(enabled, present);
        self
    }

    pub fn disabled_if(mut self, disabled: bool, present: bool) -> Self {
        self.spec = self.spec.disabled_if(disabled, present);
        self
    }

    pub fn disable(mut self) -> Self {
        self.spec = self.spec.disable();
        self
    }

    pub fn enable(mut self) -> Self {
        self.spec = self.spec.enable();
        self
    }

    pub fn disable_if(mut self, present: bool) -> Self {
        self.spec = self.spec.disable_if(present);
        self
    }

    pub fn enable_if(mut self, present: bool) -> Self {
        self.spec = self.spec.enable_if(present);
        self
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.spec.focused = focused;
        self
    }

    pub fn focus(mut self) -> Self {
        self.spec = self.spec.focus();
        self
    }

    pub fn blur(mut self) -> Self {
        self.spec = self.spec.blur();
        self
    }

    pub fn focus_if(mut self, present: bool) -> Self {
        self.spec = self.spec.focus_if(present);
        self
    }

    pub fn blur_if(mut self, present: bool) -> Self {
        self.spec = self.spec.blur_if(present);
        self
    }

    pub fn selectable_text(mut self) -> Self {
        self.spec.selectable_text = true;
        self.spec.copyable_text = true;
        self
    }

    pub fn selectable_text_if(mut self, present: bool) -> Self {
        self.spec = self.spec.selectable_text_if(present);
        self
    }

    pub fn copyable_text(mut self, copyable_text: bool) -> Self {
        self.spec.copyable_text = copyable_text;
        self
    }

    pub fn copyable_text_if(mut self, copyable_text: bool, present: bool) -> Self {
        self.spec = self.spec.copyable_text_if(copyable_text, present);
        self
    }

    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.spec.value = Some(value.into());
        self
    }

    pub fn value_if(mut self, value: impl Into<String>, present: bool) -> Self {
        self.spec = self.spec.value_if(value, present);
        self
    }

    pub fn glyph(mut self, glyph: Glyph) -> Self {
        self.spec.glyph = Some(glyph);
        self
    }

    pub fn table(mut self, table: TableSpec) -> Self {
        self.spec.table = Some(table);
        self
    }

    pub fn table_cell(mut self, table_cell: crate::table::TableCellSpec) -> Self {
        self.spec.table_cell = Some(table_cell);
        self
    }

    pub fn empty(self) -> &'a mut DocumentBuilder {
        self.push(None, Vec::new())
    }

    pub fn text(self, text: impl Into<TextContent>) -> &'a mut DocumentBuilder {
        self.push(Some(text.into()), Vec::new())
    }

    pub fn children(
        self,
        add_contents: impl FnOnce(&mut DocumentBuilder),
    ) -> &'a mut DocumentBuilder {
        let mut child_builder = DocumentBuilder::default();
        add_contents(&mut child_builder);
        self.push(None, child_builder.children)
    }

    fn push(
        self,
        text: Option<TextContent>,
        children: Vec<DocumentNode>,
    ) -> &'a mut DocumentBuilder {
        self.parent.children.push(DocumentNode {
            id: self.id,
            spec: self.spec,
            text,
            children,
        });
        self.parent
    }
}

struct DocumentMeasureInput {
    text: TextContent,
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

fn prefixed_attribute_name(prefix: &str, name: impl AsRef<str>) -> String {
    let name = name.as_ref();
    if name.starts_with(prefix) {
        name.to_owned()
    } else {
        format!("{prefix}{name}")
    }
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

fn child_parent_origin(
    parent_rect: DocumentRect,
    parent_style: &ComputedStyle,
    child_style: &ComputedStyle,
) -> Point {
    if child_style.position == Position::AbsoluteParent && child_style.anchor.is_none() {
        return Point::new(
            parent_rect.origin.x + parent_style.padding.left,
            parent_rect.origin.y + parent_style.padding.top,
        );
    }

    parent_rect.origin
}

fn resolved_document_rect(
    raw_rect: DocumentRect,
    style: &ComputedStyle,
    viewport: Size,
    parent_origin: Point,
    parent_scroll_offset: Point,
    anchors: &HashMap<ElementId, DocumentRect>,
    boundaries: &HashMap<ElementId, DocumentRect>,
) -> (DocumentRect, Option<ResolvedFloating>) {
    if let Some(anchor) = &style.anchor
        && let Some(anchor_rect) = anchors.get(&anchor.target)
    {
        return anchored_document_rect(style, *anchor_rect, raw_rect.size, viewport, boundaries);
    }

    if style.position != Position::AbsoluteViewport {
        return (
            DocumentRect::new(
                parent_origin.x + raw_rect.origin.x - parent_scroll_offset.x,
                parent_origin.y + raw_rect.origin.y - parent_scroll_offset.y,
                raw_rect.size.width,
                raw_rect.size.height,
            ),
            None,
        );
    }

    (
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
        ),
        None,
    )
}

fn anchored_document_rect(
    style: &ComputedStyle,
    anchor_rect: DocumentRect,
    measured: Size,
    viewport: Size,
    boundaries: &HashMap<ElementId, DocumentRect>,
) -> (DocumentRect, Option<ResolvedFloating>) {
    let anchor = style
        .anchor
        .as_ref()
        .expect("anchored document rect requires an anchor style");
    let arrow_size = anchor
        .options
        .arrow
        .map(|arrow| Size::new(arrow.size.width, arrow.size.height));
    let mut options = anchor.options.clone();
    options.boundary = anchor
        .boundary_target
        .as_ref()
        .and_then(|target| boundaries.get(target).copied())
        .map(floating_boundary_from_document_rect)
        .or(options.boundary)
        .or_else(|| {
            Some(FloatingBoundary::new(FloatingRect::new(
                LayoutPoint { x: 0.0, y: 0.0 },
                FloatingSize {
                    width: viewport.width,
                    height: viewport.height,
                },
            )))
        });
    let floating = compute_floating_position(
        FloatingRect::new(
            LayoutPoint {
                x: anchor_rect.origin.x,
                y: anchor_rect.origin.y,
            },
            FloatingSize {
                width: anchor_rect.size.width,
                height: anchor_rect.size.height,
            },
        ),
        FloatingSize {
            width: measured.width,
            height: measured.height,
        },
        options,
    );

    (
        DocumentRect::new(
            floating.origin.x + style.margin.left,
            floating.origin.y + style.margin.top,
            floating.size.width,
            floating.size.height,
        ),
        Some(ResolvedFloating {
            placement: floating.placement,
            arrow_offset: floating
                .arrow_offset
                .map(|offset| Point::new(offset.x, offset.y)),
            arrow_center_offset: floating.arrow.map(|arrow| arrow.center_offset),
            arrow_size,
            available_size: Size::new(
                floating.available_size.width,
                floating.available_size.height,
            ),
            hide: floating.hide,
            visibility: floating.visibility,
        }),
    )
}

fn floating_boundary_from_document_rect(rect: DocumentRect) -> FloatingBoundary {
    FloatingBoundary::new(FloatingRect::new(
        LayoutPoint {
            x: rect.origin.x,
            y: rect.origin.y,
        },
        FloatingSize {
            width: rect.size.width,
            height: rect.size.height,
        },
    ))
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

fn measure_text_content(
    text: &TextContent,
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

fn measure_normalized_text(
    text: &NormalizedText,
    style: &ComputedStyle,
    available_width: f32,
    text_measurer: &mut dyn TextMeasurer,
) -> crate::text::TextLayoutResult {
    let wrap_width = match style.text_layout.text_wrap_mode {
        crate::text::TextWrapMode::NoWrap => f32::INFINITY,
        crate::text::TextWrapMode::Wrap => available_width,
    };
    text_measurer.measure_text(TextLayoutRequest {
        text,
        font_size: style.font_size,
        color: style.text_color,
        direction: style.direction,
        wrap_width,
        layout_style: style.text_layout,
        line_height: style.line_height,
    })
}

fn measure_text_with_wrap_width(
    text: &TextContent,
    style: &ComputedStyle,
    available_width: f32,
    text_measurer: &mut dyn TextMeasurer,
) -> crate::text::TextLayoutResult {
    let normalized = NormalizedText::from_content(text, style.text_layout);
    measure_normalized_text(&normalized, style, available_width, text_measurer)
}

impl Document {
    fn layout_style_from_computed(&mut self, style: &ComputedStyle) -> LayoutStyle {
        LayoutStyle {
            display: style.display,
            direction: style.direction,
            overflow: layout_overflow(style.overflow_x, style.overflow_y),
            scrollbar_width: style.scrollbar_width,
            position: layout_position(style.position),
            inset: LayoutRect {
                left: style
                    .inset
                    .left
                    .map_or_else(LengthPercentageAuto::auto, |length| {
                        self.length_percentage_auto_from_document(length)
                    }),
                right: style
                    .inset
                    .right
                    .map_or_else(LengthPercentageAuto::auto, |length| {
                        self.length_percentage_auto_from_document(length)
                    }),
                top: style
                    .inset
                    .top
                    .map_or_else(LengthPercentageAuto::auto, |length| {
                        self.length_percentage_auto_from_document(length)
                    }),
                bottom: style
                    .inset
                    .bottom
                    .map_or_else(LengthPercentageAuto::auto, |length| {
                        self.length_percentage_auto_from_document(length)
                    }),
            },
            size: LayoutSize {
                width: self.dimension_from_document(style.width),
                height: self.dimension_from_document(style.height),
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
            align_self: style.align_self.map(layout_align_items),
            align_content: Some(layout_align_content(style.align_content)),
            justify_items: style.justify_items.map(layout_align_items),
            justify_self: style.justify_self.map(layout_align_items),
            justify_content: Some(layout_justify_content(style.justify_content)),
            gap: LayoutSize {
                width: self.length_percentage_from_document(style.column_gap),
                height: self.length_percentage_from_document(style.row_gap),
            },
            flex_direction: layout_flex_direction(style.flex_direction),
            flex_wrap: layout_flex_wrap(style.flex_wrap),
            flex_basis: self.dimension_from_document(style.flex_basis),
            flex_grow: style.flex_grow,
            flex_shrink: style.flex_shrink,
            grid_template_rows: style.grid_template_rows.clone(),
            grid_template_columns: style.grid_template_columns.clone(),
            grid_auto_rows: style.grid_auto_rows.clone(),
            grid_auto_columns: style.grid_auto_columns.clone(),
            grid_auto_flow: style.grid_auto_flow,
            grid_template_areas: style.grid_template_areas.clone(),
            grid_template_column_names: style.grid_template_column_names.clone(),
            grid_template_row_names: style.grid_template_row_names.clone(),
            grid_row: style.grid_row.clone(),
            grid_column: style.grid_column.clone(),
            ..Default::default()
        }
    }

    fn intern_calc_length(&mut self, percent: f32, px: f32) -> *const () {
        let key = CalcLengthKey::new(percent, px);
        let value = self
            .calc_lengths
            .entry(key)
            .or_insert_with(|| Box::new(LayoutCalcLength { percent, px }));
        (&**value as *const LayoutCalcLength).cast::<()>()
    }

    fn dimension_from_document(&mut self, length_value: Length) -> Dimension {
        match length_value {
            Length::Auto => Dimension::auto(),
            Length::Px(value) => length(value),
            Length::Fill => percent(1.0),
            Length::Percent(value) => percent(value),
            Length::Calc { percent, px } => Dimension::calc(self.intern_calc_length(percent, px)),
        }
    }

    fn length_percentage_from_document(&mut self, length_value: Length) -> LengthPercentage {
        match length_value {
            Length::Auto => length(0.0),
            Length::Px(value) => length(value),
            Length::Fill => percent(1.0),
            Length::Percent(value) => percent(value),
            Length::Calc { percent, px } => {
                LengthPercentage::calc(self.intern_calc_length(percent, px))
            }
        }
    }

    fn length_percentage_auto_from_document(
        &mut self,
        length_value: Length,
    ) -> LengthPercentageAuto {
        match length_value {
            Length::Auto => LengthPercentageAuto::auto(),
            Length::Px(value) => length(value),
            Length::Fill => LengthPercentageAuto::auto(),
            Length::Percent(value) => percent(value),
            Length::Calc { percent, px } => {
                LengthPercentageAuto::calc(self.intern_calc_length(percent, px))
            }
        }
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

fn layout_overflow(x: Overflow, y: Overflow) -> des_layout::geometry::Point<LayoutOverflow> {
    des_layout::geometry::Point {
        x: match x {
            Overflow::Visible => LayoutOverflow::Visible,
            Overflow::Clip => LayoutOverflow::Clip,
            Overflow::Hidden => LayoutOverflow::Hidden,
            Overflow::Auto => LayoutOverflow::Hidden,
            Overflow::Scroll => LayoutOverflow::Scroll,
        },
        y: match y {
            Overflow::Visible => LayoutOverflow::Visible,
            Overflow::Clip => LayoutOverflow::Clip,
            Overflow::Hidden => LayoutOverflow::Hidden,
            Overflow::Auto => LayoutOverflow::Hidden,
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
        AlignItems::FlexStart => LayoutAlignItems::FlexStart,
        AlignItems::Center => LayoutAlignItems::Center,
        AlignItems::FlexEnd => LayoutAlignItems::FlexEnd,
        AlignItems::End => LayoutAlignItems::End,
        AlignItems::Baseline => LayoutAlignItems::Baseline,
        AlignItems::Stretch => LayoutAlignItems::Stretch,
    }
}

fn layout_align_content(align_content: AlignContent) -> LayoutAlignContent {
    match align_content {
        AlignContent::Start => LayoutAlignContent::Start,
        AlignContent::FlexStart => LayoutAlignContent::FlexStart,
        AlignContent::Center => LayoutAlignContent::Center,
        AlignContent::FlexEnd => LayoutAlignContent::FlexEnd,
        AlignContent::End => LayoutAlignContent::End,
        AlignContent::Stretch => LayoutAlignContent::Stretch,
        AlignContent::SpaceBetween => LayoutAlignContent::SpaceBetween,
        AlignContent::SpaceEvenly => LayoutAlignContent::SpaceEvenly,
        AlignContent::SpaceAround => LayoutAlignContent::SpaceAround,
    }
}

fn layout_justify_content(justify_content: JustifyContent) -> LayoutJustifyContent {
    match justify_content {
        JustifyContent::Start => LayoutJustifyContent::Start,
        JustifyContent::FlexStart => LayoutJustifyContent::FlexStart,
        JustifyContent::Center => LayoutJustifyContent::Center,
        JustifyContent::FlexEnd => LayoutJustifyContent::FlexEnd,
        JustifyContent::End => LayoutJustifyContent::End,
        JustifyContent::Stretch => LayoutJustifyContent::Stretch,
        JustifyContent::SpaceBetween => LayoutJustifyContent::SpaceBetween,
        JustifyContent::SpaceEvenly => LayoutJustifyContent::SpaceEvenly,
        JustifyContent::SpaceAround => LayoutJustifyContent::SpaceAround,
    }
}

fn layout_flex_direction(flex_direction: FlexDirection) -> LayoutFlexDirection {
    match flex_direction {
        FlexDirection::Row => LayoutFlexDirection::Row,
        FlexDirection::Column => LayoutFlexDirection::Column,
        FlexDirection::RowReverse => LayoutFlexDirection::RowReverse,
        FlexDirection::ColumnReverse => LayoutFlexDirection::ColumnReverse,
    }
}

fn layout_flex_wrap(flex_wrap: FlexWrap) -> LayoutFlexWrap {
    match flex_wrap {
        FlexWrap::NoWrap => LayoutFlexWrap::NoWrap,
        FlexWrap::Wrap => LayoutFlexWrap::Wrap,
        FlexWrap::WrapReverse => LayoutFlexWrap::WrapReverse,
    }
}

fn layout_error(error: des_layout::LayoutError) -> DocumentError {
    DocumentError::Layout(error.to_string())
}
