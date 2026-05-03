use crate::geometry::Size;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TextWrapMode {
    Extend,
    Wrap,
    Truncate,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextLayoutRequest<'a> {
    pub text: &'a str,
    pub font_size: f32,
    pub wrap_width: f32,
    pub wrap_mode: TextWrapMode,
    pub max_lines: Option<usize>,
    pub line_height: Option<f32>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TextLayoutResult {
    pub size: Size,
    pub line_count: usize,
    pub elided: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TextMeasurerKey(&'static str);

impl TextMeasurerKey {
    pub const FALLBACK: Self = Self("fallback");

    pub const fn new(name: &'static str) -> Self {
        Self(name)
    }
}

pub trait TextMeasurer {
    fn cache_key(&self) -> TextMeasurerKey;

    fn measure_text(&mut self, request: TextLayoutRequest<'_>) -> TextLayoutResult;
}

#[derive(Default)]
pub struct FallbackTextMeasurer;

impl TextMeasurer for FallbackTextMeasurer {
    fn cache_key(&self) -> TextMeasurerKey {
        TextMeasurerKey::FALLBACK
    }

    fn measure_text(&mut self, request: TextLayoutRequest<'_>) -> TextLayoutResult {
        fallback_measure_text(request)
    }
}

fn fallback_measure_text(request: TextLayoutRequest<'_>) -> TextLayoutResult {
    let line_height = request
        .line_height
        .unwrap_or_else(|| (request.font_size * 1.25).max(18.0))
        .max(1.0);
    let unwrapped_width = fallback_text_width(request.text, request.font_size);
    if request.wrap_mode == TextWrapMode::Extend || !request.wrap_width.is_finite() {
        return TextLayoutResult {
            size: Size::new(unwrapped_width, line_height),
            line_count: 1,
            elided: false,
        };
    }

    let max_width = request.wrap_width.max(1.0);
    if request.wrap_mode == TextWrapMode::Truncate {
        return TextLayoutResult {
            size: Size::new(unwrapped_width.min(max_width), line_height),
            line_count: 1,
            elided: unwrapped_width > max_width,
        };
    }

    let mut line_count = 0usize;
    let mut max_line_width: f32 = 0.0;
    let mut elided = false;
    let max_lines = request.max_lines.unwrap_or(usize::MAX).max(1);
    for paragraph in request.text.split('\n') {
        let lines = wrap_paragraph(paragraph, request.font_size, max_width);
        for line_width in lines {
            if line_count >= max_lines {
                elided = true;
                break;
            }
            max_line_width = max_line_width.max(line_width);
            line_count += 1;
        }
        if elided {
            break;
        }
    }

    if line_count == 0 {
        line_count = 1;
    }

    TextLayoutResult {
        size: Size::new(
            max_line_width.min(max_width),
            line_height * line_count as f32,
        ),
        line_count,
        elided,
    }
}

fn wrap_paragraph(text: &str, font_size: f32, max_width: f32) -> Vec<f32> {
    if text.is_empty() {
        return vec![0.0];
    }

    let space_width = fallback_text_width(" ", font_size);
    let mut rows = Vec::new();
    let mut current_width = 0.0;
    for word in text.split_whitespace() {
        let word_width = fallback_text_width(word, font_size);
        let next_width = if current_width <= f32::EPSILON {
            word_width
        } else {
            current_width + space_width + word_width
        };
        if current_width > f32::EPSILON && next_width > max_width {
            rows.push(current_width);
            current_width = word_width;
        } else {
            current_width = next_width;
        }
    }
    rows.push(current_width);
    rows
}

fn fallback_text_width(text: &str, font_size: f32) -> f32 {
    text.chars().count() as f32 * font_size * (7.5 / 13.0)
}
