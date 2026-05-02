use crate::element::{ClassName, Document, Element, ElementId};

/// Declarative changes that app or service code can apply to a document
/// before style resolution and layout.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DocumentUpdate {
    operations: Vec<ElementUpdate>,
}

impl DocumentUpdate {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a class to the target element if it is not already present.
    pub fn add_class(mut self, target: impl Into<ElementId>, class: impl Into<ClassName>) -> Self {
        self.operations.push(ElementUpdate {
            target: target.into(),
            action: ElementUpdateAction::AddClass(class.into()),
        });
        self
    }

    /// Removes a class from the target element if it is present.
    pub fn remove_class(
        mut self,
        target: impl Into<ElementId>,
        class: impl Into<ClassName>,
    ) -> Self {
        self.operations.push(ElementUpdate {
            target: target.into(),
            action: ElementUpdateAction::RemoveClass(class.into()),
        });
        self
    }

    /// Adds the class when absent and removes it when present.
    pub fn toggle_class(
        mut self,
        target: impl Into<ElementId>,
        class: impl Into<ClassName>,
    ) -> Self {
        self.operations.push(ElementUpdate {
            target: target.into(),
            action: ElementUpdateAction::ToggleClass(class.into()),
        });
        self
    }

    /// Replaces text content on the target element.
    pub fn set_text(mut self, target: impl Into<ElementId>, text: impl Into<String>) -> Self {
        self.operations.push(ElementUpdate {
            target: target.into(),
            action: ElementUpdateAction::SetText(text.into()),
        });
        self
    }

    /// Replaces form/control value data on the target element.
    pub fn set_value(mut self, target: impl Into<ElementId>, value: impl Into<String>) -> Self {
        self.operations.push(ElementUpdate {
            target: target.into(),
            action: ElementUpdateAction::SetValue(value.into()),
        });
        self
    }

    /// Sets the authored selected state used by state selectors.
    pub fn set_selected(mut self, target: impl Into<ElementId>, selected: bool) -> Self {
        self.operations.push(ElementUpdate {
            target: target.into(),
            action: ElementUpdateAction::SetSelected(selected),
        });
        self
    }

    /// Sets the authored disabled state used by state selectors and hit testing.
    pub fn set_disabled(mut self, target: impl Into<ElementId>, disabled: bool) -> Self {
        self.operations.push(ElementUpdate {
            target: target.into(),
            action: ElementUpdateAction::SetDisabled(disabled),
        });
        self
    }

    /// Sets the authored focused state used by state selectors.
    pub fn set_focused(mut self, target: impl Into<ElementId>, focused: bool) -> Self {
        self.operations.push(ElementUpdate {
            target: target.into(),
            action: ElementUpdateAction::SetFocused(focused),
        });
        self
    }

    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }
}

#[derive(Clone, Debug, PartialEq)]
struct ElementUpdate {
    target: ElementId,
    action: ElementUpdateAction,
}

#[derive(Clone, Debug, PartialEq)]
enum ElementUpdateAction {
    AddClass(ClassName),
    RemoveClass(ClassName),
    ToggleClass(ClassName),
    SetText(String),
    SetValue(String),
    SetSelected(bool),
    SetDisabled(bool),
    SetFocused(bool),
}

/// Summary of how a [`DocumentUpdate`] interacted with a concrete document.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DocumentUpdateReport {
    /// Number of update operations whose target element existed.
    pub matched: usize,
    /// Number of matched operations that changed document data.
    pub changed: usize,
    /// Target ids for update operations that could not be applied.
    pub missing_targets: Vec<ElementId>,
}

impl Document {
    /// Applies service/app requested document changes before layout.
    pub fn apply_update(&mut self, update: &DocumentUpdate) -> DocumentUpdateReport {
        let mut report = DocumentUpdateReport::default();
        for operation in &update.operations {
            let Some(element) = self.root.find_mut(&operation.target) else {
                report.missing_targets.push(operation.target.clone());
                continue;
            };

            report.matched += 1;
            if apply_element_update(element, &operation.action) {
                report.changed += 1;
            }
        }
        report
    }

    /// Returns an updated document with the corresponding update report.
    pub fn with_update(mut self, update: &DocumentUpdate) -> (Self, DocumentUpdateReport) {
        let report = self.apply_update(update);
        (self, report)
    }
}

impl Element {
    fn find_mut(&mut self, id: &ElementId) -> Option<&mut Self> {
        if &self.id == id {
            return Some(self);
        }
        for child in &mut self.children {
            if let Some(found) = child.find_mut(id) {
                return Some(found);
            }
        }
        None
    }
}

fn apply_element_update(element: &mut Element, action: &ElementUpdateAction) -> bool {
    match action {
        ElementUpdateAction::AddClass(class) => add_class(element, class),
        ElementUpdateAction::RemoveClass(class) => remove_class(element, class),
        ElementUpdateAction::ToggleClass(class) => {
            if remove_class(element, class) {
                true
            } else {
                add_class(element, class)
            }
        }
        ElementUpdateAction::SetText(text) => set_option_string(&mut element.text, text),
        ElementUpdateAction::SetValue(value) => set_option_string(&mut element.spec.value, value),
        ElementUpdateAction::SetSelected(selected) => {
            set_bool(&mut element.spec.selected, *selected)
        }
        ElementUpdateAction::SetDisabled(disabled) => {
            set_bool(&mut element.spec.disabled, *disabled)
        }
        ElementUpdateAction::SetFocused(focused) => set_bool(&mut element.spec.focused, *focused),
    }
}

fn add_class(element: &mut Element, class: &ClassName) -> bool {
    if element
        .spec
        .classes
        .iter()
        .any(|existing| existing == class)
    {
        return false;
    }
    element.spec.classes.push(class.clone());
    true
}

fn remove_class(element: &mut Element, class: &ClassName) -> bool {
    let original_len = element.spec.classes.len();
    element.spec.classes.retain(|existing| existing != class);
    element.spec.classes.len() != original_len
}

fn set_option_string(target: &mut Option<String>, value: &str) -> bool {
    if target.as_deref() == Some(value) {
        return false;
    }
    *target = Some(value.to_owned());
    true
}

fn set_bool(target: &mut bool, value: bool) -> bool {
    if *target == value {
        return false;
    }
    *target = value;
    true
}
