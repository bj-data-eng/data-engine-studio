//! Product-specific UI runtime primitives.
//!
//! `des-ui-runtime` owns the DOM-like element tree, deterministic style
//! resolution, retained interaction state, layout frames, and input routing.
//! Rendering hosts such as egui should translate platform input into
//! [`RuntimeInput`] and paint [`RuntimeOutput::layout`].

mod animation;
mod element;
mod geometry;
mod layout;
mod runtime;
mod scroll;
mod state;
mod style;

pub use element::{
    ClassName, Color, Element, ElementId, ElementRole, ElementSpec, ElementStateSelector, Scene, Ui,
};
pub use geometry::{Direction, Insets, Length, Overflow, Point, Rect, Size};
pub use runtime::Runtime;
pub use state::{
    ChangeSet, ElementState, LayoutFrame, PointerInput, RuntimeInput, RuntimeOutput, ScrollChrome,
};
pub use style::{
    ComputedStyle, Easing, StylePatch, StyleRule, StyleSelector, StyleSheet, Transition,
};
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
                    primary_delta: Point::ZERO,
                    primary_down: false,
                    primary_clicked: false,
                }),
                scroll_delta: Point::ZERO,
            },
        );
        let card = output.layout.find("card").unwrap();

        assert_eq!(card.style.background, Some(Color::rgb(40, 70, 95)));
        assert_eq!(card.style.radius, 7.0);
    }

    #[test]
    fn transitioned_state_rules_ease_visual_style_properties() {
        let mut runtime = Runtime::default();
        let stylesheet = StyleSheet::new()
            .rule(
                StyleSelector::Role(ElementRole::Card),
                StylePatch::default()
                    .size(100.0, 40.0)
                    .background(Color::rgb(20, 20, 20))
                    .transition(Transition::ease_out(0.24)),
            )
            .rule(
                StyleSelector::State(ElementStateSelector::Hovered),
                StylePatch::default().background(Color::rgb(40, 70, 95)),
            );
        let scene = Scene::build(Size::new(320.0, 200.0), |ui| {
            ui.element(
                "card",
                ElementSpec::new(ElementRole::Card).interactive(),
                |_| {},
            );
        });

        runtime.update(&scene, &stylesheet);

        let output = runtime.update_with_input(
            &scene,
            &stylesheet,
            RuntimeInput {
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
                runtime.update_with_input(
                    &scene,
                    &stylesheet,
                    RuntimeInput {
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
    fn fill_width_uses_parent_content_width_after_box_model() {
        let mut runtime = Runtime::default();
        let stylesheet = StyleSheet::new()
            .rule(
                StyleSelector::Id("panel"),
                StylePatch::default()
                    .size(200.0, 120.0)
                    .border_width(2.0)
                    .padding(Insets::symmetric(12.0, 8.0)),
            )
            .rule(
                StyleSelector::Id("row"),
                StylePatch::default()
                    .width_fill()
                    .height(Length::Px(24.0))
                    .margin(Insets::symmetric(3.0, 0.0)),
            );
        let scene = Scene::build(Size::new(320.0, 200.0), |ui| {
            ui.element("panel", ElementSpec::new(ElementRole::Panel), |ui| {
                ui.element("row", ElementSpec::new(ElementRole::Card), |_| {});
            });
        });

        let output = runtime.update(&scene, &stylesheet);
        let row = output.layout.find("row").unwrap();

        assert_eq!(row.rect.origin, Point::new(17.0, 10.0));
        assert_eq!(row.rect.size, Size::new(166.0, 24.0));
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
                    primary_delta: Point::ZERO,
                    primary_down: true,
                    primary_clicked: true,
                }),
                scroll_delta: Point::ZERO,
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

    #[test]
    fn scroll_delta_updates_hovered_scroll_container_state() {
        let mut runtime = Runtime::default();
        let stylesheet = StyleSheet::new()
            .rule(
                StyleSelector::Id("scroll-panel"),
                StylePatch::default()
                    .size(180.0, 80.0)
                    .padding(Insets::all(8.0))
                    .gap(4.0)
                    .border_width(5.0)
                    .overflow_y(Overflow::Scroll),
            )
            .rule(
                StyleSelector::Role(ElementRole::Card),
                StylePatch::default().size(140.0, 32.0),
            );
        let scene = Scene::build(Size::new(240.0, 160.0), |ui| {
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

        let output = runtime.update_with_input(
            &scene,
            &stylesheet,
            RuntimeInput {
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
        assert_eq!(
            runtime.element_state("scroll-panel").unwrap().scroll_y,
            24.0
        );

        let output = runtime.update(&scene, &stylesheet);
        let first_row = output.layout.find("row-0").unwrap();
        assert_eq!(first_row.rect.origin.y, -11.0);
    }

    #[test]
    fn scroll_delta_is_clamped_when_content_does_not_overflow() {
        let mut runtime = Runtime::default();
        let stylesheet = StyleSheet::new()
            .rule(
                StyleSelector::Id("scroll-panel"),
                StylePatch::default()
                    .size(180.0, 120.0)
                    .padding(Insets::all(8.0))
                    .gap(4.0)
                    .overflow_y(Overflow::Scroll),
            )
            .rule(
                StyleSelector::Role(ElementRole::Card),
                StylePatch::default().size(140.0, 32.0),
            );
        let scene = Scene::build(Size::new(240.0, 160.0), |ui| {
            ui.element("scroll-panel", ElementSpec::new(ElementRole::Panel), |ui| {
                ui.element("row-0", ElementSpec::new(ElementRole::Card), |_| {});
                ui.element("row-1", ElementSpec::new(ElementRole::Card), |_| {});
            });
        });

        runtime.update_with_input(
            &scene,
            &stylesheet,
            RuntimeInput {
                pointer: Some(PointerInput {
                    position: Point::new(20.0, 20.0),
                    primary_delta: Point::ZERO,
                    primary_down: false,
                    primary_clicked: false,
                }),
                scroll_delta: Point::new(0.0, -240.0),
            },
        );

        assert_eq!(runtime.element_state("scroll-panel").unwrap().scroll_y, 0.0);
    }

    #[test]
    fn overflow_scroll_container_emits_draggable_scroll_chrome() {
        let mut runtime = Runtime::default();
        let stylesheet = StyleSheet::new()
            .rule(
                StyleSelector::Id("scroll-panel"),
                StylePatch::default()
                    .size(180.0, 80.0)
                    .padding(Insets::all(8.0))
                    .gap(4.0)
                    .overflow_y(Overflow::Scroll),
            )
            .rule(
                StyleSelector::Role(ElementRole::Card),
                StylePatch::default().size(140.0, 32.0),
            );
        let scene = Scene::build(Size::new(240.0, 160.0), |ui| {
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

        let output = runtime.update(&scene, &stylesheet);
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
        runtime.update_with_input(
            &scene,
            &stylesheet,
            RuntimeInput {
                pointer: Some(PointerInput {
                    position: grab,
                    primary_delta: Point::ZERO,
                    primary_down: true,
                    primary_clicked: false,
                }),
                scroll_delta: Point::ZERO,
            },
        );
        runtime.update_with_input(
            &scene,
            &stylesheet,
            RuntimeInput {
                pointer: Some(PointerInput {
                    position: Point::new(grab.x, grab.y + 24.0),
                    primary_delta: Point::new(0.0, 24.0),
                    primary_down: true,
                    primary_clicked: false,
                }),
                scroll_delta: Point::ZERO,
            },
        );

        assert!(runtime.element_state("scroll-panel").unwrap().scroll_y > 0.0);
    }

    #[test]
    fn scroll_chrome_appears_on_container_hover_and_expands_on_hit_strip() {
        let mut runtime = Runtime::default();
        let stylesheet = StyleSheet::new()
            .rule(
                StyleSelector::Id("scroll-panel"),
                StylePatch::default()
                    .size(180.0, 80.0)
                    .padding(Insets::all(8.0))
                    .gap(4.0)
                    .overflow_y(Overflow::Scroll),
            )
            .rule(
                StyleSelector::Role(ElementRole::Card),
                StylePatch::default().size(140.0, 32.0),
            );
        let scene = Scene::build(Size::new(240.0, 160.0), |ui| {
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

        let output = runtime.update_with_input(
            &scene,
            &stylesheet,
            RuntimeInput {
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

        let output = runtime.update_with_input(
            &scene,
            &stylesheet,
            RuntimeInput {
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
                    .size(180.0, 40.0)
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
