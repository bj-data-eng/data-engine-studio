use crate::{AppFrame, WindowApp};
use des_ui_document::{
    AlignItems, Color, Document, DocumentEngine, DocumentInput, DocumentOutput, Element, Insets,
    JustifyContent, Length, Overflow, Size, Style, StyleSelector, StyleSheet, TextWrapMode,
};

const SAMPLE_TEXT: &str = "Native document text";
const SMALL_TEXT: &str = "Hamburgefonts 0123456789 Il1 O0 MW pi data";

pub struct NativeTextRenderingDemo {
    engine: DocumentEngine,
    stylesheet: StyleSheet,
    frame_count: u32,
    exit_after_frames: Option<u32>,
    last_output: Option<DocumentOutput>,
}

impl NativeTextRenderingDemo {
    pub fn new() -> Self {
        Self {
            engine: DocumentEngine::default(),
            stylesheet: text_rendering_stylesheet(),
            frame_count: 0,
            exit_after_frames: None,
            last_output: None,
        }
    }

    pub fn with_exit_after_frames(mut self, frames: u32) -> Self {
        self.exit_after_frames = Some(frames.max(1));
        self
    }

    pub fn last_output(&self) -> Option<&DocumentOutput> {
        self.last_output.as_ref()
    }

    fn update_document(&mut self, viewport: Size, input: DocumentInput) -> DocumentOutput {
        let mut document = text_rendering_document(viewport);
        self.engine
            .update_with_input(&mut document, &self.stylesheet, input)
    }
}

impl Default for NativeTextRenderingDemo {
    fn default() -> Self {
        Self::new()
    }
}

impl WindowApp for NativeTextRenderingDemo {
    fn update(&mut self, frame: &mut AppFrame) {
        let viewport = frame.viewport().logical_size();
        let output = self.update_document(viewport, *frame.input());
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

fn text_rendering_document(viewport: Size) -> Document {
    Document::build(viewport, |ui| {
        ui.main("text-aa-shell").children(|ui| {
            ui.section("text-aa-board").children(|ui| {
                ui.div("text-aa-header").children(|ui| {
                    ui.text("text-aa-title", "Text rendering and aliasing specimen");
                    ui.text(
                        "text-aa-caption",
                        "DES layout, GPU atlas upload, coverage mapping, clipping, sampling, and blend path in one resizable document.",
                    );
                });

                ui.div("text-aa-grid").children(|ui| {
                    specimen(ui, "text-aa-size-card", "Size ladder", |ui| {
                        for (id, size, label) in [
                            ("text-size-9", "9 px", "9"),
                            ("text-size-10", "10 px", "10"),
                            ("text-size-11", "11 px", "11"),
                            ("text-size-12", "12 px", "12"),
                            ("text-size-13", "13 px", "13"),
                            ("text-size-14", "14 px", "14"),
                            ("text-size-16", "16 px", "16"),
                            ("text-size-20", "20 px", "20"),
                            ("text-size-24", "24 px", "24"),
                            ("text-size-32", "32 px", "32"),
                        ] {
                            ui.div(format!("{id}-row")).class("text-aa-row").children(|ui| {
                                text_class(ui, format!("{id}-label"), label, "text-aa-row-label");
                                text_class(ui, id, format!("{size} {SAMPLE_TEXT}"), "text-aa-sample");
                            });
                        }
                    });

                    specimen(ui, "text-aa-contrast-card", "Contrast pairs", |ui| {
                        for (id, class, label) in [
                            ("contrast-dark-on-light", "contrast-light", "Dark on light"),
                            ("contrast-muted-on-light", "contrast-muted", "Muted on light"),
                            ("contrast-light-on-dark", "contrast-dark", "Light on dark"),
                            ("contrast-accent-on-dark", "contrast-accent", "Accent on dark"),
                        ] {
                            ui.div(id).class("contrast-row").class(class).children(|ui| {
                                text_class(ui, format!("{id}-label"), label, "contrast-label");
                                text_class(ui, format!("{id}-sample"), SMALL_TEXT, "contrast-sample");
                            });
                        }
                    });

                    specimen(ui, "text-aa-subpixel-card", "Subpixel offsets", |ui| {
                        for (id, label) in [
                            ("subpixel-0", "x + 0.00"),
                            ("subpixel-25", "x + 0.25"),
                            ("subpixel-50", "x + 0.50"),
                            ("subpixel-75", "x + 0.75"),
                        ] {
                            ui.div(id).class("subpixel-row").children(|ui| {
                                text_class(ui, format!("{id}-label"), label, "text-aa-row-label");
                                text_class(ui, format!("{id}-sample"), SMALL_TEXT, "subpixel-sample");
                            });
                        }
                    });

                    specimen(ui, "text-aa-wrap-card", "Wrap, clip, truncate", |ui| {
                        text_class(
                            ui,
                            "wrap-sample",
                            "Wrapped text should keep even color on diagonals, punctuation, and descenders while using document line breaks inside a fixed-width region.",
                            "wrap-sample",
                        );
                        ui.div("clip-window").children(|ui| {
                            text_class(
                                ui,
                                "clip-sample",
                                "Clipped text exposes scissor edges and atlas sampling. ABCDEFGHIJKLMNOPQRSTUVWXYZ abcdefghijklmnopqrstuvwxyz 0123456789",
                                "clip-sample",
                            );
                        });
                        text_class(
                            ui,
                            "truncate-sample",
                            "Single-line truncation should not shimmer or fade unevenly at the right edge.",
                            "truncate-sample",
                        );
                    });

                    specimen(ui, "text-aa-edge-card", "Text beside geometry", |ui| {
                        for (id, label, class) in [
                            ("edge-square", "Square edge", "edge-square"),
                            ("edge-radius", "Rounded edge", "edge-radius"),
                            ("edge-hairline", "Hairline border", "edge-hairline"),
                            ("edge-pill", "Pill curve", "edge-pill"),
                        ] {
                            ui.div(format!("{id}-row")).class("edge-row").children(|ui| {
                                ui.div(id).class("edge-mark").class(class).empty();
                                text_class(ui, format!("{id}-text"), label, "edge-text");
                            });
                        }
                    });

                    specimen(ui, "text-aa-weight-card", "Density strings", |ui| {
                        for (id, text) in [
                            ("density-vertical", "IIII llll 1111 tttt ffff rrrr"),
                            ("density-round", "oooo eeee aaaa ssss cccc 0000"),
                            ("density-wide", "MMMM WWWW NNNN HHHH ####"),
                            ("density-punct", ".,:;!?'\"()[]{} /\\ -- ++ =="),
                        ] {
                            text_class(ui, id, text, "density-sample");
                        }
                    });
                });
            });
        });
    })
}

fn specimen(
    ui: &mut des_ui_document::DocumentBuilder,
    id: &'static str,
    title: &'static str,
    body: impl FnOnce(&mut des_ui_document::DocumentBuilder),
) {
    ui.div(id).class("text-aa-card").children(|ui| {
        text_class(ui, format!("{id}-title"), title, "text-aa-card-title");
        ui.div(format!("{id}-body"))
            .class("text-aa-card-body")
            .children(body);
    });
}

fn text_class(
    ui: &mut des_ui_document::DocumentBuilder,
    id: impl Into<des_ui_document::ElementId>,
    text: impl Into<String>,
    class: &'static str,
) {
    ui.child(id, Element::Text).class(class).text(text);
}

fn text_rendering_stylesheet() -> StyleSheet {
    StyleSheet::new()
        .rule(
            StyleSelector::Element(Element::Main),
            Style::default()
                .width_fill()
                .height_fill()
                .padding(Insets::all(24.0))
                .background(Color::rgb(246, 241, 249))
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::Center),
        )
        .rule(
            StyleSelector::id("text-aa-board"),
            Style::default()
                .width_percent(0.96)
                .height_percent(0.94)
                .max_size(1180.0, 820.0)
                .padding(Insets::all(22.0))
                .gap(18.0)
                .background(Color::rgb(255, 252, 255))
                .border(Color::rgb(194, 184, 203))
                .border_width(1.5)
                .radius(18.0),
        )
        .rule(
            StyleSelector::id("text-aa-header"),
            Style::default().width_fill().gap(5.0),
        )
        .rule(
            StyleSelector::id("text-aa-title"),
            Style::default()
                .width_fill()
                .height(Length::Px(34.0))
                .font_size(24.0)
                .text_color(Color::rgb(31, 27, 36)),
        )
        .rule(
            StyleSelector::id("text-aa-caption"),
            Style::default()
                .width_fill()
                .height(Length::Px(24.0))
                .font_size(13.0)
                .line_height(17.0)
                .text_color(Color::rgb(91, 82, 101)),
        )
        .rule(
            StyleSelector::id("text-aa-grid"),
            Style::default()
                .width_fill()
                .height_fill()
                .flex_direction(des_ui_document::FlexDirection::Row)
                .flex_wrap(des_ui_document::FlexWrap::Wrap)
                .gap(14.0)
                .align_items(AlignItems::Stretch),
        )
        .rule(
            StyleSelector::class("text-aa-card"),
            Style::default()
                .width(Length::Calc {
                    percent: 0.333_333_34,
                    px: -10.0,
                })
                .height(Length::Px(216.0))
                .padding(Insets::all(14.0))
                .gap(10.0)
                .background(Color::rgb(250, 247, 252))
                .border(Color::rgb(205, 195, 214))
                .border_width(1.0)
                .radius(12.0),
        )
        .viewport_max_width(
            900.0,
            StyleSelector::class("text-aa-card"),
            Style::default().width(Length::Calc {
                percent: 0.5,
                px: -8.0,
            }),
        )
        .viewport_max_width(
            640.0,
            StyleSelector::class("text-aa-card"),
            Style::default().width_percent(1.0),
        )
        .rule(
            StyleSelector::class("text-aa-card-title"),
            Style::default()
                .width_fill()
                .height(Length::Px(22.0))
                .font_size(15.0)
                .text_color(Color::rgb(31, 27, 36)),
        )
        .rule(
            StyleSelector::class("text-aa-card-body"),
            Style::default().width_fill().height_fill().gap(6.0),
        )
        .rule(
            StyleSelector::class("text-aa-row"),
            Style::default()
                .width_fill()
                .height(Length::Px(14.0))
                .flex_direction(des_ui_document::FlexDirection::Row)
                .align_items(AlignItems::Center)
                .gap(8.0),
        )
        .rule(
            StyleSelector::class("text-aa-row-label"),
            Style::default()
                .size(44.0, 14.0)
                .font_size(9.0)
                .text_color(Color::rgb(112, 103, 121)),
        )
        .rule(
            StyleSelector::class("text-aa-sample"),
            Style::default()
                .width_fill()
                .height(Length::Px(14.0))
                .font_size(12.0)
                .text_color(Color::rgb(31, 27, 36)),
        )
        .rule(
            StyleSelector::id("text-size-9"),
            Style::default().font_size(9.0),
        )
        .rule(
            StyleSelector::id("text-size-10"),
            Style::default().font_size(10.0),
        )
        .rule(
            StyleSelector::id("text-size-11"),
            Style::default().font_size(11.0),
        )
        .rule(
            StyleSelector::id("text-size-12"),
            Style::default().font_size(12.0),
        )
        .rule(
            StyleSelector::id("text-size-13"),
            Style::default().font_size(13.0),
        )
        .rule(
            StyleSelector::id("text-size-14"),
            Style::default().font_size(14.0),
        )
        .rule(
            StyleSelector::id("text-size-16"),
            Style::default().font_size(16.0),
        )
        .rule(
            StyleSelector::id("text-size-20"),
            Style::default().font_size(20.0),
        )
        .rule(
            StyleSelector::id("text-size-24"),
            Style::default().font_size(24.0),
        )
        .rule(
            StyleSelector::id("text-size-32"),
            Style::default().font_size(32.0),
        )
        .rule(
            StyleSelector::class("contrast-row"),
            Style::default()
                .width_fill()
                .height(Length::Px(34.0))
                .padding(Insets::symmetric(10.0, 7.0))
                .gap(2.0)
                .radius(8.0),
        )
        .rule(
            StyleSelector::class("contrast-light"),
            Style::default()
                .background(Color::rgb(255, 255, 255))
                .border(Color::rgb(221, 214, 228))
                .border_width(1.0)
                .text_color(Color::rgb(31, 27, 36)),
        )
        .rule(
            StyleSelector::class("contrast-muted"),
            Style::default()
                .background(Color::rgb(255, 255, 255))
                .border(Color::rgb(221, 214, 228))
                .border_width(1.0)
                .text_color(Color::rgb(118, 108, 128)),
        )
        .rule(
            StyleSelector::class("contrast-dark"),
            Style::default()
                .background(Color::rgb(31, 27, 36))
                .text_color(Color::rgb(255, 252, 255)),
        )
        .rule(
            StyleSelector::class("contrast-accent"),
            Style::default()
                .background(Color::rgb(31, 27, 36))
                .text_color(Color::rgb(244, 177, 255)),
        )
        .rule(
            StyleSelector::class("contrast-label"),
            Style::default()
                .width_fill()
                .height(Length::Px(13.0))
                .font_size(9.0),
        )
        .rule(
            StyleSelector::class("contrast-sample"),
            Style::default()
                .width_fill()
                .height(Length::Px(15.0))
                .font_size(11.0),
        )
        .rule(
            StyleSelector::class("subpixel-row"),
            Style::default()
                .width_fill()
                .height(Length::Px(32.0))
                .gap(2.0)
                .border(Color::rgb(224, 217, 231))
                .border_width(1.0)
                .radius(7.0),
        )
        .rule(
            StyleSelector::class("subpixel-sample"),
            Style::default()
                .width_fill()
                .height(Length::Px(15.0))
                .font_size(12.0)
                .text_color(Color::rgb(31, 27, 36)),
        )
        .rule(
            StyleSelector::id("subpixel-0"),
            Style::default().padding(Insets {
                top: 7.0,
                right: 8.0,
                bottom: 7.0,
                left: 8.0,
            }),
        )
        .rule(
            StyleSelector::id("subpixel-25"),
            Style::default().padding(Insets {
                top: 7.0,
                right: 8.0,
                bottom: 7.0,
                left: 8.25,
            }),
        )
        .rule(
            StyleSelector::id("subpixel-50"),
            Style::default().padding(Insets {
                top: 7.0,
                right: 8.0,
                bottom: 7.0,
                left: 8.5,
            }),
        )
        .rule(
            StyleSelector::id("subpixel-75"),
            Style::default().padding(Insets {
                top: 7.0,
                right: 8.0,
                bottom: 7.0,
                left: 8.75,
            }),
        )
        .rule(
            StyleSelector::class("wrap-sample"),
            Style::default()
                .width_fill()
                .height(Length::Px(54.0))
                .font_size(12.0)
                .line_height(16.0)
                .text_color(Color::rgb(31, 27, 36)),
        )
        .rule(
            StyleSelector::id("clip-window"),
            Style::default()
                .width_fill()
                .height(Length::Px(38.0))
                .padding(Insets::symmetric(9.0, 6.0))
                .overflow(Overflow::Scroll)
                .background(Color::rgb(255, 255, 255))
                .border(Color::rgb(205, 195, 214))
                .border_width(1.0)
                .radius(8.0),
        )
        .rule(
            StyleSelector::class("clip-sample"),
            Style::default()
                .width(Length::Px(520.0))
                .height(Length::Px(32.0))
                .font_size(13.0)
                .line_height(17.0)
                .text_color(Color::rgb(31, 27, 36)),
        )
        .rule(
            StyleSelector::class("truncate-sample"),
            Style::default()
                .width_fill()
                .height(Length::Px(18.0))
                .font_size(13.0)
                .text_wrap(TextWrapMode::Truncate)
                .text_color(Color::rgb(91, 82, 101)),
        )
        .rule(
            StyleSelector::class("edge-row"),
            Style::default()
                .width_fill()
                .height(Length::Px(34.0))
                .flex_direction(des_ui_document::FlexDirection::Row)
                .align_items(AlignItems::Center)
                .gap(10.0),
        )
        .rule(
            StyleSelector::class("edge-mark"),
            Style::default()
                .size(38.0, 20.0)
                .background(Color::rgb(218, 209, 236))
                .border(Color::rgb(122, 91, 181))
                .border_width(1.0),
        )
        .rule(
            StyleSelector::class("edge-radius"),
            Style::default().radius(8.0),
        )
        .rule(
            StyleSelector::class("edge-hairline"),
            Style::default()
                .background(Color::rgba(255, 255, 255, 0))
                .border_width(0.5),
        )
        .rule(
            StyleSelector::class("edge-pill"),
            Style::default().radius(10.0),
        )
        .rule(
            StyleSelector::class("edge-text"),
            Style::default()
                .width_fill()
                .height(Length::Px(18.0))
                .font_size(13.0)
                .text_color(Color::rgb(31, 27, 36)),
        )
        .rule(
            StyleSelector::class("density-sample"),
            Style::default()
                .width_fill()
                .height(Length::Px(23.0))
                .font_size(14.0)
                .text_color(Color::rgb(31, 27, 36)),
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AppFrame;
    use des_ui_winit::HostViewport;

    #[test]
    fn text_rendering_demo_builds_comprehensive_text_specimen() {
        let mut app = NativeTextRenderingDemo::new();
        let mut frame = AppFrame::new(HostViewport::new(1180, 820, 1.0), DocumentInput::default());

        app.update(&mut frame);

        let output = frame.into_output(des_ui_wgpu::RenderOptions::default());
        let text_items = output
            .render_plan
            .text_batches
            .iter()
            .map(|batch| &batch.text)
            .collect::<Vec<_>>();

        assert!(
            text_items.len() >= 35,
            "the specimen should include enough text cases to expose aliasing regressions"
        );
        assert!(
            text_items
                .iter()
                .any(|text| text.element_id.as_str() == "text-size-9" && text.font_size == 9.0)
        );
        assert!(
            text_items
                .iter()
                .any(|text| text.element_id.as_str() == "subpixel-50-sample")
        );
        assert!(
            text_items
                .iter()
                .any(|text| text.element_id.as_str() == "clip-sample")
        );
    }
}
