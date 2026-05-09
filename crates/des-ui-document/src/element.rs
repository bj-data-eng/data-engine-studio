use crate::state::ResolvedElement;
use crate::table::{TableCellSpec, TableSpec};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Element {
    Root,
    Div,
    Span,
    Main,
    Section,
    Article,
    Header,
    Footer,
    Nav,
    Aside,
    P,
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
    Text,
    Button,
    Input,
    Checkbox,
    Radio,
    Select,
    Option,
    Textarea,
    Label,
    Canvas,
    Icon,
    Table,
    Thead,
    Tbody,
    Tr,
    Th,
    Td,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Glyph {
    Check,
    ChevronDown,
    ChevronUp,
    DragHandle,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ElementStateSelector {
    Hovered,
    Pressed,
    Dragged,
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
    pub element: Element,
    pub classes: Vec<ClassName>,
    pub interactive: bool,
    pub selected: bool,
    pub disabled: bool,
    pub focused: bool,
    pub selectable_text: bool,
    pub copyable_text: bool,
    pub value: Option<String>,
    pub glyph: Option<Glyph>,
    pub table: Option<TableSpec>,
    pub table_cell: Option<TableCellSpec>,
}

impl ElementSpec {
    pub fn new(element: Element) -> Self {
        Self {
            element,
            classes: Vec::new(),
            interactive: false,
            selected: false,
            disabled: false,
            focused: false,
            selectable_text: false,
            copyable_text: false,
            value: None,
            glyph: None,
            table: None,
            table_cell: None,
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

    pub fn selectable_text(mut self) -> Self {
        self.selectable_text = true;
        self.copyable_text = true;
        self
    }

    pub fn copyable_text(mut self, copyable_text: bool) -> Self {
        self.copyable_text = copyable_text;
        self
    }

    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    pub fn glyph(mut self, glyph: Glyph) -> Self {
        self.glyph = Some(glyph);
        self
    }

    pub fn table(mut self, table: TableSpec) -> Self {
        self.table = Some(table);
        self
    }

    pub fn table_cell(mut self, table_cell: TableCellSpec) -> Self {
        self.table_cell = Some(table_cell);
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct DocumentNode {
    pub id: ElementId,
    pub spec: ElementSpec,
    pub text: Option<String>,
    pub children: Vec<DocumentNode>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct VisualElementClone {
    pub source_id: ElementId,
    pub element: Element,
    pub classes: Vec<ClassName>,
    pub text: Option<String>,
    pub value: Option<String>,
    pub glyph: Option<Glyph>,
    pub children: Vec<VisualElementClone>,
}

impl VisualElementClone {
    pub fn from_resolved(element: &ResolvedElement) -> Self {
        Self {
            source_id: element.id.clone(),
            element: element.element,
            classes: element.classes.clone(),
            text: element.text.clone(),
            value: element.value.clone(),
            glyph: element.glyph,
            children: element.children.iter().map(Self::from_resolved).collect(),
        }
    }

    pub fn cloned_ids(&self, options: &VisualCloneOptions) -> Vec<ElementId> {
        let mut ids = Vec::new();
        self.collect_cloned_ids(options, true, &mut ids);
        ids
    }

    pub fn source_ids(&self) -> Vec<ElementId> {
        let mut ids = Vec::new();
        self.collect_source_ids(&mut ids);
        ids
    }

    fn collect_cloned_ids(
        &self,
        options: &VisualCloneOptions,
        is_root: bool,
        ids: &mut Vec<ElementId>,
    ) {
        ids.push(self.clone_id(options, is_root));
        for child in &self.children {
            child.collect_cloned_ids(options, false, ids);
        }
    }

    fn collect_source_ids(&self, ids: &mut Vec<ElementId>) {
        ids.push(self.source_id.clone());
        for child in &self.children {
            child.collect_source_ids(ids);
        }
    }

    pub(crate) fn to_element(&self, options: &VisualCloneOptions, is_root: bool) -> DocumentNode {
        let mut spec = ElementSpec::new(self.element);
        spec.classes = self.classes.clone();
        if is_root {
            spec.classes.extend(options.root_classes.iter().cloned());
        }
        spec.value = self.value.clone();
        spec.glyph = self.glyph;
        spec.interactive = options.interactive;

        DocumentNode {
            id: self.clone_id(options, is_root),
            spec,
            text: self.text.clone(),
            children: self
                .children
                .iter()
                .map(|child| child.to_element(options, false))
                .collect(),
        }
    }

    fn clone_id(&self, options: &VisualCloneOptions, is_root: bool) -> ElementId {
        if is_root {
            return options.root_id.clone();
        }
        ElementId::new(format!("{}{}", options.id_prefix, self.source_id.as_str()))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct VisualCloneOptions {
    pub root_id: ElementId,
    pub id_prefix: String,
    pub root_classes: Vec<ClassName>,
    pub interactive: bool,
}

impl VisualCloneOptions {
    pub fn new(root_id: impl Into<ElementId>, id_prefix: impl Into<String>) -> Self {
        Self {
            root_id: root_id.into(),
            id_prefix: id_prefix.into(),
            root_classes: Vec::new(),
            interactive: false,
        }
    }

    pub fn root_class(mut self, class: impl Into<ClassName>) -> Self {
        self.root_classes.push(class.into());
        self
    }

    pub fn interactive(mut self, interactive: bool) -> Self {
        self.interactive = interactive;
        self
    }
}
