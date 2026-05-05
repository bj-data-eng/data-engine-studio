use des_ui_document::{
    AlignItems, ComputedStyle, Direction, DocumentScene, ElementId, ElementRole, ElementSpec,
    ElementStateSelector, Insets, JustifyContent, Length, Overflow, Rect, Size, Style,
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
