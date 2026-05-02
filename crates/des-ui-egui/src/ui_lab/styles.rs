use super::{
    BACKGROUND, CARD, CARD_HOVER, CARD_PRESSED, CARD_SELECTED, GREEN, PANEL, PANEL_ALT, PURPLE,
    STROKE, STROKE_SELECTED, TEXT, TEXT_ACCENT, TEXT_MUTED,
};
use des_ui_document::{
    Color, Direction, ElementRole, ElementStateSelector, Insets, Length, Overflow, Style,
    StyleSelector, StyleSheet, Transition,
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
                .scrollbar_width(2.0)
                .scrollbar_handle_color(Color::rgba(232, 236, 240, 118))
                .scrollbar_track_color(Color::rgba(2, 8, 12, 84))
                .scrollbar_radius(6.0)
                .transition(Transition::ease_out(0.2)),
        )
        .rule(
            StyleSelector::class_state("stage", ElementStateSelector::ScrollbarHovered),
            Style::default()
                .scrollbar_width(10.0)
                .scrollbar_track_color(Color::rgba(2, 8, 12, 84)),
        )
        .rule(
            StyleSelector::class_state("stage", ElementStateSelector::Pressed),
            Style::default()
                .scrollbar_width(10.0)
                .scrollbar_track_color(Color::rgba(2, 8, 12, 84))
                .scrollbar_handle_color(Color::rgba(190, 217, 255, 238))
                .scrollbar_handle_border_color(Color::rgba(255, 255, 255, 120))
                .scrollbar_handle_border_width(1.0),
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
                .scrollbar_width(2.0)
                .scrollbar_handle_color(Color::rgba(232, 236, 240, 118))
                .scrollbar_track_color(Color::rgba(2, 8, 12, 84))
                .scrollbar_radius(6.0)
                .transition(Transition::ease_out(0.2)),
        )
        .rule(
            StyleSelector::class_state(
                "box-subject-scroll-overflow",
                ElementStateSelector::ScrollbarHovered,
            ),
            Style::default()
                .scrollbar_width(10.0)
                .scrollbar_track_color(Color::rgba(2, 8, 12, 84)),
        )
        .rule(
            StyleSelector::class_state(
                "box-subject-scroll-overflow",
                ElementStateSelector::Pressed,
            ),
            Style::default()
                .scrollbar_width(10.0)
                .scrollbar_track_color(Color::rgba(2, 8, 12, 84))
                .scrollbar_handle_color(Color::rgba(190, 217, 255, 238))
                .scrollbar_handle_border_color(Color::rgba(255, 255, 255, 120))
                .scrollbar_handle_border_width(1.0),
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
                .size(318.0, 420.0)
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
                .height(des_ui_document::Length::Px(370.0))
                .padding(Insets::symmetric(4.0, 4.0))
                .gap(7.0)
                .overflow_y(Overflow::Scroll)
                .scrollbar_width(2.0)
                .scrollbar_handle_color(Color::rgba(232, 236, 240, 118))
                .scrollbar_track_color(Color::rgba(2, 8, 12, 84))
                .scrollbar_radius(6.0)
                .transition(Transition::ease_out(0.2)),
        )
        .rule(
            StyleSelector::class_state("scroll-panel", ElementStateSelector::Hovered),
            Style::default().border(STROKE_SELECTED),
        )
        .rule(
            StyleSelector::class_state("scroll-list", ElementStateSelector::ScrollbarHovered),
            Style::default()
                .scrollbar_width(10.0)
                .scrollbar_track_color(Color::rgba(2, 8, 12, 84)),
        )
        .rule(
            StyleSelector::class_state("scroll-list", ElementStateSelector::Pressed),
            Style::default()
                .scrollbar_width(10.0)
                .scrollbar_track_color(Color::rgba(2, 8, 12, 84))
                .scrollbar_handle_color(Color::rgba(190, 217, 255, 238))
                .scrollbar_handle_border_color(Color::rgba(255, 255, 255, 120))
                .scrollbar_handle_border_width(1.0),
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
            StyleSelector::class_state("scroll-row-card", ElementStateSelector::Hovered),
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
