use crate::{AppFrame, WindowApp};
use des_ui_document::{
    AlignItems, Color, Document, DocumentEngine, DocumentEventKind, DocumentInput, DocumentOutput,
    Element, ElementStateSelector, FlexDirection, FlexWrap, Glyph, Insets, JustifyContent, Length,
    Overflow, Point, Shadow, Size, Style, StyleSelector, StyleSheet, Transition,
};

const ACTION_ID: &str = "native-action";

pub struct NativeDocumentDemo {
    engine: DocumentEngine,
    stylesheet: StyleSheet,
    clicks: u32,
    frame_count: u32,
    exit_after_frames: Option<u32>,
    last_output: Option<DocumentOutput>,
}

impl NativeDocumentDemo {
    pub fn new() -> Self {
        Self {
            engine: DocumentEngine::default(),
            stylesheet: demo_stylesheet(),
            clicks: 0,
            frame_count: 0,
            exit_after_frames: None,
            last_output: None,
        }
    }

    pub fn with_exit_after_frames(mut self, frames: u32) -> Self {
        self.exit_after_frames = Some(frames.max(1));
        self
    }

    pub fn clicks(&self) -> u32 {
        self.clicks
    }

    pub fn last_output(&self) -> Option<&DocumentOutput> {
        self.last_output.as_ref()
    }

    fn update_document(&mut self, viewport: Size, input: DocumentInput) -> DocumentOutput {
        let mut document = demo_document(viewport, self.clicks);
        self.engine
            .update_with_input(&mut document, &self.stylesheet, input)
    }
}

impl Default for NativeDocumentDemo {
    fn default() -> Self {
        Self::new()
    }
}

impl WindowApp for NativeDocumentDemo {
    fn update(&mut self, frame: &mut AppFrame) {
        let viewport = frame.viewport().logical_size();
        let input = *frame.input();
        let mut output = self.update_document(viewport, input);
        let clicked = output.events.iter().any(|event| {
            event.target.as_str() == ACTION_ID && event.kind == DocumentEventKind::Clicked
        });

        if clicked {
            self.clicks = self.clicks.wrapping_add(1);
            frame.request_repaint();
            output = self.update_document(viewport, input.without_primary_activation());
        }
        if output.animating {
            frame.request_repaint();
        }

        frame.set_document_output(&output);
        self.last_output = Some(output);
        self.frame_count = self.frame_count.saturating_add(1);

        if self
            .exit_after_frames
            .is_some_and(|limit| self.frame_count >= limit)
        {
            frame.request_close();
        }
    }
}

fn demo_document(viewport: Size, clicks: u32) -> Document {
    Document::build(viewport, |ui| {
        ui.main("native-shell").children(|ui| {
            ui.section("native-card").children(|ui| {
                ui.div("text-specimen").children(|ui| {
                    ui.text("native-title", "Native document text");
                    ui.text(
                        "native-copy",
                        "The non-egui renderer now draws arbitrary document strings, wraps text inside styled elements, and keeps π, Σ, and UTF-8 data visible in the same pass as shapes.",
                    );
                    ui.text("native-meta", "Resize the window; this remains document layout text.");
                });

                ui.div("swatch-row").children(|ui| {
                    for (index, id) in ["swatch-a", "swatch-b", "swatch-c"].iter().enumerate() {
                        let mut swatch = ui.div(*id).class("swatch");
                        if index as u32 == clicks % 3 {
                            swatch = swatch.class("is-active");
                        }
                        swatch.empty();
                    }
                });

                ui.div("native-action-wrap").children(|ui| {
                    ui.button(ACTION_ID).interactive().children(|ui| {
                        ui.icon("native-action-glyph")
                            .glyph(if clicks % 2 == 0 {
                                Glyph::ChevronDown
                            } else {
                                Glyph::Check
                            })
                            .empty();
                        ui.text("native-action-label", format!("Clicks: {clicks}"));
                    });
                });

                ui.div("native-clip-window").children(|ui| {
                    ui.text(
                        "native-clip-copy",
                        "This long document text is intentionally clipped by an overflow container so native text, layout, and scissor planning move through the same renderer path.",
                    );
                });

                ui.div("responsive-grid").children(|ui| {
                    for id in ["tile-one", "tile-two", "tile-three", "tile-four"] {
                        ui.div(id).class("tile").empty();
                    }
                });
            });
        });
    })
}

fn demo_stylesheet() -> StyleSheet {
    StyleSheet::new()
        .rule(
            StyleSelector::Element(Element::Main),
            Style::default()
                .width_fill()
                .height_fill()
                .padding(Insets::all(28.0))
                .background(Color::rgb(246, 241, 249))
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::Center),
        )
        .rule(
            StyleSelector::id("native-card"),
            Style::default()
                .width_percent(0.72)
                .min_size(460.0, 420.0)
                .max_size(780.0, 560.0)
                .padding(Insets::all(28.0))
                .gap(24.0)
                .background(Color::rgb(255, 252, 255))
                .border(Color::rgb(194, 184, 203))
                .border_width(1.5)
                .radius(18.0)
                .shadow(Shadow {
                    offset: Point::new(0.0, 10.0),
                    blur: 22.0,
                    spread: 0.0,
                    color: Color::rgba(70, 48, 92, 42),
                }),
        )
        .viewport_max_width(
            720.0,
            StyleSelector::id("native-card"),
            Style::default()
                .width_percent(0.92)
                .padding(Insets::all(18.0)),
        )
        .rule(
            StyleSelector::id("text-specimen"),
            Style::default().gap(8.0).width_fill(),
        )
        .rule(
            StyleSelector::id("native-title"),
            Style::default()
                .width_fill()
                .height(Length::Px(34.0))
                .font_size(24.0)
                .text_color(Color::rgb(31, 27, 36)),
        )
        .rule(
            StyleSelector::id("native-copy"),
            Style::default()
                .width_fill()
                .height(Length::Px(72.0))
                .font_size(15.0)
                .line_height(20.0)
                .text_color(Color::rgb(91, 82, 101)),
        )
        .rule(
            StyleSelector::id("native-meta"),
            Style::default()
                .width_fill()
                .height(Length::Px(22.0))
                .font_size(13.0)
                .text_color(Color::rgb(122, 91, 181)),
        )
        .rule(
            StyleSelector::id("swatch-row"),
            Style::default()
                .flex_direction(FlexDirection::Row)
                .gap(14.0)
                .height(Length::Px(76.0))
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::Center),
        )
        .rule(
            StyleSelector::class("swatch"),
            Style::default()
                .size(58.0, 58.0)
                .radius(16.0)
                .background(Color::rgb(218, 209, 236))
                .border(Color::rgb(122, 91, 181))
                .border_width(1.5)
                .animate_paint(true)
                .animate_size(true)
                .transition(Transition::ease_out(0.22)),
        )
        .rule(
            StyleSelector::compound()
                .class("swatch")
                .class("is-active")
                .selector(),
            Style::default()
                .size(72.0, 58.0)
                .background(Color::rgb(246, 57, 94))
                .border(Color::rgb(246, 57, 94)),
        )
        .rule(
            StyleSelector::id("native-action-wrap"),
            Style::default()
                .flex_direction(FlexDirection::Row)
                .justify_content(JustifyContent::Center),
        )
        .rule(
            StyleSelector::id(ACTION_ID),
            Style::default()
                .flex_direction(FlexDirection::Row)
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::Center)
                .gap(10.0)
                .size(180.0, 58.0)
                .padding(Insets::symmetric(18.0, 12.0))
                .background(Color::rgb(113, 82, 181))
                .border(Color::rgb(84, 60, 140))
                .border_width(1.5)
                .radius(14.0)
                .text_color(Color::rgb(255, 255, 255))
                .font_size(16.0)
                .animate_paint(true)
                .animate_size(true)
                .transition(Transition::ease_out(0.20)),
        )
        .rule(
            StyleSelector::id_state(ACTION_ID, ElementStateSelector::Hovered),
            Style::default()
                .size(194.0, 62.0)
                .background(Color::rgb(137, 99, 214))
                .border(Color::rgb(92, 63, 159)),
        )
        .rule(
            StyleSelector::id_state(ACTION_ID, ElementStateSelector::Pressed),
            Style::default()
                .size(174.0, 54.0)
                .background(Color::rgb(78, 54, 130))
                .border(Color::rgb(57, 39, 98)),
        )
        .rule(
            StyleSelector::id("native-action-glyph"),
            Style::default()
                .size(22.0, 22.0)
                .text_color(Color::rgb(255, 255, 255)),
        )
        .rule(
            StyleSelector::id("native-action-label"),
            Style::default().size(86.0, 22.0),
        )
        .rule(
            StyleSelector::id("native-clip-window"),
            Style::default()
                .width_fill()
                .height(Length::Px(38.0))
                .padding(Insets::symmetric(12.0, 7.0))
                .background(Color::rgb(248, 244, 252))
                .border(Color::rgb(204, 194, 215))
                .border_width(1.0)
                .radius(10.0)
                .overflow_y(Overflow::Scroll)
                .scrollbar_width(2.0)
                .scrollbar_handle_color(Color::rgba(103, 80, 164, 128)),
        )
        .rule(
            StyleSelector::id("native-clip-copy"),
            Style::default()
                .width_fill()
                .height(Length::Px(96.0))
                .font_size(14.0)
                .line_height(19.0)
                .text_color(Color::rgb(91, 82, 101)),
        )
        .rule(
            StyleSelector::id("responsive-grid"),
            Style::default()
                .width_fill()
                .flex_direction(FlexDirection::Row)
                .flex_wrap(FlexWrap::Wrap)
                .gap(14.0)
                .justify_content(JustifyContent::Center),
        )
        .rule(
            StyleSelector::class("tile"),
            Style::default()
                .width(Length::Calc {
                    percent: 0.5,
                    px: -7.0,
                })
                .height(Length::Px(46.0))
                .radius(12.0)
                .background(Color::rgb(232, 227, 239))
                .border(Color::rgb(202, 192, 211))
                .border_width(1.0),
        )
        .viewport_max_width(
            640.0,
            StyleSelector::class("tile"),
            Style::default().width_percent(1.0),
        )
}

#[cfg(test)]
pub(crate) fn click_input_at(position: des_ui_document::Point, time_seconds: f64) -> DocumentInput {
    pointer_input_at(position, time_seconds, false, false, true, 1)
}

#[cfg(test)]
pub(crate) fn hover_input_at(position: des_ui_document::Point, time_seconds: f64) -> DocumentInput {
    pointer_input_at(position, time_seconds, false, false, false, 0)
}

#[cfg(test)]
pub(crate) fn press_input_at(position: des_ui_document::Point, time_seconds: f64) -> DocumentInput {
    pointer_input_at(position, time_seconds, true, true, false, 0)
}

#[cfg(test)]
fn pointer_input_at(
    position: des_ui_document::Point,
    time_seconds: f64,
    primary_down: bool,
    primary_pressed: bool,
    primary_clicked: bool,
    primary_click_count: u8,
) -> DocumentInput {
    DocumentInput {
        pointer: Some(des_ui_document::PointerInput {
            position,
            primary_delta: des_ui_document::Point::ZERO,
            primary_down,
            primary_pressed,
            primary_clicked,
            primary_click_count,
            secondary_clicked: false,
            time_seconds,
        }),
        scroll_delta: des_ui_document::Point::ZERO,
    }
}
