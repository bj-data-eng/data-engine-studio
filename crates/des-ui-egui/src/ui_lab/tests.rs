use super::*;
use crate::graphics_testing::{
    TEST_HEIGHT, TEST_WIDTH, assert_exact_image_match, compare_images, image_stats, render_harness,
    test_harness,
};
use des_ui_document::{
    Document, DocumentEngine, DocumentInput, DocumentOutput, ElementRole, ElementSpec, Insets,
    Length, Point, PointerInput, ResolvedElement, ScrollAxis, Size, Style, StyleSelector,
    StyleSheet, TextWrapMode,
};
use egui_kittest::Harness;

const INTERACTION_LOOP_SCROLL_Y: f32 = 300.0;

fn lab_harness(initial_view: &str) -> Harness<'_, UiLabState> {
    test_harness(UiLabState::new(Some(initial_view)), |ui, state| {
        state.render(ui, false);
    })
}

fn lab_image(initial_view: &str) -> image::RgbaImage {
    render_harness(&mut lab_harness(initial_view))
}

fn lab_rect(id: &str) -> des_ui_document::Rect {
    lab_rect_in("layout", id)
}

fn lab_rect_in(initial_view: &str, id: &str) -> des_ui_document::Rect {
    let state = UiLabState::new(Some(initial_view));
    state_rect(&state, id)
}

fn state_rect(state: &UiLabState, id: &str) -> des_ui_document::Rect {
    let mut engine = DocumentEngine::default();
    let document = state.document(Size::new(TEST_WIDTH, TEST_HEIGHT), false);
    let output = engine.update(&document, &stylesheet());
    find_frame(&output.layout, id)
        .unwrap_or_else(|| panic!("expected layout frame for {id}"))
        .rect
}

fn lab_output(initial_view: &str) -> DocumentOutput {
    lab_output_with_size(initial_view, Size::new(TEST_WIDTH, TEST_HEIGHT))
}

fn lab_output_with_size(initial_view: &str, size: Size) -> DocumentOutput {
    let mut engine = DocumentEngine::default();
    let document = UiLabState::new(Some(initial_view)).document(size, false);
    engine.update(&document, &stylesheet())
}

fn lab_output_with_stage_scroll(initial_view: &str, scroll_y: f32) -> DocumentOutput {
    let mut engine = DocumentEngine::default();
    let document =
        UiLabState::new(Some(initial_view)).document(Size::new(TEST_WIDTH, TEST_HEIGHT), false);
    let stylesheet = stylesheet();
    engine.update(&document, &stylesheet);
    engine.element_state_mut("stage").unwrap().scroll_y = scroll_y;
    engine.update(&document, &stylesheet)
}

fn find_frame<'a>(frame: &'a ResolvedElement, id: &str) -> Option<&'a ResolvedElement> {
    if frame.id.as_str() == id {
        return Some(frame);
    }
    frame
        .children
        .iter()
        .find_map(|child| find_frame(child, id))
}

fn frame<'a>(output: &'a DocumentOutput, id: &str) -> &'a ResolvedElement {
    find_frame(&output.layout, id).unwrap_or_else(|| panic!("expected layout frame for {id}"))
}

fn state_output(state: &UiLabState) -> DocumentOutput {
    let mut engine = DocumentEngine::default();
    let document = state.document(Size::new(TEST_WIDTH, TEST_HEIGHT), false);
    engine.update(&document, &state.active_stylesheet())
}

fn state_output_with_egui_text(state: &UiLabState, ctx: &egui::Context) -> DocumentOutput {
    let mut engine = DocumentEngine::default();
    let mut text_measurer = super::egui_adapter::EguiTextMeasurer::new(ctx);
    let document = state.document(Size::new(TEST_WIDTH, TEST_HEIGHT), false);
    engine.update_with_input_and_text_measurer(
        &document,
        &state.active_stylesheet(),
        DocumentInput::default(),
        &mut text_measurer,
    )
}

fn state_rect_with_egui_text(
    state: &UiLabState,
    ctx: &egui::Context,
    id: &str,
) -> des_ui_document::Rect {
    let output = state_output_with_egui_text(state, ctx);
    find_frame(&output.layout, id)
        .unwrap_or_else(|| panic!("expected layout frame for {id}"))
        .rect
}

fn state_output_with_scroll(state: &UiLabState, scroll_y: f32) -> DocumentOutput {
    let mut engine = DocumentEngine::default();
    let stylesheet = state.active_stylesheet();
    let document = state.document(Size::new(TEST_WIDTH, TEST_HEIGHT), false);
    engine.update(&document, &stylesheet);
    engine.element_state_mut("stage").unwrap().scroll_y = scroll_y;
    engine.update(&document, &stylesheet)
}

fn has_class(frame: &ResolvedElement, class: &str) -> bool {
    frame
        .classes
        .iter()
        .any(|element_class| element_class.as_str() == class)
}

fn assert_close(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() < 0.01,
        "expected {actual} to be close to {expected}"
    );
}

fn center(rect: des_ui_document::Rect) -> egui::Pos2 {
    egui::pos2(
        rect.origin.x + rect.size.width / 2.0,
        rect.origin.y + rect.size.height / 2.0,
    )
}

fn scroll_harness_stage(harness: &mut Harness<'_, UiLabState>, scroll_y: f32) {
    harness.run();
    harness
        .state_mut()
        .document_engine
        .element_state_mut("stage")
        .unwrap()
        .scroll_y = scroll_y;
    harness.run();
}

fn assert_scroll_chrome(output: &DocumentOutput, id: &str, axis: ScrollAxis) {
    assert!(
        output.scroll_chrome.iter().any(|chrome| {
            chrome.element_id.as_str() == id && chrome.axis == axis && chrome.max_scroll > 0.0
        }),
        "expected {axis:?} scroll chrome for {id}"
    );
}

#[test]
fn kittest_renders_lab_frame_to_shapes() {
    let mut harness = lab_harness("layout");

    harness.run();

    assert!(
        harness.output().shapes.len() > 20,
        "expected the UI lab to produce a non-trivial painted document"
    );
}

#[test]
fn kittest_renders_lab_frame_to_pixels() {
    let image = lab_image("layout");
    let stats = image_stats(&image);

    assert_eq!(stats.width, TEST_WIDTH as u32);
    assert_eq!(stats.height, TEST_HEIGHT as u32);
    assert!(
        stats.non_transparent_pixels > stats.total_pixels / 4,
        "expected the rendered UI lab image to contain visible pixels"
    );
}

#[test]
fn kittest_pointer_click_reaches_document_owned_nav_item() {
    let mut harness = lab_harness("layout");
    let rect = lab_rect("view-interaction");
    let interaction_nav_item = egui::pos2(
        rect.origin.x + rect.size.width / 2.0,
        rect.origin.y + rect.size.height / 2.0,
    );

    harness.hover_at(interaction_nav_item);
    harness.drag_at(interaction_nav_item);
    harness.drop_at(interaction_nav_item);
    harness.run();

    assert_eq!(harness.state().view, LabView::Interaction);
}

#[test]
fn graphical_comparison_matches_identical_lab_views() {
    let first = lab_image("layout");
    let second = lab_image("layout");

    assert_exact_image_match(&first, &second);
}

#[test]
fn graphical_comparison_detects_different_lab_views() {
    let layout = lab_image("layout");
    let scrolling = lab_image("scrolling");
    let comparison = compare_images(&layout, &scrolling);

    assert!(
        comparison.differing_pixels > comparison.compared_pixels / 20,
        "expected visibly different lab views, got {comparison:?}"
    );
}

#[test]
fn clicked_nav_view_matches_directly_seeded_view() {
    let mut clicked = lab_harness("layout");
    let rect = lab_rect("view-interaction");
    let interaction_nav_item = egui::pos2(
        rect.origin.x + rect.size.width / 2.0,
        rect.origin.y + rect.size.height / 2.0,
    );

    clicked.hover_at(interaction_nav_item);
    clicked.drag_at(interaction_nav_item);
    clicked.drop_at(interaction_nav_item);
    clicked.run();

    let clicked_image = render_harness(&mut clicked);
    let direct_image = lab_image("interaction");

    assert_exact_image_match(&clicked_image, &direct_image);
}

#[test]
fn clicked_table_nav_view_matches_directly_seeded_view() {
    let mut clicked = lab_harness("layout");
    let table_nav_item = center(lab_rect("view-table"));

    clicked.hover_at(table_nav_item);
    clicked.drag_at(table_nav_item);
    clicked.drop_at(table_nav_item);
    clicked.run();

    let clicked_image = render_harness(&mut clicked);
    let direct_image = lab_image("table");

    assert_exact_image_match(&clicked_image, &direct_image);
}

#[test]
fn clicked_text_nav_view_matches_directly_seeded_view() {
    let mut clicked = lab_harness("layout");
    let text_nav_item = center(lab_rect("view-text"));

    clicked.hover_at(text_nav_item);
    clicked.drag_at(text_nav_item);
    clicked.drop_at(text_nav_item);
    clicked.run();

    let clicked_image = render_harness(&mut clicked);
    let direct_image = lab_image("text");

    assert_exact_image_match(&clicked_image, &direct_image);
}

#[test]
fn styling_view_renders_structural_selector_specimens() {
    let output = lab_output("styling");

    assert_eq!(
        frame(&output, "structural-main-one").style.background,
        Some(SUCCESS_CONTAINER)
    );
    assert_eq!(
        frame(&output, "structural-main-two").style.background,
        Some(PRIMARY_CONTAINER)
    );
    assert_close(
        frame(&output, "structural-main-three")
            .style
            .border_width
            .left,
        5.0,
    );
    assert_eq!(
        frame(&output, "structural-main-four").style.border,
        Some(PURPLE)
    );
    assert_eq!(
        frame(&output, "structural-nested-a-one").style.background,
        Some(SUCCESS_CONTAINER),
        "first-child should resolve within each nested parent"
    );
    assert_eq!(
        frame(&output, "structural-nested-b-one").style.background,
        Some(SUCCESS_CONTAINER),
        "first-child should reset for sibling lists"
    );
    assert_eq!(
        frame(&output, "structural-nested-a-two").style.border,
        Some(PURPLE),
        "last-child should resolve within each nested parent"
    );
    assert_eq!(frame(&output, "shadow-single").style.shadows.len(), 2);
    assert_eq!(frame(&output, "shadow-layered").style.shadows.len(), 2);
    assert_eq!(frame(&output, "shadow-light-top").style.shadows.len(), 2);
    assert_close(
        frame(&output, "shadow-negative-spread").style.shadows[1].spread,
        6.0,
    );
}

#[test]
fn interaction_view_renders_common_control_roles() {
    let output = lab_output("interaction");

    assert_eq!(
        frame(&output, "control-checkbox").role,
        ElementRole::Checkbox
    );
    assert_eq!(
        frame(&output, "control-radio-local").role,
        ElementRole::Radio
    );
    assert_eq!(
        frame(&output, "control-dropdown").role,
        ElementRole::Dropdown
    );
    assert_eq!(
        frame(&output, "control-input-name").role,
        ElementRole::TextInput
    );
    assert!(frame(&output, "control-checkbox").interactive);
    assert!(frame(&output, "control-input-name").interactive);
    assert!(!frame(&output, "control-input-disabled").interactive);
}

#[test]
fn common_control_clicks_update_lab_state() {
    let mut harness = lab_harness("interaction");

    for (id, assert_state) in [
        (
            "control-checkbox",
            Box::new(|state: &UiLabState| assert!(!state.checkbox_enabled))
                as Box<dyn Fn(&UiLabState)>,
        ),
        (
            "control-radio-remote",
            Box::new(|state: &UiLabState| assert_eq!(state.radio_choice, 1)),
        ),
        (
            "control-dropdown",
            Box::new(|state: &UiLabState| assert!(state.dropdown_open)),
        ),
        (
            "control-dropdown-option-python",
            Box::new(|state: &UiLabState| {
                assert_eq!(state.dropdown_choice, 2);
                assert!(!state.dropdown_open);
            }),
        ),
    ] {
        let rect = state_rect(harness.state(), id);
        let target = egui::pos2(
            rect.origin.x + rect.size.width / 2.0,
            rect.origin.y + rect.size.height / 2.0,
        );
        harness.hover_at(target);
        harness.drag_at(target);
        harness.drop_at(target);
        harness.run();
        assert_state(harness.state());
    }
}

#[test]
fn dropdown_click_away_closes_open_menu() {
    let mut harness = lab_harness("interaction");

    let dropdown = state_rect(harness.state(), "control-dropdown");
    let dropdown_target = egui::pos2(
        dropdown.origin.x + dropdown.size.width / 2.0,
        dropdown.origin.y + dropdown.size.height / 2.0,
    );
    harness.hover_at(dropdown_target);
    harness.drag_at(dropdown_target);
    harness.drop_at(dropdown_target);
    harness.run();
    assert!(harness.state().dropdown_open);

    let input = state_rect(harness.state(), "control-input-name");
    let input_target = egui::pos2(
        input.origin.x + input.size.width / 2.0,
        input.origin.y + input.size.height / 2.0,
    );
    harness.hover_at(input_target);
    harness.drag_at(input_target);
    harness.drop_at(input_target);
    harness.run();

    assert!(!harness.state().dropdown_open);
}

#[test]
fn interaction_update_loop_mutates_target_boxes_from_control_events() {
    let mut harness = lab_harness("interaction");
    let output = state_output(harness.state());

    assert_eq!(
        frame(&output, "loop-checkbox-result").text.as_deref(),
        Some("Profiling: enabled by checkbox")
    );
    assert!(
        frame(&output, "loop-checkbox-result-box")
            .style
            .background
            .is_some()
    );
    assert!(has_class(
        frame(&output, "loop-radio-result-box"),
        "loop-runtime-local"
    ));
    assert!(has_class(
        frame(&output, "loop-dropdown-result-box"),
        "loop-source-duckdb"
    ));

    scroll_harness_stage(&mut harness, INTERACTION_LOOP_SCROLL_Y);
    let loop_button = frame(
        &state_output_with_scroll(harness.state(), INTERACTION_LOOP_SCROLL_Y),
        "loop-action-button",
    )
    .rect;
    let target = center(loop_button);
    harness.hover_at(target);
    harness.drag_at(target);
    harness.drop_at(target);
    harness.run();

    scroll_harness_stage(&mut harness, 0.0);
    for id in [
        "control-checkbox",
        "control-radio-remote",
        "control-dropdown",
        "control-dropdown-option-python",
    ] {
        let target = center(state_rect(harness.state(), id));
        harness.hover_at(target);
        harness.drag_at(target);
        harness.drop_at(target);
        harness.run();
    }

    let output = state_output(harness.state());

    assert_eq!(harness.state().loop_action_count, 1);
    assert_eq!(
        frame(&output, "loop-button-result").text.as_deref(),
        Some("Button events received: 1")
    );
    assert_eq!(
        frame(&output, "loop-button-result-box").value.as_deref(),
        Some("button-count=1")
    );
    assert_eq!(
        frame(&output, "loop-checkbox-result").text.as_deref(),
        Some("Profiling: disabled by checkbox")
    );
    assert!(
        !frame(&output, "loop-checkbox-result-box")
            .style
            .background
            .is_some_and(|color| color == SUCCESS_CONTAINER)
    );
    assert_eq!(
        frame(&output, "loop-radio-result").text.as_deref(),
        Some("Runtime target: Remote worker")
    );
    assert!(has_class(
        frame(&output, "loop-radio-result-box"),
        "loop-runtime-remote"
    ));
    assert_eq!(
        frame(&output, "loop-dropdown-result").text.as_deref(),
        Some("Source adapter: Python node")
    );
    assert!(has_class(
        frame(&output, "loop-dropdown-result-box"),
        "loop-source-python"
    ));
    assert_eq!(
        frame(&output, "loop-summary-result").text.as_deref(),
        Some("profile off | remote | python | 1 click")
    );
    assert_eq!(
        frame(&output, "loop-summary-result-box").style.border,
        Some(PURPLE)
    );
}

#[test]
fn interaction_update_loop_refreshes_text_on_repeated_button_clicks() {
    let mut harness = lab_harness("interaction");
    scroll_harness_stage(&mut harness, INTERACTION_LOOP_SCROLL_Y);

    for expected in 1..=2 {
        let rect = frame(
            &state_output_with_scroll(harness.state(), INTERACTION_LOOP_SCROLL_Y),
            "loop-action-button",
        )
        .rect;
        let target = center(rect);
        harness.hover_at(target);
        harness.drag_at(target);
        harness.drop_at(target);
        harness.run();

        let output = state_output(harness.state());
        let expected_text = format!("Button events received: {expected}");
        let expected_value = format!("button-count={expected}");
        assert_eq!(
            frame(&output, "loop-button-result").text.as_deref(),
            Some(expected_text.as_str())
        );
        assert_eq!(
            frame(&output, "loop-button-result-box").value.as_deref(),
            Some(expected_value.as_str())
        );
    }
}

#[test]
fn interaction_drag_drop_grid_moves_items_between_cells() {
    let mut harness = lab_harness("interaction");

    assert_eq!(harness.state().drag_item_cells, [0, 2, 4]);

    let start = center(state_rect_with_egui_text(
        harness.state(),
        &harness.ctx,
        "drag-handle-0",
    ));
    let destination = center(state_rect_with_egui_text(
        harness.state(),
        &harness.ctx,
        "drag-cell-3",
    ));
    harness.hover_at(start);
    harness.drag_at(start);
    harness.run();
    assert_eq!(
        harness.state().active_drag_item(),
        None,
        "pointer down should not activate drag until movement passes threshold"
    );

    harness.hover_at(destination);
    harness.run();
    assert_eq!(harness.state().active_drag_item(), Some(SortableItemId(0)));
    assert!(harness.state().active_drag.is_some());
    let output = state_output_with_egui_text(harness.state(), &harness.ctx);
    let overlay = frame(&output, "drag-overlay");
    assert_eq!(overlay.text.as_deref(), None);
    assert!(has_class(overlay, "drag-overlay"));
    assert!(
        has_class(frame(&output, "drag-item-0"), "drag-origin-collapsed"),
        "source placeholder should collapse once a new drop position opens"
    );
    assert!(
        overlay
            .rect
            .contains(Point::new(destination.x, destination.y)),
        "drag overlay should follow the pointer before drop"
    );

    harness.event(egui::Event::PointerButton {
        pos: destination,
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: egui::Modifiers::NONE,
    });
    harness.run();

    assert_eq!(harness.state().active_drag_item(), None);
    assert!(harness.state().active_drag.is_none());
    assert_eq!(harness.state().drag_item_cells[0], 3);

    let output = state_output_with_egui_text(harness.state(), &harness.ctx);
    let item = frame(&output, "drag-item-0");
    let cell = frame(&output, "drag-cell-3");
    assert_eq!(item.value.as_deref(), Some("Customers"));
    assert!(
        cell.rect.contains(Point::new(
            item.rect.origin.x + item.rect.size.width / 2.0,
            item.rect.origin.y + item.rect.size.height / 2.0,
        )),
        "moved item should be laid out inside the destination cell"
    );

    let document = harness
        .state()
        .document(Size::new(TEST_WIDTH, TEST_HEIGHT), false);
    let stylesheet = harness.state().active_stylesheet();
    let output = harness
        .state_mut()
        .document_engine
        .update(&document, &stylesheet);
    let item = frame(&output, "drag-item-0");
    assert_eq!(
        item.style.height,
        Length::Px(34.0),
        "dropped item should snap to its full card height instead of easing out of the collapsed placeholder"
    );
    assert_eq!(
        item.style.padding,
        Insets::symmetric(9.0, 6.0),
        "dropped item should rematerialize with full padding on its first final-frame layout"
    );
}

#[test]
fn interaction_drag_drop_reorders_with_nearest_item_gap() {
    let mut harness = lab_harness("interaction");
    harness.state_mut().drag_item_cells = [0, 0, 2];
    harness.state_mut().drag_item_order = [0, 1, 2];
    harness.run();

    let start = center(state_rect(harness.state(), "drag-handle-0"));
    let activation_point = egui::pos2(start.x + 8.0, start.y);
    harness.hover_at(start);
    harness.drag_at(start);
    harness.hover_at(activation_point);
    harness.run();

    let output = state_output(harness.state());
    let second_item = frame(&output, "drag-item-1").rect;
    let destination = egui::pos2(
        second_item.origin.x + second_item.size.width / 2.0,
        second_item.origin.y + second_item.size.height - 4.0,
    );
    harness.hover_at(destination);
    harness.run_steps(4);

    let output = state_output(harness.state());
    assert!(harness.state().drag_drop_preview.is_some());
    assert!(
        has_class(frame(&output, "drag-item-0"), "drag-origin-collapsed"),
        "source placeholder should collapse while an insertion gap opens"
    );

    harness.event(egui::Event::PointerButton {
        pos: destination,
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: egui::Modifiers::NONE,
    });
    harness.run();

    assert_eq!(harness.state().drag_item_cells[0], 0);
    assert!(harness.state().drag_item_order[0] > harness.state().drag_item_order[1]);

    let document = harness
        .state()
        .document(Size::new(TEST_WIDTH, TEST_HEIGHT), false);
    let stylesheet = harness.state().active_stylesheet();
    let output = harness
        .state_mut()
        .document_engine
        .update(&document, &stylesheet);
    assert_eq!(
        frame(&output, "drag-item-1").style.margin,
        Insets::ZERO,
        "nearest item should snap out of the temporary insertion gap when the drop is committed"
    );
}

#[test]
fn interaction_drag_drop_suppresses_gap_at_original_position() {
    let mut harness = lab_harness("interaction");
    harness.state_mut().drag_item_cells = [0, 0, 2];
    harness.state_mut().drag_item_order = [0, 1, 2];
    harness.run();

    let start = center(state_rect(harness.state(), "drag-handle-1"));
    harness.hover_at(start);
    harness.drag_at(start);
    harness.hover_at(egui::pos2(start.x + 8.0, start.y));
    harness.run();
    let output = state_output(harness.state());
    let first_item = frame(&output, "drag-item-0").rect;
    let original_position = egui::pos2(
        first_item.origin.x + first_item.size.width / 2.0,
        first_item.origin.y + first_item.size.height + 2.0,
    );
    harness.hover_at(original_position);
    harness.run();

    let output = state_output(harness.state());
    assert!(harness.state().drag_source_placeholder_visible());
    assert!(
        has_class(frame(&output, "drag-item-1"), "drag-origin-space"),
        "source item should keep one hidden placeholder"
    );
    assert!(
        !has_class(frame(&output, "drag-item-0"), "drag-gap-after"),
        "no second insertion gap should appear at the original position"
    );
}

#[test]
fn interaction_drag_drop_requires_handle_to_drag_parent() {
    let mut harness = lab_harness("interaction");

    let card_center = center(state_rect(harness.state(), "drag-item-0"));
    let destination = center(state_rect(harness.state(), "drag-cell-3"));
    harness.hover_at(card_center);
    harness.drag_at(card_center);
    harness.run();

    assert_eq!(harness.state().active_drag_item(), None);

    harness.hover_at(destination);
    harness.event(egui::Event::PointerButton {
        pos: destination,
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: egui::Modifiers::NONE,
    });
    harness.run();

    assert_eq!(harness.state().drag_item_cells[0], 0);
}

#[test]
fn interaction_drag_drop_styles_are_animated() {
    let mut harness = lab_harness("interaction");
    let start = center(state_rect(harness.state(), "drag-handle-0"));
    let destination = center(state_rect(harness.state(), "drag-cell-3"));
    harness.hover_at(start);
    harness.drag_at(start);
    harness.hover_at(destination);
    harness.run();
    let output = state_output(harness.state());

    for id in ["drag-grid", "drag-cell-0", "drag-item-1", "drag-handle-1"] {
        assert!(
            frame(&output, id).style.transition.is_some(),
            "{id} should define a transition for drag/drop styling"
        );
    }
    assert!(
        frame(&output, "drag-overlay").style.transition.is_some(),
        "drag overlay should define a transition for drag/drop styling"
    );
    assert_eq!(
        frame(&output, "drag-overlay").style.shadows.len(),
        2,
        "drag overlay should use material elevation layers"
    );
    assert_eq!(
        frame(&output, "drag-scroll-list-card").style.shadows.len(),
        2,
        "scrollable drag list should use resting elevation"
    );
}

#[test]
fn interaction_drag_drop_auto_scrolls_opted_in_list_pane() {
    let mut harness = lab_harness("interaction");
    let start = center(state_rect_with_egui_text(
        harness.state(),
        &harness.ctx,
        "drag-scroll-handle-0",
    ));
    let list = state_rect_with_egui_text(harness.state(), &harness.ctx, "drag-scroll-list-0");
    let near_bottom = egui::pos2(
        list.origin.x + list.size.width / 2.0,
        list.origin.y + list.size.height - 4.0,
    );

    harness.hover_at(start);
    harness.drag_at(start);
    harness.hover_at(near_bottom);
    harness.run_steps(4);

    assert!(harness.state().active_drag.is_some());
    assert!(
        harness
            .state()
            .document_engine
            .element_state("drag-scroll-list-0")
            .unwrap()
            .scroll_y
            > 0.0,
        "dragging near the bottom of an opted-in list pane should auto-scroll it"
    );
}

#[test]
fn interaction_drag_drop_uses_snapshot_path_for_drop_targets() {
    let output = lab_output("interaction");
    let cell = frame(&output, "drag-cell-5").rect;
    let point = Point::new(
        cell.origin.x + cell.size.width / 2.0,
        cell.origin.y + cell.size.height / 2.0,
    );
    assert_eq!(drop_cell_at(&output, point), Some(5));
}

#[test]
fn interaction_drag_drop_cells_expand_to_fit_stacked_items() {
    let mut state = UiLabState::new(Some("interaction"));
    state.drag_item_cells = [0, 0, 0];
    let output = state_output(&state);
    let cell = frame(&output, "drag-cell-0");

    assert!(
        cell.rect.size.height > 140.0,
        "stacked drag cell should expand beyond the single-item minimum height"
    );
    for item in ["drag-item-0", "drag-item-1", "drag-item-2"] {
        let item = frame(&output, item);
        assert!(
            cell.rect.contains(Point::new(
                item.rect.origin.x + item.rect.size.width / 2.0,
                item.rect.origin.y + item.rect.size.height / 2.0,
            )),
            "stacked drag item should remain inside the expanded drop cell"
        );
    }
    assert!(
        frame(&output, "drag-grid").rect.bottom() >= cell.rect.bottom(),
        "drag grid should expand around an expanded drop cell"
    );
}

#[test]
fn box_model_specimens_cover_size_inset_and_flow_contracts() {
    let output = lab_output_with_size("layout", Size::new(TEST_WIDTH, 1600.0));

    assert!(
        output.scroll_chrome.iter().any(|chrome| {
            chrome.element_id.as_str() == "stage"
                && chrome.handle_rect.size.width == 2.0
                && chrome.handle_color.a == 118
                && chrome.track_color.is_some()
        }),
        "stage view pane should use styled scrollbar chrome"
    );

    assert_close(frame(&output, "box-auto-subject").rect.size.width, 12.0);
    assert_close(frame(&output, "box-auto-subject").rect.size.height, 12.0);
    assert_close(frame(&output, "box-px-subject").rect.size.width, 96.0);
    assert_close(frame(&output, "box-px-subject").rect.size.height, 44.0);
    assert_close(frame(&output, "box-min-subject").rect.size.width, 40.0);
    assert_close(frame(&output, "box-min-subject").rect.size.height, 40.0);
    assert_close(frame(&output, "box-max-subject").rect.size.width, 52.0);
    assert_close(frame(&output, "box-max-subject").rect.size.height, 34.0);
    assert!(
        frame(&output, "box-max-wide-child").rect.right()
            > frame(&output, "box-max-subject").rect.right(),
        "max-size specimen child should reveal the parent clamp"
    );

    assert_close(frame(&output, "box-fill-subject").rect.size.width, 298.0);
    assert_close(frame(&output, "box-percent-subject").rect.size.width, 149.0);
    assert_close(
        frame(&output, "box-height-fill-subject").rect.size.height,
        84.0,
    );

    let margin_subject = frame(&output, "box-margin-subject");
    let margin_frame = frame(&output, "box-margin-frame");
    assert_close(
        margin_subject.rect.origin.x - margin_frame.rect.origin.x,
        13.0,
    );
    assert_close(
        margin_subject.rect.origin.y - margin_frame.rect.origin.y,
        13.0,
    );

    assert_close(frame(&output, "box-padding-subject").rect.size.width, 36.0);
    assert_close(frame(&output, "box-padding-subject").rect.size.height, 36.0);
    assert_close(
        frame(&output, "box-border-subject").style.border_width.top,
        5.0,
    );
    assert_close(frame(&output, "box-border-subject").rect.size.width, 44.0);
    assert_close(frame(&output, "box-border-subject").rect.size.height, 44.0);

    assert_close(frame(&output, "box-row-gap-subject").rect.size.width, 56.0);
    assert_close(frame(&output, "box-row-gap-subject").rect.size.height, 12.0);
    let first_row_chip = frame(&output, "box-row-gap-chip-0");
    let second_row_chip = frame(&output, "box-row-gap-chip-1");
    assert_close(
        second_row_chip.rect.origin.x - first_row_chip.rect.origin.x,
        22.0,
    );

    assert_close(
        frame(&output, "box-column-gap-subject").rect.size.width,
        12.0,
    );
    assert_close(
        frame(&output, "box-column-gap-subject").rect.size.height,
        48.0,
    );
    let first_column_chip = frame(&output, "box-column-gap-chip-0");
    let second_column_chip = frame(&output, "box-column-gap-chip-1");
    assert_close(
        second_column_chip.rect.origin.y - first_column_chip.rect.origin.y,
        18.0,
    );

    let row_align_subject = frame(&output, "box-row-align-subject");
    let row_align_chip_0 = frame(&output, "box-row-align-chip-0");
    let row_align_chip_1 = frame(&output, "box-row-align-chip-1");
    assert_close(
        row_align_chip_0.rect.origin.x - row_align_subject.rect.origin.x,
        32.0,
    );
    assert_close(
        row_align_chip_1.rect.origin.x - row_align_chip_0.rect.origin.x,
        20.0,
    );
    assert_close(
        row_align_chip_0.rect.origin.y - row_align_subject.rect.origin.y,
        42.0,
    );

    let column_align_subject = frame(&output, "box-column-align-subject");
    let column_align_chip_0 = frame(&output, "box-column-align-chip-0");
    let column_align_chip_1 = frame(&output, "box-column-align-chip-1");
    let column_align_chip_2 = frame(&output, "box-column-align-chip-2");
    assert_close(
        column_align_chip_0.rect.origin.x - column_align_subject.rect.origin.x,
        34.0,
    );
    assert_close(
        column_align_chip_1.rect.origin.y - column_align_chip_0.rect.origin.y,
        40.0,
    );
    assert_close(
        column_align_chip_2.rect.origin.y - column_align_chip_1.rect.origin.y,
        40.0,
    );

    let visible_overflow_child = frame(&output, "box-visible-overflow-overflow-child");
    let visible_overflow_subject = frame(&output, "box-visible-overflow-subject");
    assert!(
        visible_overflow_child.rect.bottom() > visible_overflow_subject.rect.bottom(),
        "visible overflow child should extend beyond its square subject"
    );

    assert_close(
        frame(&output, "box-nested-nine-subject").rect.size.width,
        74.0,
    );
    assert_close(
        frame(&output, "box-nested-nine-subject").rect.size.height,
        74.0,
    );
    assert_close(
        frame(&output, "box-nested-nine-inner").rect.size.width,
        52.0,
    );
    assert_close(
        frame(&output, "box-nested-nine-inner").rect.size.height,
        52.0,
    );
    assert_close(
        frame(&output, "box-nested-nine-cell-0-1").rect.origin.x
            - frame(&output, "box-nested-nine-cell-0-0").rect.origin.x,
        14.0,
    );
    assert_close(
        frame(&output, "box-nested-nine-cell-1-0").rect.origin.y
            - frame(&output, "box-nested-nine-cell-0-0").rect.origin.y,
        14.0,
    );

    assert_close(
        frame(&output, "box-inset-percent-child").rect.size.width,
        34.0,
    );
    assert_close(
        frame(&output, "box-inset-percent-child").rect.size.height,
        34.0,
    );

    let absolute_parent_frame = frame(&output, "box-absolute-parent-parent");
    let absolute_parent_child = frame(&output, "box-absolute-parent-child");
    assert_close(
        absolute_parent_child.rect.origin.x - absolute_parent_frame.rect.origin.x,
        24.0,
    );
    assert_close(
        absolute_parent_child.rect.origin.y - absolute_parent_frame.rect.origin.y,
        18.0,
    );
    assert_close(
        frame(&output, "box-absolute-parent-subject")
            .rect
            .size
            .width,
        88.0,
    );

    let absolute_window_child = frame(&output, "box-absolute-window-child");
    assert_eq!(absolute_window_child.rect.origin, Point::new(420.0, 140.0));

    let overflow_output = lab_output_with_stage_scroll("layout", 620.0);
    assert!(
        overflow_output.scroll_chrome.iter().any(|chrome| {
            chrome.element_id.as_str() == "box-scroll-overflow-subject"
                && chrome.axis == ScrollAxis::Vertical
                && chrome.handle_color.a == 118
        }),
        "scroll overflow specimen should emit scroll chrome"
    );
    assert!(
        overflow_output.scroll_chrome.iter().any(|chrome| {
            chrome.element_id.as_str() == "box-scroll-x-overflow-subject"
                && chrome.axis == ScrollAxis::Horizontal
                && chrome.handle_rect.size.height == 2.0
                && chrome.handle_color.a == 118
        }),
        "horizontal scroll specimen should emit horizontal scroll chrome"
    );
    let two_axis_count = overflow_output
        .scroll_chrome
        .iter()
        .filter(|chrome| chrome.element_id.as_str() == "box-scroll-xy-overflow-subject")
        .count();
    assert_eq!(
        two_axis_count, 2,
        "two-axis scroll specimen should emit one chrome per axis"
    );
}

#[test]
fn scrolling_view_exercises_direct_and_nested_axis_overflow() {
    let output = lab_output("scrolling");

    assert_scroll_chrome(&output, "scroll-panel-a-list", ScrollAxis::Vertical);
    assert_scroll_chrome(&output, "scroll-panel-b-list", ScrollAxis::Horizontal);
    assert_scroll_chrome(
        &output,
        "scroll-panel-b-row-0-mini-list",
        ScrollAxis::Vertical,
    );
    assert_scroll_chrome(&output, "scroll-panel-c-list", ScrollAxis::Horizontal);
    assert_scroll_chrome(&output, "scroll-panel-c-list", ScrollAxis::Vertical);

    let nested_output = lab_output_with_stage_scroll("scrolling", 340.0);
    assert_scroll_chrome(
        &nested_output,
        "scroll-nested-vertical-list",
        ScrollAxis::Vertical,
    );
    assert_scroll_chrome(
        &nested_output,
        "scroll-nested-horizontal-list",
        ScrollAxis::Horizontal,
    );
    assert_scroll_chrome(
        &nested_output,
        "scroll-nested-horizontal-row-0-mini-list",
        ScrollAxis::Vertical,
    );
    assert_scroll_chrome(
        &nested_output,
        "scroll-nested-two-axis-list",
        ScrollAxis::Horizontal,
    );
    assert_scroll_chrome(
        &nested_output,
        "scroll-nested-two-axis-list",
        ScrollAxis::Vertical,
    );
}

#[test]
fn table_view_renders_document_table_roles_and_shared_tracks() {
    let output = lab_output("table");
    let table = frame(&output, "customer-preview-table");
    let header_customer = frame(&output, "customer-preview-header-customer");
    let row_customer = frame(&output, "customer-preview-row-0-customer");
    let header_revenue = frame(&output, "customer-preview-header-revenue");
    let row_revenue = frame(&output, "customer-preview-row-0-revenue");

    assert_eq!(table.role, ElementRole::Table);
    assert_eq!(header_customer.role, ElementRole::TableCell);
    assert_close(
        header_customer.rect.size.width,
        row_customer.rect.size.width,
    );
    assert_close(header_revenue.rect.origin.x, row_revenue.rect.origin.x);
    assert_scroll_chrome(&output, "customer-preview-table", ScrollAxis::Horizontal);
}

#[test]
fn text_view_renders_wrapped_and_truncated_specimens() {
    let output = lab_output("text");
    let wrapped = frame(&output, "text-wrap-body");
    let truncated = frame(&output, "text-truncate-body");
    let max_lines = frame(&output, "text-max-lines-body");

    assert_eq!(wrapped.style.text_wrap, TextWrapMode::Wrap);
    assert!(
        wrapped.text_layout.unwrap().line_count > 1,
        "text wrap specimen should be measured as multiple lines"
    );
    assert_eq!(truncated.style.text_wrap, TextWrapMode::Truncate);
    assert!(truncated.text_layout.unwrap().elided);
    assert_eq!(max_lines.style.max_lines, Some(2));
    assert!(max_lines.text_layout.unwrap().line_count <= 2);
    assert!(
        wrapped.rect.size.height > wrapped.text_layout.unwrap().size.height,
        "text specimens should include padding in the border-box height"
    );
}

#[test]
fn text_view_allows_pointer_selection_on_selectable_text() {
    let mut harness = lab_harness("text");
    let rect = state_rect_with_egui_text(harness.state(), &harness.ctx, "text-wrap-body");
    let start = egui::pos2(rect.origin.x + 12.0, rect.origin.y + 12.0);
    let end = egui::pos2(rect.origin.x + 145.0, rect.origin.y + 34.0);

    harness.hover_at(start);
    harness.drag_at(start);
    harness.hover_at(end);
    harness.run();

    let selection = harness
        .state()
        .document_engine
        .text_selection()
        .expect("dragging selectable text should create document text selection");
    assert_eq!(selection.target, ElementId::new("text-wrap-body"));
    assert!(selection.active);
    assert!(rect.contains(selection.anchor));
    assert!(rect.contains(selection.focus));
    assert_ne!(selection.anchor, selection.focus);
    assert_ne!(selection.anchor_index, selection.focus_index);
}

#[test]
fn text_view_copy_event_sends_selected_text_to_clipboard() {
    let mut harness = lab_harness("text");
    let rect = state_rect_with_egui_text(harness.state(), &harness.ctx, "text-wrap-body");
    let start = egui::pos2(rect.origin.x + 12.0, rect.origin.y + 12.0);
    let end = egui::pos2(rect.origin.x + 145.0, rect.origin.y + 34.0);

    harness.hover_at(start);
    harness.drag_at(start);
    harness.hover_at(end);
    harness.run();
    assert!(
        harness
            .state()
            .document_engine
            .text_selection()
            .is_some_and(|selection| !selection.is_empty()),
        "drag should leave a non-empty document text selection before copy"
    );
    harness.input_mut().modifiers = egui::Modifiers::COMMAND;
    harness.input_mut().events.push(egui::Event::Key {
        key: egui::Key::C,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::COMMAND,
    });
    harness.step();

    assert!(
        harness.output().platform_output.commands.iter().any(
            |command| matches!(command, egui::OutputCommand::CopyText(text) if !text.is_empty())
        ),
        "copy event should send the document selection to the platform clipboard"
    );
}

#[test]
fn text_context_menu_copy_uses_opened_selection_snapshot() {
    let mut harness = lab_harness("text");
    harness.state_mut().text_context_menu = Some(TextContextMenu {
        target: ElementId::new("text-wrap-body"),
        position: Point::new(300.0, 300.0),
        selected_text: Some("Customer analytics".to_owned()),
    });
    harness.run();

    harness.input_mut().events.push(egui::Event::PointerButton {
        pos: egui::pos2(320.0, 315.0),
        button: egui::PointerButton::Primary,
        pressed: true,
        modifiers: egui::Modifiers::NONE,
    });
    harness.step();
    harness.input_mut().events.push(egui::Event::PointerButton {
        pos: egui::pos2(320.0, 315.0),
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: egui::Modifiers::NONE,
    });
    harness.step();

    assert!(
        harness.output().platform_output.commands.iter().any(
            |command| matches!(command, egui::OutputCommand::CopyText(text) if text == "Customer analytics")
        ),
        "context menu copy should use the selection captured when the menu opened"
    );
    assert!(harness.state().text_context_menu.is_none());
}

#[test]
fn text_context_menu_closes_on_click_away() {
    let mut harness = lab_harness("text");
    harness.state_mut().text_context_menu = Some(TextContextMenu {
        target: ElementId::new("text-wrap-body"),
        position: Point::new(300.0, 300.0),
        selected_text: Some("Customer analytics".to_owned()),
    });
    harness.run();

    harness.drag_at(egui::pos2(20.0, 20.0));
    harness.drop_at(egui::pos2(20.0, 20.0));
    harness.run();

    assert!(harness.state().text_context_menu.is_none());
}

#[test]
fn animation_view_renders_state_driven_specimens() {
    let output = lab_output("animation");

    assert!(frame(&output, "animation-hover-size-box").interactive);
    assert!(frame(&output, "animation-hover-margin-target").interactive);
    assert!(frame(&output, "animation-pressed-border-box").interactive);

    let selected = frame(&output, "animation-selected-spacing-box");
    assert_eq!(selected.style.width, Length::Px(210.0));
    assert_eq!(selected.style.height, Length::Px(92.0));
    assert_close(selected.style.padding.top, 16.0);
    assert_close(selected.style.margin.top, 10.0);
    assert_close(selected.style.gap, 18.0);
    assert_eq!(selected.style.background, Some(SUCCESS_CONTAINER));
    assert_close(
        frame(&output, "animation-selected-spacing-box-label")
            .style
            .font_size,
        18.0,
    );

    let disabled = frame(&output, "animation-disabled-color-box");
    assert!(!disabled.interactive);
    assert_eq!(disabled.style.background, Some(SURFACE_CONTAINER));

    let focused = frame(&output, "animation-focused-min-size-box");
    assert_eq!(focused.style.width, Length::Px(226.0));
    assert_eq!(focused.style.height, Length::Px(88.0));
    assert_close(focused.style.min_size.width, 210.0);
    assert_close(focused.style.min_size.height, 78.0);
    assert_close(focused.style.border_width.top, 6.0);
    assert_eq!(focused.style.background, Some(SECONDARY_CONTAINER));

    assert_eq!(
        frame(&output, "drag-scroll-list-card").style.shadows.len(),
        2,
        "animation view should include the elevated drag list specimen"
    );
    assert!(
        frame(&output, "drag-scroll-handle-0").interactive,
        "animation drag specimen should preserve the real handle interaction"
    );
}

#[test]
fn animation_margin_specimen_expands_layout_on_hover() {
    let mut engine = DocumentEngine::default();
    let document =
        UiLabState::new(Some("animation")).document(Size::new(TEST_WIDTH, TEST_HEIGHT), false);
    let stylesheet = stylesheet();
    let base = engine.update(&document, &stylesheet);
    let target = frame(&base, "animation-hover-margin-target");
    let row = frame(&base, "animation-hover-margin-row");
    let after = frame(&base, "animation-hover-margin-after");
    let pointer = Point::new(
        target.rect.origin.x + target.rect.size.width / 2.0,
        target.rect.origin.y + target.rect.size.height / 2.0,
    );

    let hovered = engine.update_with_input(
        &document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: pointer,
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
    let hovered_target = frame(&hovered, "animation-hover-margin-target");
    let hovered_row = frame(&hovered, "animation-hover-margin-row");
    let hovered_after = frame(&hovered, "animation-hover-margin-after");

    assert!(
        hovered_target.style.margin.left > 0.0,
        "expected hover to ease margin above zero"
    );
    assert!(
        hovered_after.rect.origin.x > after.rect.origin.x,
        "expected animated target margin to push the following chip"
    );
    assert!(
        hovered_row.rect.size.height > row.rect.size.height,
        "expected auto-height parent to expand around animated child margin"
    );
}

#[test]
fn animation_margin_specimen_returns_to_idle_after_hover_exit() {
    let mut engine = DocumentEngine::default();
    let document =
        UiLabState::new(Some("animation")).document(Size::new(TEST_WIDTH, TEST_HEIGHT), false);
    let stylesheet = stylesheet();
    let base = engine.update(&document, &stylesheet);
    let target = frame(&base, "animation-hover-margin-target");
    let hover_pointer = Point::new(
        target.rect.origin.x + target.rect.size.width / 2.0,
        target.rect.origin.y + target.rect.size.height / 2.0,
    );

    let mut output = engine.update_with_input(
        &document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: hover_pointer,
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
    for _ in 0..32 {
        if !output.animating && !output.metrics.animation_changed_style {
            break;
        }
        output = engine.update_with_input(
            &document,
            &stylesheet,
            DocumentInput {
                pointer: Some(PointerInput {
                    position: hover_pointer,
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
    }
    assert!(!output.animating);
    assert!(!output.metrics.animation_changed_style);

    let exit_pointer = Point::new(4.0, 4.0);
    output = engine.update_with_input(
        &document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: exit_pointer,
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
    for _ in 0..32 {
        if !output.animating
            && !output.metrics.animation_changed_style
            && !output.metrics.input_changed_state
        {
            break;
        }
        output = engine.update_with_input(
            &document,
            &stylesheet,
            DocumentInput {
                pointer: Some(PointerInput {
                    position: exit_pointer,
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
    }

    assert!(!output.animating);
    assert!(!output.metrics.animation_changed_style);
    assert!(!output.metrics.input_changed_state);
    assert!(output.metrics.reused_input_layout);
}

#[test]
fn external_style_contract_can_drive_document_without_ui_lab_internals() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("outer"),
            Style::default()
                .width(Length::Auto)
                .height(Length::Auto)
                .margin(Insets::all(8.0))
                .border_width(3.0),
        )
        .rule(
            StyleSelector::id("inner"),
            Style::default()
                .width(Length::Auto)
                .height(Length::Auto)
                .padding(Insets::all(5.0))
                .gap(4.0)
                .border_width(2.0),
        )
        .rule(
            StyleSelector::class("row"),
            Style::default()
                .direction(des_ui_document::Direction::Row)
                .width(Length::Auto)
                .height(Length::Auto)
                .gap(4.0),
        )
        .rule(
            StyleSelector::class("cell"),
            Style::default().size(10.0, 10.0),
        );
    let document = Document::build(Size::new(240.0, 160.0), |ui| {
        ui.element("outer", ElementSpec::new(ElementRole::Panel), |ui| {
            ui.element("inner", ElementSpec::new(ElementRole::Panel), |ui| {
                for row in 0..3 {
                    ui.element(
                        format!("row-{row}"),
                        ElementSpec::new(ElementRole::Panel).class("row"),
                        |ui| {
                            for column in 0..3 {
                                ui.element(
                                    format!("cell-{row}-{column}"),
                                    ElementSpec::new(ElementRole::Panel).class("cell"),
                                    |_| {},
                                );
                            }
                        },
                    );
                }
            });
        });
    });

    let output = engine.update(&document, &stylesheet);

    assert_close(output.layout.find("outer").unwrap().rect.size.width, 58.0);
    assert_close(output.layout.find("outer").unwrap().rect.size.height, 58.0);
    assert_close(output.layout.find("inner").unwrap().rect.size.width, 52.0);
    assert_close(output.layout.find("inner").unwrap().rect.size.height, 52.0);
}
