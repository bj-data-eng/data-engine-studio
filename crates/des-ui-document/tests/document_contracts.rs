use des_ui_document::{
    AlignItems, Color, CornerRadii, Document, DocumentEngine, DocumentEvent, DocumentEventKind,
    DocumentInput, DocumentUpdate, ElementId, ElementRole, ElementSpec, ElementStateSelector,
    Insets, JustifyContent, Length, Overflow, Point, PointerInput, ScrollAxis, Size, Style,
    StyleSelector, StyleSheet, TableCellSpec, TableColumnSpec, TableSpec, TableTrackSize,
    TextLayoutRequest, TextLayoutResult, TextMeasurer, TextMeasurerKey, TextSelectionGranularity,
    TextWrapMode, Transition,
};

fn assert_close(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() < 0.01,
        "expected {actual} to be close to {expected}"
    );
}

fn pointer_input(
    position: Point,
    primary_down: bool,
    primary_pressed: bool,
    primary_clicked: bool,
    time_seconds: f64,
) -> DocumentInput {
    DocumentInput {
        pointer: Some(PointerInput {
            position,
            primary_delta: Point::ZERO,
            primary_down,
            primary_pressed,
            primary_clicked,
            primary_click_count: u8::from(primary_clicked),
            secondary_clicked: false,
            time_seconds,
        }),
        scroll_delta: Point::ZERO,
    }
}

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

    assert_eq!(card.style.background, Some(Color::rgb(40, 70, 95)));
    assert_eq!(card.style.radius, CornerRadii::all(7.0));
}

#[test]
fn document_update_can_add_remove_and_toggle_classes_before_layout() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::Role(ElementRole::Card),
            Style::default()
                .size(100.0, 40.0)
                .background(Color::rgb(20, 20, 20)),
        )
        .rule(
            StyleSelector::class("expanded"),
            Style::default().size(140.0, 60.0),
        )
        .rule(
            StyleSelector::class("accent"),
            Style::default().background(Color::rgb(35, 56, 78)),
        );
    let mut document = Document::build(Size::new(320.0, 200.0), |ui| {
        ui.element("card", ElementSpec::new(ElementRole::Card), |_| {});
    });

    let report = document.apply_update(
        &DocumentUpdate::new()
            .add_class("card", "expanded")
            .toggle_class("card", "accent")
            .add_class("missing", "accent"),
    );
    assert_eq!(report.matched, 2);
    assert_eq!(report.changed, 2);
    assert_eq!(report.missing_targets, vec![ElementId::new("missing")]);

    let output = engine.update(&document, &stylesheet);
    let card = output.layout.find("card").unwrap();
    assert_eq!(card.rect.size, Size::new(140.0, 60.0));
    assert_eq!(card.style.background, Some(Color::rgb(35, 56, 78)));

    let report = document.apply_update(
        &DocumentUpdate::new()
            .remove_class("card", "expanded")
            .toggle_class("card", "accent"),
    );
    assert_eq!(report.matched, 2);
    assert_eq!(report.changed, 2);

    let output = engine.update(&document, &stylesheet);
    let card = output.layout.find("card").unwrap();
    assert_eq!(card.rect.size, Size::new(100.0, 40.0));
    assert_eq!(card.style.background, Some(Color::rgb(20, 20, 20)));
}

#[test]
fn document_update_can_set_text_value_and_authored_states() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::Role(ElementRole::Text),
            Style::default().text_color(Color::rgb(20, 20, 20)),
        )
        .rule(
            StyleSelector::State(ElementStateSelector::Selected),
            Style::default().background(Color::rgb(35, 56, 78)),
        )
        .rule(
            StyleSelector::State(ElementStateSelector::Disabled),
            Style::default().text_color(Color::rgb(90, 96, 102)),
        )
        .rule(
            StyleSelector::State(ElementStateSelector::Focused),
            Style::default().border(Color::rgb(88, 157, 230)),
        );
    let mut document = Document::build(Size::new(320.0, 200.0), |ui| {
        ui.text("label", "Short");
        ui.element(
            "control",
            ElementSpec::new(ElementRole::Control)
                .interactive()
                .value("initial"),
            |_| {},
        );
    });

    let output = engine.update(&document, &stylesheet);
    let label = output.layout.find("label").unwrap();
    assert_eq!(label.text.as_deref(), Some("Short"));
    assert_eq!(label.rect.size.width, 37.5);

    let report = document.apply_update(
        &DocumentUpdate::new()
            .set_text("label", "Much longer text")
            .set_value("control", "updated")
            .set_selected("control", true)
            .set_disabled("control", true)
            .set_focused("control", true),
    );
    assert_eq!(report.matched, 5);
    assert_eq!(report.changed, 5);

    let output = engine.update(&document, &stylesheet);
    let label = output.layout.find("label").unwrap();
    let control = output.layout.find("control").unwrap();

    assert_eq!(label.text.as_deref(), Some("Much longer text"));
    assert_eq!(label.rect.size.width, 120.0);
    assert_eq!(control.value.as_deref(), Some("updated"));
    assert_eq!(control.style.background, Some(Color::rgb(35, 56, 78)));
    assert_eq!(control.style.text_color, Color::rgb(90, 96, 102));
    assert_eq!(control.style.border, Some(Color::rgb(88, 157, 230)));
    assert!(!control.interactive);
}

#[test]
fn document_snapshot_queries_resolved_elements_without_mutation_access() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::Role(ElementRole::Card),
            Style::default().size(80.0, 30.0),
        )
        .rule(
            StyleSelector::class("drop-zone"),
            Style::default().background(Color::rgb(35, 56, 78)),
        );
    let document = Document::build(Size::new(320.0, 200.0), |ui| {
        ui.element(
            "drop-target",
            ElementSpec::new(ElementRole::Card)
                .class("drop-zone")
                .value("target-a")
                .interactive(),
            |ui| {
                ui.text("drop-label", "Drop here");
            },
        );
        ui.element("plain-card", ElementSpec::new(ElementRole::Card), |_| {});
    });

    let output = engine.update(&document, &stylesheet);
    let snapshot = output.snapshot();
    let drop_target = snapshot.find("drop-target").unwrap();

    assert_eq!(snapshot.root().id().as_str(), "root");
    assert_eq!(drop_target.role(), ElementRole::Card);
    assert!(drop_target.has_class("drop-zone"));
    assert_eq!(drop_target.value(), Some("target-a"));
    assert!(drop_target.interactive());
    assert_eq!(drop_target.rect().size, Size::new(80.0, 30.0));
    assert_eq!(
        snapshot.find("drop-label").unwrap().text(),
        Some("Drop here")
    );
    assert_eq!(snapshot.elements_with_class("drop-zone").len(), 1);
    assert_eq!(snapshot.elements_by_role(ElementRole::Card).len(), 2);
}

#[test]
fn document_snapshot_hit_test_returns_target_and_path() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("panel"),
            Style::default().size(120.0, 80.0),
        )
        .rule(StyleSelector::id("base"), Style::default().size(80.0, 40.0))
        .rule(
            StyleSelector::id("overlay"),
            Style::default()
                .absolute_parent()
                .left(Length::Px(20.0))
                .top(Length::Px(10.0))
                .z_index(5)
                .size(80.0, 40.0),
        );
    let document = Document::build(Size::new(320.0, 200.0), |ui| {
        ui.element("panel", ElementSpec::new(ElementRole::Panel), |ui| {
            ui.element("base", ElementSpec::new(ElementRole::Card), |_| {});
            ui.element("overlay", ElementSpec::new(ElementRole::Card), |_| {});
        });
    });

    let output = engine.update(&document, &stylesheet);
    let hit = output
        .snapshot()
        .hit_test(Point::new(30.0, 20.0))
        .expect("expected hit result");
    let path: Vec<_> = hit
        .path
        .iter()
        .map(|element| element.id().as_str())
        .collect();

    assert_eq!(hit.target.id().as_str(), "overlay");
    assert_eq!(hit.point, Point::new(30.0, 20.0));
    assert_eq!(path, vec!["root", "panel", "overlay"]);
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
fn structural_selectors_match_first_last_and_nth_children() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::Role(ElementRole::Card),
            Style::default().size(20.0, 20.0),
        )
        .rule(
            StyleSelector::first_child(),
            Style::default().background(Color::rgb(10, 20, 30)),
        )
        .rule(
            StyleSelector::nth_child(2),
            Style::default().background(Color::rgb(40, 50, 60)),
        )
        .rule(
            StyleSelector::last_child(),
            Style::default().border(Color::rgb(70, 80, 90)),
        )
        .rule(
            StyleSelector::compound()
                .class("item")
                .nth_child(3)
                .selector(),
            Style::default().radius(9.0),
        );
    let document = Document::build(Size::new(320.0, 200.0), |ui| {
        ui.element("panel", ElementSpec::new(ElementRole::Panel), |ui| {
            ui.element(
                "first",
                ElementSpec::new(ElementRole::Card).class("item"),
                |_| {},
            );
            ui.element(
                "second",
                ElementSpec::new(ElementRole::Card).class("item"),
                |_| {},
            );
            ui.element(
                "third",
                ElementSpec::new(ElementRole::Card).class("item"),
                |_| {},
            );
        });
    });

    let output = engine.update(&document, &stylesheet);
    let first = output.layout.find("first").unwrap();
    let second = output.layout.find("second").unwrap();
    let third = output.layout.find("third").unwrap();

    assert_eq!(first.style.background, Some(Color::rgb(10, 20, 30)));
    assert_eq!(first.style.border, None);
    assert_eq!(second.style.background, Some(Color::rgb(40, 50, 60)));
    assert_eq!(third.style.border, Some(Color::rgb(70, 80, 90)));
    assert_eq!(third.style.radius, CornerRadii::all(9.0));
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
    assert!(output.metrics.reused_input_layout);

    let output = engine.update_with_input(
        &document,
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
                        primary_pressed: false,
                        primary_clicked: false,
                        primary_click_count: 0,
                        secondary_clicked: false,
                        time_seconds: 0.0,
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
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: false,
                time_seconds: 0.0,
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
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: false,
                time_seconds: 0.0,
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
fn table_layout_resolves_shared_column_tracks_for_header_and_body_cells() {
    let mut engine = DocumentEngine::default();
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
    let document = table_fixture_document();
    let output = engine.update(&document, &stylesheet);

    let header_customer = output.layout.find("customers-header-customer").unwrap();
    let row_customer = output.layout.find("customers-row-0-customer").unwrap();
    let header_orders = output.layout.find("customers-header-orders").unwrap();
    let row_orders = output.layout.find("customers-row-0-orders").unwrap();

    assert_eq!(header_customer.role, ElementRole::TableCell);
    assert_close(
        header_customer.rect.size.width,
        row_customer.rect.size.width,
    );
    assert_close(header_orders.rect.origin.x, row_orders.rect.origin.x);
    assert_close(header_orders.rect.size.width, 80.0);
    assert!(
        row_customer.rect.origin.y > header_customer.rect.origin.y,
        "body rows should be laid out below the header row"
    );
    assert!(
        output.scroll_chrome.iter().any(|chrome| {
            chrome.element_id == ElementId::new("customers")
                && chrome.axis == ScrollAxis::Horizontal
                && chrome.max_scroll > 0.0
        }),
        "table content wider than the styled table frame should expose horizontal overflow"
    );
}

#[test]
fn text_layout_uses_document_wrap_and_truncation_styles() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("wrapped"),
            Style::default()
                .width(Length::Px(90.0))
                .text_wrap(TextWrapMode::Wrap),
        )
        .rule(
            StyleSelector::id("truncated"),
            Style::default()
                .width(Length::Px(90.0))
                .text_wrap(TextWrapMode::Truncate),
        );
    let document = Document::build(Size::new(240.0, 160.0), |ui| {
        ui.text_element(
            "wrapped",
            ElementSpec::new(ElementRole::Text),
            "Customer analytics pipeline preview",
        );
        ui.text_element(
            "truncated",
            ElementSpec::new(ElementRole::Text),
            "Customer analytics pipeline preview",
        );
    });

    let output = engine.update(&document, &stylesheet);
    let wrapped = output.layout.find("wrapped").unwrap();
    let truncated = output.layout.find("truncated").unwrap();

    assert!(
        wrapped.text_layout.unwrap().line_count > 1,
        "wrapped text should report multiple measured lines"
    );
    assert_eq!(truncated.text_layout.unwrap().line_count, 1);
    assert!(truncated.text_layout.unwrap().elided);
}

#[test]
fn text_layout_respects_padding_and_border_box_size() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new().rule(
        StyleSelector::id("label"),
        Style::default().padding(Insets::all(4.0)).border_width(2.0),
    );
    let document = Document::build(Size::new(240.0, 160.0), |ui| {
        ui.text_element("label", ElementSpec::new(ElementRole::Text), "Hi");
    });

    let output = engine.update(&document, &stylesheet);
    let label = output.layout.find("label").unwrap();

    assert_close(label.text_layout.unwrap().size.width, 15.0);
    assert_close(label.text_layout.unwrap().size.height, 18.0);
    assert_close(label.rect.size.width, 27.0);
    assert_close(label.rect.size.height, 30.0);
}

#[test]
fn text_measurer_cache_key_invalidates_cached_layout() {
    struct FixedTextMeasurer {
        key: TextMeasurerKey,
        width: f32,
    }

    impl TextMeasurer for FixedTextMeasurer {
        fn cache_key(&self) -> TextMeasurerKey {
            self.key
        }

        fn measure_text(&mut self, _request: TextLayoutRequest<'_>) -> TextLayoutResult {
            TextLayoutResult {
                size: Size::new(self.width, 18.0),
                line_count: 1,
                elided: false,
            }
        }
    }

    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new();
    let document = Document::build(Size::new(240.0, 160.0), |ui| {
        ui.text_element("label", ElementSpec::new(ElementRole::Text), "Text");
    });
    let mut narrow = FixedTextMeasurer {
        key: TextMeasurerKey::new("narrow"),
        width: 24.0,
    };
    let mut wide = FixedTextMeasurer {
        key: TextMeasurerKey::new("wide"),
        width: 96.0,
    };

    let first = engine.update_with_input_and_text_measurer(
        &document,
        &stylesheet,
        DocumentInput::default(),
        &mut narrow,
    );
    let second = engine.update_with_input_and_text_measurer(
        &document,
        &stylesheet,
        DocumentInput::default(),
        &mut wide,
    );

    assert_close(first.layout.find("label").unwrap().rect.size.width, 24.0);
    assert_close(second.layout.find("label").unwrap().rect.size.width, 96.0);
    assert!(!second.metrics.reused_cached_layout);
}

#[test]
fn selectable_text_tracks_pointer_selection_points() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new().rule(
        StyleSelector::id("label"),
        Style::default()
            .width(Length::Px(160.0))
            .text_wrap(TextWrapMode::Wrap),
    );
    let document = Document::build(Size::new(240.0, 160.0), |ui| {
        ui.text_element(
            "label",
            ElementSpec::new(ElementRole::Text).selectable_text(),
            "Customer analytics pipeline preview",
        );
    });

    let start = Point::new(4.0, 4.0);
    let end = Point::new(86.0, 24.0);
    engine.update_with_input(
        &document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: start,
                primary_delta: Point::ZERO,
                primary_down: true,
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: false,
                time_seconds: 0.0,
            }),
            scroll_delta: Point::ZERO,
        },
    );
    let dragging = engine.update_with_input(
        &document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: end,
                primary_delta: Point::new(end.x - start.x, end.y - start.y),
                primary_down: true,
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: false,
                time_seconds: 0.0,
            }),
            scroll_delta: Point::ZERO,
        },
    );

    let selection = dragging.text_selection.unwrap();
    assert_eq!(selection.target, ElementId::new("label"));
    assert_eq!(selection.anchor, start);
    assert_eq!(selection.focus, end);
    assert!(selection.focus_index > selection.anchor_index);
    assert!(
        selection
            .selected_text_from("Customer analytics pipeline preview")
            .is_some()
    );
    assert!(selection.active);

    let released = engine.update_with_input(
        &document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: end,
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

    assert!(!released.text_selection.unwrap().active);
}

#[test]
fn selectable_text_exposes_selected_text_for_copy() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new().rule(
        StyleSelector::id("label"),
        Style::default()
            .width(Length::Px(320.0))
            .text_wrap(TextWrapMode::Extend),
    );
    let document = Document::build(Size::new(360.0, 120.0), |ui| {
        ui.text_element(
            "label",
            ElementSpec::new(ElementRole::Text).selectable_text(),
            "Customer analytics",
        );
    });

    let start = Point::new(0.0, 4.0);
    let end = Point::new(60.0, 4.0);
    engine.update_with_input(
        &document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: start,
                primary_delta: Point::ZERO,
                primary_down: true,
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: false,
                time_seconds: 0.0,
            }),
            scroll_delta: Point::ZERO,
        },
    );
    let output = engine.update_with_input(
        &document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: end,
                primary_delta: Point::new(end.x - start.x, end.y - start.y),
                primary_down: true,
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: false,
                time_seconds: 0.0,
            }),
            scroll_delta: Point::ZERO,
        },
    );

    assert_eq!(output.text_selection.as_ref().unwrap().char_range(), 0..8);
    assert_eq!(output.selected_text().as_deref(), Some("Customer"));
    assert!(output.snapshot().find("label").unwrap().selectable_text());
    assert!(output.snapshot().find("label").unwrap().copyable_text());
}

#[test]
fn selectable_text_can_disable_copy_without_disabling_selection() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new().rule(
        StyleSelector::id("label"),
        Style::default()
            .width(Length::Px(320.0))
            .text_wrap(TextWrapMode::Extend),
    );
    let document = Document::build(Size::new(360.0, 120.0), |ui| {
        ui.text_element(
            "label",
            ElementSpec::new(ElementRole::Text)
                .selectable_text()
                .copyable_text(false),
            "Customer analytics",
        );
    });

    let start = Point::new(0.0, 4.0);
    let end = Point::new(60.0, 4.0);
    engine.update_with_input(
        &document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: start,
                primary_delta: Point::ZERO,
                primary_down: true,
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: false,
                time_seconds: 0.0,
            }),
            scroll_delta: Point::ZERO,
        },
    );
    let output = engine.update_with_input(
        &document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: end,
                primary_delta: Point::new(end.x - start.x, end.y - start.y),
                primary_down: true,
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: false,
                time_seconds: 0.0,
            }),
            scroll_delta: Point::ZERO,
        },
    );

    assert!(output.text_selection.is_some());
    assert!(output.snapshot().find("label").unwrap().selectable_text());
    assert!(!output.snapshot().find("label").unwrap().copyable_text());
    assert_eq!(output.selected_text(), None);
}

#[test]
fn selectable_text_double_click_selects_word_and_word_drags() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new().rule(
        StyleSelector::id("label"),
        Style::default()
            .width(Length::Px(320.0))
            .text_wrap(TextWrapMode::Extend),
    );
    let document = Document::build(Size::new(360.0, 120.0), |ui| {
        ui.text_element(
            "label",
            ElementSpec::new(ElementRole::Text).selectable_text(),
            "alpha beta gamma",
        );
    });
    let beta = Point::new(56.0, 4.0);
    let gamma = Point::new(100.0, 4.0);

    engine.update_with_input(
        &document,
        &stylesheet,
        pointer_input(beta, true, true, false, 0.0),
    );
    engine.update_with_input(
        &document,
        &stylesheet,
        pointer_input(beta, false, false, true, 0.05),
    );
    let word = engine.update_with_input(
        &document,
        &stylesheet,
        pointer_input(beta, true, true, false, 0.20),
    );

    assert_eq!(word.text_selection.as_ref().unwrap().char_range(), 6..10);
    assert_eq!(word.selected_text().as_deref(), Some("beta"));

    let multi_word = engine.update_with_input(
        &document,
        &stylesheet,
        pointer_input(gamma, true, false, false, 0.24),
    );

    assert_eq!(
        multi_word.text_selection.as_ref().unwrap().granularity,
        TextSelectionGranularity::Word
    );
    assert_eq!(multi_word.selected_text().as_deref(), Some("beta gamma"));
}

#[test]
fn selectable_text_double_click_drag_left_keeps_original_word() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new().rule(
        StyleSelector::id("label"),
        Style::default()
            .width(Length::Px(320.0))
            .text_wrap(TextWrapMode::Extend),
    );
    let document = Document::build(Size::new(360.0, 120.0), |ui| {
        ui.text_element(
            "label",
            ElementSpec::new(ElementRole::Text).selectable_text(),
            "alpha beta gamma",
        );
    });
    let beta = Point::new(56.0, 4.0);
    let alpha = Point::new(12.0, 4.0);

    engine.update_with_input(
        &document,
        &stylesheet,
        pointer_input(beta, true, true, false, 0.0),
    );
    engine.update_with_input(
        &document,
        &stylesheet,
        pointer_input(beta, false, false, true, 0.05),
    );
    engine.update_with_input(
        &document,
        &stylesheet,
        pointer_input(beta, true, true, false, 0.20),
    );
    let multi_word = engine.update_with_input(
        &document,
        &stylesheet,
        pointer_input(alpha, true, false, false, 0.24),
    );

    assert_eq!(
        multi_word.text_selection.as_ref().unwrap().granularity,
        TextSelectionGranularity::Word
    );
    assert_eq!(multi_word.selected_text().as_deref(), Some("alpha beta"));
}

#[test]
fn selectable_text_triple_click_selects_paragraph() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new().rule(
        StyleSelector::id("label"),
        Style::default()
            .width(Length::Px(320.0))
            .text_wrap(TextWrapMode::Wrap),
    );
    let document = Document::build(Size::new(360.0, 160.0), |ui| {
        ui.text_element(
            "label",
            ElementSpec::new(ElementRole::Text).selectable_text(),
            "first paragraph\nsecond paragraph",
        );
    });
    let second = Point::new(8.0, 24.0);

    for (index, time_seconds) in [0.0, 0.2, 0.4].into_iter().enumerate() {
        engine.update_with_input(
            &document,
            &stylesheet,
            pointer_input(second, true, true, false, time_seconds),
        );
        if index < 2 {
            engine.update_with_input(
                &document,
                &stylesheet,
                pointer_input(second, false, false, true, time_seconds + 0.05),
            );
        }
    }
    let output = engine.update(&document, &stylesheet);

    assert_eq!(
        output.text_selection.as_ref().unwrap().granularity,
        TextSelectionGranularity::Paragraph
    );
    assert_eq!(output.selected_text().as_deref(), Some("second paragraph"));
}

#[test]
fn selectable_text_secondary_click_requests_context() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new();
    let document = Document::build(Size::new(240.0, 120.0), |ui| {
        ui.text_element(
            "label",
            ElementSpec::new(ElementRole::Text).selectable_text(),
            "copy me",
        );
    });

    let output = engine.update_with_input(
        &document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(8.0, 4.0),
                primary_delta: Point::ZERO,
                primary_down: false,
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: true,
                time_seconds: 0.0,
            }),
            scroll_delta: Point::ZERO,
        },
    );

    assert!(output.events.iter().any(|event| {
        event.target == ElementId::new("label") && event.kind == DocumentEventKind::ContextRequested
    }));
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
fn absolute_anchor_positions_against_resolved_element_rect() {
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
    let document = Document::build(Size::new(320.0, 200.0), |ui| {
        ui.element("panel", ElementSpec::new(ElementRole::Panel), |ui| {
            ui.element("popover", ElementSpec::new(ElementRole::Card), |_| {});
            ui.element("anchor", ElementSpec::new(ElementRole::Card), |_| {});
        });
    });

    let output = engine.update(&document, &stylesheet);
    let panel = output.layout.find("panel").unwrap();
    let anchor = output.layout.find("anchor").unwrap();
    let popover = output.layout.find("popover").unwrap();

    assert_eq!(panel.rect.size, Size::new(104.0, 54.0));
    assert_eq!(anchor.rect.origin, Point::new(12.0, 12.0));
    assert_eq!(popover.rect.origin, Point::new(12.0, 41.0));
    assert_eq!(popover.rect.size, Size::new(60.0, 20.0));
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
                primary_pressed: false,
                primary_clicked: true,
                primary_click_count: 1,
                secondary_clicked: false,
                time_seconds: 0.0,
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
fn pointer_input_emits_document_interaction_events() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new().rule(
        StyleSelector::Role(ElementRole::Card),
        Style::default().size(100.0, 40.0),
    );
    let document = Document::build(Size::new(320.0, 200.0), |ui| {
        ui.element(
            "card",
            ElementSpec::new(ElementRole::Card).interactive(),
            |_| {},
        );
    });

    let output = engine.update_with_input(
        &document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(20.0, 20.0),
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

    assert_eq!(output.hit_id, Some(ElementId::new("card")));
    assert!(
        output
            .events
            .contains(&DocumentEvent::pointer_entered("card"))
    );
    assert!(output.events.contains(&DocumentEvent::pressed("card")));
    assert!(output.events.contains(&DocumentEvent::clicked("card")));

    let output = engine.update_with_input(
        &document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(20.0, 20.0),
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

    assert!(output.events.contains(&DocumentEvent::released("card")));

    let output = engine.update_with_input(
        &document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(180.0, 120.0),
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

    assert!(
        output
            .events
            .contains(&DocumentEvent::pointer_exited("card"))
    );
}

#[test]
fn document_engine_captures_primary_pointer_drag() {
    let mut engine = DocumentEngine::default();
    let stylesheet = StyleSheet::new().rule(
        StyleSelector::id("card"),
        Style::default().size(100.0, 40.0),
    );
    let document = Document::build(Size::new(320.0, 200.0), |ui| {
        ui.element(
            "card",
            ElementSpec::new(ElementRole::Card).interactive(),
            |_| {},
        );
    });

    let output = engine.update_with_input(
        &document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(12.0, 10.0),
                primary_delta: Point::ZERO,
                primary_down: true,
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: false,
                time_seconds: 0.0,
            }),
            scroll_delta: Point::ZERO,
        },
    );

    assert!(
        output.active_drag.is_none(),
        "pointer down should capture a pending drag without activating it"
    );
    assert!(output.completed_drag.is_none());
    assert!(!output.events.contains(&DocumentEvent::drag_started("card")));
    assert!(!engine.element_state("card").unwrap().dragging);

    let output = engine.update_with_input(
        &document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(180.0, 120.0),
                primary_delta: Point::new(168.0, 110.0),
                primary_down: true,
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: false,
                time_seconds: 0.0,
            }),
            scroll_delta: Point::ZERO,
        },
    );

    let drag = output
        .active_drag
        .expect("movement past activation distance should start drag");
    assert_eq!(drag.target, ElementId::new("card"));
    assert_eq!(drag.origin, Point::new(12.0, 10.0));
    assert_eq!(drag.current, Point::new(180.0, 120.0));
    assert_eq!(drag.delta, Point::new(168.0, 110.0));
    assert_eq!(drag.pointer_offset, Point::new(12.0, 10.0));
    assert!(output.completed_drag.is_none());
    assert_eq!(output.hit_id, Some(ElementId::new("card")));
    assert!(output.events.contains(&DocumentEvent::drag_started("card")));

    let output = engine.update_with_input(
        &document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(180.0, 120.0),
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

    let completed = output
        .completed_drag
        .expect("release should expose completed drag");
    assert!(output.active_drag.is_none());
    assert_eq!(completed.target, ElementId::new("card"));
    assert_eq!(completed.current, Point::new(180.0, 120.0));
    assert_eq!(completed.delta, Point::new(168.0, 110.0));
    assert!(output.events.contains(&DocumentEvent::drag_ended("card")));
    assert!(!engine.element_state("card").unwrap().dragging);
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
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: false,
                time_seconds: 0.0,
            }),
            scroll_delta: Point::new(0.0, -24.0),
        },
    );

    assert_eq!(output.hit_id, Some(ElementId::new("row-0")));
    assert!(output.events.contains(&DocumentEvent::scrolled(
        "scroll-panel",
        ScrollAxis::Vertical
    )));
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
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: false,
                time_seconds: 0.0,
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
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: false,
                time_seconds: 0.0,
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
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: false,
                time_seconds: 0.0,
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
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: false,
                time_seconds: 0.0,
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
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: false,
                time_seconds: 0.0,
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
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: false,
                time_seconds: 0.0,
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
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: false,
                time_seconds: 0.0,
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
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: false,
                time_seconds: 0.0,
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
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: false,
                time_seconds: 0.0,
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
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: false,
                time_seconds: 0.0,
            }),
            scroll_delta: Point::ZERO,
        },
    );

    assert!(engine.element_state("scroll-panel").unwrap().scroll_y > 0.0);
}

#[test]
fn active_document_drag_is_not_stolen_by_scrollbar_hitbox() {
    let mut engine = DocumentEngine::default();
    let stylesheet = scroll_fixture_stylesheet(80.0).rule(
        StyleSelector::id("drag-source"),
        Style::default().size(80.0, 32.0),
    );
    let document = Document::build(Size::new(240.0, 160.0), |ui| {
        ui.element(
            "drag-source",
            ElementSpec::new(ElementRole::Card).interactive(),
            |_| {},
        );
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

    let output = engine.update(&document, &stylesheet);
    let chrome = output
        .scroll_chrome
        .iter()
        .find(|chrome| chrome.element_id == ElementId::new("scroll-panel"))
        .expect("overflowing panel should emit scroll chrome");
    let scrollbar_point = Point::new(
        chrome.hit_rect.origin.x + chrome.hit_rect.size.width / 2.0,
        chrome.hit_rect.origin.y + chrome.hit_rect.size.height / 2.0,
    );

    engine.update_with_input(
        &document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(20.0, 16.0),
                primary_delta: Point::ZERO,
                primary_down: true,
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: false,
                time_seconds: 0.0,
            }),
            scroll_delta: Point::ZERO,
        },
    );
    let output = engine.update_with_input(
        &document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: scrollbar_point,
                primary_delta: Point::new(scrollbar_point.x - 20.0, scrollbar_point.y - 16.0),
                primary_down: true,
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
        output.hit_id.as_ref().map(|id| id.as_str()),
        Some("drag-source")
    );
    assert!(
        output
            .active_drag
            .as_ref()
            .is_some_and(|drag| drag.target == ElementId::new("drag-source")),
        "document drags should continue even while the pointer crosses scrollbar hitboxes"
    );
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
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: false,
                time_seconds: 0.0,
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
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: false,
                time_seconds: 0.0,
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
                primary_pressed: false,
                primary_clicked: true,
                primary_click_count: 1,
                secondary_clicked: false,
                time_seconds: 0.0,
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

fn table_fixture_document() -> Document {
    let table = TableSpec::new(vec![
        TableColumnSpec::new("customer", "Customer").width(TableTrackSize::px(120.0)),
        TableColumnSpec::new("country", "Country").width(TableTrackSize::px(100.0)),
        TableColumnSpec::new("orders", "Orders").width(TableTrackSize::px(80.0)),
    ])
    .header_height(28.0)
    .row_height(26.0);

    Document::build(Size::new(320.0, 220.0), |ui| {
        ui.element(
            "customers",
            ElementSpec::new(ElementRole::Table).table(table),
            |ui| {
                ui.element(
                    "customers-header",
                    ElementSpec::new(ElementRole::TableHeader),
                    |ui| {
                        table_cell(ui, "customers-header-customer", "customer", "Customer");
                        table_cell(ui, "customers-header-country", "country", "Country");
                        table_cell(ui, "customers-header-orders", "orders", "Orders");
                    },
                );
                ui.element(
                    "customers-row-0",
                    ElementSpec::new(ElementRole::TableRow),
                    |ui| {
                        table_cell(ui, "customers-row-0-customer", "customer", "Acme");
                        table_cell(ui, "customers-row-0-country", "country", "US");
                        table_cell(ui, "customers-row-0-orders", "orders", "42");
                    },
                );
            },
        );
    })
}

fn table_cell(
    ui: &mut des_ui_document::DocumentBuilder,
    id: &'static str,
    column_id: &'static str,
    text: &'static str,
) {
    ui.text_element(
        id,
        ElementSpec::new(ElementRole::TableCell).table_cell(TableCellSpec::new(column_id)),
        text,
    );
}
