use super::super::{CARD, PANEL_ALT, SHADOW_COLOR, STROKE, STROKE_SELECTED, TEXT_MUTED};
use des_ui_document::{
    AlignItems, Color, Insets, JustifyContent, Length, Overflow, Point, Shadow, Style,
    StyleSelector, StyleSheet, Transition,
};

pub(super) fn stylesheet() -> StyleSheet {
    StyleSheet::new()
        .rule(
            StyleSelector::class("drag-item"),
            Style::default()
                .flex_direction(des_ui_document::FlexDirection::Row)
                .width_fill()
                .height(Length::Px(34.0))
                .padding(Insets::symmetric(9.0, 6.0))
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::SpaceBetween)
                .background(CARD)
                .border(STROKE)
                .radius(6.0)
                .shadows(drag_rest_shadow())
                .transition(Transition::ease_out(0.14)),
        )
        .rule(
            StyleSelector::class("drag-origin-space"),
            Style::default()
                .background(Color::rgba(0, 0, 0, 0))
                .border(Color::rgba(0, 0, 0, 0))
                .text_color(Color::rgba(0, 0, 0, 0))
                .shadows(Vec::new())
                .animate_paint(false)
                .animate_shadows(false),
        )
        .rule(
            StyleSelector::class("drag-origin-collapsed"),
            Style::default()
                .height(Length::Px(0.0))
                .padding(Insets::ZERO)
                .border_width(0.0)
                .margin(Insets {
                    top: 0.0,
                    right: 0.0,
                    bottom: -5.0,
                    left: 0.0,
                }),
        )
        .rule(
            StyleSelector::class("drag-handle"),
            Style::default()
                .size(24.0, 22.0)
                .padding(Insets::symmetric(4.0, 2.0))
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::Center)
                .background(Color::rgba(0, 0, 0, 0))
                .border(Color::rgba(0, 0, 0, 0))
                .radius(3.0)
                .transition(Transition::ease_out(0.1)),
        )
        .rule(
            StyleSelector::class_state(
                "drag-handle",
                des_ui_document::ElementStateSelector::Hovered,
            ),
            Style::default().background(Color::rgba(103, 80, 164, 20)),
        )
        .rule(
            StyleSelector::class("drag-handle-glyph"),
            Style::default().font_size(12.0).text_color(TEXT_MUTED),
        )
        .rule(
            StyleSelector::class("drag-origin-content"),
            Style::default()
                .background(Color::rgba(0, 0, 0, 0))
                .border(Color::rgba(0, 0, 0, 0))
                .text_color(Color::rgba(0, 0, 0, 0)),
        )
        .rule(
            StyleSelector::class_state("drag-item", des_ui_document::ElementStateSelector::Hovered),
            Style::default()
                .background(CARD)
                .shadows(drag_hover_shadow()),
        )
        .rule(
            StyleSelector::class_state("drag-item", des_ui_document::ElementStateSelector::Pressed),
            Style::default()
                .background(CARD)
                .shadows(drag_hover_shadow()),
        )
        .rule(
            StyleSelector::class("drag-handle-pressed"),
            Style::default()
                .background(CARD)
                .shadows(drag_hover_shadow()),
        )
        .rule(
            StyleSelector::class_state(
                "drag-origin-space",
                des_ui_document::ElementStateSelector::Hovered,
            ),
            transparent_surface(),
        )
        .rule(
            StyleSelector::class_state(
                "drag-origin-space",
                des_ui_document::ElementStateSelector::Pressed,
            ),
            transparent_surface(),
        )
        .rule(
            StyleSelector::class("drag-overlay"),
            Style::default()
                .z_index(1000)
                .shadows(drag_hover_shadow())
                .animate_size(false)
                .transition(Transition::ease_out(0.18)),
        )
        .rule(
            StyleSelector::class("drag-overlay-idle"),
            Style::default()
                .absolute_viewport()
                .left(Length::Px(-10_000.0))
                .top(Length::Px(-10_000.0))
                .z_index(-1)
                .background(Color::rgba(0, 0, 0, 0))
                .border(Color::rgba(0, 0, 0, 0))
                .shadows(Vec::new()),
        )
        .rule(
            StyleSelector::class("drag-gap-before"),
            Style::default().margin(Insets {
                top: 39.0,
                right: 0.0,
                bottom: 0.0,
                left: 0.0,
            }),
        )
        .rule(
            StyleSelector::class("drag-gap-after"),
            Style::default().margin(Insets {
                top: 0.0,
                right: 0.0,
                bottom: 39.0,
                left: 0.0,
            }),
        )
        .rule(
            StyleSelector::class("drag-scroll-list-card"),
            Style::default()
                .width(Length::Px(410.0))
                .padding(Insets::all(10.0))
                .gap(8.0)
                .background(PANEL_ALT)
                .border(STROKE)
                .radius(6.0),
        )
        .rule(
            StyleSelector::class("drag-scroll-list"),
            Style::default()
                .width_fill()
                .height(Length::Px(126.0))
                .padding(Insets::all(6.0))
                .gap(5.0)
                .overflow_y(Overflow::Scroll)
                .scrollbar_width(2.0)
                .scrollbar_expanded_width(10.0)
                .scrollbar_radius(5.0)
                .scrollbar_handle_color(Color::rgba(103, 80, 164, 118))
                .scrollbar_track_color(Color::rgba(103, 80, 164, 28))
                .scrollbar_hover_handle_color(Color::rgba(103, 80, 164, 118))
                .scrollbar_hover_track_color(Color::rgba(103, 80, 164, 28))
                .scrollbar_pressed_handle_color(Color::rgba(103, 80, 164, 176))
                .scrollbar_pressed_handle_border_color(STROKE_SELECTED)
                .scrollbar_pressed_handle_border_width(1.0)
                .transition(Transition::ease_out(0.12)),
        )
}

fn transparent_surface() -> Style {
    Style::default()
        .background(Color::rgba(0, 0, 0, 0))
        .border(Color::rgba(0, 0, 0, 0))
        .shadows(Vec::new())
}

fn drag_rest_shadow() -> Vec<Shadow> {
    single_shadow(SHADOW_COLOR, 0.0, 0.0, 7.0 * 0.55, -7.0 * 0.55, 80)
}

fn drag_hover_shadow() -> Vec<Shadow> {
    single_shadow(SHADOW_COLOR, 0.0, 5.0 * 0.55, 20.0 * 0.55, -15.0 * 0.55, 80)
}

fn single_shadow(color: Color, x: f32, y: f32, blur: f32, spread: f32, alpha: u8) -> Vec<Shadow> {
    vec![Shadow {
        offset: Point::new(x, y),
        blur,
        spread,
        color: Color { a: alpha, ..color },
    }]
}
