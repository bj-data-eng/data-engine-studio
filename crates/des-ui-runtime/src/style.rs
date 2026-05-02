use crate::element::{Color, ElementRole, ElementStateSelector};
use crate::geometry::{Direction, Insets, Length, Overflow, Size};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StyleSelector {
    Role(ElementRole),
    Class(&'static str),
    Id(&'static str),
    State(ElementStateSelector),
    ClassState(&'static str, ElementStateSelector),
    IdState(&'static str, ElementStateSelector),
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
pub struct StylePatch {
    pub direction: Option<Direction>,
    pub gap: Option<f32>,
    pub margin: Option<Insets>,
    pub padding: Option<Insets>,
    pub width: Option<Length>,
    pub height: Option<Length>,
    pub min_size: Option<Size>,
    pub background: Option<Color>,
    pub border: Option<Color>,
    pub border_width: Option<f32>,
    pub text_color: Option<Color>,
    pub font_size: Option<f32>,
    pub radius: Option<f32>,
    pub overflow_y: Option<Overflow>,
    pub z_index: Option<i32>,
    pub transition: Option<Transition>,
}

impl StylePatch {
    pub fn direction(mut self, direction: Direction) -> Self {
        self.direction = Some(direction);
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
        if self.border_width.is_none() {
            self.border_width = Some(1.0);
        }
        self
    }

    pub fn border_width(mut self, width: f32) -> Self {
        self.border_width = Some(width);
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
        self.radius = Some(radius);
        self
    }

    pub fn overflow_y(mut self, overflow: Overflow) -> Self {
        self.overflow_y = Some(overflow);
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
    pub gap: f32,
    pub margin: Insets,
    pub padding: Insets,
    pub width: Length,
    pub height: Length,
    pub min_size: Size,
    pub background: Option<Color>,
    pub border: Option<Color>,
    pub border_width: f32,
    pub text_color: Color,
    pub font_size: f32,
    pub radius: f32,
    pub overflow_y: Overflow,
    pub z_index: i32,
    pub transition: Option<Transition>,
}

impl Default for ComputedStyle {
    fn default() -> Self {
        Self {
            direction: Direction::Column,
            gap: 0.0,
            margin: Insets::ZERO,
            padding: Insets::ZERO,
            width: Length::Auto,
            height: Length::Auto,
            min_size: Size::new(0.0, 0.0),
            background: None,
            border: None,
            border_width: 0.0,
            text_color: Color::rgb(218, 226, 234),
            font_size: 13.0,
            radius: 0.0,
            overflow_y: Overflow::Visible,
            z_index: 0,
            transition: None,
        }
    }
}

impl ComputedStyle {
    pub(crate) fn apply(&mut self, patch: &StylePatch) {
        if let Some(value) = patch.direction {
            self.direction = value;
        }
        if let Some(value) = patch.gap {
            self.gap = value;
        }
        if let Some(value) = patch.margin {
            self.margin = value;
        }
        if let Some(value) = patch.padding {
            self.padding = value;
        }
        if let Some(value) = patch.width {
            self.width = value;
        }
        if let Some(value) = patch.height {
            self.height = value;
        }
        if let Some(value) = patch.min_size {
            self.min_size = value;
        }
        if let Some(value) = patch.background {
            self.background = Some(value);
        }
        if let Some(value) = patch.border {
            self.border = Some(value);
        }
        if let Some(value) = patch.border_width {
            self.border_width = value.max(0.0);
        }
        if let Some(value) = patch.text_color {
            self.text_color = value;
        }
        if let Some(value) = patch.font_size {
            self.font_size = value;
        }
        if let Some(value) = patch.radius {
            self.radius = value;
        }
        if let Some(value) = patch.overflow_y {
            self.overflow_y = value;
        }
        if let Some(value) = patch.z_index {
            self.z_index = value;
        }
        if let Some(value) = patch.transition {
            self.transition = Some(value);
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct StyleRule {
    pub(crate) selector: StyleSelector,
    pub(crate) patch: StylePatch,
}

impl StyleRule {
    pub fn new(selector: StyleSelector, patch: StylePatch) -> Self {
        Self { selector, patch }
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

    pub fn rule(mut self, selector: StyleSelector, patch: StylePatch) -> Self {
        self.rules.push(StyleRule::new(selector, patch));
        self
    }

    pub fn push_rule(&mut self, selector: StyleSelector, patch: StylePatch) {
        self.rules.push(StyleRule::new(selector, patch));
    }
}
