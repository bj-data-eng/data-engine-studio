use crate::element::{ClassName, Color, ElementId, ElementRole, Glyph};
use crate::geometry::{Point, Rect, ScrollAxis};
use crate::query::DocumentSnapshot;
use crate::style::{ComputedStyle, Transition};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ElementState {
    pub scroll_x: f32,
    pub scroll_y: f32,
    pub hovered: bool,
    pub pressed: bool,
    pub scrollbar_hovered_axis: Option<ScrollAxis>,
    pub scrollbar_dragged_axis: Option<ScrollAxis>,
    pub(crate) scrollbar_visual_width_x: Option<f32>,
    pub(crate) scrollbar_visual_width_y: Option<f32>,
    pub focused: bool,
    pub click_count: u32,
    pub(crate) rendered_style: Option<ComputedStyle>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ChangeSet {
    pub created: Vec<ElementId>,
    pub retained: Vec<ElementId>,
    pub removed: Vec<ElementId>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedElement {
    pub id: ElementId,
    pub role: ElementRole,
    pub classes: Vec<ClassName>,
    pub rect: Rect,
    pub style: ComputedStyle,
    pub text: Option<String>,
    pub value: Option<String>,
    pub glyph: Option<Glyph>,
    pub interactive: bool,
    pub children: Vec<ResolvedElement>,
}

impl ResolvedElement {
    pub fn find(&self, id: &str) -> Option<&Self> {
        if self.id.as_str() == id {
            return Some(self);
        }
        self.children.iter().find_map(|child| child.find(id))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DocumentOutput {
    pub changes: ChangeSet,
    pub layout: ResolvedElement,
    pub hit_id: Option<ElementId>,
    pub events: Vec<DocumentEvent>,
    pub scroll_chrome: Vec<ScrollChrome>,
    pub animating: bool,
    pub metrics: DocumentMetrics,
}

impl DocumentOutput {
    pub fn snapshot(&self) -> DocumentSnapshot<'_> {
        DocumentSnapshot::new(&self.layout)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentEvent {
    pub target: ElementId,
    pub kind: DocumentEventKind,
}

impl DocumentEvent {
    pub fn new(target: impl Into<ElementId>, kind: DocumentEventKind) -> Self {
        Self {
            target: target.into(),
            kind,
        }
    }

    pub fn pointer_entered(target: impl Into<ElementId>) -> Self {
        Self::new(target, DocumentEventKind::PointerEntered)
    }

    pub fn pointer_exited(target: impl Into<ElementId>) -> Self {
        Self::new(target, DocumentEventKind::PointerExited)
    }

    pub fn pressed(target: impl Into<ElementId>) -> Self {
        Self::new(target, DocumentEventKind::Pressed)
    }

    pub fn released(target: impl Into<ElementId>) -> Self {
        Self::new(target, DocumentEventKind::Released)
    }

    pub fn clicked(target: impl Into<ElementId>) -> Self {
        Self::new(target, DocumentEventKind::Clicked)
    }

    pub fn scrolled(target: impl Into<ElementId>, axis: ScrollAxis) -> Self {
        Self::new(target, DocumentEventKind::Scrolled(axis))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocumentEventKind {
    PointerEntered,
    PointerExited,
    Pressed,
    Released,
    Clicked,
    Scrolled(ScrollAxis),
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DocumentMetrics {
    pub element_count: usize,
    pub scroll_chrome_count: usize,
    pub reused_cached_layout: bool,
    pub reused_input_layout: bool,
    pub input_changed_state: bool,
    pub animation_changed_style: bool,
    pub animation_changed_layout: bool,
    pub animation_changed_paint: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DocumentInput {
    pub pointer: Option<PointerInput>,
    pub scroll_delta: Point,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointerInput {
    pub position: Point,
    pub primary_delta: Point,
    pub primary_down: bool,
    pub primary_clicked: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScrollChrome {
    pub element_id: ElementId,
    pub axis: ScrollAxis,
    pub track_rect: Rect,
    pub hit_rect: Rect,
    pub handle_rect: Rect,
    pub handle_color: Color,
    pub track_color: Option<Color>,
    pub handle_border_color: Option<Color>,
    pub handle_border_width: f32,
    pub radius: f32,
    pub max_scroll: f32,
    pub visible: bool,
    pub expanded: bool,
    pub hovered: bool,
    pub dragged: bool,
    pub(crate) compact_visual_width: f32,
    pub(crate) expanded_visual_width: f32,
    pub(crate) transition: Option<Transition>,
}
