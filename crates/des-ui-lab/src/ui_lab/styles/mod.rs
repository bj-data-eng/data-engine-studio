mod framework;

use super::{
    BACKGROUND, CARD, CARD_HOVER, CARD_PRESSED, CARD_SELECTED, GREEN, PANEL, PANEL_ALT,
    PRIMARY_CONTAINER, PURPLE, SECONDARY_CONTAINER, SHADOW_COLOR, STROKE, STROKE_SELECTED,
    SUCCESS_CONTAINER, SURFACE_CONTAINER, SURFACE_CONTAINER_HIGH, TERTIARY_CONTAINER, TEXT,
    TEXT_ACCENT, TEXT_MUTED, WARNING_CONTAINER,
};
use des_document::{
    AlignItems, BorderStyle, Color, Direction, Element, ElementStateSelector, FlexDirection,
    FlexWrap, FloatingAxisOffset, FloatingPlacement, FloatingShift, Insets, JustifyContent, Length,
    Overflow, OverflowWrap, Point, Shadow, Style, StyleSelector, StyleSheet, TextAlign,
    TextLayoutStyle, TextOverflow, TextTransform, TextWrapMode, Transition, ViewportQuery,
    WhiteSpace,
};

const DRAGGABLE_STACK_VIEWPORT_WIDTH: f32 = 1268.0;

pub(super) fn stylesheet() -> StyleSheet {
    let mut stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::Element(Element::Root),
            Style::default()
                .flex_direction(FlexDirection::Column)
                .background(BACKGROUND),
        )
        .rule(
            StyleSelector::Element(Element::Div),
            Style::default().flex_direction(FlexDirection::Column),
        )
        .rule(
            StyleSelector::Element(Element::Button),
            Style::default()
                .padding(Insets::symmetric(12.0, 7.0))
                .background(CARD)
                .border(STROKE)
                .radius(5.0),
        )
        .rule(
            StyleSelector::Element(Element::Checkbox),
            Style::default()
                .flex_direction(FlexDirection::Row)
                .align_items(AlignItems::Center)
                .padding(Insets::symmetric(9.0, 7.0))
                .gap(8.0)
                .background(CARD)
                .border(STROKE)
                .radius(5.0),
        )
        .rule(
            StyleSelector::Element(Element::Radio),
            Style::default()
                .flex_direction(FlexDirection::Row)
                .align_items(AlignItems::Center)
                .padding(Insets::symmetric(9.0, 7.0))
                .gap(8.0)
                .background(CARD)
                .border(STROKE)
                .radius(5.0),
        )
        .rule(
            StyleSelector::Element(Element::Select),
            Style::default()
                .flex_direction(FlexDirection::Row)
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::SpaceBetween)
                .padding(Insets::symmetric(10.0, 7.0))
                .gap(8.0)
                .background(CARD)
                .border(STROKE)
                .radius(5.0),
        )
        .rule(
            StyleSelector::Element(Element::Input),
            Style::default()
                .padding(Insets::symmetric(10.0, 7.0))
                .background(PANEL)
                .border(STROKE)
                .radius(5.0),
        )
        .rule(
            StyleSelector::Element(Element::Icon),
            Style::default()
                .size(14.0, 14.0)
                .font_size(14.0)
                .text_color(TEXT_MUTED),
        )
        .rule(
            StyleSelector::Element(Element::Text),
            Style::default()
                .font_size(13.0)
                .text_color(TEXT)
                .text_selection_background(Color::rgba(103, 80, 164, 220))
                .text_selection_color(PANEL),
        )
        .rule(
            StyleSelector::class("lab-root"),
            Style::default()
                .width_fill()
                .height_fill()
                .background(BACKGROUND)
                .gap(0.0),
        )
        .rule(
            StyleSelector::class("topbar"),
            Style::default()
                .width_fill()
                .height(Length::Px(58.0))
                .padding(Insets::symmetric(18.0, 10.0))
                .gap(3.0)
                .background(PANEL),
        )
        .rule(
            StyleSelector::class("lab-body"),
            Style::default()
                .flex_direction(FlexDirection::Row)
                .width_fill()
                .height(Length::Px(0.0))
                .flex_grow(1.0)
                .padding(Insets::all(14.0))
                .gap(14.0)
                .background(BACKGROUND),
        )
        .rule(
            StyleSelector::class("nav"),
            Style::default()
                .width(Length::Px(242.0))
                .height_fill()
                .padding(Insets::all(12.0))
                .gap(8.0)
                .background(PANEL)
                .border(STROKE)
                .radius(8.0)
                .overflow_y(Overflow::Scroll)
                .z_index(10),
        )
        .rule(
            StyleSelector::class("stage"),
            Style::default()
                .width(Length::Px(0.0))
                .height_fill()
                .flex_grow(1.0)
                .padding(Insets::all(18.0))
                .gap(12.0)
                .background(PANEL_ALT)
                .border(STROKE)
                .radius(8.0)
                .overflow_y(Overflow::Scroll)
                .scrollbar_width(2.0),
        )
        .rule(
            StyleSelector::class("box-model-grid"),
            Style::default()
                .width_fill()
                .gap(10.0)
                .background(PANEL_ALT),
        )
        .rule(
            StyleSelector::class("box-model-row"),
            Style::default()
                .flex_direction(FlexDirection::Row)
                .flex_wrap(FlexWrap::Wrap)
                .width_fill()
                .height(Length::Auto)
                .gap(10.0)
                .background(PANEL_ALT),
        )
        .rule(
            StyleSelector::class("box-model-case"),
            Style::default()
                .width(Length::Px(318.0))
                .height(Length::Auto)
                .padding(Insets::all(8.0))
                .gap(3.0)
                .background(SURFACE_CONTAINER)
                .border(STROKE)
                .radius(5.0),
        )
        .rule(
            StyleSelector::class("box-section-label"),
            Style::default()
                .font_size(14.0)
                .text_color(TEXT_ACCENT)
                .height(Length::Px(18.0)),
        )
        .rule(
            StyleSelector::class("box-label"),
            Style::default()
                .font_size(13.0)
                .text_color(TEXT)
                .height(Length::Px(16.0)),
        )
        .rule(
            StyleSelector::class("box-note"),
            Style::default()
                .font_size(11.0)
                .text_color(TEXT_MUTED)
                .height(Length::Px(14.0)),
        )
        .rule(
            StyleSelector::class("box-rule"),
            Style::default()
                .font_size(10.0)
                .text_color(TEXT_ACCENT)
                .height(Length::Px(12.0)),
        )
        .rule(
            StyleSelector::class("box-subject-frame"),
            Style::default()
                .width_fill()
                .height(Length::Auto)
                .min_size(0.0, 86.0)
                .background(PANEL)
                .border(STROKE)
                .overflow_y(Overflow::Visible),
        )
        .rule(
            StyleSelector::class("box-subject"),
            Style::default()
                .size(32.0, 32.0)
                .gap(0.0)
                .padding(Insets::ZERO)
                .background(Color::rgb(210, 228, 250)),
        )
        .rule(
            StyleSelector::class("box-chip"),
            Style::default()
                .size(12.0, 12.0)
                .background(SUCCESS_CONTAINER),
        )
        .rule(
            StyleSelector::class("box-overflow-child"),
            Style::default()
                .size(112.0, 112.0)
                .background(Color::rgb(255, 220, 170)),
        )
        .rule(
            StyleSelector::class("box-subject-auto"),
            Style::default().width(Length::Auto).height(Length::Auto),
        )
        .rule(
            StyleSelector::class("box-subject-px"),
            Style::default().size(96.0, 44.0),
        )
        .rule(
            StyleSelector::class("box-subject-min"),
            Style::default()
                .width(Length::Auto)
                .height(Length::Auto)
                .min_size(40.0, 40.0),
        )
        .rule(
            StyleSelector::class("box-subject-max"),
            Style::default()
                .width(Length::Auto)
                .height(Length::Auto)
                .max_size(52.0, 34.0),
        )
        .rule(
            StyleSelector::class("box-max-wide-child"),
            Style::default()
                .size(88.0, 48.0)
                .background(SUCCESS_CONTAINER),
        )
        .rule(
            StyleSelector::class("box-subject-fill"),
            Style::default().width_fill().height(Length::Px(28.0)),
        )
        .rule(
            StyleSelector::class("box-subject-percent"),
            Style::default().width_percent(0.5).height(Length::Px(28.0)),
        )
        .rule(
            StyleSelector::class("box-subject-height-fill"),
            Style::default().width(Length::Px(64.0)).height_fill(),
        )
        .rule(
            StyleSelector::class("box-subject-margin"),
            Style::default().size(32.0, 32.0).margin(Insets::all(12.0)),
        )
        .rule(
            StyleSelector::class("box-subject-padding"),
            Style::default()
                .width(Length::Auto)
                .height(Length::Auto)
                .padding(Insets::all(12.0)),
        )
        .rule(
            StyleSelector::class("box-subject-border"),
            Style::default()
                .size(44.0, 44.0)
                .border(PURPLE)
                .border_width(5.0),
        )
        .rule(
            StyleSelector::class("box-subject-row-gap"),
            Style::default()
                .flex_direction(FlexDirection::Row)
                .width(Length::Auto)
                .height(Length::Auto)
                .gap(10.0),
        )
        .rule(
            StyleSelector::class("box-subject-column-gap"),
            Style::default()
                .flex_direction(FlexDirection::Column)
                .width(Length::Auto)
                .height(Length::Auto)
                .gap(6.0),
        )
        .rule(
            StyleSelector::class("box-subject-row-align"),
            Style::default()
                .flex_direction(FlexDirection::Row)
                .size(96.0, 54.0)
                .gap(8.0)
                .justify_content(JustifyContent::Center)
                .align_items(AlignItems::End),
        )
        .rule(
            StyleSelector::class("box-subject-column-align"),
            Style::default()
                .flex_direction(FlexDirection::Column)
                .size(80.0, 92.0)
                .gap(4.0)
                .justify_content(JustifyContent::SpaceBetween)
                .align_items(AlignItems::Center),
        )
        .rule(
            StyleSelector::class("box-subject-visible-overflow"),
            Style::default()
                .size(44.0, 44.0)
                .overflow_y(Overflow::Visible),
        )
        .rule(
            StyleSelector::class("box-subject-scroll-overflow"),
            Style::default()
                .size(44.0, 44.0)
                .overflow_y(Overflow::Scroll)
                .scrollbar_width(2.0),
        )
        .rule(
            StyleSelector::class("box-subject-scroll-x-overflow"),
            Style::default()
                .size(44.0, 44.0)
                .overflow_x(Overflow::Scroll)
                .overflow_y(Overflow::Visible)
                .scrollbar_width(2.0),
        )
        .rule(
            StyleSelector::class("box-subject-scroll-xy-overflow"),
            Style::default()
                .size(44.0, 44.0)
                .overflow(Overflow::Scroll)
                .scrollbar_width(2.0),
        )
        .rule(
            StyleSelector::class("box-subject-side-radius"),
            Style::default()
                .size(64.0, 44.0)
                .border(PURPLE)
                .border_width(2.0)
                .radius(4.0),
        )
        .rule(
            StyleSelector::class("box-subject-side-radius"),
            Style::default()
                .border_left_width(8.0)
                .border_bottom_width(5.0)
                .top_right_radius(14.0)
                .bottom_left_radius(0.0),
        )
        .rule(
            StyleSelector::class("box-subject-nested-nine"),
            Style::default().width(Length::Auto).height(Length::Auto),
        )
        .rule(
            StyleSelector::class("box-nested-outer"),
            Style::default()
                .width(Length::Auto)
                .height(Length::Auto)
                .margin(Insets::all(8.0))
                .border(PURPLE)
                .border_width(3.0)
                .background(PRIMARY_CONTAINER),
        )
        .rule(
            StyleSelector::class("box-nested-inner"),
            Style::default()
                .width(Length::Auto)
                .height(Length::Auto)
                .padding(Insets::all(5.0))
                .gap(4.0)
                .border(GREEN)
                .border_width(2.0)
                .background(PRIMARY_CONTAINER),
        )
        .rule(
            StyleSelector::class("box-nested-row"),
            Style::default()
                .flex_direction(FlexDirection::Row)
                .width(Length::Auto)
                .height(Length::Auto)
                .gap(4.0)
                .background(Color::rgb(210, 228, 250)),
        )
        .rule(
            StyleSelector::class("box-nested-cell"),
            Style::default()
                .size(10.0, 10.0)
                .background(SUCCESS_CONTAINER),
        )
        .rule(
            StyleSelector::class("box-subject-inset-percent"),
            Style::default().width(Length::Auto).height(Length::Auto),
        )
        .rule(
            StyleSelector::class("box-inset-percent-parent"),
            Style::default()
                .size(88.0, 88.0)
                .padding(Insets::all(8.0))
                .border(PURPLE)
                .border_width(2.0)
                .background(Color::rgb(210, 228, 250)),
        )
        .rule(
            StyleSelector::class("box-inset-percent-child"),
            Style::default()
                .width_percent(0.5)
                .height_percent(0.5)
                .background(SUCCESS_CONTAINER),
        )
        .rule(
            StyleSelector::class("box-subject-absolute-parent"),
            Style::default().width(Length::Auto).height(Length::Auto),
        )
        .rule(
            StyleSelector::class("box-absolute-parent-frame"),
            Style::default()
                .size(88.0, 64.0)
                .padding(Insets::all(8.0))
                .border(PURPLE)
                .border_width(2.0)
                .background(Color::rgb(210, 228, 250)),
        )
        .rule(
            StyleSelector::class("box-absolute-flow-child"),
            Style::default()
                .size(16.0, 16.0)
                .background(Color::rgb(190, 215, 246)),
        )
        .rule(
            StyleSelector::class("box-absolute-parent-child"),
            Style::default()
                .absolute_parent()
                .top(Length::Px(8.0))
                .left(Length::Px(14.0))
                .size(26.0, 26.0)
                .z_index(2)
                .background(SUCCESS_CONTAINER)
                .border(GREEN),
        )
        .rule(
            StyleSelector::class("box-subject-absolute-window"),
            Style::default().width(Length::Auto).height(Length::Auto),
        )
        .rule(
            StyleSelector::class("box-absolute-window-host"),
            Style::default()
                .size(88.0, 64.0)
                .border(PURPLE)
                .border_width(2.0)
                .background(PRIMARY_CONTAINER),
        )
        .rule(
            StyleSelector::class("box-absolute-window-child"),
            Style::default()
                .absolute_viewport()
                .top(Length::Px(140.0))
                .left(Length::Px(420.0))
                .size(26.0, 26.0)
                .z_index(20)
                .background(PRIMARY_CONTAINER)
                .border(PURPLE),
        )
        .rule(
            StyleSelector::class("nav-item"),
            Style::default()
                .width_fill()
                .height(des_document::Length::Px(64.0))
                .padding(Insets::all(12.0))
                .gap(5.0)
                .background(CARD)
                .border(STROKE)
                .radius(7.0),
        )
        .rule(
            StyleSelector::class_state("nav-item", ElementStateSelector::Selected),
            Style::default()
                .background(CARD_SELECTED)
                .border(STROKE_SELECTED),
        )
        .rule(
            StyleSelector::class_state("nav-item", ElementStateSelector::Hovered),
            Style::default()
                .background(CARD_HOVER)
                .border(STROKE_SELECTED),
        )
        .rule(
            StyleSelector::class("toolbar-row"),
            Style::default()
                .flex_direction(FlexDirection::Row)
                .gap(8.0)
                .background(PANEL_ALT),
        )
        .rule(
            StyleSelector::class("button"),
            Style::default()
                .size(156.0, 36.0)
                .background(SURFACE_CONTAINER_HIGH)
                .border(STROKE),
        )
        .rule(
            StyleSelector::class_state("button", ElementStateSelector::Selected),
            Style::default()
                .background(CARD_SELECTED)
                .border(STROKE_SELECTED),
        )
        .rule(
            StyleSelector::class_state("button", ElementStateSelector::Hovered),
            Style::default()
                .background(CARD_HOVER)
                .border(STROKE_SELECTED),
        )
        .rule(
            StyleSelector::class_state("button", ElementStateSelector::Pressed),
            Style::default().background(CARD_PRESSED),
        )
        .rule(
            StyleSelector::class("button-label"),
            Style::default().font_size(13.0).text_color(TEXT),
        )
        .rule(
            StyleSelector::class("card-row"),
            Style::default()
                .flex_direction(FlexDirection::Row)
                .gap(12.0)
                .background(PANEL_ALT),
        )
        .rule(
            StyleSelector::class("card-row-dense"),
            Style::default()
                .flex_direction(FlexDirection::Row)
                .gap(6.0)
                .background(PANEL_ALT),
        )
        .rule(
            StyleSelector::class("controls-grid"),
            Style::default()
                .flex_direction(FlexDirection::Row)
                .flex_wrap(FlexWrap::Wrap)
                .width_fill()
                .height(Length::Auto)
                .gap(12.0)
                .background(PANEL_ALT),
        )
        .rule(
            StyleSelector::class("control-card"),
            Style::default()
                .width(Length::Px(235.0))
                .height(Length::Auto)
                .min_size(0.0, 132.0)
                .padding(Insets::all(10.0))
                .gap(8.0)
                .background(PANEL)
                .border(STROKE)
                .radius(6.0),
        )
        .rule(
            StyleSelector::class("control-row"),
            Style::default().width_fill().height(Length::Px(36.0)),
        )
        .rule(
            StyleSelector::class_state("control-row", ElementStateSelector::Hovered),
            Style::default()
                .background(CARD_HOVER)
                .border(STROKE_SELECTED),
        )
        .rule(
            StyleSelector::class_state("control-row", ElementStateSelector::Pressed),
            Style::default().background(CARD_PRESSED),
        )
        .rule(
            StyleSelector::class_state("control-row", ElementStateSelector::Selected),
            Style::default()
                .background(CARD_SELECTED)
                .border(STROKE_SELECTED),
        )
        .rule(
            StyleSelector::class("checkbox-mark"),
            Style::default()
                .size(18.0, 18.0)
                .padding(Insets::ZERO)
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::Center)
                .background(PANEL)
                .border(STROKE)
                .radius(4.0),
        )
        .rule(
            StyleSelector::class_state("checkbox-mark", ElementStateSelector::Selected),
            Style::default()
                .background(STROKE_SELECTED)
                .border(STROKE_SELECTED),
        )
        .rule(
            StyleSelector::class("check-glyph"),
            Style::default()
                .size(13.0, 13.0)
                .font_size(13.0)
                .text_color(TEXT),
        )
        .rule(
            StyleSelector::class("radio-dot"),
            Style::default()
                .size(18.0, 18.0)
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::Center)
                .background(PANEL)
                .border(STROKE)
                .border_width(2.0)
                .radius(9.0),
        )
        .rule(
            StyleSelector::class_state("radio-dot", ElementStateSelector::Selected),
            Style::default().border(STROKE_SELECTED).border_width(2.0),
        )
        .rule(
            StyleSelector::class("radio-dot-fill"),
            Style::default()
                .size(8.0, 8.0)
                .background(STROKE_SELECTED)
                .radius(4.0),
        )
        .rule(
            StyleSelector::class("control-label"),
            Style::default().font_size(12.5).text_color(TEXT),
        )
        .rule(
            StyleSelector::class_state("control-label", ElementStateSelector::Selected),
            Style::default().text_color(TEXT_ACCENT),
        )
        .rule(
            StyleSelector::class_state("control-label", ElementStateSelector::Disabled),
            Style::default().text_color(TEXT_MUTED),
        )
        .rule(
            StyleSelector::class("dropdown-field"),
            Style::default().width_fill().height(Length::Px(38.0)),
        )
        .rule(
            StyleSelector::class("dropdown-control"),
            Style::default().width_fill().height_fill(),
        )
        .rule(
            StyleSelector::class_state("dropdown-control", ElementStateSelector::Hovered),
            Style::default()
                .background(CARD_HOVER)
                .border(STROKE_SELECTED),
        )
        .rule(
            StyleSelector::class_state("dropdown-control", ElementStateSelector::Selected),
            Style::default()
                .border(STROKE_SELECTED)
                .bottom_left_radius(0.0)
                .bottom_right_radius(0.0),
        )
        .rule(
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
        )
        .rule(
            StyleSelector::class("dropdown-option"),
            Style::default()
                .width_fill()
                .height(Length::Px(30.0))
                .padding(Insets::symmetric(8.0, 6.0)),
        )
        .rule(
            StyleSelector::class_state("dropdown-option", ElementStateSelector::Hovered),
            Style::default().background(CARD_HOVER),
        )
        .rule(
            StyleSelector::class_state("dropdown-option", ElementStateSelector::Selected),
            Style::default().background(CARD_SELECTED),
        )
        .rule(
            StyleSelector::class("input-field"),
            Style::default().width_fill().height(Length::Px(38.0)),
        )
        .rule(
            StyleSelector::class_state("input-field", ElementStateSelector::Focused),
            Style::default().border(STROKE_SELECTED).border_width(2.0),
        )
        .rule(
            StyleSelector::class_state("input-field", ElementStateSelector::Disabled),
            Style::default().background(PANEL_ALT).border(STROKE),
        )
        .rule(
            StyleSelector::class("loop-grid"),
            Style::default()
                .flex_direction(FlexDirection::Row)
                .flex_wrap(FlexWrap::Wrap)
                .width_fill()
                .height(Length::Auto)
                .gap(10.0)
                .background(PANEL_ALT),
        )
        .rule(
            StyleSelector::class("loop-control-card"),
            Style::default()
                .width(Length::Px(190.0))
                .height(Length::Px(104.0))
                .padding(Insets::all(10.0))
                .gap(8.0)
                .background(PANEL)
                .border(STROKE)
                .radius(6.0),
        )
        .rule(
            StyleSelector::class("loop-button"),
            Style::default()
                .width_fill()
                .height(Length::Px(34.0))
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::Center)
                .background(CARD_SELECTED)
                .border(STROKE_SELECTED)
                .radius(5.0),
        )
        .rule(
            StyleSelector::class_state("loop-button", ElementStateSelector::Hovered),
            Style::default().background(PRIMARY_CONTAINER),
        )
        .rule(
            StyleSelector::class_state("loop-button", ElementStateSelector::Pressed),
            Style::default().background(CARD_PRESSED),
        )
        .rule(
            StyleSelector::class("loop-result-card"),
            Style::default()
                .width(Length::Px(190.0))
                .height(Length::Px(104.0))
                .padding(Insets::all(10.0))
                .gap(8.0)
                .background(CARD)
                .border(STROKE),
        )
        .rule(
            StyleSelector::class_state("loop-result-card", ElementStateSelector::Selected),
            Style::default().background(SUCCESS_CONTAINER).border(GREEN),
        )
        .rule(
            StyleSelector::class_state("loop-result-card", ElementStateSelector::Focused),
            Style::default()
                .background(SECONDARY_CONTAINER)
                .border(PURPLE),
        )
        .rule(
            StyleSelector::class("loop-runtime-local"),
            Style::default().border(STROKE_SELECTED),
        )
        .rule(
            StyleSelector::class("loop-runtime-remote"),
            Style::default().border(GREEN),
        )
        .rule(
            StyleSelector::class("loop-runtime-hybrid"),
            Style::default().border(PURPLE),
        )
        .rule(
            StyleSelector::class("loop-source-csv"),
            Style::default().background(PRIMARY_CONTAINER),
        )
        .rule(
            StyleSelector::class("loop-source-duckdb"),
            Style::default().background(WARNING_CONTAINER),
        )
        .rule(
            StyleSelector::class("loop-source-python"),
            Style::default().background(TERTIARY_CONTAINER),
        )
        .rule(
            StyleSelector::class("drag-workbench"),
            Style::default()
                .flex_direction(FlexDirection::Row)
                .flex_wrap(FlexWrap::Wrap)
                .align_items(AlignItems::Start)
                .width_fill()
                .height(Length::Auto)
                .gap(14.0),
        )
        .rule(
            StyleSelector::class("drag-grid"),
            Style::default()
                .flex_direction(FlexDirection::Row)
                .flex_wrap(FlexWrap::Wrap)
                .width(Length::Px(0.0))
                .height(Length::Auto)
                .flex_basis(Length::Px(0.0))
                .flex_grow(1.0)
                .padding(Insets::all(10.0))
                .gap(8.0)
                .background(PANEL)
                .border(STROKE)
                .radius(6.0)
                .transition(Transition::ease_out(0.12)),
        )
        .rule(
            StyleSelector::class("drag-cell"),
            Style::default()
                .width_percent(0.45)
                .height(Length::Auto)
                .flex_basis(Length::Percent(0.45))
                .flex_grow(1.0)
                .min_size(0.0, 70.0)
                .padding(Insets::all(7.0))
                .gap(5.0)
                .background(SURFACE_CONTAINER)
                .border(STROKE)
                .radius(5.0)
                .transition(Transition::ease_out(0.12)),
        )
        .rule(
            StyleSelector::class_state("drag-cell", ElementStateSelector::Hovered),
            Style::default()
                .background(SURFACE_CONTAINER_HIGH)
                .border(STROKE_SELECTED),
        )
        .rule(
            StyleSelector::class("specimen-card"),
            Style::default()
                .width(Length::Px(600.0))
                .height(Length::Auto)
                .padding(Insets::all(12.0))
                .gap(10.0)
                .background(PANEL)
                .border(STROKE)
                .radius(6.0),
        )
        .rule(
            StyleSelector::class("text-copy"),
            Style::default().text_wrap_mode(TextWrapMode::Wrap),
        )
        .rule(
            StyleSelector::class("text-antialias-panel"),
            Style::default()
                .width_fill()
                .height(Length::Auto)
                .padding(Insets::all(12.0))
                .gap(7.0)
                .background(PANEL)
                .border(STROKE)
                .radius(6.0),
        )
        .rule(
            StyleSelector::class("text-ramp-line"),
            Style::default()
                .white_space(WhiteSpace::Pre)
                .line_height(42.0)
                .text_color(TEXT),
        )
        .rule(
            StyleSelector::class("text-render-path-row"),
            Style::default()
                .flex_direction(FlexDirection::Row)
                .width_fill()
                .gap(12.0),
        )
        .rule(
            StyleSelector::class("text-diagnostics"),
            Style::default()
                .font_size(11.0)
                .line_height(15.0)
                .text_wrap_mode(TextWrapMode::Wrap),
        )
        .rule(
            StyleSelector::class("text-render-path-pane"),
            Style::default()
                .width_percent(0.5)
                .height(Length::Px(150.0))
                .padding(Insets::all(8.0))
                .gap(6.0)
                .background(PANEL_ALT)
                .border(STROKE)
                .radius(4.0),
        )
        .rule(
            StyleSelector::class("text-render-path-slot"),
            Style::default()
                .width_fill()
                .height(Length::Px(112.0))
                .white_space(WhiteSpace::Pre)
                .text_color(TEXT),
        )
        .rule(
            StyleSelector::class("text-size-9"),
            Style::default().font_size(9.0).line_height(13.0),
        )
        .rule(
            StyleSelector::class("text-size-10"),
            Style::default().font_size(10.0).line_height(14.0),
        )
        .rule(
            StyleSelector::class("text-size-11"),
            Style::default().font_size(11.0).line_height(15.0),
        )
        .rule(
            StyleSelector::class("text-size-12"),
            Style::default().font_size(12.0).line_height(16.0),
        )
        .rule(
            StyleSelector::class("text-size-13"),
            Style::default().font_size(13.0).line_height(18.0),
        )
        .rule(
            StyleSelector::class("text-size-14"),
            Style::default().font_size(14.0).line_height(19.0),
        )
        .rule(
            StyleSelector::class("text-size-16"),
            Style::default().font_size(16.0).line_height(22.0),
        )
        .rule(
            StyleSelector::class("text-size-18"),
            Style::default().font_size(18.0).line_height(25.0),
        )
        .rule(
            StyleSelector::class("text-size-22"),
            Style::default().font_size(22.0).line_height(30.0),
        )
        .rule(
            StyleSelector::class("text-size-28"),
            Style::default().font_size(28.0).line_height(38.0),
        )
        .rule(
            StyleSelector::class("text-size-36"),
            Style::default().font_size(36.0).line_height(48.0),
        )
        .rule(
            StyleSelector::class("text-size-100"),
            Style::default().font_size(100.0).line_height(112.0),
        )
        .rule(
            StyleSelector::class("text-tone-line"),
            Style::default()
                .white_space(WhiteSpace::Pre)
                .font_size(17.0)
                .line_height(25.0)
                .padding(Insets::symmetric(8.0, 5.0))
                .background(PANEL_ALT)
                .border(STROKE)
                .radius(4.0),
        )
        .rule(
            StyleSelector::class("text-tone-dark"),
            Style::default().text_color(Color::rgb(29, 27, 32)),
        )
        .rule(
            StyleSelector::class("text-tone-muted"),
            Style::default().text_color(TEXT_MUTED),
        )
        .rule(
            StyleSelector::class("text-tone-accent"),
            Style::default().text_color(TEXT_ACCENT),
        )
        .rule(
            StyleSelector::class("text-tone-inverse"),
            Style::default()
                .background(Color::rgb(29, 27, 32))
                .text_color(Color::rgb(255, 251, 254)),
        )
        .rule(
            StyleSelector::class("text-specimen-grid"),
            Style::default()
                .flex_direction(FlexDirection::Row)
                .flex_wrap(FlexWrap::Wrap)
                .width_fill()
                .height(Length::Auto)
                .gap(12.0)
                .background(PANEL_ALT),
        )
        .rule(
            StyleSelector::class("text-specimen-card"),
            Style::default()
                .width(Length::Px(280.0))
                .height(Length::Auto)
                .padding(Insets::all(12.0))
                .gap(8.0)
                .background(PANEL)
                .border(STROKE)
                .radius(6.0),
        )
        .rule(
            StyleSelector::class("text-rule"),
            Style::default()
                .font_size(11.0)
                .text_color(TEXT_ACCENT)
                .text_wrap_mode(TextWrapMode::Wrap),
        )
        .rule(
            StyleSelector::class("text-box"),
            Style::default()
                .width(Length::Px(220.0))
                .height(Length::Auto)
                .padding(Insets::all(8.0))
                .font_size(13.0)
                .line_height(18.0)
                .background(PANEL)
                .border(STROKE)
                .radius(4.0),
        )
        .rule(
            StyleSelector::class("text-box-extend"),
            Style::default().white_space(WhiteSpace::Pre),
        )
        .rule(
            StyleSelector::class("text-box-wrap"),
            Style::default().text_wrap_mode(TextWrapMode::Wrap),
        )
        .rule(
            StyleSelector::class("text-box-break-word"),
            Style::default()
                .text_wrap_mode(TextWrapMode::Wrap)
                .overflow_wrap(OverflowWrap::BreakWord),
        )
        .rule(
            StyleSelector::class("text-box-truncate"),
            Style::default().text_layout(TextLayoutStyle {
                max_lines: Some(1),
                text_overflow: TextOverflow::Ellipsis,
                ..TextLayoutStyle::default()
            }),
        )
        .rule(
            StyleSelector::class("text-box-max-lines"),
            Style::default()
                .text_wrap_mode(TextWrapMode::Wrap)
                .max_lines(2),
        )
        .rule(
            StyleSelector::class("text-box-pre"),
            Style::default().white_space(WhiteSpace::Pre),
        )
        .rule(
            StyleSelector::class("text-box-break-spaces"),
            Style::default().white_space(WhiteSpace::BreakSpaces),
        )
        .rule(
            StyleSelector::class("text-box-uppercase"),
            Style::default()
                .text_wrap_mode(TextWrapMode::Wrap)
                .text_transform(TextTransform::Uppercase),
        )
        .rule(
            StyleSelector::class("text-box-rtl"),
            Style::default()
                .text_wrap_mode(TextWrapMode::Wrap)
                .text_align(TextAlign::Start)
                .direction(Direction::Rtl),
        )
        .rule(
            StyleSelector::class("text-rich-line"),
            Style::default()
                .width_fill()
                .white_space(WhiteSpace::PreWrap)
                .font_size(17.0)
                .line_height(26.0)
                .padding(Insets::symmetric(10.0, 6.0))
                .background(PANEL_ALT)
                .border(STROKE)
                .radius(4.0),
        )
        .rule(
            StyleSelector::class("data-table"),
            Style::default()
                .width(Length::Px(520.0))
                .height(Length::Auto)
                .background(PANEL)
                .border(STROKE)
                .radius(5.0)
                .overflow_x(Overflow::Scroll)
                .scrollbar_width(2.0),
        )
        .rule(
            StyleSelector::class("table-header-row"),
            Style::default().background(SURFACE_CONTAINER_HIGH),
        )
        .rule(
            StyleSelector::class("table-row"),
            Style::default().background(SURFACE_CONTAINER),
        )
        .rule(
            StyleSelector::class("table-header-cell"),
            Style::default()
                .font_size(12.0)
                .text_color(TEXT_ACCENT)
                .border(STROKE)
                .border_widths(Insets {
                    top: 0.0,
                    right: 1.0,
                    bottom: 1.0,
                    left: 0.0,
                }),
        )
        .rule(
            StyleSelector::class("table-cell"),
            Style::default()
                .font_size(12.0)
                .text_color(TEXT)
                .border(STROKE)
                .border_widths(Insets {
                    top: 0.0,
                    right: 1.0,
                    bottom: 1.0,
                    left: 0.0,
                }),
        )
        .rule(
            StyleSelector::class("feature-card"),
            Style::default()
                .size(250.0, 98.0)
                .background(CARD)
                .border(STROKE),
        )
        .rule(
            StyleSelector::class_state("feature-card", ElementStateSelector::Hovered),
            Style::default()
                .background(CARD_HOVER)
                .border(STROKE_SELECTED),
        )
        .rule(
            StyleSelector::class_state("feature-card", ElementStateSelector::Pressed),
            Style::default().background(CARD_PRESSED),
        )
        .rule(
            StyleSelector::class("stack"),
            Style::default()
                .size(620.0, 320.0)
                .padding(Insets::all(10.0))
                .gap(8.0)
                .background(PANEL)
                .border(STROKE)
                .radius(7.0),
        )
        .rule(
            StyleSelector::class("list-row"),
            Style::default()
                .size(600.0, 58.0)
                .background(SURFACE_CONTAINER)
                .border(STROKE)
                .radius(5.0),
        )
        .rule(
            StyleSelector::class("specificity-proof"),
            Style::default()
                .background(SURFACE_CONTAINER_HIGH)
                .border(STROKE),
        )
        .rule(
            StyleSelector::class_state("specificity-proof", ElementStateSelector::Hovered),
            Style::default()
                .background(SECONDARY_CONTAINER)
                .border(GREEN),
        )
        .rule(
            StyleSelector::id("style-row-state"),
            Style::default().border(PURPLE),
        )
        .rule(
            StyleSelector::id_state("style-row-state", ElementStateSelector::Hovered),
            Style::default()
                .background(SECONDARY_CONTAINER)
                .border(TEXT_ACCENT),
        )
        .rule(
            StyleSelector::class("shadow-grid"),
            Style::default()
                .flex_direction(FlexDirection::Row)
                .flex_wrap(FlexWrap::Wrap)
                .width_fill()
                .height(Length::Auto)
                .padding(Insets::all(18.0))
                .gap(18.0)
                .background(PANEL_ALT)
                .border(STROKE)
                .radius(7.0),
        )
        .rule(
            StyleSelector::class("shadow-card"),
            Style::default()
                .size(220.0, 88.0)
                .padding(Insets::all(12.0))
                .gap(5.0)
                .background(CARD)
                .border(STROKE)
                .radius(8.0),
        )
        .rule(
            StyleSelector::class("shadow-single"),
            Style::default().shadows(web_elevation(1, SHADOW_COLOR)),
        )
        .rule(
            StyleSelector::class("shadow-layered"),
            Style::default().shadows(web_elevation(2, SHADOW_COLOR)),
        )
        .rule(
            StyleSelector::class("shadow-negative-spread"),
            Style::default().shadows(web_elevation(3, SHADOW_COLOR)),
        )
        .rule(
            StyleSelector::class("shadow-light-stage"),
            Style::default()
                .width(Length::Px(740.0))
                .height(Length::Auto)
                .padding(Insets::symmetric(18.0, 16.0))
                .gap(18.0)
                .background(Color::rgb(241, 230, 244))
                .radius(7.0),
        )
        .rule(
            StyleSelector::class("shadow-light-card"),
            Style::default()
                .flex_direction(FlexDirection::Row)
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::SpaceBetween)
                .width(Length::Px(360.0))
                .height(Length::Px(64.0))
                .padding(Insets::symmetric(24.0, 0.0))
                .background(Color::rgb(250, 250, 252))
                .border(Color::rgba(198, 188, 205, 145))
                .radius(6.0)
                .shadows(web_elevation(1, SHADOW_COLOR)),
        )
        .rule(
            StyleSelector::class("shadow-light-card-raised"),
            Style::default().shadows(web_elevation(2, SHADOW_COLOR)),
        )
        .rule(
            StyleSelector::class("shadow-light-label"),
            Style::default()
                .font_size(15.0)
                .text_color(Color::rgb(137, 132, 146)),
        )
        .rule(
            StyleSelector::class("shadow-light-handle"),
            Style::default()
                .size(36.0, 48.0)
                .font_size(14.0)
                .text_color(Color::rgb(120, 137, 153))
                .background(Color::rgba(232, 231, 236, 190))
                .radius(6.0),
        )
        .rule(
            StyleSelector::class("shadow-web-stage"),
            Style::default()
                .width(Length::Px(370.0))
                .height(Length::Auto)
                .padding(Insets::symmetric(18.0, 16.0))
                .gap(18.0)
                .background(Color::rgb(235, 244, 247))
                .radius(7.0),
        )
        .rule(
            StyleSelector::class("shadow-web-card"),
            Style::default()
                .flex_direction(FlexDirection::Row)
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::SpaceBetween)
                .width(Length::Px(306.0))
                .height(Length::Px(76.0))
                .padding(Insets::symmetric(20.0, 0.0))
                .background(Color::rgb(255, 255, 255))
                .border(Color::rgba(188, 183, 196, 120))
                .radius(5.0),
        )
        .rule(
            StyleSelector::class("shadow-web-card-raised"),
            Style::default().shadows(web_elevation(2, SHADOW_COLOR)),
        )
        .rule(
            StyleSelector::class("shadow-web-label"),
            Style::default()
                .font_size(15.0)
                .text_color(Color::rgb(116, 111, 125)),
        )
        .rule(
            StyleSelector::class("shadow-web-handle"),
            Style::default()
                .size(36.0, 48.0)
                .font_size(14.0)
                .text_color(Color::rgb(120, 137, 153))
                .background(Color::rgba(242, 242, 244, 220))
                .radius(6.0),
        )
        .rule(
            StyleSelector::class("shadow-tune-panel"),
            Style::default()
                .flex_direction(FlexDirection::Row)
                .width_fill()
                .height(Length::Auto)
                .padding(Insets::all(12.0))
                .gap(14.0)
                .background(SURFACE_CONTAINER)
                .border(STROKE)
                .radius(7.0),
        )
        .rule(
            StyleSelector::class("shadow-tune-preview"),
            Style::default()
                .width(Length::Px(370.0))
                .height(Length::Px(280.0))
                .padding(Insets::all(20.0))
                .gap(14.0)
                .background(Color::rgb(235, 244, 247))
                .radius(7.0),
        )
        .rule(
            StyleSelector::class("shadow-tune-preview-card"),
            Style::default()
                .flex_direction(FlexDirection::Row)
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::SpaceBetween)
                .width(Length::Px(306.0))
                .height(Length::Px(76.0))
                .padding(Insets::symmetric(20.0, 0.0))
                .background(Color::rgb(255, 255, 255))
                .border(Color::rgba(188, 183, 196, 120))
                .radius(5.0)
                .transition(Transition::ease_out(0.12)),
        )
        .rule(
            StyleSelector::class("shadow-tune-controls"),
            Style::default()
                .flex_direction(FlexDirection::Column)
                .width(Length::Px(320.0))
                .height(Length::Auto)
                .gap(10.0)
                .background(SURFACE_CONTAINER),
        )
        .rule(
            StyleSelector::class("shadow-tune-layer"),
            Style::default()
                .width(Length::Px(300.0))
                .height(Length::Auto)
                .padding(Insets::all(8.0))
                .gap(6.0)
                .background(CARD)
                .border(STROKE)
                .radius(6.0),
        )
        .rule(
            StyleSelector::class("shadow-tune-header"),
            Style::default()
                .flex_direction(FlexDirection::Row)
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::SpaceBetween)
                .width_fill()
                .height(Length::Px(30.0))
                .background(CARD),
        )
        .rule(
            StyleSelector::class("shadow-tune-toggle"),
            Style::default()
                .width(Length::Px(70.0))
                .height(Length::Px(28.0))
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::Center)
                .background(SURFACE_CONTAINER)
                .border(STROKE)
                .radius(5.0),
        )
        .rule(
            StyleSelector::class_state("shadow-tune-toggle", ElementStateSelector::Hovered),
            Style::default()
                .background(CARD_HOVER)
                .border(STROKE_SELECTED),
        )
        .rule(
            StyleSelector::class("shadow-tune-row"),
            Style::default()
                .flex_direction(FlexDirection::Row)
                .align_items(AlignItems::Center)
                .width_fill()
                .height(Length::Px(28.0))
                .gap(5.0)
                .background(CARD),
        )
        .rule(
            StyleSelector::class("shadow-tune-label"),
            Style::default()
                .width(Length::Px(48.0))
                .height(Length::Px(18.0))
                .font_size(12.0)
                .text_color(TEXT_MUTED),
        )
        .rule(
            StyleSelector::class("shadow-tune-value"),
            Style::default()
                .width(Length::Px(48.0))
                .height(Length::Px(18.0))
                .font_size(12.0)
                .text_color(TEXT),
        )
        .rule(
            StyleSelector::class("shadow-tune-button"),
            Style::default()
                .size(28.0, 24.0)
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::Center)
                .background(SURFACE_CONTAINER)
                .border(STROKE)
                .radius(5.0),
        )
        .rule(
            StyleSelector::class_state("shadow-tune-button", ElementStateSelector::Hovered),
            Style::default()
                .background(CARD_HOVER)
                .border(STROKE_SELECTED),
        )
        .rule(
            StyleSelector::class_state("shadow-tune-button", ElementStateSelector::Pressed),
            Style::default().background(CARD_PRESSED),
        )
        .rule(
            StyleSelector::class("shadow-tune-output"),
            Style::default()
                .width_fill()
                .height(Length::Px(42.0))
                .font_size(11.0)
                .text_color(TEXT_MUTED)
                .text_wrap_mode(TextWrapMode::Wrap),
        )
        .rule(
            StyleSelector::class("structural-grid"),
            Style::default()
                .flex_direction(FlexDirection::Row)
                .flex_wrap(FlexWrap::Wrap)
                .width_fill()
                .height(Length::Auto)
                .gap(12.0)
                .background(PANEL_ALT),
        )
        .rule(
            StyleSelector::class("structural-list"),
            Style::default()
                .width(Length::Px(380.0))
                .height(Length::Auto)
                .padding(Insets::all(8.0))
                .gap(6.0)
                .background(PANEL)
                .border(STROKE)
                .radius(6.0),
        )
        .rule(
            StyleSelector::class("structural-nested-shell"),
            Style::default()
                .flex_direction(FlexDirection::Row)
                .flex_wrap(FlexWrap::NoWrap)
                .width(Length::Px(790.0))
                .height(Length::Auto)
                .gap(10.0)
                .background(PANEL_ALT),
        )
        .rule(
            StyleSelector::class("structural-item"),
            Style::default()
                .width_fill()
                .height(Length::Px(58.0))
                .padding(Insets::symmetric(10.0, 7.0))
                .background(SURFACE_CONTAINER)
                .border(STROKE)
                .radius(5.0),
        )
        .rule(
            StyleSelector::compound()
                .class("structural-item")
                .first_child()
                .selector(),
            Style::default().background(SUCCESS_CONTAINER).border(GREEN),
        )
        .rule(
            StyleSelector::compound()
                .class("structural-item")
                .nth_child(2)
                .selector(),
            Style::default()
                .background(PRIMARY_CONTAINER)
                .border(TEXT_ACCENT),
        )
        .rule(
            StyleSelector::compound()
                .class("structural-item")
                .nth_child(3)
                .selector(),
            Style::default().border_widths(Insets {
                top: 1.0,
                right: 1.0,
                bottom: 1.0,
                left: 5.0,
            }),
        )
        .rule(
            StyleSelector::compound()
                .class("structural-item")
                .last_child()
                .selector(),
            Style::default().border(PURPLE).radius(8.0),
        )
        .rule(
            StyleSelector::class("animation-grid"),
            Style::default()
                .flex_direction(FlexDirection::Row)
                .flex_wrap(FlexWrap::Wrap)
                .width_fill()
                .height(Length::Auto)
                .gap(12.0)
                .background(PANEL_ALT),
        )
        .rule(
            StyleSelector::class("animation-specimen"),
            Style::default()
                .width(Length::Px(318.0))
                .height(Length::Auto)
                .padding(Insets::all(8.0))
                .gap(3.0)
                .background(SURFACE_CONTAINER)
                .border(STROKE)
                .radius(5.0),
        )
        .rule(
            StyleSelector::class("animation-surface"),
            Style::default()
                .width_fill()
                .height(Length::Px(132.0))
                .padding(Insets::all(10.0))
                .background(PANEL)
                .border(STROKE),
        )
        .rule(
            StyleSelector::class("animation-box"),
            Style::default()
                .size(150.0, 58.0)
                .min_size(120.0, 44.0)
                .padding(Insets::all(8.0))
                .gap(4.0)
                .background(PRIMARY_CONTAINER)
                .border(STROKE_SELECTED)
                .border_width(2.0)
                .radius(4.0)
                .transition(Transition::ease_out(0.14)),
        )
        .rule(
            StyleSelector::class("animation-box-label"),
            Style::default().font_size(13.0).text_color(TEXT),
        )
        .rule(
            StyleSelector::class("animation-box-body"),
            Style::default().font_size(11.0).text_color(TEXT_MUTED),
        )
        .rule(
            StyleSelector::class("animation-margin-row"),
            Style::default()
                .flex_direction(FlexDirection::Row)
                .width_fill()
                .height(Length::Auto)
                .padding(Insets::all(8.0))
                .gap(0.0)
                .background(PANEL)
                .border(STROKE)
                .radius(3.0),
        )
        .rule(
            StyleSelector::class("animation-margin-chip"),
            Style::default()
                .size(48.0, 48.0)
                .background(STROKE)
                .border(STROKE)
                .border_width(2.0)
                .radius(4.0)
                .transition(Transition::ease_out(0.18)),
        )
        .rule(
            StyleSelector::class("animation-margin-reference"),
            Style::default().background(SURFACE_CONTAINER_HIGH),
        )
        .rule(
            StyleSelector::class_state("animation-box-hover-size", ElementStateSelector::Hovered),
            Style::default().size(220.0, 84.0),
        )
        .rule(
            StyleSelector::class_state("animation-box-hover-margin", ElementStateSelector::Hovered),
            Style::default()
                .margin(Insets::all(18.0))
                .background(SUCCESS_CONTAINER)
                .border(GREEN),
        )
        .rule(
            StyleSelector::class_state(
                "animation-box-pressed-border",
                ElementStateSelector::Pressed,
            ),
            Style::default().border_width(10.0).radius(22.0),
        )
        .rule(
            StyleSelector::class_state(
                "animation-box-selected-spacing",
                ElementStateSelector::Selected,
            ),
            Style::default()
                .size(210.0, 92.0)
                .padding(Insets::all(16.0))
                .margin(Insets::all(10.0))
                .gap(18.0)
                .background(SUCCESS_CONTAINER)
                .border(GREEN)
                .radius(12.0),
        )
        .rule(
            StyleSelector::class_state("animation-box-label", ElementStateSelector::Selected),
            Style::default().font_size(18.0),
        )
        .rule(
            StyleSelector::class_state(
                "animation-box-disabled-color",
                ElementStateSelector::Disabled,
            ),
            Style::default()
                .background(SURFACE_CONTAINER)
                .border(STROKE)
                .text_color(TEXT_MUTED),
        )
        .rule(
            StyleSelector::class_state("animation-box-label", ElementStateSelector::Disabled),
            Style::default().text_color(TEXT_MUTED),
        )
        .rule(
            StyleSelector::class_state("animation-box-body", ElementStateSelector::Disabled),
            Style::default().text_color(TEXT_MUTED),
        )
        .rule(
            StyleSelector::class_state(
                "animation-box-focused-min-size",
                ElementStateSelector::Focused,
            ),
            Style::default()
                .size(226.0, 88.0)
                .min_size(210.0, 78.0)
                .border_width(6.0)
                .border(PRIMARY_CONTAINER)
                .background(SECONDARY_CONTAINER)
                .radius(16.0),
        )
        .rule(
            StyleSelector::class("scroll-panel"),
            Style::default()
                .size(318.0, 300.0)
                .padding(Insets::all(10.0))
                .gap(7.0)
                .background(SURFACE_CONTAINER)
                .border(STROKE)
                .radius(7.0),
        )
        .rule(
            StyleSelector::class("scroll-list"),
            Style::default()
                .width_fill()
                .height(des_document::Length::Px(250.0))
                .padding(Insets::symmetric(4.0, 4.0))
                .gap(7.0)
                .overflow_y(Overflow::Scroll)
                .scrollbar_width(2.0),
        )
        .rule(
            StyleSelector::class("scroll-list-horizontal"),
            Style::default()
                .flex_direction(FlexDirection::Row)
                .width_fill()
                .height(des_document::Length::Px(250.0))
                .padding(Insets::symmetric(4.0, 4.0))
                .gap(7.0)
                .overflow_x(Overflow::Scroll)
                .overflow_y(Overflow::Visible)
                .scrollbar_width(2.0),
        )
        .rule(
            StyleSelector::class("scroll-list-two-axis"),
            Style::default()
                .width_fill()
                .height(des_document::Length::Px(250.0))
                .padding(Insets::symmetric(4.0, 4.0))
                .gap(7.0)
                .overflow(Overflow::Scroll)
                .scrollbar_width(2.0),
        )
        .rule(
            StyleSelector::class("scroll-list-nested"),
            Style::default().height(des_document::Length::Px(222.0)),
        )
        .rule(
            StyleSelector::class("scroll-nested-shell"),
            Style::default()
                .width_fill()
                .height(des_document::Length::Px(250.0))
                .padding(Insets::all(12.0))
                .background(PANEL)
                .border(STROKE)
                .radius(5.0),
        )
        .rule(
            StyleSelector::class_state("scroll-panel", ElementStateSelector::Hovered),
            Style::default().border(STROKE_SELECTED),
        )
        .rule(
            styled_scrollbar_selector().selector(),
            styled_scrollbar_style(),
        )
        .rule(
            StyleSelector::class("scroll-row-card"),
            Style::default()
                .width_fill()
                .height(des_document::Length::Px(34.0))
                .padding(Insets::symmetric(9.0, 7.0))
                .background(CARD)
                .border(STROKE)
                .radius(4.0),
        )
        .rule(
            StyleSelector::class("scroll-wide-row-card"),
            Style::default()
                .size(156.0, 214.0)
                .padding(Insets::symmetric(9.0, 7.0))
                .gap(7.0)
                .background(CARD)
                .border(STROKE)
                .radius(4.0),
        )
        .rule(
            StyleSelector::class("scroll-mini-list"),
            Style::default()
                .width_fill()
                .height(des_document::Length::Px(158.0))
                .padding(Insets::symmetric(3.0, 3.0))
                .gap(4.0)
                .background(SURFACE_CONTAINER)
                .border(STROKE)
                .radius(4.0)
                .overflow_y(Overflow::Scroll)
                .scrollbar_width(2.0),
        )
        .rule(
            StyleSelector::class("scroll-mini-row"),
            Style::default()
                .width_fill()
                .height(des_document::Length::Px(24.0))
                .padding(Insets::symmetric(6.0, 4.0))
                .background(SURFACE_CONTAINER_HIGH)
                .border(STROKE)
                .radius(3.0),
        )
        .rule(
            StyleSelector::class("scroll-xy-row-card"),
            Style::default()
                .width(des_document::Length::Px(430.0))
                .height(des_document::Length::Px(34.0))
                .padding(Insets::symmetric(9.0, 7.0))
                .background(CARD)
                .border(STROKE)
                .radius(4.0),
        )
        .rule(
            StyleSelector::class_state("scroll-row-card", ElementStateSelector::Hovered),
            Style::default()
                .background(CARD_HOVER)
                .border(STROKE_SELECTED),
        )
        .rule(
            StyleSelector::class_state("scroll-wide-row-card", ElementStateSelector::Hovered),
            Style::default()
                .background(CARD_HOVER)
                .border(STROKE_SELECTED),
        )
        .rule(
            StyleSelector::class_state("scroll-mini-row", ElementStateSelector::Hovered),
            Style::default()
                .background(SURFACE_CONTAINER_HIGH)
                .border(STROKE_SELECTED),
        )
        .rule(
            StyleSelector::class_state("scroll-xy-row-card", ElementStateSelector::Hovered),
            Style::default()
                .background(CARD_HOVER)
                .border(STROKE_SELECTED),
        )
        .rule(
            StyleSelector::class("floating-playground"),
            Style::default()
                .width_fill()
                .height(Length::Auto)
                .flex_direction(FlexDirection::Row)
                .flex_wrap(FlexWrap::Wrap)
                .padding(Insets::all(10.0))
                .gap(10.0)
                .background(PANEL_ALT),
        )
        .rule(
            StyleSelector::class("floating-specimen-box"),
            Style::default()
                .width(Length::calc(0.33333334, -6.666667))
                .flex_basis(Length::calc(0.33333334, -6.666667))
                .flex_shrink(0.0)
                .height(Length::Px(160.0))
                .padding(Insets {
                    top: 8.0,
                    right: 12.0,
                    bottom: 10.0,
                    left: 12.0,
                })
                .gap(6.0)
                .background(SURFACE_CONTAINER)
                .border(STROKE)
                .radius(8.0),
        )
        .rule(
            StyleSelector::class("floating-main-axis-specimen"),
            Style::default().height(Length::Px(210.0)),
        )
        .rule(
            StyleSelector::class("floating-centered-axis-specimen"),
            Style::default().height(Length::Px(160.0)),
        )
        .rule(
            StyleSelector::class("floating-offset-row"),
            Style::default()
                .width_fill()
                .height(Length::Px(112.0))
                .flex_direction(FlexDirection::Row)
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::Center)
                .gap(24.0),
        )
        .rule(
            StyleSelector::class("floating-main-axis-stack"),
            Style::default()
                .width_fill()
                .height(Length::Auto)
                .padding(Insets {
                    top: 10.0,
                    right: 0.0,
                    bottom: 0.0,
                    left: 0.0,
                })
                .gap(36.0),
        )
        .rule(
            StyleSelector::class("floating-main-axis-row"),
            Style::default()
                .width_fill()
                .height(Length::Auto)
                .flex_direction(FlexDirection::Row)
                .justify_content(JustifyContent::Center)
                .gap(22.0),
        )
        .rule(
            StyleSelector::class("floating-centered-axis-row"),
            Style::default()
                .width_fill()
                .height_fill()
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::Center),
        )
        .rule(
            StyleSelector::class("floating-scroll-shift-panel"),
            Style::default()
                .width_fill()
                .height(Length::Px(148.0))
                .overflow_x(Overflow::Scroll)
                .overflow_y(Overflow::Visible)
                .scrollbar_visible(true)
                .border(STROKE)
                .radius(8.0),
        )
        .rule(
            StyleSelector::class("floating-scroll-shift-track"),
            Style::default()
                .width(Length::Px(620.0))
                .height(Length::Px(108.0))
                .flex_shrink(0.0),
        )
        .rule(
            StyleSelector::class("floating-vertical-overlap-panel"),
            Style::default()
                .width_fill()
                .height(Length::Px(148.0))
                .overflow_x(Overflow::Visible)
                .overflow_y(Overflow::Scroll)
                .scrollbar_visible(true)
                .border(STROKE)
                .radius(8.0),
        )
        .rule(
            StyleSelector::class("floating-vertical-overlap-track"),
            Style::default()
                .width_fill()
                .height(Length::Px(420.0))
                .flex_shrink(0.0),
        )
        .rule(
            StyleSelector::class("floating-vertical-flip-panel"),
            Style::default()
                .width_fill()
                .height(Length::Px(148.0))
                .overflow_x(Overflow::Visible)
                .overflow_y(Overflow::Scroll)
                .scrollbar_visible(true)
                .border(STROKE)
                .radius(8.0),
        )
        .rule(
            StyleSelector::class("floating-vertical-flip-track"),
            Style::default()
                .width_fill()
                .height(Length::Px(420.0))
                .flex_shrink(0.0),
        )
        .rule(
            StyleSelector::class("floating-edge-flip-panel"),
            Style::default()
                .width_fill()
                .height(Length::Px(148.0))
                .overflow_x(Overflow::Scroll)
                .overflow_y(Overflow::Scroll)
                .scrollbar_visible(true)
                .border(STROKE)
                .radius(8.0),
        )
        .rule(
            StyleSelector::class("floating-edge-flip-track"),
            Style::default()
                .width(Length::Px(560.0))
                .height(Length::Px(340.0))
                .flex_shrink(0.0),
        )
        .rule(
            StyleSelector::class("floating-offset-reference"),
            Style::default()
                .width(Length::Px(56.0))
                .height(Length::Px(56.0))
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::Center)
                .background(Color::rgba(255, 255, 255, 0))
                .border(Color::rgba(30, 31, 38, 255))
                .border_width(2.0)
                .border_style(BorderStyle::Dashed)
                .radius(0.0),
        )
        .rule(
            StyleSelector::class("floating-main-axis-reference"),
            Style::default(),
        )
        .rule(
            StyleSelector::class("floating-alignment-axis-reference"),
            Style::default().gap(3.0),
        )
        .rule(
            StyleSelector::class("floating-offset-reference-label"),
            Style::default()
                .font_size(9.0)
                .text_color(Color::rgba(30, 31, 38, 255)),
        )
        .rule(
            StyleSelector::class("floating-alignment-axis-placement-label"),
            Style::default()
                .font_size(8.0)
                .text_color(Color::rgba(30, 31, 38, 255)),
        )
        .rule(
            StyleSelector::class("floating-alignment-axis-axis-label"),
            Style::default()
                .font_size(7.0)
                .text_color(Color::rgba(30, 31, 38, 255)),
        )
        .rule(
            StyleSelector::class("floating-offset-popover"),
            Style::default()
                .width(Length::Px(52.0))
                .height(Length::Px(22.0))
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::Center)
                .background(Color::rgba(244, 52, 87, 255))
                .border(Color::rgba(244, 52, 87, 255))
                .border_width(1.0)
                .radius(0.0)
                .z_index(2300),
        )
        .rule(
            StyleSelector::class("floating-offset-popover-label"),
            Style::default()
                .font_size(9.0)
                .text_color(Color::rgba(255, 255, 255, 255)),
        )
        .rule(
            StyleSelector::id("floating-offset-zero-popover"),
            Style::default()
                .floating_to("floating-offset-zero-reference")
                .floating_placement(FloatingPlacement::Bottom)
                .floating_offset(0.0, 0.0),
        )
        .rule(
            StyleSelector::id("floating-offset-ten-popover"),
            Style::default()
                .floating_to("floating-offset-ten-reference")
                .floating_placement(FloatingPlacement::Bottom)
                .floating_offset(10.0, 0.0),
        )
        .rule(
            StyleSelector::id("floating-main-axis-top-popover"),
            Style::default()
                .floating_to("floating-main-axis-top-reference")
                .floating_placement(FloatingPlacement::Top),
        )
        .rule(
            StyleSelector::id("floating-main-axis-bottom-popover"),
            Style::default()
                .floating_to("floating-main-axis-bottom-reference")
                .floating_placement(FloatingPlacement::Bottom),
        )
        .rule(
            StyleSelector::id("floating-main-axis-left-popover"),
            Style::default()
                .floating_to("floating-main-axis-left-reference")
                .floating_placement(FloatingPlacement::Left),
        )
        .rule(
            StyleSelector::id("floating-main-axis-right-popover"),
            Style::default()
                .floating_to("floating-main-axis-right-reference")
                .floating_placement(FloatingPlacement::Right),
        )
        .rule(
            StyleSelector::id("floating-cross-axis-top-popover"),
            Style::default()
                .floating_to("floating-cross-axis-top-reference")
                .floating_placement(FloatingPlacement::TopEnd),
        )
        .rule(
            StyleSelector::id("floating-cross-axis-bottom-popover"),
            Style::default()
                .floating_to("floating-cross-axis-bottom-reference")
                .floating_placement(FloatingPlacement::BottomEnd),
        )
        .rule(
            StyleSelector::id("floating-cross-axis-left-popover"),
            Style::default()
                .floating_to("floating-cross-axis-left-reference")
                .floating_placement(FloatingPlacement::LeftEnd),
        )
        .rule(
            StyleSelector::id("floating-cross-axis-right-popover"),
            Style::default()
                .floating_to("floating-cross-axis-right-reference")
                .floating_placement(FloatingPlacement::RightEnd),
        )
        .rule(
            StyleSelector::id("floating-alignment-axis-cross-start-popover"),
            Style::default()
                .floating_to("floating-alignment-axis-cross-start-reference")
                .floating_placement(FloatingPlacement::TopStart)
                .floating_offset(0.0, 8.0),
        )
        .rule(
            StyleSelector::id("floating-alignment-axis-cross-end-popover"),
            Style::default()
                .floating_to("floating-alignment-axis-cross-end-reference")
                .floating_placement(FloatingPlacement::TopEnd)
                .floating_offset(0.0, 8.0),
        )
        .rule(
            StyleSelector::id("floating-alignment-axis-start-popover"),
            Style::default()
                .floating_to("floating-alignment-axis-start-reference")
                .floating_placement(FloatingPlacement::TopStart)
                .floating_alignment_axis(8.0),
        )
        .rule(
            StyleSelector::id("floating-alignment-axis-end-popover"),
            Style::default()
                .floating_to("floating-alignment-axis-end-reference")
                .floating_placement(FloatingPlacement::TopEnd)
                .floating_alignment_axis(8.0),
        )
        .rule(
            StyleSelector::id("floating-centered-axis-popover"),
            Style::default()
                .floating_to("floating-centered-axis-reference")
                .floating_placement(FloatingPlacement::Center),
        )
        .rule(
            StyleSelector::id("floating-top-start-popover"),
            Style::default()
                .floating_to("floating-top-start-reference")
                .floating_placement(FloatingPlacement::TopStart)
                .floating_alignment_axis_offset(FloatingAxisOffset::floating_width(-1.0)),
        )
        .rule(
            StyleSelector::id("floating-scroll-shift-reference"),
            Style::default().margin(Insets {
                top: 50.0,
                right: 0.0,
                bottom: 0.0,
                left: 520.0,
            }),
        )
        .rule(
            StyleSelector::id("floating-scroll-attach-reference"),
            Style::default().margin(Insets {
                top: 50.0,
                right: 0.0,
                bottom: 0.0,
                left: 280.0,
            }),
        )
        .rule(
            StyleSelector::id("floating-vertical-overlap-reference"),
            Style::default().margin(Insets {
                top: 92.0,
                right: 0.0,
                bottom: 0.0,
                left: 118.0,
            }),
        )
        .rule(
            StyleSelector::id("floating-vertical-flip-reference"),
            Style::default().margin(Insets {
                top: 132.0,
                right: 0.0,
                bottom: 0.0,
                left: 118.0,
            }),
        )
        .rule(
            StyleSelector::id("floating-edge-flip-reference"),
            Style::default().margin(Insets {
                top: 132.0,
                right: 0.0,
                bottom: 0.0,
                left: 118.0,
            }),
        )
        .rule(
            StyleSelector::id("floating-scroll-shift-popover"),
            Style::default()
                .floating_to("floating-scroll-shift-reference")
                .floating_placement(FloatingPlacement::Bottom)
                .floating_boundary_to("floating-scroll-shift-panel")
                .floating_shift(FloatingShift::new(false, true)),
        )
        .rule(
            StyleSelector::id("floating-scroll-attach-popover"),
            Style::default()
                .floating_to("floating-scroll-attach-reference")
                .floating_placement(FloatingPlacement::Bottom),
        )
        .rule(
            StyleSelector::id("floating-vertical-overlap-popover"),
            Style::default()
                .floating_to("floating-vertical-overlap-reference")
                .floating_placement(FloatingPlacement::Top)
                .floating_boundary_to("floating-vertical-overlap-panel")
                .floating_shift(FloatingShift::new(true, false)),
        )
        .rule(
            StyleSelector::id("floating-vertical-flip-popover"),
            Style::default()
                .floating_to("floating-vertical-flip-reference")
                .floating_placement(FloatingPlacement::Bottom)
                .floating_boundary_to("floating-vertical-flip-panel")
                .floating_flip(true),
        )
        .rule(
            StyleSelector::id("floating-edge-flip-popover"),
            Style::default()
                .floating_to("floating-edge-flip-reference")
                .floating_placement(FloatingPlacement::BottomStart)
                .floating_boundary_to("floating-edge-flip-panel")
                .floating_flip(true),
        )
        .rule(
            StyleSelector::class("nest-outer"),
            Style::default()
                .size(650.0, 430.0)
                .padding(Insets::all(28.0))
                .gap(16.0)
                .background(SURFACE_CONTAINER)
                .border(STROKE)
                .radius(8.0),
        )
        .rule(
            StyleSelector::class("nest-middle"),
            Style::default()
                .size(500.0, 270.0)
                .padding(Insets::all(24.0))
                .gap(14.0)
                .background(PRIMARY_CONTAINER)
                .border(STROKE_SELECTED)
                .radius(7.0),
        )
        .rule(
            StyleSelector::class("nest-inner"),
            Style::default()
                .size(360.0, 130.0)
                .padding(Insets::all(18.0))
                .gap(6.0)
                .background(SECONDARY_CONTAINER)
                .border(PURPLE)
                .radius(7.0),
        )
        .rule(
            StyleSelector::class_state("nest-inner", ElementStateSelector::Hovered),
            Style::default()
                .background(SECONDARY_CONTAINER)
                .border(TEXT_ACCENT),
        )
        .rule(
            StyleSelector::class("canvas-placeholder"),
            Style::default()
                .size(720.0, 360.0)
                .padding(Insets::all(18.0))
                .gap(8.0)
                .background(PANEL)
                .border(STROKE)
                .radius(7.0),
        )
        .rule(
            StyleSelector::class("debug-overlay-root"),
            Style::default()
                .size(1320.0, 780.0)
                .background(Color::rgba(0, 0, 0, 0)),
        )
        .rule(
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
        )
        .rule(
            StyleSelector::class("debug-overlay-title"),
            Style::default()
                .font_size(14.0)
                .text_color(TEXT)
                .height(Length::Px(18.0)),
        )
        .rule(
            StyleSelector::class("debug-row"),
            Style::default()
                .flex_direction(FlexDirection::Row)
                .justify_content(JustifyContent::SpaceBetween)
                .width_fill()
                .height(Length::Px(18.0))
                .background(Color::rgba(0, 0, 0, 0)),
        )
        .rule(
            StyleSelector::class("debug-label"),
            Style::default()
                .font_size(12.0)
                .text_color(TEXT_MUTED)
                .height(Length::Px(16.0)),
        )
        .rule(
            StyleSelector::class("debug-value"),
            Style::default()
                .font_size(12.0)
                .text_color(TEXT)
                .height(Length::Px(16.0)),
        )
        .rule(
            StyleSelector::class("title"),
            Style::default().font_size(21.0).text_color(TEXT),
        )
        .rule(
            StyleSelector::class("heading"),
            Style::default().font_size(24.0).text_color(TEXT),
        )
        .rule(
            StyleSelector::class("section-title"),
            Style::default().font_size(13.0).text_color(TEXT_ACCENT),
        )
        .rule(
            StyleSelector::class("card-title"),
            Style::default()
                .font_size(16.0)
                .line_height(18.0)
                .height(Length::Px(18.0))
                .text_color(TEXT),
        )
        .rule(
            StyleSelector::class("muted"),
            Style::default().font_size(12.5).text_color(TEXT_MUTED),
        )
        .rule(
            StyleSelector::id_state("interaction-card-two", ElementStateSelector::Hovered),
            Style::default()
                .border(GREEN)
                .transition(Transition::ease_out(0.24)),
        )
        .rule(
            StyleSelector::id("interaction-card-three"),
            Style::default().transition(Transition::ease_out(0.06)),
        )
        .rule(
            StyleSelector::id_state("interaction-card-three", ElementStateSelector::Pressed),
            Style::default()
                .background(SECONDARY_CONTAINER)
                .border(PURPLE),
        );
    stylesheet.extend(framework::stylesheet());
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
    stylesheet
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
