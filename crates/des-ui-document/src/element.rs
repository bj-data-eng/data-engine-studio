use crate::geometry::Size;
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ElementRole {
    Root,
    Panel,
    Card,
    Text,
    Canvas,
    Control,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ElementStateSelector {
    Hovered,
    Pressed,
    ScrollbarHovered,
    ScrollbarDragged,
    Focused,
    Selected,
    Disabled,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub(crate) fn lerp(self, target: Self, amount: f32) -> Self {
        fn channel(from: u8, to: u8, amount: f32) -> u8 {
            (from as f32 + (to as f32 - from as f32) * amount)
                .round()
                .clamp(0.0, 255.0) as u8
        }

        Self {
            r: channel(self.r, target.r, amount),
            g: channel(self.g, target.g, amount),
            b: channel(self.b, target.b, amount),
            a: channel(self.a, target.a, amount),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ElementId(String);

impl ElementId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for ElementId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for ElementId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ClassName(String);

impl ClassName {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for ClassName {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for ClassName {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ElementSpec {
    pub role: ElementRole,
    pub classes: Vec<ClassName>,
    pub interactive: bool,
    pub selected: bool,
    pub disabled: bool,
    pub focused: bool,
}

impl ElementSpec {
    pub fn new(role: ElementRole) -> Self {
        Self {
            role,
            classes: Vec::new(),
            interactive: false,
            selected: false,
            disabled: false,
            focused: false,
        }
    }

    pub fn class(mut self, class: impl Into<ClassName>) -> Self {
        self.classes.push(class.into());
        self
    }

    pub fn interactive(mut self) -> Self {
        self.interactive = true;
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Element {
    pub id: ElementId,
    pub spec: ElementSpec,
    pub text: Option<String>,
    pub children: Vec<Element>,
}

impl Element {
    pub(crate) fn collect_ids(&self, ids: &mut BTreeSet<ElementId>) {
        ids.insert(self.id.clone());
        for child in &self.children {
            child.collect_ids(ids);
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Document {
    pub viewport: Size,
    pub root: Element,
}

impl Document {
    pub fn build(viewport: Size, add_contents: impl FnOnce(&mut DocumentBuilder)) -> Self {
        let mut ui = DocumentBuilder::default();
        add_contents(&mut ui);
        Self {
            viewport,
            root: Element {
                id: ElementId::new("root"),
                spec: ElementSpec::new(ElementRole::Root),
                text: None,
                children: ui.children,
            },
        }
    }
}

#[derive(Default)]
pub struct DocumentBuilder {
    children: Vec<Element>,
}

impl DocumentBuilder {
    pub fn element(
        &mut self,
        id: impl Into<ElementId>,
        spec: ElementSpec,
        add_contents: impl FnOnce(&mut DocumentBuilder),
    ) {
        let mut child_ui = DocumentBuilder::default();
        add_contents(&mut child_ui);
        self.children.push(Element {
            id: id.into(),
            spec,
            text: None,
            children: child_ui.children,
        });
    }

    pub fn text(&mut self, id: impl Into<ElementId>, text: impl Into<String>) {
        self.text_element(id, ElementSpec::new(ElementRole::Text), text);
    }

    pub fn text_element(
        &mut self,
        id: impl Into<ElementId>,
        spec: ElementSpec,
        text: impl Into<String>,
    ) {
        self.children.push(Element {
            id: id.into(),
            spec,
            text: Some(text.into()),
            children: Vec::new(),
        });
    }
}
