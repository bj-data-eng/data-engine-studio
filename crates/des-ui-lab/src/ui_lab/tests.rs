use super::*;
use crate::graphics_testing::{
    TEST_HEIGHT, TEST_WIDTH, assert_exact_image_match, compare_images, image_stats, render_harness,
    test_harness,
};
use des_document::{
    DocumentEngine, DocumentInput, DocumentOutput, Element, ElementSpec, FontStyle, Insets, Length,
    OverflowWrap, Point, PointerInput, Position, ResolvedElement, ScrollAxis, Size, Style,
    StyleSelector, StyleSheet, TextDecoration, TextOverflow, TextWrapMode,
};
use des_egui::adapter::EguiTextMeasurer;
use egui_kittest::Harness;
#[cfg(not(debug_assertions))]
use std::time::{Duration, Instant};

const INTERACTION_LOOP_SCROLL_Y: f32 = 300.0;

fn lab_harness(initial_view: &str) -> Harness<'_, UiLabState> {
    test_harness(UiLabState::new(Some(initial_view)), |ui, state| {
        state.render(ui, false);
    })
}

fn lab_image(initial_view: &str) -> image::RgbaImage {
    render_harness(&mut lab_harness(initial_view))
}

fn lab_rect(id: &str) -> des_document::Rect {
    lab_rect_in("layout", id)
}

fn lab_rect_in(initial_view: &str, id: &str) -> des_document::Rect {
    let state = UiLabState::new(Some(initial_view));
    state_rect(&state, id)
}

fn state_rect(state: &UiLabState, id: &str) -> des_document::Rect {
    let output = state_output(state);
    find_frame(&output.layout, id)
        .unwrap_or_else(|| panic!("expected layout frame for {id}"))
        .rect
}

fn lab_output(initial_view: &str) -> DocumentOutput {
    lab_output_with_size(initial_view, Size::new(TEST_WIDTH, TEST_HEIGHT))
}

fn lab_output_with_size(initial_view: &str, size: Size) -> DocumentOutput {
    UiLabState::new(Some(initial_view)).lab_document_output_for_test(size)
}

fn lab_output_with_stage_scroll(initial_view: &str, scroll_y: f32) -> DocumentOutput {
    UiLabState::new(Some(initial_view)).lab_document_output_with_stage_scroll_for_test(
        Size::new(TEST_WIDTH, TEST_HEIGHT),
        scroll_y,
    )
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

fn frame_text<'a>(output: &'a DocumentOutput, id: &str) -> Option<&'a str> {
    frame(output, id)
        .text
        .as_ref()
        .map(|text| text.semantic_text())
}

fn state_output(state: &UiLabState) -> DocumentOutput {
    let viewport = state_viewport(state);
    let mut state = state.clone_for_retained_test();
    state.lab_document_output_for_test(viewport)
}

fn state_output_with_egui_text(state: &UiLabState, ctx: &egui::Context) -> DocumentOutput {
    let viewport = state_viewport(state);
    let mut state = state.clone_for_retained_test();
    let mut text_measurer = EguiTextMeasurer::new(ctx);
    state.lab_document_output_with_text_measurer_for_test(viewport, &mut text_measurer)
}

fn state_rect_with_egui_text(
    state: &UiLabState,
    ctx: &egui::Context,
    id: &str,
) -> des_document::Rect {
    let output = state_output_with_egui_text(state, ctx);
    find_frame(&output.layout, id)
        .unwrap_or_else(|| panic!("expected layout frame for {id}"))
        .rect
}

fn state_output_with_scroll(state: &UiLabState, scroll_y: f32) -> DocumentOutput {
    let viewport = state_viewport(state);
    let mut state = state.clone_for_retained_test();
    state.lab_document_output_with_stage_scroll_for_test(viewport, scroll_y)
}

fn state_viewport(state: &UiLabState) -> Size {
    state
        .lab_document
        .as_ref()
        .map(|retained| retained.viewport)
        .unwrap_or(Size::new(TEST_WIDTH, TEST_HEIGHT))
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

fn assert_length_px(actual: Length, expected: f32) {
    assert_eq!(actual, Length::Px(expected));
}

fn length_px(actual: Length) -> f32 {
    match actual {
        Length::Px(value) => value,
        other => panic!("expected px length, got {other:?}"),
    }
}

fn center(rect: des_document::Rect) -> egui::Pos2 {
    egui::pos2(
        rect.origin.x + rect.size.width / 2.0,
        rect.origin.y + rect.size.height / 2.0,
    )
}

fn click_at_stays_hovered(harness: &mut Harness<'_, UiLabState>, pos: egui::Pos2) {
    harness.event(egui::Event::PointerMoved(pos));
    harness.event(egui::Event::PointerButton {
        pos,
        button: egui::PointerButton::Primary,
        pressed: true,
        modifiers: egui::Modifiers::NONE,
    });
    harness.event(egui::Event::PointerButton {
        pos,
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: egui::Modifiers::NONE,
    });
    harness.run();
}

#[cfg(not(debug_assertions))]
fn wheel_at(harness: &mut Harness<'_, UiLabState>, pos: egui::Pos2, delta: egui::Vec2) {
    harness.event(egui::Event::PointerMoved(pos));
    harness.event(egui::Event::MouseWheel {
        unit: egui::MouseWheelUnit::Point,
        delta,
        phase: egui::TouchPhase::Move,
        modifiers: egui::Modifiers::default(),
    });
}

fn count_visible_text_selection_pixels_in_rect(
    image: &image::RgbaImage,
    rect: des_document::Rect,
) -> usize {
    let min_x = rect.origin.x.floor().max(0.0) as u32;
    let min_y = rect.origin.y.floor().max(0.0) as u32;
    let max_x = rect.right().ceil().clamp(0.0, image.width() as f32) as u32;
    let max_y = rect.bottom().ceil().clamp(0.0, image.height() as f32) as u32;
    let mut count = 0usize;
    for y in min_y..max_y {
        for x in min_x..max_x {
            let [red, green, blue, alpha] = image.get_pixel(x, y).0;
            if alpha > 220
                && (90..=150).contains(&red)
                && (70..=125).contains(&green)
                && (145..=205).contains(&blue)
            {
                count += 1;
            }
        }
    }
    count
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
        "expected the UI lab to produce a non-trivial painted output"
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
fn clicked_draggable_nav_view_matches_directly_seeded_view() {
    let mut clicked = lab_harness("layout");
    let draggable_nav_item = center(lab_rect("view-draggable"));

    clicked.hover_at(draggable_nav_item);
    clicked.drag_at(draggable_nav_item);
    clicked.drop_at(draggable_nav_item);
    clicked.run();

    let clicked_image = render_harness(&mut clicked);
    let direct_image = lab_image("draggable");

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
fn clicked_floating_nav_view_matches_directly_seeded_view() {
    let mut clicked = lab_harness("layout");
    let floating_nav_item = center(lab_rect("view-floating"));

    clicked.hover_at(floating_nav_item);
    clicked.drag_at(floating_nav_item);
    clicked.drop_at(floating_nav_item);
    clicked.run();

    let clicked_image = render_harness(&mut clicked);
    let direct_image = lab_image("floating");

    assert_exact_image_match(&clicked_image, &direct_image);
}

#[test]
fn floating_view_exercises_fallback_shift_and_optional_arrow() {
    std::thread::Builder::new()
        .name("floating-contracts".to_string())
        .stack_size(8 * 1024 * 1024)
        .spawn(floating_view_exercises_fallback_shift_and_optional_arrow_body)
        .expect("floating contract test thread should start")
        .join()
        .expect("floating contract test should pass");
}

fn floating_view_exercises_fallback_shift_and_optional_arrow_body() {
    let output = lab_output("floating");

    let playground = frame(&output, "floating-playground");
    let specimen = frame(&output, "floating-offset-specimen");
    let specimen_title = frame(&output, "floating-offset-specimen-title");
    let offset_row = frame(&output, "floating-offset-row");
    let zero_reference = frame(&output, "floating-offset-zero-reference");
    let zero_reference_label = frame(&output, "floating-offset-zero-reference-label");
    let zero_popover = frame(&output, "floating-offset-zero-popover");
    let zero_popover_label = frame(&output, "floating-offset-zero-popover-label");
    let ten_reference = frame(&output, "floating-offset-ten-reference");
    let ten_popover = frame(&output, "floating-offset-ten-popover");
    let main_axis_specimen = frame(&output, "floating-main-axis-specimen");
    let top_reference = frame(&output, "floating-main-axis-top-reference");
    let top_popover = frame(&output, "floating-main-axis-top-popover");
    let bottom_reference = frame(&output, "floating-main-axis-bottom-reference");
    let bottom_popover = frame(&output, "floating-main-axis-bottom-popover");
    let left_reference = frame(&output, "floating-main-axis-left-reference");
    let left_popover = frame(&output, "floating-main-axis-left-popover");
    let right_reference = frame(&output, "floating-main-axis-right-reference");
    let right_popover = frame(&output, "floating-main-axis-right-popover");
    let cross_axis_specimen = frame(&output, "floating-cross-axis-specimen");
    let cross_top_reference = frame(&output, "floating-cross-axis-top-reference");
    let cross_top_popover = frame(&output, "floating-cross-axis-top-popover");
    let cross_bottom_reference = frame(&output, "floating-cross-axis-bottom-reference");
    let cross_bottom_popover = frame(&output, "floating-cross-axis-bottom-popover");
    let cross_left_reference = frame(&output, "floating-cross-axis-left-reference");
    let cross_left_popover = frame(&output, "floating-cross-axis-left-popover");
    let cross_right_reference = frame(&output, "floating-cross-axis-right-reference");
    let cross_right_popover = frame(&output, "floating-cross-axis-right-popover");
    let alignment_axis_specimen = frame(&output, "floating-alignment-axis-specimen");
    let alignment_cross_start_reference =
        frame(&output, "floating-alignment-axis-cross-start-reference");
    let alignment_cross_start_popover =
        frame(&output, "floating-alignment-axis-cross-start-popover");
    let alignment_cross_end_reference =
        frame(&output, "floating-alignment-axis-cross-end-reference");
    let alignment_cross_end_popover = frame(&output, "floating-alignment-axis-cross-end-popover");
    let alignment_start_reference = frame(&output, "floating-alignment-axis-start-reference");
    let alignment_start_popover = frame(&output, "floating-alignment-axis-start-popover");
    let alignment_end_reference = frame(&output, "floating-alignment-axis-end-reference");
    let alignment_end_popover = frame(&output, "floating-alignment-axis-end-popover");
    let centered_axis_specimen = frame(&output, "floating-centered-axis-specimen");
    let centered_reference = frame(&output, "floating-centered-axis-reference");
    let centered_popover = frame(&output, "floating-centered-axis-popover");
    let top_start_specimen = frame(&output, "floating-top-start-specimen");
    let top_start_reference = frame(&output, "floating-top-start-reference");
    let top_start_popover = frame(&output, "floating-top-start-popover");
    let scroll_shift_specimen = frame(&output, "floating-scroll-shift-specimen");
    let scroll_shift_panel = frame(&output, "floating-scroll-shift-panel");
    let scroll_shift_reference = frame(&output, "floating-scroll-shift-reference");
    let scroll_shift_popover = frame(&output, "floating-scroll-shift-popover");
    let scroll_attach_specimen = frame(&output, "floating-scroll-attach-specimen");
    let scroll_attach_reference = frame(&output, "floating-scroll-attach-reference");
    let scroll_attach_popover = frame(&output, "floating-scroll-attach-popover");
    let vertical_overlap_specimen = frame(&output, "floating-vertical-overlap-specimen");
    let vertical_overlap_panel = frame(&output, "floating-vertical-overlap-panel");
    let vertical_overlap_reference = frame(&output, "floating-vertical-overlap-reference");
    let vertical_overlap_popover = frame(&output, "floating-vertical-overlap-popover");
    let vertical_flip_specimen = frame(&output, "floating-vertical-flip-specimen");
    let vertical_flip_reference = frame(&output, "floating-vertical-flip-reference");
    let vertical_flip_popover = frame(&output, "floating-vertical-flip-popover");
    let edge_flip_specimen = frame(&output, "floating-edge-flip-specimen");
    let edge_flip_panel = frame(&output, "floating-edge-flip-panel");
    let edge_flip_reference = frame(&output, "floating-edge-flip-reference");
    let edge_flip_popover = frame(&output, "floating-edge-flip-popover");

    assert_eq!(specimen.style.position, Position::Flow);
    assert_eq!(main_axis_specimen.style.position, Position::Flow);
    assert_eq!(cross_axis_specimen.style.position, Position::Flow);
    assert_eq!(alignment_axis_specimen.style.position, Position::Flow);
    assert_eq!(centered_axis_specimen.style.position, Position::Flow);
    assert_eq!(top_start_specimen.style.position, Position::Flow);
    assert_eq!(scroll_shift_specimen.style.position, Position::Flow);
    assert_eq!(scroll_attach_specimen.style.position, Position::Flow);
    assert_eq!(vertical_overlap_specimen.style.position, Position::Flow);
    assert_eq!(vertical_flip_specimen.style.position, Position::Flow);
    assert_eq!(edge_flip_specimen.style.position, Position::Flow);
    assert_eq!(zero_reference.style.position, Position::Flow);
    assert_eq!(ten_reference.style.position, Position::Flow);
    assert_eq!(top_reference.style.position, Position::Flow);
    assert_eq!(bottom_reference.style.position, Position::Flow);
    assert_eq!(left_reference.style.position, Position::Flow);
    assert_eq!(right_reference.style.position, Position::Flow);
    for reference in [
        zero_reference,
        ten_reference,
        top_reference,
        bottom_reference,
        left_reference,
        right_reference,
        cross_top_reference,
        cross_bottom_reference,
        cross_left_reference,
        cross_right_reference,
        alignment_cross_start_reference,
        alignment_cross_end_reference,
        alignment_start_reference,
        alignment_end_reference,
        centered_reference,
        top_start_reference,
        scroll_shift_reference,
        scroll_attach_reference,
        vertical_overlap_reference,
        vertical_flip_reference,
        edge_flip_reference,
    ] {
        assert_close(reference.rect.size.width, 56.0);
        assert_close(reference.rect.size.height, 56.0);
    }
    assert_close(zero_reference_label.style.font_size, 9.0);
    assert_close(zero_popover_label.style.font_size, 9.0);
    let playground_gap = length_px(playground.style.column_gap);
    let playground_content_width =
        playground.rect.size.width - playground.style.padding.horizontal();
    assert_eq!(specimen.style.width, Length::calc(0.33333334, -6.666667));
    let expected_specimen_width = playground_content_width / 3.0 - playground_gap * 2.0 / 3.0;
    assert!((specimen.rect.size.width - expected_specimen_width).abs() <= 1.0);
    assert_close(
        specimen_title.rect.origin.y,
        specimen.rect.origin.y + specimen.style.border_width.top + specimen.style.padding.top,
    );
    assert_close(specimen.style.padding.top, 8.0);
    assert_close(specimen_title.rect.size.height, 18.0);
    let offset_gap = length_px(offset_row.style.gap);
    let offset_pair_width =
        zero_reference.rect.size.width + ten_reference.rect.size.width + offset_gap;
    let expected_zero_reference_x =
        offset_row.rect.origin.x + (offset_row.rect.size.width - offset_pair_width) * 0.5;
    assert!((zero_reference.rect.origin.x - expected_zero_reference_x).abs() <= 1.0);
    assert_close(
        ten_reference.rect.origin.x,
        zero_reference.rect.right() + offset_gap,
    );
    assert_close(
        zero_reference.rect.origin.y + zero_reference.rect.size.height * 0.5,
        offset_row.rect.origin.y + offset_row.rect.size.height * 0.5,
    );
    assert_close(
        ten_reference.rect.origin.y + ten_reference.rect.size.height * 0.5,
        offset_row.rect.origin.y + offset_row.rect.size.height * 0.5,
    );
    assert!((main_axis_specimen.rect.size.width - specimen.rect.size.width).abs() <= 1.0);
    assert_close(
        main_axis_specimen.rect.origin.x,
        specimen.rect.right() + playground_gap,
    );
    assert_close(main_axis_specimen.rect.origin.y, specimen.rect.origin.y);
    assert!((cross_axis_specimen.rect.size.width - specimen.rect.size.width).abs() <= 1.0);
    assert_close(cross_axis_specimen.rect.origin.x, specimen.rect.origin.x);
    assert_close(
        cross_axis_specimen.rect.origin.y,
        main_axis_specimen.rect.bottom() + playground_gap,
    );
    assert!((alignment_axis_specimen.rect.size.width - specimen.rect.size.width).abs() <= 1.0);
    assert_close(
        alignment_axis_specimen.rect.origin.x,
        cross_axis_specimen.rect.right() + playground_gap,
    );
    assert_close(
        alignment_axis_specimen.rect.origin.y,
        cross_axis_specimen.rect.origin.y,
    );
    assert!((centered_axis_specimen.rect.size.width - specimen.rect.size.width).abs() <= 1.0);
    assert_close(centered_axis_specimen.rect.origin.x, specimen.rect.origin.x);
    assert_close(
        centered_axis_specimen.rect.origin.y,
        cross_axis_specimen.rect.bottom() + playground_gap,
    );
    assert!((top_start_specimen.rect.size.width - specimen.rect.size.width).abs() <= 1.0);
    assert_close(
        top_start_specimen.rect.origin.x,
        centered_axis_specimen.rect.right() + playground_gap,
    );
    assert_close(
        top_start_specimen.rect.origin.y,
        centered_axis_specimen.rect.origin.y,
    );
    assert!((scroll_shift_specimen.rect.size.width - specimen.rect.size.width).abs() <= 1.0);
    assert_close(scroll_shift_specimen.rect.origin.x, specimen.rect.origin.x);
    assert_close(
        scroll_shift_specimen.rect.origin.y,
        centered_axis_specimen.rect.bottom() + playground_gap,
    );
    assert!((scroll_attach_specimen.rect.size.width - specimen.rect.size.width).abs() <= 1.0);
    assert_close(
        scroll_attach_specimen.rect.origin.x,
        scroll_shift_specimen.rect.right() + playground_gap,
    );
    assert_close(
        scroll_attach_specimen.rect.origin.y,
        scroll_shift_specimen.rect.origin.y,
    );
    assert!((vertical_overlap_specimen.rect.size.width - specimen.rect.size.width).abs() <= 1.0);
    assert_close(
        vertical_overlap_specimen.rect.origin.x,
        specimen.rect.origin.x,
    );
    assert!(vertical_overlap_specimen.rect.origin.y >= scroll_shift_specimen.rect.origin.y);
    assert!((vertical_flip_specimen.rect.size.width - specimen.rect.size.width).abs() <= 1.0);
    assert!(vertical_flip_specimen.rect.origin.x >= specimen.rect.origin.x);
    assert!(vertical_flip_specimen.rect.origin.y >= scroll_shift_specimen.rect.origin.y);
    assert!((edge_flip_specimen.rect.size.width - specimen.rect.size.width).abs() <= 1.0);
    assert_close(edge_flip_specimen.rect.origin.x, specimen.rect.origin.x);
    assert!(edge_flip_specimen.rect.origin.y >= vertical_overlap_specimen.rect.origin.y);
    assert!(zero_reference.rect.origin.x >= specimen.rect.origin.x);
    assert!(ten_reference.rect.origin.x > zero_reference.rect.origin.x);
    assert_close(ten_reference.rect.origin.y, zero_reference.rect.origin.y);
    assert_close(top_popover.rect.bottom(), top_reference.rect.origin.y);
    assert_close(bottom_popover.rect.origin.y, bottom_reference.rect.bottom());
    assert_close(left_popover.rect.right(), left_reference.rect.origin.x);
    assert_close(right_popover.rect.origin.x, right_reference.rect.right());
    assert_close(
        cross_top_popover.rect.bottom(),
        cross_top_reference.rect.origin.y,
    );
    assert_close(
        cross_top_popover.rect.right(),
        cross_top_reference.rect.right(),
    );
    assert_close(
        cross_bottom_popover.rect.origin.y,
        cross_bottom_reference.rect.bottom(),
    );
    assert_close(
        cross_bottom_popover.rect.right(),
        cross_bottom_reference.rect.right(),
    );
    assert_close(
        cross_left_popover.rect.right(),
        cross_left_reference.rect.origin.x,
    );
    assert_close(
        cross_left_popover.rect.bottom(),
        cross_left_reference.rect.bottom(),
    );
    assert_close(
        cross_right_popover.rect.origin.x,
        cross_right_reference.rect.right(),
    );
    assert_close(
        cross_right_popover.rect.bottom(),
        cross_right_reference.rect.bottom(),
    );
    assert_close(
        alignment_cross_start_popover.rect.bottom(),
        alignment_cross_start_reference.rect.origin.y,
    );
    assert_close(
        alignment_cross_start_popover.rect.origin.x,
        alignment_cross_start_reference.rect.origin.x + 8.0,
    );
    assert_close(
        alignment_cross_end_popover.rect.bottom(),
        alignment_cross_end_reference.rect.origin.y,
    );
    assert_close(
        alignment_cross_end_popover.rect.origin.x,
        alignment_cross_end_reference.rect.right() - alignment_cross_end_popover.rect.size.width
            + 8.0,
    );
    assert_close(
        alignment_start_popover.rect.bottom(),
        alignment_start_reference.rect.origin.y,
    );
    assert_close(
        alignment_start_popover.rect.origin.x,
        alignment_start_reference.rect.origin.x + 8.0,
    );
    assert_close(
        alignment_end_popover.rect.bottom(),
        alignment_end_reference.rect.origin.y,
    );
    assert_close(
        alignment_end_popover.rect.origin.x,
        alignment_end_reference.rect.right() - alignment_end_popover.rect.size.width - 8.0,
    );
    assert_close(
        centered_popover.rect.origin.x + centered_popover.rect.size.width * 0.5,
        centered_reference.rect.origin.x + centered_reference.rect.size.width * 0.5,
    );
    assert_close(
        centered_popover.rect.origin.y + centered_popover.rect.size.height * 0.5,
        centered_reference.rect.origin.y + centered_reference.rect.size.height * 0.5,
    );
    assert_close(
        top_start_popover.rect.origin.x,
        top_start_reference.rect.origin.x - top_start_popover.rect.size.width,
    );
    assert_close(
        top_start_popover.rect.bottom(),
        top_start_reference.rect.origin.y,
    );
    let scroll_boundary_right =
        scroll_shift_panel.rect.right() - scroll_shift_panel.style.border_width.right;
    assert!(scroll_shift_reference.rect.origin.x > scroll_boundary_right);
    assert_close(scroll_shift_popover.rect.right(), scroll_boundary_right);
    assert_close(
        scroll_attach_popover.rect.origin.y,
        scroll_attach_reference.rect.bottom(),
    );
    assert_close(
        scroll_attach_popover.rect.origin.x + scroll_attach_popover.rect.size.width * 0.5,
        scroll_attach_reference.rect.origin.x + scroll_attach_reference.rect.size.width * 0.5,
    );
    assert_close(
        vertical_overlap_popover.rect.bottom(),
        vertical_overlap_reference.rect.origin.y,
    );
    assert_close(
        vertical_overlap_popover.rect.origin.x + vertical_overlap_popover.rect.size.width * 0.5,
        vertical_overlap_reference.rect.origin.x + vertical_overlap_reference.rect.size.width * 0.5,
    );
    assert_close(
        vertical_flip_popover.rect.bottom(),
        vertical_flip_reference.rect.origin.y,
    );
    assert_close(
        vertical_flip_popover.rect.origin.x + vertical_flip_popover.rect.size.width * 0.5,
        vertical_flip_reference.rect.origin.x + vertical_flip_reference.rect.size.width * 0.5,
    );
    assert_close(
        edge_flip_popover.rect.bottom(),
        edge_flip_reference.rect.origin.y,
    );
    assert_close(
        edge_flip_popover.rect.origin.x,
        edge_flip_reference.rect.origin.x,
    );
    let scrolled_output = lab_output_with_stage_scroll("floating", 900.0);
    assert!(
        scrolled_output.scroll_chrome.iter().any(|chrome| {
            chrome.element_id.as_str() == "floating-scroll-shift-panel"
                && chrome.axis == ScrollAxis::Horizontal
                && chrome.visible
        }),
        "floating scroll boundary specimen should expose a visible horizontal scrollbar when it is within the stage viewport"
    );
    let mut inner_scrolled = UiLabState::new(Some("floating"));
    inner_scrolled.lab_document_output_for_test(Size::new(TEST_WIDTH, TEST_HEIGHT));
    inner_scrolled
        .document_engine
        .element_state_mut("floating-scroll-attach-panel")
        .unwrap()
        .scroll_x = 240.0;
    let inner_scrolled_output =
        inner_scrolled.lab_document_output_for_test(Size::new(TEST_WIDTH, TEST_HEIGHT));
    let scrolled_attach_reference = frame(&scrolled_output, "floating-scroll-attach-reference");
    let scrolled_attach_popover = frame(&scrolled_output, "floating-scroll-attach-popover");
    let inner_scrolled_attach_reference =
        frame(&inner_scrolled_output, "floating-scroll-attach-reference");
    let inner_scrolled_attach_popover =
        frame(&inner_scrolled_output, "floating-scroll-attach-popover");
    assert_close(
        scrolled_attach_popover.rect.origin.x + scrolled_attach_popover.rect.size.width * 0.5,
        scrolled_attach_reference.rect.origin.x + scrolled_attach_reference.rect.size.width * 0.5,
    );
    assert_close(
        inner_scrolled_attach_popover.rect.origin.x
            + inner_scrolled_attach_popover.rect.size.width * 0.5,
        inner_scrolled_attach_reference.rect.origin.x
            + inner_scrolled_attach_reference.rect.size.width * 0.5,
    );
    assert!(
        inner_scrolled_attach_popover.rect.origin.x < scroll_attach_popover.rect.origin.x,
        "attached popover should continue moving with horizontally scrolled content"
    );
    let mut vertical_scrolled = UiLabState::new(Some("floating"));
    vertical_scrolled.lab_document_output_for_test(Size::new(TEST_WIDTH, TEST_HEIGHT));
    vertical_scrolled
        .document_engine
        .element_state_mut("floating-vertical-overlap-panel")
        .unwrap()
        .scroll_y = 96.0;
    let vertical_scrolled_output =
        vertical_scrolled.lab_document_output_for_test(Size::new(TEST_WIDTH, TEST_HEIGHT));
    let scrolled_vertical_reference = frame(
        &vertical_scrolled_output,
        "floating-vertical-overlap-reference",
    );
    let scrolled_vertical_popover = frame(
        &vertical_scrolled_output,
        "floating-vertical-overlap-popover",
    );
    let vertical_boundary_top =
        vertical_overlap_panel.rect.origin.y + vertical_overlap_panel.style.border_width.top;
    assert_close(
        scrolled_vertical_popover.rect.origin.y,
        vertical_boundary_top,
    );
    assert!(
        scrolled_vertical_popover.rect.bottom() > scrolled_vertical_reference.rect.origin.y,
        "vertical boundary shift should allow the floating element to overlap its reference"
    );
    let vertical_scrolled_stage = lab_output_with_stage_scroll("floating", 1200.0);
    assert!(
        vertical_scrolled_stage.scroll_chrome.iter().any(|chrome| {
            chrome.element_id.as_str() == "floating-vertical-overlap-panel"
                && chrome.axis == ScrollAxis::Vertical
                && chrome.visible
        }),
        "vertical overlap specimen should expose a visible vertical scrollbar when it is within the stage viewport"
    );
    let mut vertical_unflipped = UiLabState::new(Some("floating"));
    vertical_unflipped.lab_document_output_for_test(Size::new(TEST_WIDTH, TEST_HEIGHT));
    vertical_unflipped
        .document_engine
        .element_state_mut("floating-vertical-flip-panel")
        .unwrap()
        .scroll_y = 170.0;
    let vertical_unflipped_output =
        vertical_unflipped.lab_document_output_for_test(Size::new(TEST_WIDTH, TEST_HEIGHT));
    let unflipped_vertical_reference = frame(
        &vertical_unflipped_output,
        "floating-vertical-flip-reference",
    );
    let unflipped_vertical_popover =
        frame(&vertical_unflipped_output, "floating-vertical-flip-popover");
    assert_close(
        unflipped_vertical_popover.rect.origin.y,
        unflipped_vertical_reference.rect.bottom(),
    );
    assert!(
        vertical_flip_popover.rect.bottom() <= vertical_flip_reference.rect.origin.y,
        "vertical flip should keep the floating element from overlapping its reference"
    );
    let mut edge_aligned = UiLabState::new(Some("floating"));
    edge_aligned.lab_document_output_for_test(Size::new(TEST_WIDTH, TEST_HEIGHT));
    edge_aligned
        .document_engine
        .element_state_mut("floating-edge-flip-panel")
        .unwrap()
        .scroll_x = 0.0;
    edge_aligned
        .document_engine
        .element_state_mut("floating-edge-flip-panel")
        .unwrap()
        .scroll_y = 170.0;
    let edge_aligned_output =
        edge_aligned.lab_document_output_for_test(Size::new(TEST_WIDTH, TEST_HEIGHT));
    let aligned_edge_reference = frame(&edge_aligned_output, "floating-edge-flip-reference");
    let aligned_edge_popover = frame(&edge_aligned_output, "floating-edge-flip-popover");
    assert_close(
        aligned_edge_popover.rect.origin.y,
        aligned_edge_reference.rect.bottom(),
    );
    assert_close(
        aligned_edge_popover.rect.origin.x,
        aligned_edge_reference.rect.origin.x,
    );
    let mut edge_start = UiLabState::new(Some("floating"));
    edge_start.lab_document_output_for_test(Size::new(TEST_WIDTH, TEST_HEIGHT));
    {
        let state = edge_start
            .document_engine
            .element_state_mut("floating-edge-flip-panel")
            .unwrap();
        state.scroll_x = 210.0;
        state.scroll_y = 170.0;
    }
    let edge_start_output =
        edge_start.lab_document_output_for_test(Size::new(TEST_WIDTH, TEST_HEIGHT));
    let start_edge_reference = frame(&edge_start_output, "floating-edge-flip-reference");
    let start_edge_popover = frame(&edge_start_output, "floating-edge-flip-popover");
    assert_close(
        start_edge_popover.rect.origin.y,
        start_edge_reference.rect.bottom(),
    );
    assert!((start_edge_popover.rect.origin.x - start_edge_reference.rect.origin.x).abs() <= 4.0);
    let mut edge_corner = UiLabState::new(Some("floating"));
    edge_corner.lab_document_output_for_test(Size::new(TEST_WIDTH, TEST_HEIGHT));
    {
        let state = edge_corner
            .document_engine
            .element_state_mut("floating-edge-flip-panel")
            .unwrap();
        state.scroll_x = 0.0;
        state.scroll_y = 0.0;
    }
    let edge_corner_output =
        edge_corner.lab_document_output_for_test(Size::new(TEST_WIDTH, TEST_HEIGHT));
    let corner_edge_reference = frame(&edge_corner_output, "floating-edge-flip-reference");
    let corner_edge_popover = frame(&edge_corner_output, "floating-edge-flip-popover");
    let edge_boundary_right =
        edge_flip_panel.rect.right() - edge_flip_panel.style.border_width.right;
    assert_close(
        corner_edge_popover.rect.bottom(),
        corner_edge_reference.rect.origin.y,
    );
    assert_close(
        corner_edge_popover.rect.origin.x,
        corner_edge_reference.rect.origin.x,
    );
    assert!(corner_edge_popover.rect.right() <= edge_boundary_right);
    assert!(zero_reference.interactive);
    assert!(zero_popover.interactive);
    assert!(top_reference.interactive);
    assert!(top_popover.interactive);
    assert_eq!(
        zero_reference.style.border_style,
        des_document::BorderStyle::Dashed
    );
    assert_close(zero_popover.rect.origin.y, zero_reference.rect.bottom());
    assert_close(
        ten_popover.rect.origin.y - ten_reference.rect.bottom(),
        10.0,
    );
}

#[test]
fn nav_uses_explicit_card_spacing_and_styled_scrollbar() {
    let output = lab_output("text");
    let nav = frame(&output, "nav");
    let nav_item = frame(&output, "view-layout");

    assert!(has_class(nav, "styled-scrollbar"));
    assert_eq!(
        nav.style.scrollbar_handle_color,
        Color::rgba(103, 80, 164, 118)
    );
    assert_eq!(
        nav.style.scrollbar_track_color,
        Some(Color::rgba(103, 80, 164, 28))
    );
    assert_close(nav_item.style.padding.top, 12.0);
    assert_close(nav_item.style.padding.right, 12.0);
    assert_close(nav_item.style.padding.bottom, 12.0);
    assert_close(nav_item.style.padding.left, 12.0);
    assert_length_px(nav_item.style.gap, 5.0);
    assert_close(nav_item.style.radius.top_left, 7.0);
    assert_close(nav_item.style.radius.top_right, 7.0);
    assert_close(nav_item.style.radius.bottom_right, 7.0);
    assert_close(nav_item.style.radius.bottom_left, 7.0);
}

#[test]
fn lab_shell_tracks_document_viewport_size() {
    for viewport in [Size::new(1180.0, 720.0), Size::new(1480.0, 920.0)] {
        let output = lab_output_with_size("layout", viewport);
        let lab_root = frame(&output, "lab-root");
        let topbar = frame(&output, "topbar");
        let lab_body = frame(&output, "lab-body");
        let nav = frame(&output, "nav");
        let stage = frame(&output, "stage");

        assert_eq!(lab_root.rect.size, viewport);
        assert_eq!(topbar.rect.size.width, viewport.width);
        assert_eq!(lab_body.rect.size.width, viewport.width);
        assert_eq!(
            lab_body.rect.size.height,
            viewport.height - topbar.rect.size.height
        );
        assert_eq!(nav.rect.size.width, 242.0);
        assert_eq!(
            nav.rect.size.height,
            lab_body.rect.size.height - lab_body.style.padding.top - lab_body.style.padding.bottom
        );
        assert_eq!(stage.rect.size.height, nav.rect.size.height);
        assert_eq!(
            stage.rect.size.width,
            lab_body.rect.size.width
                - lab_body.style.padding.left
                - lab_body.style.padding.right
                - length_px(lab_body.style.gap)
                - nav.rect.size.width
        );
    }
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
    assert_eq!(frame(&output, "shadow-single").style.shadows.len(), 1);
    assert_eq!(frame(&output, "shadow-layered").style.shadows.len(), 1);
    assert_eq!(frame(&output, "shadow-light-top").style.shadows.len(), 1);
    assert_eq!(frame(&output, "shadow-web-top").style.shadows.len(), 1);
    assert_eq!(frame(&output, "shadow-web-bottom").style.shadows.len(), 0);
    assert_close(
        frame(&output, "shadow-negative-spread").style.shadows[0].spread,
        0.0,
    );
    let shadow_tune_copy = frame(&output, "shadow-tune-copy");
    assert_eq!(shadow_tune_copy.style.width, Length::Percent(1.0));
    assert_eq!(shadow_tune_copy.style.height, Length::Auto);
}

#[test]
fn interaction_view_renders_common_control_elements() {
    let output = lab_output("interaction");

    assert_eq!(
        frame(&output, "control-checkbox").element,
        Element::Checkbox
    );
    assert_eq!(
        frame(&output, "control-radio-local").element,
        Element::Radio
    );
    assert_eq!(
        frame(&output, "control-dropdown-trigger").element,
        Element::Select
    );
    assert_eq!(frame(&output, "control-input-name").element, Element::Input);
    assert!(frame(&output, "control-checkbox").interactive);
    assert!(frame(&output, "control-dropdown").interactive);
    assert!(frame(&output, "control-input-name").interactive);
    assert!(!frame(&output, "control-input-disabled").interactive);
    assert!(
        output
            .snapshot()
            .find("control-radio-local-dot-fill")
            .is_some(),
        "selected radio option should render an explicit inner dot"
    );
    assert!(
        output
            .snapshot()
            .find("control-radio-remote-dot-fill")
            .is_none(),
        "unselected radio option should not render an inner dot"
    );
    assert_eq!(
        frame(&output, "control-radio-local-dot").style.background,
        Some(PANEL),
        "selected radio ring should not collapse into a solid filled circle"
    );
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
fn radio_click_refreshes_retained_document_in_same_frame() {
    let mut harness = lab_harness("interaction");
    let target = center(state_rect(harness.state(), "control-radio-remote"));

    harness.hover_at(target);
    harness.drag_at(target);
    harness.drop_at(target);
    harness.run();

    assert_eq!(harness.state().radio_choice, 1);
    assert_eq!(
        harness
            .state()
            .lab_document
            .as_ref()
            .expect("render should retain the lab document")
            .key
            .radio_choice,
        1,
        "clicked radio action should rebuild the retained document before paint settles"
    );
}

#[test]
fn dropdown_menu_matches_trigger_width() {
    let mut state = UiLabState::new(Some("interaction"));
    state.dropdown_open = true;
    let output = state.lab_document_output_for_test(Size::new(TEST_WIDTH, TEST_HEIGHT));
    let trigger = frame(&output, "control-dropdown-trigger");
    let menu = frame(&output, "control-dropdown-menu");

    assert_close(menu.rect.origin.x, trigger.rect.origin.x);
    assert_close(menu.rect.size.width, trigger.rect.size.width);
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
        frame_text(&output, "loop-checkbox-result"),
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
        frame_text(&output, "loop-button-result"),
        Some("Button events received: 1")
    );
    assert_eq!(
        frame(&output, "loop-button-result-box").value.as_deref(),
        Some("button-count=1")
    );
    assert_eq!(
        frame_text(&output, "loop-checkbox-result"),
        Some("Profiling: disabled by checkbox")
    );
    assert!(
        !frame(&output, "loop-checkbox-result-box")
            .style
            .background
            .is_some_and(|color| color == SUCCESS_CONTAINER)
    );
    assert_eq!(
        frame_text(&output, "loop-radio-result"),
        Some("Runtime target: Remote worker")
    );
    assert!(has_class(
        frame(&output, "loop-radio-result-box"),
        "loop-runtime-remote"
    ));
    assert_eq!(
        frame_text(&output, "loop-dropdown-result"),
        Some("Source adapter: Python node")
    );
    assert!(has_class(
        frame(&output, "loop-dropdown-result-box"),
        "loop-source-python"
    ));
    assert_eq!(
        frame_text(&output, "loop-summary-result"),
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
            frame_text(&output, "loop-button-result"),
            Some(expected_text.as_str())
        );
        assert_eq!(
            frame(&output, "loop-button-result-box").value.as_deref(),
            Some(expected_value.as_str())
        );
    }
}

#[test]
fn interaction_update_loop_counts_repeated_hovered_button_clicks() {
    let mut harness = lab_harness("interaction");
    scroll_harness_stage(&mut harness, INTERACTION_LOOP_SCROLL_Y);

    for expected in 1..=3 {
        let rect = frame(
            &state_output_with_scroll(harness.state(), INTERACTION_LOOP_SCROLL_Y),
            "loop-action-button",
        )
        .rect;
        click_at_stays_hovered(&mut harness, center(rect));

        assert_eq!(harness.state().loop_action_count, expected);
    }
}

#[test]
fn draggable_drag_drop_grid_moves_items_between_cells() {
    let mut harness = lab_harness("draggable");

    assert_eq!(harness.state().drag_item_cells, [0, 2, 4]);
    let source_style = frame(
        &state_output_with_egui_text(harness.state(), &harness.ctx),
        "drag-item-0",
    )
    .style
    .clone();

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
    assert_eq!(overlay.text.as_ref().map(|text| text.semantic_text()), None);
    assert!(has_class(overlay, "drag-overlay"));
    assert!(
        !has_class(overlay, "drag-item-active"),
        "drag overlay should clone source styling without active visual classes"
    );
    assert_eq!(
        Some(overlay.rect.size),
        harness.state().drag_source_size,
        "drag overlay should resolve to the exact original source item size"
    );
    assert_eq!(overlay.style.background, source_style.background);
    assert_eq!(overlay.style.border, source_style.border);
    assert_eq!(overlay.style.border_width, source_style.border_width);
    assert_eq!(overlay.style.radius, source_style.radius);
    assert_eq!(overlay.style.padding, source_style.padding);
    assert_eq!(overlay.style.shadows.len(), source_style.shadows.len());
    assert!(
        overlay.style.shadows[0].offset.y > source_style.shadows[0].offset.y,
        "drag overlay should use the lifted hover shadow rather than the resting source shadow"
    );
    assert!(
        overlay.style.shadows[0].blur > source_style.shadows[0].blur,
        "drag overlay should keep the broader hover shadow after pickup"
    );
    assert!(
        has_class(frame(&output, "drag-overlay/drag-handle-0"), "drag-handle"),
        "drag overlay should preserve the source grab handle"
    );
    assert!(
        has_class(
            frame(&output, "drag-overlay/drag-handle-0-glyph"),
            "drag-handle-glyph"
        ),
        "drag overlay should preserve the source grab handle glyph"
    );
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

    let output = harness
        .state_mut()
        .lab_document_output_for_test(Size::new(TEST_WIDTH, TEST_HEIGHT));
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
fn draggable_drag_drop_reorders_with_nearest_item_gap() {
    let mut harness = lab_harness("draggable");
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

    let output = harness
        .state_mut()
        .lab_document_output_for_test(Size::new(TEST_WIDTH, TEST_HEIGHT));
    assert_eq!(
        frame(&output, "drag-item-1").style.margin,
        Insets::ZERO,
        "nearest item should snap out of the temporary insertion gap when the drop is committed"
    );
}

#[test]
fn draggable_drag_drop_suppresses_gap_at_original_position() {
    let mut harness = lab_harness("draggable");
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
    assert_eq!(
        frame(&output, "drag-item-1").style.shadows.len(),
        0,
        "hidden placeholders should reserve layout without painting a shadow"
    );
    assert_eq!(
        frame(&output, "drag-item-1").style.animate_paint,
        false,
        "hidden placeholders should snap old paint away without changing layout animation timing"
    );
    assert_eq!(
        frame(&output, "drag-item-1").style.animate_shadows,
        false,
        "hidden placeholders should explicitly snap shadows away"
    );
    assert!(
        !has_class(frame(&output, "drag-item-0"), "drag-gap-after"),
        "no second insertion gap should appear at the original position"
    );
}

#[test]
fn draggable_drag_drop_requires_handle_to_drag_parent() {
    let mut harness = lab_harness("draggable");

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
fn draggable_drag_drop_sets_handle_cursors() {
    let mut state = UiLabState::new(Some("draggable"));
    let output = state.lab_document_output_for_test(Size::new(TEST_WIDTH, TEST_HEIGHT));
    let start = center(frame(&output, "drag-handle-0").rect);
    let output = state.lab_document_output_with_input_for_test(
        Size::new(TEST_WIDTH, TEST_HEIGHT),
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(start.x, start.y),
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
        cursor_icon_for_output(&output),
        Some(egui::CursorIcon::PointingHand)
    );

    let mut output = state.lab_document_output_for_test(Size::new(TEST_WIDTH, TEST_HEIGHT));
    output.active_drag = Some(DocumentDrag {
        target: ElementId::new("drag-handle-0"),
        origin: Point::new(start.x, start.y),
        current: Point::new(start.x + 40.0, start.y),
        delta: Point::new(40.0, 0.0),
        pointer_offset: Point::ZERO,
    });
    assert_eq!(
        cursor_icon_for_output(&output),
        Some(egui::CursorIcon::PointingHand)
    );
}

#[test]
fn draggable_drag_handle_press_keeps_parent_lifted() {
    let mut harness = lab_harness("draggable");
    let start = center(state_rect_with_egui_text(
        harness.state(),
        &harness.ctx,
        "drag-handle-0",
    ));

    harness.hover_at(start);
    harness.drag_at(start);
    harness.run();
    let output = state_output_with_egui_text(harness.state(), &harness.ctx);
    let item = frame(&output, "drag-item-0");
    assert!(
        has_class(item, "drag-handle-pressed"),
        "pressing a drag handle should keep the parent card in the lifted style"
    );
    assert!(
        item.style.shadows[0].blur > 5.0,
        "pressed handle state should keep the hover shadow instead of falling back to rest"
    );
}

#[test]
fn draggable_drag_drop_styles_are_animated() {
    let mut harness = lab_harness("draggable");
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
        1,
        "drag overlay should inherit the source item's one soft shadow"
    );
    assert_eq!(
        frame(&output, "drag-overlay").style.animate_size,
        false,
        "drag overlay should snap to source size instead of easing in from idle width"
    );
    assert_eq!(
        frame(&output, "drag-scroll-list-card").style.shadows.len(),
        0,
        "resting drag list container should not cast a shadow"
    );
}

#[test]
fn draggable_drag_drop_auto_scrolls_opted_in_list_pane() {
    let mut harness = lab_harness("draggable");
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
fn draggable_scroll_list_keeps_scrollbar_visible_for_testing() {
    let output = lab_output("draggable");
    let list = frame(&output, "drag-scroll-list-0");
    assert!(
        list.style.scrollbar_visible,
        "draggable scroll-list scrollbar should stay visible while testing scroll-limit changes"
    );
    assert!(
        output
            .scroll_chrome
            .iter()
            .any(|chrome| { chrome.element_id.as_str() == "drag-scroll-list-0" && chrome.visible }),
        "draggable scroll-list should emit visible scrollbar chrome without hover"
    );
}

#[test]
fn draggable_scroll_list_preview_margin_updates_scrollbar_range() {
    let base_output = lab_output("draggable");
    let base_scroll = base_output
        .scroll_chrome
        .iter()
        .find(|chrome| {
            chrome.element_id.as_str() == "drag-scroll-list-0"
                && chrome.axis == ScrollAxis::Vertical
        })
        .expect("draggable scroll-list should emit vertical scroll chrome")
        .max_scroll;

    let mut state = UiLabState::new(Some("draggable"));
    state.scroll_list_drop_preview = Some(SortableDropPreview {
        zone: DropZoneId(0),
        nearest_item: Some(SortableItemId(3)),
        edge: des_widgets::DropEdge::After,
    });
    let preview_output = state_output(&state);
    let preview_scroll = preview_output
        .scroll_chrome
        .iter()
        .find(|chrome| {
            chrome.element_id.as_str() == "drag-scroll-list-0"
                && chrome.axis == ScrollAxis::Vertical
        })
        .expect("preview margin should keep vertical scroll chrome")
        .max_scroll;

    assert!(
        preview_scroll >= base_scroll + 30.0,
        "preview margin should expand scrollbar range before the drop; base {base_scroll}, preview {preview_scroll}"
    );
}

#[test]
fn draggable_drag_drop_uses_snapshot_path_for_drop_targets() {
    let output = lab_output("draggable");
    let cell = frame(&output, "drag-cell-5").rect;
    let point = Point::new(
        cell.origin.x + cell.size.width / 2.0,
        cell.origin.y + cell.size.height / 2.0,
    );
    assert_eq!(drop_cell_at(&output, point), Some(5));
}

#[test]
fn draggable_drag_drop_cells_expand_to_fit_stacked_items() {
    let mut state = UiLabState::new(Some("draggable"));
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
fn draggable_workbench_wraps_panels_when_stage_narrows() {
    let output = lab_output_with_size("draggable", Size::new(1080.0, 720.0));
    let workbench = frame(&output, "drag-workbench");
    let list_card = frame(&output, "drag-scroll-list-card");
    let drag_grid = frame(&output, "drag-grid");

    assert!(
        drag_grid.rect.origin.y > list_card.rect.bottom(),
        "drag grid should wrap below the scroll list card when the stage cannot fit both panels"
    );
    assert_eq!(drag_grid.rect.origin.x, list_card.rect.origin.x);
    assert_close(list_card.rect.size.width, workbench.rect.size.width);
    assert_close(drag_grid.rect.size.width, workbench.rect.size.width);
    for cell in 0..6 {
        let cell = frame(&output, &format!("drag-cell-{cell}"));
        assert!(
            drag_grid.rect.bottom() >= cell.rect.bottom(),
            "stacked drag grid should expand around all rows"
        );
    }
}

#[test]
fn draggable_workbench_keeps_panels_side_by_side_at_default_width() {
    let harness = lab_harness("draggable");
    let output = state_output_with_egui_text(harness.state(), &harness.ctx);
    let list_card = frame(&output, "drag-scroll-list-card");
    let drag_grid = frame(&output, "drag-grid");

    assert_eq!(drag_grid.rect.origin.y, list_card.rect.origin.y);
    assert!(
        drag_grid.rect.origin.x > list_card.rect.right(),
        "drag grid should remain beside the scroll list card at the default lab width"
    );
    assert_close(list_card.rect.size.width, drag_grid.rect.size.width);
}

#[test]
fn draggable_grid_cells_fill_rows_and_remain_inside_grid() {
    let output = lab_output_with_size("draggable", Size::new(1500.0, 780.0));
    let grid = frame(&output, "drag-grid");
    let left = frame(&output, "drag-cell-0");
    let right = frame(&output, "drag-cell-1");

    assert_eq!(left.rect.origin.y, right.rect.origin.y);
    assert!(
        right.rect.right() > grid.rect.right() - 40.0,
        "right drag cell should fluidly fill the row"
    );
    for cell in 0..6 {
        let cell = frame(&output, &format!("drag-cell-{cell}"));
        assert!(
            grid.rect.bottom() >= cell.rect.bottom(),
            "drag grid should expand around all rows"
        );
    }
}

#[test]
fn draggable_view_reuses_retained_document_on_warm_update() {
    let mut state = UiLabState::new(Some("draggable"));
    let first = state.lab_document_output_for_test(Size::new(TEST_WIDTH, TEST_HEIGHT));
    let warm = state.lab_document_output_for_test(Size::new(TEST_WIDTH, TEST_HEIGHT));

    assert!(first.metrics.style_nodes_visited > 0);
    assert_eq!(warm.metrics.style_nodes_visited, 0);
    assert!(warm.metrics.reused_input_layout);
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
    assert!(output.metrics.style_nodes_visited > 0);

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
fn scrolling_view_reuses_retained_document_on_warm_update() {
    let mut state = UiLabState::new(Some("scrolling"));
    let first = state.lab_document_output_for_test(Size::new(TEST_WIDTH, TEST_HEIGHT));
    let warm = state.lab_document_output_for_test(Size::new(TEST_WIDTH, TEST_HEIGHT));

    assert!(first.metrics.style_nodes_visited > 0);
    assert_eq!(warm.metrics.style_nodes_visited, 0);
    assert!(warm.metrics.reused_input_layout);
}

#[test]
fn every_lab_view_reuses_retained_document_on_warm_update() {
    for view in [
        "layout",
        "interaction",
        "draggable",
        "styling",
        "animation",
        "scrolling",
        "table",
        "text",
        "nesting",
        "graph",
    ] {
        let mut state = UiLabState::new(Some(view));
        let first = state.lab_document_output_for_test(Size::new(TEST_WIDTH, TEST_HEIGHT));
        let warm = state.lab_document_output_for_test(Size::new(TEST_WIDTH, TEST_HEIGHT));

        assert!(
            first.metrics.style_nodes_visited > 0,
            "{view} should resolve document styles on the cold update"
        );
        assert_eq!(
            warm.metrics.style_nodes_visited, 0,
            "{view} should skip document style traversal on the warm update"
        );
        assert!(
            warm.metrics.reused_input_layout,
            "{view} should reuse input layout on the warm update"
        );
    }
}

#[test]
fn table_view_renders_document_table_elements_and_shared_tracks() {
    let output = lab_output("table");
    let table = frame(&output, "customer-preview-table");
    let header_customer = frame(&output, "customer-preview-header-customer");
    let row_customer = frame(&output, "customer-preview-row-0-customer");
    let header_revenue = frame(&output, "customer-preview-header-revenue");
    let row_revenue = frame(&output, "customer-preview-row-0-revenue");

    assert_eq!(table.element, Element::Table);
    assert_eq!(header_customer.element, Element::Td);
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
    let wrapped_card = frame(&output, "text-wrap");
    let wrapped_title = frame(&output, "text-wrap-title");
    let wrapped_rule = frame(&output, "text-wrap-rule");
    let rich_title = frame(&output, "text-rich-title");
    let break_word = frame(&output, "text-break-word-body");
    let truncated = frame(&output, "text-truncate-body");
    let max_lines = frame(&output, "text-max-lines-body");
    let pre = frame(&output, "text-pre-body");
    let break_spaces = frame(&output, "text-break-spaces-body");
    let rtl = frame(&output, "text-rtl-start-body");
    let intro_copy = frame(&output, "text-copy");

    assert_eq!(intro_copy.style.width, Length::Fill);
    assert_eq!(intro_copy.style.height, Length::Auto);
    assert_eq!(wrapped_title.style.width, Length::Fill);
    assert_eq!(wrapped_title.style.height, Length::Auto);
    assert_eq!(rich_title.style.width, Length::Fill);
    assert_eq!(rich_title.style.height, Length::Auto);
    assert_eq!(wrapped_rule.style.width, Length::Fill);
    assert_eq!(wrapped_rule.style.height, Length::Auto);
    assert_eq!(wrapped.style.width, Length::Fill);
    assert_eq!(wrapped.style.height, Length::Auto);
    assert!(
        wrapped.rect.size.width > 240.0,
        "standalone specimen boxes should fill the card row instead of keeping the old narrow fixed width"
    );
    assert!(
        wrapped.rect.size.width < wrapped_card.rect.size.width,
        "filled specimen boxes should still respect card padding and border"
    );

    assert_eq!(wrapped.style.text_layout.text_wrap_mode, TextWrapMode::Wrap);
    assert!(
        wrapped.text_layout.as_ref().unwrap().line_count > 1,
        "text wrap specimen should be measured as multiple lines"
    );
    assert_eq!(
        break_word.style.text_layout.overflow_wrap,
        OverflowWrap::BreakWord
    );
    assert!(
        break_word.text_layout.as_ref().unwrap().line_count > 1,
        "overflow-wrap: break-word should break an otherwise unbreakable token"
    );
    assert_eq!(truncated.style.text_layout.max_lines, Some(1));
    assert_eq!(
        truncated.style.text_layout.text_overflow,
        TextOverflow::Ellipsis
    );
    assert!(truncated.text_layout.as_ref().unwrap().elided);
    assert_eq!(max_lines.style.text_layout.max_lines, Some(2));
    assert!(max_lines.text_layout.as_ref().unwrap().line_count <= 2);
    assert_eq!(
        pre.style.text_layout.white_space_collapse,
        des_document::WhiteSpaceCollapse::Preserve
    );
    assert_eq!(pre.style.text_layout.tab_size, 8);
    assert_eq!(
        pre.text.as_ref().unwrap().semantic_text(),
        "columns:\talpha\tbeta\nspaces:   one   two   three"
    );
    assert_eq!(
        pre.text_layout.as_ref().unwrap().line_count,
        2,
        "pre specimen should preserve the explicit newline as a measured line break"
    );
    assert_eq!(
        break_spaces.style.text_layout.white_space_collapse,
        des_document::WhiteSpaceCollapse::BreakSpaces
    );
    assert_eq!(
        break_spaces.style.text_layout.overflow_wrap,
        OverflowWrap::Anywhere
    );
    assert!(
        break_spaces
            .normalized_text
            .as_ref()
            .unwrap()
            .layout_text()
            .contains("spaces   \n"),
        "break-spaces specimen should preserve trailing spaces before explicit line breaks"
    );
    assert_eq!(rtl.style.direction, des_document::Direction::Rtl);
    assert_eq!(
        rtl.style.text_layout.text_align,
        des_document::TextAlign::Start
    );
    assert!(
        rtl.text_layout.as_ref().unwrap().lines[0].x_offset > 0.0,
        "RTL start specimen should be physically aligned to the right edge"
    );
    assert!(
        wrapped.rect.size.height > wrapped.text_layout.as_ref().unwrap().size.height,
        "text specimens should include padding in the border-box height"
    );

    let legacy_pane = frame(&output, "text-legacy-100-pane");
    let diagnostics = frame_text(&output, "text-cosmic-diagnostics").unwrap();
    let rich_sample = frame(&output, "text-rich-100-sample");
    let rich_weight = frame(&output, "text-rich-weight");
    let rich_shape = frame(&output, "text-rich-shape");
    let rich_spacing = frame(&output, "text-rich-spacing");
    let rich_decoration = frame(&output, "text-rich-decoration");
    let rich_family = frame(&output, "text-rich-family");
    let rich_baseline = frame(&output, "text-rich-baseline");
    assert_eq!(rich_sample.style.font_size, 100.0);
    assert_eq!(
        rich_sample.text.as_ref().unwrap().semantic_text(),
        "Ag 100px"
    );
    assert_eq!(
        frame_text(&output, "text-rich-shape"),
        Some("normal italic oblique")
    );
    assert_eq!(
        frame_text(&output, "text-rich-family"),
        Some("fallback Aptos -> Inter generic sans bundled mono")
    );
    let weight_runs = rich_weight
        .normalized_text
        .as_ref()
        .expect("rich weight specimen should retain normalized text")
        .runs();
    let shape_runs = rich_shape
        .normalized_text
        .as_ref()
        .expect("rich shape specimen should retain normalized text")
        .runs();
    assert_eq!(weight_runs[0].style.font_weight, Some(FontWeight::new(300)));
    assert_eq!(weight_runs[1].style.font_weight, Some(FontWeight::NORMAL));
    assert_eq!(weight_runs[2].style.font_weight, Some(FontWeight::new(600)));
    assert_eq!(weight_runs[3].style.font_weight, Some(FontWeight::BOLD));
    assert_eq!(shape_runs[0].style.font_style, None);
    assert_eq!(shape_runs[1].style.font_style, Some(FontStyle::Italic));
    assert_eq!(shape_runs[2].style.font_style, Some(FontStyle::Oblique));
    let family_runs = rich_family
        .normalized_text
        .as_ref()
        .expect("rich family specimen should retain normalized text")
        .runs();
    let decoration_runs = rich_decoration
        .normalized_text
        .as_ref()
        .expect("rich decoration specimen should retain normalized text")
        .runs();
    let spacing_runs = rich_spacing
        .normalized_text
        .as_ref()
        .expect("rich spacing specimen should retain normalized text")
        .runs();
    let baseline_runs = rich_baseline
        .normalized_text
        .as_ref()
        .expect("rich baseline specimen should retain normalized text")
        .runs();
    assert_eq!(spacing_runs[0].style.letter_spacing, Some(-0.75));
    assert_eq!(spacing_runs[1].style.letter_spacing, Some(0.0));
    assert_eq!(spacing_runs[2].style.letter_spacing, Some(2.0));
    assert_eq!(
        decoration_runs[0].style.text_decoration,
        Some(
            TextDecoration::UNDERLINE
                .color(Color::rgb(103, 80, 164))
                .thickness(1.0)
        )
    );
    assert!(decoration_runs[1].style.text_decoration.is_none());
    assert_eq!(
        decoration_runs[2].style.text_decoration,
        Some(
            TextDecoration::LINE_THROUGH
                .color(Color::rgb(122, 71, 0))
                .thickness(1.0)
        )
    );
    assert!(decoration_runs[3].style.text_decoration.is_none());
    assert_eq!(
        decoration_runs[4].style.text_decoration,
        Some(
            TextDecoration::OVERLINE
                .color(Color::rgb(0, 95, 102))
                .thickness(1.0)
        )
    );
    assert!(decoration_runs[5].style.text_decoration.is_none());
    assert_eq!(
        decoration_runs[6].style.text_decoration,
        Some(
            TextDecoration::lines(true, true, true)
                .color(Color::rgb(86, 69, 0))
                .thickness(1.0)
        )
    );
    assert!(decoration_runs[7].style.text_decoration.is_none());
    assert_eq!(
        family_runs[0].style.font_family.as_deref(),
        Some("Aptos, Inter, sans-serif")
    );
    assert_eq!(
        family_runs[1].style.font_family.as_deref(),
        Some("sans-serif")
    );
    assert_eq!(
        family_runs[2].style.font_family.as_deref(),
        Some("monospace")
    );
    assert_eq!(
        baseline_runs[1].style.vertical_align,
        Some(des_document::TextVerticalAlign::Super)
    );
    assert_eq!(
        baseline_runs[3].style.vertical_align,
        Some(des_document::TextVerticalAlign::Sub)
    );
    assert!(diagnostics.contains("cosmic-text advanced shaping + Swash raster"));
    assert!(diagnostics.contains("JetBrains Mono Variable"));
    assert!(diagnostics.contains("bundled-only default"));
    assert!(
        legacy_pane.rect.size.width > 300.0,
        "legacy simple rendering slot should sit beside the rich text sample"
    );
}

#[test]
fn text_view_uses_glyph_atlas_on_warm_paint() {
    let mut harness = lab_harness("text");
    let first = render_harness(&mut harness);
    let first_stats = harness.state().last_perf.text_paint;
    let warm = render_harness(&mut harness);
    let warm_stats = harness.state().last_perf.text_paint;

    assert!(
        image_stats(&first).non_transparent_pixels > 20_000,
        "text view should render visible specimen output"
    );
    assert_exact_image_match(&first, &warm);
    assert!(
        first_stats.cached_glyphs > 0,
        "text paint should populate the glyph atlas"
    );
    assert!(
        first_stats.paint_text_requests > 0,
        "text view should report cosmic glyph-run paint requests"
    );
    assert!(
        first_stats.atlas_pages > 0,
        "text paint should create at least one glyph atlas page"
    );
    assert!(
        warm_stats.measure_requests == 0,
        "warm text paint should reuse retained document layout without text measurement requests"
    );
    assert_eq!(
        warm_stats.rasterizations, 0,
        "warm text paint should reuse the glyph atlas without rasterizing glyphs"
    );
    assert_eq!(
        warm_stats.uploaded_pixels, 0,
        "warm text paint should not upload new glyph atlas pixels"
    );
    assert_eq!(
        warm_stats.layout_cache_misses, 0,
        "warm text paint should reuse retained cosmic text buffers without reshaping misses"
    );
    assert_eq!(
        warm_stats.paint_run_cache_misses, 0,
        "warm text paint should reuse retained visible glyph runs"
    );
    assert!(
        warm_stats.paint_run_cache_hits > 0,
        "warm text paint should hit retained visible glyph runs"
    );
    assert!(
        warm_stats.layout_cache_entries > 0,
        "warm text paint should retain cosmic text buffers even when paint runs avoid touching them"
    );
    assert!(
        first_stats.glyph_mesh_cache_entries > 0,
        "cold text paint should populate retained glyph meshes"
    );
    assert!(
        warm_stats.glyph_mesh_cache_hits > 0,
        "warm text paint should hit retained glyph meshes"
    );
    assert_eq!(
        warm_stats.glyph_mesh_cache_misses, 0,
        "warm text paint should not rebuild retained glyph meshes"
    );
    assert_eq!(
        warm_stats.glyph_cache_hits, 0,
        "warm text paint should skip per-glyph atlas lookups when retained meshes are available"
    );
    assert!(
        warm_stats.glyph_meshes > 0 && warm_stats.glyph_meshes < warm_stats.glyphs_painted,
        "warm text paint should batch glyphs into fewer egui meshes than glyph image quads; stats={warm_stats:?}"
    );
}

#[test]
fn text_view_uses_glyph_atlas_on_warm_scrolled_paint() {
    let mut harness = lab_harness("text");
    let initial = render_harness(&mut harness);
    harness
        .state_mut()
        .document_engine
        .element_state_mut("stage")
        .expect("text view has stage scroll state")
        .scroll_y = 650.0;
    let scrolled = render_harness(&mut harness);
    let populated_stats = harness.state().last_perf.text_paint;
    let warm_scrolled = render_harness(&mut harness);
    let warm_stats = harness.state().last_perf.text_paint;

    assert!(
        image_stats(&initial).non_transparent_pixels > 20_000,
        "initial text view should render visible specimen output"
    );
    assert!(
        image_stats(&scrolled).non_transparent_pixels > 20_000,
        "scrolled text view should render visible specimen output"
    );
    assert!(
        compare_images(&initial, &scrolled).differing_pixels > 10_000,
        "test should actually exercise a different scrolled text viewport"
    );
    assert_exact_image_match(&scrolled, &warm_scrolled);
    assert!(
        populated_stats.cached_glyphs > 0,
        "scrolled text paint should populate or reuse atlas glyphs"
    );
    assert!(
        populated_stats.atlas_pages > 0,
        "scrolled text paint should create at least one glyph atlas page"
    );
    assert!(
        populated_stats.layout_cache_entries > 0,
        "scrolled text paint should retain cosmic text buffers"
    );
    assert_eq!(
        warm_stats.measure_requests, 0,
        "warm scrolled text paint should reuse retained document layout without text measurement requests"
    );
    assert_eq!(
        warm_stats.rasterizations, 0,
        "warm scrolled text paint should reuse the glyph atlas without rasterizing glyphs"
    );
    assert_eq!(
        warm_stats.uploaded_pixels, 0,
        "warm scrolled text paint should not upload new glyph atlas pixels"
    );
    assert_eq!(
        warm_stats.layout_cache_misses, 0,
        "warm scrolled text paint should reuse retained cosmic text buffers without reshaping misses"
    );
    assert_eq!(
        warm_stats.paint_run_cache_misses, 0,
        "warm scrolled text paint should reuse retained visible glyph runs"
    );
    assert!(
        warm_stats.paint_run_cache_hits > 0,
        "warm scrolled text paint should hit retained visible glyph runs"
    );
    assert!(
        warm_stats.layout_cache_entries > 0,
        "warm scrolled text paint should retain cosmic text buffers even when paint runs avoid touching them"
    );
    assert!(
        populated_stats.glyph_mesh_cache_entries > 0,
        "scrolled text paint should populate retained glyph meshes"
    );
    assert!(
        warm_stats.glyph_mesh_cache_hits > 0,
        "warm scrolled text paint should hit retained glyph meshes"
    );
    assert_eq!(
        warm_stats.glyph_mesh_cache_misses, 0,
        "warm scrolled text paint should not rebuild retained glyph meshes"
    );
    assert_eq!(
        warm_stats.glyph_cache_hits, 0,
        "warm scrolled text paint should skip per-glyph atlas lookups when retained meshes are available"
    );
    assert!(
        warm_stats.glyph_meshes > 0 && warm_stats.glyph_meshes < warm_stats.glyphs_painted,
        "warm scrolled text paint should batch glyphs into fewer egui meshes than glyph image quads; stats={warm_stats:?}"
    );
}

#[test]
fn text_view_reuses_text_paint_runs_during_nearby_scroll() {
    let mut harness = lab_harness("text");
    let initial = render_harness(&mut harness);
    harness
        .state_mut()
        .document_engine
        .element_state_mut("stage")
        .expect("text view has stage scroll state")
        .scroll_y = 32.0;
    let nearby = render_harness(&mut harness);
    let nearby_stats = harness.state().last_perf.text_paint;

    assert!(
        image_stats(&initial).non_transparent_pixels > 20_000,
        "initial text view should render visible specimen output"
    );
    assert!(
        image_stats(&nearby).non_transparent_pixels > 20_000,
        "nearby scrolled text view should render visible specimen output"
    );
    assert!(
        compare_images(&initial, &nearby).differing_pixels > 1_000,
        "test should actually exercise a changed scroll viewport"
    );
    assert!(
        nearby_stats.paint_run_cache_hits > 0,
        "nearby scroll should reuse retained expanded cosmic text paint runs"
    );
}

#[cfg(not(debug_assertions))]
#[test]
fn text_view_warm_repaint_release_measurement() {
    const WARMUP_FRAMES: usize = 4;
    const MEASURED_FRAMES: usize = 24;

    let mut harness = lab_harness("text");
    let baseline = render_harness(&mut harness);
    for _ in 0..WARMUP_FRAMES {
        render_harness(&mut harness);
    }

    let mut paint_total = Duration::ZERO;
    let mut glyph_run_total = Duration::ZERO;
    let mut glyph_paint_total = Duration::ZERO;
    let mut max_paint = Duration::ZERO;
    let mut mesh_hits = 0usize;
    let mut mesh_misses = 0usize;
    let mut rasterizations = 0usize;
    let mut uploaded_pixels = 0u64;

    for _ in 0..MEASURED_FRAMES {
        let frame = render_harness(&mut harness);
        assert_exact_image_match(&baseline, &frame);
        let perf = harness.state().last_perf;
        paint_total += perf.paint_time;
        glyph_run_total += perf.text_paint.glyph_run_time;
        glyph_paint_total += perf.text_paint.glyph_paint_time;
        max_paint = max_paint.max(perf.paint_time);
        mesh_hits += perf.text_paint.glyph_mesh_cache_hits;
        mesh_misses += perf.text_paint.glyph_mesh_cache_misses;
        rasterizations += perf.text_paint.rasterizations;
        uploaded_pixels += perf.text_paint.uploaded_pixels;
    }

    let frame_count = MEASURED_FRAMES as u32;
    eprintln!(
        "text warm repaint release measurement: frames={MEASURED_FRAMES} avg_paint={} max_paint={} avg_glyph_run={} avg_glyph_paint={} mesh_hits={mesh_hits} mesh_misses={mesh_misses} rasterizations={rasterizations} uploaded_pixels={uploaded_pixels}",
        format_duration_for_test(paint_total / frame_count),
        format_duration_for_test(max_paint),
        format_duration_for_test(glyph_run_total / frame_count),
        format_duration_for_test(glyph_paint_total / frame_count),
    );

    assert!(
        mesh_hits > 0,
        "warm release renders should hit retained text meshes"
    );
    assert_eq!(
        mesh_misses, 0,
        "warm release renders should not rebuild retained text meshes"
    );
    assert_eq!(
        rasterizations, 0,
        "warm release renders should not rasterize glyphs"
    );
    assert_eq!(
        uploaded_pixels, 0,
        "warm release renders should not upload glyph pixels"
    );
}

#[cfg(not(debug_assertions))]
#[test]
fn text_view_nav_hover_release_measurement() {
    const WARMUP_ROUNDS: usize = 2;
    const MEASURED_ROUNDS: usize = 8;

    let mut harness = lab_harness("text");
    render_harness(&mut harness);
    let hover_points = [
        center(state_rect(harness.state(), "view-layout")),
        center(state_rect(harness.state(), "view-text")),
        center(state_rect(harness.state(), "view-graph")),
    ];

    for _ in 0..WARMUP_ROUNDS {
        for point in hover_points {
            harness.hover_at(point);
            render_harness(&mut harness);
        }
    }

    let mut paint_total = Duration::ZERO;
    let mut glyph_run_total = Duration::ZERO;
    let mut glyph_paint_total = Duration::ZERO;
    let mut max_paint = Duration::ZERO;
    let mut mesh_hits = 0usize;
    let mut mesh_misses = 0usize;
    let mut rasterizations = 0usize;
    let mut uploaded_pixels = 0u64;
    let mut frames = 0usize;

    for _ in 0..MEASURED_ROUNDS {
        for point in hover_points {
            harness.hover_at(point);
            let frame = render_harness(&mut harness);
            assert!(
                image_stats(&frame).non_transparent_pixels > 20_000,
                "hover repaint should keep rendering visible lab output"
            );
            let perf = harness.state().last_perf;
            paint_total += perf.paint_time;
            glyph_run_total += perf.text_paint.glyph_run_time;
            glyph_paint_total += perf.text_paint.glyph_paint_time;
            max_paint = max_paint.max(perf.paint_time);
            mesh_hits += perf.text_paint.glyph_mesh_cache_hits;
            mesh_misses += perf.text_paint.glyph_mesh_cache_misses;
            rasterizations += perf.text_paint.rasterizations;
            uploaded_pixels += perf.text_paint.uploaded_pixels;
            frames += 1;
        }
    }

    let frame_count = frames as u32;
    eprintln!(
        "text nav hover release measurement: frames={frames} avg_paint={} max_paint={} avg_glyph_run={} avg_glyph_paint={} mesh_hits={mesh_hits} mesh_misses={mesh_misses} rasterizations={rasterizations} uploaded_pixels={uploaded_pixels}",
        format_duration_for_test(paint_total / frame_count),
        format_duration_for_test(max_paint),
        format_duration_for_test(glyph_run_total / frame_count),
        format_duration_for_test(glyph_paint_total / frame_count),
    );

    assert!(
        mesh_hits > 0,
        "hover release renders should hit retained text meshes"
    );
    assert_eq!(
        mesh_misses, 0,
        "warmed hover release renders should not rebuild retained text meshes"
    );
    assert_eq!(
        rasterizations, 0,
        "hover release renders should not rasterize glyphs"
    );
    assert_eq!(
        uploaded_pixels, 0,
        "hover release renders should not upload glyph pixels"
    );
}

#[test]
fn blank_topbar_pointer_moves_reuse_last_document_output() {
    let mut harness = lab_harness("text");
    render_harness(&mut harness);
    let topbar = state_rect(harness.state(), "topbar");
    let first = egui::pos2(
        topbar.origin.x + 320.0,
        topbar.origin.y + topbar.size.height - 7.0,
    );
    let second = egui::pos2(topbar.origin.x + 420.0, first.y);

    harness.hover_at(first);
    render_harness(&mut harness);
    harness.hover_at(second);
    let second_frame = render_harness(&mut harness);
    let second_perf = harness.state().last_perf;

    assert!(image_stats(&second_frame).non_transparent_pixels > 20_000);
    assert_eq!(
        second_perf.engine_time,
        Duration::ZERO,
        "same inert topbar hit should reuse retained document output"
    );
    assert!(
        second_perf.paint_time > Duration::ZERO,
        "reused document output should still be painted"
    );
}

#[cfg(not(debug_assertions))]
#[test]
fn whole_lab_interaction_release_measurement() {
    let mut text = lab_harness("text");
    render_harness(&mut text);
    for _ in 0..4 {
        render_harness(&mut text);
    }

    let hover_points = [
        center(state_rect(text.state(), "view-layout")),
        center(state_rect(text.state(), "view-text")),
        center(state_rect(text.state(), "view-graph")),
    ];
    let mut hover_totals = ReleaseFrameTotals::default();
    for _ in 0..8 {
        for point in hover_points {
            text.hover_at(point);
            let frame = measure_release_frame(&mut text, &mut hover_totals);
            assert!(image_stats(&frame).non_transparent_pixels > 20_000);
        }
    }
    hover_totals.report("whole text nav hover");

    let topbar = state_rect(text.state(), "topbar");
    let mut topbar_totals = ReleaseFrameTotals::default();
    for index in 0..36 {
        let x = topbar.origin.x + 320.0 + (index % 12) as f32 * 18.0;
        let y = topbar.origin.y + topbar.size.height - 7.0;
        text.hover_at(egui::pos2(x, y));
        let frame = measure_release_frame(&mut text, &mut topbar_totals);
        assert!(image_stats(&frame).non_transparent_pixels > 20_000);
    }
    topbar_totals.report("whole text blank topbar pointer move");

    let mut text_scroll = lab_harness("text");
    render_harness(&mut text_scroll);
    let text_stage = center(state_rect(text_scroll.state(), "stage"));
    let mut text_scroll_totals = ReleaseFrameTotals::default();
    for delta_y in [-96.0, -96.0, -96.0, 96.0, 96.0, 96.0]
        .into_iter()
        .cycle()
        .take(24)
    {
        wheel_at(&mut text_scroll, text_stage, egui::vec2(0.0, delta_y));
        let frame = measure_release_frame(&mut text_scroll, &mut text_scroll_totals);
        assert!(image_stats(&frame).non_transparent_pixels > 20_000);
    }
    text_scroll_totals.report("whole text stage scroll");

    let mut scrolling = lab_harness("scrolling");
    render_harness(&mut scrolling);
    let scrolling_stage = center(state_rect(scrolling.state(), "stage"));
    let mut nested_scroll_totals = ReleaseFrameTotals::default();
    for delta_y in [-120.0, -120.0, -120.0, 120.0, 120.0, 120.0]
        .into_iter()
        .cycle()
        .take(24)
    {
        wheel_at(&mut scrolling, scrolling_stage, egui::vec2(0.0, delta_y));
        let frame = measure_release_frame(&mut scrolling, &mut nested_scroll_totals);
        assert!(image_stats(&frame).non_transparent_pixels > 20_000);
    }
    nested_scroll_totals.report("whole scrolling stage scroll");

    let mut selection = lab_harness("text");
    render_harness(&mut selection);
    let rect = state_rect(selection.state(), "text-wrap-body");
    let start = egui::pos2(rect.origin.x + 12.0, rect.origin.y + 12.0);
    let end = egui::pos2(rect.origin.x + 145.0, rect.origin.y + 34.0);
    let mut selection_totals = ReleaseFrameTotals::default();
    for _ in 0..12 {
        selection.hover_at(start);
        selection.drag_at(start);
        measure_release_frame(&mut selection, &mut selection_totals);
        selection.hover_at(end);
        let frame = measure_release_frame(&mut selection, &mut selection_totals);
        assert!(image_stats(&frame).non_transparent_pixels > 20_000);
        selection.drop_at(end);
        measure_release_frame(&mut selection, &mut selection_totals);
    }
    selection_totals.report("whole text selection drag");
}

#[cfg(not(debug_assertions))]
#[derive(Default)]
struct ReleaseFrameTotals {
    frames: usize,
    egui_steps: u64,
    total_frame: Duration,
    egui_run: Duration,
    pixel_render: Duration,
    max_frame: Duration,
    shape_count: usize,
    stylesheet: Duration,
    document: Duration,
    engine: Duration,
    paint: Duration,
    text_glyph_run: Duration,
    text_glyph_paint: Duration,
    mesh_hits: usize,
    mesh_misses: usize,
    rasterizations: usize,
    uploaded_pixels: u64,
    input_changed_frames: usize,
    relayout_frames: usize,
    animation_frames: usize,
}

#[cfg(not(debug_assertions))]
impl ReleaseFrameTotals {
    fn record(
        &mut self,
        egui_steps: u64,
        egui_run_time: Duration,
        pixel_render_time: Duration,
        shape_count: usize,
        perf: UiLabPerf,
    ) {
        self.frames += 1;
        self.egui_steps += egui_steps;
        let frame_time = egui_run_time + pixel_render_time;
        self.total_frame += frame_time;
        self.egui_run += egui_run_time;
        self.pixel_render += pixel_render_time;
        self.max_frame = self.max_frame.max(frame_time);
        self.shape_count += shape_count;
        self.stylesheet += perf.stylesheet_time;
        self.document += perf.document_time;
        self.engine += perf.engine_time;
        self.paint += perf.paint_time;
        self.text_glyph_run += perf.text_paint.glyph_run_time;
        self.text_glyph_paint += perf.text_paint.glyph_paint_time;
        self.mesh_hits += perf.text_paint.glyph_mesh_cache_hits;
        self.mesh_misses += perf.text_paint.glyph_mesh_cache_misses;
        self.rasterizations += perf.text_paint.rasterizations;
        self.uploaded_pixels += perf.text_paint.uploaded_pixels;
        self.input_changed_frames += usize::from(perf.metrics.input_changed_state);
        self.relayout_frames += usize::from(!perf.metrics.reused_input_layout);
        self.animation_frames += usize::from(perf.metrics.animation_changed_paint);
    }

    fn report(&self, label: &str) {
        let frames = self.frames as u32;
        let known = self.document + self.engine + self.paint;
        let other = self.total_frame.saturating_sub(known);
        eprintln!(
            "{label}: frames={} egui_steps={} avg_steps_per_frame={:.1} avg_total={} max_total={} avg_egui_run={} avg_egui_step={} avg_pixel_render={} avg_shapes={} avg_stylesheet={} avg_document={} avg_engine={} avg_doc_paint={} avg_outer_other={} avg_text_glyph_run={} avg_text_glyph_paint={} mesh_hits={} mesh_misses={} rasterizations={} uploaded_pixels={} input_changed_frames={} relayout_frames={} animation_frames={}",
            self.frames,
            self.egui_steps,
            self.egui_steps as f64 / self.frames as f64,
            format_duration_for_test(self.total_frame / frames),
            format_duration_for_test(self.max_frame),
            format_duration_for_test(self.egui_run / frames),
            format_duration_for_test(self.egui_run / self.egui_steps.max(1) as u32),
            format_duration_for_test(self.pixel_render / frames),
            self.shape_count / self.frames,
            format_duration_for_test(self.stylesheet / frames),
            format_duration_for_test(self.document / frames),
            format_duration_for_test(self.engine / frames),
            format_duration_for_test(self.paint / frames),
            format_duration_for_test(other / frames),
            format_duration_for_test(self.text_glyph_run / frames),
            format_duration_for_test(self.text_glyph_paint / frames),
            self.mesh_hits,
            self.mesh_misses,
            self.rasterizations,
            self.uploaded_pixels,
            self.input_changed_frames,
            self.relayout_frames,
            self.animation_frames,
        );
    }
}

#[cfg(not(debug_assertions))]
fn measure_release_frame(
    harness: &mut Harness<'_, UiLabState>,
    totals: &mut ReleaseFrameTotals,
) -> image::RgbaImage {
    let run_start = Instant::now();
    let egui_steps = harness.run();
    let egui_run_time = run_start.elapsed();
    let shape_count = harness.output().shapes.len();
    let render_start = Instant::now();
    let frame = harness
        .render()
        .expect("render egui output through kittest");
    let pixel_render_time = render_start.elapsed();
    totals.record(
        egui_steps,
        egui_run_time,
        pixel_render_time,
        shape_count,
        harness.state().last_perf,
    );
    frame
}

#[cfg(not(debug_assertions))]
fn format_duration_for_test(duration: Duration) -> String {
    let micros = duration.as_micros();
    if micros >= 1_000 {
        format!("{:.2}ms", micros as f64 / 1_000.0)
    } else {
        format!("{micros}us")
    }
}

#[test]
fn debug_overlay_reports_split_text_paint_timings() {
    let mut document = Document::build(Size::new(TEST_WIDTH, TEST_HEIGHT), |ui| {
        render_debug_overlay_layer(
            ui,
            UiLabPerf {
                text_paint: TextPaintStats {
                    glyph_image_time: std::time::Duration::from_micros(1_250),
                    glyph_upload_time: std::time::Duration::from_micros(2_500),
                    glyph_paint_time: std::time::Duration::from_micros(3_750),
                    glyph_meshes: 9,
                    glyph_mesh_cache_hits: 7,
                    glyph_mesh_cache_misses: 2,
                    glyph_mesh_cache_entries: 5,
                    ..TextPaintStats::default()
                },
                ..UiLabPerf::default()
            },
        );
    });
    let output = DocumentEngine::default().update(&mut document, &stylesheet());

    assert_eq!(
        frame_text(&output, "debug-text-glyph-image-time-label"),
        Some("text glyph image")
    );
    assert_eq!(
        frame_text(&output, "debug-text-glyph-image-time-value"),
        Some("1.25 ms")
    );
    assert_eq!(
        frame_text(&output, "debug-text-upload-time-label"),
        Some("text upload")
    );
    assert_eq!(
        frame_text(&output, "debug-text-upload-time-value"),
        Some("2.50 ms")
    );
    assert_eq!(
        frame_text(&output, "debug-text-glyph-paint-time-label"),
        Some("text glyph paint")
    );
    assert_eq!(
        frame_text(&output, "debug-text-glyph-paint-time-value"),
        Some("3.75 ms")
    );
    assert_eq!(
        frame_text(&output, "debug-text-glyph-meshes-label"),
        Some("text glyph meshes")
    );
    assert_eq!(
        frame_text(&output, "debug-text-glyph-meshes-value"),
        Some("9")
    );
    assert_eq!(
        frame_text(&output, "debug-text-mesh-cache-label"),
        Some("text mesh cache")
    );
    assert_eq!(
        frame_text(&output, "debug-text-mesh-cache-value"),
        Some("7 hit / 2 miss / 5 cached")
    );
}

#[test]
fn text_view_allows_pointer_selection_on_selectable_text() {
    let mut harness = lab_harness("text");
    let rect = state_rect(harness.state(), "text-wrap-body");
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
fn text_view_paints_pointer_selection_on_selectable_text() {
    let mut harness = lab_harness("text");
    let before = render_harness(&mut harness);
    let rect = state_rect(harness.state(), "text-wrap-body");
    let start = egui::pos2(rect.origin.x + 12.0, rect.origin.y + 12.0);
    let end = egui::pos2(rect.origin.x + 145.0, rect.origin.y + 34.0);

    harness.hover_at(start);
    harness.drag_at(start);
    harness.hover_at(end);
    let after = render_harness(&mut harness);

    let comparison = compare_images(&before, &after);
    let selection_pixels = count_visible_text_selection_pixels_in_rect(&after, rect);
    assert!(
        comparison.differing_pixels > 0,
        "dragging selectable text should visibly paint a document text selection"
    );
    assert!(
        selection_pixels > 20,
        "text selection should use the visible accent selection color; found {selection_pixels} matching pixels"
    );
}

#[test]
fn text_view_paints_rtl_pointer_selection_on_selectable_text() {
    let mut harness = lab_harness("text");
    let before = render_harness(&mut harness);
    let rect = state_rect(harness.state(), "text-rtl-start-body");
    let y = rect.origin.y + (rect.size.height / 2.0);
    let start = egui::pos2(rect.origin.x + rect.size.width - 12.0, y);
    let end = egui::pos2(rect.origin.x + 12.0, y);

    harness.hover_at(start);
    harness.drag_at(start);
    harness.hover_at(end);
    let after = render_harness(&mut harness);

    let selection = harness
        .state()
        .document_engine
        .text_selection()
        .expect("dragging RTL selectable text should create document text selection");
    let comparison = compare_images(&before, &after);
    let selection_pixels = count_visible_text_selection_pixels_in_rect(&after, rect);

    assert_eq!(selection.target, ElementId::new("text-rtl-start-body"));
    assert_ne!(selection.anchor_index, selection.focus_index);
    assert!(
        comparison.differing_pixels > 0,
        "dragging RTL selectable text should visibly paint a document text selection"
    );
    assert!(
        selection_pixels > 20,
        "RTL text selection should use the visible accent selection color; found {selection_pixels} matching pixels"
    );
}

#[test]
fn text_view_keeps_selection_visible_after_pointer_release() {
    let mut harness = lab_harness("text");
    let rect = state_rect(harness.state(), "text-wrap-body");
    let start = egui::pos2(rect.origin.x + 12.0, rect.origin.y + 12.0);
    let end = egui::pos2(rect.origin.x + 145.0, rect.origin.y + 34.0);

    harness.hover_at(start);
    harness.drag_at(start);
    harness.hover_at(end);
    harness.drop_at(end);
    harness.run();

    let selection = harness
        .state()
        .document_engine
        .text_selection()
        .expect("released drag should keep a completed document text selection");
    assert_eq!(selection.target, ElementId::new("text-wrap-body"));
    assert!(!selection.active);
    assert!(!selection.is_empty());
}

#[test]
fn text_view_copy_event_sends_selected_text_to_clipboard() {
    let mut harness = lab_harness("text");
    let rect = state_rect(harness.state(), "text-wrap-body");
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
fn text_context_menu_renders_document_widget_with_framework_style() {
    let mut state = UiLabState::new(Some("text"));
    state.text_context_menu = Some(TextContextMenu {
        target: ElementId::new("text-wrap-body"),
        position: Point::new(300.0, 300.0),
        selected_text: Some("Customer analytics".to_owned()),
    });

    let output = state_output(&state);

    let menu = frame(&output, "text-context-menu");
    assert!(has_class(menu, "context-menu"));
    assert_eq!(menu.style.background, Some(PANEL));
    assert_eq!(menu.style.border, Some(STROKE));
    assert_close(menu.rect.origin.x, 300.0);
    assert_close(menu.rect.origin.y, 300.0);

    let copy = frame(&output, "text-context-menu-copy");
    assert!(has_class(copy, "context-menu-item"));
    assert!(copy.interactive);
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
    assert_length_px(selected.style.gap, 18.0);
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

    assert!(
        output.snapshot().find("drag-scroll-list-card").is_none(),
        "animation view should leave drag/drop specimens to the dedicated draggable view"
    );
}

#[test]
fn animation_margin_specimen_expands_layout_on_hover() {
    let mut state = UiLabState::new(Some("animation"));
    let base = state.lab_document_output_for_test(Size::new(TEST_WIDTH, TEST_HEIGHT));
    let target = frame(&base, "animation-hover-margin-target");
    let row = frame(&base, "animation-hover-margin-row");
    let after = frame(&base, "animation-hover-margin-after");
    let pointer = Point::new(
        target.rect.origin.x + target.rect.size.width / 2.0,
        target.rect.origin.y + target.rect.size.height / 2.0,
    );

    let hovered = state.lab_document_output_with_input_for_test(
        Size::new(TEST_WIDTH, TEST_HEIGHT),
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
    let mut state = UiLabState::new(Some("animation"));
    let base = state.lab_document_output_for_test(Size::new(TEST_WIDTH, TEST_HEIGHT));
    let target = frame(&base, "animation-hover-margin-target");
    let hover_pointer = Point::new(
        target.rect.origin.x + target.rect.size.width / 2.0,
        target.rect.origin.y + target.rect.size.height / 2.0,
    );

    let mut output = state.lab_document_output_with_input_for_test(
        Size::new(TEST_WIDTH, TEST_HEIGHT),
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
        output = state.lab_document_output_with_input_for_test(
            Size::new(TEST_WIDTH, TEST_HEIGHT),
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
    output = state.lab_document_output_with_input_for_test(
        Size::new(TEST_WIDTH, TEST_HEIGHT),
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
        output = state.lab_document_output_with_input_for_test(
            Size::new(TEST_WIDTH, TEST_HEIGHT),
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
                .flex_direction(des_document::FlexDirection::Row)
                .width(Length::Auto)
                .height(Length::Auto)
                .gap(4.0),
        )
        .rule(
            StyleSelector::class("cell"),
            Style::default().size(10.0, 10.0),
        );
    let mut document = Document::build(Size::new(240.0, 160.0), |ui| {
        ui.element("outer", ElementSpec::new(Element::Div), |ui| {
            ui.element("inner", ElementSpec::new(Element::Div), |ui| {
                for row in 0..3 {
                    ui.element(
                        format!("row-{row}"),
                        ElementSpec::new(Element::Div).class("row"),
                        |ui| {
                            for column in 0..3 {
                                ui.element(
                                    format!("cell-{row}-{column}"),
                                    ElementSpec::new(Element::Div).class("cell"),
                                    |_| {},
                                );
                            }
                        },
                    );
                }
            });
        });
    });

    let output = engine.update(&mut document, &stylesheet);

    assert_close(output.layout.find("outer").unwrap().rect.size.width, 58.0);
    assert_close(output.layout.find("outer").unwrap().rect.size.height, 58.0);
    assert_close(output.layout.find("inner").unwrap().rect.size.width, 52.0);
    assert_close(output.layout.find("inner").unwrap().rect.size.height, 52.0);
}
