use crate::element::{ClassName, Color, Element, ElementId, ElementRole, ElementStateSelector};
use crate::geometry::{
    CornerRadii, Direction, Insets, Length, Overflow, Position, PositionInsets, Size,
};
use crate::state::ElementState;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StyleSelector {
    Role(ElementRole),
    Class(ClassName),
    Id(ElementId),
    State(ElementStateSelector),
    ClassState(ClassName, ElementStateSelector),
    IdState(ElementId, ElementStateSelector),
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

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Style {
    pub direction: Option<Direction>,
    pub wrap: Option<bool>,
    pub gap: Option<f32>,
    pub margin: Option<Insets>,
    pub padding: Option<Insets>,
    pub width: Option<Length>,
    pub height: Option<Length>,
    pub min_size: Option<Size>,
    pub background: Option<Color>,
    pub border: Option<Color>,
    pub border_width: EdgeStyle,
    pub text_color: Option<Color>,
    pub font_size: Option<f32>,
    pub radius: CornerStyle,
    pub overflow_y: Option<Overflow>,
    pub position: Option<Position>,
    pub inset: PositionInsets,
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
    pub gap: f32,
    pub margin: Insets,
    pub padding: Insets,
    pub width: Length,
    pub height: Length,
    pub min_size: Size,
    pub background: Option<Color>,
    pub border: Option<Color>,
    pub border_width: Insets,
    pub text_color: Color,
    pub font_size: f32,
    pub radius: CornerRadii,
    pub overflow_y: Overflow,
    pub position: Position,
    pub inset: PositionInsets,
    pub z_index: i32,
    pub transition: Option<Transition>,
}

impl Default for ComputedStyle {
    fn default() -> Self {
        Self {
            direction: Direction::Column,
            wrap: false,
            gap: 0.0,
            margin: Insets::ZERO,
            padding: Insets::ZERO,
            width: Length::Auto,
            height: Length::Auto,
            min_size: Size::new(0.0, 0.0),
            background: None,
            border: None,
            border_width: Insets::ZERO,
            text_color: Color::rgb(218, 226, 234),
            font_size: 13.0,
            radius: CornerRadii::ZERO,
            overflow_y: Overflow::Visible,
            position: Position::Flow,
            inset: PositionInsets::ZERO,
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

pub(crate) fn resolve_style(
    element: &Element,
    stylesheet: &StyleSheet,
    state: Option<&ElementState>,
) -> ComputedStyle {
    let mut style = ComputedStyle::default();

    for rule in &stylesheet.rules {
        if selector_matches(&rule.selector, element, state) {
            style.apply(&rule.style);
        }
    }

    style
}

fn selector_matches(
    selector: &StyleSelector,
    element: &Element,
    state: Option<&ElementState>,
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
        ElementStateSelector::Focused => {
            element.spec.focused || state.is_some_and(|state| state.focused)
        }
        ElementStateSelector::Selected => element.spec.selected,
        ElementStateSelector::Disabled => element.spec.disabled,
    }
}
