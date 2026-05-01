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
        self.children.push(Element {
            id: id.into(),
            spec: ElementSpec::new(ElementRole::Text),
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
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct StylePatch {
    pub direction: Option<Direction>,
    pub gap: Option<f32>,
    pub margin: Option<Insets>,
    pub padding: Option<Insets>,
    pub size: Option<Size>,
    pub min_size: Option<Size>,
    pub background: Option<Color>,
    pub border: Option<Color>,
    pub radius: Option<f32>,
    pub overflow_y: Option<Overflow>,
    pub z_index: Option<i32>,
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
        self.size = Some(Size::new(width, height));
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
}

#[derive(Clone, Debug, PartialEq)]
pub struct ComputedStyle {
    pub direction: Direction,
    pub gap: f32,
    pub margin: Insets,
    pub padding: Insets,
    pub size: Option<Size>,
    pub min_size: Size,
    pub background: Option<Color>,
    pub border: Option<Color>,
    pub radius: f32,
    pub overflow_y: Overflow,
    pub z_index: i32,
}

impl Default for ComputedStyle {
    fn default() -> Self {
        Self {
            direction: Direction::Column,
            gap: 0.0,
            margin: Insets::ZERO,
            padding: Insets::ZERO,
            size: None,
            min_size: Size::new(0.0, 0.0),
            background: None,
            border: None,
            radius: 0.0,
            overflow_y: Overflow::Visible,
            z_index: 0,
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
        if let Some(value) = patch.size {
            self.size = Some(value);
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
        if let Some(value) = patch.radius {
            self.radius = value;
        }
        if let Some(value) = patch.overflow_y {
            self.overflow_y = value;
        }
        if let Some(value) = patch.z_index {
            self.z_index = value;
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
    pub focused: bool,
    pub click_count: u32,
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
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RuntimeInput {
    pub pointer: Option<PointerInput>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointerInput {
    pub position: Point,
    pub primary_down: bool,
    pub primary_clicked: bool,
}

#[derive(Default)]
pub struct Runtime {
    states: HashMap<ElementId, ElementState>,
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
        let input_layout = layout_element(
            &scene.root,
            Rect::new(0.0, 0.0, scene.viewport.width, scene.viewport.height),
            stylesheet,
            &self.states,
        );
        let hit_id = self.apply_input(&input_layout, input);
        let layout = layout_element(
            &scene.root,
            Rect::new(0.0, 0.0, scene.viewport.width, scene.viewport.height),
            stylesheet,
            &self.states,
        );

        RuntimeOutput {
            changes,
            layout,
            hit_id,
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

    fn apply_input(&mut self, layout: &LayoutFrame, input: RuntimeInput) -> Option<ElementId> {
        for state in self.states.values_mut() {
            state.hovered = false;
            state.pressed = false;
        }

        let pointer = input.pointer?;
        let path = hit_path(layout, pointer.position)?;
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
}

fn layout_element(
    element: &Element,
    parent_rect: Rect,
    stylesheet: &StyleSheet,
    states: &HashMap<ElementId, ElementState>,
) -> LayoutFrame {
    let style = resolve_style(element, stylesheet, states.get(&element.id));
    let rect = element_rect(element, &style, parent_rect, stylesheet, states);
    let content_rect = rect.inset(style.padding);
    let children = layout_children(element, &style, content_rect, stylesheet, states);

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
) -> Vec<LayoutFrame> {
    let mut cursor = content_rect.origin;
    let mut frames = Vec::with_capacity(element.children.len());

    for child in &element.children {
        let child_style = resolve_style(child, stylesheet, states.get(&child.id));
        let measured = measure_element(child, &child_style, content_rect.size, stylesheet, states);
        let child_rect = Rect::new(cursor.x, cursor.y, measured.width, measured.height);
        frames.push(layout_element(child, child_rect, stylesheet, states));

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
    if let Some(size) = style.size {
        return size;
    }

    match element.spec.role {
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
                (content.width + style.padding.horizontal()).max(style.min_size.width),
                (content.height + style.padding.vertical()).max(style.min_size.height),
            )
        }
    }
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
        let child_size = measure_element(child, &child_style, parent_size, stylesheet, states);
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

    Size::new(width.min(parent_size.width), height.min(parent_size.height))
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
                    primary_down: false,
                    primary_clicked: false,
                }),
            },
        );
        let card = output.layout.find("card").unwrap();

        assert_eq!(card.style.background, Some(Color::rgb(40, 70, 95)));
        assert_eq!(card.style.radius, 7.0);
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
                    primary_down: true,
                    primary_clicked: true,
                }),
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
