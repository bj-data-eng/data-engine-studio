mod framework;

use super::{PANEL, STROKE_SELECTED};
use des_document::{
    FlexDirection, FloatingAxisOffset, FloatingPlacement, FloatingShift, Insets, Length, Style,
    StyleSelector, StyleSheet, ViewportQuery,
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
