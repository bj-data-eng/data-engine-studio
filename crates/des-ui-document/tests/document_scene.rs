use des_ui_document::{
    AlignItems, ComputedStyle, Direction, DocumentEngine, DocumentEventKind, DocumentInput,
    DocumentScene, ElementId, ElementRole, ElementSpec, ElementStateSelector, Insets,
    JustifyContent, Length, Overflow, Point, PointerInput, Rect, ScrollAxis, Size, Style,
    StyleSelector, StyleSheet,
};
use layout_engine::prelude::{
    AlignItems as LayoutAlignItems, Dimension, FlexDirection,
    JustifyContent as LayoutJustifyContent, LengthPercentageAuto, Size as LayoutSize, length,
    percent,
};
use layout_engine::style::Overflow as LayoutOverflow;
use std::collections::HashMap;

#[test]
fn scene_reparents_existing_element_without_reallocating_layout_node() {
    let mut scene = DocumentScene::new(Size::new(800.0, 600.0));
    scene
        .append_element("root", "panel", ElementSpec::new(ElementRole::Panel))
        .unwrap();
    scene
        .append_text(
            "panel",
            "label",
            ElementSpec::new(ElementRole::Text),
            "Retained text",
        )
        .unwrap();

    let original_layout_node = scene.layout_node("label").unwrap();

    scene.reparent("label", "root").unwrap();

    assert_eq!(scene.layout_node("label"), Some(original_layout_node));
    assert_eq!(scene.parent("label").unwrap(), Some(ElementId::new("root")));
    assert!(scene.children("panel").unwrap().is_empty());
    assert_eq!(
        scene.children("root").unwrap(),
        vec![ElementId::new("panel"), ElementId::new("label")]
    );
}

#[test]
fn scene_remove_prunes_descendants_from_model_and_layout_graph() {
    let mut scene = DocumentScene::new(Size::new(800.0, 600.0));
    scene
        .append_element("root", "panel", ElementSpec::new(ElementRole::Panel))
        .unwrap();
    scene
        .append_text(
            "panel",
            "label",
            ElementSpec::new(ElementRole::Text),
            "Retained text",
        )
        .unwrap();

    scene.remove("panel").unwrap();

    assert_eq!(scene.children("root").unwrap(), Vec::<ElementId>::new());
    assert!(scene.layout_node("panel").is_none());
    assert!(scene.layout_node("label").is_none());
    assert!(scene.parent("label").is_err());
}

#[test]
fn scene_applies_document_style_to_existing_layout_node() {
    let mut scene = DocumentScene::new(Size::new(800.0, 600.0));
    scene
        .append_element("root", "panel", ElementSpec::new(ElementRole::Panel))
        .unwrap();
    let original_layout_node = scene.layout_node("panel").unwrap();

    let mut style = ComputedStyle::default();
    style.direction = Direction::Row;
    style.wrap = true;
    style.align_items = AlignItems::Stretch;
    style.justify_content = JustifyContent::SpaceBetween;
    style.gap = 12.0;
    style.margin = Insets::symmetric(8.0, 4.0);
    style.padding = Insets::all(6.0);
    style.width = Length::Percent(0.5);
    style.height = Length::Px(240.0);
    style.min_size = Size::new(120.0, 80.0);
    style.max_size = Size::new(640.0, 480.0);
    style.overflow_x = Overflow::Scroll;

    scene.apply_computed_style("panel", &style).unwrap();

    assert_eq!(scene.layout_node("panel"), Some(original_layout_node));
    let layout_style = scene.layout_style("panel").unwrap();
    assert_eq!(layout_style.flex_direction, FlexDirection::Row);
    assert_eq!(
        layout_style.flex_wrap,
        layout_engine::prelude::FlexWrap::Wrap
    );
    assert_eq!(layout_style.align_items, Some(LayoutAlignItems::Stretch));
    assert_eq!(
        layout_style.justify_content,
        Some(LayoutJustifyContent::SpaceBetween)
    );
    assert_eq!(layout_style.gap, LayoutSize::length(12.0));
    assert_eq!(
        layout_style.margin,
        layout_engine::prelude::Rect {
            left: length::<_, LengthPercentageAuto>(8.0),
            right: length::<_, LengthPercentageAuto>(8.0),
            top: length::<_, LengthPercentageAuto>(4.0),
            bottom: length::<_, LengthPercentageAuto>(4.0),
        }
    );
    assert_eq!(
        layout_style.padding,
        layout_engine::prelude::Rect::length(6.0)
    );
    assert_eq!(
        layout_style.size,
        LayoutSize {
            width: percent::<_, Dimension>(0.5),
            height: length::<_, Dimension>(240.0),
        }
    );
    assert_eq!(
        layout_style.min_size,
        LayoutSize {
            width: length::<_, Dimension>(120.0),
            height: length::<_, Dimension>(80.0),
        }
    );
    assert_eq!(
        layout_style.max_size,
        LayoutSize {
            width: length::<_, Dimension>(640.0),
            height: length::<_, Dimension>(480.0),
        }
    );
    assert_eq!(layout_style.overflow.x, LayoutOverflow::Scroll);
    assert_eq!(layout_style.overflow.y, LayoutOverflow::Visible);
}

#[test]
fn scene_computes_layout_rects_from_retained_graph() {
    let mut scene = DocumentScene::new(Size::new(800.0, 600.0));
    scene
        .append_element("root", "panel", ElementSpec::new(ElementRole::Panel))
        .unwrap();

    let mut style = ComputedStyle::default();
    style.width = Length::Px(200.0);
    style.height = Length::Px(100.0);
    scene.apply_computed_style("panel", &style).unwrap();

    scene.compute_layout().unwrap();

    assert_eq!(
        scene.layout_rect("root").unwrap(),
        Rect::new(0.0, 0.0, 800.0, 600.0)
    );
    assert_eq!(
        scene.layout_rect("panel").unwrap(),
        Rect::new(0.0, 0.0, 200.0, 100.0)
    );
}

#[test]
fn scene_resolves_stylesheet_over_retained_elements() {
    let mut scene = DocumentScene::new(Size::new(800.0, 600.0));
    scene
        .append_element(
            "root",
            "first",
            ElementSpec::new(ElementRole::Panel).class("primary"),
        )
        .unwrap();
    scene
        .append_element("root", "second", ElementSpec::new(ElementRole::Panel))
        .unwrap();
    let first_node = scene.layout_node("first").unwrap();
    let second_node = scene.layout_node("second").unwrap();

    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::Role(ElementRole::Root),
            Style::default().direction(Direction::Row),
        )
        .rule(
            StyleSelector::class("primary"),
            Style::default().width(Length::Px(120.0)),
        )
        .rule(
            StyleSelector::first_child(),
            Style::default().height(Length::Px(40.0)),
        )
        .rule(
            StyleSelector::id_state("second", ElementStateSelector::Hovered),
            Style::default().width(Length::Px(240.0)),
        );
    let mut states = HashMap::new();
    let mut second_state = des_ui_document::ElementState::default();
    second_state.hovered = true;
    states.insert(ElementId::new("second"), second_state);

    scene.apply_stylesheet(&stylesheet, &states).unwrap();

    assert_eq!(scene.layout_node("first"), Some(first_node));
    assert_eq!(scene.layout_node("second"), Some(second_node));
    assert_eq!(
        scene.layout_style("root").unwrap().flex_direction,
        FlexDirection::Row
    );
    assert_eq!(
        scene.layout_style("root").unwrap().size,
        LayoutSize {
            width: length::<_, Dimension>(800.0),
            height: length::<_, Dimension>(600.0),
        }
    );
    assert_eq!(
        scene.layout_style("first").unwrap().size,
        LayoutSize {
            width: length::<_, Dimension>(120.0),
            height: length::<_, Dimension>(40.0),
        }
    );
    assert_eq!(
        scene.layout_style("second").unwrap().size.width,
        length::<_, Dimension>(240.0)
    );
}

#[test]
fn scene_emits_resolved_element_tree_from_retained_layout() {
    let mut scene = DocumentScene::new(Size::new(800.0, 600.0));
    scene
        .append_element(
            "root",
            "panel",
            ElementSpec::new(ElementRole::Panel)
                .class("primary")
                .interactive(),
        )
        .unwrap();
    scene
        .append_text(
            "panel",
            "label",
            ElementSpec::new(ElementRole::Text).selectable_text(),
            "Retained text",
        )
        .unwrap();

    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("panel"),
            Style::default()
                .size(200.0, 100.0)
                .padding(Insets::all(10.0)),
        )
        .rule(
            StyleSelector::id("label"),
            Style::default().size(80.0, 20.0),
        );

    scene
        .apply_stylesheet(&stylesheet, &HashMap::new())
        .unwrap();
    scene.compute_layout().unwrap();

    let root = scene.resolved_layout().unwrap();
    let panel = root.find("panel").unwrap();
    let label = root.find("label").unwrap();

    assert_eq!(root.rect, Rect::new(0.0, 0.0, 800.0, 600.0));
    assert_eq!(panel.role, ElementRole::Panel);
    assert_eq!(panel.classes, vec!["primary".into()]);
    assert_eq!(panel.rect, Rect::new(0.0, 0.0, 200.0, 100.0));
    assert_eq!(panel.style.padding, Insets::all(10.0));
    assert!(panel.interactive);
    assert_eq!(label.text.as_deref(), Some("Retained text"));
    assert_eq!(label.rect, Rect::new(10.0, 10.0, 80.0, 20.0));
    assert!(label.selectable_text);
    assert!(label.copyable_text);
}

#[test]
fn scene_resolves_styles_computes_layout_and_emits_tree_in_one_pass() {
    let mut scene = DocumentScene::new(Size::new(800.0, 600.0));
    scene
        .append_element("root", "panel", ElementSpec::new(ElementRole::Panel))
        .unwrap();
    let stylesheet = StyleSheet::new().rule(
        StyleSelector::id("panel"),
        Style::default().size(320.0, 180.0),
    );

    let root = scene.resolve_layout(&stylesheet, &HashMap::new()).unwrap();

    assert_eq!(
        root.find("panel").unwrap().rect,
        Rect::new(0.0, 0.0, 320.0, 180.0)
    );
}

#[test]
fn document_engine_can_update_from_retained_scene() {
    let mut scene = DocumentScene::new(Size::new(800.0, 600.0));
    scene
        .append_element("root", "panel", ElementSpec::new(ElementRole::Panel))
        .unwrap();
    let stylesheet = StyleSheet::new().rule(
        StyleSelector::id("panel"),
        Style::default().size(320.0, 180.0),
    );
    let mut engine = DocumentEngine::default();

    let output = engine.update_scene(&mut scene, &stylesheet);

    assert_eq!(
        output.layout.find("panel").unwrap().rect,
        Rect::new(0.0, 0.0, 320.0, 180.0)
    );
    assert_eq!(
        output.changes.created,
        vec![ElementId::new("panel"), ElementId::new("root")]
    );
    assert_eq!(output.metrics.element_count, 2);
    assert_eq!(engine.element_state("panel"), Some(&Default::default()));
}

#[test]
fn document_engine_update_scene_reports_scroll_chrome_for_overflow() {
    let mut scene = DocumentScene::new(Size::new(800.0, 600.0));
    scene
        .append_element("root", "scroll", ElementSpec::new(ElementRole::Panel))
        .unwrap();
    scene
        .append_element("scroll", "content", ElementSpec::new(ElementRole::Panel))
        .unwrap();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("scroll"),
            Style::default()
                .size(100.0, 100.0)
                .overflow_y(Overflow::Scroll),
        )
        .rule(
            StyleSelector::id("content"),
            Style::default().size(100.0, 300.0),
        );
    let mut engine = DocumentEngine::default();

    let output = engine.update_scene(&mut scene, &stylesheet);

    assert_eq!(output.scroll_chrome.len(), 1);
    assert_eq!(output.scroll_chrome[0].element_id, ElementId::new("scroll"));
    assert_eq!(output.scroll_chrome[0].max_scroll, 200.0);
    assert_eq!(output.metrics.scroll_chrome_count, 1);
}

#[test]
fn document_engine_update_scene_with_input_clicks_interactive_element() {
    let mut scene = DocumentScene::new(Size::new(800.0, 600.0));
    scene
        .append_element(
            "root",
            "button",
            ElementSpec::new(ElementRole::Control).interactive(),
        )
        .unwrap();
    let stylesheet = StyleSheet::new().rule(
        StyleSelector::id("button"),
        Style::default().size(120.0, 40.0),
    );
    let mut engine = DocumentEngine::default();

    let output = engine.update_scene_with_input(
        &mut scene,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(10.0, 10.0),
                primary_delta: Point::ZERO,
                primary_down: true,
                primary_pressed: true,
                primary_clicked: true,
                primary_click_count: 1,
                secondary_clicked: false,
                time_seconds: 0.0,
            }),
            scroll_delta: Point::ZERO,
        },
    );

    assert_eq!(output.hit_id, Some(ElementId::new("button")));
    assert!(engine.element_state("button").unwrap().hovered);
    assert!(engine.element_state("button").unwrap().pressed);
    assert!(
        output
            .events
            .iter()
            .any(|event| event.target == ElementId::new("button")
                && event.kind == DocumentEventKind::Clicked)
    );
}

#[test]
fn document_engine_update_scene_with_input_scrolls_overflow_container() {
    let mut scene = DocumentScene::new(Size::new(800.0, 600.0));
    scene
        .append_element("root", "scroll", ElementSpec::new(ElementRole::Panel))
        .unwrap();
    scene
        .append_element("scroll", "content", ElementSpec::new(ElementRole::Panel))
        .unwrap();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("scroll"),
            Style::default()
                .size(100.0, 100.0)
                .overflow_y(Overflow::Scroll),
        )
        .rule(
            StyleSelector::id("content"),
            Style::default().size(100.0, 300.0),
        );
    let mut engine = DocumentEngine::default();

    let output = engine.update_scene_with_input(
        &mut scene,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(50.0, 50.0),
                primary_delta: Point::ZERO,
                primary_down: false,
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: false,
                time_seconds: 0.0,
            }),
            scroll_delta: Point::new(0.0, -40.0),
        },
    );

    assert_eq!(engine.element_state("scroll").unwrap().scroll_y, 40.0);
    assert!(
        output
            .events
            .iter()
            .any(|event| event.target == ElementId::new("scroll")
                && event.kind == DocumentEventKind::Scrolled(ScrollAxis::Vertical))
    );
    assert!(output.metrics.input_changed_state);
}

#[test]
fn document_engine_update_scene_with_input_rerenders_state_styles() {
    let mut scene = DocumentScene::new(Size::new(800.0, 600.0));
    scene
        .append_element(
            "root",
            "button",
            ElementSpec::new(ElementRole::Control).interactive(),
        )
        .unwrap();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("button"),
            Style::default().size(120.0, 40.0),
        )
        .rule(
            StyleSelector::id_state("button", ElementStateSelector::Hovered),
            Style::default().size(160.0, 40.0),
        );
    let mut engine = DocumentEngine::default();

    let output = engine.update_scene_with_input(
        &mut scene,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(10.0, 10.0),
                primary_delta: Point::ZERO,
                primary_down: false,
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: false,
                time_seconds: 0.0,
            }),
            scroll_delta: Point::ZERO,
        },
    );

    assert_eq!(
        output.layout.find("button").unwrap().rect,
        Rect::new(0.0, 0.0, 160.0, 40.0)
    );
    assert!(!output.metrics.reused_input_layout);
}
