use super::*;
use crate::graphics_testing::{
    TEST_HEIGHT, TEST_WIDTH, assert_exact_image_match, compare_images, image_stats, render_harness,
    test_harness,
};
use des_ui_document::{
    Document, DocumentEngine, DocumentOutput, ElementRole, ElementSpec, Insets, Length,
    ResolvedElement, Size, StylePatch, StyleSelector, StyleSheet,
};
use egui_kittest::Harness;

fn lab_harness(initial_view: &str) -> Harness<'_, UiLabState> {
    test_harness(UiLabState::new(Some(initial_view)), |ui, state| {
        state.render(ui, false);
    })
}

fn lab_image(initial_view: &str) -> image::RgbaImage {
    render_harness(&mut lab_harness(initial_view))
}

fn lab_rect(id: &str) -> des_ui_document::Rect {
    let mut engine = DocumentEngine::default();
    let document =
        UiLabState::new(Some("layout")).document(Size::new(TEST_WIDTH, TEST_HEIGHT), false);
    let output = engine.update(&document, &stylesheet());
    find_frame(&output.layout, id)
        .unwrap_or_else(|| panic!("expected layout frame for {id}"))
        .rect
}

fn lab_output(initial_view: &str) -> DocumentOutput {
    let mut engine = DocumentEngine::default();
    let document =
        UiLabState::new(Some(initial_view)).document(Size::new(TEST_WIDTH, TEST_HEIGHT), false);
    engine.update(&document, &stylesheet())
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

fn assert_close(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() < 0.01,
        "expected {actual} to be close to {expected}"
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
fn box_model_specimens_cover_size_inset_and_flow_contracts() {
    let output = lab_output("layout");

    assert_close(frame(&output, "box-auto-subject").rect.size.width, 12.0);
    assert_close(frame(&output, "box-auto-subject").rect.size.height, 12.0);
    assert_close(frame(&output, "box-px-subject").rect.size.width, 96.0);
    assert_close(frame(&output, "box-px-subject").rect.size.height, 44.0);
    assert_close(frame(&output, "box-min-subject").rect.size.width, 40.0);
    assert_close(frame(&output, "box-min-subject").rect.size.height, 40.0);

    assert_close(frame(&output, "box-fill-subject").rect.size.width, 292.0);
    assert_close(frame(&output, "box-percent-subject").rect.size.width, 146.0);
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
    assert_close(frame(&output, "box-border-subject").style.border_width, 5.0);
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

    assert!(
        output
            .scroll_chrome
            .iter()
            .any(|chrome| chrome.element_id.as_str() == "box-scroll-overflow-subject"),
        "scroll overflow specimen should emit scroll chrome"
    );
}

#[test]
fn external_style_contract_can_drive_document_without_ui_lab_internals() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::Id("outer"),
            StylePatch::default()
                .width(Length::Auto)
                .height(Length::Auto)
                .margin(Insets::all(8.0))
                .border_width(3.0),
        )
        .rule(
            StyleSelector::Id("inner"),
            StylePatch::default()
                .width(Length::Auto)
                .height(Length::Auto)
                .padding(Insets::all(5.0))
                .gap(4.0)
                .border_width(2.0),
        )
        .rule(
            StyleSelector::Class("row"),
            StylePatch::default()
                .direction(des_ui_document::Direction::Row)
                .width(Length::Auto)
                .height(Length::Auto)
                .gap(4.0),
        )
        .rule(
            StyleSelector::Class("cell"),
            StylePatch::default().size(10.0, 10.0),
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
