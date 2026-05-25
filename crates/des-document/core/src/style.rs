use crate::element::{ClassName, Color, DocumentNode, Element, ElementId, ElementStateSelector};
use crate::geometry::{
    AlignContent, AlignItems, AlignSelf, CornerRadii, FlexDirection, FlexWrap, Insets,
    JustifyContent, Length, Overflow, Point, Position, PositionInsets, Size,
};
use crate::state::ElementState;
use crate::text::{
    OverflowWrap, TextAlign, TextLayoutStyle, TextOverflow, TextTransform, TextWrapMode,
    WhiteSpace, WhiteSpaceCollapse, WordBreak,
};
pub use des_layout::prelude::{
    Display, GridAutoFlow, MaxTrackSizingFunction, MinTrackSizingFunction, RepetitionCount,
    TrackSizingFunction,
};
pub use des_layout::style::Direction;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

pub use des_layout::floating::{
    FloatingArrow, FloatingArrowData, FloatingAutoPlacement, FloatingAxisOffset, FloatingBoundary,
    FloatingFallbackAxisSideDirection, FloatingFallbackStrategy, FloatingFlip,
    FloatingFlipCrossAxis, FloatingHide, FloatingHideData, FloatingHideStrategy, FloatingInline,
    FloatingOffset, FloatingOptions, FloatingPlacement, FloatingShift, FloatingShiftLimiter,
    FloatingSize, FloatingVisibility,
};

pub type GridPlacement = des_layout::prelude::GridPlacement<String>;
pub type GridPlacementLine = des_layout::geometry::Line<GridPlacement>;
pub type GridTemplateArea = des_layout::style::GridTemplateArea<String>;
pub type GridTemplateComponent = des_layout::prelude::GridTemplateComponent<String>;
pub type GridTemplateRepetition = des_layout::style::GridTemplateRepetition<String>;
pub type GridTrack = TrackSizingFunction;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StyleSelector {
    Element(Element),
    Class(ClassName),
    Id(ElementId),
    State(ElementStateSelector),
    FirstChild,
    LastChild,
    NthChild(usize),
    NthChildFormula(NthChildFormula),
    ClassState(ClassName, ElementStateSelector),
    IdState(ElementId, ElementStateSelector),
    Compound(CompoundSelector),
    Descendant(Vec<CompoundSelector>),
    Complex(ComplexSelector),
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ViewportQuery {
    pub min_width: Option<f32>,
    pub max_width: Option<f32>,
    pub min_height: Option<f32>,
    pub max_height: Option<f32>,
}

impl ViewportQuery {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn min_width(width: f32) -> Self {
        Self::new().with_min_width(width)
    }

    pub fn max_width(width: f32) -> Self {
        Self::new().with_max_width(width)
    }

    pub fn min_height(height: f32) -> Self {
        Self::new().with_min_height(height)
    }

    pub fn max_height(height: f32) -> Self {
        Self::new().with_max_height(height)
    }

    pub fn with_min_width(mut self, width: f32) -> Self {
        self.min_width = Some(width);
        self
    }

    pub fn with_max_width(mut self, width: f32) -> Self {
        self.max_width = Some(width);
        self
    }

    pub fn with_min_height(mut self, height: f32) -> Self {
        self.min_height = Some(height);
        self
    }

    pub fn with_max_height(mut self, height: f32) -> Self {
        self.max_height = Some(height);
        self
    }

    pub(crate) fn matches(self, viewport: Size) -> bool {
        self.min_width
            .is_none_or(|min_width| viewport.width >= min_width)
            && self
                .max_width
                .is_none_or(|max_width| viewport.width <= max_width)
            && self
                .min_height
                .is_none_or(|min_height| viewport.height >= min_height)
            && self
                .max_height
                .is_none_or(|max_height| viewport.height <= max_height)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ContainerQuery {
    pub min_width: Option<f32>,
    pub max_width: Option<f32>,
    pub min_height: Option<f32>,
    pub max_height: Option<f32>,
}

impl ContainerQuery {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn min_width(width: f32) -> Self {
        Self::new().with_min_width(width)
    }

    pub fn max_width(width: f32) -> Self {
        Self::new().with_max_width(width)
    }

    pub fn min_height(height: f32) -> Self {
        Self::new().with_min_height(height)
    }

    pub fn max_height(height: f32) -> Self {
        Self::new().with_max_height(height)
    }

    pub fn with_min_width(mut self, width: f32) -> Self {
        self.min_width = Some(width);
        self
    }

    pub fn with_max_width(mut self, width: f32) -> Self {
        self.max_width = Some(width);
        self
    }

    pub fn with_min_height(mut self, height: f32) -> Self {
        self.min_height = Some(height);
        self
    }

    pub fn with_max_height(mut self, height: f32) -> Self {
        self.max_height = Some(height);
        self
    }

    pub(crate) fn matches(self, container: Option<Size>) -> bool {
        let Some(container) = container else {
            return false;
        };
        self.min_width
            .is_none_or(|min_width| container.width >= min_width)
            && self
                .max_width
                .is_none_or(|max_width| container.width <= max_width)
            && self
                .min_height
                .is_none_or(|min_height| container.height >= min_height)
            && self
                .max_height
                .is_none_or(|max_height| container.height <= max_height)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StyleCondition {
    Viewport(ViewportQuery),
    Container(ContainerQuery),
}

impl StyleCondition {
    pub fn viewport(query: ViewportQuery) -> Self {
        Self::Viewport(query)
    }

    pub fn container(query: ContainerQuery) -> Self {
        Self::Container(query)
    }

    pub(crate) fn matches(self, viewport: Size, container: Option<Size>) -> bool {
        match self {
            Self::Viewport(query) => query.matches(viewport),
            Self::Container(query) => query.matches(container),
        }
    }

    pub(crate) fn is_container(self) -> bool {
        matches!(self, Self::Container(_))
    }
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

    pub fn matches_nth_formula(self, formula: NthChildFormula) -> bool {
        formula.matches(self.index + 1)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NthChildFormula {
    pub step: usize,
    pub offset: usize,
}

impl NthChildFormula {
    pub fn new(step: usize, offset: usize) -> Self {
        Self { step, offset }
    }

    pub fn odd() -> Self {
        Self::new(2, 1)
    }

    pub fn even() -> Self {
        Self::new(2, 0)
    }

    pub fn matches(self, position: usize) -> bool {
        if position == 0 {
            return false;
        }
        if self.step == 0 {
            return self.offset > 0 && position == self.offset;
        }
        if self.offset == 0 {
            return position.is_multiple_of(self.step);
        }
        position >= self.offset && (position - self.offset).is_multiple_of(self.step)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChildPositionSelector {
    First,
    Last,
    Nth(usize),
    NthFormula(NthChildFormula),
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CompoundSelector {
    pub(crate) element: Option<Element>,
    pub(crate) id: Option<ElementId>,
    pub(crate) classes: Vec<ClassName>,
    pub(crate) states: Vec<ElementStateSelector>,
    pub(crate) child_position: Option<ChildPositionSelector>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectorCombinator {
    Descendant,
    Child,
    AdjacentSibling,
    GeneralSibling,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComplexSelector {
    pub(crate) parts: Vec<ComplexSelectorPart>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComplexSelectorPart {
    pub(crate) combinator: Option<SelectorCombinator>,
    pub(crate) selector: CompoundSelector,
}

impl StyleSelector {
    pub fn element(element: Element) -> Self {
        Self::Element(element)
    }

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

    pub fn nth_child_formula(step: usize, offset: usize) -> Self {
        Self::NthChildFormula(NthChildFormula::new(step, offset))
    }

    pub fn nth_child_odd() -> Self {
        Self::NthChildFormula(NthChildFormula::odd())
    }

    pub fn nth_child_even() -> Self {
        Self::NthChildFormula(NthChildFormula::even())
    }

    pub fn compound() -> CompoundSelector {
        CompoundSelector::default()
    }
}

impl ComplexSelector {
    pub(crate) fn new(parts: Vec<ComplexSelectorPart>) -> Self {
        Self { parts }
    }
}

impl ComplexSelectorPart {
    pub(crate) fn root(selector: CompoundSelector) -> Self {
        Self {
            combinator: None,
            selector,
        }
    }

    pub(crate) fn related(combinator: SelectorCombinator, selector: CompoundSelector) -> Self {
        Self {
            combinator: Some(combinator),
            selector,
        }
    }
}

impl CompoundSelector {
    pub fn element(mut self, element: Element) -> Self {
        self.element = Some(element);
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

    pub fn nth_child_formula(mut self, step: usize, offset: usize) -> Self {
        self.child_position = Some(ChildPositionSelector::NthFormula(NthChildFormula::new(
            step, offset,
        )));
        self
    }

    pub fn nth_child_odd(mut self) -> Self {
        self.child_position = Some(ChildPositionSelector::NthFormula(NthChildFormula::odd()));
        self
    }

    pub fn nth_child_even(mut self) -> Self {
        self.child_position = Some(ChildPositionSelector::NthFormula(NthChildFormula::even()));
        self
    }

    pub fn selector(self) -> StyleSelector {
        StyleSelector::Compound(self)
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct StyleMatchContext<'a> {
    pub element: &'a DocumentNode,
    pub state: Option<&'a ElementState>,
    pub position: Option<ChildPosition>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct StyleResolutionContext<'a> {
    pub element: &'a DocumentNode,
    pub state: Option<&'a ElementState>,
    pub position: Option<ChildPosition>,
    pub ancestors: &'a [StyleMatchContext<'a>],
    pub previous_siblings: &'a [StyleMatchContext<'a>],
    pub viewport: Size,
    pub container: Option<Size>,
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

pub type AnchorPlacement = FloatingPlacement;

#[derive(Clone, Debug, PartialEq)]
pub struct Anchor {
    pub target: ElementId,
    pub options: FloatingOptions,
    pub boundary_target: Option<ElementId>,
}

impl Anchor {
    pub fn new(target: impl Into<ElementId>, placement: AnchorPlacement, offset: Point) -> Self {
        Self {
            target: target.into(),
            options: FloatingOptions::new(placement).offset(
                legacy_main_axis_offset(placement, offset),
                legacy_cross_axis_offset(placement, offset),
            ),
            boundary_target: None,
        }
    }

    pub fn floating(target: impl Into<ElementId>) -> Self {
        Self {
            target: target.into(),
            options: FloatingOptions::new(FloatingPlacement::BottomStart),
            boundary_target: None,
        }
    }

    pub fn with_options(target: impl Into<ElementId>, options: FloatingOptions) -> Self {
        Self {
            target: target.into(),
            options,
            boundary_target: None,
        }
    }

    pub fn placement(mut self, placement: FloatingPlacement) -> Self {
        self.options.placement = placement;
        self
    }

    pub fn offset(mut self, main_axis: f32, cross_axis: f32) -> Self {
        self.options.offset = FloatingOffset::new(main_axis, cross_axis);
        self
    }

    pub fn fallbacks(mut self, fallbacks: impl Into<Vec<FloatingPlacement>>) -> Self {
        self.options.fallbacks = fallbacks.into();
        self
    }

    pub fn flip(mut self, flip: bool) -> Self {
        self.options = self.options.flip(flip);
        self
    }

    pub fn flip_options(mut self, flip: FloatingFlip) -> Self {
        self.options = self.options.flip_options(flip);
        self
    }

    pub fn auto_placement(mut self, auto_placement: FloatingAutoPlacement) -> Self {
        self.options = self.options.auto_placement(auto_placement);
        self
    }

    pub fn shift(mut self, shift: FloatingShift) -> Self {
        self.options.shift = Some(shift);
        self
    }

    pub fn size(mut self, size: FloatingSize) -> Self {
        self.options.size = Some(size);
        self
    }

    pub fn arrow(mut self, arrow: FloatingArrow) -> Self {
        self.options.arrow = Some(arrow);
        self
    }

    pub fn hide(mut self, strategy: FloatingHideStrategy) -> Self {
        self.options.hide.push(FloatingHide::new(strategy));
        self
    }

    pub fn hide_options(mut self, hide: FloatingHide) -> Self {
        self.options.hide.push(hide);
        self
    }

    pub fn inline(mut self, inline: FloatingInline) -> Self {
        self.options.inline = Some(inline);
        self
    }

    pub fn boundary_to(mut self, boundary_target: impl Into<ElementId>) -> Self {
        self.boundary_target = Some(boundary_target.into());
        self
    }

    pub fn rtl(mut self, rtl: bool) -> Self {
        self.options.rtl = rtl;
        self
    }
}

fn legacy_main_axis_offset(placement: AnchorPlacement, offset: Point) -> f32 {
    match placement.side() {
        des_layout::floating::FloatingSide::Top => -offset.y,
        des_layout::floating::FloatingSide::Right => offset.x,
        des_layout::floating::FloatingSide::Bottom => offset.y,
        des_layout::floating::FloatingSide::Left => -offset.x,
    }
}

fn legacy_cross_axis_offset(placement: AnchorPlacement, offset: Point) -> f32 {
    match placement.side() {
        des_layout::floating::FloatingSide::Top | des_layout::floating::FloatingSide::Bottom => {
            offset.x
        }
        des_layout::floating::FloatingSide::Right | des_layout::floating::FloatingSide::Left => {
            offset.y
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Style {
    pub display: Option<Display>,
    pub direction: Option<Direction>,
    pub flex_direction: Option<FlexDirection>,
    pub flex_wrap: Option<FlexWrap>,
    pub flex_basis: Option<Length>,
    pub flex_grow: Option<f32>,
    pub flex_shrink: Option<f32>,
    pub align_content: Option<AlignContent>,
    pub align_items: Option<AlignItems>,
    pub align_self: Option<AlignSelf>,
    pub justify_items: Option<AlignItems>,
    pub justify_self: Option<AlignSelf>,
    pub justify_content: Option<JustifyContent>,
    pub gap: Option<Length>,
    pub row_gap: Option<Length>,
    pub column_gap: Option<Length>,
    pub grid_template_rows: Option<Vec<GridTemplateComponent>>,
    pub grid_template_columns: Option<Vec<GridTemplateComponent>>,
    pub grid_auto_rows: Option<Vec<GridTrack>>,
    pub grid_auto_columns: Option<Vec<GridTrack>>,
    pub grid_auto_flow: Option<GridAutoFlow>,
    pub grid_template_areas: Option<Vec<GridTemplateArea>>,
    pub grid_template_column_names: Option<Vec<Vec<String>>>,
    pub grid_template_row_names: Option<Vec<Vec<String>>>,
    pub grid_row: Option<GridPlacementLine>,
    pub grid_column: Option<GridPlacementLine>,
    pub margin: Option<Insets>,
    pub padding: Option<Insets>,
    pub width: Option<Length>,
    pub height: Option<Length>,
    pub min_size: Option<Size>,
    pub max_size: Option<Size>,
    pub animate_size: Option<bool>,
    pub background: Option<Color>,
    pub border: Option<Color>,
    pub border_width: EdgeStyle,
    pub border_style: Option<BorderStyle>,
    pub shadows: Option<Vec<Shadow>>,
    pub animate_paint: Option<bool>,
    pub animate_shadows: Option<bool>,
    pub text_color: Option<Color>,
    pub text_selection_background: Option<Color>,
    pub text_selection_color: Option<Color>,
    pub font_size: Option<f32>,
    pub text_layout: Option<TextLayoutStyle>,
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
    pub scrollbar_visible: Option<bool>,
    pub position: Option<Position>,
    pub inset: PositionInsets,
    pub anchor: Option<Anchor>,
    pub z_index: Option<i32>,
    pub transition: Option<Transition>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum BorderStyle {
    #[default]
    Solid,
    Dashed,
    Dotted,
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shadow {
    pub offset: Point,
    pub blur: f32,
    pub spread: f32,
    pub color: Color,
}

impl Default for Shadow {
    fn default() -> Self {
        Self {
            offset: Point::ZERO,
            blur: 0.0,
            spread: 0.0,
            color: Color::rgba(0, 0, 0, 0),
        }
    }
}

impl Style {
    pub fn display(mut self, display: Display) -> Self {
        self.display = Some(display);
        self
    }

    pub fn flex_direction(mut self, flex_direction: FlexDirection) -> Self {
        self.flex_direction = Some(flex_direction);
        self
    }

    pub fn flex_wrap(mut self, flex_wrap: FlexWrap) -> Self {
        self.flex_wrap = Some(flex_wrap);
        self
    }

    pub fn flex_basis(mut self, flex_basis: Length) -> Self {
        self.flex_basis = Some(flex_basis);
        self
    }

    pub fn flex_grow(mut self, flex_grow: f32) -> Self {
        self.flex_grow = Some(flex_grow.max(0.0));
        self
    }

    pub fn flex_shrink(mut self, flex_shrink: f32) -> Self {
        self.flex_shrink = Some(flex_shrink.max(0.0));
        self
    }

    pub fn align_content(mut self, align_content: AlignContent) -> Self {
        self.align_content = Some(align_content);
        self
    }

    pub fn align_items(mut self, align_items: AlignItems) -> Self {
        self.align_items = Some(align_items);
        self
    }

    pub fn align_self(mut self, align_self: AlignSelf) -> Self {
        self.align_self = Some(align_self);
        self
    }

    pub fn justify_items(mut self, justify_items: AlignItems) -> Self {
        self.justify_items = Some(justify_items);
        self
    }

    pub fn justify_self(mut self, justify_self: AlignSelf) -> Self {
        self.justify_self = Some(justify_self);
        self
    }

    pub fn justify_content(mut self, justify_content: JustifyContent) -> Self {
        self.justify_content = Some(justify_content);
        self
    }

    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = Some(Length::Px(gap));
        self.row_gap = Some(Length::Px(gap));
        self.column_gap = Some(Length::Px(gap));
        self
    }

    pub fn gap_length(mut self, gap: Length) -> Self {
        self.gap = Some(gap);
        self.row_gap = Some(gap);
        self.column_gap = Some(gap);
        self
    }

    pub fn gap_percent(self, factor: f32) -> Self {
        self.gap_length(Length::Percent(factor))
    }

    pub fn row_gap(mut self, row_gap: f32) -> Self {
        self.row_gap = Some(Length::Px(row_gap));
        self
    }

    pub fn row_gap_length(mut self, row_gap: Length) -> Self {
        self.row_gap = Some(row_gap);
        self
    }

    pub fn row_gap_percent(self, factor: f32) -> Self {
        self.row_gap_length(Length::Percent(factor))
    }

    pub fn column_gap(mut self, column_gap: f32) -> Self {
        self.column_gap = Some(Length::Px(column_gap));
        self
    }

    pub fn column_gap_length(mut self, column_gap: Length) -> Self {
        self.column_gap = Some(column_gap);
        self
    }

    pub fn column_gap_percent(self, factor: f32) -> Self {
        self.column_gap_length(Length::Percent(factor))
    }

    pub fn grid_template_rows(mut self, grid_template_rows: Vec<GridTemplateComponent>) -> Self {
        self.grid_template_rows = Some(grid_template_rows);
        self
    }

    pub fn grid_template_columns(
        mut self,
        grid_template_columns: Vec<GridTemplateComponent>,
    ) -> Self {
        self.grid_template_columns = Some(grid_template_columns);
        self
    }

    pub fn grid_auto_rows(mut self, grid_auto_rows: Vec<GridTrack>) -> Self {
        self.grid_auto_rows = Some(grid_auto_rows);
        self
    }

    pub fn grid_auto_columns(mut self, grid_auto_columns: Vec<GridTrack>) -> Self {
        self.grid_auto_columns = Some(grid_auto_columns);
        self
    }

    pub fn grid_auto_flow(mut self, grid_auto_flow: GridAutoFlow) -> Self {
        self.grid_auto_flow = Some(grid_auto_flow);
        self
    }

    pub fn grid_template_areas(mut self, grid_template_areas: Vec<GridTemplateArea>) -> Self {
        self.grid_template_areas = Some(grid_template_areas);
        self
    }

    pub fn grid_template_column_names(
        mut self,
        grid_template_column_names: Vec<Vec<String>>,
    ) -> Self {
        self.grid_template_column_names = Some(grid_template_column_names);
        self
    }

    pub fn grid_template_row_names(mut self, grid_template_row_names: Vec<Vec<String>>) -> Self {
        self.grid_template_row_names = Some(grid_template_row_names);
        self
    }

    pub fn grid_row(mut self, start: GridPlacement, end: GridPlacement) -> Self {
        self.grid_row = Some(GridPlacementLine { start, end });
        self
    }

    pub fn grid_column(mut self, start: GridPlacement, end: GridPlacement) -> Self {
        self.grid_column = Some(GridPlacementLine { start, end });
        self
    }

    pub fn grid_row_line(mut self, grid_row: GridPlacementLine) -> Self {
        self.grid_row = Some(grid_row);
        self
    }

    pub fn grid_column_line(mut self, grid_column: GridPlacementLine) -> Self {
        self.grid_column = Some(grid_column);
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

    pub fn animate_size(mut self, animate: bool) -> Self {
        self.animate_size = Some(animate);
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

    pub fn border_style(mut self, style: BorderStyle) -> Self {
        self.border_style = Some(style);
        self
    }

    pub fn border_solid(self) -> Self {
        self.border_style(BorderStyle::Solid)
    }

    pub fn border_dashed(self) -> Self {
        self.border_style(BorderStyle::Dashed)
    }

    pub fn border_dotted(self) -> Self {
        self.border_style(BorderStyle::Dotted)
    }

    pub fn shadow(mut self, shadow: Shadow) -> Self {
        self.shadows = Some(vec![shadow]);
        self
    }

    pub fn shadows(mut self, shadows: impl IntoIterator<Item = Shadow>) -> Self {
        self.shadows = Some(shadows.into_iter().collect());
        self
    }

    pub fn animate_paint(mut self, animate: bool) -> Self {
        self.animate_paint = Some(animate);
        self
    }

    pub fn animate_shadows(mut self, animate: bool) -> Self {
        self.animate_shadows = Some(animate);
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

    pub fn text_selection_background(mut self, color: Color) -> Self {
        self.text_selection_background = Some(color);
        self
    }

    pub fn text_selection_color(mut self, color: Color) -> Self {
        self.text_selection_color = Some(color);
        self
    }

    pub fn font_size(mut self, font_size: f32) -> Self {
        self.font_size = Some(font_size);
        self
    }

    pub fn direction(mut self, direction: Direction) -> Self {
        self.direction = Some(direction);
        self
    }

    pub fn text_layout(mut self, layout: TextLayoutStyle) -> Self {
        self.text_layout = Some(layout);
        self
    }

    pub fn white_space(mut self, white_space: WhiteSpace) -> Self {
        self.text_layout = Some(TextLayoutStyle {
            max_lines: self.text_layout.and_then(|layout| layout.max_lines),
            ..TextLayoutStyle::white_space(white_space)
        });
        self
    }

    pub fn text_wrap_mode(mut self, wrap_mode: TextWrapMode) -> Self {
        let mut layout = self.text_layout.unwrap_or_default();
        layout.text_wrap_mode = wrap_mode;
        self.text_layout = Some(layout);
        self
    }

    pub fn white_space_collapse(mut self, collapse: WhiteSpaceCollapse) -> Self {
        let mut layout = self.text_layout.unwrap_or_default();
        layout.white_space_collapse = collapse;
        self.text_layout = Some(layout);
        self
    }

    pub fn overflow_wrap(mut self, overflow_wrap: OverflowWrap) -> Self {
        let mut layout = self.text_layout.unwrap_or_default();
        layout.overflow_wrap = overflow_wrap;
        self.text_layout = Some(layout);
        self
    }

    pub fn word_break(mut self, word_break: WordBreak) -> Self {
        let mut layout = self.text_layout.unwrap_or_default();
        layout.word_break = word_break;
        self.text_layout = Some(layout);
        self
    }

    pub fn text_align(mut self, text_align: TextAlign) -> Self {
        let mut layout = self.text_layout.unwrap_or_default();
        layout.text_align = text_align;
        self.text_layout = Some(layout);
        self
    }

    pub fn text_overflow(mut self, text_overflow: TextOverflow) -> Self {
        let mut layout = self.text_layout.unwrap_or_default();
        layout.text_overflow = text_overflow;
        self.text_layout = Some(layout);
        self
    }

    pub fn text_transform(mut self, text_transform: TextTransform) -> Self {
        let mut layout = self.text_layout.unwrap_or_default();
        layout.text_transform = text_transform;
        self.text_layout = Some(layout);
        self
    }

    pub fn max_lines(mut self, max_lines: usize) -> Self {
        let mut layout = self.text_layout.unwrap_or_default();
        layout.max_lines = Some(max_lines.max(1));
        self.text_layout = Some(layout);
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

    pub fn scrollbar_visible(mut self, visible: bool) -> Self {
        self.scrollbar_visible = Some(visible);
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

    pub fn floating_to(mut self, target: impl Into<ElementId>) -> Self {
        self.position = Some(Position::AbsoluteViewport);
        self.anchor = Some(Anchor::floating(target));
        self
    }

    pub fn floating_options(
        mut self,
        target: impl Into<ElementId>,
        options: FloatingOptions,
    ) -> Self {
        self.position = Some(Position::AbsoluteViewport);
        self.anchor = Some(Anchor::with_options(target, options));
        self
    }

    pub fn floating_placement(mut self, placement: FloatingPlacement) -> Self {
        if let Some(anchor) = &mut self.anchor {
            anchor.options.placement = placement;
        }
        self
    }

    pub fn floating_offset(mut self, main_axis: f32, cross_axis: f32) -> Self {
        if let Some(anchor) = &mut self.anchor {
            anchor.options.offset = FloatingOffset::new(main_axis, cross_axis);
        }
        self
    }

    pub fn floating_offset_axes(
        mut self,
        main_axis: FloatingAxisOffset,
        cross_axis: FloatingAxisOffset,
    ) -> Self {
        if let Some(anchor) = &mut self.anchor {
            anchor.options.offset = FloatingOffset::from_axes(main_axis, cross_axis);
        }
        self
    }

    pub fn floating_alignment_axis(mut self, alignment_axis: f32) -> Self {
        if let Some(anchor) = &mut self.anchor {
            anchor.options.offset.alignment_axis = Some(FloatingAxisOffset::px(alignment_axis));
        }
        self
    }

    pub fn floating_alignment_axis_offset(mut self, alignment_axis: FloatingAxisOffset) -> Self {
        if let Some(anchor) = &mut self.anchor {
            anchor.options.offset.alignment_axis = Some(alignment_axis);
        }
        self
    }

    pub fn floating_fallbacks(
        mut self,
        fallbacks: impl IntoIterator<Item = FloatingPlacement>,
    ) -> Self {
        if let Some(anchor) = &mut self.anchor {
            anchor.options.fallbacks = fallbacks.into_iter().collect();
        }
        self
    }

    pub fn floating_flip(mut self, flip: bool) -> Self {
        if let Some(anchor) = &mut self.anchor {
            anchor.options = anchor.options.clone().flip(flip);
        }
        self
    }

    pub fn floating_flip_options(mut self, flip: FloatingFlip) -> Self {
        if let Some(anchor) = &mut self.anchor {
            anchor.options = anchor.options.clone().flip_options(flip);
        }
        self
    }

    pub fn floating_auto_placement(mut self, auto_placement: FloatingAutoPlacement) -> Self {
        if let Some(anchor) = &mut self.anchor {
            anchor.options = anchor.options.clone().auto_placement(auto_placement);
        }
        self
    }

    pub fn floating_shift(mut self, shift: FloatingShift) -> Self {
        if let Some(anchor) = &mut self.anchor {
            anchor.options.shift = Some(shift);
        }
        self
    }

    pub fn floating_size(mut self, size: FloatingSize) -> Self {
        if let Some(anchor) = &mut self.anchor {
            anchor.options.size = Some(size);
        }
        self
    }

    pub fn floating_boundary_to(mut self, boundary_target: impl Into<ElementId>) -> Self {
        if let Some(anchor) = &mut self.anchor {
            anchor.boundary_target = Some(boundary_target.into());
        }
        self
    }

    pub fn floating_arrow(mut self, arrow: FloatingArrow) -> Self {
        if let Some(anchor) = &mut self.anchor {
            anchor.options.arrow = Some(arrow);
        }
        self
    }

    pub fn floating_arrow_size(mut self, width: f32, height: f32, padding: f32) -> Self {
        if let Some(anchor) = &mut self.anchor {
            anchor.options.arrow = Some(
                FloatingArrow::new(des_layout::geometry::Size { width, height }).padding(padding),
            );
        }
        self
    }

    pub fn floating_hide(mut self, strategy: FloatingHideStrategy) -> Self {
        if let Some(anchor) = &mut self.anchor {
            anchor.options.hide.push(FloatingHide::new(strategy));
        }
        self
    }

    pub fn floating_hide_options(mut self, hide: FloatingHide) -> Self {
        if let Some(anchor) = &mut self.anchor {
            anchor.options.hide.push(hide);
        }
        self
    }

    pub fn floating_inline(mut self, inline: FloatingInline) -> Self {
        if let Some(anchor) = &mut self.anchor {
            anchor.options.inline = Some(inline);
        }
        self
    }

    pub fn floating_rtl(mut self, rtl: bool) -> Self {
        if let Some(anchor) = &mut self.anchor {
            anchor.options.rtl = rtl;
        }
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
    pub display: Display,
    pub direction: Direction,
    pub flex_direction: FlexDirection,
    pub flex_wrap: FlexWrap,
    pub flex_basis: Length,
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub align_content: AlignContent,
    pub align_items: AlignItems,
    pub align_self: Option<AlignSelf>,
    pub justify_items: Option<AlignItems>,
    pub justify_self: Option<AlignSelf>,
    pub justify_content: JustifyContent,
    pub gap: Length,
    pub row_gap: Length,
    pub column_gap: Length,
    pub grid_template_rows: Vec<GridTemplateComponent>,
    pub grid_template_columns: Vec<GridTemplateComponent>,
    pub grid_auto_rows: Vec<GridTrack>,
    pub grid_auto_columns: Vec<GridTrack>,
    pub grid_auto_flow: GridAutoFlow,
    pub grid_template_areas: Vec<GridTemplateArea>,
    pub grid_template_column_names: Vec<Vec<String>>,
    pub grid_template_row_names: Vec<Vec<String>>,
    pub grid_row: GridPlacementLine,
    pub grid_column: GridPlacementLine,
    pub margin: Insets,
    pub padding: Insets,
    pub width: Length,
    pub height: Length,
    pub min_size: Size,
    pub max_size: Size,
    pub animate_size: bool,
    pub background: Option<Color>,
    pub border: Option<Color>,
    pub border_width: Insets,
    pub border_style: BorderStyle,
    pub shadows: Vec<Shadow>,
    pub animate_paint: bool,
    pub animate_shadows: bool,
    pub text_color: Color,
    pub text_selection_background: Color,
    pub text_selection_color: Color,
    pub font_size: f32,
    pub text_layout: TextLayoutStyle,
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
    pub scrollbar_visible: bool,
    pub position: Position,
    pub inset: PositionInsets,
    pub anchor: Option<Anchor>,
    pub z_index: i32,
    pub transition: Option<Transition>,
}

impl Default for ComputedStyle {
    fn default() -> Self {
        Self {
            display: Display::Flex,
            direction: Direction::Ltr,
            flex_direction: FlexDirection::Column,
            flex_wrap: FlexWrap::NoWrap,
            flex_basis: Length::Auto,
            flex_grow: 0.0,
            flex_shrink: 0.0,
            align_content: AlignContent::Stretch,
            align_items: AlignItems::Start,
            align_self: None,
            justify_items: None,
            justify_self: None,
            justify_content: JustifyContent::Start,
            gap: Length::Px(0.0),
            row_gap: Length::Px(0.0),
            column_gap: Length::Px(0.0),
            grid_template_rows: Vec::new(),
            grid_template_columns: Vec::new(),
            grid_auto_rows: Vec::new(),
            grid_auto_columns: Vec::new(),
            grid_auto_flow: GridAutoFlow::Row,
            grid_template_areas: Vec::new(),
            grid_template_column_names: Vec::new(),
            grid_template_row_names: Vec::new(),
            grid_row: GridPlacementLine::default(),
            grid_column: GridPlacementLine::default(),
            margin: Insets::ZERO,
            padding: Insets::ZERO,
            width: Length::Auto,
            height: Length::Auto,
            min_size: Size::new(0.0, 0.0),
            max_size: Size::new(f32::INFINITY, f32::INFINITY),
            animate_size: true,
            background: None,
            border: None,
            border_width: Insets::ZERO,
            border_style: BorderStyle::Solid,
            shadows: Vec::new(),
            animate_paint: true,
            animate_shadows: true,
            text_color: Color::rgb(218, 226, 234),
            text_selection_background: Color::rgba(234, 221, 255, 190),
            text_selection_color: Color::rgb(29, 27, 32),
            font_size: 13.0,
            text_layout: TextLayoutStyle::default(),
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
            scrollbar_visible: false,
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
        if let Some(value) = style.display {
            self.display = value;
        }
        if let Some(value) = style.direction {
            self.direction = value;
        }
        if let Some(value) = style.flex_direction {
            self.flex_direction = value;
        }
        if let Some(value) = style.flex_wrap {
            self.flex_wrap = value;
        }
        if let Some(value) = style.flex_basis {
            self.flex_basis = value;
        }
        if let Some(value) = style.flex_grow {
            self.flex_grow = value.max(0.0);
        }
        if let Some(value) = style.flex_shrink {
            self.flex_shrink = value.max(0.0);
        }
        if let Some(value) = style.align_content {
            self.align_content = value;
        }
        if let Some(value) = style.align_items {
            self.align_items = value;
        }
        if let Some(value) = style.align_self {
            self.align_self = Some(value);
        }
        if let Some(value) = style.justify_items {
            self.justify_items = Some(value);
        }
        if let Some(value) = style.justify_self {
            self.justify_self = Some(value);
        }
        if let Some(value) = style.justify_content {
            self.justify_content = value;
        }
        if let Some(value) = style.gap {
            self.gap = value;
            self.row_gap = value;
            self.column_gap = value;
        }
        if let Some(value) = style.row_gap {
            self.row_gap = value;
        }
        if let Some(value) = style.column_gap {
            self.column_gap = value;
        }
        if let Some(value) = &style.grid_template_rows {
            self.grid_template_rows = value.clone();
        }
        if let Some(value) = &style.grid_template_columns {
            self.grid_template_columns = value.clone();
        }
        if let Some(value) = &style.grid_auto_rows {
            self.grid_auto_rows = value.clone();
        }
        if let Some(value) = &style.grid_auto_columns {
            self.grid_auto_columns = value.clone();
        }
        if let Some(value) = style.grid_auto_flow {
            self.grid_auto_flow = value;
        }
        if let Some(value) = &style.grid_template_areas {
            self.grid_template_areas = value.clone();
        }
        if let Some(value) = &style.grid_template_column_names {
            self.grid_template_column_names = value.clone();
        }
        if let Some(value) = &style.grid_template_row_names {
            self.grid_template_row_names = value.clone();
        }
        if let Some(value) = &style.grid_row {
            self.grid_row = value.clone();
        }
        if let Some(value) = &style.grid_column {
            self.grid_column = value.clone();
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
        if let Some(value) = style.animate_size {
            self.animate_size = value;
        }
        if let Some(value) = style.background {
            self.background = Some(value);
        }
        if let Some(value) = style.border {
            self.border = Some(value);
        }
        if let Some(values) = &style.shadows {
            self.shadows = values
                .iter()
                .copied()
                .map(|value| Shadow {
                    offset: value.offset,
                    blur: value.blur.max(0.0),
                    spread: value.spread,
                    color: value.color,
                })
                .collect();
        }
        if let Some(value) = style.animate_shadows {
            self.animate_shadows = value;
        }
        if let Some(value) = style.animate_paint {
            self.animate_paint = value;
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
        if let Some(value) = style.border_style {
            self.border_style = value;
        }
        if let Some(value) = style.text_color {
            self.text_color = value;
        }
        if let Some(value) = style.text_selection_background {
            self.text_selection_background = value;
        }
        if let Some(value) = style.text_selection_color {
            self.text_selection_color = value;
        }
        if let Some(value) = style.font_size {
            self.font_size = value;
        }
        if let Some(value) = style.text_layout {
            self.text_layout = value;
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
        if let Some(value) = style.scrollbar_visible {
            self.scrollbar_visible = value;
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

    pub(crate) fn normalize_overflow_axes(&mut self) {
        let (overflow_x, overflow_y) = normalize_overflow_pair(self.overflow_x, self.overflow_y);
        self.overflow_x = overflow_x;
        self.overflow_y = overflow_y;
    }
}

fn normalize_overflow_pair(x: Overflow, y: Overflow) -> (Overflow, Overflow) {
    if x.forces_cross_axis_normalization() || y.forces_cross_axis_normalization() {
        (normalize_overflow_axis(x), normalize_overflow_axis(y))
    } else {
        (x, y)
    }
}

fn normalize_overflow_axis(overflow: Overflow) -> Overflow {
    match overflow {
        Overflow::Visible => Overflow::Auto,
        Overflow::Clip => Overflow::Hidden,
        overflow => overflow,
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct StyleRule {
    pub(crate) selector: StyleSelector,
    pub(crate) style: Style,
    pub(crate) condition: Option<StyleCondition>,
}

impl StyleRule {
    pub fn new(selector: StyleSelector, style: Style) -> Self {
        Self {
            selector,
            style,
            condition: None,
        }
    }

    pub fn conditional(condition: StyleCondition, selector: StyleSelector, style: Style) -> Self {
        Self {
            selector,
            style,
            condition: Some(condition),
        }
    }
}

static NEXT_STYLESHEET_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct StyleSheetKey {
    id: u64,
    revision: u64,
}

impl StyleSheetKey {
    fn new() -> Self {
        Self {
            id: NEXT_STYLESHEET_ID.fetch_add(1, Ordering::Relaxed),
            revision: 0,
        }
    }

    fn bump(&mut self) {
        self.revision = self.revision.wrapping_add(1);
    }
}

#[derive(Clone, Debug)]
pub struct StyleSheet {
    pub(crate) rules: Vec<StyleRule>,
    index: StyleRuleIndex,
    key: StyleSheetKey,
}

impl Default for StyleSheet {
    fn default() -> Self {
        Self {
            rules: Vec::new(),
            index: StyleRuleIndex::default(),
            key: StyleSheetKey::new(),
        }
    }
}

impl PartialEq for StyleSheet {
    fn eq(&self, other: &Self) -> bool {
        self.rules == other.rules && self.index == other.index
    }
}

impl StyleSheet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn rule(mut self, selector: StyleSelector, style: Style) -> Self {
        self.push_rule(selector, style);
        self
    }

    pub fn conditional_rule(
        mut self,
        condition: StyleCondition,
        selector: StyleSelector,
        style: Style,
    ) -> Self {
        self.push_conditional_rule(condition, selector, style);
        self
    }

    pub fn viewport_rule(
        self,
        query: ViewportQuery,
        selector: StyleSelector,
        style: Style,
    ) -> Self {
        self.conditional_rule(StyleCondition::viewport(query), selector, style)
    }

    pub fn viewport_min_width(self, width: f32, selector: StyleSelector, style: Style) -> Self {
        self.viewport_rule(ViewportQuery::min_width(width), selector, style)
    }

    pub fn viewport_max_width(self, width: f32, selector: StyleSelector, style: Style) -> Self {
        self.viewport_rule(ViewportQuery::max_width(width), selector, style)
    }

    pub fn viewport_min_height(self, height: f32, selector: StyleSelector, style: Style) -> Self {
        self.viewport_rule(ViewportQuery::min_height(height), selector, style)
    }

    pub fn viewport_max_height(self, height: f32, selector: StyleSelector, style: Style) -> Self {
        self.viewport_rule(ViewportQuery::max_height(height), selector, style)
    }

    pub fn container_rule(
        self,
        query: ContainerQuery,
        selector: StyleSelector,
        style: Style,
    ) -> Self {
        self.conditional_rule(StyleCondition::container(query), selector, style)
    }

    pub fn container_min_width(self, width: f32, selector: StyleSelector, style: Style) -> Self {
        self.container_rule(ContainerQuery::min_width(width), selector, style)
    }

    pub fn container_max_width(self, width: f32, selector: StyleSelector, style: Style) -> Self {
        self.container_rule(ContainerQuery::max_width(width), selector, style)
    }

    pub fn container_min_height(self, height: f32, selector: StyleSelector, style: Style) -> Self {
        self.container_rule(ContainerQuery::min_height(height), selector, style)
    }

    pub fn container_max_height(self, height: f32, selector: StyleSelector, style: Style) -> Self {
        self.container_rule(ContainerQuery::max_height(height), selector, style)
    }

    pub fn push_rule(&mut self, selector: StyleSelector, style: Style) {
        self.push_style_rule(StyleRule::new(selector, style));
    }

    pub fn push_conditional_rule(
        &mut self,
        condition: StyleCondition,
        selector: StyleSelector,
        style: Style,
    ) {
        self.push_style_rule(StyleRule::conditional(condition, selector, style));
    }

    pub fn push_viewport_rule(
        &mut self,
        query: ViewportQuery,
        selector: StyleSelector,
        style: Style,
    ) {
        self.push_conditional_rule(StyleCondition::viewport(query), selector, style);
    }

    pub fn push_container_rule(
        &mut self,
        query: ContainerQuery,
        selector: StyleSelector,
        style: Style,
    ) {
        self.push_conditional_rule(StyleCondition::container(query), selector, style);
    }

    pub fn extend(&mut self, stylesheet: StyleSheet) {
        self.rules.reserve(stylesheet.rules.len());
        for rule in stylesheet.rules {
            self.push_style_rule(rule);
        }
    }

    pub fn parse_css(input: &str) -> Result<Self, crate::CssParseError> {
        crate::css::parse_stylesheet(input)
    }

    pub fn parse_css_forgiving(input: &str) -> Result<Self, crate::CssParseError> {
        crate::css::parse_stylesheet_forgiving(input)
    }

    pub fn extend_css(&mut self, input: &str) -> Result<(), crate::CssParseError> {
        self.extend(Self::parse_css(input)?);
        Ok(())
    }

    pub fn extend_css_forgiving(&mut self, input: &str) -> Result<(), crate::CssParseError> {
        self.extend(Self::parse_css_forgiving(input)?);
        Ok(())
    }

    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }

    #[cfg(test)]
    pub(crate) fn candidate_rule_count(&self, element: &DocumentNode) -> usize {
        self.index.candidates_for(element).len()
    }

    pub(crate) fn key(&self) -> StyleSheetKey {
        self.key
    }

    fn push_style_rule(&mut self, rule: StyleRule) {
        let index = self.rules.len();
        self.index.insert(index, &rule.selector);
        self.rules.push(rule);
        self.key.bump();
    }

    pub(crate) fn has_container_rules(&self) -> bool {
        self.rules
            .iter()
            .any(|rule| rule.condition.is_some_and(StyleCondition::is_container))
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
struct StyleRuleIndex {
    universal: Vec<usize>,
    by_id: HashMap<ElementId, Vec<usize>>,
    by_class: HashMap<ClassName, Vec<usize>>,
    by_element: HashMap<Element, Vec<usize>>,
}

impl StyleRuleIndex {
    fn insert(&mut self, index: usize, selector: &StyleSelector) {
        match primary_selector_key(selector) {
            SelectorIndexKey::Universal => self.universal.push(index),
            SelectorIndexKey::Id(id) => self.by_id.entry(id).or_default().push(index),
            SelectorIndexKey::Class(class) => self.by_class.entry(class).or_default().push(index),
            SelectorIndexKey::Element(element) => {
                self.by_element.entry(element).or_default().push(index);
            }
        }
    }

    fn candidates_for(&self, element: &DocumentNode) -> Vec<usize> {
        let mut candidates = Vec::with_capacity(
            self.universal.len()
                + self.by_id.get(&element.id).map_or(0, Vec::len)
                + self
                    .by_element
                    .get(&element.spec.element)
                    .map_or(0, Vec::len)
                + element
                    .spec
                    .classes
                    .iter()
                    .map(|class| self.by_class.get(class).map_or(0, Vec::len))
                    .sum::<usize>(),
        );
        candidates.extend(self.universal.iter().copied());
        if let Some(rules) = self.by_id.get(&element.id) {
            candidates.extend(rules.iter().copied());
        }
        if let Some(rules) = self.by_element.get(&element.spec.element) {
            candidates.extend(rules.iter().copied());
        }
        for class in &element.spec.classes {
            if let Some(rules) = self.by_class.get(class) {
                candidates.extend(rules.iter().copied());
            }
        }
        candidates.sort_unstable();
        candidates.dedup();
        candidates
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum SelectorIndexKey {
    Universal,
    Id(ElementId),
    Class(ClassName),
    Element(Element),
}

fn primary_selector_key(selector: &StyleSelector) -> SelectorIndexKey {
    match selector {
        StyleSelector::Element(element) => SelectorIndexKey::Element(*element),
        StyleSelector::Class(class) => SelectorIndexKey::Class(class.clone()),
        StyleSelector::Id(id) => SelectorIndexKey::Id(id.clone()),
        StyleSelector::ClassState(class, _) => SelectorIndexKey::Class(class.clone()),
        StyleSelector::IdState(id, _) => SelectorIndexKey::Id(id.clone()),
        StyleSelector::Compound(selector) => primary_compound_selector_key(selector),
        StyleSelector::Descendant(selectors) => selectors
            .last()
            .map(primary_compound_selector_key)
            .unwrap_or(SelectorIndexKey::Universal),
        StyleSelector::Complex(selector) => selector
            .parts
            .last()
            .map(|part| primary_compound_selector_key(&part.selector))
            .unwrap_or(SelectorIndexKey::Universal),
        StyleSelector::State(_)
        | StyleSelector::FirstChild
        | StyleSelector::LastChild
        | StyleSelector::NthChild(_)
        | StyleSelector::NthChildFormula(_) => SelectorIndexKey::Universal,
    }
}

fn primary_compound_selector_key(selector: &CompoundSelector) -> SelectorIndexKey {
    if let Some(id) = &selector.id {
        SelectorIndexKey::Id(id.clone())
    } else if let Some(class) = selector.classes.last() {
        SelectorIndexKey::Class(class.clone())
    } else if let Some(element) = selector.element {
        SelectorIndexKey::Element(element)
    } else {
        SelectorIndexKey::Universal
    }
}

pub(crate) fn resolve_style_with_position(
    stylesheet: &StyleSheet,
    context: StyleResolutionContext<'_>,
) -> ComputedStyle {
    let mut style = ComputedStyle::default();

    for index in stylesheet.index.candidates_for(context.element) {
        let rule = &stylesheet.rules[index];
        if rule_matches(rule, context) {
            style.apply(&rule.style);
        }
    }

    style.normalize_overflow_axes();

    style
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct StyleInvalidation {
    pub paint_changed: bool,
    pub layout_changed: bool,
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
    previous.display != next.display
        || previous.flex_direction != next.flex_direction
        || previous.flex_wrap != next.flex_wrap
        || previous.flex_basis != next.flex_basis
        || previous.flex_grow != next.flex_grow
        || previous.flex_shrink != next.flex_shrink
        || previous.align_content != next.align_content
        || previous.align_items != next.align_items
        || previous.align_self != next.align_self
        || previous.justify_items != next.justify_items
        || previous.justify_self != next.justify_self
        || previous.justify_content != next.justify_content
        || previous.gap != next.gap
        || previous.row_gap != next.row_gap
        || previous.column_gap != next.column_gap
        || previous.grid_template_rows != next.grid_template_rows
        || previous.grid_template_columns != next.grid_template_columns
        || previous.grid_auto_rows != next.grid_auto_rows
        || previous.grid_auto_columns != next.grid_auto_columns
        || previous.grid_auto_flow != next.grid_auto_flow
        || previous.grid_template_areas != next.grid_template_areas
        || previous.grid_template_column_names != next.grid_template_column_names
        || previous.grid_template_row_names != next.grid_template_row_names
        || previous.grid_row != next.grid_row
        || previous.grid_column != next.grid_column
        || previous.direction != next.direction
        || previous.margin != next.margin
        || previous.padding != next.padding
        || previous.width != next.width
        || previous.height != next.height
        || previous.min_size != next.min_size
        || previous.max_size != next.max_size
        || previous.border_width != next.border_width
        || previous.font_size != next.font_size
        || previous.text_layout != next.text_layout
        || previous.line_height != next.line_height
        || previous.overflow_x != next.overflow_x
        || previous.overflow_y != next.overflow_y
        || previous.position != next.position
        || previous.inset != next.inset
        || previous.anchor != next.anchor
}

fn selector_matches(
    selector: &StyleSelector,
    element: &DocumentNode,
    state: Option<&ElementState>,
    position: Option<ChildPosition>,
    ancestors: &[StyleMatchContext<'_>],
    previous_siblings: &[StyleMatchContext<'_>],
) -> bool {
    match selector {
        StyleSelector::Element(target) => element.spec.element == *target,
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
        StyleSelector::NthChildFormula(formula) => {
            position.is_some_and(|position| position.matches_nth_formula(*formula))
        }
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
        StyleSelector::Descendant(selectors) => {
            descendant_selector_matches(selectors, element, state, position, ancestors)
        }
        StyleSelector::Complex(selector) => complex_selector_matches(
            selector,
            element,
            state,
            position,
            ancestors,
            previous_siblings,
        ),
    }
}

fn rule_matches(rule: &StyleRule, context: StyleResolutionContext<'_>) -> bool {
    rule.condition
        .is_none_or(|condition| condition.matches(context.viewport, context.container))
        && selector_matches(
            &rule.selector,
            context.element,
            context.state,
            context.position,
            context.ancestors,
            context.previous_siblings,
        )
}

fn compound_selector_matches(
    selector: &CompoundSelector,
    element: &DocumentNode,
    state: Option<&ElementState>,
    position: Option<ChildPosition>,
) -> bool {
    if selector
        .element
        .is_some_and(|target| element.spec.element != target)
    {
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

fn descendant_selector_matches(
    selectors: &[CompoundSelector],
    element: &DocumentNode,
    state: Option<&ElementState>,
    position: Option<ChildPosition>,
    ancestors: &[StyleMatchContext<'_>],
) -> bool {
    let Some((target, ancestor_selectors)) = selectors.split_last() else {
        return false;
    };
    if !compound_selector_matches(target, element, state, position) {
        return false;
    }

    let mut next_ancestor = ancestor_selectors.len();
    for ancestor in ancestors.iter().rev() {
        if next_ancestor == 0 {
            break;
        }
        let selector = &ancestor_selectors[next_ancestor - 1];
        if compound_selector_matches(
            selector,
            ancestor.element,
            ancestor.state,
            ancestor.position,
        ) {
            next_ancestor -= 1;
        }
    }

    next_ancestor == 0
}

fn complex_selector_matches(
    selector: &ComplexSelector,
    element: &DocumentNode,
    state: Option<&ElementState>,
    position: Option<ChildPosition>,
    ancestors: &[StyleMatchContext<'_>],
    previous_siblings: &[StyleMatchContext<'_>],
) -> bool {
    let Some(target) = selector.parts.last() else {
        return false;
    };
    if !compound_selector_matches(&target.selector, element, state, position) {
        return false;
    }

    let mut ancestor_end = ancestors.len();
    let mut sibling_end = previous_siblings.len();
    for index in (1..selector.parts.len()).rev() {
        let previous = &selector.parts[index - 1].selector;
        match selector.parts[index].combinator {
            Some(SelectorCombinator::Child) => {
                let Some(parent_index) = ancestor_end.checked_sub(1) else {
                    return false;
                };
                let parent = &ancestors[parent_index];
                if !compound_selector_matches(
                    previous,
                    parent.element,
                    parent.state,
                    parent.position,
                ) {
                    return false;
                }
                ancestor_end = parent_index;
                sibling_end = 0;
            }
            Some(SelectorCombinator::Descendant) => {
                let Some(found_index) = (0..ancestor_end).rev().find(|index| {
                    let ancestor = &ancestors[*index];
                    compound_selector_matches(
                        previous,
                        ancestor.element,
                        ancestor.state,
                        ancestor.position,
                    )
                }) else {
                    return false;
                };
                ancestor_end = found_index;
                sibling_end = 0;
            }
            Some(SelectorCombinator::AdjacentSibling) => {
                let Some(sibling_index) = sibling_end.checked_sub(1) else {
                    return false;
                };
                let sibling = &previous_siblings[sibling_index];
                if !compound_selector_matches(
                    previous,
                    sibling.element,
                    sibling.state,
                    sibling.position,
                ) {
                    return false;
                }
                sibling_end = sibling_index;
            }
            Some(SelectorCombinator::GeneralSibling) => {
                let Some(found_index) = (0..sibling_end).rev().find(|index| {
                    let sibling = &previous_siblings[*index];
                    compound_selector_matches(
                        previous,
                        sibling.element,
                        sibling.state,
                        sibling.position,
                    )
                }) else {
                    return false;
                };
                sibling_end = found_index;
            }
            None => return false,
        }
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
        ChildPositionSelector::NthFormula(formula) => position.matches_nth_formula(formula),
    }
}

fn state_selector_matches(
    selector: ElementStateSelector,
    element: &DocumentNode,
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
