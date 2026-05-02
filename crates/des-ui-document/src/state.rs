use crate::element::{ClassName, Color, ElementId, ElementRole};
use crate::geometry::{Point, Rect};
use crate::style::ComputedStyle;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ElementState {
    pub scroll_y: f32,
    pub hovered: bool,
    pub pressed: bool,
    pub scrollbar_hovered: bool,
    pub scrollbar_dragged: bool,
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
    pub scroll_chrome: Vec<ScrollChrome>,
    pub animating: bool,
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
}
