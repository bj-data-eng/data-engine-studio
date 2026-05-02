use des_ui_document::{
    Color, CornerRadii, Document, DocumentEngine, DocumentInput, ElementId, ElementRole,
    ElementSpec, ElementStateSelector, Insets, Length, Overflow, Point, PointerInput, Size, Style,
    StyleSelector, StyleSheet, Transition,
};

#[test]
fn update_reports_created_retained_and_removed_elements() {
    let mut engine = DocumentEngine::default();
    let stylesheet = probe_stylesheet();
    let first = catalog_document("Projects");
    let first_output = engine.update(&first, &stylesheet);

    assert!(
        first_output
            .changes
            .created
            .contains(&ElementId::new("catalog"))
    );
    assert!(first_output.changes.retained.is_empty());

    engine.element_state_mut("catalog").unwrap().scroll_y = 42.0;

    let second = catalog_document("Flows");
    let second_output = engine.update(&second, &stylesheet);

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
    assert_eq!(engine.element_state("catalog").unwrap().scroll_y, 42.0);
}

#[test]
fn style_rules_resolve_role_class_state_and_id_in_order() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::Role(ElementRole::Card),
            Style::default()
                .size(100.0, 40.0)
                .background(Color::rgb(20, 20, 20)),
        )
        .rule(
            StyleSelector::class("selected"),
            Style::default().background(Color::rgb(35, 56, 78)),
        )
        .rule(
            StyleSelector::State(ElementStateSelector::Hovered),
            Style::default().background(Color::rgb(40, 70, 95)),
        )
        .rule(StyleSelector::id("card"), Style::default().radius(7.0));
    let document = Document::build(Size::new(320.0, 200.0), |ui| {
        ui.element(
            "card",
            ElementSpec::new(ElementRole::Card)
                .class("selected")
                .interactive(),
            |_| {},
        );
    });

    let output = engine.update_with_input(
        &document,
        &stylesheet,
        DocumentInput {
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
    assert_eq!(card.style.radius, CornerRadii::all(7.0));
}

#[test]
fn border_and_radius_rules_can_target_individual_sides_and_corners() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("card"),
            Style::default()
                .size(120.0, 80.0)
                .border_width(2.0)
                .radius(4.0),
        )
        .rule(
            StyleSelector::id("card"),
            Style::default()
                .border_left_width(8.0)
                .border_bottom_width(5.0)
                .top_right_radius(14.0)
                .bottom_left_radius(0.0),
        );
    let document = Document::build(Size::new(240.0, 160.0), |ui| {
        ui.element("card", ElementSpec::new(ElementRole::Card), |_| {});
    });

    let output = engine.update(&document, &stylesheet);
    let card = output.layout.find("card").unwrap();

    assert_eq!(
        card.style.border_width,
        Insets {
            top: 2.0,
            right: 2.0,
            bottom: 5.0,
            left: 8.0,
        }
    );
    assert_eq!(
        card.style.radius,
        CornerRadii {
            top_left: 4.0,
            top_right: 14.0,
            bottom_right: 4.0,
            bottom_left: 0.0,
        }
    );
}

#[test]
fn transitioned_state_rules_ease_visual_style_properties() {
    let mut engine = DocumentEngine::default();
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
    let document = Document::build(Size::new(320.0, 200.0), |ui| {
        ui.element(
            "card",
            ElementSpec::new(ElementRole::Card).interactive(),
            |_| {},
        );
    });

    engine.update(&document, &stylesheet);

    let output = engine.update_with_input(
        &document,
        &stylesheet,
        DocumentInput {
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
            engine.update_with_input(
                &document,
                &stylesheet,
                DocumentInput {
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
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("catalog"),
            Style::default().padding(Insets::all(10.0)).gap(4.0),
        )
        .rule(
            StyleSelector::class("indented"),
            Style::default().margin(Insets::symmetric(3.0, 2.0)),
        );
    let document = Document::build(Size::new(320.0, 200.0), |ui| {
        ui.element("catalog", ElementSpec::new(ElementRole::Panel), |ui| {
            ui.text("one", "One");
            ui.element(
                "two",
                ElementSpec::new(ElementRole::Text).class("indented"),
                |_| {},
            );
        });
    });

    let output = engine.update(&document, &stylesheet);
    let one = output.layout.find("one").unwrap();
    let two = output.layout.find("two").unwrap();

    assert_eq!(one.rect.origin, Point::new(10.0, 10.0));
    assert_eq!(two.rect.origin, Point::new(13.0, 34.0));
}

#[test]
fn fill_width_uses_parent_content_width_after_box_model() {
    let mut engine = DocumentEngine::default();
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
    let document = Document::build(Size::new(320.0, 200.0), |ui| {
        ui.element("panel", ElementSpec::new(ElementRole::Panel), |ui| {
            ui.element("row", ElementSpec::new(ElementRole::Card), |_| {});
        });
    });

    let output = engine.update(&document, &stylesheet);
    let row = output.layout.find("row").unwrap();

    assert_eq!(row.rect.origin, Point::new(17.0, 10.0));
    assert_eq!(row.rect.size, Size::new(166.0, 24.0));
}

#[test]
fn wrapped_row_layout_rearranges_children_and_expands_container_height() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("row"),
            Style::default()
                .direction(des_ui_document::Direction::Row)
                .wrap(true)
                .width(Length::Px(120.0))
                .height(Length::Auto)
                .gap(10.0),
        )
        .rule(
            StyleSelector::class("item"),
            Style::default().size(50.0, 20.0),
        );
    let document = Document::build(Size::new(240.0, 160.0), |ui| {
        ui.element("row", ElementSpec::new(ElementRole::Panel), |ui| {
            ui.element(
                "item-0",
                ElementSpec::new(ElementRole::Card).class("item"),
                |_| {},
            );
            ui.element(
                "item-1",
                ElementSpec::new(ElementRole::Card).class("item"),
                |_| {},
            );
            ui.element(
                "item-2",
                ElementSpec::new(ElementRole::Card).class("item"),
                |_| {},
            );
        });
    });

    let output = engine.update(&document, &stylesheet);
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
fn fill_size_does_not_inflate_auto_sized_parent_during_intrinsic_measurement() {
    let mut engine = DocumentEngine::default();
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
    let document = Document::build(Size::new(240.0, 160.0), |ui| {
        ui.element("panel", ElementSpec::new(ElementRole::Panel), |ui| {
            ui.element("child", ElementSpec::new(ElementRole::Card), |_| {});
        });
    });

    let output = engine.update(&document, &stylesheet);
    let panel = output.layout.find("panel").unwrap();

    assert_eq!(panel.rect.size, Size::new(24.0, 24.0));
}

#[test]
fn pointer_input_targets_interactive_owner_instead_of_inner_text() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new().rule(
        StyleSelector::Role(ElementRole::Card),
        Style::default().size(100.0, 40.0),
    );
    let document = Document::build(Size::new(320.0, 200.0), |ui| {
        ui.element(
            "card",
            ElementSpec::new(ElementRole::Card).interactive(),
            |ui| {
                ui.text("label", "Click target");
            },
        );
    });

    let output = engine.update_with_input(
        &document,
        &stylesheet,
        DocumentInput {
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
    let card_state = engine.element_state("card").unwrap();
    assert!(card_state.hovered);
    assert!(card_state.pressed);
    assert_eq!(card_state.click_count, 1);

    let label_state = engine.element_state("label").unwrap();
    assert!(label_state.hovered);
    assert!(!label_state.pressed);
    assert_eq!(label_state.click_count, 0);
}

#[test]
fn scroll_delta_updates_hovered_scroll_container_state() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("scroll-panel"),
            Style::default()
                .size(180.0, 80.0)
                .padding(Insets::all(8.0))
                .gap(4.0)
                .border_width(5.0)
                .overflow_y(Overflow::Scroll),
        )
        .rule(
            StyleSelector::Role(ElementRole::Card),
            Style::default().size(120.0, 36.0),
        );
    let document = overflowing_scroll_document();

    let output = engine.update_with_input(
        &document,
        &stylesheet,
        DocumentInput {
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
    assert_eq!(engine.element_state("scroll-panel").unwrap().scroll_y, 24.0);

    let output = engine.update(&document, &stylesheet);
    let first_row = output.layout.find("row-0").unwrap();
    assert_eq!(first_row.rect.origin.y, -11.0);
}

#[test]
fn scroll_delta_is_clamped_when_content_does_not_overflow() {
    let mut engine = DocumentEngine::default();
    let stylesheet = scroll_fixture_stylesheet(120.0);
    let document = Document::build(Size::new(240.0, 160.0), |ui| {
        ui.element("scroll-panel", ElementSpec::new(ElementRole::Panel), |ui| {
            ui.element("row-0", ElementSpec::new(ElementRole::Card), |_| {});
            ui.element("row-1", ElementSpec::new(ElementRole::Card), |_| {});
        });
    });

    engine.update_with_input(
        &document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(20.0, 20.0),
                primary_delta: Point::ZERO,
                primary_down: false,
                primary_clicked: false,
            }),
            scroll_delta: Point::new(0.0, -240.0),
        },
    );

    assert_eq!(engine.element_state("scroll-panel").unwrap().scroll_y, 0.0);
}

#[test]
fn overflow_scroll_container_emits_draggable_scroll_chrome() {
    let mut engine = DocumentEngine::default();
    let stylesheet = scroll_fixture_stylesheet(80.0);
    let document = overflowing_scroll_document();

    let output = engine.update(&document, &stylesheet);
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
    engine.update_with_input(
        &document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: grab,
                primary_delta: Point::ZERO,
                primary_down: true,
                primary_clicked: false,
            }),
            scroll_delta: Point::ZERO,
        },
    );
    engine.update_with_input(
        &document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(grab.x, grab.y + 24.0),
                primary_delta: Point::new(0.0, 24.0),
                primary_down: true,
                primary_clicked: false,
            }),
            scroll_delta: Point::ZERO,
        },
    );

    assert!(engine.element_state("scroll-panel").unwrap().scroll_y > 0.0);
}

#[test]
fn scroll_chrome_appears_on_container_hover_and_expands_on_hit_strip() {
    let mut engine = DocumentEngine::default();
    let stylesheet = scroll_fixture_stylesheet(80.0);
    let document = overflowing_scroll_document();

    let output = engine.update_with_input(
        &document,
        &stylesheet,
        DocumentInput {
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

    let output = engine.update_with_input(
        &document,
        &stylesheet,
        DocumentInput {
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

fn catalog_document(title_id: &str) -> Document {
    Document::build(Size::new(240.0, 480.0), |ui| {
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
            StyleSelector::class("catalog"),
            Style::default()
                .size(180.0, 40.0)
                .padding(Insets::all(12.0))
                .gap(8.0)
                .overflow_y(Overflow::Scroll),
        )
        .rule(
            StyleSelector::Role(ElementRole::Card),
            Style::default().size(180.0, 48.0),
        )
}

fn scroll_fixture_stylesheet(panel_height: f32) -> StyleSheet {
    StyleSheet::new()
        .rule(
            StyleSelector::id("scroll-panel"),
            Style::default()
                .size(180.0, panel_height)
                .padding(Insets::all(8.0))
                .gap(4.0)
                .overflow_y(Overflow::Scroll),
        )
        .rule(
            StyleSelector::Role(ElementRole::Card),
            Style::default().size(140.0, 32.0),
        )
}

fn overflowing_scroll_document() -> Document {
    Document::build(Size::new(240.0, 160.0), |ui| {
        ui.element("scroll-panel", ElementSpec::new(ElementRole::Panel), |ui| {
            for index in 0..6 {
                ui.element(
                    format!("row-{index}"),
                    ElementSpec::new(ElementRole::Card),
                    |_| {},
                );
            }
        });
    })
}
