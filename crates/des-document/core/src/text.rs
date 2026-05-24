use crate::element::Color;
use crate::geometry::{Point, Size};
use des_layout::style::Direction;

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

    pub fn semantic_text(&self) -> &str {
        &self.semantic_text
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
    pub letter_spacing: Option<f32>,
    pub font_family: Option<String>,
    pub font_weight: Option<FontWeight>,
    pub font_stretch: Option<FontStretch>,
    pub font_style: Option<FontStyle>,
    pub text_transform: Option<TextTransform>,
    pub text_decoration: Option<TextDecoration>,
    pub vertical_align: Option<TextVerticalAlign>,
    pub background: Option<Color>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FontWeight(u16);

impl FontWeight {
    pub const MIN: u16 = 1;
    pub const MAX: u16 = 1000;
    pub const NORMAL: Self = Self(400);
    pub const BOLD: Self = Self(700);

    pub const fn new(value: u16) -> Self {
        if value < Self::MIN {
            Self(Self::MIN)
        } else if value > Self::MAX {
            Self(Self::MAX)
        } else {
            Self(value)
        }
    }

    pub const fn value(self) -> u16 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FontStretch(f32);

impl FontStretch {
    pub const MIN_PERCENT: f32 = 50.0;
    pub const MAX_PERCENT: f32 = 200.0;
    pub const NORMAL: Self = Self(100.0);
    pub const ULTRA_CONDENSED: Self = Self(50.0);
    pub const EXTRA_CONDENSED: Self = Self(62.5);
    pub const CONDENSED: Self = Self(75.0);
    pub const SEMI_CONDENSED: Self = Self(87.5);
    pub const SEMI_EXPANDED: Self = Self(112.5);
    pub const EXPANDED: Self = Self(125.0);
    pub const EXTRA_EXPANDED: Self = Self(150.0);
    pub const ULTRA_EXPANDED: Self = Self(200.0);

    pub fn percent(value: f32) -> Self {
        Self(value.clamp(Self::MIN_PERCENT, Self::MAX_PERCENT))
    }

    pub const fn value(self) -> f32 {
        self.0
    }

    pub const fn width_factor(self) -> f32 {
        self.0 / 100.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FontStyle {
    Normal,
    Italic,
    Oblique,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TextDecoration {
    pub underline: bool,
    pub overline: bool,
    pub line_through: bool,
    pub color: Option<Color>,
    pub thickness: Option<f32>,
}

impl TextDecoration {
    pub const NONE: Self = Self {
        underline: false,
        overline: false,
        line_through: false,
        color: None,
        thickness: None,
    };
    pub const UNDERLINE: Self = Self {
        underline: true,
        ..Self::NONE
    };
    pub const OVERLINE: Self = Self {
        overline: true,
        ..Self::NONE
    };
    pub const LINE_THROUGH: Self = Self {
        line_through: true,
        ..Self::NONE
    };

    pub const fn lines(underline: bool, overline: bool, line_through: bool) -> Self {
        Self {
            underline,
            overline,
            line_through,
            ..Self::NONE
        }
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    pub fn thickness(mut self, thickness: f32) -> Self {
        self.thickness = Some(thickness.max(0.0));
        self
    }

    pub fn stroke_color(self, current_color: Color) -> Color {
        self.color.unwrap_or(current_color)
    }

    pub fn stroke_thickness(self) -> f32 {
        self.thickness.unwrap_or(1.0).max(0.0)
    }

    pub const fn is_none(self) -> bool {
        !self.underline && !self.overline && !self.line_through
    }
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
pub enum TextAlign {
    Start,
    Center,
    End,
    Justify,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TextOverflow {
    Clip,
    Ellipsis,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TextVerticalAlign {
    Baseline,
    Top,
    Middle,
    Bottom,
    Sub,
    Super,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TextTransform {
    None,
    Uppercase,
    Lowercase,
    Capitalize,
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
    pub text_align: TextAlign,
    pub text_overflow: TextOverflow,
    pub text_transform: TextTransform,
    pub tab_size: u16,
    pub max_lines: Option<usize>,
}

impl TextLayoutStyle {
    pub const DEFAULT_TAB_SIZE: u16 = 8;

    pub const DEFAULT: Self = Self {
        white_space_collapse: WhiteSpaceCollapse::Collapse,
        text_wrap_mode: TextWrapMode::Wrap,
        overflow_wrap: OverflowWrap::Normal,
        word_break: WordBreak::Normal,
        text_align: TextAlign::Start,
        text_overflow: TextOverflow::Clip,
        text_transform: TextTransform::None,
        tab_size: Self::DEFAULT_TAB_SIZE,
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
    pub direction: Direction,
    pub wrap_width: f32,
    pub layout_style: TextLayoutStyle,
    pub line_height: Option<f32>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextLayoutResult {
    pub size: Size,
    pub line_count: usize,
    pub elided: bool,
    pub first_baseline: Option<f32>,
    pub lines: Vec<TextLayoutLine>,
}

impl TextLayoutResult {
    pub fn new(size: Size, line_count: usize, elided: bool) -> Self {
        Self {
            size,
            line_count,
            elided,
            first_baseline: None,
            lines: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TextLayoutLine {
    pub layout_start: usize,
    pub layout_end: usize,
    pub semantic_start: usize,
    pub semantic_end: usize,
    pub x_offset: f32,
    pub width: f32,
    pub height: f32,
    pub baseline: f32,
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
    capitalize_next: bool,
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
            capitalize_next: true,
        }
    }

    fn normalize(mut self, content: &TextContent) -> NormalizedText {
        for (run_index, run) in content.runs().iter().enumerate() {
            let run_start = self.layout_text.chars().count();
            for ch in run.text.chars() {
                let semantic_index = self.semantic_text.chars().count();
                self.semantic_text.push(ch);
                let text_transform = run
                    .style
                    .text_transform
                    .unwrap_or(self.style.text_transform);
                match self.style.white_space_collapse {
                    WhiteSpaceCollapse::Collapse => {
                        if is_css_space(ch) {
                            self.capitalize_next = true;
                            self.pending_collapsed_space.get_or_insert(semantic_index);
                            self.semantic_to_layout
                                .push(self.layout_text.chars().count());
                        } else {
                            self.flush_collapsed_space(run_index, &run.style);
                            self.emit_transformed_char(ch, semantic_index, text_transform);
                        }
                    }
                    WhiteSpaceCollapse::Preserve => {
                        self.emit_preserved_char(ch, semantic_index, text_transform)
                    }
                    WhiteSpaceCollapse::PreserveBreaks => {
                        if ch == '\n' {
                            self.emit_char(ch, semantic_index);
                            self.capitalize_next = true;
                        } else if is_css_space(ch) {
                            self.capitalize_next = true;
                            self.pending_collapsed_space.get_or_insert(semantic_index);
                            self.semantic_to_layout
                                .push(self.layout_text.chars().count());
                        } else {
                            self.flush_collapsed_space(run_index, &run.style);
                            self.emit_transformed_char(ch, semantic_index, text_transform);
                        }
                    }
                    WhiteSpaceCollapse::BreakSpaces => {
                        if ch == '\t' {
                            self.emit_tab(semantic_index);
                        } else {
                            self.emit_transformed_char(ch, semantic_index, text_transform);
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

    fn emit_transformed_char(
        &mut self,
        ch: char,
        semantic_index: usize,
        text_transform: TextTransform,
    ) {
        match text_transform {
            TextTransform::None => self.emit_char(ch, semantic_index),
            TextTransform::Uppercase => {
                for transformed in ch.to_uppercase() {
                    self.emit_char(transformed, semantic_index);
                }
            }
            TextTransform::Lowercase => {
                for transformed in ch.to_lowercase() {
                    self.emit_char(transformed, semantic_index);
                }
            }
            TextTransform::Capitalize => {
                if ch.is_alphabetic() && self.capitalize_next {
                    for transformed in ch.to_uppercase() {
                        self.emit_char(transformed, semantic_index);
                    }
                } else {
                    self.emit_char(ch, semantic_index);
                }
                self.capitalize_next = !ch.is_alphanumeric();
            }
        }
        if text_transform != TextTransform::Capitalize {
            self.capitalize_next = !ch.is_alphanumeric();
        }
    }

    fn emit_preserved_char(
        &mut self,
        ch: char,
        semantic_index: usize,
        text_transform: TextTransform,
    ) {
        if ch == '\t' {
            self.emit_tab(semantic_index);
        } else {
            self.emit_transformed_char(ch, semantic_index, text_transform);
        }
    }

    fn emit_tab(&mut self, semantic_index: usize) {
        let width = self.style.tab_size.max(1);
        for _ in 0..width {
            self.emit_char(' ', semantic_index);
        }
        self.capitalize_next = true;
    }
}

fn is_css_space(ch: char) -> bool {
    matches!(ch, ' ' | '\t' | '\n' | '\r' | '\x0C')
}

fn fallback_measure_text(request: TextLayoutRequest<'_>) -> TextLayoutResult {
    let line_height = fallback_line_height(&request);
    let max_width = fallback_wrap_width(&request);
    let lines = fallback_layout_lines(&request);
    let max_lines = request.layout_style.max_lines.unwrap_or(usize::MAX).max(1);
    let visible_lines = lines.len().min(max_lines).max(1);
    let max_line_width = lines
        .iter()
        .take(visible_lines)
        .map(|line| line.width)
        .fold(0.0, f32::max);
    let elided = lines.len() > visible_lines;

    let visible_layout_lines: Vec<_> = lines
        .iter()
        .take(visible_lines)
        .map(|line| {
            line.to_text_layout_line(
                request.text,
                line_height,
                fallback_baseline_for_line(line, line_height),
                fallback_line_x_offset(
                    request.layout_style.text_align,
                    request.direction,
                    max_width,
                    line.width,
                ),
            )
        })
        .collect();
    let first_baseline = visible_layout_lines.first().map(|line| line.baseline);
    TextLayoutResult {
        size: Size::new(
            max_line_width.min(max_width),
            line_height * visible_lines as f32,
        ),
        line_count: visible_lines,
        elided,
        first_baseline,
        lines: visible_layout_lines,
    }
}

fn fallback_wrap_width(request: &TextLayoutRequest<'_>) -> f32 {
    if request.layout_style.text_wrap_mode == TextWrapMode::NoWrap
        || !request.wrap_width.is_finite()
    {
        f32::INFINITY
    } else {
        request.wrap_width.max(1.0)
    }
}

fn fallback_layout_lines(request: &TextLayoutRequest<'_>) -> Vec<FallbackLayoutLine> {
    let paragraphs = fallback_paragraphs(request);
    if request.layout_style.text_wrap_mode == TextWrapMode::NoWrap
        || !request.wrap_width.is_finite()
    {
        return paragraphs
            .into_iter()
            .map(FallbackLayoutLine::from)
            .collect();
    }

    let max_width = fallback_wrap_width(request);
    paragraphs
        .into_iter()
        .flat_map(|paragraph| {
            wrap_paragraph(
                &paragraph,
                max_width,
                request.layout_style.overflow_wrap,
                request.layout_style.word_break,
            )
        })
        .collect()
}

fn wrap_paragraph(
    text: &[FallbackLayoutChar],
    max_width: f32,
    overflow_wrap: OverflowWrap,
    word_break: WordBreak,
) -> Vec<FallbackLayoutLine> {
    if text.is_empty() {
        return vec![FallbackLayoutLine::default()];
    }

    if overflow_wrap == OverflowWrap::Anywhere || word_break == WordBreak::BreakAll {
        return wrap_anywhere(text, max_width);
    }

    if overflow_wrap == OverflowWrap::BreakWord {
        return wrap_words_or_break_long_segments(text, max_width);
    }

    wrap_at_spaces(text, max_width)
}

fn wrap_at_spaces(text: &[FallbackLayoutChar], max_width: f32) -> Vec<FallbackLayoutLine> {
    let mut rows = Vec::new();
    let mut current = Vec::new();
    let mut current_width = 0.0;
    let mut last_break_after = None;

    for ch in text {
        current.push(*ch);
        current_width += ch.width;
        if ch.value == ' ' {
            last_break_after = Some(current.len());
        }

        if current_width > max_width
            && current.len() > 1
            && let Some(break_after) = last_break_after
            && break_after < current.len()
        {
            let remainder = current.split_off(break_after);
            rows.push(FallbackLayoutLine::from(std::mem::take(&mut current)));
            current = remainder;
            current_width = current.iter().map(|ch| ch.width).sum();
            last_break_after = current
                .iter()
                .rposition(|ch| ch.value == ' ')
                .map(|index| index + 1);
        }
    }

    rows.push(FallbackLayoutLine::from(current));
    rows
}

fn wrap_words_or_break_long_segments(
    text: &[FallbackLayoutChar],
    max_width: f32,
) -> Vec<FallbackLayoutLine> {
    wrap_at_spaces(text, max_width)
        .into_iter()
        .flat_map(|line| {
            if line.width > max_width {
                wrap_anywhere(&line.chars, max_width)
            } else {
                vec![line]
            }
        })
        .collect()
}

fn wrap_anywhere(text: &[FallbackLayoutChar], max_width: f32) -> Vec<FallbackLayoutLine> {
    let mut rows = Vec::new();
    let mut current_chars = Vec::new();
    let mut current = 0.0;
    for ch in text {
        if current > f32::EPSILON && current + ch.width > max_width {
            rows.push(FallbackLayoutLine::from(std::mem::take(&mut current_chars)));
            current = 0.0;
        }
        current_chars.push(*ch);
        current += ch.width;
    }
    if current > f32::EPSILON || rows.is_empty() {
        rows.push(FallbackLayoutLine::from(current_chars));
    }
    rows
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct FallbackLayoutChar {
    value: char,
    width: f32,
    font_size: f32,
    baseline_shift: f32,
    layout_index: usize,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct FallbackLayoutLine {
    chars: Vec<FallbackLayoutChar>,
    width: f32,
}

impl From<Vec<FallbackLayoutChar>> for FallbackLayoutLine {
    fn from(chars: Vec<FallbackLayoutChar>) -> Self {
        let width = chars.iter().map(|ch| ch.width).sum();
        Self { chars, width }
    }
}

impl FallbackLayoutLine {
    fn to_text_layout_line(
        &self,
        text: &NormalizedText,
        height: f32,
        baseline: f32,
        x_offset: f32,
    ) -> TextLayoutLine {
        let layout_start = self.chars.first().map_or(0, |ch| ch.layout_index);
        let layout_end = self
            .chars
            .last()
            .map_or(layout_start, |ch| ch.layout_index + 1);
        TextLayoutLine {
            layout_start,
            layout_end,
            semantic_start: text.layout_to_semantic_index(layout_start),
            semantic_end: text.layout_to_semantic_index(layout_end),
            x_offset,
            width: self.width,
            height,
            baseline,
        }
    }
}

fn fallback_baseline_for_line(line: &FallbackLayoutLine, line_height: f32) -> f32 {
    let font_baseline = line
        .chars
        .iter()
        .map(|ch| fallback_font_baseline(ch.font_size) + ch.baseline_shift.max(0.0))
        .fold(0.0, f32::max);
    font_baseline.clamp(0.0, line_height)
}

fn fallback_paragraphs(request: &TextLayoutRequest<'_>) -> Vec<Vec<FallbackLayoutChar>> {
    let mut paragraphs = vec![Vec::new()];
    let mut layout_index = 0usize;
    for run in request.text.runs() {
        let font_size = run.style.font_size.unwrap_or(request.font_size).max(1.0);
        let letter_spacing = run.style.letter_spacing.unwrap_or(0.0).max(0.0);
        let baseline_shift = fallback_baseline_shift(
            run.style
                .vertical_align
                .unwrap_or(TextVerticalAlign::Baseline),
            font_size,
        );
        let font_stretch = run
            .style
            .font_stretch
            .unwrap_or(FontStretch::NORMAL)
            .width_factor();
        for ch in run.text.chars() {
            if ch == '\n' {
                paragraphs.push(Vec::new());
                layout_index += 1;
            } else {
                paragraphs
                    .last_mut()
                    .expect("paragraph list always has a current paragraph")
                    .push(FallbackLayoutChar {
                        value: ch,
                        width: fallback_char_width(ch, font_size, letter_spacing, font_stretch),
                        font_size,
                        baseline_shift,
                        layout_index,
                    });
                layout_index += 1;
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
            let font_size = run.style.font_size.unwrap_or(request.font_size);
            let baseline_shift = fallback_baseline_shift(
                run.style
                    .vertical_align
                    .unwrap_or(TextVerticalAlign::Baseline),
                font_size,
            )
            .abs();
            run.style.line_height.unwrap_or_else(|| {
                fallback_default_line_height(run.style.font_size.unwrap_or(request.font_size))
            }) + baseline_shift
        })
        .fold(inherited.max(1.0), f32::max)
}

fn fallback_font_baseline(font_size: f32) -> f32 {
    font_size.max(1.0) * 0.8
}

fn fallback_baseline_shift(vertical_align: TextVerticalAlign, font_size: f32) -> f32 {
    match vertical_align {
        TextVerticalAlign::Super => font_size.max(1.0) * 0.35,
        TextVerticalAlign::Sub => -(font_size.max(1.0) * 0.2),
        TextVerticalAlign::Baseline
        | TextVerticalAlign::Top
        | TextVerticalAlign::Middle
        | TextVerticalAlign::Bottom => 0.0,
    }
}

fn fallback_default_line_height(font_size: f32) -> f32 {
    (font_size * 1.25).max(18.0)
}

fn fallback_char_width(ch: char, font_size: f32, letter_spacing: f32, font_stretch: f32) -> f32 {
    if ch == '\n' {
        0.0
    } else {
        ((font_size.max(1.0) * (7.5 / 13.0) * font_stretch.max(0.0)) + letter_spacing.max(0.0))
            .max(0.0)
    }
}

fn fallback_text_index_at(request: TextLayoutRequest<'_>, point: Point) -> usize {
    let line_height = fallback_line_height(&request);
    let target_line = (point.y / line_height).floor().max(0.0) as usize;
    let max_width = fallback_wrap_width(&request);
    let lines = fallback_layout_lines(&request);
    let Some(line) = lines.get(target_line).or_else(|| lines.last()) else {
        return 0;
    };

    if line.chars.is_empty() {
        return request.text.semantic_text().chars().count();
    }

    let x_offset = fallback_line_x_offset(
        request.layout_style.text_align,
        request.direction,
        max_width,
        line.width,
    );
    let hit_x = point.x - x_offset;
    let mut cursor_x = 0.0;
    for ch in &line.chars {
        let midpoint = cursor_x + (ch.width / 2.0);
        if hit_x <= midpoint {
            return request.text.layout_to_semantic_index(ch.layout_index);
        }
        cursor_x += ch.width;
    }

    line.chars
        .last()
        .map(|ch| request.text.layout_to_semantic_index(ch.layout_index + 1))
        .unwrap_or_else(|| request.text.semantic_text().chars().count())
}

fn fallback_line_x_offset(
    text_align: TextAlign,
    direction: Direction,
    max_width: f32,
    line_width: f32,
) -> f32 {
    if !max_width.is_finite() {
        return 0.0;
    }
    match text_align {
        TextAlign::Start | TextAlign::Justify if direction == Direction::Rtl => {
            (max_width - line_width).max(0.0)
        }
        TextAlign::Start | TextAlign::Justify => 0.0,
        TextAlign::Center => ((max_width - line_width) / 2.0).max(0.0),
        TextAlign::End if direction == Direction::Rtl => 0.0,
        TextAlign::End => (max_width - line_width).max(0.0),
    }
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
                text_align: TextAlign::Start,
                text_overflow: TextOverflow::Clip,
                text_transform: TextTransform::None,
                tab_size: TextLayoutStyle::DEFAULT_TAB_SIZE,
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
    fn preserved_tabs_expand_with_css_default_tab_size() {
        let content = TextContent::plain("a\tb");
        let normalized =
            NormalizedText::from_content(&content, TextLayoutStyle::white_space(WhiteSpace::Pre));

        assert_eq!(normalized.layout_text(), "a        b");
        assert_eq!(normalized.layout_to_semantic_index(1), 1);
        assert_eq!(normalized.layout_to_semantic_index(8), 1);
        assert_eq!(normalized.semantic_to_layout_index(1), 1);
        assert_eq!(normalized.semantic_to_layout_index(2), 9);
    }

    #[test]
    fn text_layout_style_can_override_tab_size() {
        let content = TextContent::plain("a\tb");
        let mut style = TextLayoutStyle::white_space(WhiteSpace::Pre);
        style.tab_size = 3;
        let normalized = NormalizedText::from_content(&content, style);

        assert_eq!(normalized.layout_text(), "a   b");
        assert_eq!(normalized.layout_to_semantic_index(3), 1);
        assert_eq!(normalized.semantic_to_layout_index(2), 4);
    }

    #[test]
    fn rich_runs_derive_semantic_plain_text_and_layout_runs() {
        let content = TextContent::new(vec![
            TextRun::plain("Hello"),
            TextRun::styled(
                " world",
                InlineTextStyle {
                    font_weight: Some(FontWeight::BOLD),
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
            Some(FontWeight::BOLD)
        );
    }

    #[test]
    fn font_weight_accepts_css_like_numeric_values() {
        assert_eq!(FontWeight::NORMAL.value(), 400);
        assert_eq!(FontWeight::BOLD.value(), 700);
        assert_eq!(FontWeight::new(50).value(), 50);
        assert_eq!(FontWeight::new(0).value(), FontWeight::MIN);
        assert_eq!(FontWeight::new(1200).value(), FontWeight::MAX);
    }

    #[test]
    fn font_stretch_represents_css_percentage_widths() {
        assert_eq!(FontStretch::NORMAL.value(), 100.0);
        assert_eq!(FontStretch::CONDENSED.value(), 75.0);
        assert_eq!(FontStretch::EXPANDED.value(), 125.0);
        assert_eq!(FontStretch::percent(20.0).value(), FontStretch::MIN_PERCENT);
        assert_eq!(
            FontStretch::percent(240.0).value(),
            FontStretch::MAX_PERCENT
        );
        assert_eq!(FontStretch::percent(112.5).width_factor(), 1.125);
    }

    #[test]
    fn font_style_represents_css_font_style_keywords() {
        let content = TextContent::new(vec![
            TextRun::styled(
                "normal",
                InlineTextStyle {
                    font_style: Some(FontStyle::Normal),
                    ..InlineTextStyle::default()
                },
            ),
            TextRun::styled(
                " italic",
                InlineTextStyle {
                    font_style: Some(FontStyle::Italic),
                    ..InlineTextStyle::default()
                },
            ),
            TextRun::styled(
                " oblique",
                InlineTextStyle {
                    font_style: Some(FontStyle::Oblique),
                    ..InlineTextStyle::default()
                },
            ),
        ]);
        let normalized = NormalizedText::from_content(&content, TextLayoutStyle::default());

        assert_eq!(
            normalized.runs()[0].style.font_style,
            Some(FontStyle::Normal)
        );
        assert_eq!(
            normalized.runs()[1].style.font_style,
            Some(FontStyle::Italic)
        );
        assert_eq!(
            normalized.runs()[2].style.font_style,
            Some(FontStyle::Oblique)
        );
    }

    #[test]
    fn text_decoration_represents_css_line_combinations() {
        assert!(TextDecoration::NONE.is_none());
        assert_eq!(
            TextDecoration::lines(true, true, true),
            TextDecoration {
                underline: true,
                overline: true,
                line_through: true,
                color: None,
                thickness: None,
            }
        );
        assert!(TextDecoration::UNDERLINE.underline);
        assert!(TextDecoration::OVERLINE.overline);
        assert!(TextDecoration::LINE_THROUGH.line_through);

        let styled = TextDecoration::UNDERLINE
            .color(Color::rgb(220, 40, 80))
            .thickness(2.5);
        assert_eq!(
            styled.stroke_color(Color::rgb(0, 0, 0)),
            Color::rgb(220, 40, 80)
        );
        assert_eq!(styled.stroke_thickness(), 2.5);
        assert_eq!(
            TextDecoration::UNDERLINE.stroke_color(Color::rgb(1, 2, 3)),
            Color::rgb(1, 2, 3)
        );
        assert_eq!(
            TextDecoration::UNDERLINE.thickness(-1.0).stroke_thickness(),
            0.0
        );
    }

    #[test]
    fn text_transform_changes_layout_text_without_changing_semantic_text() {
        let content = TextContent::plain("Straße title");
        let mut style = TextLayoutStyle::default();
        style.text_transform = TextTransform::Uppercase;
        let normalized = NormalizedText::from_content(&content, style);

        assert_eq!(normalized.semantic_text(), "Straße title");
        assert_eq!(normalized.layout_text(), "STRASSE TITLE");
        assert_eq!(normalized.layout_to_semantic_index(4), 4);
        assert_eq!(normalized.layout_to_semantic_index(5), 4);
        assert_eq!(normalized.semantic_to_layout_index(4), 4);
        assert_eq!(normalized.semantic_to_layout_index(5), 6);
    }

    #[test]
    fn inline_text_transform_overrides_element_transform_per_run() {
        let content = TextContent::new(vec![
            TextRun::plain("hello "),
            TextRun::styled(
                "LOUD",
                InlineTextStyle {
                    text_transform: Some(TextTransform::Lowercase),
                    ..InlineTextStyle::default()
                },
            ),
        ]);
        let mut style = TextLayoutStyle::default();
        style.text_transform = TextTransform::Capitalize;
        let normalized = NormalizedText::from_content(&content, style);

        assert_eq!(normalized.semantic_text(), "hello LOUD");
        assert_eq!(normalized.layout_text(), "Hello loud");
        assert_eq!(normalized.runs().len(), 2);
    }

    #[test]
    fn text_transform_capitalize_tracks_word_boundaries_across_runs() {
        let content =
            TextContent::new(vec![TextRun::plain("alpha-"), TextRun::plain("beta gamma")]);
        let mut style = TextLayoutStyle::white_space(WhiteSpace::PreWrap);
        style.text_transform = TextTransform::Capitalize;
        let normalized = NormalizedText::from_content(&content, style);

        assert_eq!(normalized.layout_text(), "Alpha-Beta Gamma");
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
            direction: Direction::Ltr,
            wrap_width: 20.0,
            layout_style: style,
            line_height: None,
        });

        assert!(measured.line_count > 1);
    }

    #[test]
    fn fallback_measurement_reports_line_metadata() {
        let content = TextContent::plain("Alpha beta");
        let normalized = NormalizedText::from_content(&content, TextLayoutStyle::default());
        let mut measurer = FallbackTextMeasurer;

        let measured = measurer.measure_text(TextLayoutRequest {
            text: &normalized,
            font_size: 13.0,
            color: Color::rgb(255, 255, 255),
            direction: Direction::Ltr,
            wrap_width: 40.0,
            layout_style: TextLayoutStyle::default(),
            line_height: None,
        });

        assert_eq!(measured.lines.len(), measured.line_count);
        assert_eq!(measured.lines[0].layout_start, 0);
        assert_eq!(measured.lines[0].semantic_start, 0);
        assert!(measured.lines[0].layout_end <= normalized.layout_text().chars().count());
        assert_eq!(measured.first_baseline, Some(measured.lines[0].baseline));
        assert!(measured.lines[0].baseline > 0.0);
        assert!(measured.lines[0].baseline <= measured.lines[0].height);
    }

    #[test]
    fn fallback_wrapping_preserves_space_indices() {
        let content = TextContent::plain("a   b");
        let style = TextLayoutStyle::white_space(WhiteSpace::PreWrap);
        let normalized = NormalizedText::from_content(&content, style);
        let mut measurer = FallbackTextMeasurer;

        let measured = measurer.measure_text(TextLayoutRequest {
            text: &normalized,
            font_size: 13.0,
            color: Color::rgb(255, 255, 255),
            direction: Direction::Ltr,
            wrap_width: 25.0,
            layout_style: style,
            line_height: None,
        });

        assert_eq!(measured.line_count, 2);
        assert_eq!(measured.lines[0].layout_start, 0);
        assert_eq!(measured.lines[0].layout_end, 4);
        assert_eq!(measured.lines[0].semantic_start, 0);
        assert_eq!(measured.lines[0].semantic_end, 4);
        assert_eq!(measured.lines[1].layout_start, 4);
        assert_eq!(measured.lines[1].semantic_start, 4);
    }

    #[test]
    fn fallback_hit_testing_preserved_wrapped_spaces_uses_original_indices() {
        let content = TextContent::plain("a   b");
        let style = TextLayoutStyle::white_space(WhiteSpace::PreWrap);
        let normalized = NormalizedText::from_content(&content, style);
        let mut measurer = FallbackTextMeasurer;

        let index = measurer.text_index_at(
            TextLayoutRequest {
                text: &normalized,
                font_size: 13.0,
                color: Color::rgb(255, 255, 255),
                direction: Direction::Ltr,
                wrap_width: 25.0,
                layout_style: style,
                line_height: None,
            },
            Point::new(12.0, 0.0),
        );

        assert_eq!(index, 2);
    }

    #[test]
    fn fallback_measurement_reports_aligned_line_offsets() {
        let content = TextContent::plain("abcd");
        let mut style = TextLayoutStyle::default();
        style.text_align = TextAlign::Center;
        let normalized = NormalizedText::from_content(&content, style);
        let mut measurer = FallbackTextMeasurer;

        let measured = measurer.measure_text(TextLayoutRequest {
            text: &normalized,
            font_size: 13.0,
            color: Color::rgb(255, 255, 255),
            direction: Direction::Ltr,
            wrap_width: 60.0,
            layout_style: style,
            line_height: None,
        });

        assert_eq!(measured.lines.len(), 1);
        assert!(measured.lines[0].x_offset > 0.0);
    }

    #[test]
    fn fallback_measurement_resolves_start_and_end_against_direction() {
        let content = TextContent::plain("abcd");
        let mut style = TextLayoutStyle::white_space(WhiteSpace::Pre);
        style.text_align = TextAlign::Start;
        let normalized = NormalizedText::from_content(&content, style);
        let mut measurer = FallbackTextMeasurer;

        let rtl_start = measurer.measure_text(TextLayoutRequest {
            text: &normalized,
            font_size: 13.0,
            color: Color::rgb(255, 255, 255),
            direction: Direction::Rtl,
            wrap_width: 60.0,
            layout_style: style,
            line_height: None,
        });

        style.text_align = TextAlign::End;
        let ltr_end = measurer.measure_text(TextLayoutRequest {
            text: &normalized,
            font_size: 13.0,
            color: Color::rgb(255, 255, 255),
            direction: Direction::Ltr,
            wrap_width: 60.0,
            layout_style: style,
            line_height: None,
        });
        let rtl_end = measurer.measure_text(TextLayoutRequest {
            text: &normalized,
            font_size: 13.0,
            color: Color::rgb(255, 255, 255),
            direction: Direction::Rtl,
            wrap_width: 60.0,
            layout_style: style,
            line_height: None,
        });

        assert_eq!(rtl_start.lines[0].x_offset, ltr_end.lines[0].x_offset);
        assert_eq!(rtl_end.lines[0].x_offset, 0.0);
    }

    #[test]
    fn fallback_line_baseline_tracks_largest_inline_font_on_line() {
        let plain = TextContent::plain("AA");
        let rich = TextContent::new(vec![TextRun::styled(
            "AA",
            InlineTextStyle {
                font_size: Some(26.0),
                ..InlineTextStyle::default()
            },
        )]);
        let normalized_plain = NormalizedText::from_content(&plain, TextLayoutStyle::default());
        let normalized_rich = NormalizedText::from_content(&rich, TextLayoutStyle::default());
        let mut measurer = FallbackTextMeasurer;

        let plain = measurer.measure_text(TextLayoutRequest {
            text: &normalized_plain,
            font_size: 13.0,
            color: Color::rgb(255, 255, 255),
            direction: Direction::Ltr,
            wrap_width: f32::INFINITY,
            layout_style: TextLayoutStyle::white_space(WhiteSpace::Pre),
            line_height: None,
        });
        let rich = measurer.measure_text(TextLayoutRequest {
            text: &normalized_rich,
            font_size: 13.0,
            color: Color::rgb(255, 255, 255),
            direction: Direction::Ltr,
            wrap_width: f32::INFINITY,
            layout_style: TextLayoutStyle::white_space(WhiteSpace::Pre),
            line_height: None,
        });

        assert!(rich.lines[0].baseline > plain.lines[0].baseline);
        assert_eq!(rich.first_baseline, Some(rich.lines[0].baseline));
    }

    #[test]
    fn fallback_line_metrics_account_for_super_and_subscript_runs() {
        let normal = TextContent::plain("AA");
        let shifted = TextContent::new(vec![
            TextRun::plain("A"),
            TextRun::styled(
                "A",
                InlineTextStyle {
                    vertical_align: Some(TextVerticalAlign::Super),
                    ..InlineTextStyle::default()
                },
            ),
            TextRun::styled(
                "A",
                InlineTextStyle {
                    vertical_align: Some(TextVerticalAlign::Sub),
                    ..InlineTextStyle::default()
                },
            ),
        ]);
        let normalized_normal = NormalizedText::from_content(&normal, TextLayoutStyle::default());
        let normalized_shifted = NormalizedText::from_content(&shifted, TextLayoutStyle::default());
        let mut measurer = FallbackTextMeasurer;

        let normal = measurer.measure_text(TextLayoutRequest {
            text: &normalized_normal,
            font_size: 13.0,
            color: Color::rgb(255, 255, 255),
            direction: Direction::Ltr,
            wrap_width: f32::INFINITY,
            layout_style: TextLayoutStyle::white_space(WhiteSpace::Pre),
            line_height: None,
        });
        let shifted = measurer.measure_text(TextLayoutRequest {
            text: &normalized_shifted,
            font_size: 13.0,
            color: Color::rgb(255, 255, 255),
            direction: Direction::Ltr,
            wrap_width: f32::INFINITY,
            layout_style: TextLayoutStyle::white_space(WhiteSpace::Pre),
            line_height: None,
        });

        assert!(shifted.size.height > normal.size.height);
        assert!(shifted.lines[0].baseline > normal.lines[0].baseline);
    }

    #[test]
    fn fallback_hit_testing_accounts_for_aligned_line_offsets() {
        let content = TextContent::plain("abcd");
        let mut style = TextLayoutStyle::default();
        style.text_align = TextAlign::End;
        let normalized = NormalizedText::from_content(&content, style);
        let mut measurer = FallbackTextMeasurer;

        let index = measurer.text_index_at(
            TextLayoutRequest {
                text: &normalized,
                font_size: 13.0,
                color: Color::rgb(255, 255, 255),
                direction: Direction::Ltr,
                wrap_width: 60.0,
                layout_style: style,
                line_height: None,
            },
            Point::new(31.0, 0.0),
        );

        assert_eq!(index, 0);
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
            direction: Direction::Ltr,
            wrap_width: f32::INFINITY,
            layout_style: TextLayoutStyle::white_space(WhiteSpace::Pre),
            line_height: None,
        });
        let rich = measurer.measure_text(TextLayoutRequest {
            text: &normalized_rich,
            font_size: 13.0,
            color: Color::rgb(255, 255, 255),
            direction: Direction::Ltr,
            wrap_width: f32::INFINITY,
            layout_style: TextLayoutStyle::white_space(WhiteSpace::Pre),
            line_height: None,
        });

        assert!(rich.size.width > small.size.width);
        assert!(rich.size.height > small.size.height);
    }

    #[test]
    fn fallback_measurement_uses_inline_letter_spacing() {
        let normal = TextContent::plain("MMMM");
        let spaced = TextContent::new(vec![TextRun::styled(
            "MMMM",
            InlineTextStyle {
                letter_spacing: Some(2.0),
                ..InlineTextStyle::default()
            },
        )]);
        let normalized_normal = NormalizedText::from_content(&normal, TextLayoutStyle::default());
        let normalized_spaced = NormalizedText::from_content(&spaced, TextLayoutStyle::default());
        let mut measurer = FallbackTextMeasurer;

        let normal = measurer.measure_text(TextLayoutRequest {
            text: &normalized_normal,
            font_size: 13.0,
            color: Color::rgb(255, 255, 255),
            direction: Direction::Ltr,
            wrap_width: f32::INFINITY,
            layout_style: TextLayoutStyle::white_space(WhiteSpace::Pre),
            line_height: None,
        });
        let spaced = measurer.measure_text(TextLayoutRequest {
            text: &normalized_spaced,
            font_size: 13.0,
            color: Color::rgb(255, 255, 255),
            direction: Direction::Ltr,
            wrap_width: f32::INFINITY,
            layout_style: TextLayoutStyle::white_space(WhiteSpace::Pre),
            line_height: None,
        });

        assert!(spaced.size.width > normal.size.width);
    }

    #[test]
    fn fallback_measurement_uses_inline_font_stretch() {
        let normal = TextContent::plain("MMMM");
        let condensed = TextContent::new(vec![TextRun::styled(
            "MMMM",
            InlineTextStyle {
                font_stretch: Some(FontStretch::CONDENSED),
                ..InlineTextStyle::default()
            },
        )]);
        let expanded = TextContent::new(vec![TextRun::styled(
            "MMMM",
            InlineTextStyle {
                font_stretch: Some(FontStretch::EXPANDED),
                ..InlineTextStyle::default()
            },
        )]);
        let normalized_normal = NormalizedText::from_content(&normal, TextLayoutStyle::default());
        let normalized_condensed =
            NormalizedText::from_content(&condensed, TextLayoutStyle::default());
        let normalized_expanded =
            NormalizedText::from_content(&expanded, TextLayoutStyle::default());
        let mut measurer = FallbackTextMeasurer;

        let normal = measurer.measure_text(TextLayoutRequest {
            text: &normalized_normal,
            font_size: 13.0,
            color: Color::rgb(255, 255, 255),
            direction: Direction::Ltr,
            wrap_width: f32::INFINITY,
            layout_style: TextLayoutStyle::white_space(WhiteSpace::Pre),
            line_height: None,
        });
        let condensed = measurer.measure_text(TextLayoutRequest {
            text: &normalized_condensed,
            font_size: 13.0,
            color: Color::rgb(255, 255, 255),
            direction: Direction::Ltr,
            wrap_width: f32::INFINITY,
            layout_style: TextLayoutStyle::white_space(WhiteSpace::Pre),
            line_height: None,
        });
        let expanded = measurer.measure_text(TextLayoutRequest {
            text: &normalized_expanded,
            font_size: 13.0,
            color: Color::rgb(255, 255, 255),
            direction: Direction::Ltr,
            wrap_width: f32::INFINITY,
            layout_style: TextLayoutStyle::white_space(WhiteSpace::Pre),
            line_height: None,
        });

        assert!(condensed.size.width < normal.size.width);
        assert!(expanded.size.width > normal.size.width);
    }

    #[test]
    fn fallback_hit_testing_uses_inline_run_font_sizes() {
        let rich = TextContent::new(vec![
            TextRun::plain("AA"),
            TextRun::styled(
                "BB",
                InlineTextStyle {
                    font_size: Some(26.0),
                    ..InlineTextStyle::default()
                },
            ),
        ]);
        let normalized = NormalizedText::from_content(&rich, TextLayoutStyle::default());
        let mut measurer = FallbackTextMeasurer;

        let index = measurer.text_index_at(
            TextLayoutRequest {
                text: &normalized,
                font_size: 13.0,
                color: Color::rgb(255, 255, 255),
                direction: Direction::Ltr,
                wrap_width: f32::INFINITY,
                layout_style: TextLayoutStyle::white_space(WhiteSpace::Pre),
                line_height: None,
            },
            Point::new(24.0, 0.0),
        );

        assert_eq!(index, 3);
    }

    #[test]
    fn fallback_hit_testing_uses_inline_letter_spacing() {
        let spaced = TextContent::new(vec![TextRun::styled(
            "AB",
            InlineTextStyle {
                letter_spacing: Some(6.0),
                ..InlineTextStyle::default()
            },
        )]);
        let normalized = NormalizedText::from_content(&spaced, TextLayoutStyle::default());
        let mut measurer = FallbackTextMeasurer;

        let index = measurer.text_index_at(
            TextLayoutRequest {
                text: &normalized,
                font_size: 13.0,
                color: Color::rgb(255, 255, 255),
                direction: Direction::Ltr,
                wrap_width: f32::INFINITY,
                layout_style: TextLayoutStyle::white_space(WhiteSpace::Pre),
                line_height: None,
            },
            Point::new(8.0, 0.0),
        );

        assert_eq!(index, 1);
    }

    #[test]
    fn fallback_hit_testing_maps_collapsed_layout_indices_to_semantic_text() {
        let content = TextContent::plain("a   b");
        let normalized = NormalizedText::from_content(&content, TextLayoutStyle::default());
        let mut measurer = FallbackTextMeasurer;

        let index = measurer.text_index_at(
            TextLayoutRequest {
                text: &normalized,
                font_size: 13.0,
                color: Color::rgb(255, 255, 255),
                direction: Direction::Ltr,
                wrap_width: f32::INFINITY,
                layout_style: TextLayoutStyle::default(),
                line_height: None,
            },
            Point::new(16.0, 0.0),
        );

        assert_eq!(index, 4);
    }
}
