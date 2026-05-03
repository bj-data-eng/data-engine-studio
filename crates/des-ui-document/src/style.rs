use crate::element::{ClassName, Color, Element, ElementId, ElementRole, ElementStateSelector};
use crate::geometry::{
    AlignItems, CornerRadii, Direction, Insets, JustifyContent, Length, Overflow, Point, Position,
    PositionInsets, Size,
};
use crate::state::ElementState;
use crate::text::TextWrapMode;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StyleSelector {
    Role(ElementRole),
    Class(ClassName),
    Id(ElementId),
    State(ElementStateSelector),
    FirstChild,
    LastChild,
    NthChild(usize),
    ClassState(ClassName, ElementStateSelector),
    IdState(ElementId, ElementStateSelector),
    Compound(CompoundSelector),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ChildPosition {
    pub index: usize,
    pub sibling_count: usize,
}

impl ChildPosition {
    pub fn new(index: usize, sibling_count: usize) -> Self {
        Self {
            index,
            sibling_count,
        }
    }

    pub fn is_first(self) -> bool {
        self.index == 0
    }

    pub fn is_last(self) -> bool {
        self.index + 1 == self.sibling_count
    }

    pub fn is_nth(self, nth: usize) -> bool {
        nth > 0 && self.index + 1 == nth
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChildPositionSelector {
    First,
    Last,
    Nth(usize),
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CompoundSelector {
    pub(crate) role: Option<ElementRole>,
    pub(crate) id: Option<ElementId>,
    pub(crate) classes: Vec<ClassName>,
    pub(crate) states: Vec<ElementStateSelector>,
    pub(crate) child_position: Option<ChildPositionSelector>,
}

impl StyleSelector {
    pub fn class(class: impl Into<ClassName>) -> Self {
        Self::Class(class.into())
    }

    pub fn id(id: impl Into<ElementId>) -> Self {
        Self::Id(id.into())
    }

    pub fn class_state(class: impl Into<ClassName>, state: ElementStateSelector) -> Self {
        Self::ClassState(class.into(), state)
    }

    pub fn id_state(id: impl Into<ElementId>, state: ElementStateSelector) -> Self {
        Self::IdState(id.into(), state)
    }

    pub fn first_child() -> Self {
        Self::FirstChild
    }

    pub fn last_child() -> Self {
        Self::LastChild
    }

    pub fn nth_child(nth: usize) -> Self {
        Self::NthChild(nth)
    }

    pub fn compound() -> CompoundSelector {
        CompoundSelector::default()
    }
}

impl CompoundSelector {
    pub fn role(mut self, role: ElementRole) -> Self {
        self.role = Some(role);
        self
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn class(mut self, class: impl Into<ClassName>) -> Self {
        self.classes.push(class.into());
        self
    }

    pub fn state(mut self, state: ElementStateSelector) -> Self {
        self.states.push(state);
        self
    }

    pub fn first_child(mut self) -> Self {
        self.child_position = Some(ChildPositionSelector::First);
        self
    }

    pub fn last_child(mut self) -> Self {
        self.child_position = Some(ChildPositionSelector::Last);
        self
    }

    pub fn nth_child(mut self, nth: usize) -> Self {
        self.child_position = Some(ChildPositionSelector::Nth(nth));
        self
    }

    pub fn selector(self) -> StyleSelector {
        StyleSelector::Compound(self)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Easing {
    Linear,
    EaseOutCubic,
}

impl Easing {
    pub(crate) fn sample(self, amount: f32) -> f32 {
        let amount = amount.clamp(0.0, 1.0);
        match self {
            Self::Linear => amount,
            Self::EaseOutCubic => 1.0 - (1.0 - amount).powi(3),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transition {
    pub step: f32,
    pub easing: Easing,
}

impl Transition {
    pub fn ease_out(step: f32) -> Self {
        Self {
            step,
            easing: Easing::EaseOutCubic,
        }
    }

    pub fn linear(step: f32) -> Self {
        Self {
            step,
            easing: Easing::Linear,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AnchorPlacement {
    TopStart,
    TopEnd,
    BottomStart,
    BottomEnd,
    LeftStart,
    LeftEnd,
    RightStart,
    RightEnd,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Anchor {
    pub target: ElementId,
    pub placement: AnchorPlacement,
    pub offset: Point,
}

impl Anchor {
    pub fn new(target: impl Into<ElementId>, placement: AnchorPlacement, offset: Point) -> Self {
        Self {
            target: target.into(),
            placement,
            offset,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Style {
    pub direction: Option<Direction>,
    pub wrap: Option<bool>,
    pub align_items: Option<AlignItems>,
    pub justify_content: Option<JustifyContent>,
    pub gap: Option<f32>,
    pub margin: Option<Insets>,
    pub padding: Option<Insets>,
    pub width: Option<Length>,
    pub height: Option<Length>,
    pub min_size: Option<Size>,
    pub max_size: Option<Size>,
    pub background: Option<Color>,
    pub border: Option<Color>,
    pub border_width: EdgeStyle,
    pub text_color: Option<Color>,
    pub font_size: Option<f32>,
    pub text_wrap: Option<TextWrapMode>,
    pub max_lines: Option<usize>,
    pub line_height: Option<f32>,
    pub radius: CornerStyle,
    pub overflow_x: Option<Overflow>,
    pub overflow_y: Option<Overflow>,
    pub scrollbar_width: Option<f32>,
    pub scrollbar_expanded_width: Option<f32>,
    pub scrollbar_handle_color: Option<Color>,
    pub scrollbar_track_color: Option<Color>,
    pub scrollbar_handle_border_color: Option<Color>,
    pub scrollbar_handle_border_width: Option<f32>,
    pub scrollbar_hover_handle_color: Option<Color>,
    pub scrollbar_hover_track_color: Option<Color>,
    pub scrollbar_hover_handle_border_color: Option<Color>,
    pub scrollbar_hover_handle_border_width: Option<f32>,
    pub scrollbar_pressed_handle_color: Option<Color>,
    pub scrollbar_pressed_track_color: Option<Color>,
    pub scrollbar_pressed_handle_border_color: Option<Color>,
    pub scrollbar_pressed_handle_border_width: Option<f32>,
    pub scrollbar_radius: Option<f32>,
    pub position: Option<Position>,
    pub inset: PositionInsets,
    pub anchor: Option<Anchor>,
    pub z_index: Option<i32>,
    pub transition: Option<Transition>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct EdgeStyle {
    pub top: Option<f32>,
    pub right: Option<f32>,
    pub bottom: Option<f32>,
    pub left: Option<f32>,
}

impl EdgeStyle {
    pub fn all(value: f32) -> Self {
        Self {
            top: Some(value),
            right: Some(value),
            bottom: Some(value),
            left: Some(value),
        }
    }

    pub fn from_insets(insets: Insets) -> Self {
        Self {
            top: Some(insets.top),
            right: Some(insets.right),
            bottom: Some(insets.bottom),
            left: Some(insets.left),
        }
    }

    fn is_empty(self) -> bool {
        self.top.is_none() && self.right.is_none() && self.bottom.is_none() && self.left.is_none()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CornerStyle {
    pub top_left: Option<f32>,
    pub top_right: Option<f32>,
    pub bottom_right: Option<f32>,
    pub bottom_left: Option<f32>,
}

impl CornerStyle {
    pub fn all(value: f32) -> Self {
        Self {
            top_left: Some(value),
            top_right: Some(value),
            bottom_right: Some(value),
            bottom_left: Some(value),
        }
    }

    pub fn from_radii(radii: CornerRadii) -> Self {
        Self {
            top_left: Some(radii.top_left),
            top_right: Some(radii.top_right),
            bottom_right: Some(radii.bottom_right),
            bottom_left: Some(radii.bottom_left),
        }
    }
}

impl Style {
    pub fn direction(mut self, direction: Direction) -> Self {
        self.direction = Some(direction);
        self
    }

    pub fn wrap(mut self, wrap: bool) -> Self {
        self.wrap = Some(wrap);
        self
    }

    pub fn align_items(mut self, align_items: AlignItems) -> Self {
        self.align_items = Some(align_items);
        self
    }

    pub fn justify_content(mut self, justify_content: JustifyContent) -> Self {
        self.justify_content = Some(justify_content);
        self
    }

    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = Some(gap);
        self
    }

    pub fn margin(mut self, margin: Insets) -> Self {
        self.margin = Some(margin);
        self
    }

    pub fn padding(mut self, padding: Insets) -> Self {
        self.padding = Some(padding);
        self
    }

    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width = Some(Length::Px(width));
        self.height = Some(Length::Px(height));
        self
    }

    pub fn width(mut self, width: Length) -> Self {
        self.width = Some(width);
        self
    }

    pub fn height(mut self, height: Length) -> Self {
        self.height = Some(height);
        self
    }

    pub fn width_fill(mut self) -> Self {
        self.width = Some(Length::Fill);
        self
    }

    pub fn height_fill(mut self) -> Self {
        self.height = Some(Length::Fill);
        self
    }

    pub fn width_percent(mut self, factor: f32) -> Self {
        self.width = Some(Length::Percent(factor));
        self
    }

    pub fn height_percent(mut self, factor: f32) -> Self {
        self.height = Some(Length::Percent(factor));
        self
    }

    pub fn min_size(mut self, width: f32, height: f32) -> Self {
        self.min_size = Some(Size::new(width, height));
        self
    }

    pub fn max_size(mut self, width: f32, height: f32) -> Self {
        self.max_size = Some(Size::new(width, height));
        self
    }

    pub fn background(mut self, color: Color) -> Self {
        self.background = Some(color);
        self
    }

    pub fn border(mut self, color: Color) -> Self {
        self.border = Some(color);
        if self.border_width.is_empty() {
            self.border_width = EdgeStyle::all(1.0);
        }
        self
    }

    pub fn border_width(mut self, width: f32) -> Self {
        self.border_width = EdgeStyle::all(width);
        self
    }

    pub fn border_widths(mut self, widths: Insets) -> Self {
        self.border_width = EdgeStyle::from_insets(widths);
        self
    }

    pub fn border_top_width(mut self, width: f32) -> Self {
        self.border_width.top = Some(width);
        self
    }

    pub fn border_right_width(mut self, width: f32) -> Self {
        self.border_width.right = Some(width);
        self
    }

    pub fn border_bottom_width(mut self, width: f32) -> Self {
        self.border_width.bottom = Some(width);
        self
    }

    pub fn border_left_width(mut self, width: f32) -> Self {
        self.border_width.left = Some(width);
        self
    }

    pub fn text_color(mut self, color: Color) -> Self {
        self.text_color = Some(color);
        self
    }

    pub fn font_size(mut self, font_size: f32) -> Self {
        self.font_size = Some(font_size);
        self
    }

    pub fn text_wrap(mut self, wrap_mode: TextWrapMode) -> Self {
        self.text_wrap = Some(wrap_mode);
        self
    }

    pub fn max_lines(mut self, max_lines: usize) -> Self {
        self.max_lines = Some(max_lines.max(1));
        self
    }

    pub fn line_height(mut self, line_height: f32) -> Self {
        self.line_height = Some(line_height.max(1.0));
        self
    }

    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = CornerStyle::all(radius);
        self
    }

    pub fn radii(mut self, radii: CornerRadii) -> Self {
        self.radius = CornerStyle::from_radii(radii);
        self
    }

    pub fn top_left_radius(mut self, radius: f32) -> Self {
        self.radius.top_left = Some(radius);
        self
    }

    pub fn top_right_radius(mut self, radius: f32) -> Self {
        self.radius.top_right = Some(radius);
        self
    }

    pub fn bottom_right_radius(mut self, radius: f32) -> Self {
        self.radius.bottom_right = Some(radius);
        self
    }

    pub fn bottom_left_radius(mut self, radius: f32) -> Self {
        self.radius.bottom_left = Some(radius);
        self
    }

    pub fn overflow_y(mut self, overflow: Overflow) -> Self {
        self.overflow_y = Some(overflow);
        self
    }

    pub fn overflow_x(mut self, overflow: Overflow) -> Self {
        self.overflow_x = Some(overflow);
        self
    }

    pub fn overflow(mut self, overflow: Overflow) -> Self {
        self.overflow_x = Some(overflow);
        self.overflow_y = Some(overflow);
        self
    }

    pub fn scrollbar_width(mut self, width: f32) -> Self {
        self.scrollbar_width = Some(width.max(0.0));
        self
    }

    pub fn scrollbar_expanded_width(mut self, width: f32) -> Self {
        self.scrollbar_expanded_width = Some(width.max(0.0));
        self
    }

    pub fn scrollbar_handle_color(mut self, color: Color) -> Self {
        self.scrollbar_handle_color = Some(color);
        self
    }

    pub fn scrollbar_track_color(mut self, color: Color) -> Self {
        self.scrollbar_track_color = Some(color);
        self
    }

    pub fn scrollbar_handle_border_color(mut self, color: Color) -> Self {
        self.scrollbar_handle_border_color = Some(color);
        self
    }

    pub fn scrollbar_handle_border_width(mut self, width: f32) -> Self {
        self.scrollbar_handle_border_width = Some(width.max(0.0));
        self
    }

    pub fn scrollbar_hover_handle_color(mut self, color: Color) -> Self {
        self.scrollbar_hover_handle_color = Some(color);
        self
    }

    pub fn scrollbar_hover_track_color(mut self, color: Color) -> Self {
        self.scrollbar_hover_track_color = Some(color);
        self
    }

    pub fn scrollbar_hover_handle_border_color(mut self, color: Color) -> Self {
        self.scrollbar_hover_handle_border_color = Some(color);
        self
    }

    pub fn scrollbar_hover_handle_border_width(mut self, width: f32) -> Self {
        self.scrollbar_hover_handle_border_width = Some(width.max(0.0));
        self
    }

    pub fn scrollbar_pressed_handle_color(mut self, color: Color) -> Self {
        self.scrollbar_pressed_handle_color = Some(color);
        self
    }

    pub fn scrollbar_pressed_track_color(mut self, color: Color) -> Self {
        self.scrollbar_pressed_track_color = Some(color);
        self
    }

    pub fn scrollbar_pressed_handle_border_color(mut self, color: Color) -> Self {
        self.scrollbar_pressed_handle_border_color = Some(color);
        self
    }

    pub fn scrollbar_pressed_handle_border_width(mut self, width: f32) -> Self {
        self.scrollbar_pressed_handle_border_width = Some(width.max(0.0));
        self
    }

    pub fn scrollbar_radius(mut self, radius: f32) -> Self {
        self.scrollbar_radius = Some(radius.max(0.0));
        self
    }

    pub fn position(mut self, position: Position) -> Self {
        self.position = Some(position);
        self
    }

    pub fn absolute_parent(mut self) -> Self {
        self.position = Some(Position::AbsoluteParent);
        self
    }

    pub fn absolute_viewport(mut self) -> Self {
        self.position = Some(Position::AbsoluteViewport);
        self
    }

    pub fn inset(mut self, inset: PositionInsets) -> Self {
        self.inset = inset;
        self
    }

    pub fn inset_px(mut self, top: f32, right: f32, bottom: f32, left: f32) -> Self {
        self.inset = PositionInsets::from_insets(Insets {
            top,
            right,
            bottom,
            left,
        });
        self
    }

    pub fn top(mut self, top: Length) -> Self {
        self.inset.top = Some(top);
        self
    }

    pub fn right(mut self, right: Length) -> Self {
        self.inset.right = Some(right);
        self
    }

    pub fn bottom(mut self, bottom: Length) -> Self {
        self.inset.bottom = Some(bottom);
        self
    }

    pub fn left(mut self, left: Length) -> Self {
        self.inset.left = Some(left);
        self
    }

    pub fn anchor(mut self, anchor: Anchor) -> Self {
        self.anchor = Some(anchor);
        self
    }

    pub fn anchor_bottom_start(
        mut self,
        target: impl Into<ElementId>,
        offset_x: f32,
        offset_y: f32,
    ) -> Self {
        self.anchor = Some(Anchor::new(
            target,
            AnchorPlacement::BottomStart,
            Point::new(offset_x, offset_y),
        ));
        self
    }

    pub fn z_index(mut self, z_index: i32) -> Self {
        self.z_index = Some(z_index);
        self
    }

    pub fn transition(mut self, transition: Transition) -> Self {
        self.transition = Some(transition);
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ComputedStyle {
    pub direction: Direction,
    pub wrap: bool,
    pub align_items: AlignItems,
    pub justify_content: JustifyContent,
    pub gap: f32,
    pub margin: Insets,
    pub padding: Insets,
    pub width: Length,
    pub height: Length,
    pub min_size: Size,
    pub max_size: Size,
    pub background: Option<Color>,
    pub border: Option<Color>,
    pub border_width: Insets,
    pub text_color: Color,
    pub font_size: f32,
    pub text_wrap: TextWrapMode,
    pub max_lines: Option<usize>,
    pub line_height: Option<f32>,
    pub radius: CornerRadii,
    pub overflow_x: Overflow,
    pub overflow_y: Overflow,
    pub scrollbar_width: f32,
    pub scrollbar_expanded_width: f32,
    pub scrollbar_handle_color: Color,
    pub scrollbar_track_color: Option<Color>,
    pub scrollbar_handle_border_color: Option<Color>,
    pub scrollbar_handle_border_width: f32,
    pub scrollbar_hover_handle_color: Option<Color>,
    pub scrollbar_hover_track_color: Option<Color>,
    pub scrollbar_hover_handle_border_color: Option<Color>,
    pub scrollbar_hover_handle_border_width: Option<f32>,
    pub scrollbar_pressed_handle_color: Option<Color>,
    pub scrollbar_pressed_track_color: Option<Color>,
    pub scrollbar_pressed_handle_border_color: Option<Color>,
    pub scrollbar_pressed_handle_border_width: Option<f32>,
    pub scrollbar_radius: f32,
    pub position: Position,
    pub inset: PositionInsets,
    pub anchor: Option<Anchor>,
    pub z_index: i32,
    pub transition: Option<Transition>,
}

impl Default for ComputedStyle {
    fn default() -> Self {
        Self {
            direction: Direction::Column,
            wrap: false,
            align_items: AlignItems::Start,
            justify_content: JustifyContent::Start,
            gap: 0.0,
            margin: Insets::ZERO,
            padding: Insets::ZERO,
            width: Length::Auto,
            height: Length::Auto,
            min_size: Size::new(0.0, 0.0),
            max_size: Size::new(f32::INFINITY, f32::INFINITY),
            background: None,
            border: None,
            border_width: Insets::ZERO,
            text_color: Color::rgb(218, 226, 234),
            font_size: 13.0,
            text_wrap: TextWrapMode::Extend,
            max_lines: None,
            line_height: None,
            radius: CornerRadii::ZERO,
            overflow_x: Overflow::Visible,
            overflow_y: Overflow::Visible,
            scrollbar_width: 2.0,
            scrollbar_expanded_width: 10.0,
            scrollbar_handle_color: Color::rgba(232, 236, 240, 118),
            scrollbar_track_color: None,
            scrollbar_handle_border_color: None,
            scrollbar_handle_border_width: 0.0,
            scrollbar_hover_handle_color: None,
            scrollbar_hover_track_color: None,
            scrollbar_hover_handle_border_color: None,
            scrollbar_hover_handle_border_width: None,
            scrollbar_pressed_handle_color: None,
            scrollbar_pressed_track_color: None,
            scrollbar_pressed_handle_border_color: None,
            scrollbar_pressed_handle_border_width: None,
            scrollbar_radius: 6.0,
            position: Position::Flow,
            inset: PositionInsets::ZERO,
            anchor: None,
            z_index: 0,
            transition: None,
        }
    }
}

impl ComputedStyle {
    pub(crate) fn apply(&mut self, style: &Style) {
        if let Some(value) = style.direction {
            self.direction = value;
        }
        if let Some(value) = style.wrap {
            self.wrap = value;
        }
        if let Some(value) = style.align_items {
            self.align_items = value;
        }
        if let Some(value) = style.justify_content {
            self.justify_content = value;
        }
        if let Some(value) = style.gap {
            self.gap = value;
        }
        if let Some(value) = style.margin {
            self.margin = value;
        }
        if let Some(value) = style.padding {
            self.padding = value;
        }
        if let Some(value) = style.width {
            self.width = value;
        }
        if let Some(value) = style.height {
            self.height = value;
        }
        if let Some(value) = style.min_size {
            self.min_size = value;
        }
        if let Some(value) = style.max_size {
            self.max_size = value;
        }
        if let Some(value) = style.background {
            self.background = Some(value);
        }
        if let Some(value) = style.border {
            self.border = Some(value);
        }
        if let Some(value) = style.border_width.top {
            self.border_width.top = value.max(0.0);
        }
        if let Some(value) = style.border_width.right {
            self.border_width.right = value.max(0.0);
        }
        if let Some(value) = style.border_width.bottom {
            self.border_width.bottom = value.max(0.0);
        }
        if let Some(value) = style.border_width.left {
            self.border_width.left = value.max(0.0);
        }
        if let Some(value) = style.text_color {
            self.text_color = value;
        }
        if let Some(value) = style.font_size {
            self.font_size = value;
        }
        if let Some(value) = style.text_wrap {
            self.text_wrap = value;
        }
        if let Some(value) = style.max_lines {
            self.max_lines = Some(value.max(1));
        }
        if let Some(value) = style.line_height {
            self.line_height = Some(value.max(1.0));
        }
        if let Some(value) = style.radius.top_left {
            self.radius.top_left = value.max(0.0);
        }
        if let Some(value) = style.radius.top_right {
            self.radius.top_right = value.max(0.0);
        }
        if let Some(value) = style.radius.bottom_right {
            self.radius.bottom_right = value.max(0.0);
        }
        if let Some(value) = style.radius.bottom_left {
            self.radius.bottom_left = value.max(0.0);
        }
        if let Some(value) = style.overflow_y {
            self.overflow_y = value;
        }
        if let Some(value) = style.overflow_x {
            self.overflow_x = value;
        }
        if let Some(value) = style.scrollbar_width {
            self.scrollbar_width = value.max(0.0);
        }
        if let Some(value) = style.scrollbar_expanded_width {
            self.scrollbar_expanded_width = value.max(0.0);
        }
        if let Some(value) = style.scrollbar_handle_color {
            self.scrollbar_handle_color = value;
        }
        if let Some(value) = style.scrollbar_track_color {
            self.scrollbar_track_color = Some(value);
        }
        if let Some(value) = style.scrollbar_handle_border_color {
            self.scrollbar_handle_border_color = Some(value);
        }
        if let Some(value) = style.scrollbar_handle_border_width {
            self.scrollbar_handle_border_width = value.max(0.0);
        }
        if let Some(value) = style.scrollbar_hover_handle_color {
            self.scrollbar_hover_handle_color = Some(value);
        }
        if let Some(value) = style.scrollbar_hover_track_color {
            self.scrollbar_hover_track_color = Some(value);
        }
        if let Some(value) = style.scrollbar_hover_handle_border_color {
            self.scrollbar_hover_handle_border_color = Some(value);
        }
        if let Some(value) = style.scrollbar_hover_handle_border_width {
            self.scrollbar_hover_handle_border_width = Some(value.max(0.0));
        }
        if let Some(value) = style.scrollbar_pressed_handle_color {
            self.scrollbar_pressed_handle_color = Some(value);
        }
        if let Some(value) = style.scrollbar_pressed_track_color {
            self.scrollbar_pressed_track_color = Some(value);
        }
        if let Some(value) = style.scrollbar_pressed_handle_border_color {
            self.scrollbar_pressed_handle_border_color = Some(value);
        }
        if let Some(value) = style.scrollbar_pressed_handle_border_width {
            self.scrollbar_pressed_handle_border_width = Some(value.max(0.0));
        }
        if let Some(value) = style.scrollbar_radius {
            self.scrollbar_radius = value.max(0.0);
        }
        if let Some(value) = style.position {
            self.position = value;
        }
        if let Some(value) = style.inset.top {
            self.inset.top = Some(value);
        }
        if let Some(value) = style.inset.right {
            self.inset.right = Some(value);
        }
        if let Some(value) = style.inset.bottom {
            self.inset.bottom = Some(value);
        }
        if let Some(value) = style.inset.left {
            self.inset.left = Some(value);
        }
        if let Some(value) = &style.anchor {
            self.anchor = Some(value.clone());
        }
        if let Some(value) = style.z_index {
            self.z_index = value;
        }
        if let Some(value) = style.transition {
            self.transition = Some(value);
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct StyleRule {
    pub(crate) selector: StyleSelector,
    pub(crate) style: Style,
}

impl StyleRule {
    pub fn new(selector: StyleSelector, style: Style) -> Self {
        Self { selector, style }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct StyleSheet {
    pub(crate) rules: Vec<StyleRule>,
}

impl StyleSheet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn rule(mut self, selector: StyleSelector, style: Style) -> Self {
        self.rules.push(StyleRule::new(selector, style));
        self
    }

    pub fn push_rule(&mut self, selector: StyleSelector, style: Style) {
        self.rules.push(StyleRule::new(selector, style));
    }
}

pub(crate) fn resolve_style_with_position(
    element: &Element,
    stylesheet: &StyleSheet,
    state: Option<&ElementState>,
    position: Option<ChildPosition>,
) -> ComputedStyle {
    let mut style = ComputedStyle::default();

    for rule in &stylesheet.rules {
        if selector_matches(&rule.selector, element, state, position) {
            style.apply(&rule.style);
        }
    }

    style
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct StyleInvalidation {
    pub paint_changed: bool,
    pub layout_changed: bool,
}

impl StyleInvalidation {
    pub(crate) fn changed(self) -> bool {
        self.paint_changed || self.layout_changed
    }
}

impl std::ops::AddAssign for StyleInvalidation {
    fn add_assign(&mut self, rhs: Self) {
        self.paint_changed |= rhs.paint_changed;
        self.layout_changed |= rhs.layout_changed;
    }
}

pub(crate) fn classify_computed_style_change(
    previous: Option<&ComputedStyle>,
    next: Option<&ComputedStyle>,
) -> StyleInvalidation {
    match (previous, next) {
        (Some(previous), Some(next)) if previous == next => StyleInvalidation::default(),
        (Some(previous), Some(next)) => StyleInvalidation {
            paint_changed: true,
            layout_changed: layout_relevant_style_changed(previous, next),
        },
        (None, Some(_)) => StyleInvalidation {
            paint_changed: true,
            layout_changed: false,
        },
        (Some(_), None) => StyleInvalidation {
            paint_changed: true,
            layout_changed: true,
        },
        (None, None) => StyleInvalidation::default(),
    }
}

fn layout_relevant_style_changed(previous: &ComputedStyle, next: &ComputedStyle) -> bool {
    previous.direction != next.direction
        || previous.wrap != next.wrap
        || previous.align_items != next.align_items
        || previous.justify_content != next.justify_content
        || previous.gap != next.gap
        || previous.margin != next.margin
        || previous.padding != next.padding
        || previous.width != next.width
        || previous.height != next.height
        || previous.min_size != next.min_size
        || previous.max_size != next.max_size
        || previous.border_width != next.border_width
        || previous.font_size != next.font_size
        || previous.text_wrap != next.text_wrap
        || previous.max_lines != next.max_lines
        || previous.line_height != next.line_height
        || previous.overflow_x != next.overflow_x
        || previous.overflow_y != next.overflow_y
        || previous.position != next.position
        || previous.inset != next.inset
        || previous.anchor != next.anchor
}

fn selector_matches(
    selector: &StyleSelector,
    element: &Element,
    state: Option<&ElementState>,
    position: Option<ChildPosition>,
) -> bool {
    match selector {
        StyleSelector::Role(role) => element.spec.role == *role,
        StyleSelector::Class(class) => element
            .spec
            .classes
            .iter()
            .any(|element_class| element_class == class),
        StyleSelector::Id(id) => &element.id == id,
        StyleSelector::State(selector) => state_selector_matches(*selector, element, state),
        StyleSelector::FirstChild => position.is_some_and(ChildPosition::is_first),
        StyleSelector::LastChild => position.is_some_and(ChildPosition::is_last),
        StyleSelector::NthChild(nth) => position.is_some_and(|position| position.is_nth(*nth)),
        StyleSelector::ClassState(class, selector) => {
            element
                .spec
                .classes
                .iter()
                .any(|element_class| element_class == class)
                && state_selector_matches(*selector, element, state)
        }
        StyleSelector::IdState(id, selector) => {
            &element.id == id && state_selector_matches(*selector, element, state)
        }
        StyleSelector::Compound(selector) => {
            compound_selector_matches(selector, element, state, position)
        }
    }
}

fn compound_selector_matches(
    selector: &CompoundSelector,
    element: &Element,
    state: Option<&ElementState>,
    position: Option<ChildPosition>,
) -> bool {
    if selector.role.is_some_and(|role| element.spec.role != role) {
        return false;
    }
    if selector.id.as_ref().is_some_and(|id| &element.id != id) {
        return false;
    }
    if !selector.classes.iter().all(|class| {
        element
            .spec
            .classes
            .iter()
            .any(|element_class| element_class == class)
    }) {
        return false;
    }

    if !selector
        .states
        .iter()
        .all(|selector| state_selector_matches(*selector, element, state))
    {
        return false;
    }

    if selector
        .child_position
        .is_some_and(|selector| !child_position_selector_matches(selector, position))
    {
        return false;
    }

    true
}

fn child_position_selector_matches(
    selector: ChildPositionSelector,
    position: Option<ChildPosition>,
) -> bool {
    let Some(position) = position else {
        return false;
    };
    match selector {
        ChildPositionSelector::First => position.is_first(),
        ChildPositionSelector::Last => position.is_last(),
        ChildPositionSelector::Nth(nth) => position.is_nth(nth),
    }
}

fn state_selector_matches(
    selector: ElementStateSelector,
    element: &Element,
    state: Option<&ElementState>,
) -> bool {
    match selector {
        ElementStateSelector::Hovered => state.is_some_and(|state| state.hovered),
        ElementStateSelector::Pressed => state.is_some_and(|state| state.pressed),
        ElementStateSelector::Dragged => state.is_some_and(|state| state.dragging),
        ElementStateSelector::ScrollbarHovered => {
            state.is_some_and(|state| state.scrollbar_hovered_axis.is_some())
        }
        ElementStateSelector::ScrollbarDragged => {
            state.is_some_and(|state| state.scrollbar_dragged_axis.is_some())
        }
        ElementStateSelector::Focused => {
            element.spec.focused || state.is_some_and(|state| state.focused)
        }
        ElementStateSelector::Selected => element.spec.selected,
        ElementStateSelector::Disabled => element.spec.disabled,
    }
}
