use crate::element::{ClassName, ElementId, ElementRole};
use crate::geometry::{Point, Rect};
use crate::layout::hit_path;
use crate::state::ResolvedElement;

#[derive(Clone, Copy, Debug)]
pub struct DocumentSnapshot<'a> {
    root: &'a ResolvedElement,
}

impl<'a> DocumentSnapshot<'a> {
    pub fn new(root: &'a ResolvedElement) -> Self {
        Self { root }
    }

    pub fn root(&self) -> ElementSnapshot<'a> {
        ElementSnapshot { element: self.root }
    }

    pub fn find(&self, id: &str) -> Option<ElementSnapshot<'a>> {
        find_element(self.root, id).map(|element| ElementSnapshot { element })
    }

    pub fn hit_test(&self, point: Point) -> Option<HitResult<'a>> {
        let path = self.path_at(point)?;
        path.last().copied().map(|target| HitResult {
            target,
            path,
            point,
        })
    }

    pub fn path_at(&self, point: Point) -> Option<Vec<ElementSnapshot<'a>>> {
        hit_path(self.root, point).map(|path| {
            path.into_iter()
                .map(|element| ElementSnapshot { element })
                .collect()
        })
    }

    pub fn elements_with_class(&self, class: impl Into<ClassName>) -> Vec<ElementSnapshot<'a>> {
        let class = class.into();
        let mut elements = Vec::new();
        collect_elements(self.root, &mut elements, &mut |element| {
            element
                .classes
                .iter()
                .any(|element_class| element_class == &class)
        });
        elements
    }

    pub fn elements_by_role(&self, role: ElementRole) -> Vec<ElementSnapshot<'a>> {
        let mut elements = Vec::new();
        collect_elements(self.root, &mut elements, &mut |element| {
            element.role == role
        });
        elements
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ElementSnapshot<'a> {
    element: &'a ResolvedElement,
}

impl<'a> ElementSnapshot<'a> {
    pub fn id(&self) -> &ElementId {
        &self.element.id
    }

    pub fn role(&self) -> ElementRole {
        self.element.role
    }

    pub fn classes(&self) -> &[ClassName] {
        &self.element.classes
    }

    pub fn has_class(&self, class: &str) -> bool {
        self.element
            .classes
            .iter()
            .any(|element_class| element_class.as_str() == class)
    }

    pub fn rect(&self) -> Rect {
        self.element.rect
    }

    pub fn text(&self) -> Option<&str> {
        self.element.text.as_deref()
    }

    pub fn text_layout(&self) -> Option<crate::TextLayoutResult> {
        self.element.text_layout
    }

    pub fn selectable_text(&self) -> bool {
        self.element.selectable_text
    }

    pub fn copyable_text(&self) -> bool {
        self.element.copyable_text
    }

    pub fn style(&self) -> &crate::ComputedStyle {
        &self.element.style
    }

    pub fn value(&self) -> Option<&str> {
        self.element.value.as_deref()
    }

    pub fn interactive(&self) -> bool {
        self.element.interactive
    }
}

#[derive(Clone, Debug)]
pub struct HitResult<'a> {
    pub target: ElementSnapshot<'a>,
    pub path: Vec<ElementSnapshot<'a>>,
    pub point: Point,
}

fn find_element<'a>(element: &'a ResolvedElement, id: &str) -> Option<&'a ResolvedElement> {
    if element.id.as_str() == id {
        return Some(element);
    }
    element
        .children
        .iter()
        .find_map(|child| find_element(child, id))
}

fn collect_elements<'a>(
    element: &'a ResolvedElement,
    elements: &mut Vec<ElementSnapshot<'a>>,
    predicate: &mut impl FnMut(&ResolvedElement) -> bool,
) {
    if predicate(element) {
        elements.push(ElementSnapshot { element });
    }
    for child in &element.children {
        collect_elements(child, elements, predicate);
    }
}
