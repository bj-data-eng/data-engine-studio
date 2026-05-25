use crate::{ClassName, Document, DocumentError, DocumentResult, ElementId, TextContent};

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

impl DocumentProjection {
    pub fn new() -> Self {
        Self::default()
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

    pub fn set_selected(mut self, id: impl Into<ElementId>, selected: bool) -> Self {
        self.push_selected(id, selected);
        self
    }

    pub fn push_selected(&mut self, id: impl Into<ElementId>, selected: bool) {
        self.operations
            .push(DocumentProjectionOperation::SetSelected {
                id: id.into(),
                selected,
            });
    }

    pub fn set_disabled(mut self, id: impl Into<ElementId>, disabled: bool) -> Self {
        self.push_disabled(id, disabled);
        self
    }

    pub fn push_disabled(&mut self, id: impl Into<ElementId>, disabled: bool) {
        self.operations
            .push(DocumentProjectionOperation::SetDisabled {
                id: id.into(),
                disabled,
            });
    }

    pub fn set_focused(mut self, id: impl Into<ElementId>, focused: bool) -> Self {
        self.push_focused(id, focused);
        self
    }

    pub fn push_focused(&mut self, id: impl Into<ElementId>, focused: bool) {
        self.operations
            .push(DocumentProjectionOperation::SetFocused {
                id: id.into(),
                focused,
            });
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

    pub fn operations(&self) -> &[DocumentProjectionOperation] {
        &self.operations
    }

    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }

    pub fn apply_to(&self, document: &mut Document) -> DocumentResult<DocumentProjectionReport> {
        let mut report = DocumentProjectionReport {
            operations: self.operations.len(),
            changed: 0,
        };
        for operation in &self.operations {
            if operation.apply_to(document)? {
                report.changed += 1;
            }
        }
        Ok(report)
    }
}

impl DocumentProjectionOperation {
    pub fn target(&self) -> &ElementId {
        match self {
            Self::SetText { id, .. }
            | Self::SetValue { id, .. }
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
