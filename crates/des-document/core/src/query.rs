use crate::element::{ClassName, Element, ElementId, VisualElementClone};
use crate::geometry::{ClipRect, Point, Rect};
use crate::layout::hit_path;
use crate::state::ResolvedElement;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentQueryError {
    id: String,
}

impl DocumentQueryError {
    pub fn missing(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }

    pub fn id(&self) -> &str {
        &self.id
    }
}

impl std::fmt::Display for DocumentQueryError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "document element '{}' was not found", self.id)
    }
}

impl std::error::Error for DocumentQueryError {}

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

    pub fn require(&self, id: &str) -> Result<ElementSnapshot<'a>, DocumentQueryError> {
        self.find(id).ok_or_else(|| DocumentQueryError::missing(id))
    }

    pub fn contains(&self, id: &str) -> bool {
        self.find(id).is_some()
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

    pub fn contains_class(&self, class: impl Into<ClassName>) -> bool {
        let class = class.into();
        find_matching_element(self.root, &mut |element| {
            element
                .classes
                .iter()
                .any(|element_class| element_class == &class)
        })
        .is_some()
    }

    pub fn first_with_class(&self, class: impl Into<ClassName>) -> Option<ElementSnapshot<'a>> {
        let class = class.into();
        find_matching_element(self.root, &mut |element| {
            element
                .classes
                .iter()
                .any(|element_class| element_class == &class)
        })
        .map(|element| ElementSnapshot { element })
    }

    pub fn count_with_class(&self, class: impl Into<ClassName>) -> usize {
        self.elements_with_class(class).len()
    }

    pub fn elements_with_attribute(
        &self,
        name: impl AsRef<str>,
        value: impl AsRef<str>,
    ) -> Vec<ElementSnapshot<'a>> {
        let name = name.as_ref();
        let value = value.as_ref();
        let mut elements = Vec::new();
        collect_elements(self.root, &mut elements, &mut |element| {
            element
                .attributes
                .get(name)
                .is_some_and(|found| found == value)
        });
        elements
    }

    pub fn contains_attribute(&self, name: impl AsRef<str>, value: impl AsRef<str>) -> bool {
        let name = name.as_ref();
        let value = value.as_ref();
        find_matching_element(self.root, &mut |element| {
            element
                .attributes
                .get(name)
                .is_some_and(|found| found == value)
        })
        .is_some()
    }

    pub fn first_with_attribute(
        &self,
        name: impl AsRef<str>,
        value: impl AsRef<str>,
    ) -> Option<ElementSnapshot<'a>> {
        let name = name.as_ref();
        let value = value.as_ref();
        find_matching_element(self.root, &mut |element| {
            element
                .attributes
                .get(name)
                .is_some_and(|found| found == value)
        })
        .map(|element| ElementSnapshot { element })
    }

    pub fn count_with_attribute(&self, name: impl AsRef<str>, value: impl AsRef<str>) -> usize {
        self.elements_with_attribute(name, value).len()
    }

    pub fn elements_with_data(
        &self,
        name: impl AsRef<str>,
        value: impl AsRef<str>,
    ) -> Vec<ElementSnapshot<'a>> {
        self.elements_with_attribute(prefixed_attribute_name("data-", name.as_ref()), value)
    }

    pub fn contains_data(&self, name: impl AsRef<str>, value: impl AsRef<str>) -> bool {
        self.contains_attribute(prefixed_attribute_name("data-", name.as_ref()), value)
    }

    pub fn first_with_data(
        &self,
        name: impl AsRef<str>,
        value: impl AsRef<str>,
    ) -> Option<ElementSnapshot<'a>> {
        self.first_with_attribute(prefixed_attribute_name("data-", name.as_ref()), value)
    }

    pub fn count_with_data(&self, name: impl AsRef<str>, value: impl AsRef<str>) -> usize {
        self.count_with_attribute(prefixed_attribute_name("data-", name.as_ref()), value)
    }

    pub fn elements_with_aria(
        &self,
        name: impl AsRef<str>,
        value: impl AsRef<str>,
    ) -> Vec<ElementSnapshot<'a>> {
        self.elements_with_attribute(prefixed_attribute_name("aria-", name.as_ref()), value)
    }

    pub fn contains_aria(&self, name: impl AsRef<str>, value: impl AsRef<str>) -> bool {
        self.contains_attribute(prefixed_attribute_name("aria-", name.as_ref()), value)
    }

    pub fn first_with_aria(
        &self,
        name: impl AsRef<str>,
        value: impl AsRef<str>,
    ) -> Option<ElementSnapshot<'a>> {
        self.first_with_attribute(prefixed_attribute_name("aria-", name.as_ref()), value)
    }

    pub fn count_with_aria(&self, name: impl AsRef<str>, value: impl AsRef<str>) -> usize {
        self.count_with_attribute(prefixed_attribute_name("aria-", name.as_ref()), value)
    }

    pub fn elements_by_element(&self, target: Element) -> Vec<ElementSnapshot<'a>> {
        let mut elements = Vec::new();
        collect_elements(self.root, &mut elements, &mut |element| {
            element.element == target
        });
        elements
    }

    pub fn contains_element(&self, target: Element) -> bool {
        find_matching_element(self.root, &mut |element| element.element == target).is_some()
    }

    pub fn first_by_element(&self, target: Element) -> Option<ElementSnapshot<'a>> {
        find_matching_element(self.root, &mut |element| element.element == target)
            .map(|element| ElementSnapshot { element })
    }

    pub fn count_by_element(&self, target: Element) -> usize {
        self.elements_by_element(target).len()
    }

    pub fn selected_elements(&self) -> Vec<ElementSnapshot<'a>> {
        self.elements_matching(|element| element.selected)
    }

    pub fn first_selected(&self) -> Option<ElementSnapshot<'a>> {
        self.first_matching(|element| element.selected)
    }

    pub fn contains_selected(&self) -> bool {
        self.first_selected().is_some()
    }

    pub fn count_selected(&self) -> usize {
        self.selected_elements().len()
    }

    pub fn disabled_elements(&self) -> Vec<ElementSnapshot<'a>> {
        self.elements_matching(|element| element.disabled)
    }

    pub fn first_disabled(&self) -> Option<ElementSnapshot<'a>> {
        self.first_matching(|element| element.disabled)
    }

    pub fn contains_disabled(&self) -> bool {
        self.first_disabled().is_some()
    }

    pub fn count_disabled(&self) -> usize {
        self.disabled_elements().len()
    }

    pub fn focused_elements(&self) -> Vec<ElementSnapshot<'a>> {
        self.elements_matching(|element| element.focused)
    }

    pub fn first_focused(&self) -> Option<ElementSnapshot<'a>> {
        self.first_matching(|element| element.focused)
    }

    pub fn contains_focused(&self) -> bool {
        self.first_focused().is_some()
    }

    pub fn count_focused(&self) -> usize {
        self.focused_elements().len()
    }

    pub fn interactive_elements(&self) -> Vec<ElementSnapshot<'a>> {
        self.elements_matching(|element| element.interactive)
    }

    pub fn first_interactive(&self) -> Option<ElementSnapshot<'a>> {
        self.first_matching(|element| element.interactive)
    }

    pub fn contains_interactive(&self) -> bool {
        self.first_interactive().is_some()
    }

    pub fn count_interactive(&self) -> usize {
        self.interactive_elements().len()
    }

    fn elements_matching(
        &self,
        mut predicate: impl FnMut(&ResolvedElement) -> bool,
    ) -> Vec<ElementSnapshot<'a>> {
        let mut elements = Vec::new();
        collect_elements(self.root, &mut elements, &mut predicate);
        elements
    }

    fn first_matching(
        &self,
        mut predicate: impl FnMut(&ResolvedElement) -> bool,
    ) -> Option<ElementSnapshot<'a>> {
        find_matching_element(self.root, &mut predicate).map(|element| ElementSnapshot { element })
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

    pub fn element(&self) -> Element {
        self.element.element
    }

    pub fn is_element(&self, element: Element) -> bool {
        self.element() == element
    }

    pub fn classes(&self) -> &[ClassName] {
        &self.element.classes
    }

    pub fn role(&self) -> Option<&str> {
        self.element.role.as_deref()
    }

    pub fn attributes(&self) -> &std::collections::BTreeMap<String, String> {
        &self.element.attributes
    }

    pub fn attribute(&self, name: &str) -> Option<&str> {
        self.element.attributes.get(name).map(String::as_str)
    }

    pub fn data(&self, name: &str) -> Option<&str> {
        self.attribute(&prefixed_attribute_name("data-", name))
    }

    pub fn aria(&self, name: &str) -> Option<&str> {
        self.attribute(&prefixed_attribute_name("aria-", name))
    }

    pub fn has_attribute(&self, name: &str, value: &str) -> bool {
        self.attribute(name) == Some(value)
    }

    pub fn has_data(&self, name: &str, value: &str) -> bool {
        self.data(name) == Some(value)
    }

    pub fn has_aria(&self, name: &str, value: &str) -> bool {
        self.aria(name) == Some(value)
    }

    pub fn behavior_hooks(&self) -> &[crate::ElementBehaviorHook] {
        &self.element.behavior_hooks
    }

    pub fn behavior_hooks_for(
        &self,
        event: crate::ElementBehaviorEvent,
    ) -> impl Iterator<Item = &crate::ElementBehaviorHook> {
        self.element
            .behavior_hooks
            .iter()
            .filter(move |hook| crate::ElementBehaviorEvent::from_name(&hook.event) == Some(event))
    }

    pub fn first_behavior_hook_for(
        &self,
        event: crate::ElementBehaviorEvent,
    ) -> Option<&crate::ElementBehaviorHook> {
        self.behavior_hooks_for(event).next()
    }

    pub fn has_behavior_hook(&self, event: crate::ElementBehaviorEvent, command: &str) -> bool {
        self.behavior_hooks_for(event)
            .any(|hook| hook.command == command)
    }

    pub fn has_command_hook(&self, command: &str) -> bool {
        self.element
            .behavior_hooks
            .iter()
            .any(|hook| hook.command == command)
    }

    pub fn has_class(&self, class: &str) -> bool {
        self.element
            .classes
            .iter()
            .any(|element_class| element_class.as_str() == class)
    }

    pub fn id_is(&self, id: &str) -> bool {
        self.id().as_str() == id
    }

    pub fn has_all_classes<'b>(&self, classes: impl IntoIterator<Item = &'b str>) -> bool {
        classes.into_iter().all(|class| self.has_class(class))
    }

    pub fn has_any_class<'b>(&self, classes: impl IntoIterator<Item = &'b str>) -> bool {
        classes.into_iter().any(|class| self.has_class(class))
    }

    pub fn rect(&self) -> Rect {
        self.element.rect
    }

    pub fn clip_rect(&self) -> ClipRect {
        self.element.clip_rect
    }

    pub fn text(&self) -> Option<String> {
        self.element
            .text
            .as_ref()
            .map(|text| text.semantic_text().to_owned())
    }

    pub fn text_content(&self) -> Option<&crate::TextContent> {
        self.element.text.as_ref()
    }

    pub fn text_layout(&self) -> Option<crate::TextLayoutResult> {
        self.element.text_layout.clone()
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

    pub fn selected(&self) -> bool {
        self.element.selected
    }

    pub fn disabled(&self) -> bool {
        self.element.disabled
    }

    pub fn focused(&self) -> bool {
        self.element.focused
    }

    pub fn floating(&self) -> Option<crate::ResolvedFloating> {
        self.element.floating
    }

    pub fn visual_clone(&self) -> VisualElementClone {
        VisualElementClone::from_resolved(self.element)
    }

    pub fn children(&self) -> Vec<ElementSnapshot<'a>> {
        self.element
            .children
            .iter()
            .map(|element| ElementSnapshot { element })
            .collect()
    }

    pub fn child_count(&self) -> usize {
        self.element.children.len()
    }

    pub fn is_empty(&self) -> bool {
        self.element.children.is_empty()
    }

    pub fn find(&self, id: &str) -> Option<ElementSnapshot<'a>> {
        find_element(self.element, id).map(|element| ElementSnapshot { element })
    }

    pub fn require(&self, id: &str) -> Result<ElementSnapshot<'a>, DocumentQueryError> {
        self.find(id).ok_or_else(|| DocumentQueryError::missing(id))
    }

    pub fn contains(&self, id: &str) -> bool {
        self.find(id).is_some()
    }

    pub fn elements_with_class(&self, class: impl Into<ClassName>) -> Vec<ElementSnapshot<'a>> {
        let class = class.into();
        let mut elements = Vec::new();
        collect_elements(self.element, &mut elements, &mut |element| {
            element
                .classes
                .iter()
                .any(|element_class| element_class == &class)
        });
        elements
    }

    pub fn contains_class(&self, class: impl Into<ClassName>) -> bool {
        let class = class.into();
        find_matching_element(self.element, &mut |element| {
            element
                .classes
                .iter()
                .any(|element_class| element_class == &class)
        })
        .is_some()
    }

    pub fn first_with_class(&self, class: impl Into<ClassName>) -> Option<ElementSnapshot<'a>> {
        let class = class.into();
        find_matching_element(self.element, &mut |element| {
            element
                .classes
                .iter()
                .any(|element_class| element_class == &class)
        })
        .map(|element| ElementSnapshot { element })
    }

    pub fn count_with_class(&self, class: impl Into<ClassName>) -> usize {
        self.elements_with_class(class).len()
    }

    pub fn elements_with_attribute(
        &self,
        name: impl AsRef<str>,
        value: impl AsRef<str>,
    ) -> Vec<ElementSnapshot<'a>> {
        let name = name.as_ref();
        let value = value.as_ref();
        let mut elements = Vec::new();
        collect_elements(self.element, &mut elements, &mut |element| {
            element
                .attributes
                .get(name)
                .is_some_and(|found| found == value)
        });
        elements
    }

    pub fn contains_attribute(&self, name: impl AsRef<str>, value: impl AsRef<str>) -> bool {
        let name = name.as_ref();
        let value = value.as_ref();
        find_matching_element(self.element, &mut |element| {
            element
                .attributes
                .get(name)
                .is_some_and(|found| found == value)
        })
        .is_some()
    }

    pub fn first_with_attribute(
        &self,
        name: impl AsRef<str>,
        value: impl AsRef<str>,
    ) -> Option<ElementSnapshot<'a>> {
        let name = name.as_ref();
        let value = value.as_ref();
        find_matching_element(self.element, &mut |element| {
            element
                .attributes
                .get(name)
                .is_some_and(|found| found == value)
        })
        .map(|element| ElementSnapshot { element })
    }

    pub fn count_with_attribute(&self, name: impl AsRef<str>, value: impl AsRef<str>) -> usize {
        self.elements_with_attribute(name, value).len()
    }

    pub fn elements_with_data(
        &self,
        name: impl AsRef<str>,
        value: impl AsRef<str>,
    ) -> Vec<ElementSnapshot<'a>> {
        self.elements_with_attribute(prefixed_attribute_name("data-", name.as_ref()), value)
    }

    pub fn contains_data(&self, name: impl AsRef<str>, value: impl AsRef<str>) -> bool {
        self.contains_attribute(prefixed_attribute_name("data-", name.as_ref()), value)
    }

    pub fn first_with_data(
        &self,
        name: impl AsRef<str>,
        value: impl AsRef<str>,
    ) -> Option<ElementSnapshot<'a>> {
        self.first_with_attribute(prefixed_attribute_name("data-", name.as_ref()), value)
    }

    pub fn count_with_data(&self, name: impl AsRef<str>, value: impl AsRef<str>) -> usize {
        self.count_with_attribute(prefixed_attribute_name("data-", name.as_ref()), value)
    }

    pub fn elements_with_aria(
        &self,
        name: impl AsRef<str>,
        value: impl AsRef<str>,
    ) -> Vec<ElementSnapshot<'a>> {
        self.elements_with_attribute(prefixed_attribute_name("aria-", name.as_ref()), value)
    }

    pub fn contains_aria(&self, name: impl AsRef<str>, value: impl AsRef<str>) -> bool {
        self.contains_attribute(prefixed_attribute_name("aria-", name.as_ref()), value)
    }

    pub fn first_with_aria(
        &self,
        name: impl AsRef<str>,
        value: impl AsRef<str>,
    ) -> Option<ElementSnapshot<'a>> {
        self.first_with_attribute(prefixed_attribute_name("aria-", name.as_ref()), value)
    }

    pub fn count_with_aria(&self, name: impl AsRef<str>, value: impl AsRef<str>) -> usize {
        self.count_with_attribute(prefixed_attribute_name("aria-", name.as_ref()), value)
    }

    pub fn elements_by_element(&self, target: Element) -> Vec<ElementSnapshot<'a>> {
        let mut elements = Vec::new();
        collect_elements(self.element, &mut elements, &mut |element| {
            element.element == target
        });
        elements
    }

    pub fn contains_element(&self, target: Element) -> bool {
        find_matching_element(self.element, &mut |element| element.element == target).is_some()
    }

    pub fn first_by_element(&self, target: Element) -> Option<ElementSnapshot<'a>> {
        find_matching_element(self.element, &mut |element| element.element == target)
            .map(|element| ElementSnapshot { element })
    }

    pub fn count_by_element(&self, target: Element) -> usize {
        self.elements_by_element(target).len()
    }

    pub fn selected_elements(&self) -> Vec<ElementSnapshot<'a>> {
        self.elements_matching(|element| element.selected)
    }

    pub fn first_selected(&self) -> Option<ElementSnapshot<'a>> {
        self.first_matching(|element| element.selected)
    }

    pub fn contains_selected(&self) -> bool {
        self.first_selected().is_some()
    }

    pub fn count_selected(&self) -> usize {
        self.selected_elements().len()
    }

    pub fn disabled_elements(&self) -> Vec<ElementSnapshot<'a>> {
        self.elements_matching(|element| element.disabled)
    }

    pub fn first_disabled(&self) -> Option<ElementSnapshot<'a>> {
        self.first_matching(|element| element.disabled)
    }

    pub fn contains_disabled(&self) -> bool {
        self.first_disabled().is_some()
    }

    pub fn count_disabled(&self) -> usize {
        self.disabled_elements().len()
    }

    pub fn focused_elements(&self) -> Vec<ElementSnapshot<'a>> {
        self.elements_matching(|element| element.focused)
    }

    pub fn first_focused(&self) -> Option<ElementSnapshot<'a>> {
        self.first_matching(|element| element.focused)
    }

    pub fn contains_focused(&self) -> bool {
        self.first_focused().is_some()
    }

    pub fn count_focused(&self) -> usize {
        self.focused_elements().len()
    }

    pub fn interactive_elements(&self) -> Vec<ElementSnapshot<'a>> {
        self.elements_matching(|element| element.interactive)
    }

    pub fn first_interactive(&self) -> Option<ElementSnapshot<'a>> {
        self.first_matching(|element| element.interactive)
    }

    pub fn contains_interactive(&self) -> bool {
        self.first_interactive().is_some()
    }

    pub fn count_interactive(&self) -> usize {
        self.interactive_elements().len()
    }

    fn elements_matching(
        &self,
        mut predicate: impl FnMut(&ResolvedElement) -> bool,
    ) -> Vec<ElementSnapshot<'a>> {
        let mut elements = Vec::new();
        collect_elements(self.element, &mut elements, &mut predicate);
        elements
    }

    fn first_matching(
        &self,
        mut predicate: impl FnMut(&ResolvedElement) -> bool,
    ) -> Option<ElementSnapshot<'a>> {
        find_matching_element(self.element, &mut predicate)
            .map(|element| ElementSnapshot { element })
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

fn find_matching_element<'a>(
    element: &'a ResolvedElement,
    predicate: &mut impl FnMut(&ResolvedElement) -> bool,
) -> Option<&'a ResolvedElement> {
    if predicate(element) {
        return Some(element);
    }
    element
        .children
        .iter()
        .find_map(|child| find_matching_element(child, predicate))
}

fn prefixed_attribute_name(prefix: &str, name: &str) -> String {
    if name.starts_with(prefix) {
        name.to_owned()
    } else {
        format!("{prefix}{name}")
    }
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
