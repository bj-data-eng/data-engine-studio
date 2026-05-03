use super::{
    BACKGROUND, CARD, CARD_HOVER, CARD_PRESSED, CARD_SELECTED, GREEN, PANEL, PANEL_ALT, PURPLE,
    STROKE, STROKE_SELECTED, TEXT, TEXT_ACCENT, TEXT_MUTED,
};
use des_ui_document::{
    AlignItems, Color, Direction, ElementRole, ElementStateSelector, Insets, JustifyContent,
    Length, Overflow, Style, StyleSelector, StyleSheet, Transition,
};

pub(super) fn stylesheet() -> StyleSheet {
    StyleSheet::new()
        .rule(
            StyleSelector::Role(ElementRole::Root),
            Style::default()
                .direction(Direction::Column)
                .background(BACKGROUND),
        )
        .rule(
            StyleSelector::Role(ElementRole::Panel),
            Style::default()
                .direction(Direction::Column)
                .background(PANEL),
        )
        .rule(
            StyleSelector::Role(ElementRole::Card),
            Style::default()
                .direction(Direction::Column)
                .padding(Insets::all(12.0))
                .gap(5.0)
                .background(CARD)
                .border(STROKE)
                .radius(7.0),
        )
        .rule(
            StyleSelector::Role(ElementRole::Control),
            Style::default()
                .padding(Insets::symmetric(12.0, 7.0))
                .background(CARD)
                .border(STROKE)
                .radius(5.0),
        )
        .rule(
            StyleSelector::Role(ElementRole::Checkbox),
            Style::default()
                .direction(Direction::Row)
                .align_items(AlignItems::Center)
                .padding(Insets::symmetric(9.0, 7.0))
                .gap(8.0)
                .background(CARD)
                .border(STROKE)
                .radius(5.0),
        )
        .rule(
            StyleSelector::Role(ElementRole::Radio),
            Style::default()
                .direction(Direction::Row)
                .align_items(AlignItems::Center)
                .padding(Insets::symmetric(9.0, 7.0))
                .gap(8.0)
                .background(CARD)
                .border(STROKE)
                .radius(5.0),
        )
        .rule(
            StyleSelector::Role(ElementRole::Dropdown),
            Style::default()
                .direction(Direction::Row)
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::SpaceBetween)
                .padding(Insets::symmetric(10.0, 7.0))
                .gap(8.0)
                .background(CARD)
                .border(STROKE)
                .radius(5.0),
        )
        .rule(
            StyleSelector::Role(ElementRole::TextInput),
            Style::default()
                .padding(Insets::symmetric(10.0, 7.0))
                .background(Color::rgb(13, 16, 19))
                .border(STROKE)
                .radius(5.0),
        )
        .rule(
            StyleSelector::Role(ElementRole::Icon),
            Style::default()
                .size(14.0, 14.0)
                .font_size(14.0)
                .text_color(TEXT_MUTED),
        )
        .rule(
            StyleSelector::Role(ElementRole::Text),
            Style::default().font_size(13.0).text_color(TEXT),
        )
        .rule(
            StyleSelector::class("lab-root"),
            Style::default()
                .size(1320.0, 780.0)
                .background(BACKGROUND)
                .gap(0.0),
        )
        .rule(
            StyleSelector::class("topbar"),
            Style::default()
                .size(1320.0, 58.0)
                .padding(Insets::symmetric(18.0, 10.0))
                .gap(3.0)
                .background(Color::rgb(22, 26, 30)),
        )
        .rule(
            StyleSelector::class("lab-body"),
            Style::default()
                .direction(Direction::Row)
                .size(1320.0, 722.0)
                .padding(Insets::all(14.0))
                .gap(14.0)
                .background(BACKGROUND),
        )
        .rule(
            StyleSelector::class("nav"),
            Style::default()
                .size(242.0, 690.0)
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
                .size(1036.0, 690.0)
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
                .direction(Direction::Row)
                .wrap(true)
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
                .background(Color::rgb(20, 24, 28))
                .border(Color::rgb(45, 54, 62))
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
                .background(Color::rgb(13, 16, 19))
                .border(Color::rgb(39, 48, 56))
                .overflow_y(Overflow::Visible),
        )
        .rule(
            StyleSelector::class("box-subject"),
            Style::default()
                .size(32.0, 32.0)
                .gap(0.0)
                .padding(Insets::ZERO)
                .background(Color::rgb(65, 121, 164)),
        )
        .rule(
            StyleSelector::class("box-chip"),
            Style::default()
                .size(12.0, 12.0)
                .background(Color::rgb(141, 207, 164)),
        )
        .rule(
            StyleSelector::class("box-overflow-child"),
            Style::default()
                .size(112.0, 112.0)
                .background(Color::rgb(218, 151, 77)),
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
                .background(Color::rgb(141, 207, 164)),
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
                .direction(Direction::Row)
                .width(Length::Auto)
                .height(Length::Auto)
                .gap(10.0),
        )
        .rule(
            StyleSelector::class("box-subject-column-gap"),
            Style::default()
                .direction(Direction::Column)
                .width(Length::Auto)
                .height(Length::Auto)
                .gap(6.0),
        )
        .rule(
            StyleSelector::class("box-subject-row-align"),
            Style::default()
                .direction(Direction::Row)
                .size(96.0, 54.0)
                .gap(8.0)
                .justify_content(JustifyContent::Center)
                .align_items(AlignItems::End),
        )
        .rule(
            StyleSelector::class("box-subject-column-align"),
            Style::default()
                .direction(Direction::Column)
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
                .background(Color::rgb(41, 58, 73)),
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
                .background(Color::rgb(52, 72, 88)),
        )
        .rule(
            StyleSelector::class("box-nested-row"),
            Style::default()
                .direction(Direction::Row)
                .width(Length::Auto)
                .height(Length::Auto)
                .gap(4.0)
                .background(Color::rgb(65, 121, 164)),
        )
        .rule(
            StyleSelector::class("box-nested-cell"),
            Style::default()
                .size(10.0, 10.0)
                .background(Color::rgb(141, 207, 164)),
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
                .background(Color::rgb(65, 121, 164)),
        )
        .rule(
            StyleSelector::class("box-inset-percent-child"),
            Style::default()
                .width_percent(0.5)
                .height_percent(0.5)
                .background(Color::rgb(141, 207, 164)),
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
                .background(Color::rgb(65, 121, 164)),
        )
        .rule(
            StyleSelector::class("box-absolute-flow-child"),
            Style::default()
                .size(16.0, 16.0)
                .background(Color::rgb(77, 136, 179)),
        )
        .rule(
            StyleSelector::class("box-absolute-parent-child"),
            Style::default()
                .absolute_parent()
                .top(Length::Px(8.0))
                .left(Length::Px(14.0))
                .size(26.0, 26.0)
                .z_index(2)
                .background(Color::rgb(141, 207, 164))
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
                .background(Color::rgb(45, 58, 76)),
        )
        .rule(
            StyleSelector::class("box-absolute-window-child"),
            Style::default()
                .absolute_viewport()
                .top(Length::Px(140.0))
                .left(Length::Px(420.0))
                .size(26.0, 26.0)
                .z_index(20)
                .background(Color::rgb(155, 129, 255))
                .border(PURPLE),
        )
        .rule(
            StyleSelector::class("nav-item"),
            Style::default()
                .width_fill()
                .height(des_ui_document::Length::Px(64.0))
                .background(CARD)
                .border(STROKE),
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
                .direction(Direction::Row)
                .gap(8.0)
                .background(PANEL_ALT),
        )
        .rule(
            StyleSelector::class("button"),
            Style::default()
                .size(156.0, 36.0)
                .background(Color::rgb(38, 43, 48))
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
                .direction(Direction::Row)
                .gap(12.0)
                .background(PANEL_ALT),
        )
        .rule(
            StyleSelector::class("card-row-dense"),
            Style::default()
                .direction(Direction::Row)
                .gap(6.0)
                .background(PANEL_ALT),
        )
        .rule(
            StyleSelector::class("controls-grid"),
            Style::default()
                .direction(Direction::Row)
                .wrap(true)
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
                .gap(8.0),
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
                .background(Color::rgb(16, 20, 24))
                .border(STROKE)
                .radius(4.0),
        )
        .rule(
            StyleSelector::class_state("checkbox-mark", ElementStateSelector::Selected),
            Style::default()
                .background(STROKE_SELECTED)
                .border(Color::rgb(126, 190, 255)),
        )
        .rule(
            StyleSelector::class("check-glyph"),
            Style::default()
                .size(13.0, 13.0)
                .font_size(13.0)
                .text_color(Color::rgb(244, 248, 252)),
        )
        .rule(
            StyleSelector::class("radio-dot"),
            Style::default()
                .size(18.0, 18.0)
                .background(Color::rgb(16, 20, 24))
                .border(STROKE)
                .border_width(2.0)
                .radius(9.0),
        )
        .rule(
            StyleSelector::class_state("radio-dot", ElementStateSelector::Selected),
            Style::default()
                .background(STROKE_SELECTED)
                .border(Color::rgb(160, 212, 255))
                .border_width(5.0),
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
            Style::default().text_color(Color::rgb(96, 104, 112)),
        )
        .rule(
            StyleSelector::class("dropdown-control"),
            Style::default().width_fill().height(Length::Px(38.0)),
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
                .background(Color::rgb(18, 22, 26))
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
            Style::default()
                .background(Color::rgb(21, 24, 27))
                .border(Color::rgb(40, 46, 52)),
        )
        .rule(
            StyleSelector::class("loop-grid"),
            Style::default()
                .direction(Direction::Row)
                .wrap(true)
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
                .gap(8.0),
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
            Style::default().background(Color::rgb(42, 74, 102)),
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
            Style::default()
                .background(Color::rgb(31, 52, 43))
                .border(GREEN),
        )
        .rule(
            StyleSelector::class_state("loop-result-card", ElementStateSelector::Focused),
            Style::default()
                .background(Color::rgb(39, 35, 62))
                .border(PURPLE),
        )
        .rule(
            StyleSelector::class("loop-runtime-local"),
            Style::default().border(Color::rgb(88, 157, 230)),
        )
        .rule(
            StyleSelector::class("loop-runtime-remote"),
            Style::default().border(Color::rgb(95, 204, 140)),
        )
        .rule(
            StyleSelector::class("loop-runtime-hybrid"),
            Style::default().border(Color::rgb(151, 93, 219)),
        )
        .rule(
            StyleSelector::class("loop-source-csv"),
            Style::default().background(Color::rgb(31, 43, 55)),
        )
        .rule(
            StyleSelector::class("loop-source-duckdb"),
            Style::default().background(Color::rgb(42, 39, 31)),
        )
        .rule(
            StyleSelector::class("loop-source-python"),
            Style::default().background(Color::rgb(38, 32, 48)),
        )
        .rule(
            StyleSelector::class("drag-grid"),
            Style::default()
                .direction(Direction::Row)
                .wrap(true)
                .width(Length::Px(520.0))
                .height(Length::Auto)
                .padding(Insets::all(8.0))
                .gap(8.0)
                .background(Color::rgb(17, 21, 25))
                .border(Color::rgb(42, 50, 58))
                .radius(6.0)
                .transition(Transition::ease_out(0.12)),
        )
        .rule(
            StyleSelector::class("drag-cell"),
            Style::default()
                .width(Length::Px(244.0))
                .height(Length::Auto)
                .min_size(0.0, 70.0)
                .padding(Insets::all(7.0))
                .gap(5.0)
                .background(Color::rgb(20, 25, 30))
                .border(Color::rgb(48, 57, 65))
                .radius(5.0)
                .transition(Transition::ease_out(0.12)),
        )
        .rule(
            StyleSelector::class_state("drag-cell", ElementStateSelector::Hovered),
            Style::default()
                .background(Color::rgb(25, 32, 38))
                .border(Color::rgb(88, 157, 230)),
        )
        .rule(
            StyleSelector::class("drag-item"),
            Style::default()
                .direction(Direction::Row)
                .width_fill()
                .height(Length::Px(34.0))
                .padding(Insets::symmetric(9.0, 6.0))
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::SpaceBetween)
                .background(CARD_SELECTED)
                .border(STROKE_SELECTED)
                .radius(5.0)
                .transition(Transition::ease_out(0.14)),
        )
        .rule(
            StyleSelector::class("drag-origin-space"),
            Style::default()
                .background(Color::rgba(0, 0, 0, 0))
                .border(Color::rgba(0, 0, 0, 0))
                .text_color(Color::rgba(0, 0, 0, 0)),
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
            StyleSelector::class_state("drag-handle", ElementStateSelector::Hovered),
            Style::default().background(Color::rgba(232, 236, 240, 24)),
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
            StyleSelector::class_state("drag-item", ElementStateSelector::Hovered),
            Style::default().background(Color::rgb(42, 74, 102)),
        )
        .rule(
            StyleSelector::class_state("drag-item", ElementStateSelector::Pressed),
            Style::default().background(CARD_PRESSED),
        )
        .rule(
            StyleSelector::class_state("drag-origin-space", ElementStateSelector::Hovered),
            Style::default()
                .background(Color::rgba(0, 0, 0, 0))
                .border(Color::rgba(0, 0, 0, 0)),
        )
        .rule(
            StyleSelector::class_state("drag-origin-space", ElementStateSelector::Pressed),
            Style::default()
                .background(Color::rgba(0, 0, 0, 0))
                .border(Color::rgba(0, 0, 0, 0)),
        )
        .rule(
            StyleSelector::class("drag-item-active"),
            Style::default()
                .background(Color::rgb(45, 37, 68))
                .border(PURPLE),
        )
        .rule(
            StyleSelector::class("drag-overlay"),
            Style::default()
                .width(Length::Px(230.0))
                .height(Length::Px(34.0))
                .z_index(100)
                .transition(Transition::ease_out(0.08)),
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
                .background(Color::rgb(25, 30, 34))
                .border(STROKE)
                .radius(5.0),
        )
        .rule(
            StyleSelector::class("specificity-proof"),
            Style::default()
                .background(Color::rgb(30, 37, 43))
                .border(Color::rgb(80, 91, 103)),
        )
        .rule(
            StyleSelector::class_state("specificity-proof", ElementStateSelector::Hovered),
            Style::default()
                .background(Color::rgb(38, 55, 64))
                .border(GREEN),
        )
        .rule(
            StyleSelector::id("style-row-state"),
            Style::default().border(PURPLE),
        )
        .rule(
            StyleSelector::id_state("style-row-state", ElementStateSelector::Hovered),
            Style::default()
                .background(Color::rgb(50, 41, 68))
                .border(TEXT_ACCENT),
        )
        .rule(
            StyleSelector::class("structural-grid"),
            Style::default()
                .direction(Direction::Row)
                .wrap(true)
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
                .background(Color::rgb(17, 21, 25))
                .border(Color::rgb(43, 52, 60))
                .radius(6.0),
        )
        .rule(
            StyleSelector::class("structural-nested-shell"),
            Style::default()
                .direction(Direction::Row)
                .wrap(false)
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
                .background(Color::rgb(25, 30, 34))
                .border(STROKE)
                .radius(5.0),
        )
        .rule(
            StyleSelector::compound()
                .class("structural-item")
                .first_child()
                .selector(),
            Style::default()
                .background(Color::rgb(24, 53, 42))
                .border(GREEN),
        )
        .rule(
            StyleSelector::compound()
                .class("structural-item")
                .nth_child(2)
                .selector(),
            Style::default()
                .background(Color::rgb(29, 55, 80))
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
                .direction(Direction::Row)
                .wrap(true)
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
                .background(Color::rgb(20, 24, 28))
                .border(Color::rgb(45, 54, 62))
                .radius(5.0),
        )
        .rule(
            StyleSelector::class("animation-surface"),
            Style::default()
                .width_fill()
                .height(Length::Px(132.0))
                .padding(Insets::all(10.0))
                .background(Color::rgb(13, 16, 19))
                .border(Color::rgb(39, 48, 56)),
        )
        .rule(
            StyleSelector::class("animation-box"),
            Style::default()
                .size(150.0, 58.0)
                .min_size(120.0, 44.0)
                .padding(Insets::all(8.0))
                .gap(4.0)
                .background(Color::rgb(35, 56, 78))
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
                .direction(Direction::Row)
                .width_fill()
                .height(Length::Auto)
                .padding(Insets::all(8.0))
                .gap(0.0)
                .background(Color::rgb(17, 21, 24))
                .border(Color::rgb(48, 58, 66))
                .radius(3.0),
        )
        .rule(
            StyleSelector::class("animation-margin-chip"),
            Style::default()
                .size(48.0, 48.0)
                .background(Color::rgb(39, 48, 56))
                .border(Color::rgb(70, 82, 92))
                .border_width(2.0)
                .radius(4.0)
                .transition(Transition::ease_out(0.18)),
        )
        .rule(
            StyleSelector::class("animation-margin-reference"),
            Style::default().background(Color::rgb(31, 37, 43)),
        )
        .rule(
            StyleSelector::class_state("animation-box-hover-size", ElementStateSelector::Hovered),
            Style::default().size(220.0, 84.0),
        )
        .rule(
            StyleSelector::class_state("animation-box-hover-margin", ElementStateSelector::Hovered),
            Style::default()
                .margin(Insets::all(18.0))
                .background(Color::rgb(66, 91, 58))
                .border(Color::rgb(168, 224, 137)),
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
                .background(Color::rgb(43, 76, 82))
                .border(Color::rgb(104, 222, 171))
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
                .background(Color::rgb(28, 31, 34))
                .border(Color::rgb(77, 83, 90))
                .text_color(TEXT_MUTED),
        )
        .rule(
            StyleSelector::class_state("animation-box-label", ElementStateSelector::Disabled),
            Style::default().text_color(TEXT_MUTED),
        )
        .rule(
            StyleSelector::class_state("animation-box-body", ElementStateSelector::Disabled),
            Style::default().text_color(Color::rgb(115, 124, 132)),
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
                .border(Color::rgb(155, 129, 255))
                .background(Color::rgb(48, 38, 79))
                .radius(16.0),
        )
        .rule(
            StyleSelector::class("scroll-panel"),
            Style::default()
                .size(318.0, 300.0)
                .padding(Insets::all(10.0))
                .gap(7.0)
                .background(Color::rgb(20, 24, 28))
                .border(STROKE)
                .radius(7.0),
        )
        .rule(
            StyleSelector::class("scroll-list"),
            Style::default()
                .width_fill()
                .height(des_ui_document::Length::Px(250.0))
                .padding(Insets::symmetric(4.0, 4.0))
                .gap(7.0)
                .overflow_y(Overflow::Scroll)
                .scrollbar_width(2.0),
        )
        .rule(
            StyleSelector::class("scroll-list-horizontal"),
            Style::default()
                .direction(Direction::Row)
                .width_fill()
                .height(des_ui_document::Length::Px(250.0))
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
                .height(des_ui_document::Length::Px(250.0))
                .padding(Insets::symmetric(4.0, 4.0))
                .gap(7.0)
                .overflow(Overflow::Scroll)
                .scrollbar_width(2.0),
        )
        .rule(
            StyleSelector::class("scroll-list-nested"),
            Style::default().height(des_ui_document::Length::Px(222.0)),
        )
        .rule(
            StyleSelector::class("scroll-nested-shell"),
            Style::default()
                .width_fill()
                .height(des_ui_document::Length::Px(250.0))
                .padding(Insets::all(12.0))
                .background(Color::rgb(13, 16, 19))
                .border(Color::rgb(39, 48, 56))
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
                .height(des_ui_document::Length::Px(34.0))
                .padding(Insets::symmetric(9.0, 7.0))
                .background(Color::rgb(29, 34, 39))
                .border(Color::rgb(48, 57, 65))
                .radius(4.0),
        )
        .rule(
            StyleSelector::class("scroll-wide-row-card"),
            Style::default()
                .size(156.0, 214.0)
                .padding(Insets::symmetric(9.0, 7.0))
                .gap(7.0)
                .background(Color::rgb(29, 34, 39))
                .border(Color::rgb(48, 57, 65))
                .radius(4.0),
        )
        .rule(
            StyleSelector::class("scroll-mini-list"),
            Style::default()
                .width_fill()
                .height(des_ui_document::Length::Px(158.0))
                .padding(Insets::symmetric(3.0, 3.0))
                .gap(4.0)
                .background(Color::rgb(20, 24, 28))
                .border(Color::rgb(43, 52, 60))
                .radius(4.0)
                .overflow_y(Overflow::Scroll)
                .scrollbar_width(2.0),
        )
        .rule(
            StyleSelector::class("scroll-mini-row"),
            Style::default()
                .width_fill()
                .height(des_ui_document::Length::Px(24.0))
                .padding(Insets::symmetric(6.0, 4.0))
                .background(Color::rgb(24, 30, 35))
                .border(Color::rgb(45, 54, 62))
                .radius(3.0),
        )
        .rule(
            StyleSelector::class("scroll-xy-row-card"),
            Style::default()
                .width(des_ui_document::Length::Px(430.0))
                .height(des_ui_document::Length::Px(34.0))
                .padding(Insets::symmetric(9.0, 7.0))
                .background(Color::rgb(29, 34, 39))
                .border(Color::rgb(48, 57, 65))
                .radius(4.0),
        )
        .rule(
            StyleSelector::class_state("scroll-row-card", ElementStateSelector::Hovered),
            Style::default()
                .background(Color::rgb(38, 47, 54))
                .border(STROKE_SELECTED),
        )
        .rule(
            StyleSelector::class_state("scroll-wide-row-card", ElementStateSelector::Hovered),
            Style::default()
                .background(Color::rgb(38, 47, 54))
                .border(STROKE_SELECTED),
        )
        .rule(
            StyleSelector::class_state("scroll-mini-row", ElementStateSelector::Hovered),
            Style::default()
                .background(Color::rgb(36, 44, 51))
                .border(STROKE_SELECTED),
        )
        .rule(
            StyleSelector::class_state("scroll-xy-row-card", ElementStateSelector::Hovered),
            Style::default()
                .background(Color::rgb(38, 47, 54))
                .border(STROKE_SELECTED),
        )
        .rule(
            StyleSelector::class("nest-outer"),
            Style::default()
                .size(650.0, 430.0)
                .padding(Insets::all(28.0))
                .gap(16.0)
                .background(Color::rgb(20, 24, 29))
                .border(STROKE)
                .radius(8.0),
        )
        .rule(
            StyleSelector::class("nest-middle"),
            Style::default()
                .size(500.0, 270.0)
                .padding(Insets::all(24.0))
                .gap(14.0)
                .background(Color::rgb(31, 43, 52))
                .border(STROKE_SELECTED)
                .radius(7.0),
        )
        .rule(
            StyleSelector::class("nest-inner"),
            Style::default()
                .size(360.0, 130.0)
                .padding(Insets::all(18.0))
                .gap(6.0)
                .background(Color::rgb(42, 37, 57))
                .border(PURPLE)
                .radius(7.0),
        )
        .rule(
            StyleSelector::class_state("nest-inner", ElementStateSelector::Hovered),
            Style::default()
                .background(Color::rgb(55, 50, 78))
                .border(TEXT_ACCENT),
        )
        .rule(
            StyleSelector::class("canvas-placeholder"),
            Style::default()
                .size(720.0, 360.0)
                .padding(Insets::all(18.0))
                .gap(8.0)
                .background(Color::rgb(15, 18, 21))
                .border(Color::rgb(72, 82, 92))
                .radius(7.0),
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
            Style::default().font_size(16.0).text_color(TEXT),
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
                .background(Color::rgb(53, 38, 70))
                .border(PURPLE),
        )
}

fn styled_scrollbar_selector() -> des_ui_document::CompoundSelector {
    StyleSelector::compound().class("styled-scrollbar")
}

fn styled_scrollbar_style() -> Style {
    Style::default()
        .scrollbar_handle_color(Color::rgba(232, 236, 240, 118))
        .scrollbar_track_color(Color::rgba(2, 8, 12, 84))
        .scrollbar_width(2.0)
        .scrollbar_expanded_width(10.0)
        .scrollbar_hover_track_color(Color::rgba(2, 8, 12, 84))
        .scrollbar_pressed_track_color(Color::rgba(2, 8, 12, 84))
        .scrollbar_pressed_handle_color(Color::rgba(190, 217, 255, 238))
        .scrollbar_pressed_handle_border_color(Color::rgba(255, 255, 255, 120))
        .scrollbar_pressed_handle_border_width(1.0)
        .scrollbar_radius(6.0)
        .transition(Transition::ease_out(0.14))
}
