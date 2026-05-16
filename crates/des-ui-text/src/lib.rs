//! Renderer-neutral text layout contracts.
//!
//! This crate is the document UI text boundary. It owns text layout, hit
//! testing, selection geometry, and glyph-run extraction without depending on
//! a host renderer such as egui.

use parley::{
    Alignment, AlignmentOptions, Cursor, FontContext, Layout, LayoutContext, LineHeight,
    PositionedLayoutItem, Selection, StyleProperty, layout::Affinity,
};
use std::ops::Range;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TextWrap {
    Extend,
    #[default]
    Wrap,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TextAlignment {
    #[default]
    Start,
    End,
    Left,
    Center,
    Right,
    Justify,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextLayoutRequest {
    pub text: String,
    pub font_size: f32,
    pub wrap_width: f32,
    pub wrap: TextWrap,
    pub line_height: Option<f32>,
    pub color: TextColor,
    pub alignment: TextAlignment,
    pub display_scale: f32,
}

impl TextLayoutRequest {
    pub fn new(text: impl Into<String>, font_size: f32, wrap_width: f32) -> Self {
        Self {
            text: text.into(),
            font_size,
            wrap_width,
            wrap: TextWrap::Wrap,
            line_height: None,
            color: TextColor::rgb(0, 0, 0),
            alignment: TextAlignment::Start,
            display_scale: 1.0,
        }
    }

    pub fn wrap(mut self, wrap: TextWrap) -> Self {
        self.wrap = wrap;
        self
    }

    pub fn line_height(mut self, line_height: impl Into<Option<f32>>) -> Self {
        self.line_height = line_height.into();
        self
    }

    pub fn color(mut self, color: TextColor) -> Self {
        self.color = color;
        self
    }

    pub fn alignment(mut self, alignment: TextAlignment) -> Self {
        self.alignment = alignment;
        self
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TextColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl TextColor {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TextPoint {
    pub x: f32,
    pub y: f32,
}

impl TextPoint {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TextSize {
    pub width: f32,
    pub height: f32,
}

impl TextSize {
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TextRect {
    pub origin: TextPoint,
    pub size: TextSize,
}

impl TextRect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            origin: TextPoint::new(x, y),
            size: TextSize::new(width, height),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextSelectionRect {
    pub rect: TextRect,
    pub line_index: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextGlyph {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub advance: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextGlyphRun {
    pub line_index: usize,
    pub font_size: f32,
    pub baseline: f32,
    pub offset: f32,
    pub advance: f32,
    pub color: TextColor,
    pub glyphs: Vec<TextGlyph>,
}

#[derive(Clone)]
pub struct TextLayout {
    text: String,
    layout: Layout<TextColor>,
}

impl TextLayout {
    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn size(&self) -> TextSize {
        TextSize::new(self.layout.width(), self.layout.height())
    }

    pub fn line_count(&self) -> usize {
        self.layout.len()
    }

    pub fn hit_test_byte_index(&self, point: TextPoint) -> usize {
        Cursor::from_point(&self.layout, point.x, point.y).index()
    }

    pub fn selection_rects(&self, range: Range<usize>) -> Vec<TextSelectionRect> {
        if range.start == range.end {
            return Vec::new();
        }
        let anchor = Cursor::from_byte_index(&self.layout, range.start, Affinity::Downstream);
        let focus = Cursor::from_byte_index(&self.layout, range.end, Affinity::Upstream);
        Selection::new(anchor, focus)
            .geometry(&self.layout)
            .into_iter()
            .map(|(rect, line_index)| TextSelectionRect {
                rect: TextRect::new(
                    rect.x0 as f32,
                    rect.y0 as f32,
                    rect.width() as f32,
                    rect.height() as f32,
                ),
                line_index,
            })
            .collect()
    }

    pub fn glyph_runs(&self) -> Vec<TextGlyphRun> {
        let mut runs = Vec::new();
        for (line_index, line) in self.layout.lines().enumerate() {
            for item in line.items() {
                if let PositionedLayoutItem::GlyphRun(run) = item {
                    runs.push(TextGlyphRun {
                        line_index,
                        font_size: run.run().font_size(),
                        baseline: run.baseline(),
                        offset: run.offset(),
                        advance: run.advance(),
                        color: run.style().brush,
                        glyphs: run
                            .positioned_glyphs()
                            .map(|glyph| TextGlyph {
                                id: glyph.id,
                                x: glyph.x,
                                y: glyph.y,
                                advance: glyph.advance,
                            })
                            .collect(),
                    });
                }
            }
        }
        runs
    }
}

#[derive(Default)]
pub struct TextEngine {
    font_context: FontContext,
    layout_context: LayoutContext<TextColor>,
}

impl TextEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn layout(&mut self, request: &TextLayoutRequest) -> TextLayout {
        let mut builder = self.layout_context.ranged_builder(
            &mut self.font_context,
            &request.text,
            request.display_scale,
            true,
        );
        builder.push_default(StyleProperty::FontSize(request.font_size));
        builder.push_default(StyleProperty::Brush(request.color));
        if let Some(line_height) = request.line_height {
            builder.push_default(StyleProperty::LineHeight(LineHeight::Absolute(line_height)));
        }

        let mut layout: Layout<TextColor> = builder.build(&request.text);
        layout.break_all_lines(match request.wrap {
            TextWrap::Extend => None,
            TextWrap::Wrap => Some(request.wrap_width.max(1.0)),
        });
        layout.align(
            match request.alignment {
                TextAlignment::Start => Alignment::Start,
                TextAlignment::End => Alignment::End,
                TextAlignment::Left => Alignment::Left,
                TextAlignment::Center => Alignment::Center,
                TextAlignment::Right => Alignment::Right,
                TextAlignment::Justify => Alignment::Justify,
            },
            AlignmentOptions::default(),
        );

        TextLayout {
            text: request.text.clone(),
            layout,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_engine_measures_wrapped_text() {
        let mut engine = TextEngine::new();
        let layout = engine.layout(&TextLayoutRequest::new(
            "Long labels wrap naturally inside fixed content width.",
            14.0,
            120.0,
        ));

        assert!(layout.size().width <= 120.0);
        assert!(layout.size().height > 0.0);
        assert!(layout.line_count() >= 1);
    }

    #[test]
    fn text_engine_reports_hit_test_indices() {
        let mut engine = TextEngine::new();
        let layout = engine.layout(&TextLayoutRequest::new("hello world", 14.0, 300.0));

        let start = layout.hit_test_byte_index(TextPoint::new(0.0, 0.0));
        let later = layout.hit_test_byte_index(TextPoint::new(80.0, 0.0));

        assert!(start <= later);
        assert!(later <= layout.text().len());
    }

    #[test]
    fn text_engine_reports_selection_rects() {
        let mut engine = TextEngine::new();
        let layout = engine.layout(&TextLayoutRequest::new("hello world", 14.0, 300.0));

        let rects = layout.selection_rects(0..5);

        assert!(!rects.is_empty());
        assert!(rects.iter().all(|rect| rect.rect.size.width > 0.0));
    }

    #[test]
    fn text_engine_exposes_glyph_runs() {
        let mut engine = TextEngine::new();
        let layout = engine.layout(
            &TextLayoutRequest::new("glyphs", 14.0, 300.0).color(TextColor::rgb(10, 20, 30)),
        );

        let runs = layout.glyph_runs();

        assert!(!runs.is_empty());
        assert_eq!(runs[0].color, TextColor::rgb(10, 20, 30));
        assert!(runs.iter().any(|run| !run.glyphs.is_empty()));
    }
}
