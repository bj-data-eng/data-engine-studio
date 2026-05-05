use des_ui_document::{
    AlignContent, AlignItems, Color, ComputedStyle, DocumentEngine, DocumentEventKind,
    DocumentInput, DocumentScene, ElementId, ElementRole, ElementSpec, ElementStateSelector,
    FlexDirection, FlexWrap, Insets, JustifyContent, Length, Overflow, Point, PointerInput, Rect,
    ScrollAxis, Size, Style, StyleSelector, StyleSheet, TableCellSpec, TableColumnSpec, TableSpec,
    TableTrackSize, TextLayoutRequest, TextLayoutResult, TextMeasurer, TextMeasurerKey,
    TextWrapMode, Transition,
};
use layout_engine::prelude::{
    AlignContent as LayoutAlignContent, AlignItems as LayoutAlignItems, Dimension,
    FlexDirection as LayoutFlexDirection, JustifyContent as LayoutJustifyContent,
    LengthPercentageAuto, Size as LayoutSize, length, percent,
};
use layout_engine::style::Overflow as LayoutOverflow;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Default)]
struct RecordingTextMeasurer {
    requests: Vec<(String, f32)>,
}

impl TextMeasurer for RecordingTextMeasurer {
    fn cache_key(&self) -> TextMeasurerKey {
        TextMeasurerKey::new("recording")
    }

    fn measure_text(&mut self, request: TextLayoutRequest<'_>) -> TextLayoutResult {
        self.requests
            .push((request.text.to_string(), request.wrap_width));
        TextLayoutResult {
            size: Size::new(64.0, 18.0),
            line_count: 1,
            elided: false,
        }
    }
}

static COMPUTE_TEXT_MEASURE_COUNT: AtomicUsize = AtomicUsize::new(0);

#[derive(Default)]
struct CountingTextMeasurer;

impl TextMeasurer for CountingTextMeasurer {
    fn cache_key(&self) -> TextMeasurerKey {
        TextMeasurerKey::new("counting")
    }

    fn measure_text(&mut self, request: TextLayoutRequest<'_>) -> TextLayoutResult {
        if request.text == "Measured" {
            COMPUTE_TEXT_MEASURE_COUNT.fetch_add(1, Ordering::SeqCst);
        }
        TextLayoutResult {
            size: Size::new(64.0, 18.0),
            line_count: 1,
            elided: false,
        }
    }
}

fn hover_input(position: Point) -> DocumentInput {
    DocumentInput {
        pointer: Some(PointerInput {
            position,
            primary_delta: Point::ZERO,
            primary_down: false,
            primary_pressed: false,
            primary_clicked: false,
            primary_click_count: 0,
            secondary_clicked: false,
            time_seconds: 0.0,
        }),
        scroll_delta: Point::ZERO,
    }
}

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
    style.flex_direction = FlexDirection::Row;
    style.flex_wrap = FlexWrap::Wrap;
    style.flex_basis = Length::Px(88.0);
    style.flex_grow = 2.0;
    style.flex_shrink = 0.5;
    style.align_content = AlignContent::SpaceAround;
    style.align_items = AlignItems::FlexEnd;
    style.align_self = Some(AlignItems::Baseline);
    style.justify_content = JustifyContent::SpaceEvenly;
    style.gap = 12.0;
    style.row_gap = 10.0;
    style.column_gap = 14.0;
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
    assert_eq!(layout_style.flex_direction, LayoutFlexDirection::Row);
    assert_eq!(
        layout_style.flex_wrap,
        layout_engine::prelude::FlexWrap::Wrap
    );
    assert_eq!(layout_style.flex_basis, length::<_, Dimension>(88.0));
    assert_eq!(layout_style.flex_grow, 2.0);
    assert_eq!(layout_style.flex_shrink, 0.5);
    assert_eq!(
        layout_style.align_content,
        Some(LayoutAlignContent::SpaceAround)
    );
    assert_eq!(layout_style.align_items, Some(LayoutAlignItems::FlexEnd));
    assert_eq!(layout_style.align_self, Some(LayoutAlignItems::Baseline));
    assert_eq!(
        layout_style.justify_content,
        Some(LayoutJustifyContent::SpaceEvenly)
    );
    assert_eq!(
        layout_style.gap,
        LayoutSize {
            width: length(14.0),
            height: length(10.0),
        }
    );
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
fn scene_fill_size_does_not_imply_flex_layout_style() {
    let mut scene = DocumentScene::new(Size::new(800.0, 600.0));
    scene
        .append_element("root", "panel", ElementSpec::new(ElementRole::Panel))
        .unwrap();

    let mut style = ComputedStyle::default();
    style.width = Length::Fill;
    style.height = Length::Fill;

    scene.apply_computed_style("panel", &style).unwrap();

    let layout_style = scene.layout_style("panel").unwrap();
    assert_eq!(layout_style.align_self, None);
    assert_eq!(layout_style.flex_grow, 0.0);
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
            Style::default().flex_direction(FlexDirection::Row),
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
        LayoutFlexDirection::Row
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
fn scene_does_not_dirty_layout_graph_when_resolved_layout_style_is_unchanged() {
    let mut scene = DocumentScene::new(Size::new(800.0, 600.0));
    scene
        .append_element("root", "panel", ElementSpec::new(ElementRole::Panel))
        .unwrap();
    let stylesheet = StyleSheet::new().rule(
        StyleSelector::id("panel"),
        Style::default()
            .width(Length::Px(120.0))
            .height(Length::Px(40.0)),
    );

    let first_report = scene
        .apply_stylesheet(&stylesheet, &HashMap::new())
        .unwrap();
    scene.compute_layout().unwrap();
    assert!(first_report.layout_changed);
    assert_eq!(first_report.visited, 2);
    assert!(!scene.layout_dirty("root").unwrap());
    assert!(!scene.layout_dirty("panel").unwrap());

    let second_report = scene
        .apply_stylesheet(&stylesheet, &HashMap::new())
        .unwrap();

    assert!(!second_report.changed());
    assert_eq!(second_report.visited, 2);
    assert!(!scene.layout_dirty("root").unwrap());
    assert!(!scene.layout_dirty("panel").unwrap());
}

#[test]
fn scene_paint_only_style_update_does_not_dirty_layout_graph() {
    let mut scene = DocumentScene::new(Size::new(800.0, 600.0));
    scene
        .append_element("root", "panel", ElementSpec::new(ElementRole::Panel))
        .unwrap();
    let layout_stylesheet = StyleSheet::new().rule(
        StyleSelector::id("panel"),
        Style::default()
            .width(Length::Px(120.0))
            .height(Length::Px(40.0)),
    );
    scene
        .apply_stylesheet(&layout_stylesheet, &HashMap::new())
        .unwrap();
    scene.compute_layout().unwrap();

    let paint_stylesheet = layout_stylesheet.clone().rule(
        StyleSelector::id("panel"),
        Style::default().background(des_ui_document::Color::rgb(16, 24, 32)),
    );
    let report = scene
        .apply_stylesheet(&paint_stylesheet, &HashMap::new())
        .unwrap();

    assert!(report.paint_changed);
    assert!(!report.layout_changed);
    assert!(!scene.layout_dirty("root").unwrap());
    assert!(!scene.layout_dirty("panel").unwrap());
    assert_eq!(
        scene
            .resolved_layout()
            .unwrap()
            .find("panel")
            .unwrap()
            .style
            .background,
        Some(des_ui_document::Color::rgb(16, 24, 32))
    );
}

#[test]
fn scene_layout_style_update_dirties_changed_node_and_ancestors() {
    let mut scene = DocumentScene::new(Size::new(800.0, 600.0));
    scene
        .append_element("root", "panel", ElementSpec::new(ElementRole::Panel))
        .unwrap();
    let stylesheet = StyleSheet::new().rule(
        StyleSelector::id("panel"),
        Style::default()
            .width(Length::Px(120.0))
            .height(Length::Px(40.0)),
    );
    scene
        .apply_stylesheet(&stylesheet, &HashMap::new())
        .unwrap();
    scene.compute_layout().unwrap();

    let changed_stylesheet = StyleSheet::new().rule(
        StyleSelector::id("panel"),
        Style::default()
            .width(Length::Px(240.0))
            .height(Length::Px(40.0)),
    );
    scene
        .apply_stylesheet(&changed_stylesheet, &HashMap::new())
        .unwrap();

    assert!(scene.layout_dirty("root").unwrap());
    assert!(scene.layout_dirty("panel").unwrap());
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
fn scene_emits_text_layout_from_retained_layout() {
    let mut scene = DocumentScene::new(Size::new(800.0, 600.0));
    scene
        .append_text(
            "root",
            "label",
            ElementSpec::new(ElementRole::Text),
            "Retained text",
        )
        .unwrap();
    let stylesheet = StyleSheet::new().rule(
        StyleSelector::id("label"),
        Style::default()
            .size(120.0, 40.0)
            .padding(Insets::all(8.0))
            .border_width(2.0)
            .text_wrap(TextWrapMode::Wrap),
    );
    let mut text_measurer = RecordingTextMeasurer::default();

    scene
        .apply_stylesheet(&stylesheet, &HashMap::new())
        .unwrap();
    scene.compute_layout().unwrap();
    let root = scene
        .resolved_layout_with_text_measurer(&mut text_measurer)
        .unwrap();
    let label = root.find("label").unwrap();

    assert_eq!(
        label.text_layout,
        Some(TextLayoutResult {
            size: Size::new(64.0, 18.0),
            line_count: 1,
            elided: false,
        })
    );
    assert_eq!(
        text_measurer.requests,
        vec![("Retained text".to_string(), 100.0)]
    );
}

#[test]
fn scene_uses_text_measurement_for_auto_sized_text_layout() {
    let mut scene = DocumentScene::new(Size::new(800.0, 600.0));
    scene
        .append_text(
            "root",
            "label",
            ElementSpec::new(ElementRole::Text),
            "Measured",
        )
        .unwrap();
    let stylesheet = StyleSheet::new().rule(
        StyleSelector::id("label"),
        Style::default().padding(Insets::all(8.0)).border_width(2.0),
    );
    let mut text_measurer = RecordingTextMeasurer::default();

    scene
        .apply_stylesheet(&stylesheet, &HashMap::new())
        .unwrap();
    scene
        .compute_layout_with_text_measurer(&mut text_measurer)
        .unwrap();
    let label = scene
        .resolved_layout_with_text_measurer(&mut text_measurer)
        .unwrap()
        .find("label")
        .unwrap()
        .clone();

    assert_eq!(label.rect, Rect::new(0.0, 0.0, 84.0, 38.0));
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
fn scene_resolve_layout_skips_compute_when_layout_graph_is_clean() {
    COMPUTE_TEXT_MEASURE_COUNT.store(0, Ordering::SeqCst);
    let mut scene = DocumentScene::new(Size::new(800.0, 600.0));
    scene
        .append_text(
            "root",
            "label",
            ElementSpec::new(ElementRole::Text),
            "Measured",
        )
        .unwrap();
    let stylesheet = StyleSheet::new().rule(
        StyleSelector::id("label"),
        Style::default().width(Length::Px(100.0)),
    );
    let mut text_measurer = CountingTextMeasurer;

    scene
        .resolve_layout_with_text_measurer(&stylesheet, &HashMap::new(), &mut text_measurer)
        .unwrap();
    let first_resolve_count = COMPUTE_TEXT_MEASURE_COUNT.load(Ordering::SeqCst);
    assert!(first_resolve_count > 1);

    scene
        .resolve_layout_with_text_measurer(&stylesheet, &HashMap::new(), &mut text_measurer)
        .unwrap();

    assert_eq!(
        COMPUTE_TEXT_MEASURE_COUNT.load(Ordering::SeqCst),
        first_resolve_count + 1
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
fn document_engine_update_scene_eases_transitioned_paint_styles() {
    let mut scene = DocumentScene::new(Size::new(320.0, 200.0));
    scene
        .append_element(
            "root",
            "card",
            ElementSpec::new(ElementRole::Card).interactive(),
        )
        .unwrap();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::Role(ElementRole::Card),
            Style::default()
                .size(100.0, 40.0)
                .background(Color::rgb(20, 20, 20))
                .transition(Transition::ease_out(0.24)),
        )
        .rule(
            StyleSelector::State(ElementStateSelector::Hovered),
            Style::default().background(Color::rgb(40, 70, 95)),
        );
    let mut engine = DocumentEngine::default();
    engine.update_scene(&mut scene, &stylesheet);

    let output = engine.update_scene_with_input(
        &mut scene,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(2.0, 2.0),
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
    let card = output.layout.find("card").unwrap();

    assert_eq!(card.style.background, Some(Color::rgb(31, 48, 62)));
    assert!(output.animating);
    assert!(output.metrics.animation_changed_style);
    assert!(!output.metrics.animation_changed_layout);
    assert!(output.metrics.animation_changed_paint);

    let output = engine.update_scene_with_input(
        &mut scene,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(2.0, 2.0),
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
    let card = output.layout.find("card").unwrap();

    assert!(card.style.background.unwrap().r > 31);
    assert!(output.animating);
    assert!(output.metrics.animation_changed_paint);
}

#[test]
fn document_engine_update_scene_transitioned_paint_styles_settle_to_target() {
    let mut scene = DocumentScene::new(Size::new(320.0, 200.0));
    scene
        .append_element(
            "root",
            "card",
            ElementSpec::new(ElementRole::Card).interactive(),
        )
        .unwrap();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::Role(ElementRole::Card),
            Style::default()
                .size(100.0, 40.0)
                .background(Color::rgb(20, 20, 20))
                .transition(Transition::ease_out(0.24)),
        )
        .rule(
            StyleSelector::State(ElementStateSelector::Hovered),
            Style::default().background(Color::rgb(40, 70, 95)),
        );
    let hover_input = hover_input(Point::new(2.0, 2.0));
    let mut engine = DocumentEngine::default();

    engine.update_scene(&mut scene, &stylesheet);
    let output = (0..30)
        .map(|_| engine.update_scene_with_input(&mut scene, &stylesheet, hover_input))
        .last()
        .unwrap();
    let card = output.layout.find("card").unwrap();

    assert_eq!(card.style.background, Some(Color::rgb(40, 70, 95)));
    assert!(!output.animating);
    assert!(!output.metrics.animation_changed_style);

    let output = engine.update_scene_with_input(&mut scene, &stylesheet, hover_input);

    assert!(!output.animating);
    assert!(!output.metrics.animation_changed_style);
}

#[test]
fn document_engine_update_scene_eases_transitioned_layout_styles() {
    let mut scene = DocumentScene::new(Size::new(320.0, 200.0));
    scene
        .append_element(
            "root",
            "card",
            ElementSpec::new(ElementRole::Card).interactive(),
        )
        .unwrap();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::Role(ElementRole::Card),
            Style::default()
                .size(100.0, 40.0)
                .transition(Transition::linear(0.25)),
        )
        .rule(
            StyleSelector::State(ElementStateSelector::Hovered),
            Style::default().size(140.0, 80.0),
        );
    let mut engine = DocumentEngine::default();
    engine.update_scene(&mut scene, &stylesheet);

    let output = engine.update_scene_with_input(
        &mut scene,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(4.0, 4.0),
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
    let card = output.layout.find("card").unwrap();

    assert_eq!(card.rect.size, Size::new(110.0, 50.0));
    assert!(output.animating);
    assert!(output.metrics.animation_changed_layout);
}

#[test]
fn document_engine_update_scene_eases_full_box_model_layout_styles() {
    let mut scene = DocumentScene::new(Size::new(320.0, 200.0));
    scene
        .append_element(
            "root",
            "card",
            ElementSpec::new(ElementRole::Card).interactive(),
        )
        .unwrap();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::Role(ElementRole::Card),
            Style::default()
                .size(100.0, 40.0)
                .min_size(20.0, 20.0)
                .max_size(180.0, 120.0)
                .padding(Insets::all(4.0))
                .margin(Insets::all(2.0))
                .gap(4.0)
                .border_width(2.0)
                .radius(4.0)
                .font_size(12.0)
                .transition(Transition::linear(0.25)),
        )
        .rule(
            StyleSelector::State(ElementStateSelector::Hovered),
            Style::default()
                .size(140.0, 80.0)
                .min_size(40.0, 60.0)
                .max_size(220.0, 160.0)
                .padding(Insets::all(12.0))
                .margin(Insets::all(10.0))
                .gap(20.0)
                .border_width(10.0)
                .radius(20.0)
                .font_size(20.0),
        );
    let mut engine = DocumentEngine::default();
    engine.update_scene(&mut scene, &stylesheet);

    let output =
        engine.update_scene_with_input(&mut scene, &stylesheet, hover_input(Point::new(4.0, 4.0)));
    let card = output.layout.find("card").unwrap();

    assert_eq!(card.rect.size, Size::new(110.0, 50.0));
    assert_eq!(card.style.min_size, Size::new(25.0, 30.0));
    assert_eq!(card.style.max_size, Size::new(190.0, 130.0));
    assert_eq!(card.style.padding, Insets::all(6.0));
    assert_eq!(card.style.margin, Insets::all(4.0));
    assert_eq!(card.style.gap, 8.0);
    assert_eq!(card.style.border_width, Insets::all(4.0));
    assert_eq!(card.style.radius, des_ui_document::CornerRadii::all(8.0));
    assert_eq!(card.style.font_size, 14.0);
    assert!(!output.metrics.reused_input_layout);
    assert!(output.metrics.animation_changed_style);
    assert!(output.metrics.animation_changed_layout);
    assert!(output.metrics.animation_changed_paint);

    let output =
        engine.update_scene_with_input(&mut scene, &stylesheet, hover_input(Point::new(4.0, 4.0)));

    assert!(!output.metrics.reused_input_layout);
    assert!(output.metrics.animation_changed_layout);
}

#[test]
fn document_engine_update_scene_untransitioned_hover_color_reuses_layout_and_updates_paint() {
    let mut scene = DocumentScene::new(Size::new(320.0, 200.0));
    scene
        .append_element(
            "root",
            "card",
            ElementSpec::new(ElementRole::Card).interactive(),
        )
        .unwrap();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::Role(ElementRole::Card),
            Style::default()
                .size(100.0, 40.0)
                .background(Color::rgb(20, 20, 20)),
        )
        .rule(
            StyleSelector::State(ElementStateSelector::Hovered),
            Style::default().background(Color::rgb(40, 70, 95)),
        );
    let mut engine = DocumentEngine::default();

    engine.update_scene(&mut scene, &stylesheet);
    let output =
        engine.update_scene_with_input(&mut scene, &stylesheet, hover_input(Point::new(2.0, 2.0)));
    let card = output.layout.find("card").unwrap();

    assert_eq!(card.style.background, Some(Color::rgb(40, 70, 95)));
    assert!(output.metrics.reused_input_layout);
    assert!(output.metrics.input_changed_state);
    assert!(output.metrics.animation_changed_style);
    assert!(!output.metrics.animation_changed_layout);
    assert!(output.metrics.animation_changed_paint);
}

#[test]
fn document_engine_update_scene_untransitioned_hover_layout_change_rebuilds_layout() {
    let mut scene = DocumentScene::new(Size::new(320.0, 200.0));
    scene
        .append_element(
            "root",
            "card",
            ElementSpec::new(ElementRole::Card).interactive(),
        )
        .unwrap();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::Role(ElementRole::Card),
            Style::default().size(100.0, 40.0),
        )
        .rule(
            StyleSelector::State(ElementStateSelector::Hovered),
            Style::default().size(140.0, 40.0),
        );
    let mut engine = DocumentEngine::default();

    engine.update_scene(&mut scene, &stylesheet);
    let output =
        engine.update_scene_with_input(&mut scene, &stylesheet, hover_input(Point::new(2.0, 2.0)));
    let card = output.layout.find("card").unwrap();

    assert_eq!(card.rect.size, Size::new(140.0, 40.0));
    assert!(!output.metrics.reused_input_layout);
    assert!(output.metrics.animation_changed_style);
    assert!(output.metrics.animation_changed_layout);
}

#[test]
fn document_engine_update_scene_snap_element_animation_clears_rendered_style() {
    let mut scene = DocumentScene::new(Size::new(320.0, 200.0));
    scene
        .append_element(
            "root",
            "card",
            ElementSpec::new(ElementRole::Card).interactive(),
        )
        .unwrap();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::Role(ElementRole::Card),
            Style::default()
                .size(100.0, 40.0)
                .background(Color::rgb(20, 20, 20))
                .transition(Transition::ease_out(0.24)),
        )
        .rule(
            StyleSelector::State(ElementStateSelector::Hovered),
            Style::default().background(Color::rgb(40, 70, 95)),
        );
    let mut engine = DocumentEngine::default();

    engine.update_scene(&mut scene, &stylesheet);
    engine.update_scene_with_input(&mut scene, &stylesheet, hover_input(Point::new(2.0, 2.0)));

    assert!(engine.snap_element_animation("card"));

    let output =
        engine.update_scene_with_input(&mut scene, &stylesheet, hover_input(Point::new(2.0, 2.0)));
    let card = output.layout.find("card").unwrap();

    assert_eq!(card.style.background, Some(Color::rgb(40, 70, 95)));
    assert!(!output.animating);
}

#[test]
fn document_engine_update_scene_scrollbar_hover_transition_reuses_layout() {
    let mut scene = DocumentScene::new(Size::new(180.0, 140.0));
    scene
        .append_element("root", "scroll-panel", ElementSpec::new(ElementRole::Panel))
        .unwrap();
    scene
        .append_element(
            "scroll-panel",
            "content",
            ElementSpec::new(ElementRole::Card),
        )
        .unwrap();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("scroll-panel"),
            Style::default()
                .size(70.0, 70.0)
                .overflow(Overflow::Scroll)
                .scrollbar_width(2.0)
                .scrollbar_expanded_width(10.0)
                .transition(Transition::ease_out(0.25)),
        )
        .rule(
            StyleSelector::id("content"),
            Style::default().size(70.0, 140.0),
        );
    let mut engine = DocumentEngine::default();

    engine.update_scene(&mut scene, &stylesheet);
    let output = engine.update_scene_with_input(
        &mut scene,
        &stylesheet,
        hover_input(Point::new(64.0, 20.0)),
    );
    let vertical = output
        .scroll_chrome
        .iter()
        .find(|chrome| {
            chrome.element_id == ElementId::new("scroll-panel")
                && chrome.axis == ScrollAxis::Vertical
        })
        .unwrap();

    assert!(vertical.expanded);
    assert!(vertical.handle_rect.size.width > 2.0);
    assert!(vertical.handle_rect.size.width < 10.0);
    assert!(output.animating);
    assert!(output.metrics.reused_input_layout);
    assert!(!output.metrics.animation_changed_layout);

    let output = engine.update_scene_with_input(
        &mut scene,
        &stylesheet,
        hover_input(Point::new(64.0, 20.0)),
    );

    assert!(output.metrics.reused_input_layout);
    assert!(!output.metrics.animation_changed_layout);
}

#[test]
fn document_engine_update_scene_uses_scene_text_layout() {
    let mut scene = DocumentScene::new(Size::new(800.0, 600.0));
    scene
        .append_text(
            "root",
            "label",
            ElementSpec::new(ElementRole::Text),
            "Retained text",
        )
        .unwrap();
    let stylesheet = StyleSheet::new().rule(
        StyleSelector::id("label"),
        Style::default().size(120.0, 40.0),
    );
    let mut engine = DocumentEngine::default();
    let mut text_measurer = RecordingTextMeasurer::default();

    let output = engine.update_scene_with_input_and_text_measurer(
        &mut scene,
        &stylesheet,
        DocumentInput::default(),
        &mut text_measurer,
    );

    assert_eq!(
        output
            .layout
            .find("label")
            .unwrap()
            .text_layout
            .unwrap()
            .size,
        Size::new(64.0, 18.0)
    );
}

#[test]
fn document_engine_update_scene_uses_text_measurement_for_auto_text_size() {
    let mut scene = DocumentScene::new(Size::new(800.0, 600.0));
    scene
        .append_text(
            "root",
            "label",
            ElementSpec::new(ElementRole::Text),
            "Measured",
        )
        .unwrap();
    let stylesheet = StyleSheet::new();
    let mut engine = DocumentEngine::default();
    let mut text_measurer = RecordingTextMeasurer::default();

    let output = engine.update_scene_with_input_and_text_measurer(
        &mut scene,
        &stylesheet,
        DocumentInput::default(),
        &mut text_measurer,
    );

    assert_eq!(
        output.layout.find("label").unwrap().rect,
        Rect::new(0.0, 0.0, 64.0, 18.0)
    );
}

#[test]
fn document_engine_update_scene_resolves_table_column_tracks() {
    let table = TableSpec::new(vec![
        TableColumnSpec::new("customer", "Customer").width(TableTrackSize::px(120.0)),
        TableColumnSpec::new("country", "Country").width(TableTrackSize::px(100.0)),
        TableColumnSpec::new("orders", "Orders").width(TableTrackSize::px(80.0)),
    ])
    .header_height(28.0)
    .row_height(26.0);
    let mut scene = DocumentScene::new(Size::new(320.0, 220.0));
    scene
        .append_element(
            "root",
            "customers",
            ElementSpec::new(ElementRole::Table).table(table),
        )
        .unwrap();
    scene
        .append_element(
            "customers",
            "customers-header",
            ElementSpec::new(ElementRole::TableHeader),
        )
        .unwrap();
    scene
        .append_text(
            "customers-header",
            "customers-header-customer",
            ElementSpec::new(ElementRole::TableCell).table_cell(TableCellSpec::new("customer")),
            "Customer",
        )
        .unwrap();
    scene
        .append_text(
            "customers-header",
            "customers-header-country",
            ElementSpec::new(ElementRole::TableCell).table_cell(TableCellSpec::new("country")),
            "Country",
        )
        .unwrap();
    scene
        .append_text(
            "customers-header",
            "customers-header-orders",
            ElementSpec::new(ElementRole::TableCell).table_cell(TableCellSpec::new("orders")),
            "Orders",
        )
        .unwrap();
    scene
        .append_element(
            "customers",
            "customers-row-0",
            ElementSpec::new(ElementRole::TableRow),
        )
        .unwrap();
    scene
        .append_text(
            "customers-row-0",
            "customers-row-0-customer",
            ElementSpec::new(ElementRole::TableCell).table_cell(TableCellSpec::new("customer")),
            "Acme",
        )
        .unwrap();
    scene
        .append_text(
            "customers-row-0",
            "customers-row-0-country",
            ElementSpec::new(ElementRole::TableCell).table_cell(TableCellSpec::new("country")),
            "US",
        )
        .unwrap();
    scene
        .append_text(
            "customers-row-0",
            "customers-row-0-orders",
            ElementSpec::new(ElementRole::TableCell).table_cell(TableCellSpec::new("orders")),
            "42",
        )
        .unwrap();

    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("customers"),
            Style::default()
                .width(Length::Px(240.0))
                .overflow_x(Overflow::Scroll),
        )
        .rule(
            StyleSelector::Role(ElementRole::TableCell),
            Style::default().border_width(1.0),
        );
    let mut engine = DocumentEngine::default();

    let output = engine.update_scene(&mut scene, &stylesheet);
    let header_customer = output.layout.find("customers-header-customer").unwrap();
    let row_customer = output.layout.find("customers-row-0-customer").unwrap();
    let header_orders = output.layout.find("customers-header-orders").unwrap();
    let row_orders = output.layout.find("customers-row-0-orders").unwrap();

    assert_eq!(header_customer.role, ElementRole::TableCell);
    assert_eq!(
        header_customer.rect.size.width,
        row_customer.rect.size.width
    );
    assert_eq!(header_orders.rect.origin.x, row_orders.rect.origin.x);
    assert_eq!(header_orders.rect.size.width, 80.0);
    assert!(row_customer.rect.origin.y > header_customer.rect.origin.y);
    assert!(output.scroll_chrome.iter().any(|chrome| {
        chrome.element_id == ElementId::new("customers")
            && chrome.axis == ScrollAxis::Horizontal
            && chrome.max_scroll == 60.0
    }));
}

#[test]
fn document_engine_update_scene_anchors_absolute_viewport_elements_to_window() {
    let mut scene = DocumentScene::new(Size::new(320.0, 200.0));
    scene
        .append_element("root", "panel", ElementSpec::new(ElementRole::Panel))
        .unwrap();
    scene
        .append_element(
            "panel",
            "absolute-child",
            ElementSpec::new(ElementRole::Card),
        )
        .unwrap();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("panel"),
            Style::default()
                .size(120.0, 80.0)
                .padding(Insets::all(10.0)),
        )
        .rule(
            StyleSelector::id("absolute-child"),
            Style::default()
                .absolute_viewport()
                .right(Length::Px(8.0))
                .bottom(Length::Px(9.0))
                .size(40.0, 20.0),
        );
    let mut engine = DocumentEngine::default();

    let output = engine.update_scene(&mut scene, &stylesheet);
    let absolute_child = output.layout.find("absolute-child").unwrap();

    assert_eq!(absolute_child.rect.origin, Point::new(272.0, 171.0));
}

#[test]
fn document_engine_update_scene_hits_absolute_viewport_child_outside_parent() {
    let mut scene = DocumentScene::new(Size::new(320.0, 200.0));
    scene
        .append_element("root", "panel", ElementSpec::new(ElementRole::Panel))
        .unwrap();
    scene
        .append_element(
            "panel",
            "absolute-child",
            ElementSpec::new(ElementRole::Card).interactive(),
        )
        .unwrap();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("panel"),
            Style::default().size(60.0, 40.0),
        )
        .rule(
            StyleSelector::id("absolute-child"),
            Style::default()
                .absolute_viewport()
                .left(Length::Px(140.0))
                .top(Length::Px(80.0))
                .size(40.0, 20.0),
        );
    let mut engine = DocumentEngine::default();

    let output = engine.update_scene_with_input(
        &mut scene,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(150.0, 90.0),
                primary_delta: Point::ZERO,
                primary_down: true,
                primary_pressed: false,
                primary_clicked: true,
                primary_click_count: 1,
                secondary_clicked: false,
                time_seconds: 0.0,
            }),
            scroll_delta: Point::ZERO,
        },
    );

    assert_eq!(output.hit_id, Some(ElementId::new("absolute-child")));
    assert!(engine.element_state("absolute-child").unwrap().pressed);
}

#[test]
fn document_engine_update_scene_positions_absolute_parent_without_flow_measurement() {
    let mut scene = DocumentScene::new(Size::new(320.0, 200.0));
    scene
        .append_element("root", "panel", ElementSpec::new(ElementRole::Panel))
        .unwrap();
    scene
        .append_element(
            "panel",
            "absolute-child",
            ElementSpec::new(ElementRole::Card),
        )
        .unwrap();
    scene
        .append_element("panel", "flow-child", ElementSpec::new(ElementRole::Card))
        .unwrap();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("panel"),
            Style::default()
                .width(Length::Auto)
                .height(Length::Auto)
                .padding(Insets::all(10.0))
                .border_width(2.0),
        )
        .rule(
            StyleSelector::id("flow-child"),
            Style::default().size(50.0, 20.0),
        )
        .rule(
            StyleSelector::id("absolute-child"),
            Style::default()
                .absolute_parent()
                .left(Length::Px(7.0))
                .top(Length::Px(5.0))
                .size(40.0, 20.0),
        );
    let mut engine = DocumentEngine::default();

    let output = engine.update_scene(&mut scene, &stylesheet);
    let panel = output.layout.find("panel").unwrap();
    let absolute_child = output.layout.find("absolute-child").unwrap();
    let flow_child = output.layout.find("flow-child").unwrap();

    assert_eq!(panel.rect.size, Size::new(74.0, 44.0));
    assert_eq!(flow_child.rect.origin, Point::new(12.0, 12.0));
    assert_eq!(absolute_child.rect.origin, Point::new(19.0, 17.0));
}

#[test]
fn document_engine_update_scene_positions_absolute_anchor_after_target_layout() {
    let mut scene = DocumentScene::new(Size::new(320.0, 200.0));
    scene
        .append_element("root", "panel", ElementSpec::new(ElementRole::Panel))
        .unwrap();
    scene
        .append_element("panel", "popover", ElementSpec::new(ElementRole::Card))
        .unwrap();
    scene
        .append_element("panel", "anchor", ElementSpec::new(ElementRole::Card))
        .unwrap();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("panel"),
            Style::default()
                .width(Length::Auto)
                .height(Length::Auto)
                .padding(Insets::all(10.0))
                .border_width(2.0),
        )
        .rule(
            StyleSelector::id("anchor"),
            Style::default().size(80.0, 30.0),
        )
        .rule(
            StyleSelector::id("popover"),
            Style::default()
                .absolute_parent()
                .anchor_bottom_start("anchor", 0.0, -1.0)
                .size(60.0, 20.0),
        );
    let mut engine = DocumentEngine::default();

    let output = engine.update_scene(&mut scene, &stylesheet);
    let panel = output.layout.find("panel").unwrap();
    let anchor = output.layout.find("anchor").unwrap();
    let popover = output.layout.find("popover").unwrap();

    assert_eq!(panel.rect.size, Size::new(104.0, 54.0));
    assert_eq!(anchor.rect.origin, Point::new(12.0, 12.0));
    assert_eq!(popover.rect.origin, Point::new(12.0, 41.0));
    assert_eq!(popover.rect.size, Size::new(60.0, 20.0));
}

#[test]
fn document_engine_update_scene_wraps_row_children_and_expands_height() {
    let mut scene = DocumentScene::new(Size::new(240.0, 160.0));
    scene
        .append_element("root", "row", ElementSpec::new(ElementRole::Panel))
        .unwrap();
    for index in 0..3 {
        scene
            .append_element(
                "row",
                format!("item-{index}"),
                ElementSpec::new(ElementRole::Card).class("item"),
            )
            .unwrap();
    }
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("row"),
            Style::default()
                .flex_direction(FlexDirection::Row)
                .flex_wrap(FlexWrap::Wrap)
                .width(Length::Px(120.0))
                .height(Length::Auto)
                .gap(10.0),
        )
        .rule(
            StyleSelector::class("item"),
            Style::default().size(50.0, 20.0),
        );
    let mut engine = DocumentEngine::default();

    let output = engine.update_scene(&mut scene, &stylesheet);
    let row = output.layout.find("row").unwrap();
    let item_0 = output.layout.find("item-0").unwrap();
    let item_1 = output.layout.find("item-1").unwrap();
    let item_2 = output.layout.find("item-2").unwrap();

    assert_eq!(row.rect.size, Size::new(120.0, 50.0));
    assert_eq!(item_0.rect.origin, Point::new(0.0, 0.0));
    assert_eq!(item_1.rect.origin, Point::new(60.0, 0.0));
    assert_eq!(item_2.rect.origin, Point::new(0.0, 30.0));
}

#[test]
fn document_engine_update_scene_fill_width_uses_parent_content_width_after_box_model() {
    let mut scene = DocumentScene::new(Size::new(320.0, 200.0));
    scene
        .append_element("root", "panel", ElementSpec::new(ElementRole::Panel))
        .unwrap();
    scene
        .append_element("panel", "row", ElementSpec::new(ElementRole::Card))
        .unwrap();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("panel"),
            Style::default()
                .size(200.0, 120.0)
                .border_width(2.0)
                .padding(Insets::symmetric(12.0, 8.0)),
        )
        .rule(
            StyleSelector::id("row"),
            Style::default()
                .width_fill()
                .height(Length::Px(24.0))
                .margin(Insets::symmetric(3.0, 0.0)),
        );
    let mut engine = DocumentEngine::default();

    let output = engine.update_scene(&mut scene, &stylesheet);
    let row = output.layout.find("row").unwrap();

    assert_eq!(row.rect.origin, Point::new(17.0, 10.0));
    assert_eq!(row.rect.size, Size::new(172.0, 24.0));
}

#[test]
fn document_engine_update_scene_fill_size_does_not_inflate_auto_parent() {
    let mut scene = DocumentScene::new(Size::new(240.0, 160.0));
    scene
        .append_element("root", "panel", ElementSpec::new(ElementRole::Panel))
        .unwrap();
    scene
        .append_element("panel", "child", ElementSpec::new(ElementRole::Card))
        .unwrap();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("panel"),
            Style::default().width(Length::Auto).height(Length::Auto),
        )
        .rule(
            StyleSelector::id("child"),
            Style::default()
                .width_fill()
                .height_fill()
                .min_size(24.0, 24.0),
        );
    let mut engine = DocumentEngine::default();

    let output = engine.update_scene(&mut scene, &stylesheet);
    let panel = output.layout.find("panel").unwrap();

    assert_eq!(panel.rect.size, Size::new(24.0, 24.0));
}

#[test]
fn document_engine_update_scene_max_size_clamps_auto_explicit_and_fill_sizes() {
    let mut scene = DocumentScene::new(Size::new(260.0, 180.0));
    scene
        .append_element("root", "panel", ElementSpec::new(ElementRole::Panel))
        .unwrap();
    scene
        .append_element("panel", "auto-child", ElementSpec::new(ElementRole::Panel))
        .unwrap();
    scene
        .append_element(
            "auto-child",
            "wide-child",
            ElementSpec::new(ElementRole::Card).class("wide"),
        )
        .unwrap();
    scene
        .append_element("panel", "fixed-child", ElementSpec::new(ElementRole::Card))
        .unwrap();
    scene
        .append_element("panel", "fill-child", ElementSpec::new(ElementRole::Card))
        .unwrap();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("panel"),
            Style::default()
                .size(200.0, 120.0)
                .padding(Insets::all(10.0)),
        )
        .rule(
            StyleSelector::id("auto-child"),
            Style::default()
                .width(Length::Auto)
                .height(Length::Auto)
                .max_size(40.0, 30.0),
        )
        .rule(
            StyleSelector::id("fixed-child"),
            Style::default().size(96.0, 70.0).max_size(42.0, 28.0),
        )
        .rule(
            StyleSelector::id("fill-child"),
            Style::default()
                .width_fill()
                .height_fill()
                .max_size(50.0, 34.0),
        )
        .rule(
            StyleSelector::class("wide"),
            Style::default().size(80.0, 20.0),
        );
    let mut engine = DocumentEngine::default();

    let output = engine.update_scene(&mut scene, &stylesheet);

    assert_eq!(
        output.layout.find("auto-child").unwrap().rect.size,
        Size::new(40.0, 20.0)
    );
    assert_eq!(
        output.layout.find("fixed-child").unwrap().rect.size,
        Size::new(42.0, 28.0)
    );
    assert_eq!(
        output.layout.find("fill-child").unwrap().rect.size,
        Size::new(50.0, 34.0)
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
fn document_engine_update_scene_scroll_only_final_pass_skips_style_resolution() {
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
    let hover_input = DocumentInput {
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
        scroll_delta: Point::ZERO,
    };
    engine.update_scene_with_input(&mut scene, &stylesheet, hover_input);

    let output = engine.update_scene_with_input(
        &mut scene,
        &stylesheet,
        DocumentInput {
            scroll_delta: Point::new(0.0, -40.0),
            ..hover_input
        },
    );

    assert_eq!(output.metrics.scene_style_nodes_visited, 0);
    assert!(!output.metrics.reused_input_layout);
    assert_eq!(
        output.layout.find("content").unwrap().rect.origin,
        Point::new(0.0, -40.0)
    );
}

#[test]
fn document_engine_update_scene_with_input_offsets_scrolled_child_rects() {
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

    assert_eq!(
        output.layout.find("content").unwrap().rect,
        Rect::new(0.0, -40.0, 100.0, 300.0)
    );
}

#[test]
fn document_engine_update_scene_with_input_hit_tests_scrolled_child_position() {
    let mut scene = DocumentScene::new(Size::new(800.0, 600.0));
    scene
        .append_element("root", "scroll", ElementSpec::new(ElementRole::Panel))
        .unwrap();
    scene
        .append_element("scroll", "content", ElementSpec::new(ElementRole::Panel))
        .unwrap();
    scene
        .append_element("content", "spacer", ElementSpec::new(ElementRole::Panel))
        .unwrap();
    scene
        .append_element(
            "content",
            "target",
            ElementSpec::new(ElementRole::Control).interactive(),
        )
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
        )
        .rule(
            StyleSelector::id("spacer"),
            Style::default().size(100.0, 100.0),
        )
        .rule(
            StyleSelector::id("target"),
            Style::default().size(100.0, 30.0),
        );
    let mut engine = DocumentEngine::default();
    engine.update_scene_with_input(
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

    let output = engine.update_scene_with_input(
        &mut scene,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(50.0, 70.0),
                primary_delta: Point::ZERO,
                primary_down: true,
                primary_pressed: true,
                primary_clicked: true,
                primary_click_count: 1,
                secondary_clicked: false,
                time_seconds: 0.1,
            }),
            scroll_delta: Point::ZERO,
        },
    );

    assert_eq!(output.hit_id, Some(ElementId::new("target")));
    assert!(engine.element_state("target").unwrap().pressed);
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
