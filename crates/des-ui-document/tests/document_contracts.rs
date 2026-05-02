use des_ui_document::{
    AlignItems, Color, CornerRadii, Document, DocumentEngine, DocumentInput, ElementId,
    ElementRole, ElementSpec, ElementStateSelector, Insets, JustifyContent, Length, Overflow,
    Point, PointerInput, ScrollAxis, Size, Style, StyleSelector, StyleSheet, Transition,
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
fn compound_selectors_require_all_parts_without_specificity_weighting() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::class("surface"),
            Style::default().background(Color::rgb(20, 20, 20)),
        )
        .rule(
            StyleSelector::compound()
                .role(ElementRole::Card)
                .class("surface")
                .class("compact")
                .selector(),
            Style::default().background(Color::rgb(35, 56, 78)),
        )
        .rule(
            StyleSelector::compound()
                .class("surface")
                .class("compact")
                .state(ElementStateSelector::Selected)
                .selector(),
            Style::default().border(Color::rgb(90, 180, 240)),
        )
        .rule(
            StyleSelector::class("surface"),
            Style::default().radius(3.0),
        );
    let document = Document::build(Size::new(320.0, 200.0), |ui| {
        ui.element(
            "matching",
            ElementSpec::new(ElementRole::Card)
                .class("surface")
                .class("compact")
                .selected(true)
                .interactive(),
            |_| {},
        );
        ui.element(
            "missing-compact",
            ElementSpec::new(ElementRole::Card)
                .class("surface")
                .interactive(),
            |_| {},
        );
    });

    let output = engine.update(&document, &stylesheet);
    let matching = output.layout.find("matching").unwrap();
    let missing = output.layout.find("missing-compact").unwrap();

    assert_eq!(matching.style.background, Some(Color::rgb(35, 56, 78)));
    assert_eq!(matching.style.border, Some(Color::rgb(90, 180, 240)));
    assert_eq!(matching.style.radius, CornerRadii::all(3.0));
    assert_eq!(missing.style.background, Some(Color::rgb(20, 20, 20)));
    assert_eq!(missing.style.border, None);
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
    assert!(output.metrics.reused_input_layout);

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

    assert!(card.style.background.unwrap().r > 31);
    assert!(output.metrics.reused_input_layout);
    assert!(output.metrics.animation_changed_style);
    assert!(!output.metrics.animation_changed_layout);
    assert!(output.metrics.animation_changed_paint);

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
    assert!(!output.animating);
    assert!(!output.metrics.animation_changed_style);

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

    assert!(!output.animating);
    assert!(!output.metrics.animation_changed_style);
}

#[test]
fn untransitioned_hover_color_reuses_layout_and_updates_paint() {
    let mut engine = DocumentEngine::default();
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

    assert_eq!(card.style.background, Some(Color::rgb(40, 70, 95)));
    assert!(output.metrics.reused_input_layout);
    assert!(output.metrics.input_changed_state);
    assert!(output.metrics.animation_changed_style);
    assert!(!output.metrics.animation_changed_layout);
    assert!(output.metrics.animation_changed_paint);
}

#[test]
fn untransitioned_hover_layout_change_rebuilds_layout() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::Role(ElementRole::Card),
            Style::default().size(100.0, 40.0),
        )
        .rule(
            StyleSelector::State(ElementStateSelector::Hovered),
            Style::default().size(140.0, 40.0),
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

    assert_eq!(card.rect.size, Size::new(140.0, 40.0));
    assert!(!output.metrics.reused_input_layout);
    assert!(output.metrics.animation_changed_style);
    assert!(output.metrics.animation_changed_layout);
}

#[test]
fn transitioned_state_rules_ease_layout_and_box_model_properties() {
    let mut engine = DocumentEngine::default();
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
                position: Point::new(4.0, 4.0),
                primary_delta: Point::ZERO,
                primary_down: false,
                primary_clicked: false,
            }),
            scroll_delta: Point::ZERO,
        },
    );
    let card = output.layout.find("card").unwrap();

    assert_eq!(card.rect.size, Size::new(110.0, 50.0));
    assert_eq!(card.style.min_size, Size::new(25.0, 30.0));
    assert_eq!(card.style.max_size, Size::new(190.0, 130.0));
    assert_eq!(card.style.padding, Insets::all(6.0));
    assert_eq!(card.style.margin, Insets::all(4.0));
    assert_eq!(card.style.gap, 8.0);
    assert_eq!(card.style.border_width, Insets::all(4.0));
    assert_eq!(card.style.radius, CornerRadii::all(8.0));
    assert_eq!(card.style.font_size, 14.0);
    assert!(!output.metrics.reused_input_layout);
    assert!(output.metrics.animation_changed_style);
    assert!(output.metrics.animation_changed_layout);
    assert!(output.metrics.animation_changed_paint);

    let output = engine.update_with_input(
        &document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(4.0, 4.0),
                primary_delta: Point::ZERO,
                primary_down: false,
                primary_clicked: false,
            }),
            scroll_delta: Point::ZERO,
        },
    );

    assert!(!output.metrics.reused_input_layout);
    assert!(output.metrics.animation_changed_layout);
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
fn row_layout_applies_main_and_cross_axis_alignment() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("row"),
            Style::default()
                .direction(des_ui_document::Direction::Row)
                .size(160.0, 80.0)
                .gap(10.0)
                .justify_content(JustifyContent::Center)
                .align_items(AlignItems::End),
        )
        .rule(
            StyleSelector::class("item"),
            Style::default().size(40.0, 20.0),
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
        });
    });

    let output = engine.update(&document, &stylesheet);

    assert_eq!(
        output.layout.find("item-0").unwrap().rect.origin,
        Point::new(35.0, 60.0)
    );
    assert_eq!(
        output.layout.find("item-1").unwrap().rect.origin,
        Point::new(85.0, 60.0)
    );
}

#[test]
fn column_layout_applies_main_and_cross_axis_alignment() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("column"),
            Style::default()
                .direction(des_ui_document::Direction::Column)
                .size(120.0, 120.0)
                .gap(5.0)
                .justify_content(JustifyContent::SpaceBetween)
                .align_items(AlignItems::Center),
        )
        .rule(
            StyleSelector::class("item"),
            Style::default().size(30.0, 20.0),
        );
    let document = Document::build(Size::new(180.0, 160.0), |ui| {
        ui.element("column", ElementSpec::new(ElementRole::Panel), |ui| {
            for index in 0..3 {
                ui.element(
                    format!("item-{index}"),
                    ElementSpec::new(ElementRole::Card).class("item"),
                    |_| {},
                );
            }
        });
    });

    let output = engine.update(&document, &stylesheet);

    assert_eq!(
        output.layout.find("item-0").unwrap().rect.origin,
        Point::new(45.0, 0.0)
    );
    assert_eq!(
        output.layout.find("item-1").unwrap().rect.origin,
        Point::new(45.0, 50.0)
    );
    assert_eq!(
        output.layout.find("item-2").unwrap().rect.origin,
        Point::new(45.0, 100.0)
    );
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
fn max_size_clamps_auto_explicit_and_fill_sizes() {
    let mut engine = DocumentEngine::default();
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
    let document = Document::build(Size::new(260.0, 180.0), |ui| {
        ui.element("panel", ElementSpec::new(ElementRole::Panel), |ui| {
            ui.element("auto-child", ElementSpec::new(ElementRole::Panel), |ui| {
                ui.element(
                    "wide-child",
                    ElementSpec::new(ElementRole::Card).class("wide"),
                    |_| {},
                );
            });
            ui.element("fixed-child", ElementSpec::new(ElementRole::Card), |_| {});
            ui.element("fill-child", ElementSpec::new(ElementRole::Card), |_| {});
        });
    });

    let output = engine.update(&document, &stylesheet);

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
fn absolute_parent_position_uses_parent_content_rect_and_leaves_flow_measurement() {
    let mut engine = DocumentEngine::default();
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
    let document = Document::build(Size::new(320.0, 200.0), |ui| {
        ui.element("panel", ElementSpec::new(ElementRole::Panel), |ui| {
            ui.element(
                "absolute-child",
                ElementSpec::new(ElementRole::Card),
                |_| {},
            );
            ui.element("flow-child", ElementSpec::new(ElementRole::Card), |_| {});
        });
    });

    let output = engine.update(&document, &stylesheet);
    let panel = output.layout.find("panel").unwrap();
    let absolute_child = output.layout.find("absolute-child").unwrap();
    let flow_child = output.layout.find("flow-child").unwrap();

    assert_eq!(panel.rect.size, Size::new(74.0, 44.0));
    assert_eq!(flow_child.rect.origin, Point::new(12.0, 12.0));
    assert_eq!(absolute_child.rect.origin, Point::new(19.0, 17.0));
}

#[test]
fn absolute_viewport_position_uses_window_rect() {
    let mut engine = DocumentEngine::default();
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
    let document = Document::build(Size::new(320.0, 200.0), |ui| {
        ui.element("panel", ElementSpec::new(ElementRole::Panel), |ui| {
            ui.element(
                "absolute-child",
                ElementSpec::new(ElementRole::Card),
                |_| {},
            );
        });
    });

    let output = engine.update(&document, &stylesheet);
    let absolute_child = output.layout.find("absolute-child").unwrap();

    assert_eq!(absolute_child.rect.origin, Point::new(272.0, 171.0));
}

#[test]
fn pointer_input_can_target_absolute_child_outside_parent_box() {
    let mut engine = DocumentEngine::default();
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
    let document = Document::build(Size::new(320.0, 200.0), |ui| {
        ui.element("panel", ElementSpec::new(ElementRole::Panel), |ui| {
            ui.element(
                "absolute-child",
                ElementSpec::new(ElementRole::Card).interactive(),
                |_| {},
            );
        });
    });

    let output = engine.update_with_input(
        &document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(150.0, 90.0),
                primary_delta: Point::ZERO,
                primary_down: true,
                primary_clicked: true,
            }),
            scroll_delta: Point::ZERO,
        },
    );

    assert_eq!(output.hit_id, Some(ElementId::new("absolute-child")));
    assert!(engine.element_state("absolute-child").unwrap().pressed);
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
fn horizontal_overflow_scrolls_child_content_on_x_axis() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("scroll-panel"),
            Style::default()
                .direction(des_ui_document::Direction::Row)
                .size(80.0, 70.0)
                .gap(4.0)
                .overflow_x(Overflow::Scroll),
        )
        .rule(
            StyleSelector::Role(ElementRole::Card),
            Style::default().size(50.0, 32.0),
        );
    let document = Document::build(Size::new(180.0, 120.0), |ui| {
        ui.element("scroll-panel", ElementSpec::new(ElementRole::Panel), |ui| {
            ui.element("item-0", ElementSpec::new(ElementRole::Card), |_| {});
            ui.element("item-1", ElementSpec::new(ElementRole::Card), |_| {});
            ui.element("item-2", ElementSpec::new(ElementRole::Card), |_| {});
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
            scroll_delta: Point::new(-30.0, 0.0),
        },
    );

    assert_eq!(engine.element_state("scroll-panel").unwrap().scroll_x, 30.0);
    let output = engine.update(&document, &stylesheet);
    assert_eq!(
        output.layout.find("item-0").unwrap().rect.origin,
        Point::new(-30.0, 0.0)
    );
    assert!(
        output.scroll_chrome.iter().any(|chrome| {
            chrome.element_id == ElementId::new("scroll-panel")
                && chrome.axis == ScrollAxis::Horizontal
                && chrome.max_scroll > 0.0
        }),
        "horizontal overflow should emit horizontal scroll chrome"
    );
}

#[test]
fn two_axis_overflow_keeps_independent_scroll_state_and_chrome() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("scroll-panel"),
            Style::default()
                .size(70.0, 70.0)
                .overflow(Overflow::Scroll)
                .scrollbar_width(2.0)
                .scrollbar_expanded_width(10.0)
                .scrollbar_pressed_handle_color(Color::rgba(190, 217, 255, 238))
                .scrollbar_pressed_handle_border_color(Color::rgba(255, 255, 255, 120))
                .scrollbar_pressed_handle_border_width(1.0),
        )
        .rule(
            StyleSelector::id("content"),
            Style::default().size(140.0, 140.0),
        );
    let document = Document::build(Size::new(180.0, 140.0), |ui| {
        ui.element("scroll-panel", ElementSpec::new(ElementRole::Panel), |ui| {
            ui.element("content", ElementSpec::new(ElementRole::Card), |_| {});
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
            scroll_delta: Point::new(-16.0, -24.0),
        },
    );

    let state = engine.element_state("scroll-panel").unwrap();
    assert_eq!(state.scroll_x, 16.0);
    assert_eq!(state.scroll_y, 24.0);

    let output = engine.update(&document, &stylesheet);
    let content = output.layout.find("content").unwrap();
    assert_eq!(content.rect.origin, Point::new(-16.0, -24.0));
    assert!(
        output.scroll_chrome.iter().any(|chrome| {
            chrome.element_id == ElementId::new("scroll-panel")
                && chrome.axis == ScrollAxis::Horizontal
        }),
        "two-axis overflow should emit horizontal chrome"
    );
    assert!(
        output.scroll_chrome.iter().any(|chrome| {
            chrome.element_id == ElementId::new("scroll-panel")
                && chrome.axis == ScrollAxis::Vertical
        }),
        "two-axis overflow should emit vertical chrome"
    );

    let output = engine.update_with_input(
        &document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(64.0, 20.0),
                primary_delta: Point::ZERO,
                primary_down: false,
                primary_clicked: false,
            }),
            scroll_delta: Point::ZERO,
        },
    );
    let vertical = output
        .scroll_chrome
        .iter()
        .find(|chrome| {
            chrome.element_id == ElementId::new("scroll-panel")
                && chrome.axis == ScrollAxis::Vertical
        })
        .unwrap();
    let horizontal = output
        .scroll_chrome
        .iter()
        .find(|chrome| {
            chrome.element_id == ElementId::new("scroll-panel")
                && chrome.axis == ScrollAxis::Horizontal
        })
        .unwrap();
    assert!(vertical.expanded);
    assert_eq!(vertical.handle_rect.size.width, 10.0);
    assert!(!horizontal.expanded);
    assert_eq!(horizontal.handle_rect.size.height, 2.0);

    let output = engine.update_with_input(
        &document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(64.0, 20.0),
                primary_delta: Point::ZERO,
                primary_down: true,
                primary_clicked: false,
            }),
            scroll_delta: Point::ZERO,
        },
    );
    let vertical = output
        .scroll_chrome
        .iter()
        .find(|chrome| {
            chrome.element_id == ElementId::new("scroll-panel")
                && chrome.axis == ScrollAxis::Vertical
        })
        .unwrap();
    let horizontal = output
        .scroll_chrome
        .iter()
        .find(|chrome| {
            chrome.element_id == ElementId::new("scroll-panel")
                && chrome.axis == ScrollAxis::Horizontal
        })
        .unwrap();
    assert!(vertical.dragged);
    assert_eq!(vertical.handle_rect.size.width, 10.0);
    assert_eq!(vertical.handle_color.a, 238);
    assert!(vertical.handle_border_color.is_some());
    assert!(!horizontal.dragged);
    assert_eq!(horizontal.handle_rect.size.height, 2.0);
    assert_eq!(horizontal.handle_color.a, 118);
    assert!(horizontal.handle_border_color.is_none());
}

#[test]
fn scrollbar_hover_transition_reuses_layout() {
    let mut engine = DocumentEngine::default();
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
    let document = Document::build(Size::new(180.0, 140.0), |ui| {
        ui.element("scroll-panel", ElementSpec::new(ElementRole::Panel), |ui| {
            ui.element("content", ElementSpec::new(ElementRole::Card), |_| {});
        });
    });

    engine.update(&document, &stylesheet);
    let output = engine.update_with_input(
        &document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(64.0, 20.0),
                primary_delta: Point::ZERO,
                primary_down: false,
                primary_clicked: false,
            }),
            scroll_delta: Point::ZERO,
        },
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

    let output = engine.update_with_input(
        &document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(64.0, 20.0),
                primary_delta: Point::ZERO,
                primary_down: false,
                primary_clicked: false,
            }),
            scroll_delta: Point::ZERO,
        },
    );

    assert!(output.metrics.reused_input_layout);
    assert!(!output.metrics.animation_changed_layout);
}

#[test]
fn nested_scroll_chrome_is_clipped_by_ancestor_scroll_viewport() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("horizontal-parent"),
            Style::default()
                .direction(des_ui_document::Direction::Row)
                .size(120.0, 96.0)
                .gap(10.0)
                .overflow_x(Overflow::Scroll)
                .overflow_y(Overflow::Visible)
                .scrollbar_width(2.0),
        )
        .rule(
            StyleSelector::class("nested-list"),
            Style::default()
                .size(70.0, 74.0)
                .overflow_y(Overflow::Scroll)
                .scrollbar_width(2.0)
                .scrollbar_expanded_width(10.0),
        )
        .rule(
            StyleSelector::class("nested-row"),
            Style::default().size(54.0, 28.0),
        );
    let document = Document::build(Size::new(180.0, 140.0), |ui| {
        ui.element(
            "horizontal-parent",
            ElementSpec::new(ElementRole::Panel),
            |ui| {
                for list_index in 0..3 {
                    ui.element(
                        format!("nested-list-{list_index}"),
                        ElementSpec::new(ElementRole::Panel).class("nested-list"),
                        |ui| {
                            for row_index in 0..5 {
                                ui.element(
                                    format!("nested-list-{list_index}-row-{row_index}"),
                                    ElementSpec::new(ElementRole::Card).class("nested-row"),
                                    |_| {},
                                );
                            }
                        },
                    );
                }
            },
        );
    });

    let output = engine.update(&document, &stylesheet);
    let visible_parent_right = output
        .layout
        .find("horizontal-parent")
        .unwrap()
        .rect
        .right();
    let nested_vertical_chrome: Vec<_> = output
        .scroll_chrome
        .iter()
        .filter(|chrome| {
            chrome.element_id.as_str().starts_with("nested-list-")
                && chrome.axis == ScrollAxis::Vertical
        })
        .collect();

    assert_eq!(
        nested_vertical_chrome.len(),
        1,
        "only the fully visible nested list should expose vertical chrome"
    );
    let chrome = nested_vertical_chrome[0];
    assert_eq!(chrome.element_id, ElementId::new("nested-list-0"));
    assert!(chrome.hit_rect.right() <= visible_parent_right);
    assert!(chrome.track_rect.right() <= visible_parent_right);
    assert!(chrome.handle_rect.right() <= visible_parent_right);
}

#[test]
fn clipped_scroll_chrome_does_not_drive_animation_work() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("horizontal-parent"),
            Style::default()
                .direction(des_ui_document::Direction::Row)
                .size(120.0, 96.0)
                .gap(10.0)
                .overflow_x(Overflow::Scroll)
                .overflow_y(Overflow::Visible)
                .scrollbar_width(2.0)
                .scrollbar_expanded_width(10.0)
                .transition(Transition::ease_out(0.2)),
        )
        .rule(
            StyleSelector::class("nested-list"),
            Style::default()
                .size(70.0, 74.0)
                .overflow_y(Overflow::Scroll)
                .scrollbar_width(2.0)
                .scrollbar_expanded_width(10.0)
                .transition(Transition::ease_out(0.2)),
        )
        .rule(
            StyleSelector::class("nested-row"),
            Style::default().size(54.0, 28.0),
        );
    let document = Document::build(Size::new(180.0, 140.0), |ui| {
        ui.element(
            "horizontal-parent",
            ElementSpec::new(ElementRole::Panel),
            |ui| {
                for list_index in 0..3 {
                    ui.element(
                        format!("nested-list-{list_index}"),
                        ElementSpec::new(ElementRole::Panel).class("nested-list"),
                        |ui| {
                            for row_index in 0..5 {
                                ui.element(
                                    format!("nested-list-{list_index}-row-{row_index}"),
                                    ElementSpec::new(ElementRole::Card).class("nested-row"),
                                    |_| {},
                                );
                            }
                        },
                    );
                }
            },
        );
    });

    engine.update(&document, &stylesheet);
    engine
        .element_state_mut("horizontal-parent")
        .unwrap()
        .scroll_x = 110.0;
    let output = engine.update(&document, &stylesheet);
    let nested_scrollbar = output
        .scroll_chrome
        .iter()
        .find(|chrome| chrome.element_id == ElementId::new("nested-list-2"))
        .expect("nested list should be visible after horizontal scroll");
    let pointer = Point::new(
        nested_scrollbar.hit_rect.origin.x + nested_scrollbar.hit_rect.size.width / 2.0,
        nested_scrollbar.hit_rect.origin.y + nested_scrollbar.hit_rect.size.height / 2.0,
    );
    engine.update_with_input(
        &document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: pointer,
                primary_delta: Point::ZERO,
                primary_down: false,
                primary_clicked: false,
            }),
            scroll_delta: Point::ZERO,
        },
    );
    engine
        .element_state_mut("horizontal-parent")
        .unwrap()
        .scroll_x = 0.0;
    let output = engine.update(&document, &stylesheet);

    assert!(
        !output.animating,
        "offscreen nested scrollbars should not keep the document animating"
    );
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
        .find(|chrome| {
            chrome.element_id == ElementId::new("scroll-panel")
                && chrome.axis == ScrollAxis::Vertical
        })
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
    assert!(chrome.track_color.is_some());
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
    assert!(chrome.handle_rect.size.width > 2.0);
    assert!(chrome.handle_rect.size.width < 10.0);
    assert!(chrome.track_color.is_some());
    assert_eq!(chrome.handle_color.a, 118);
    assert!(chrome.handle_border_color.is_none());

    let output = engine.update_with_input(
        &document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(170.0, 20.0),
                primary_delta: Point::ZERO,
                primary_down: true,
                primary_clicked: true,
            }),
            scroll_delta: Point::ZERO,
        },
    );
    let chrome = output
        .scroll_chrome
        .iter()
        .find(|chrome| chrome.element_id == ElementId::new("scroll-panel"))
        .unwrap();
    assert!(chrome.dragged);
    assert!(chrome.handle_rect.size.width > 2.0);
    assert!(chrome.handle_rect.size.width < 10.0);
    assert_eq!(chrome.track_color, Some(Color::rgba(2, 8, 12, 84)));
    assert!(chrome.handle_color.a > 118);
    assert!(chrome.handle_border_color.is_some());
    assert!(chrome.handle_border_width > 0.0);
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
                .overflow_y(Overflow::Scroll)
                .scrollbar_width(2.0)
                .scrollbar_expanded_width(10.0)
                .scrollbar_handle_color(Color::rgba(232, 236, 240, 118))
                .scrollbar_track_color(Color::rgba(2, 8, 12, 84))
                .scrollbar_hover_track_color(Color::rgba(2, 8, 12, 84))
                .scrollbar_pressed_track_color(Color::rgba(2, 8, 12, 84))
                .scrollbar_pressed_handle_color(Color::rgba(190, 217, 255, 238))
                .scrollbar_pressed_handle_border_color(Color::rgba(255, 255, 255, 120))
                .scrollbar_pressed_handle_border_width(1.0)
                .scrollbar_radius(6.0)
                .transition(Transition::ease_out(0.2)),
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
