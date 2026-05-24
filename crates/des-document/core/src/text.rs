use crate::element::Color;
use crate::geometry::{Point, Size};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextContent {
    runs: Vec<TextRun>,
    semantic_text: String,
}

impl TextContent {
    pub fn new(runs: impl Into<Vec<TextRun>>) -> Self {
        let runs = runs.into();
        let semantic_text = runs.iter().map(|run| run.text.as_str()).collect();
        Self {
            runs,
            semantic_text,
        }
    }

    pub fn plain(text: impl Into<String>) -> Self {
        let text = text.into();
        Self {
            semantic_text: text.clone(),
            runs: vec![TextRun::plain(text)],
        }
    }

    pub fn runs(&self) -> &[TextRun] {
        &self.runs
    }

    pub fn is_empty(&self) -> bool {
        self.runs.iter().all(|run| run.text.is_empty())
    }

    pub fn semantic_text(&self) -> String {
        self.semantic_text.clone()
    }

    pub fn as_str(&self) -> &str {
        &self.semantic_text
    }
}

impl std::ops::Deref for TextContent {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl From<&str> for TextContent {
    fn from(value: &str) -> Self {
        Self::plain(value)
    }
}

impl From<String> for TextContent {
    fn from(value: String) -> Self {
        Self::plain(value)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextRun {
    pub text: String,
    pub style: InlineTextStyle,
}

impl TextRun {
    pub fn plain(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            style: InlineTextStyle::default(),
        }
    }

    pub fn styled(text: impl Into<String>, style: InlineTextStyle) -> Self {
        Self {
            text: text.into(),
            style,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct InlineTextStyle {
    pub color: Option<Color>,
    pub font_size: Option<f32>,
    pub line_height: Option<f32>,
    pub font_family: Option<String>,
    pub font_weight: Option<FontWeight>,
    pub italic: Option<bool>,
    pub underline: Option<bool>,
    pub strikethrough: Option<bool>,
    pub background: Option<Color>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FontWeight {
    Normal,
    Bold,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WhiteSpaceCollapse {
    Collapse,
    Preserve,
    PreserveBreaks,
    BreakSpaces,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TextWrapMode {
    Wrap,
    NoWrap,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OverflowWrap {
    Normal,
    Anywhere,
    BreakWord,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WordBreak {
    Normal,
    BreakAll,
    KeepAll,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WhiteSpace {
    Normal,
    Nowrap,
    Pre,
    PreWrap,
    PreLine,
    BreakSpaces,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TextLayoutStyle {
    pub white_space_collapse: WhiteSpaceCollapse,
    pub text_wrap_mode: TextWrapMode,
    pub overflow_wrap: OverflowWrap,
    pub word_break: WordBreak,
    pub max_lines: Option<usize>,
}

impl TextLayoutStyle {
    pub const DEFAULT: Self = Self {
        white_space_collapse: WhiteSpaceCollapse::Collapse,
        text_wrap_mode: TextWrapMode::Wrap,
        overflow_wrap: OverflowWrap::Normal,
        word_break: WordBreak::Normal,
        max_lines: None,
    };

    pub fn white_space(value: WhiteSpace) -> Self {
        match value {
            WhiteSpace::Normal => Self::DEFAULT,
            WhiteSpace::Nowrap => Self {
                text_wrap_mode: TextWrapMode::NoWrap,
                ..Self::DEFAULT
            },
            WhiteSpace::Pre => Self {
                white_space_collapse: WhiteSpaceCollapse::Preserve,
                text_wrap_mode: TextWrapMode::NoWrap,
                ..Self::DEFAULT
            },
            WhiteSpace::PreWrap => Self {
                white_space_collapse: WhiteSpaceCollapse::Preserve,
                text_wrap_mode: TextWrapMode::Wrap,
                ..Self::DEFAULT
            },
            WhiteSpace::PreLine => Self {
                white_space_collapse: WhiteSpaceCollapse::PreserveBreaks,
                text_wrap_mode: TextWrapMode::Wrap,
                ..Self::DEFAULT
            },
            WhiteSpace::BreakSpaces => Self {
                white_space_collapse: WhiteSpaceCollapse::BreakSpaces,
                text_wrap_mode: TextWrapMode::Wrap,
                overflow_wrap: OverflowWrap::Anywhere,
                ..Self::DEFAULT
            },
        }
    }
}

impl Default for TextLayoutStyle {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct NormalizedText {
    semantic_text: String,
    layout_text: String,
    layout_to_semantic: Vec<usize>,
    semantic_to_layout: Vec<usize>,
    runs: Vec<TextLayoutRun>,
}

impl NormalizedText {
    pub fn from_content(content: &TextContent, style: TextLayoutStyle) -> Self {
        TextNormalizer::new(style).normalize(content)
    }

    pub fn semantic_text(&self) -> &str {
        &self.semantic_text
    }

    pub fn layout_text(&self) -> &str {
        &self.layout_text
    }

    pub fn runs(&self) -> &[TextLayoutRun] {
        &self.runs
    }

    pub fn layout_to_semantic_index(&self, layout_index: usize) -> usize {
        if self.layout_text.is_empty() {
            return 0;
        }
        self.layout_to_semantic
            .get(layout_index.min(self.layout_to_semantic.len().saturating_sub(1)))
            .copied()
            .unwrap_or(self.semantic_text.chars().count())
    }

    pub fn semantic_to_layout_index(&self, semantic_index: usize) -> usize {
        self.semantic_to_layout
            .get(semantic_index.min(self.semantic_to_layout.len().saturating_sub(1)))
            .copied()
            .unwrap_or(self.layout_text.chars().count())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextLayoutRun {
    pub text: String,
    pub style: InlineTextStyle,
    pub source_run: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextLayoutRequest<'a> {
    pub text: &'a NormalizedText,
    pub font_size: f32,
    pub color: Color,
    pub wrap_width: f32,
    pub layout_style: TextLayoutStyle,
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

    fn text_index_at(&mut self, request: TextLayoutRequest<'_>, point: Point) -> usize {
        fallback_text_index_at(request, point)
    }
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

struct TextNormalizer {
    style: TextLayoutStyle,
    semantic_text: String,
    layout_text: String,
    layout_to_semantic: Vec<usize>,
    semantic_to_layout: Vec<usize>,
    runs: Vec<TextLayoutRun>,
    pending_collapsed_space: Option<usize>,
    last_emitted_space: bool,
}

impl TextNormalizer {
    fn new(style: TextLayoutStyle) -> Self {
        Self {
            style,
            semantic_text: String::new(),
            layout_text: String::new(),
            layout_to_semantic: Vec::new(),
            semantic_to_layout: Vec::new(),
            runs: Vec::new(),
            pending_collapsed_space: None,
            last_emitted_space: false,
        }
    }

    fn normalize(mut self, content: &TextContent) -> NormalizedText {
        for (run_index, run) in content.runs().iter().enumerate() {
            let run_start = self.layout_text.chars().count();
            for ch in run.text.chars() {
                let semantic_index = self.semantic_text.chars().count();
                self.semantic_text.push(ch);
                match self.style.white_space_collapse {
                    WhiteSpaceCollapse::Collapse => {
                        if is_css_space(ch) {
                            self.pending_collapsed_space.get_or_insert(semantic_index);
                            self.semantic_to_layout
                                .push(self.layout_text.chars().count());
                        } else {
                            self.flush_collapsed_space(run_index, &run.style);
                            self.emit_char(ch, semantic_index);
                        }
                    }
                    WhiteSpaceCollapse::Preserve => self.emit_char(ch, semantic_index),
                    WhiteSpaceCollapse::PreserveBreaks => {
                        if ch == '\n' {
                            self.emit_char(ch, semantic_index);
                        } else if is_css_space(ch) {
                            self.pending_collapsed_space.get_or_insert(semantic_index);
                            self.semantic_to_layout
                                .push(self.layout_text.chars().count());
                        } else {
                            self.flush_collapsed_space(run_index, &run.style);
                            self.emit_char(ch, semantic_index);
                        }
                    }
                    WhiteSpaceCollapse::BreakSpaces => {
                        if ch == '\t' {
                            self.emit_char(' ', semantic_index);
                        } else {
                            self.emit_char(ch, semantic_index);
                        }
                    }
                }
            }
            let run_end = self.layout_text.chars().count();
            if run_end > run_start {
                self.runs.push(TextLayoutRun {
                    text: layout_slice_by_chars(&self.layout_text, run_start, run_end),
                    style: run.style.clone(),
                    source_run: run_index,
                });
            }
        }

        let end = self.layout_text.chars().count();
        self.semantic_to_layout.push(end);
        self.layout_to_semantic
            .push(self.semantic_text.chars().count());

        NormalizedText {
            semantic_text: self.semantic_text,
            layout_text: self.layout_text,
            layout_to_semantic: self.layout_to_semantic,
            semantic_to_layout: self.semantic_to_layout,
            runs: self.runs,
        }
    }

    fn flush_collapsed_space(&mut self, _run_index: usize, _style: &InlineTextStyle) {
        let Some(semantic_index) = self.pending_collapsed_space.take() else {
            return;
        };
        if self.layout_text.is_empty() || self.last_emitted_space {
            return;
        }
        self.emit_char(' ', semantic_index);
    }

    fn emit_char(&mut self, ch: char, semantic_index: usize) {
        let layout_index = self.layout_text.chars().count();
        self.layout_text.push(ch);
        self.layout_to_semantic.push(semantic_index);
        if self.semantic_to_layout.len() == semantic_index {
            self.semantic_to_layout.push(layout_index);
        }
        self.last_emitted_space = ch == ' ' || ch == '\n';
    }
}

fn is_css_space(ch: char) -> bool {
    matches!(ch, ' ' | '\t' | '\n' | '\r' | '\x0C')
}

fn fallback_measure_text(request: TextLayoutRequest<'_>) -> TextLayoutResult {
    let line_height = fallback_line_height(&request);
    let paragraphs = fallback_paragraphs(&request);
    if request.layout_style.text_wrap_mode == TextWrapMode::NoWrap
        || !request.wrap_width.is_finite()
    {
        let line_count = paragraphs.len().max(1);
        let unwrapped_width = paragraphs
            .iter()
            .map(|paragraph| paragraph.iter().map(|ch| ch.width).sum::<f32>())
            .fold(0.0, f32::max);
        return TextLayoutResult {
            size: Size::new(unwrapped_width, line_height * line_count as f32),
            line_count,
            elided: false,
        };
    }

    let max_width = request.wrap_width.max(1.0);
    let max_lines = request.layout_style.max_lines.unwrap_or(usize::MAX).max(1);
    let mut line_count = 0usize;
    let mut max_line_width: f32 = 0.0;
    let mut elided = false;
    for paragraph in paragraphs {
        let lines = wrap_paragraph(
            &paragraph,
            max_width,
            request.layout_style.overflow_wrap,
            request.layout_style.word_break,
        );
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

fn wrap_paragraph(
    text: &[FallbackLayoutChar],
    max_width: f32,
    overflow_wrap: OverflowWrap,
    word_break: WordBreak,
) -> Vec<f32> {
    if text.is_empty() {
        return vec![0.0];
    }

    if overflow_wrap == OverflowWrap::Anywhere || word_break == WordBreak::BreakAll {
        return wrap_anywhere(text, max_width);
    }

    let mut rows = Vec::new();
    let mut current_width = 0.0;
    let mut saw_word = false;
    let mut words = text.split(|ch| ch.value == ' ').peekable();
    while let Some(word) = words.next() {
        let word_width = word.iter().map(|ch| ch.width).sum::<f32>();
        let next_width = if !saw_word {
            word_width
        } else {
            let space_width =
                fallback_char_width(' ', word.first().map_or(13.0, |ch| ch.font_size));
            current_width + space_width + word_width
        };
        if current_width > f32::EPSILON && next_width > max_width {
            rows.push(current_width);
            current_width = word_width;
        } else {
            current_width = next_width;
        }
        saw_word = true;
    }
    if overflow_wrap == OverflowWrap::BreakWord && current_width > max_width {
        rows.extend(wrap_anywhere(text, max_width));
    } else {
        rows.push(current_width);
    }
    rows
}

fn wrap_anywhere(text: &[FallbackLayoutChar], max_width: f32) -> Vec<f32> {
    let mut rows = Vec::new();
    let mut current = 0.0;
    for ch in text {
        if current > f32::EPSILON && current + ch.width > max_width {
            rows.push(current);
            current = 0.0;
        }
        current += ch.width;
    }
    if current > f32::EPSILON || rows.is_empty() {
        rows.push(current);
    }
    rows
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct FallbackLayoutChar {
    value: char,
    width: f32,
    font_size: f32,
}

fn fallback_paragraphs(request: &TextLayoutRequest<'_>) -> Vec<Vec<FallbackLayoutChar>> {
    let mut paragraphs = vec![Vec::new()];
    for run in request.text.runs() {
        let font_size = run.style.font_size.unwrap_or(request.font_size).max(1.0);
        for ch in run.text.chars() {
            if ch == '\n' {
                paragraphs.push(Vec::new());
            } else {
                paragraphs
                    .last_mut()
                    .expect("paragraph list always has a current paragraph")
                    .push(FallbackLayoutChar {
                        value: ch,
                        width: fallback_char_width(ch, font_size),
                        font_size,
                    });
            }
        }
    }
    paragraphs
}

fn fallback_line_height(request: &TextLayoutRequest<'_>) -> f32 {
    let inherited = request
        .line_height
        .unwrap_or_else(|| fallback_default_line_height(request.font_size));
    request
        .text
        .runs()
        .iter()
        .map(|run| {
            run.style.line_height.unwrap_or_else(|| {
                fallback_default_line_height(run.style.font_size.unwrap_or(request.font_size))
            })
        })
        .fold(inherited.max(1.0), f32::max)
}

fn fallback_default_line_height(font_size: f32) -> f32 {
    (font_size * 1.25).max(18.0)
}

fn fallback_char_width(ch: char, font_size: f32) -> f32 {
    if ch == '\n' {
        0.0
    } else {
        font_size.max(1.0) * (7.5 / 13.0)
    }
}

fn fallback_text_index_at(request: TextLayoutRequest<'_>, point: Point) -> usize {
    let char_width = (request.font_size * (7.5 / 13.0)).max(1.0);
    let line_height = request
        .line_height
        .unwrap_or_else(|| (request.font_size * 1.25).max(18.0))
        .max(1.0);
    let target_line = (point.y / line_height).floor().max(0.0) as usize;
    let target_column = (point.x / char_width).round().max(0.0) as usize;
    let mut line = 0usize;
    let mut layout_index = 0usize;

    for segment in request.text.layout_text().split_inclusive('\n') {
        let line_text = segment.strip_suffix('\n').unwrap_or(segment);
        let line_len = line_text.chars().count();
        if line == target_line {
            return request
                .text
                .layout_to_semantic_index(layout_index + target_column.min(line_len));
        }
        layout_index += segment.chars().count();
        line += 1;
    }

    request.text.semantic_text().chars().count()
}

fn layout_slice_by_chars(value: &str, start: usize, end: usize) -> String {
    value.chars().skip(start).take(end - start).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_css_text_collapses_space_and_newlines() {
        let content = TextContent::plain("Alpha\t beta\n\n gamma");
        let normalized = NormalizedText::from_content(&content, TextLayoutStyle::default());

        assert_eq!(normalized.semantic_text(), "Alpha\t beta\n\n gamma");
        assert_eq!(normalized.layout_text(), "Alpha beta gamma");
    }

    #[test]
    fn css_white_space_presets_choose_browser_like_axes() {
        assert_eq!(
            TextLayoutStyle::white_space(WhiteSpace::Normal),
            TextLayoutStyle {
                white_space_collapse: WhiteSpaceCollapse::Collapse,
                text_wrap_mode: TextWrapMode::Wrap,
                overflow_wrap: OverflowWrap::Normal,
                word_break: WordBreak::Normal,
                max_lines: None,
            }
        );
        assert_eq!(
            TextLayoutStyle::white_space(WhiteSpace::Pre).text_wrap_mode,
            TextWrapMode::NoWrap
        );
        assert_eq!(
            TextLayoutStyle::white_space(WhiteSpace::PreWrap).white_space_collapse,
            WhiteSpaceCollapse::Preserve
        );
        assert_eq!(
            TextLayoutStyle::white_space(WhiteSpace::PreLine).white_space_collapse,
            WhiteSpaceCollapse::PreserveBreaks
        );
        assert_eq!(
            TextLayoutStyle::white_space(WhiteSpace::BreakSpaces).overflow_wrap,
            OverflowWrap::Anywhere
        );
    }

    #[test]
    fn pre_line_preserves_breaks_but_collapses_other_space() {
        let content = TextContent::plain("Alpha   beta\n  gamma");
        let normalized = NormalizedText::from_content(
            &content,
            TextLayoutStyle::white_space(WhiteSpace::PreLine),
        );

        assert_eq!(normalized.layout_text(), "Alpha beta\ngamma");
    }

    #[test]
    fn rich_runs_derive_semantic_plain_text_and_layout_runs() {
        let content = TextContent::new(vec![
            TextRun::plain("Hello"),
            TextRun::styled(
                " world",
                InlineTextStyle {
                    font_weight: Some(FontWeight::Bold),
                    ..InlineTextStyle::default()
                },
            ),
        ]);
        let normalized = NormalizedText::from_content(&content, TextLayoutStyle::default());

        assert_eq!(content.semantic_text(), "Hello world");
        assert_eq!(normalized.layout_text(), "Hello world");
        assert_eq!(normalized.runs().len(), 2);
        assert_eq!(
            normalized.runs()[1].style.font_weight,
            Some(FontWeight::Bold)
        );
    }

    #[test]
    fn normalized_text_maps_between_semantic_and_layout_indices() {
        let content = TextContent::plain("a   b");
        let normalized = NormalizedText::from_content(&content, TextLayoutStyle::default());

        assert_eq!(normalized.layout_text(), "a b");
        assert_eq!(normalized.layout_to_semantic_index(2), 4);
        assert_eq!(normalized.semantic_to_layout_index(4), 2);
    }

    #[test]
    fn fallback_measurement_respects_anywhere_breaking() {
        let content = TextContent::plain("Supercalifragilistic");
        let mut style = TextLayoutStyle::default();
        style.overflow_wrap = OverflowWrap::Anywhere;
        let normalized = NormalizedText::from_content(&content, style);
        let mut measurer = FallbackTextMeasurer;

        let measured = measurer.measure_text(TextLayoutRequest {
            text: &normalized,
            font_size: 13.0,
            color: Color::rgb(255, 255, 255),
            wrap_width: 20.0,
            layout_style: style,
            line_height: None,
        });

        assert!(measured.line_count > 1);
    }

    #[test]
    fn fallback_measurement_uses_inline_run_font_sizes() {
        let small = TextContent::plain("MMMM");
        let rich = TextContent::new(vec![
            TextRun::plain("MM"),
            TextRun::styled(
                "MM",
                InlineTextStyle {
                    font_size: Some(26.0),
                    ..InlineTextStyle::default()
                },
            ),
        ]);
        let normalized_small = NormalizedText::from_content(&small, TextLayoutStyle::default());
        let normalized_rich = NormalizedText::from_content(&rich, TextLayoutStyle::default());
        let mut measurer = FallbackTextMeasurer;

        let small = measurer.measure_text(TextLayoutRequest {
            text: &normalized_small,
            font_size: 13.0,
            color: Color::rgb(255, 255, 255),
            wrap_width: f32::INFINITY,
            layout_style: TextLayoutStyle::white_space(WhiteSpace::Pre),
            line_height: None,
        });
        let rich = measurer.measure_text(TextLayoutRequest {
            text: &normalized_rich,
            font_size: 13.0,
            color: Color::rgb(255, 255, 255),
            wrap_width: f32::INFINITY,
            layout_style: TextLayoutStyle::white_space(WhiteSpace::Pre),
            line_height: None,
        });

        assert!(rich.size.width > small.size.width);
        assert!(rich.size.height > small.size.height);
    }
}
