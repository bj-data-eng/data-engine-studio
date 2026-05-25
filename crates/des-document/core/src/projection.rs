use crate::{ClassName, Document, DocumentError, DocumentResult, ElementId, TextContent};
use std::borrow::Borrow;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DocumentProjection {
    operations: Vec<DocumentProjectionOperation>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum DocumentProjectionOperation {
    SetText {
        id: ElementId,
        text: TextContent,
    },
    SetValue {
        id: ElementId,
        value: String,
    },
    SetAttribute {
        id: ElementId,
        name: String,
        value: String,
    },
    RemoveAttribute {
        id: ElementId,
        name: String,
    },
    SetSelected {
        id: ElementId,
        selected: bool,
    },
    SetDisabled {
        id: ElementId,
        disabled: bool,
    },
    SetFocused {
        id: ElementId,
        focused: bool,
    },
    SetClass {
        id: ElementId,
        class: ClassName,
        present: bool,
    },
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DocumentProjectionReport {
    pub operations: usize,
    pub changed: usize,
}

impl DocumentProjectionReport {
    pub const fn new(operations: usize, changed: usize) -> Self {
        Self {
            operations,
            changed,
        }
    }

    pub const fn operation_count(&self) -> usize {
        self.operations
    }

    pub const fn changed_count(&self) -> usize {
        self.changed
    }

    pub const fn is_empty(&self) -> bool {
        self.operations == 0
    }

    pub const fn changed_any(&self) -> bool {
        self.changed > 0
    }

    pub const fn unchanged(&self) -> bool {
        self.changed == 0
    }

    pub const fn changed_all(&self) -> bool {
        self.changed == self.operations
    }
}

impl DocumentProjection {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_operations(
        operations: impl IntoIterator<Item = DocumentProjectionOperation>,
    ) -> Self {
        Self {
            operations: operations.into_iter().collect(),
        }
    }

    pub fn element(&mut self, id: impl Into<ElementId>) -> ElementProjection<'_> {
        ElementProjection {
            projection: self,
            id: id.into(),
        }
    }

    pub fn with_element(
        mut self,
        id: impl Into<ElementId>,
        project: impl FnOnce(ElementProjection<'_>),
    ) -> Self {
        project(self.element(id));
        self
    }

    pub fn element_if(
        &mut self,
        id: impl Into<ElementId>,
        present: bool,
        project: impl FnOnce(ElementProjection<'_>),
    ) -> &mut Self {
        if present {
            project(self.element(id));
        }
        self
    }

    pub fn with_element_if(
        mut self,
        id: impl Into<ElementId>,
        present: bool,
        project: impl FnOnce(ElementProjection<'_>),
    ) -> Self {
        self.element_if(id, present, project);
        self
    }

    pub fn elements<I, Id>(
        &mut self,
        ids: I,
        mut project: impl FnMut(ElementProjection<'_>),
    ) -> &mut Self
    where
        I: IntoIterator<Item = Id>,
        Id: Into<ElementId>,
    {
        for id in ids {
            project(self.element(id));
        }
        self
    }

    pub fn with_elements<I, Id>(
        mut self,
        ids: I,
        project: impl FnMut(ElementProjection<'_>),
    ) -> Self
    where
        I: IntoIterator<Item = Id>,
        Id: Into<ElementId>,
    {
        self.elements(ids, project);
        self
    }

    pub fn elements_if<I, Id>(
        &mut self,
        ids: I,
        present: bool,
        project: impl FnMut(ElementProjection<'_>),
    ) -> &mut Self
    where
        I: IntoIterator<Item = Id>,
        Id: Into<ElementId>,
    {
        if present {
            self.elements(ids, project);
        }
        self
    }

    pub fn with_elements_if<I, Id>(
        mut self,
        ids: I,
        present: bool,
        project: impl FnMut(ElementProjection<'_>),
    ) -> Self
    where
        I: IntoIterator<Item = Id>,
        Id: Into<ElementId>,
    {
        self.elements_if(ids, present, project);
        self
    }

    pub fn items<I, Id, IdFor, Project>(
        &mut self,
        items: I,
        mut id_for: IdFor,
        mut project: Project,
    ) -> &mut Self
    where
        I: IntoIterator,
        Id: Into<ElementId>,
        IdFor: FnMut(&I::Item) -> Id,
        Project: FnMut(ElementProjection<'_>, I::Item),
    {
        for item in items {
            let id = id_for(&item);
            project(self.element(id), item);
        }
        self
    }

    pub fn with_items<I, Id, IdFor, Project>(
        mut self,
        items: I,
        id_for: IdFor,
        project: Project,
    ) -> Self
    where
        I: IntoIterator,
        Id: Into<ElementId>,
        IdFor: FnMut(&I::Item) -> Id,
        Project: FnMut(ElementProjection<'_>, I::Item),
    {
        self.items(items, id_for, project);
        self
    }

    pub fn items_if<I, Id, IdFor, Project>(
        &mut self,
        items: I,
        present: bool,
        id_for: IdFor,
        project: Project,
    ) -> &mut Self
    where
        I: IntoIterator,
        Id: Into<ElementId>,
        IdFor: FnMut(&I::Item) -> Id,
        Project: FnMut(ElementProjection<'_>, I::Item),
    {
        if present {
            self.items(items, id_for, project);
        }
        self
    }

    pub fn with_items_if<I, Id, IdFor, Project>(
        mut self,
        items: I,
        present: bool,
        id_for: IdFor,
        project: Project,
    ) -> Self
    where
        I: IntoIterator,
        Id: Into<ElementId>,
        IdFor: FnMut(&I::Item) -> Id,
        Project: FnMut(ElementProjection<'_>, I::Item),
    {
        self.items_if(items, present, id_for, project);
        self
    }

    pub fn when(mut self, present: bool, project: impl FnOnce(&mut Self)) -> Self {
        self.project_if(present, project);
        self
    }

    pub fn project_if(&mut self, present: bool, project: impl FnOnce(&mut Self)) -> &mut Self {
        if present {
            project(self);
        }
        self
    }

    pub fn set_patch(
        mut self,
        id: impl Into<ElementId>,
        patch: impl Borrow<ElementProjectionPatch>,
    ) -> Self {
        self.push_patch(id, patch);
        self
    }

    pub fn push_patch(
        &mut self,
        id: impl Into<ElementId>,
        patch: impl Borrow<ElementProjectionPatch>,
    ) {
        patch.borrow().apply_to(self.element(id));
    }

    pub fn set_text(mut self, id: impl Into<ElementId>, text: impl Into<TextContent>) -> Self {
        self.push_text(id, text);
        self
    }

    pub fn push_text(&mut self, id: impl Into<ElementId>, text: impl Into<TextContent>) {
        self.operations.push(DocumentProjectionOperation::SetText {
            id: id.into(),
            text: text.into(),
        });
    }

    pub fn set_value(mut self, id: impl Into<ElementId>, value: impl Into<String>) -> Self {
        self.push_value(id, value);
        self
    }

    pub fn push_value(&mut self, id: impl Into<ElementId>, value: impl Into<String>) {
        self.operations.push(DocumentProjectionOperation::SetValue {
            id: id.into(),
            value: value.into(),
        });
    }

    pub fn set_attribute(
        mut self,
        id: impl Into<ElementId>,
        name: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        self.push_attribute(id, name, value);
        self
    }

    pub fn push_attribute(
        &mut self,
        id: impl Into<ElementId>,
        name: impl Into<String>,
        value: impl Into<String>,
    ) {
        self.operations
            .push(DocumentProjectionOperation::SetAttribute {
                id: id.into(),
                name: name.into(),
                value: value.into(),
            });
    }

    pub fn set_attributes<I, K, V>(mut self, id: impl Into<ElementId>, attributes: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        self.push_attributes(id, attributes);
        self
    }

    pub fn push_attributes<I, K, V>(&mut self, id: impl Into<ElementId>, attributes: I)
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        let id = id.into();
        for (name, value) in attributes {
            self.push_attribute(id.clone(), name, value);
        }
    }

    pub fn set_attribute_if(
        self,
        id: impl Into<ElementId>,
        name: impl Into<String>,
        value: impl Into<String>,
        present: bool,
    ) -> Self {
        if present {
            self.set_attribute(id, name, value)
        } else {
            self.remove_attribute(id, name)
        }
    }

    pub fn push_attribute_if(
        &mut self,
        id: impl Into<ElementId>,
        name: impl Into<String>,
        value: impl Into<String>,
        present: bool,
    ) {
        if present {
            self.push_attribute(id, name, value);
        } else {
            self.push_remove_attribute(id, name);
        }
    }

    pub fn set_data(
        self,
        id: impl Into<ElementId>,
        name: impl AsRef<str>,
        value: impl Into<String>,
    ) -> Self {
        self.set_attribute(id, prefixed_attribute_name("data-", name), value)
    }

    pub fn push_data(
        &mut self,
        id: impl Into<ElementId>,
        name: impl AsRef<str>,
        value: impl Into<String>,
    ) {
        self.push_attribute(id, prefixed_attribute_name("data-", name), value);
    }

    pub fn set_data_if(
        self,
        id: impl Into<ElementId>,
        name: impl AsRef<str>,
        value: impl Into<String>,
        present: bool,
    ) -> Self {
        if present {
            self.set_data(id, name, value)
        } else {
            self.remove_data(id, name)
        }
    }

    pub fn push_data_if(
        &mut self,
        id: impl Into<ElementId>,
        name: impl AsRef<str>,
        value: impl Into<String>,
        present: bool,
    ) {
        if present {
            self.push_data(id, name, value);
        } else {
            self.push_remove_data(id, name);
        }
    }

    pub fn set_aria(
        self,
        id: impl Into<ElementId>,
        name: impl AsRef<str>,
        value: impl Into<String>,
    ) -> Self {
        self.set_attribute(id, prefixed_attribute_name("aria-", name), value)
    }

    pub fn push_aria(
        &mut self,
        id: impl Into<ElementId>,
        name: impl AsRef<str>,
        value: impl Into<String>,
    ) {
        self.push_attribute(id, prefixed_attribute_name("aria-", name), value);
    }

    pub fn set_aria_if(
        self,
        id: impl Into<ElementId>,
        name: impl AsRef<str>,
        value: impl Into<String>,
        present: bool,
    ) -> Self {
        if present {
            self.set_aria(id, name, value)
        } else {
            self.remove_aria(id, name)
        }
    }

    pub fn push_aria_if(
        &mut self,
        id: impl Into<ElementId>,
        name: impl AsRef<str>,
        value: impl Into<String>,
        present: bool,
    ) {
        if present {
            self.push_aria(id, name, value);
        } else {
            self.push_remove_aria(id, name);
        }
    }

    pub fn remove_attribute(mut self, id: impl Into<ElementId>, name: impl Into<String>) -> Self {
        self.push_remove_attribute(id, name);
        self
    }

    pub fn push_remove_attribute(&mut self, id: impl Into<ElementId>, name: impl Into<String>) {
        self.operations
            .push(DocumentProjectionOperation::RemoveAttribute {
                id: id.into(),
                name: name.into(),
            });
    }

    pub fn remove_attributes<I, K>(mut self, id: impl Into<ElementId>, names: I) -> Self
    where
        I: IntoIterator<Item = K>,
        K: Into<String>,
    {
        self.push_remove_attributes(id, names);
        self
    }

    pub fn push_remove_attributes<I, K>(&mut self, id: impl Into<ElementId>, names: I)
    where
        I: IntoIterator<Item = K>,
        K: Into<String>,
    {
        let id = id.into();
        for name in names {
            self.push_remove_attribute(id.clone(), name);
        }
    }

    pub fn remove_data(self, id: impl Into<ElementId>, name: impl AsRef<str>) -> Self {
        self.remove_attribute(id, prefixed_attribute_name("data-", name))
    }

    pub fn push_remove_data(&mut self, id: impl Into<ElementId>, name: impl AsRef<str>) {
        self.push_remove_attribute(id, prefixed_attribute_name("data-", name));
    }

    pub fn remove_aria(self, id: impl Into<ElementId>, name: impl AsRef<str>) -> Self {
        self.remove_attribute(id, prefixed_attribute_name("aria-", name))
    }

    pub fn push_remove_aria(&mut self, id: impl Into<ElementId>, name: impl AsRef<str>) {
        self.push_remove_attribute(id, prefixed_attribute_name("aria-", name));
    }

    pub fn set_selected(mut self, id: impl Into<ElementId>, selected: bool) -> Self {
        self.push_selected(id, selected);
        self
    }

    pub fn select(self, id: impl Into<ElementId>) -> Self {
        self.set_selected(id, true)
    }

    pub fn select_if(mut self, id: impl Into<ElementId>, present: bool) -> Self {
        if present {
            self.push_selected(id, true);
        }
        self
    }

    pub fn deselect(self, id: impl Into<ElementId>) -> Self {
        self.set_selected(id, false)
    }

    pub fn deselect_if(mut self, id: impl Into<ElementId>, present: bool) -> Self {
        if present {
            self.push_selected(id, false);
        }
        self
    }

    pub fn push_selected(&mut self, id: impl Into<ElementId>, selected: bool) {
        self.operations
            .push(DocumentProjectionOperation::SetSelected {
                id: id.into(),
                selected,
            });
    }

    pub fn push_select_if(&mut self, id: impl Into<ElementId>, present: bool) {
        if present {
            self.push_selected(id, true);
        }
    }

    pub fn push_deselect_if(&mut self, id: impl Into<ElementId>, present: bool) {
        if present {
            self.push_selected(id, false);
        }
    }

    pub fn set_disabled(mut self, id: impl Into<ElementId>, disabled: bool) -> Self {
        self.push_disabled(id, disabled);
        self
    }

    pub fn disable(self, id: impl Into<ElementId>) -> Self {
        self.set_disabled(id, true)
    }

    pub fn disable_if(mut self, id: impl Into<ElementId>, present: bool) -> Self {
        if present {
            self.push_disabled(id, true);
        }
        self
    }

    pub fn enable(self, id: impl Into<ElementId>) -> Self {
        self.set_disabled(id, false)
    }

    pub fn enable_if(mut self, id: impl Into<ElementId>, present: bool) -> Self {
        if present {
            self.push_disabled(id, false);
        }
        self
    }

    pub fn push_disabled(&mut self, id: impl Into<ElementId>, disabled: bool) {
        self.operations
            .push(DocumentProjectionOperation::SetDisabled {
                id: id.into(),
                disabled,
            });
    }

    pub fn push_disable_if(&mut self, id: impl Into<ElementId>, present: bool) {
        if present {
            self.push_disabled(id, true);
        }
    }

    pub fn push_enable_if(&mut self, id: impl Into<ElementId>, present: bool) {
        if present {
            self.push_disabled(id, false);
        }
    }

    pub fn set_focused(mut self, id: impl Into<ElementId>, focused: bool) -> Self {
        self.push_focused(id, focused);
        self
    }

    pub fn focus(self, id: impl Into<ElementId>) -> Self {
        self.set_focused(id, true)
    }

    pub fn focus_if(mut self, id: impl Into<ElementId>, present: bool) -> Self {
        if present {
            self.push_focused(id, true);
        }
        self
    }

    pub fn blur(self, id: impl Into<ElementId>) -> Self {
        self.set_focused(id, false)
    }

    pub fn blur_if(mut self, id: impl Into<ElementId>, present: bool) -> Self {
        if present {
            self.push_focused(id, false);
        }
        self
    }

    pub fn push_focused(&mut self, id: impl Into<ElementId>, focused: bool) {
        self.operations
            .push(DocumentProjectionOperation::SetFocused {
                id: id.into(),
                focused,
            });
    }

    pub fn push_focus_if(&mut self, id: impl Into<ElementId>, present: bool) {
        if present {
            self.push_focused(id, true);
        }
    }

    pub fn push_blur_if(&mut self, id: impl Into<ElementId>, present: bool) {
        if present {
            self.push_focused(id, false);
        }
    }

    pub fn set_class(
        mut self,
        id: impl Into<ElementId>,
        class: impl Into<ClassName>,
        present: bool,
    ) -> Self {
        self.push_class(id, class, present);
        self
    }

    pub fn push_class(
        &mut self,
        id: impl Into<ElementId>,
        class: impl Into<ClassName>,
        present: bool,
    ) {
        self.operations.push(DocumentProjectionOperation::SetClass {
            id: id.into(),
            class: class.into(),
            present,
        });
    }

    pub fn add_class(self, id: impl Into<ElementId>, class: impl Into<ClassName>) -> Self {
        self.set_class(id, class, true)
    }

    pub fn class_if(
        self,
        id: impl Into<ElementId>,
        class: impl Into<ClassName>,
        present: bool,
    ) -> Self {
        self.set_class(id, class, present)
    }

    pub fn push_add_class(&mut self, id: impl Into<ElementId>, class: impl Into<ClassName>) {
        self.push_class(id, class, true);
    }

    pub fn push_class_if(
        &mut self,
        id: impl Into<ElementId>,
        class: impl Into<ClassName>,
        present: bool,
    ) {
        self.push_class(id, class, present);
    }

    pub fn add_classes<I, C>(mut self, id: impl Into<ElementId>, classes: I) -> Self
    where
        I: IntoIterator<Item = C>,
        C: Into<ClassName>,
    {
        self.push_add_classes(id, classes);
        self
    }

    pub fn classes_if<I, C>(mut self, id: impl Into<ElementId>, classes: I, present: bool) -> Self
    where
        I: IntoIterator<Item = C>,
        C: Into<ClassName>,
    {
        self.push_classes_if(id, classes, present);
        self
    }

    pub fn push_add_classes<I, C>(&mut self, id: impl Into<ElementId>, classes: I)
    where
        I: IntoIterator<Item = C>,
        C: Into<ClassName>,
    {
        let id = id.into();
        for class in classes {
            self.push_class(id.clone(), class, true);
        }
    }

    pub fn push_classes_if<I, C>(&mut self, id: impl Into<ElementId>, classes: I, present: bool)
    where
        I: IntoIterator<Item = C>,
        C: Into<ClassName>,
    {
        let id = id.into();
        for class in classes {
            self.push_class(id.clone(), class, present);
        }
    }

    pub fn remove_class(self, id: impl Into<ElementId>, class: impl Into<ClassName>) -> Self {
        self.set_class(id, class, false)
    }

    pub fn push_remove_class(&mut self, id: impl Into<ElementId>, class: impl Into<ClassName>) {
        self.push_class(id, class, false);
    }

    pub fn remove_classes<I, C>(mut self, id: impl Into<ElementId>, classes: I) -> Self
    where
        I: IntoIterator<Item = C>,
        C: Into<ClassName>,
    {
        self.push_remove_classes(id, classes);
        self
    }

    pub fn push_remove_classes<I, C>(&mut self, id: impl Into<ElementId>, classes: I)
    where
        I: IntoIterator<Item = C>,
        C: Into<ClassName>,
    {
        let id = id.into();
        for class in classes {
            self.push_class(id.clone(), class, false);
        }
    }

    pub fn push_operation(&mut self, operation: DocumentProjectionOperation) {
        self.operations.push(operation);
    }

    pub fn extend(&mut self, projection: impl Into<DocumentProjection>) {
        self.operations.extend(projection.into().operations);
    }

    pub fn with_projection(mut self, projection: impl Into<DocumentProjection>) -> Self {
        self.extend(projection);
        self
    }

    pub fn extend_if(&mut self, projection: impl Into<DocumentProjection>, present: bool) {
        if present {
            self.extend(projection);
        }
    }

    pub fn with_projection_if(
        mut self,
        projection: impl Into<DocumentProjection>,
        present: bool,
    ) -> Self {
        self.extend_if(projection, present);
        self
    }

    pub fn operations(&self) -> &[DocumentProjectionOperation] {
        &self.operations
    }

    pub fn len(&self) -> usize {
        self.operations.len()
    }

    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }

    pub fn clear(&mut self) {
        self.operations.clear();
    }

    pub fn apply_to(&self, document: &mut Document) -> DocumentResult<DocumentProjectionReport> {
        let mut report = DocumentProjectionReport::new(self.operations.len(), 0);
        for operation in &self.operations {
            if operation.apply_to(document)? {
                report.changed += 1;
            }
        }
        Ok(report)
    }
}

impl FromIterator<DocumentProjectionOperation> for DocumentProjection {
    fn from_iter<T: IntoIterator<Item = DocumentProjectionOperation>>(iter: T) -> Self {
        Self::from_operations(iter)
    }
}

pub struct ElementProjection<'a> {
    projection: &'a mut DocumentProjection,
    id: ElementId,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ElementProjectionPatch {
    text: Option<TextContent>,
    value: Option<String>,
    attributes: Vec<(String, String)>,
    removed_attributes: Vec<String>,
    selected: Option<bool>,
    disabled: Option<bool>,
    focused: Option<bool>,
    classes: Vec<(ClassName, bool)>,
}

impl ElementProjectionPatch {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn text(mut self, text: impl Into<TextContent>) -> Self {
        self.text = Some(text.into());
        self
    }

    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    pub fn attribute(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.push((name.into(), value.into()));
        self
    }

    pub fn attribute_if(
        mut self,
        name: impl Into<String>,
        value: impl Into<String>,
        present: bool,
    ) -> Self {
        if present {
            self.attributes.push((name.into(), value.into()));
        } else {
            self.removed_attributes.push(name.into());
        }
        self
    }

    pub fn attributes<I, K, V>(mut self, attributes: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        self.attributes.extend(
            attributes
                .into_iter()
                .map(|(name, value)| (name.into(), value.into())),
        );
        self
    }

    pub fn data(self, name: impl AsRef<str>, value: impl Into<String>) -> Self {
        self.attribute(prefixed_attribute_name("data-", name), value)
    }

    pub fn data_if(self, name: impl AsRef<str>, value: impl Into<String>, present: bool) -> Self {
        self.attribute_if(prefixed_attribute_name("data-", name), value, present)
    }

    pub fn aria(self, name: impl AsRef<str>, value: impl Into<String>) -> Self {
        self.attribute(prefixed_attribute_name("aria-", name), value)
    }

    pub fn aria_if(self, name: impl AsRef<str>, value: impl Into<String>, present: bool) -> Self {
        self.attribute_if(prefixed_attribute_name("aria-", name), value, present)
    }

    pub fn remove_attribute(mut self, name: impl Into<String>) -> Self {
        self.removed_attributes.push(name.into());
        self
    }

    pub fn remove_attributes<I, K>(mut self, names: I) -> Self
    where
        I: IntoIterator<Item = K>,
        K: Into<String>,
    {
        self.removed_attributes
            .extend(names.into_iter().map(Into::into));
        self
    }

    pub fn remove_data(self, name: impl AsRef<str>) -> Self {
        self.remove_attribute(prefixed_attribute_name("data-", name))
    }

    pub fn remove_aria(self, name: impl AsRef<str>) -> Self {
        self.remove_attribute(prefixed_attribute_name("aria-", name))
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = Some(selected);
        self
    }

    pub fn select(self) -> Self {
        self.selected(true)
    }

    pub fn deselect(self) -> Self {
        self.selected(false)
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = Some(disabled);
        self
    }

    pub fn disable(self) -> Self {
        self.disabled(true)
    }

    pub fn enable(self) -> Self {
        self.disabled(false)
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = Some(focused);
        self
    }

    pub fn focus(self) -> Self {
        self.focused(true)
    }

    pub fn blur(self) -> Self {
        self.focused(false)
    }

    pub fn class(mut self, class: impl Into<ClassName>, present: bool) -> Self {
        self.classes.push((class.into(), present));
        self
    }

    pub fn add_class(self, class: impl Into<ClassName>) -> Self {
        self.class(class, true)
    }

    pub fn remove_class(self, class: impl Into<ClassName>) -> Self {
        self.class(class, false)
    }

    pub fn add_classes<I, C>(mut self, classes: I) -> Self
    where
        I: IntoIterator<Item = C>,
        C: Into<ClassName>,
    {
        self.classes
            .extend(classes.into_iter().map(|class| (class.into(), true)));
        self
    }

    pub fn remove_classes<I, C>(mut self, classes: I) -> Self
    where
        I: IntoIterator<Item = C>,
        C: Into<ClassName>,
    {
        self.classes
            .extend(classes.into_iter().map(|class| (class.into(), false)));
        self
    }

    pub fn class_if(self, class: impl Into<ClassName>, present: bool) -> Self {
        self.class(class, present)
    }

    pub fn classes_if<I, C>(mut self, classes: I, present: bool) -> Self
    where
        I: IntoIterator<Item = C>,
        C: Into<ClassName>,
    {
        self.classes
            .extend(classes.into_iter().map(|class| (class.into(), present)));
        self
    }

    pub fn is_empty(&self) -> bool {
        self.operation_count() == 0
    }

    pub fn operation_count(&self) -> usize {
        usize::from(self.text.is_some())
            + usize::from(self.value.is_some())
            + self.attributes.len()
            + self.removed_attributes.len()
            + usize::from(self.selected.is_some())
            + usize::from(self.disabled.is_some())
            + usize::from(self.focused.is_some())
            + self.classes.len()
    }

    pub fn apply_to(&self, mut element: ElementProjection<'_>) {
        if let Some(text) = &self.text {
            element.text(text.clone());
        }
        if let Some(value) = &self.value {
            element.value(value.clone());
        }
        for (name, value) in &self.attributes {
            element.attribute(name.clone(), value.clone());
        }
        for name in &self.removed_attributes {
            element.remove_attribute(name.clone());
        }
        if let Some(selected) = self.selected {
            element.selected(selected);
        }
        if let Some(disabled) = self.disabled {
            element.disabled(disabled);
        }
        if let Some(focused) = self.focused {
            element.focused(focused);
        }
        for (class, present) in &self.classes {
            element.class(class.clone(), *present);
        }
    }
}

impl ElementProjection<'_> {
    pub fn id(&self) -> &ElementId {
        &self.id
    }

    pub fn when(&mut self, present: bool, project: impl FnOnce(&mut Self)) -> &mut Self {
        if present {
            project(self);
        }
        self
    }

    pub fn text(&mut self, text: impl Into<TextContent>) -> &mut Self {
        self.projection.push_text(self.id.clone(), text);
        self
    }

    pub fn patch(&mut self, patch: impl Borrow<ElementProjectionPatch>) -> &mut Self {
        patch.borrow().apply_to(ElementProjection {
            projection: self.projection,
            id: self.id.clone(),
        });
        self
    }

    pub fn value(&mut self, value: impl Into<String>) -> &mut Self {
        self.projection.push_value(self.id.clone(), value);
        self
    }

    pub fn attribute(&mut self, name: impl Into<String>, value: impl Into<String>) -> &mut Self {
        self.projection
            .push_attribute(self.id.clone(), name.into(), value.into());
        self
    }

    pub fn attribute_if(
        &mut self,
        name: impl Into<String>,
        value: impl Into<String>,
        present: bool,
    ) -> &mut Self {
        self.projection
            .push_attribute_if(self.id.clone(), name, value, present);
        self
    }

    pub fn attributes<I, K, V>(&mut self, attributes: I) -> &mut Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        self.projection.push_attributes(self.id.clone(), attributes);
        self
    }

    pub fn data(&mut self, name: impl AsRef<str>, value: impl Into<String>) -> &mut Self {
        self.projection.push_data(self.id.clone(), name, value);
        self
    }

    pub fn data_if(
        &mut self,
        name: impl AsRef<str>,
        value: impl Into<String>,
        present: bool,
    ) -> &mut Self {
        self.projection
            .push_data_if(self.id.clone(), name, value, present);
        self
    }

    pub fn aria(&mut self, name: impl AsRef<str>, value: impl Into<String>) -> &mut Self {
        self.projection.push_aria(self.id.clone(), name, value);
        self
    }

    pub fn aria_if(
        &mut self,
        name: impl AsRef<str>,
        value: impl Into<String>,
        present: bool,
    ) -> &mut Self {
        self.projection
            .push_aria_if(self.id.clone(), name, value, present);
        self
    }

    pub fn remove_attribute(&mut self, name: impl Into<String>) -> &mut Self {
        self.projection
            .push_remove_attribute(self.id.clone(), name.into());
        self
    }

    pub fn remove_attributes<I, K>(&mut self, names: I) -> &mut Self
    where
        I: IntoIterator<Item = K>,
        K: Into<String>,
    {
        self.projection
            .push_remove_attributes(self.id.clone(), names);
        self
    }

    pub fn remove_data(&mut self, name: impl AsRef<str>) -> &mut Self {
        self.projection.push_remove_data(self.id.clone(), name);
        self
    }

    pub fn remove_aria(&mut self, name: impl AsRef<str>) -> &mut Self {
        self.projection.push_remove_aria(self.id.clone(), name);
        self
    }

    pub fn selected(&mut self, selected: bool) -> &mut Self {
        self.projection.push_selected(self.id.clone(), selected);
        self
    }

    pub fn select(&mut self) -> &mut Self {
        self.selected(true)
    }

    pub fn select_if(&mut self, present: bool) -> &mut Self {
        if present {
            self.select();
        }
        self
    }

    pub fn deselect(&mut self) -> &mut Self {
        self.selected(false)
    }

    pub fn deselect_if(&mut self, present: bool) -> &mut Self {
        if present {
            self.deselect();
        }
        self
    }

    pub fn disabled(&mut self, disabled: bool) -> &mut Self {
        self.projection.push_disabled(self.id.clone(), disabled);
        self
    }

    pub fn disable(&mut self) -> &mut Self {
        self.disabled(true)
    }

    pub fn disable_if(&mut self, present: bool) -> &mut Self {
        if present {
            self.disable();
        }
        self
    }

    pub fn enable(&mut self) -> &mut Self {
        self.disabled(false)
    }

    pub fn enable_if(&mut self, present: bool) -> &mut Self {
        if present {
            self.enable();
        }
        self
    }

    pub fn focused(&mut self, focused: bool) -> &mut Self {
        self.projection.push_focused(self.id.clone(), focused);
        self
    }

    pub fn focus(&mut self) -> &mut Self {
        self.focused(true)
    }

    pub fn focus_if(&mut self, present: bool) -> &mut Self {
        if present {
            self.focus();
        }
        self
    }

    pub fn blur(&mut self) -> &mut Self {
        self.focused(false)
    }

    pub fn blur_if(&mut self, present: bool) -> &mut Self {
        if present {
            self.blur();
        }
        self
    }

    pub fn class(&mut self, class: impl Into<ClassName>, present: bool) -> &mut Self {
        self.projection
            .push_class(self.id.clone(), class.into(), present);
        self
    }

    pub fn add_class(&mut self, class: impl Into<ClassName>) -> &mut Self {
        self.projection.push_add_class(self.id.clone(), class);
        self
    }

    pub fn class_if(&mut self, class: impl Into<ClassName>, present: bool) -> &mut Self {
        self.projection
            .push_class_if(self.id.clone(), class, present);
        self
    }

    pub fn add_classes<I, C>(&mut self, classes: I) -> &mut Self
    where
        I: IntoIterator<Item = C>,
        C: Into<ClassName>,
    {
        self.projection.push_add_classes(self.id.clone(), classes);
        self
    }

    pub fn classes_if<I, C>(&mut self, classes: I, present: bool) -> &mut Self
    where
        I: IntoIterator<Item = C>,
        C: Into<ClassName>,
    {
        self.projection
            .push_classes_if(self.id.clone(), classes, present);
        self
    }

    pub fn remove_class(&mut self, class: impl Into<ClassName>) -> &mut Self {
        self.projection.push_remove_class(self.id.clone(), class);
        self
    }

    pub fn remove_classes<I, C>(&mut self, classes: I) -> &mut Self
    where
        I: IntoIterator<Item = C>,
        C: Into<ClassName>,
    {
        self.projection
            .push_remove_classes(self.id.clone(), classes);
        self
    }
}

fn prefixed_attribute_name(prefix: &str, name: impl AsRef<str>) -> String {
    let name = name.as_ref();
    if name.starts_with(prefix) {
        name.to_owned()
    } else {
        format!("{prefix}{name}")
    }
}

impl DocumentProjectionOperation {
    pub fn target(&self) -> &ElementId {
        match self {
            Self::SetText { id, .. }
            | Self::SetValue { id, .. }
            | Self::SetAttribute { id, .. }
            | Self::RemoveAttribute { id, .. }
            | Self::SetSelected { id, .. }
            | Self::SetDisabled { id, .. }
            | Self::SetFocused { id, .. }
            | Self::SetClass { id, .. } => id,
        }
    }

    fn apply_to(&self, document: &mut Document) -> Result<bool, DocumentError> {
        match self {
            Self::SetText { id, text } => document.set_text(id.clone(), text.clone()),
            Self::SetValue { id, value } => document.set_value(id.clone(), value.clone()),
            Self::SetAttribute { id, name, value } => {
                document.set_attribute(id.clone(), name.clone(), value.clone())
            }
            Self::RemoveAttribute { id, name } => {
                document.remove_attribute(id.clone(), name.clone())
            }
            Self::SetSelected { id, selected } => document.set_selected(id.clone(), *selected),
            Self::SetDisabled { id, disabled } => document.set_disabled(id.clone(), *disabled),
            Self::SetFocused { id, focused } => document.set_focused(id.clone(), *focused),
            Self::SetClass { id, class, present } => {
                if *present {
                    document.add_class(id.clone(), class.clone())
                } else {
                    document.remove_class(id.clone(), class.clone())
                }
            }
        }
    }
}
