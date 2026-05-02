#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    pub origin: Point,
    pub size: Size,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            origin: Point::new(x, y),
            size: Size::new(width, height),
        }
    }

    pub fn right(self) -> f32 {
        self.origin.x + self.size.width
    }

    pub fn bottom(self) -> f32 {
        self.origin.y + self.size.height
    }

    pub fn inset(self, insets: Insets) -> Self {
        Self::new(
            self.origin.x + insets.left,
            self.origin.y + insets.top,
            (self.size.width - insets.horizontal()).max(0.0),
            (self.size.height - insets.vertical()).max(0.0),
        )
    }

    pub fn contains(self, point: Point) -> bool {
        point.x >= self.origin.x
            && point.x <= self.right()
            && point.y >= self.origin.y
            && point.y <= self.bottom()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Insets {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl Insets {
    pub const ZERO: Self = Self {
        top: 0.0,
        right: 0.0,
        bottom: 0.0,
        left: 0.0,
    };

    pub fn all(value: f32) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    pub fn symmetric(horizontal: f32, vertical: f32) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }

    pub fn horizontal(self) -> f32 {
        self.left + self.right
    }

    pub fn vertical(self) -> f32 {
        self.top + self.bottom
    }

    pub fn is_uniform(self) -> bool {
        self.top == self.right && self.top == self.bottom && self.top == self.left
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CornerRadii {
    pub top_left: f32,
    pub top_right: f32,
    pub bottom_right: f32,
    pub bottom_left: f32,
}

impl CornerRadii {
    pub const ZERO: Self = Self {
        top_left: 0.0,
        top_right: 0.0,
        bottom_right: 0.0,
        bottom_left: 0.0,
    };

    pub fn all(value: f32) -> Self {
        Self {
            top_left: value,
            top_right: value,
            bottom_right: value,
            bottom_left: value,
        }
    }

    pub fn corners(top_left: f32, top_right: f32, bottom_right: f32, bottom_left: f32) -> Self {
        Self {
            top_left,
            top_right,
            bottom_right,
            bottom_left,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Direction {
    Row,
    Column,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AlignItems {
    Start,
    Center,
    End,
    Stretch,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JustifyContent {
    Start,
    Center,
    End,
    SpaceBetween,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Overflow {
    Visible,
    Scroll,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScrollAxis {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Position {
    Flow,
    AbsoluteParent,
    AbsoluteViewport,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PositionInsets {
    pub top: Option<Length>,
    pub right: Option<Length>,
    pub bottom: Option<Length>,
    pub left: Option<Length>,
}

impl PositionInsets {
    pub const ZERO: Self = Self {
        top: None,
        right: None,
        bottom: None,
        left: None,
    };

    pub fn top_left(top: Length, left: Length) -> Self {
        Self {
            top: Some(top),
            right: None,
            bottom: None,
            left: Some(left),
        }
    }

    pub fn from_insets(insets: Insets) -> Self {
        Self {
            top: Some(Length::Px(insets.top)),
            right: Some(Length::Px(insets.right)),
            bottom: Some(Length::Px(insets.bottom)),
            left: Some(Length::Px(insets.left)),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Length {
    Auto,
    Px(f32),
    Fill,
    Percent(f32),
}

impl Length {
    pub(crate) fn resolve(self, available: f32, auto: f32) -> f32 {
        match self {
            Self::Auto => auto,
            Self::Px(value) => value,
            Self::Fill => available,
            Self::Percent(factor) => available * factor,
        }
        .max(0.0)
    }

    pub(crate) fn resolve_intrinsic(self, available: f32, auto: f32) -> f32 {
        match self {
            Self::Fill => auto,
            Self::Percent(factor) => available * factor,
            Self::Auto => auto,
            Self::Px(value) => value,
        }
        .max(0.0)
    }
}
