//! Product-specific UI runtime primitives.
//!
//! `des-ui-runtime` owns the DOM-like element tree, deterministic style
//! resolution, retained interaction state, layout frames, and input routing.
//! Rendering hosts such as egui should translate platform input into
//! [`RuntimeInput`] and paint [`RuntimeOutput::layout`].

use std::collections::{BTreeSet, HashMap};

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
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Direction {
    Row,
    Column,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Overflow {
    Visible,
    Scroll,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Length {
    Auto,
    Px(f32),
    Fill,
    Percent(f32),
}

impl Length {
    fn resolve(self, available: f32, auto: f32) -> f32 {
        match self {
            Self::Auto => auto,
            Self::Px(value) => value,
            Self::Fill => available,
            Self::Percent(factor) => available * factor,
        }
        .max(0.0)
    }
}

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

    fn lerp(self, target: Self, amount: f32) -> Self {
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

#[derive(Clone, Debug, Eq, PartialEq)]
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
}

impl ElementSpec {
    pub fn new(role: ElementRole) -> Self {
        Self {
            role,
            classes: Vec::new(),
            interactive: false,
            selected: false,
            disabled: false,
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
}

#[derive(Clone, Debug, PartialEq)]
pub struct Element {
    pub id: ElementId,
    pub spec: ElementSpec,
    pub text: Option<String>,
    pub children: Vec<Element>,
}

impl Element {
    fn collect_ids(&self, ids: &mut BTreeSet<ElementId>) {
        ids.insert(self.id.clone());
        for child in &self.children {
            child.collect_ids(ids);
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Scene {
    pub viewport: Size,
    pub root: Element,
}

impl Scene {
    pub fn build(viewport: Size, add_contents: impl FnOnce(&mut Ui)) -> Self {
        let mut ui = Ui::default();
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
pub struct Ui {
    children: Vec<Element>,
}

impl Ui {
    pub fn element(
        &mut self,
        id: impl Into<ElementId>,
        spec: ElementSpec,
        add_contents: impl FnOnce(&mut Ui),
    ) {
        let mut child_ui = Ui::default();
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
    fn sample(self, amount: f32) -> f32 {
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
    fn apply(&mut self, patch: &StylePatch) {
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
    selector: StyleSelector,
    patch: StylePatch,
}

impl StyleRule {
    pub fn new(selector: StyleSelector, patch: StylePatch) -> Self {
        Self { selector, patch }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct StyleSheet {
    rules: Vec<StyleRule>,
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

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ElementState {
    pub scroll_y: f32,
    pub hovered: bool,
    pub pressed: bool,
    pub scrollbar_hovered: bool,
    pub scrollbar_dragged: bool,
    pub focused: bool,
    pub click_count: u32,
    rendered_style: Option<ComputedStyle>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ChangeSet {
    pub created: Vec<ElementId>,
    pub retained: Vec<ElementId>,
    pub removed: Vec<ElementId>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LayoutFrame {
    pub id: ElementId,
    pub role: ElementRole,
    pub classes: Vec<ClassName>,
    pub rect: Rect,
    pub style: ComputedStyle,
    pub text: Option<String>,
    pub interactive: bool,
    pub children: Vec<LayoutFrame>,
}

impl LayoutFrame {
    pub fn find(&self, id: &str) -> Option<&Self> {
        if self.id.as_str() == id {
            return Some(self);
        }
        self.children.iter().find_map(|child| child.find(id))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeOutput {
    pub changes: ChangeSet,
    pub layout: LayoutFrame,
    pub hit_id: Option<ElementId>,
    pub scroll_chrome: Vec<ScrollChrome>,
    pub animating: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RuntimeInput {
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
    pub max_scroll: f32,
    pub visible: bool,
    pub expanded: bool,
    pub hovered: bool,
    pub dragged: bool,
}

#[derive(Clone, Debug)]
struct ScrollDrag {
    element_id: ElementId,
    pointer_offset_from_handle_top: f32,
}

#[derive(Default)]
pub struct Runtime {
    states: HashMap<ElementId, ElementState>,
    scroll_limits: HashMap<ElementId, f32>,
    active_scroll_drag: Option<ScrollDrag>,
}

impl Runtime {
    pub fn update(&mut self, scene: &Scene, stylesheet: &StyleSheet) -> RuntimeOutput {
        self.update_with_input(scene, stylesheet, RuntimeInput::default())
    }

    pub fn update_with_input(
        &mut self,
        scene: &Scene,
        stylesheet: &StyleSheet,
        input: RuntimeInput,
    ) -> RuntimeOutput {
        let changes = self.sync_element_states(scene);
        let mut scroll_limits = HashMap::new();
        let input_layout = layout_element(
            &scene.root,
            Rect::new(0.0, 0.0, scene.viewport.width, scene.viewport.height),
            stylesheet,
            &self.states,
            &mut scroll_limits,
        );
        self.scroll_limits = scroll_limits;
        let input_scroll_chrome = scroll_chrome(&input_layout, &self.states, &self.scroll_limits);
        let hit_id = self.apply_input(&input_layout, &input_scroll_chrome, input);
        self.clamp_scroll_states();
        let input_animating = self.update_style_animation(scene, stylesheet);

        let mut scroll_limits = HashMap::new();
        let layout = layout_element(
            &scene.root,
            Rect::new(0.0, 0.0, scene.viewport.width, scene.viewport.height),
            stylesheet,
            &self.states,
            &mut scroll_limits,
        );
        self.scroll_limits = scroll_limits;
        self.clamp_scroll_states();
        let scroll_chrome = scroll_chrome(&layout, &self.states, &self.scroll_limits);

        RuntimeOutput {
            changes,
            layout,
            hit_id,
            scroll_chrome,
            animating: input_animating,
        }
    }

    pub fn element_state(&self, id: &str) -> Option<&ElementState> {
        self.states.get(&ElementId::new(id))
    }

    pub fn element_state_mut(&mut self, id: &str) -> Option<&mut ElementState> {
        self.states.get_mut(&ElementId::new(id))
    }

    fn sync_element_states(&mut self, scene: &Scene) -> ChangeSet {
        let mut next_ids = BTreeSet::new();
        scene.root.collect_ids(&mut next_ids);

        let existing_ids: BTreeSet<_> = self.states.keys().cloned().collect();
        let mut changes = ChangeSet::default();

        for id in &next_ids {
            if existing_ids.contains(id) {
                changes.retained.push(id.clone());
            } else {
                changes.created.push(id.clone());
                self.states.insert(id.clone(), ElementState::default());
            }
        }

        for id in existing_ids.difference(&next_ids) {
            changes.removed.push(id.clone());
            self.states.remove(id);
        }

        changes
    }

    fn apply_input(
        &mut self,
        layout: &LayoutFrame,
        scroll_chrome: &[ScrollChrome],
        input: RuntimeInput,
    ) -> Option<ElementId> {
        for state in self.states.values_mut() {
            state.hovered = false;
            state.pressed = false;
            state.scrollbar_hovered = false;
            state.scrollbar_dragged = false;
        }

        let pointer = input.pointer?;
        let scrollbar_hit = scroll_chrome
            .iter()
            .rev()
            .find(|chrome| chrome.hit_rect.contains(pointer.position));
        self.apply_scrollbar_input(scroll_chrome, scrollbar_hit, pointer);

        if let Some(chrome) = scrollbar_hit {
            if let Some(state) = self.states.get_mut(&chrome.element_id) {
                state.hovered = true;
                state.pressed = pointer.primary_down;
                state.scrollbar_hovered = true;
            }
            return Some(chrome.element_id.clone());
        }
        if let Some(active_drag) = &self.active_scroll_drag {
            if let Some(state) = self.states.get_mut(&active_drag.element_id) {
                state.hovered = true;
                state.pressed = true;
                state.scrollbar_dragged = true;
            }
            return Some(active_drag.element_id.clone());
        }

        let path = hit_path(layout, pointer.position)?;
        if input.scroll_delta.y.abs() > f32::EPSILON
            && let Some(scroll_frame) = path
                .iter()
                .rev()
                .find(|frame| frame.style.overflow_y == Overflow::Scroll)
            && let Some(state) = self.states.get_mut(&scroll_frame.id)
        {
            let max_scroll = self
                .scroll_limits
                .get(&scroll_frame.id)
                .copied()
                .unwrap_or_default();
            state.scroll_y = (state.scroll_y - input.scroll_delta.y).clamp(0.0, max_scroll);
        }

        for frame in &path {
            if let Some(state) = self.states.get_mut(&frame.id) {
                state.hovered = true;
            }
        }

        let hit_id = path
            .iter()
            .rev()
            .find(|frame| frame.interactive)
            .unwrap_or_else(|| path.last().expect("hit path is never empty"))
            .id
            .clone();
        if let Some(state) = self.states.get_mut(&hit_id) {
            state.pressed = pointer.primary_down;
            if pointer.primary_clicked {
                state.click_count += 1;
            }
        }

        Some(hit_id)
    }

    fn apply_scrollbar_input(
        &mut self,
        scroll_chrome: &[ScrollChrome],
        hit: Option<&ScrollChrome>,
        pointer: PointerInput,
    ) {
        if !pointer.primary_down {
            self.active_scroll_drag = None;
            return;
        }

        let active_id = self
            .active_scroll_drag
            .as_ref()
            .map(|drag| drag.element_id.clone());
        let active_chrome = active_id
            .as_ref()
            .and_then(|id| scroll_chrome.iter().find(|chrome| &chrome.element_id == id));
        let chrome = active_chrome.or(hit);
        let Some(chrome) = chrome else {
            return;
        };

        if self
            .active_scroll_drag
            .as_ref()
            .is_none_or(|drag| drag.element_id != chrome.element_id)
        {
            let offset = if chrome.handle_rect.contains(pointer.position) {
                pointer.position.y - chrome.handle_rect.origin.y
            } else {
                chrome.handle_rect.size.height / 2.0
            };
            self.active_scroll_drag = Some(ScrollDrag {
                element_id: chrome.element_id.clone(),
                pointer_offset_from_handle_top: offset,
            });
        }

        let Some(drag) = &self.active_scroll_drag else {
            return;
        };
        let track_travel =
            (chrome.track_rect.size.height - chrome.handle_rect.size.height).max(0.0);
        let handle_top = pointer.position.y - drag.pointer_offset_from_handle_top;
        let handle_progress = if track_travel <= f32::EPSILON {
            0.0
        } else {
            ((handle_top - chrome.track_rect.origin.y) / track_travel).clamp(0.0, 1.0)
        };
        if let Some(state) = self.states.get_mut(&chrome.element_id) {
            state.scroll_y = handle_progress * chrome.max_scroll;
            state.scrollbar_dragged = true;
        }
    }

    fn clamp_scroll_states(&mut self) {
        for (id, state) in &mut self.states {
            let max_scroll = self.scroll_limits.get(id).copied().unwrap_or_default();
            state.scroll_y = state.scroll_y.clamp(0.0, max_scroll);
        }
    }

    fn update_style_animation(&mut self, scene: &Scene, stylesheet: &StyleSheet) -> bool {
        const SNAP_EPSILON: f32 = 0.001;
        update_element_style_animation(&scene.root, stylesheet, &mut self.states, SNAP_EPSILON)
    }
}

fn update_element_style_animation(
    element: &Element,
    stylesheet: &StyleSheet,
    states: &mut HashMap<ElementId, ElementState>,
    snap_epsilon: f32,
) -> bool {
    let target_style = resolve_style(element, stylesheet, states.get(&element.id));
    let mut animating = false;

    if let Some(state) = states.get_mut(&element.id) {
        let next_style = match &state.rendered_style {
            Some(current_style) => {
                if let Some(transition) = target_style.transition {
                    let (style, still_animating) =
                        eased_style(current_style, &target_style, transition, snap_epsilon);
                    animating |= still_animating;
                    style
                } else {
                    target_style
                }
            }
            None => target_style,
        };
        state.rendered_style = Some(next_style);
    }

    for child in &element.children {
        animating |= update_element_style_animation(child, stylesheet, states, snap_epsilon);
    }

    animating
}

fn eased_style(
    current: &ComputedStyle,
    target: &ComputedStyle,
    transition: Transition,
    snap_epsilon: f32,
) -> (ComputedStyle, bool) {
    let amount = transition.easing.sample(transition.step.clamp(0.0, 1.0));
    let mut next = target.clone();
    let mut animating = false;

    next.background =
        ease_optional_color(current.background, target.background, amount, snap_epsilon);
    animating |= next.background != target.background;

    next.border = ease_optional_color(current.border, target.border, amount, snap_epsilon);
    animating |= next.border != target.border;

    next.text_color = current.text_color.lerp(target.text_color, amount);
    if color_distance(next.text_color, target.text_color) <= snap_epsilon {
        next.text_color = target.text_color;
    }
    animating |= next.text_color != target.text_color;

    next.border_width = ease_f32(
        current.border_width,
        target.border_width,
        amount,
        snap_epsilon,
    );
    animating |= (next.border_width - target.border_width).abs() > snap_epsilon;

    (next, animating)
}

fn ease_optional_color(
    current: Option<Color>,
    target: Option<Color>,
    amount: f32,
    snap_epsilon: f32,
) -> Option<Color> {
    match (current, target) {
        (Some(current), Some(target)) => {
            let next = current.lerp(target, amount);
            if color_distance(next, target) <= snap_epsilon {
                Some(target)
            } else {
                Some(next)
            }
        }
        (None, Some(target)) => {
            let next = Color { a: 0, ..target }.lerp(target, amount);
            if color_distance(next, target) <= snap_epsilon {
                Some(target)
            } else {
                Some(next)
            }
        }
        (Some(current), None) => {
            let transparent = Color { a: 0, ..current };
            let next = current.lerp(transparent, amount);
            if color_distance(next, transparent) <= snap_epsilon {
                None
            } else {
                Some(next)
            }
        }
        (None, None) => None,
    }
}

fn color_distance(left: Color, right: Color) -> f32 {
    (left.r as f32 - right.r as f32).abs()
        + (left.g as f32 - right.g as f32).abs()
        + (left.b as f32 - right.b as f32).abs()
        + (left.a as f32 - right.a as f32).abs()
}

fn ease_f32(current: f32, target: f32, amount: f32, snap_epsilon: f32) -> f32 {
    let next = current + (target - current) * amount;
    if (next - target).abs() <= snap_epsilon {
        target
    } else {
        next
    }
}

fn layout_element(
    element: &Element,
    parent_rect: Rect,
    stylesheet: &StyleSheet,
    states: &HashMap<ElementId, ElementState>,
    scroll_limits: &mut HashMap<ElementId, f32>,
) -> LayoutFrame {
    let style = resolve_style(element, stylesheet, states.get(&element.id));
    let style = states
        .get(&element.id)
        .and_then(|state| state.rendered_style.clone())
        .unwrap_or(style);
    let rect = element_rect(element, &style, parent_rect, stylesheet, states);
    let inner_rect = rect.inset(Insets::all(style.border_width));
    let mut content_rect = inner_rect.inset(style.padding);
    let content_size = measure_children(element, &style, content_rect.size, stylesheet, states);
    if style.overflow_y == Overflow::Scroll {
        scroll_limits.insert(
            element.id.clone(),
            (content_size.height - content_rect.size.height).max(0.0),
        );
    }
    if style.overflow_y == Overflow::Scroll
        && let Some(state) = states.get(&element.id)
    {
        content_rect.origin.y -= state.scroll_y;
    }
    let children = layout_children(
        element,
        &style,
        content_rect,
        stylesheet,
        states,
        scroll_limits,
    );

    LayoutFrame {
        id: element.id.clone(),
        role: element.spec.role,
        classes: element.spec.classes.clone(),
        rect,
        style,
        text: element.text.clone(),
        interactive: element.spec.interactive && !element.spec.disabled,
        children,
    }
}

fn resolve_style(
    element: &Element,
    stylesheet: &StyleSheet,
    state: Option<&ElementState>,
) -> ComputedStyle {
    let mut style = ComputedStyle::default();

    for rule in &stylesheet.rules {
        if selector_matches(rule.selector, element, state) {
            style.apply(&rule.patch);
        }
    }

    style
}

fn selector_matches(
    selector: StyleSelector,
    element: &Element,
    state: Option<&ElementState>,
) -> bool {
    match selector {
        StyleSelector::Role(role) => element.spec.role == role,
        StyleSelector::Class(class) => element
            .spec
            .classes
            .iter()
            .any(|element_class| element_class.as_str() == class),
        StyleSelector::Id(id) => element.id.as_str() == id,
        StyleSelector::State(selector) => match selector {
            ElementStateSelector::Hovered => state.is_some_and(|state| state.hovered),
            ElementStateSelector::Pressed => state.is_some_and(|state| state.pressed),
            ElementStateSelector::Focused => state.is_some_and(|state| state.focused),
            ElementStateSelector::Selected => element.spec.selected,
            ElementStateSelector::Disabled => element.spec.disabled,
        },
        StyleSelector::ClassState(class, selector) => {
            element
                .spec
                .classes
                .iter()
                .any(|element_class| element_class.as_str() == class)
                && selector_matches(StyleSelector::State(selector), element, state)
        }
        StyleSelector::IdState(id, selector) => {
            element.id.as_str() == id
                && selector_matches(StyleSelector::State(selector), element, state)
        }
    }
}

fn element_rect(
    element: &Element,
    style: &ComputedStyle,
    parent_rect: Rect,
    stylesheet: &StyleSheet,
    states: &HashMap<ElementId, ElementState>,
) -> Rect {
    if element.spec.role == ElementRole::Root {
        return parent_rect;
    }

    let measured = measure_element(element, style, parent_rect.size, stylesheet, states);
    Rect::new(
        parent_rect.origin.x + style.margin.left,
        parent_rect.origin.y + style.margin.top,
        measured.width,
        measured.height,
    )
}

fn layout_children(
    element: &Element,
    style: &ComputedStyle,
    content_rect: Rect,
    stylesheet: &StyleSheet,
    states: &HashMap<ElementId, ElementState>,
    scroll_limits: &mut HashMap<ElementId, f32>,
) -> Vec<LayoutFrame> {
    let mut cursor = content_rect.origin;
    let mut frames = Vec::with_capacity(element.children.len());

    for child in &element.children {
        let child_style = resolve_style(child, stylesheet, states.get(&child.id));
        let child_available = Size::new(
            (content_rect.size.width - child_style.margin.horizontal()).max(0.0),
            (content_rect.size.height - child_style.margin.vertical()).max(0.0),
        );
        let measured = measure_element(child, &child_style, child_available, stylesheet, states);
        let child_rect = Rect::new(
            cursor.x,
            cursor.y,
            child_available.width,
            child_available.height,
        );
        frames.push(layout_element(
            child,
            child_rect,
            stylesheet,
            states,
            scroll_limits,
        ));

        match style.direction {
            Direction::Column => {
                cursor.y += measured.height + child_style.margin.vertical() + style.gap
            }
            Direction::Row => {
                cursor.x += measured.width + child_style.margin.horizontal() + style.gap
            }
        }
    }

    frames
}

fn measure_element(
    element: &Element,
    style: &ComputedStyle,
    parent_size: Size,
    stylesheet: &StyleSheet,
    states: &HashMap<ElementId, ElementState>,
) -> Size {
    let auto_size = match element.spec.role {
        ElementRole::Text => {
            let width = element
                .text
                .as_ref()
                .map(|text| text.chars().count() as f32 * 7.5)
                .unwrap_or_default();
            Size::new(width.max(style.min_size.width), 18.0)
        }
        _ => {
            let content = measure_children(element, style, parent_size, stylesheet, states);
            Size::new(
                content.width + style.padding.horizontal() + style.border_width * 2.0,
                content.height + style.padding.vertical() + style.border_width * 2.0,
            )
        }
    };

    Size::new(
        style
            .width
            .resolve(parent_size.width, auto_size.width)
            .max(style.min_size.width),
        style
            .height
            .resolve(parent_size.height, auto_size.height)
            .max(style.min_size.height),
    )
}

fn measure_children(
    element: &Element,
    style: &ComputedStyle,
    parent_size: Size,
    stylesheet: &StyleSheet,
    states: &HashMap<ElementId, ElementState>,
) -> Size {
    let mut width: f32 = 0.0;
    let mut height: f32 = 0.0;
    let child_count = element.children.len();

    for child in &element.children {
        let child_style = resolve_style(child, stylesheet, states.get(&child.id));
        let child_available = Size::new(
            (parent_size.width - child_style.margin.horizontal()).max(0.0),
            (parent_size.height - child_style.margin.vertical()).max(0.0),
        );
        let child_size = measure_element(child, &child_style, child_available, stylesheet, states);
        let outer_width = child_size.width + child_style.margin.horizontal();
        let outer_height = child_size.height + child_style.margin.vertical();
        match style.direction {
            Direction::Column => {
                width = width.max(outer_width);
                height += outer_height;
            }
            Direction::Row => {
                width += outer_width;
                height = height.max(outer_height);
            }
        }
    }

    if child_count > 1 {
        let gap = style.gap * (child_count - 1) as f32;
        match style.direction {
            Direction::Column => height += gap,
            Direction::Row => width += gap,
        }
    }

    Size::new(width.min(parent_size.width), height)
}

fn hit_path(frame: &LayoutFrame, point: Point) -> Option<Vec<&LayoutFrame>> {
    if !frame.rect.contains(point) {
        return None;
    }

    let mut children: Vec<_> = frame.children.iter().collect();
    children.sort_by_key(|child| child.style.z_index);

    let mut path = vec![frame];
    if let Some(mut child_path) = children
        .into_iter()
        .rev()
        .find_map(|child| hit_path(child, point))
    {
        path.append(&mut child_path);
    }

    Some(path)
}

fn scroll_chrome(
    frame: &LayoutFrame,
    states: &HashMap<ElementId, ElementState>,
    scroll_limits: &HashMap<ElementId, f32>,
) -> Vec<ScrollChrome> {
    let mut chrome = Vec::new();
    collect_scroll_chrome(frame, states, scroll_limits, &mut chrome);
    chrome
}

fn collect_scroll_chrome(
    frame: &LayoutFrame,
    states: &HashMap<ElementId, ElementState>,
    scroll_limits: &HashMap<ElementId, f32>,
    chrome: &mut Vec<ScrollChrome>,
) {
    if frame.style.overflow_y == Overflow::Scroll {
        let max_scroll = scroll_limits.get(&frame.id).copied().unwrap_or_default();
        if max_scroll > 0.0 {
            chrome.push(scroll_chrome_for_frame(frame, states, max_scroll));
        }
    }

    for child in &frame.children {
        collect_scroll_chrome(child, states, scroll_limits, chrome);
    }
}

fn scroll_chrome_for_frame(
    frame: &LayoutFrame,
    states: &HashMap<ElementId, ElementState>,
    max_scroll: f32,
) -> ScrollChrome {
    const BAR_WIDTH: f32 = 10.0;
    const IDLE_WIDTH: f32 = 2.0;
    const HIT_WIDTH: f32 = 12.0;
    const MIN_HANDLE_LENGTH: f32 = 18.0;

    let state = states.get(&frame.id);
    let container_hovered = state.is_some_and(|state| state.hovered);
    let scrollbar_hovered = state.is_some_and(|state| state.scrollbar_hovered);
    let dragged = state.is_some_and(|state| state.scrollbar_dragged);
    let visible = container_hovered || scrollbar_hovered || dragged;
    let expanded = scrollbar_hovered || dragged;
    let visual_width = if expanded { BAR_WIDTH } else { IDLE_WIDTH };
    let viewport_rect = frame
        .rect
        .inset(Insets::all(frame.style.border_width))
        .inset(frame.style.padding);
    let content_height = viewport_rect.size.height + max_scroll;
    let handle_height = if content_height <= f32::EPSILON {
        viewport_rect.size.height
    } else {
        (viewport_rect.size.height / content_height * viewport_rect.size.height)
            .max(MIN_HANDLE_LENGTH)
            .min(viewport_rect.size.height)
    };
    let state_scroll = state.map(|state| state.scroll_y).unwrap_or_default();
    let track_travel = (viewport_rect.size.height - handle_height).max(0.0);
    let handle_top = if max_scroll <= f32::EPSILON {
        viewport_rect.origin.y
    } else {
        viewport_rect.origin.y + (state_scroll / max_scroll).clamp(0.0, 1.0) * track_travel
    };
    let track_rect = Rect::new(
        viewport_rect.right() - visual_width,
        viewport_rect.origin.y,
        visual_width,
        viewport_rect.size.height,
    );
    let hit_rect = Rect::new(
        viewport_rect.right() - HIT_WIDTH,
        viewport_rect.origin.y,
        HIT_WIDTH,
        viewport_rect.size.height,
    );
    let handle_rect = Rect::new(
        viewport_rect.right() - visual_width,
        handle_top,
        visual_width,
        handle_height,
    );

    ScrollChrome {
        element_id: frame.id.clone(),
        track_rect,
        hit_rect,
        handle_rect,
        max_scroll,
        visible,
        expanded,
        hovered: scrollbar_hovered,
        dragged,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_reports_created_retained_and_removed_elements() {
        let mut runtime = Runtime::default();
        let stylesheet = probe_stylesheet();
        let first = catalog_scene("Projects");
        let first_output = runtime.update(&first, &stylesheet);

        assert!(
            first_output
                .changes
                .created
                .contains(&ElementId::new("catalog"))
        );
        assert!(first_output.changes.retained.is_empty());

        runtime.element_state_mut("catalog").unwrap().scroll_y = 42.0;

        let second = catalog_scene("Flows");
        let second_output = runtime.update(&second, &stylesheet);

        assert!(
            second_output
                .changes
                .retained
                .contains(&ElementId::new("catalog"))
        );
        assert!(
            second_output
                .changes
                .removed
                .contains(&ElementId::new("Projects"))
        );
        assert_eq!(runtime.element_state("catalog").unwrap().scroll_y, 42.0);
    }

    #[test]
    fn style_rules_resolve_role_class_state_and_id_in_order() {
        let mut runtime = Runtime::default();
        let stylesheet = StyleSheet::new()
            .rule(
                StyleSelector::Role(ElementRole::Card),
                StylePatch::default()
                    .size(100.0, 40.0)
                    .background(Color::rgb(20, 20, 20)),
            )
            .rule(
                StyleSelector::Class("selected"),
                StylePatch::default().background(Color::rgb(35, 56, 78)),
            )
            .rule(
                StyleSelector::State(ElementStateSelector::Hovered),
                StylePatch::default().background(Color::rgb(40, 70, 95)),
            )
            .rule(StyleSelector::Id("card"), StylePatch::default().radius(7.0));
        let scene = Scene::build(Size::new(320.0, 200.0), |ui| {
            ui.element(
                "card",
                ElementSpec::new(ElementRole::Card)
                    .class("selected")
                    .interactive(),
                |_| {},
            );
        });

        let output = runtime.update_with_input(
            &scene,
            &stylesheet,
            RuntimeInput {
                pointer: Some(PointerInput {
                    position: Point::new(2.0, 2.0),
                    primary_delta: Point::ZERO,
                    primary_down: false,
                    primary_clicked: false,
                }),
                scroll_delta: Point::ZERO,
            },
        );
        let card = output.layout.find("card").unwrap();

        assert_eq!(card.style.background, Some(Color::rgb(40, 70, 95)));
        assert_eq!(card.style.radius, 7.0);
    }

    #[test]
    fn transitioned_state_rules_ease_visual_style_properties() {
        let mut runtime = Runtime::default();
        let stylesheet = StyleSheet::new()
            .rule(
                StyleSelector::Role(ElementRole::Card),
                StylePatch::default()
                    .size(100.0, 40.0)
                    .background(Color::rgb(20, 20, 20))
                    .transition(Transition::ease_out(0.24)),
            )
            .rule(
                StyleSelector::State(ElementStateSelector::Hovered),
                StylePatch::default().background(Color::rgb(40, 70, 95)),
            );
        let scene = Scene::build(Size::new(320.0, 200.0), |ui| {
            ui.element(
                "card",
                ElementSpec::new(ElementRole::Card).interactive(),
                |_| {},
            );
        });

        runtime.update(&scene, &stylesheet);

        let output = runtime.update_with_input(
            &scene,
            &stylesheet,
            RuntimeInput {
                pointer: Some(PointerInput {
                    position: Point::new(2.0, 2.0),
                    primary_delta: Point::ZERO,
                    primary_down: false,
                    primary_clicked: false,
                }),
                scroll_delta: Point::ZERO,
            },
        );
        let card = output.layout.find("card").unwrap();

        assert_eq!(card.style.background, Some(Color::rgb(31, 48, 62)));

        let output = (0..28)
            .map(|_| {
                runtime.update_with_input(
                    &scene,
                    &stylesheet,
                    RuntimeInput {
                        pointer: Some(PointerInput {
                            position: Point::new(2.0, 2.0),
                            primary_delta: Point::ZERO,
                            primary_down: false,
                            primary_clicked: false,
                        }),
                        scroll_delta: Point::ZERO,
                    },
                )
            })
            .last()
            .unwrap();
        let card = output.layout.find("card").unwrap();

        assert_eq!(card.style.background, Some(Color::rgb(40, 70, 95)));
    }

    #[test]
    fn column_layout_applies_padding_gap_and_margin() {
        let mut runtime = Runtime::default();
        let stylesheet = StyleSheet::new()
            .rule(
                StyleSelector::Id("catalog"),
                StylePatch::default().padding(Insets::all(10.0)).gap(4.0),
            )
            .rule(
                StyleSelector::Class("indented"),
                StylePatch::default().margin(Insets::symmetric(3.0, 2.0)),
            );
        let scene = Scene::build(Size::new(320.0, 200.0), |ui| {
            ui.element("catalog", ElementSpec::new(ElementRole::Panel), |ui| {
                ui.text("one", "One");
                ui.element(
                    "two",
                    ElementSpec::new(ElementRole::Text).class("indented"),
                    |_| {},
                );
            });
        });

        let output = runtime.update(&scene, &stylesheet);
        let one = output.layout.find("one").unwrap();
        let two = output.layout.find("two").unwrap();

        assert_eq!(one.rect.origin, Point::new(10.0, 10.0));
        assert_eq!(two.rect.origin, Point::new(13.0, 34.0));
    }

    #[test]
    fn fill_width_uses_parent_content_width_after_box_model() {
        let mut runtime = Runtime::default();
        let stylesheet = StyleSheet::new()
            .rule(
                StyleSelector::Id("panel"),
                StylePatch::default()
                    .size(200.0, 120.0)
                    .border_width(2.0)
                    .padding(Insets::symmetric(12.0, 8.0)),
            )
            .rule(
                StyleSelector::Id("row"),
                StylePatch::default()
                    .width_fill()
                    .height(Length::Px(24.0))
                    .margin(Insets::symmetric(3.0, 0.0)),
            );
        let scene = Scene::build(Size::new(320.0, 200.0), |ui| {
            ui.element("panel", ElementSpec::new(ElementRole::Panel), |ui| {
                ui.element("row", ElementSpec::new(ElementRole::Card), |_| {});
            });
        });

        let output = runtime.update(&scene, &stylesheet);
        let row = output.layout.find("row").unwrap();

        assert_eq!(row.rect.origin, Point::new(17.0, 10.0));
        assert_eq!(row.rect.size, Size::new(166.0, 24.0));
    }

    #[test]
    fn pointer_input_targets_interactive_owner_instead_of_inner_text() {
        let mut runtime = Runtime::default();
        let stylesheet = StyleSheet::new().rule(
            StyleSelector::Role(ElementRole::Card),
            StylePatch::default().size(100.0, 40.0),
        );
        let scene = Scene::build(Size::new(320.0, 200.0), |ui| {
            ui.element(
                "card",
                ElementSpec::new(ElementRole::Card).interactive(),
                |ui| {
                    ui.text("label", "Click target");
                },
            );
        });

        let output = runtime.update_with_input(
            &scene,
            &stylesheet,
            RuntimeInput {
                pointer: Some(PointerInput {
                    position: Point::new(4.0, 4.0),
                    primary_delta: Point::ZERO,
                    primary_down: true,
                    primary_clicked: true,
                }),
                scroll_delta: Point::ZERO,
            },
        );

        assert_eq!(output.hit_id, Some(ElementId::new("card")));
        let card_state = runtime.element_state("card").unwrap();
        assert!(card_state.hovered);
        assert!(card_state.pressed);
        assert_eq!(card_state.click_count, 1);

        let label_state = runtime.element_state("label").unwrap();
        assert!(label_state.hovered);
        assert!(!label_state.pressed);
        assert_eq!(label_state.click_count, 0);
    }

    #[test]
    fn scroll_delta_updates_hovered_scroll_container_state() {
        let mut runtime = Runtime::default();
        let stylesheet = StyleSheet::new()
            .rule(
                StyleSelector::Id("scroll-panel"),
                StylePatch::default()
                    .size(180.0, 80.0)
                    .padding(Insets::all(8.0))
                    .gap(4.0)
                    .border_width(5.0)
                    .overflow_y(Overflow::Scroll),
            )
            .rule(
                StyleSelector::Role(ElementRole::Card),
                StylePatch::default().size(140.0, 32.0),
            );
        let scene = Scene::build(Size::new(240.0, 160.0), |ui| {
            ui.element("scroll-panel", ElementSpec::new(ElementRole::Panel), |ui| {
                for index in 0..6 {
                    ui.element(
                        format!("row-{index}"),
                        ElementSpec::new(ElementRole::Card),
                        |_| {},
                    );
                }
            });
        });

        let output = runtime.update_with_input(
            &scene,
            &stylesheet,
            RuntimeInput {
                pointer: Some(PointerInput {
                    position: Point::new(20.0, 20.0),
                    primary_delta: Point::ZERO,
                    primary_down: false,
                    primary_clicked: false,
                }),
                scroll_delta: Point::new(0.0, -24.0),
            },
        );

        assert_eq!(output.hit_id, Some(ElementId::new("row-0")));
        assert_eq!(
            runtime.element_state("scroll-panel").unwrap().scroll_y,
            24.0
        );

        let output = runtime.update(&scene, &stylesheet);
        let first_row = output.layout.find("row-0").unwrap();
        assert_eq!(first_row.rect.origin.y, -11.0);
    }

    #[test]
    fn scroll_delta_is_clamped_when_content_does_not_overflow() {
        let mut runtime = Runtime::default();
        let stylesheet = StyleSheet::new()
            .rule(
                StyleSelector::Id("scroll-panel"),
                StylePatch::default()
                    .size(180.0, 120.0)
                    .padding(Insets::all(8.0))
                    .gap(4.0)
                    .overflow_y(Overflow::Scroll),
            )
            .rule(
                StyleSelector::Role(ElementRole::Card),
                StylePatch::default().size(140.0, 32.0),
            );
        let scene = Scene::build(Size::new(240.0, 160.0), |ui| {
            ui.element("scroll-panel", ElementSpec::new(ElementRole::Panel), |ui| {
                ui.element("row-0", ElementSpec::new(ElementRole::Card), |_| {});
                ui.element("row-1", ElementSpec::new(ElementRole::Card), |_| {});
            });
        });

        runtime.update_with_input(
            &scene,
            &stylesheet,
            RuntimeInput {
                pointer: Some(PointerInput {
                    position: Point::new(20.0, 20.0),
                    primary_delta: Point::ZERO,
                    primary_down: false,
                    primary_clicked: false,
                }),
                scroll_delta: Point::new(0.0, -240.0),
            },
        );

        assert_eq!(runtime.element_state("scroll-panel").unwrap().scroll_y, 0.0);
    }

    #[test]
    fn overflow_scroll_container_emits_draggable_scroll_chrome() {
        let mut runtime = Runtime::default();
        let stylesheet = StyleSheet::new()
            .rule(
                StyleSelector::Id("scroll-panel"),
                StylePatch::default()
                    .size(180.0, 80.0)
                    .padding(Insets::all(8.0))
                    .gap(4.0)
                    .overflow_y(Overflow::Scroll),
            )
            .rule(
                StyleSelector::Role(ElementRole::Card),
                StylePatch::default().size(140.0, 32.0),
            );
        let scene = Scene::build(Size::new(240.0, 160.0), |ui| {
            ui.element("scroll-panel", ElementSpec::new(ElementRole::Panel), |ui| {
                for index in 0..6 {
                    ui.element(
                        format!("row-{index}"),
                        ElementSpec::new(ElementRole::Card),
                        |_| {},
                    );
                }
            });
        });

        let output = runtime.update(&scene, &stylesheet);
        let chrome = output
            .scroll_chrome
            .iter()
            .find(|chrome| chrome.element_id == ElementId::new("scroll-panel"))
            .expect("overflowing panel should emit scroll chrome");
        assert!(chrome.max_scroll > 0.0);
        assert!(chrome.handle_rect.size.height < chrome.track_rect.size.height);

        let grab = Point::new(
            chrome.handle_rect.origin.x + chrome.handle_rect.size.width / 2.0,
            chrome.handle_rect.origin.y + chrome.handle_rect.size.height / 2.0,
        );
        runtime.update_with_input(
            &scene,
            &stylesheet,
            RuntimeInput {
                pointer: Some(PointerInput {
                    position: grab,
                    primary_delta: Point::ZERO,
                    primary_down: true,
                    primary_clicked: false,
                }),
                scroll_delta: Point::ZERO,
            },
        );
        runtime.update_with_input(
            &scene,
            &stylesheet,
            RuntimeInput {
                pointer: Some(PointerInput {
                    position: Point::new(grab.x, grab.y + 24.0),
                    primary_delta: Point::new(0.0, 24.0),
                    primary_down: true,
                    primary_clicked: false,
                }),
                scroll_delta: Point::ZERO,
            },
        );

        assert!(runtime.element_state("scroll-panel").unwrap().scroll_y > 0.0);
    }

    #[test]
    fn scroll_chrome_appears_on_container_hover_and_expands_on_hit_strip() {
        let mut runtime = Runtime::default();
        let stylesheet = StyleSheet::new()
            .rule(
                StyleSelector::Id("scroll-panel"),
                StylePatch::default()
                    .size(180.0, 80.0)
                    .padding(Insets::all(8.0))
                    .gap(4.0)
                    .overflow_y(Overflow::Scroll),
            )
            .rule(
                StyleSelector::Role(ElementRole::Card),
                StylePatch::default().size(140.0, 32.0),
            );
        let scene = Scene::build(Size::new(240.0, 160.0), |ui| {
            ui.element("scroll-panel", ElementSpec::new(ElementRole::Panel), |ui| {
                for index in 0..6 {
                    ui.element(
                        format!("row-{index}"),
                        ElementSpec::new(ElementRole::Card),
                        |_| {},
                    );
                }
            });
        });

        let output = runtime.update_with_input(
            &scene,
            &stylesheet,
            RuntimeInput {
                pointer: Some(PointerInput {
                    position: Point::new(20.0, 20.0),
                    primary_delta: Point::ZERO,
                    primary_down: false,
                    primary_clicked: false,
                }),
                scroll_delta: Point::ZERO,
            },
        );
        let chrome = output
            .scroll_chrome
            .iter()
            .find(|chrome| chrome.element_id == ElementId::new("scroll-panel"))
            .unwrap();
        assert!(chrome.visible);
        assert!(!chrome.expanded);
        assert!(!chrome.hovered);
        assert_eq!(chrome.handle_rect.size.width, 2.0);
        assert_eq!(chrome.hit_rect.size.width, 12.0);

        let output = runtime.update_with_input(
            &scene,
            &stylesheet,
            RuntimeInput {
                pointer: Some(PointerInput {
                    position: Point::new(170.0, 20.0),
                    primary_delta: Point::ZERO,
                    primary_down: false,
                    primary_clicked: false,
                }),
                scroll_delta: Point::ZERO,
            },
        );
        let chrome = output
            .scroll_chrome
            .iter()
            .find(|chrome| chrome.element_id == ElementId::new("scroll-panel"))
            .unwrap();
        assert!(chrome.visible);
        assert!(chrome.expanded);
        assert!(chrome.hovered);
        assert_eq!(chrome.handle_rect.size.width, 10.0);
    }

    fn catalog_scene(title_id: &str) -> Scene {
        Scene::build(Size::new(240.0, 480.0), |ui| {
            ui.element(
                "catalog",
                ElementSpec::new(ElementRole::Panel).class("catalog"),
                |ui| {
                    ui.text(title_id, title_id);
                    ui.element(
                        "project-card",
                        ElementSpec::new(ElementRole::Card).class("selected"),
                        |ui| {
                            ui.text("project-name", "Customer 360");
                        },
                    );
                },
            );
        })
    }

    fn probe_stylesheet() -> StyleSheet {
        StyleSheet::new()
            .rule(
                StyleSelector::Class("catalog"),
                StylePatch::default()
                    .size(180.0, 40.0)
                    .padding(Insets::all(12.0))
                    .gap(8.0)
                    .overflow_y(Overflow::Scroll),
            )
            .rule(
                StyleSelector::Role(ElementRole::Card),
                StylePatch::default().size(180.0, 48.0),
            )
    }
}
