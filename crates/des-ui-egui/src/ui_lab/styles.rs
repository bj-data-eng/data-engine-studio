use super::{
    BACKGROUND, CARD, CARD_HOVER, CARD_PRESSED, CARD_SELECTED, GREEN, PANEL, PANEL_ALT, PURPLE,
    STROKE, STROKE_SELECTED, TEXT, TEXT_ACCENT, TEXT_MUTED,
};
use des_ui_document::{
    Color, Direction, ElementRole, ElementStateSelector, Insets, Length, Overflow, StylePatch,
    StyleSelector, StyleSheet, Transition,
};

pub(super) fn stylesheet() -> StyleSheet {
    StyleSheet::new()
        .rule(
            StyleSelector::Role(ElementRole::Root),
            StylePatch::default()
                .direction(Direction::Column)
                .background(BACKGROUND),
        )
        .rule(
            StyleSelector::Role(ElementRole::Panel),
            StylePatch::default()
                .direction(Direction::Column)
                .background(PANEL),
        )
        .rule(
            StyleSelector::Role(ElementRole::Card),
            StylePatch::default()
                .direction(Direction::Column)
                .padding(Insets::all(12.0))
                .gap(5.0)
                .background(CARD)
                .border(STROKE)
                .radius(7.0),
        )
        .rule(
            StyleSelector::Role(ElementRole::Control),
            StylePatch::default()
                .padding(Insets::symmetric(12.0, 7.0))
                .background(CARD)
                .border(STROKE)
                .radius(5.0),
        )
        .rule(
            StyleSelector::Role(ElementRole::Text),
            StylePatch::default().font_size(13.0).text_color(TEXT),
        )
        .rule(
            StyleSelector::class("lab-root"),
            StylePatch::default()
                .size(1320.0, 780.0)
                .background(BACKGROUND)
                .gap(0.0),
        )
        .rule(
            StyleSelector::class("topbar"),
            StylePatch::default()
                .size(1320.0, 58.0)
                .padding(Insets::symmetric(18.0, 10.0))
                .gap(3.0)
                .background(Color::rgb(22, 26, 30)),
        )
        .rule(
            StyleSelector::class("lab-body"),
            StylePatch::default()
                .direction(Direction::Row)
                .size(1320.0, 722.0)
                .padding(Insets::all(14.0))
                .gap(14.0)
                .background(BACKGROUND),
        )
        .rule(
            StyleSelector::class("nav"),
            StylePatch::default()
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
            StylePatch::default()
                .size(1036.0, 690.0)
                .padding(Insets::all(18.0))
                .gap(12.0)
                .background(PANEL_ALT)
                .border(STROKE)
                .radius(8.0)
                .overflow_y(Overflow::Scroll),
        )
        .rule(
            StyleSelector::class("box-model-grid"),
            StylePatch::default()
                .width_fill()
                .gap(10.0)
                .background(PANEL_ALT),
        )
        .rule(
            StyleSelector::class("box-model-row"),
            StylePatch::default()
                .direction(Direction::Row)
                .width_fill()
                .height(Length::Px(172.0))
                .gap(10.0)
                .background(PANEL_ALT),
        )
        .rule(
            StyleSelector::class("box-model-case"),
            StylePatch::default()
                .size(318.0, 172.0)
                .padding(Insets::all(8.0))
                .gap(3.0)
                .background(Color::rgb(20, 24, 28))
                .border(Color::rgb(45, 54, 62))
                .radius(5.0),
        )
        .rule(
            StyleSelector::class("box-section-label"),
            StylePatch::default()
                .font_size(14.0)
                .text_color(TEXT_ACCENT)
                .height(Length::Px(18.0)),
        )
        .rule(
            StyleSelector::class("box-label"),
            StylePatch::default()
                .font_size(13.0)
                .text_color(TEXT)
                .height(Length::Px(16.0)),
        )
        .rule(
            StyleSelector::class("box-note"),
            StylePatch::default()
                .font_size(11.0)
                .text_color(TEXT_MUTED)
                .height(Length::Px(14.0)),
        )
        .rule(
            StyleSelector::class("box-rule"),
            StylePatch::default()
                .font_size(10.0)
                .text_color(TEXT_ACCENT)
                .height(Length::Px(24.0)),
        )
        .rule(
            StyleSelector::class("box-subject-frame"),
            StylePatch::default()
                .size(294.0, 86.0)
                .background(Color::rgb(13, 16, 19))
                .border(Color::rgb(39, 48, 56))
                .overflow_y(Overflow::Visible),
        )
        .rule(
            StyleSelector::class("box-subject"),
            StylePatch::default()
                .size(32.0, 32.0)
                .gap(0.0)
                .padding(Insets::ZERO)
                .background(Color::rgb(65, 121, 164)),
        )
        .rule(
            StyleSelector::class("box-chip"),
            StylePatch::default()
                .size(12.0, 12.0)
                .background(Color::rgb(141, 207, 164)),
        )
        .rule(
            StyleSelector::class("box-overflow-child"),
            StylePatch::default()
                .size(112.0, 112.0)
                .background(Color::rgb(218, 151, 77)),
        )
        .rule(
            StyleSelector::class("box-subject-auto"),
            StylePatch::default()
                .width(Length::Auto)
                .height(Length::Auto),
        )
        .rule(
            StyleSelector::class("box-subject-px"),
            StylePatch::default().size(96.0, 44.0),
        )
        .rule(
            StyleSelector::class("box-subject-min"),
            StylePatch::default()
                .width(Length::Auto)
                .height(Length::Auto)
                .min_size(40.0, 40.0),
        )
        .rule(
            StyleSelector::class("box-subject-fill"),
            StylePatch::default().width_fill().height(Length::Px(28.0)),
        )
        .rule(
            StyleSelector::class("box-subject-percent"),
            StylePatch::default()
                .width_percent(0.5)
                .height(Length::Px(28.0)),
        )
        .rule(
            StyleSelector::class("box-subject-height-fill"),
            StylePatch::default().width(Length::Px(64.0)).height_fill(),
        )
        .rule(
            StyleSelector::class("box-subject-margin"),
            StylePatch::default()
                .size(32.0, 32.0)
                .margin(Insets::all(12.0)),
        )
        .rule(
            StyleSelector::class("box-subject-padding"),
            StylePatch::default()
                .width(Length::Auto)
                .height(Length::Auto)
                .padding(Insets::all(12.0)),
        )
        .rule(
            StyleSelector::class("box-subject-border"),
            StylePatch::default()
                .size(44.0, 44.0)
                .border(PURPLE)
                .border_width(5.0),
        )
        .rule(
            StyleSelector::class("box-subject-row-gap"),
            StylePatch::default()
                .direction(Direction::Row)
                .width(Length::Auto)
                .height(Length::Auto)
                .gap(10.0),
        )
        .rule(
            StyleSelector::class("box-subject-column-gap"),
            StylePatch::default()
                .direction(Direction::Column)
                .width(Length::Auto)
                .height(Length::Auto)
                .gap(6.0),
        )
        .rule(
            StyleSelector::class("box-subject-visible-overflow"),
            StylePatch::default()
                .size(44.0, 44.0)
                .overflow_y(Overflow::Visible),
        )
        .rule(
            StyleSelector::class("box-subject-scroll-overflow"),
            StylePatch::default()
                .size(44.0, 44.0)
                .overflow_y(Overflow::Scroll),
        )
        .rule(
            StyleSelector::class("box-subject-nested-nine"),
            StylePatch::default()
                .width(Length::Auto)
                .height(Length::Auto),
        )
        .rule(
            StyleSelector::class("box-nested-outer"),
            StylePatch::default()
                .width(Length::Auto)
                .height(Length::Auto)
                .margin(Insets::all(8.0))
                .border(PURPLE)
                .border_width(3.0)
                .background(Color::rgb(41, 58, 73)),
        )
        .rule(
            StyleSelector::class("box-nested-inner"),
            StylePatch::default()
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
            StylePatch::default()
                .direction(Direction::Row)
                .width(Length::Auto)
                .height(Length::Auto)
                .gap(4.0)
                .background(Color::rgb(65, 121, 164)),
        )
        .rule(
            StyleSelector::class("box-nested-cell"),
            StylePatch::default()
                .size(10.0, 10.0)
                .background(Color::rgb(141, 207, 164)),
        )
        .rule(
            StyleSelector::class("box-subject-inset-percent"),
            StylePatch::default()
                .width(Length::Auto)
                .height(Length::Auto),
        )
        .rule(
            StyleSelector::class("box-inset-percent-parent"),
            StylePatch::default()
                .size(88.0, 88.0)
                .padding(Insets::all(8.0))
                .border(PURPLE)
                .border_width(2.0)
                .background(Color::rgb(65, 121, 164)),
        )
        .rule(
            StyleSelector::class("box-inset-percent-child"),
            StylePatch::default()
                .width_percent(0.5)
                .height_percent(0.5)
                .background(Color::rgb(141, 207, 164)),
        )
        .rule(
            StyleSelector::class("nav-item"),
            StylePatch::default()
                .width_fill()
                .height(des_ui_document::Length::Px(64.0))
                .background(CARD)
                .border(STROKE),
        )
        .rule(
            StyleSelector::class_state("nav-item", ElementStateSelector::Selected),
            StylePatch::default()
                .background(CARD_SELECTED)
                .border(STROKE_SELECTED),
        )
        .rule(
            StyleSelector::class_state("nav-item", ElementStateSelector::Hovered),
            StylePatch::default()
                .background(CARD_HOVER)
                .border(STROKE_SELECTED),
        )
        .rule(
            StyleSelector::class("toolbar-row"),
            StylePatch::default()
                .direction(Direction::Row)
                .gap(8.0)
                .background(PANEL_ALT),
        )
        .rule(
            StyleSelector::class("button"),
            StylePatch::default()
                .size(156.0, 36.0)
                .background(Color::rgb(38, 43, 48))
                .border(STROKE),
        )
        .rule(
            StyleSelector::class_state("button", ElementStateSelector::Selected),
            StylePatch::default()
                .background(CARD_SELECTED)
                .border(STROKE_SELECTED),
        )
        .rule(
            StyleSelector::class_state("button", ElementStateSelector::Hovered),
            StylePatch::default()
                .background(CARD_HOVER)
                .border(STROKE_SELECTED),
        )
        .rule(
            StyleSelector::class_state("button", ElementStateSelector::Pressed),
            StylePatch::default().background(CARD_PRESSED),
        )
        .rule(
            StyleSelector::class("button-label"),
            StylePatch::default().font_size(13.0).text_color(TEXT),
        )
        .rule(
            StyleSelector::class("card-row"),
            StylePatch::default()
                .direction(Direction::Row)
                .gap(12.0)
                .background(PANEL_ALT),
        )
        .rule(
            StyleSelector::class("card-row-dense"),
            StylePatch::default()
                .direction(Direction::Row)
                .gap(6.0)
                .background(PANEL_ALT),
        )
        .rule(
            StyleSelector::class("feature-card"),
            StylePatch::default()
                .size(250.0, 98.0)
                .background(CARD)
                .border(STROKE),
        )
        .rule(
            StyleSelector::class_state("feature-card", ElementStateSelector::Hovered),
            StylePatch::default()
                .background(CARD_HOVER)
                .border(STROKE_SELECTED),
        )
        .rule(
            StyleSelector::class_state("feature-card", ElementStateSelector::Pressed),
            StylePatch::default().background(CARD_PRESSED),
        )
        .rule(
            StyleSelector::class("stack"),
            StylePatch::default()
                .size(620.0, 320.0)
                .padding(Insets::all(10.0))
                .gap(8.0)
                .background(PANEL)
                .border(STROKE)
                .radius(7.0),
        )
        .rule(
            StyleSelector::class("list-row"),
            StylePatch::default()
                .size(600.0, 58.0)
                .background(Color::rgb(25, 30, 34))
                .border(STROKE)
                .radius(5.0),
        )
        .rule(
            StyleSelector::class("specificity-proof"),
            StylePatch::default()
                .background(Color::rgb(30, 37, 43))
                .border(Color::rgb(80, 91, 103)),
        )
        .rule(
            StyleSelector::class_state("specificity-proof", ElementStateSelector::Hovered),
            StylePatch::default()
                .background(Color::rgb(38, 55, 64))
                .border(GREEN),
        )
        .rule(
            StyleSelector::id("style-row-state"),
            StylePatch::default().border(PURPLE),
        )
        .rule(
            StyleSelector::id_state("style-row-state", ElementStateSelector::Hovered),
            StylePatch::default()
                .background(Color::rgb(50, 41, 68))
                .border(TEXT_ACCENT),
        )
        .rule(
            StyleSelector::class("scroll-panel"),
            StylePatch::default()
                .size(318.0, 420.0)
                .padding(Insets::all(10.0))
                .gap(7.0)
                .background(Color::rgb(20, 24, 28))
                .border(STROKE)
                .radius(7.0),
        )
        .rule(
            StyleSelector::class("scroll-list"),
            StylePatch::default()
                .width_fill()
                .height(des_ui_document::Length::Px(370.0))
                .padding(Insets::symmetric(4.0, 4.0))
                .gap(7.0)
                .overflow_y(Overflow::Scroll),
        )
        .rule(
            StyleSelector::class_state("scroll-panel", ElementStateSelector::Hovered),
            StylePatch::default().border(STROKE_SELECTED),
        )
        .rule(
            StyleSelector::class("scroll-row-card"),
            StylePatch::default()
                .width_fill()
                .height(des_ui_document::Length::Px(34.0))
                .padding(Insets::symmetric(9.0, 7.0))
                .background(Color::rgb(29, 34, 39))
                .border(Color::rgb(48, 57, 65))
                .radius(4.0),
        )
        .rule(
            StyleSelector::class_state("scroll-row-card", ElementStateSelector::Hovered),
            StylePatch::default()
                .background(Color::rgb(38, 47, 54))
                .border(STROKE_SELECTED),
        )
        .rule(
            StyleSelector::class("nest-outer"),
            StylePatch::default()
                .size(650.0, 430.0)
                .padding(Insets::all(28.0))
                .gap(16.0)
                .background(Color::rgb(20, 24, 29))
                .border(STROKE)
                .radius(8.0),
        )
        .rule(
            StyleSelector::class("nest-middle"),
            StylePatch::default()
                .size(500.0, 270.0)
                .padding(Insets::all(24.0))
                .gap(14.0)
                .background(Color::rgb(31, 43, 52))
                .border(STROKE_SELECTED)
                .radius(7.0),
        )
        .rule(
            StyleSelector::class("nest-inner"),
            StylePatch::default()
                .size(360.0, 130.0)
                .padding(Insets::all(18.0))
                .gap(6.0)
                .background(Color::rgb(42, 37, 57))
                .border(PURPLE)
                .radius(7.0),
        )
        .rule(
            StyleSelector::class_state("nest-inner", ElementStateSelector::Hovered),
            StylePatch::default()
                .background(Color::rgb(55, 50, 78))
                .border(TEXT_ACCENT),
        )
        .rule(
            StyleSelector::class("canvas-placeholder"),
            StylePatch::default()
                .size(720.0, 360.0)
                .padding(Insets::all(18.0))
                .gap(8.0)
                .background(Color::rgb(15, 18, 21))
                .border(Color::rgb(72, 82, 92))
                .radius(7.0),
        )
        .rule(
            StyleSelector::class("title"),
            StylePatch::default().font_size(21.0).text_color(TEXT),
        )
        .rule(
            StyleSelector::class("heading"),
            StylePatch::default().font_size(24.0).text_color(TEXT),
        )
        .rule(
            StyleSelector::class("section-title"),
            StylePatch::default()
                .font_size(13.0)
                .text_color(TEXT_ACCENT),
        )
        .rule(
            StyleSelector::class("card-title"),
            StylePatch::default().font_size(16.0).text_color(TEXT),
        )
        .rule(
            StyleSelector::class("muted"),
            StylePatch::default().font_size(12.5).text_color(TEXT_MUTED),
        )
        .rule(
            StyleSelector::id_state("interaction-card-two", ElementStateSelector::Hovered),
            StylePatch::default()
                .border(GREEN)
                .transition(Transition::ease_out(0.24)),
        )
        .rule(
            StyleSelector::id("interaction-card-three"),
            StylePatch::default().transition(Transition::ease_out(0.06)),
        )
        .rule(
            StyleSelector::id_state("interaction-card-three", ElementStateSelector::Pressed),
            StylePatch::default()
                .background(Color::rgb(53, 38, 70))
                .border(PURPLE),
        )
}
