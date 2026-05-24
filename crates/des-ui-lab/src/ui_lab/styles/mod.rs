mod framework;

use super::{
    GREEN, PANEL, PRIMARY_CONTAINER, PURPLE, SECONDARY_CONTAINER, SHADOW_COLOR, STROKE,
    STROKE_SELECTED, SURFACE_CONTAINER, TEXT, TEXT_ACCENT, TEXT_MUTED,
};
use des_document::{
    Color, ElementStateSelector, FlexDirection, FloatingAxisOffset, FloatingPlacement,
    FloatingShift, Insets, JustifyContent, Length, Point, Shadow, Style, StyleSelector, StyleSheet,
    TextWrapMode, Transition, ViewportQuery,
};

const DRAGGABLE_STACK_VIEWPORT_WIDTH: f32 = 1268.0;
const LAB_CSS: &str = include_str!("lab.css");

pub(super) fn stylesheet() -> StyleSheet {
    let mut stylesheet = lab_stylesheet();
    stylesheet.extend(framework::stylesheet());
    push_responsive_styles(&mut stylesheet);
    stylesheet
}

fn lab_stylesheet() -> StyleSheet {
    let mut stylesheet = StyleSheet::parse_css(LAB_CSS).expect("lab CSS stylesheet is valid");
    stylesheet.push_rule(
        StyleSelector::class("dropdown-menu"),
        Style::default()
            .absolute_parent()
            .anchor_bottom_start("control-dropdown", 0.0, -1.0)
            .z_index(20)
            .width_fill()
            .height(Length::Auto)
            .padding(Insets::all(4.0))
            .gap(4.0)
            .background(PANEL)
            .border(STROKE_SELECTED)
            .top_left_radius(0.0)
            .top_right_radius(0.0)
            .bottom_left_radius(5.0)
            .bottom_right_radius(5.0),
    );
    stylesheet.push_rule(
        styled_scrollbar_selector().selector(),
        styled_scrollbar_style(),
    );
    stylesheet.push_rule(
        StyleSelector::id("floating-offset-zero-popover"),
        Style::default()
            .floating_to("floating-offset-zero-reference")
            .floating_placement(FloatingPlacement::Bottom)
            .floating_offset(0.0, 0.0),
    );
    stylesheet.push_rule(
        StyleSelector::id("floating-offset-ten-popover"),
        Style::default()
            .floating_to("floating-offset-ten-reference")
            .floating_placement(FloatingPlacement::Bottom)
            .floating_offset(10.0, 0.0),
    );
    stylesheet.push_rule(
        StyleSelector::id("floating-main-axis-top-popover"),
        Style::default()
            .floating_to("floating-main-axis-top-reference")
            .floating_placement(FloatingPlacement::Top),
    );
    stylesheet.push_rule(
        StyleSelector::id("floating-main-axis-bottom-popover"),
        Style::default()
            .floating_to("floating-main-axis-bottom-reference")
            .floating_placement(FloatingPlacement::Bottom),
    );
    stylesheet.push_rule(
        StyleSelector::id("floating-main-axis-left-popover"),
        Style::default()
            .floating_to("floating-main-axis-left-reference")
            .floating_placement(FloatingPlacement::Left),
    );
    stylesheet.push_rule(
        StyleSelector::id("floating-main-axis-right-popover"),
        Style::default()
            .floating_to("floating-main-axis-right-reference")
            .floating_placement(FloatingPlacement::Right),
    );
    stylesheet.push_rule(
        StyleSelector::id("floating-cross-axis-top-popover"),
        Style::default()
            .floating_to("floating-cross-axis-top-reference")
            .floating_placement(FloatingPlacement::TopEnd),
    );
    stylesheet.push_rule(
        StyleSelector::id("floating-cross-axis-bottom-popover"),
        Style::default()
            .floating_to("floating-cross-axis-bottom-reference")
            .floating_placement(FloatingPlacement::BottomEnd),
    );
    stylesheet.push_rule(
        StyleSelector::id("floating-cross-axis-left-popover"),
        Style::default()
            .floating_to("floating-cross-axis-left-reference")
            .floating_placement(FloatingPlacement::LeftEnd),
    );
    stylesheet.push_rule(
        StyleSelector::id("floating-cross-axis-right-popover"),
        Style::default()
            .floating_to("floating-cross-axis-right-reference")
            .floating_placement(FloatingPlacement::RightEnd),
    );
    stylesheet.push_rule(
        StyleSelector::id("floating-alignment-axis-cross-start-popover"),
        Style::default()
            .floating_to("floating-alignment-axis-cross-start-reference")
            .floating_placement(FloatingPlacement::TopStart)
            .floating_offset(0.0, 8.0),
    );
    stylesheet.push_rule(
        StyleSelector::id("floating-alignment-axis-cross-end-popover"),
        Style::default()
            .floating_to("floating-alignment-axis-cross-end-reference")
            .floating_placement(FloatingPlacement::TopEnd)
            .floating_offset(0.0, 8.0),
    );
    stylesheet.push_rule(
        StyleSelector::id("floating-alignment-axis-start-popover"),
        Style::default()
            .floating_to("floating-alignment-axis-start-reference")
            .floating_placement(FloatingPlacement::TopStart)
            .floating_alignment_axis(8.0),
    );
    stylesheet.push_rule(
        StyleSelector::id("floating-alignment-axis-end-popover"),
        Style::default()
            .floating_to("floating-alignment-axis-end-reference")
            .floating_placement(FloatingPlacement::TopEnd)
            .floating_alignment_axis(8.0),
    );
    stylesheet.push_rule(
        StyleSelector::id("floating-centered-axis-popover"),
        Style::default()
            .floating_to("floating-centered-axis-reference")
            .floating_placement(FloatingPlacement::Center),
    );
    stylesheet.push_rule(
        StyleSelector::id("floating-top-start-popover"),
        Style::default()
            .floating_to("floating-top-start-reference")
            .floating_placement(FloatingPlacement::TopStart)
            .floating_alignment_axis_offset(FloatingAxisOffset::floating_width(-1.0)),
    );
    stylesheet.push_rule(
        StyleSelector::id("floating-scroll-shift-popover"),
        Style::default()
            .floating_to("floating-scroll-shift-reference")
            .floating_placement(FloatingPlacement::Bottom)
            .floating_boundary_to("floating-scroll-shift-panel")
            .floating_shift(FloatingShift::new(false, true)),
    );
    stylesheet.push_rule(
        StyleSelector::id("floating-scroll-attach-popover"),
        Style::default()
            .floating_to("floating-scroll-attach-reference")
            .floating_placement(FloatingPlacement::Bottom),
    );
    stylesheet.push_rule(
        StyleSelector::id("floating-vertical-overlap-popover"),
        Style::default()
            .floating_to("floating-vertical-overlap-reference")
            .floating_placement(FloatingPlacement::Top)
            .floating_boundary_to("floating-vertical-overlap-panel")
            .floating_shift(FloatingShift::new(true, false)),
    );
    stylesheet.push_rule(
        StyleSelector::id("floating-vertical-flip-popover"),
        Style::default()
            .floating_to("floating-vertical-flip-reference")
            .floating_placement(FloatingPlacement::Bottom)
            .floating_boundary_to("floating-vertical-flip-panel")
            .floating_flip(true),
    );
    stylesheet.push_rule(
        StyleSelector::id("floating-edge-flip-popover"),
        Style::default()
            .floating_to("floating-edge-flip-reference")
            .floating_placement(FloatingPlacement::BottomStart)
            .floating_boundary_to("floating-edge-flip-panel")
            .floating_flip(true),
    );
    stylesheet.push_rule(
        StyleSelector::class("nest-outer"),
        Style::default()
            .size(650.0, 430.0)
            .padding(Insets::all(28.0))
            .gap(16.0)
            .background(SURFACE_CONTAINER)
            .border(STROKE)
            .radius(8.0),
    );
    stylesheet.push_rule(
        StyleSelector::class("nest-middle"),
        Style::default()
            .size(500.0, 270.0)
            .padding(Insets::all(24.0))
            .gap(14.0)
            .background(PRIMARY_CONTAINER)
            .border(STROKE_SELECTED)
            .radius(7.0),
    );
    stylesheet.push_rule(
        StyleSelector::class("nest-inner"),
        Style::default()
            .size(360.0, 130.0)
            .padding(Insets::all(18.0))
            .gap(6.0)
            .background(SECONDARY_CONTAINER)
            .border(PURPLE)
            .radius(7.0),
    );
    stylesheet.push_rule(
        StyleSelector::class_state("nest-inner", ElementStateSelector::Hovered),
        Style::default()
            .background(SECONDARY_CONTAINER)
            .border(TEXT_ACCENT),
    );
    stylesheet.push_rule(
        StyleSelector::class("canvas-placeholder"),
        Style::default()
            .size(720.0, 360.0)
            .padding(Insets::all(18.0))
            .gap(8.0)
            .background(PANEL)
            .border(STROKE)
            .radius(7.0),
    );
    stylesheet.push_rule(
        StyleSelector::class("debug-overlay-root"),
        Style::default()
            .size(1320.0, 780.0)
            .background(Color::rgba(0, 0, 0, 0)),
    );
    stylesheet.push_rule(
        StyleSelector::class("debug-overlay"),
        Style::default()
            .absolute_viewport()
            .left(Length::Px(1042.0))
            .top(Length::Px(12.0))
            .width(Length::Px(264.0))
            .height(Length::Auto)
            .padding(Insets::symmetric(12.0, 10.0))
            .gap(5.0)
            .background(Color::rgba(255, 251, 254, 232))
            .border(STROKE)
            .radius(6.0)
            .z_index(2000)
            .shadows(web_elevation(1, SHADOW_COLOR)),
    );
    stylesheet.push_rule(
        StyleSelector::class("debug-overlay-title"),
        Style::default()
            .font_size(14.0)
            .text_color(TEXT)
            .height(Length::Px(18.0)),
    );
    stylesheet.push_rule(
        StyleSelector::class("debug-row"),
        Style::default()
            .flex_direction(FlexDirection::Row)
            .justify_content(JustifyContent::SpaceBetween)
            .width_fill()
            .height(Length::Px(18.0))
            .background(Color::rgba(0, 0, 0, 0)),
    );
    stylesheet.push_rule(
        StyleSelector::class("debug-label"),
        Style::default()
            .font_size(12.0)
            .text_color(TEXT_MUTED)
            .height(Length::Px(16.0)),
    );
    stylesheet.push_rule(
        StyleSelector::class("debug-value"),
        Style::default()
            .font_size(12.0)
            .text_color(TEXT)
            .height(Length::Px(16.0)),
    );
    stylesheet.push_rule(
        StyleSelector::class("title"),
        Style::default().font_size(21.0).text_color(TEXT),
    );
    stylesheet.push_rule(
        StyleSelector::class("heading"),
        Style::default().font_size(24.0).text_color(TEXT),
    );
    stylesheet.push_rule(
        StyleSelector::class("section-title"),
        Style::default()
            .width_fill()
            .height(Length::Auto)
            .font_size(13.0)
            .text_wrap_mode(TextWrapMode::Wrap)
            .text_color(TEXT_ACCENT),
    );
    stylesheet.push_rule(
        StyleSelector::class("card-title"),
        Style::default()
            .width_fill()
            .height(Length::Auto)
            .font_size(16.0)
            .line_height(18.0)
            .text_wrap_mode(TextWrapMode::Wrap)
            .text_color(TEXT),
    );
    stylesheet.push_rule(
        StyleSelector::class("muted"),
        Style::default().font_size(12.5).text_color(TEXT_MUTED),
    );
    stylesheet.push_rule(
        StyleSelector::id_state("interaction-card-two", ElementStateSelector::Hovered),
        Style::default()
            .border(GREEN)
            .transition(Transition::ease_out(0.24)),
    );
    stylesheet.push_rule(
        StyleSelector::id("interaction-card-three"),
        Style::default().transition(Transition::ease_out(0.06)),
    );
    stylesheet.push_rule(
        StyleSelector::id_state("interaction-card-three", ElementStateSelector::Pressed),
        Style::default()
            .background(SECONDARY_CONTAINER)
            .border(PURPLE),
    );
    stylesheet
}

fn push_responsive_styles(stylesheet: &mut StyleSheet) {
    stylesheet.push_viewport_rule(
        ViewportQuery::max_width(DRAGGABLE_STACK_VIEWPORT_WIDTH),
        StyleSelector::class("drag-workbench"),
        Style::default().flex_direction(FlexDirection::Column),
    );
    stylesheet.push_viewport_rule(
        ViewportQuery::max_width(DRAGGABLE_STACK_VIEWPORT_WIDTH),
        StyleSelector::class("drag-scroll-list-card"),
        Style::default()
            .width_fill()
            .flex_basis(Length::Auto)
            .flex_grow(0.0),
    );
    stylesheet.push_viewport_rule(
        ViewportQuery::max_width(DRAGGABLE_STACK_VIEWPORT_WIDTH),
        StyleSelector::class("drag-grid"),
        Style::default()
            .width_fill()
            .flex_basis(Length::Auto)
            .flex_grow(0.0),
    );
}

fn styled_scrollbar_selector() -> des_document::CompoundSelector {
    StyleSelector::compound().class("styled-scrollbar")
}

fn styled_scrollbar_style() -> Style {
    Style::default()
        .scrollbar_handle_color(Color::rgba(103, 80, 164, 118))
        .scrollbar_track_color(Color::rgba(103, 80, 164, 28))
        .scrollbar_width(2.0)
        .scrollbar_expanded_width(10.0)
        .scrollbar_hover_track_color(Color::rgba(103, 80, 164, 28))
        .scrollbar_pressed_track_color(Color::rgba(103, 80, 164, 28))
        .scrollbar_pressed_handle_color(Color::rgba(103, 80, 164, 176))
        .scrollbar_pressed_handle_border_color(Color::rgba(255, 251, 254, 180))
        .scrollbar_pressed_handle_border_width(1.0)
        .scrollbar_radius(6.0)
        .transition(Transition::ease_out(0.14))
}

fn web_elevation(level: u8, color: Color) -> Vec<Shadow> {
    match level.min(5) {
        0 => Vec::new(),
        1 => single_shadow(color, 0.0, 4.0, 10.0, 0.0, 58),
        2 => single_shadow(color, 0.0, 8.0, 18.0, 0.0, 66),
        3 => single_shadow(color, 0.0, 12.0, 28.0, 0.0, 74),
        4 => single_shadow(color, 0.0, 16.0, 36.0, 0.0, 82),
        _ => single_shadow(color, 0.0, 20.0, 44.0, 0.0, 90),
    }
}

fn single_shadow(color: Color, x: f32, y: f32, blur: f32, spread: f32, alpha: u8) -> Vec<Shadow> {
    vec![Shadow {
        offset: Point::new(x, y),
        blur,
        spread,
        color: with_alpha(color, alpha),
    }]
}

fn with_alpha(color: Color, alpha: u8) -> Color {
    Color { a: alpha, ..color }
}
