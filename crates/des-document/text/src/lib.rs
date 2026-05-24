use cosmic_text::{
    Align, Attrs, Buffer, CacheKey, Color as CosmicColor, Ellipsize, EllipsizeHeightLimit, Family,
    FontSystem, Metrics, PhysicalGlyph, Renderer, Shaping, Stretch, Style, SwashCache,
    SwashContent, UnderlineStyle, Weight, Wrap, render_decoration,
};
use des_document::{
    Color, Direction, FontStyle, InlineTextStyle, Point, Rect, Size, TextAlign, TextLayoutLine,
    TextLayoutRequest, TextLayoutResult, TextMeasurer, TextMeasurerKey, TextOverflow, TextWrapMode,
};
use std::{collections::HashMap, hash::Hash, sync::Arc};

pub const INTER_FAMILY: &str = "Inter";
pub const JETBRAINS_MONO_FAMILY: &str = "JetBrains Mono";
pub type TextGlyphCacheKey = CacheKey;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SystemFontLoading {
    BundledOnly,
    IncludeSystemFonts,
}

#[derive(Clone, Debug)]
pub struct FontAsset {
    pub family: &'static str,
    pub bytes: &'static [u8],
}

impl FontAsset {
    pub const fn new(family: &'static str, bytes: &'static [u8]) -> Self {
        Self { family, bytes }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextSurface {
    pub size: Size,
    pub offset: Point,
    pub pixels_per_point: f32,
    pub width_px: u32,
    pub height_px: u32,
    pub rgba: Vec<u8>,
}

impl TextSurface {
    pub fn is_empty(&self) -> bool {
        self.width_px == 0 || self.height_px == 0 || self.rgba.is_empty()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RasterizedText {
    pub surface: TextSurface,
    pub layout: TextLayoutResult,
    pub diagnostics: TextDiagnostics,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextGlyphRun {
    pub layout: TextLayoutResult,
    pub glyphs: Vec<TextGlyph>,
    pub backgrounds: Vec<TextGlyphRect>,
    pub decorations: Vec<TextGlyphRect>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TextGlyphImageContent {
    Mask,
    Color,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextGlyphImage {
    pub width_px: u32,
    pub height_px: u32,
    pub left_px: i32,
    pub top_px: i32,
    pub content: TextGlyphImageContent,
    pub rgba: Vec<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextGlyph {
    pub cache_key: TextGlyphCacheKey,
    pub x_px: i32,
    pub y_px: i32,
    pub color: Color,
    pub run_index: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextGlyphRect {
    pub x_px: i32,
    pub y_px: i32,
    pub width_px: u32,
    pub height_px: u32,
    pub color: Color,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextDiagnostics {
    pub backend: &'static str,
    pub proportional_family: &'static str,
    pub monospace_family: &'static str,
    pub pixels_per_point: f32,
    pub width_px: u32,
    pub height_px: u32,
    pub glyph_rects: usize,
}

pub struct CosmicTextRenderer {
    font_system: FontSystem,
    swash_cache: SwashCache,
    buffers: HashMap<TextBufferKey, Buffer>,
    buffer_cache_hits: usize,
    buffer_cache_misses: usize,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TextBufferStats {
    pub cache_entries: usize,
    pub cache_hits: usize,
    pub cache_misses: usize,
}

impl CosmicTextRenderer {
    pub fn new(fonts: impl IntoIterator<Item = FontAsset>) -> Self {
        Self::with_system_font_loading(fonts, SystemFontLoading::BundledOnly)
    }

    pub fn with_system_font_loading(
        fonts: impl IntoIterator<Item = FontAsset>,
        system_font_loading: SystemFontLoading,
    ) -> Self {
        let mut db = fontdb::Database::new();
        if system_font_loading == SystemFontLoading::IncludeSystemFonts {
            db.load_system_fonts();
        }
        for font in fonts {
            db.load_font_source(fontdb::Source::Binary(Arc::new(font.bytes.to_vec())));
        }
        db.set_sans_serif_family(INTER_FAMILY);
        db.set_monospace_family(JETBRAINS_MONO_FAMILY);

        let locale = std::env::var("LANG").unwrap_or_else(|_| "en-US".to_string());
        Self {
            font_system: FontSystem::new_with_locale_and_db(locale, db),
            swash_cache: SwashCache::new(),
            buffers: HashMap::new(),
            buffer_cache_hits: 0,
            buffer_cache_misses: 0,
        }
    }

    pub fn begin_frame(&mut self) {
        self.buffer_cache_hits = 0;
        self.buffer_cache_misses = 0;
    }

    pub fn buffer_stats(&self) -> TextBufferStats {
        TextBufferStats {
            cache_entries: self.buffers.len(),
            cache_hits: self.buffer_cache_hits,
            cache_misses: self.buffer_cache_misses,
        }
    }

    pub fn rasterize(
        &mut self,
        request: TextLayoutRequest<'_>,
        pixels_per_point: f32,
    ) -> RasterizedText {
        self.rasterize_visible(request, pixels_per_point, None)
    }

    pub fn rasterize_visible(
        &mut self,
        request: TextLayoutRequest<'_>,
        pixels_per_point: f32,
        visible_rect: Option<Rect>,
    ) -> RasterizedText {
        let scale = pixels_per_point.max(1.0);
        let (layout, surface) = self.with_buffer(request.clone(), scale, |buffer, swash_cache| {
            let layout = layout_result(&request, buffer);
            let surface = rasterize_surface(
                buffer,
                swash_cache,
                request.color,
                scale,
                &layout,
                visible_rect,
            );
            (layout, surface)
        });

        let diagnostics = TextDiagnostics {
            backend: "cosmic-text",
            proportional_family: INTER_FAMILY,
            monospace_family: JETBRAINS_MONO_FAMILY,
            pixels_per_point: scale,
            width_px: surface.surface.width_px,
            height_px: surface.surface.height_px,
            glyph_rects: surface.glyph_rects,
        };

        RasterizedText {
            surface: surface.surface,
            layout: TextLayoutResult {
                size: Size::new(layout.size.width / scale, layout.size.height / scale),
                ..layout.scale_lines(1.0 / scale)
            },
            diagnostics,
        }
    }

    pub fn layout(
        &mut self,
        request: TextLayoutRequest<'_>,
        pixels_per_point: f32,
    ) -> TextLayoutResult {
        let scale = pixels_per_point.max(1.0);
        let layout = self.with_buffer(request.clone(), scale, |buffer, _| {
            layout_result(&request, buffer)
        });
        TextLayoutResult {
            size: Size::new(layout.size.width / scale, layout.size.height / scale),
            ..layout.scale_lines(1.0 / scale)
        }
    }

    pub fn glyphs(
        &mut self,
        request: TextLayoutRequest<'_>,
        pixels_per_point: f32,
        visible_rect: Option<Rect>,
    ) -> TextGlyphRun {
        let scale = pixels_per_point.max(1.0);
        let (layout, glyphs, backgrounds, decorations) =
            self.with_buffer(request.clone(), scale, |buffer, _| {
                let layout = layout_result(&request, buffer);
                let mut glyphs = Vec::new();
                let mut backgrounds = Vec::new();
                let mut decorations = Vec::new();
                let run_backgrounds = run_backgrounds(request.text);
                let baseline_shifts = run_baseline_shifts(request.text, request.font_size, scale);
                for run in buffer.layout_runs() {
                    collect_run_backgrounds(
                        run.glyphs,
                        run.line_top,
                        run.line_height,
                        &run_backgrounds,
                        &baseline_shifts,
                        &mut backgrounds,
                    );
                    for glyph in run.glyphs {
                        let baseline_shift =
                            baseline_shifts.get(glyph.metadata).copied().unwrap_or(0.0);
                        let physical = glyph.physical((0.0, run.line_y - baseline_shift), 1.0);
                        let x = physical.x as f32 / scale;
                        let y = physical.y as f32 / scale;
                        let margin = glyph.font_size * 2.0;
                        if let Some(visible) = visible_rect
                            && (x > visible.right() + margin
                                || y > visible.bottom() + margin
                                || x < visible.origin.x - margin
                                || y < visible.origin.y - margin)
                        {
                            continue;
                        }
                        glyphs.push(TextGlyph {
                            cache_key: physical.cache_key,
                            x_px: physical.x,
                            y_px: physical.y,
                            color: glyph
                                .color_opt
                                .map_or(request.color, cosmic_to_document_color),
                            run_index: glyph.metadata.saturating_sub(1),
                        });
                    }
                    let mut collector = DecorationCollector {
                        rects: &mut decorations,
                    };
                    render_decoration(&mut collector, &run, cosmic_color(request.color));
                }
                (layout, glyphs, backgrounds, decorations)
            });
        TextGlyphRun {
            layout: TextLayoutResult {
                size: Size::new(layout.size.width / scale, layout.size.height / scale),
                ..layout.scale_lines(1.0 / scale)
            },
            glyphs,
            backgrounds,
            decorations,
        }
    }

    pub fn glyph_image(&mut self, cache_key: TextGlyphCacheKey) -> Option<TextGlyphImage> {
        let image = self
            .swash_cache
            .get_image(&mut self.font_system, cache_key)
            .as_ref()?;
        let content = match image.content {
            SwashContent::Mask | SwashContent::SubpixelMask => TextGlyphImageContent::Mask,
            SwashContent::Color => TextGlyphImageContent::Color,
        };
        let rgba = match image.content {
            SwashContent::Mask => image
                .data
                .iter()
                .flat_map(|alpha| [255, 255, 255, *alpha])
                .collect(),
            SwashContent::SubpixelMask | SwashContent::Color => image.data.clone(),
        };
        Some(TextGlyphImage {
            width_px: image.placement.width,
            height_px: image.placement.height,
            left_px: image.placement.left,
            top_px: image.placement.top,
            content,
            rgba,
        })
    }

    fn hit_index(&mut self, request: TextLayoutRequest<'_>, point: Point) -> usize {
        let scale = 1.0;
        let cursor = self
            .with_buffer(request.clone(), scale, |buffer, _| {
                buffer.hit(point.x * scale, point.y * scale)
            })
            .unwrap_or_default();
        let layout_index =
            line_byte_to_layout_index(request.text.layout_text(), cursor.line, cursor.index);
        request.text.layout_to_semantic_index(layout_index)
    }

    fn with_buffer<R>(
        &mut self,
        request: TextLayoutRequest<'_>,
        scale: f32,
        f: impl FnOnce(&mut cosmic_text::BorrowedWithFontSystem<'_, Buffer>, &mut SwashCache) -> R,
    ) -> R {
        let key = TextBufferKey::new(&request, scale);
        if self.buffers.len() > 512 && !self.buffers.contains_key(&key) {
            self.buffers.clear();
        }
        if self.buffers.contains_key(&key) {
            self.buffer_cache_hits += 1;
        } else {
            self.buffer_cache_misses += 1;
        }
        let font_system = &mut self.font_system;
        let buffer = self.buffers.entry(key).or_insert_with(|| {
            let metrics = buffer_metrics(&request, scale);
            let mut buffer = Buffer::new_empty(metrics);
            let mut borrowed = buffer.borrow_with(font_system);
            configure_buffer(&mut borrowed, &request, scale);
            buffer
        });
        let mut buffer = buffer.borrow_with(font_system);
        f(&mut buffer, &mut self.swash_cache)
    }
}

impl TextMeasurer for CosmicTextRenderer {
    fn cache_key(&self) -> TextMeasurerKey {
        TextMeasurerKey::new("cosmic-text")
    }

    fn measure_text(&mut self, request: TextLayoutRequest<'_>) -> TextLayoutResult {
        self.layout(request, 1.0)
    }

    fn text_index_at(&mut self, request: TextLayoutRequest<'_>, point: Point) -> usize {
        self.hit_index(request, point)
    }
}

trait ScaleTextLayoutResult {
    fn scale_lines(self, factor: f32) -> Self;
}

impl ScaleTextLayoutResult for TextLayoutResult {
    fn scale_lines(mut self, factor: f32) -> Self {
        for line in &mut self.lines {
            line.x_offset *= factor;
            line.width *= factor;
            line.height *= factor;
            line.baseline *= factor;
        }
        self
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct TextBufferKey {
    layout_text: String,
    font_size: u32,
    color: [u8; 4],
    direction: DirectionKey,
    wrap_width: u32,
    layout_style: TextLayoutStyleKey,
    line_height: Option<u32>,
    scale: u32,
    runs: Vec<TextRunKey>,
}

impl TextBufferKey {
    fn new(request: &TextLayoutRequest<'_>, scale: f32) -> Self {
        Self {
            layout_text: request.text.layout_text().to_string(),
            font_size: f32_key(request.font_size),
            color: color_key(request.color),
            direction: DirectionKey::from(request.direction),
            wrap_width: f32_key(request.wrap_width),
            layout_style: TextLayoutStyleKey::from(request.layout_style),
            line_height: request.line_height.map(f32_key),
            scale: f32_key(scale),
            runs: request
                .text
                .runs()
                .iter()
                .map(|run| TextRunKey {
                    text: run.text.clone(),
                    style: InlineStyleKey::from(&run.style),
                })
                .collect(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum DirectionKey {
    Ltr,
    Rtl,
}

impl From<Direction> for DirectionKey {
    fn from(direction: Direction) -> Self {
        match direction {
            Direction::Ltr => Self::Ltr,
            Direction::Rtl => Self::Rtl,
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct TextRunKey {
    text: String,
    style: InlineStyleKey,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct InlineStyleKey {
    color: Option<[u8; 4]>,
    font_size: Option<u32>,
    line_height: Option<u32>,
    letter_spacing: Option<u32>,
    font_family: Option<String>,
    font_weight: Option<u16>,
    font_stretch: Option<u32>,
    font_style: Option<FontStyleKey>,
    text_decoration: Option<TextDecorationKey>,
    vertical_align: Option<TextVerticalAlignKey>,
    background: Option<[u8; 4]>,
}

impl From<&InlineTextStyle> for InlineStyleKey {
    fn from(style: &InlineTextStyle) -> Self {
        Self {
            color: style.color.map(color_key),
            font_size: style.font_size.map(f32_key),
            line_height: style.line_height.map(f32_key),
            letter_spacing: style.letter_spacing.map(f32_key),
            font_family: style.font_family.clone(),
            font_weight: style.font_weight.map(|weight| weight.value()),
            font_stretch: style.font_stretch.map(|stretch| f32_key(stretch.value())),
            font_style: style.font_style.map(FontStyleKey::from),
            text_decoration: style.text_decoration.map(TextDecorationKey::from),
            vertical_align: style.vertical_align.map(TextVerticalAlignKey::from),
            background: style.background.map(color_key),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum FontStyleKey {
    Normal,
    Italic,
    Oblique,
}

impl From<FontStyle> for FontStyleKey {
    fn from(style: FontStyle) -> Self {
        match style {
            FontStyle::Normal => Self::Normal,
            FontStyle::Italic => Self::Italic,
            FontStyle::Oblique => Self::Oblique,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct TextDecorationKey {
    underline: bool,
    overline: bool,
    line_through: bool,
    color: Option<[u8; 4]>,
    thickness: Option<u32>,
}

impl From<des_document::TextDecoration> for TextDecorationKey {
    fn from(decoration: des_document::TextDecoration) -> Self {
        Self {
            underline: decoration.underline,
            overline: decoration.overline,
            line_through: decoration.line_through,
            color: decoration.color.map(color_key),
            thickness: decoration.thickness.map(f32_key),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum TextVerticalAlignKey {
    Baseline,
    Top,
    Middle,
    Bottom,
    Sub,
    Super,
}

impl From<des_document::TextVerticalAlign> for TextVerticalAlignKey {
    fn from(align: des_document::TextVerticalAlign) -> Self {
        match align {
            des_document::TextVerticalAlign::Baseline => Self::Baseline,
            des_document::TextVerticalAlign::Top => Self::Top,
            des_document::TextVerticalAlign::Middle => Self::Middle,
            des_document::TextVerticalAlign::Bottom => Self::Bottom,
            des_document::TextVerticalAlign::Sub => Self::Sub,
            des_document::TextVerticalAlign::Super => Self::Super,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct TextLayoutStyleKey {
    white_space_collapse: WhiteSpaceCollapseKey,
    text_wrap_mode: TextWrapModeKey,
    overflow_wrap: OverflowWrapKey,
    word_break: WordBreakKey,
    text_align: TextAlignKey,
    text_overflow: TextOverflowKey,
    text_transform: TextTransformKey,
    tab_size: u16,
    max_lines: Option<usize>,
}

impl From<des_document::TextLayoutStyle> for TextLayoutStyleKey {
    fn from(style: des_document::TextLayoutStyle) -> Self {
        Self {
            white_space_collapse: WhiteSpaceCollapseKey::from(style.white_space_collapse),
            text_wrap_mode: TextWrapModeKey::from(style.text_wrap_mode),
            overflow_wrap: OverflowWrapKey::from(style.overflow_wrap),
            word_break: WordBreakKey::from(style.word_break),
            text_align: TextAlignKey::from(style.text_align),
            text_overflow: TextOverflowKey::from(style.text_overflow),
            text_transform: TextTransformKey::from(style.text_transform),
            tab_size: style.tab_size,
            max_lines: style.max_lines,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum WhiteSpaceCollapseKey {
    Collapse,
    Preserve,
    PreserveBreaks,
    BreakSpaces,
}

impl From<des_document::WhiteSpaceCollapse> for WhiteSpaceCollapseKey {
    fn from(value: des_document::WhiteSpaceCollapse) -> Self {
        match value {
            des_document::WhiteSpaceCollapse::Collapse => Self::Collapse,
            des_document::WhiteSpaceCollapse::Preserve => Self::Preserve,
            des_document::WhiteSpaceCollapse::PreserveBreaks => Self::PreserveBreaks,
            des_document::WhiteSpaceCollapse::BreakSpaces => Self::BreakSpaces,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum TextWrapModeKey {
    Wrap,
    NoWrap,
}

impl From<TextWrapMode> for TextWrapModeKey {
    fn from(value: TextWrapMode) -> Self {
        match value {
            TextWrapMode::Wrap => Self::Wrap,
            TextWrapMode::NoWrap => Self::NoWrap,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum OverflowWrapKey {
    Normal,
    Anywhere,
    BreakWord,
}

impl From<des_document::OverflowWrap> for OverflowWrapKey {
    fn from(value: des_document::OverflowWrap) -> Self {
        match value {
            des_document::OverflowWrap::Normal => Self::Normal,
            des_document::OverflowWrap::Anywhere => Self::Anywhere,
            des_document::OverflowWrap::BreakWord => Self::BreakWord,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum WordBreakKey {
    Normal,
    BreakAll,
    KeepAll,
}

impl From<des_document::WordBreak> for WordBreakKey {
    fn from(value: des_document::WordBreak) -> Self {
        match value {
            des_document::WordBreak::Normal => Self::Normal,
            des_document::WordBreak::BreakAll => Self::BreakAll,
            des_document::WordBreak::KeepAll => Self::KeepAll,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum TextAlignKey {
    Start,
    Center,
    End,
    Justify,
}

impl From<TextAlign> for TextAlignKey {
    fn from(value: TextAlign) -> Self {
        match value {
            TextAlign::Start => Self::Start,
            TextAlign::Center => Self::Center,
            TextAlign::End => Self::End,
            TextAlign::Justify => Self::Justify,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum TextOverflowKey {
    Clip,
    Ellipsis,
}

impl From<TextOverflow> for TextOverflowKey {
    fn from(value: TextOverflow) -> Self {
        match value {
            TextOverflow::Clip => Self::Clip,
            TextOverflow::Ellipsis => Self::Ellipsis,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum TextTransformKey {
    None,
    Uppercase,
    Lowercase,
    Capitalize,
}

impl From<des_document::TextTransform> for TextTransformKey {
    fn from(value: des_document::TextTransform) -> Self {
        match value {
            des_document::TextTransform::None => Self::None,
            des_document::TextTransform::Uppercase => Self::Uppercase,
            des_document::TextTransform::Lowercase => Self::Lowercase,
            des_document::TextTransform::Capitalize => Self::Capitalize,
        }
    }
}

fn f32_key(value: f32) -> u32 {
    if value == 0.0 {
        0.0_f32.to_bits()
    } else {
        value.to_bits()
    }
}

fn color_key(color: Color) -> [u8; 4] {
    [color.r, color.g, color.b, color.a]
}

fn buffer_metrics(request: &TextLayoutRequest<'_>, scale: f32) -> Metrics {
    Metrics::new(
        request.font_size.max(1.0) * scale,
        request
            .line_height
            .unwrap_or(request.font_size * 1.2)
            .max(1.0)
            * scale,
    )
}

fn configure_buffer(
    buffer: &mut cosmic_text::BorrowedWithFontSystem<'_, Buffer>,
    request: &TextLayoutRequest<'_>,
    scale: f32,
) {
    let metrics = buffer_metrics(request, scale);
    buffer.set_metrics(metrics);
    buffer.set_wrap(cosmic_wrap(request));
    buffer.set_size(wrap_width(request, scale), height_limit(request, metrics));
    if request.layout_style.text_overflow == TextOverflow::Ellipsis {
        let height = height_limit(request, metrics);
        let limit = request
            .layout_style
            .max_lines
            .map(|lines| EllipsizeHeightLimit::Lines(lines.max(1)))
            .or(height.map(EllipsizeHeightLimit::Height))
            .unwrap_or(EllipsizeHeightLimit::Lines(1));
        buffer.set_ellipsize(Ellipsize::End(limit));
    }

    let default_attrs = cosmic_attrs(
        &InlineTextStyle::default(),
        request.font_size,
        request.color,
        request.line_height,
        scale,
        0,
    );
    let spans = request.text.runs().iter().enumerate().map(|(index, run)| {
        (
            run.text.as_str(),
            cosmic_attrs(
                &run.style,
                request.font_size,
                request.color,
                request.line_height,
                scale,
                index + 1,
            ),
        )
    });
    buffer.set_rich_text(
        spans,
        &default_attrs,
        shaping_for(request),
        Some(cosmic_align(
            request.layout_style.text_align,
            request.direction,
        )),
    );
}

fn wrap_width(request: &TextLayoutRequest<'_>, scale: f32) -> Option<f32> {
    match request.layout_style.text_wrap_mode {
        TextWrapMode::NoWrap => None,
        TextWrapMode::Wrap if request.wrap_width.is_finite() && request.wrap_width > 1.0 => {
            Some((request.wrap_width * scale).max(1.0))
        }
        TextWrapMode::Wrap => None,
    }
}

fn height_limit(request: &TextLayoutRequest<'_>, metrics: Metrics) -> Option<f32> {
    request
        .layout_style
        .max_lines
        .map(|lines| lines.max(1) as f32 * metrics.line_height)
}

fn layout_result(
    request: &TextLayoutRequest<'_>,
    buffer: &mut cosmic_text::BorrowedWithFontSystem<'_, Buffer>,
) -> TextLayoutResult {
    let mut lines = Vec::new();
    let mut max_width: f32 = 0.0;
    let mut max_height: f32 = 0.0;
    let mut line_count = 0usize;
    for run in buffer.layout_runs() {
        line_count += 1;
        max_width = max_width.max(run.line_w);
        max_height = max_height.max(run.line_top + run.line_height);
        let layout_start = run.glyphs.first().map(|glyph| glyph.start).unwrap_or(0);
        let layout_end = run
            .glyphs
            .last()
            .map(|glyph| glyph.end)
            .unwrap_or(layout_start);
        lines.push(TextLayoutLine {
            layout_start,
            layout_end,
            semantic_start: request.text.layout_to_semantic_index(layout_start),
            semantic_end: request.text.layout_to_semantic_index(layout_end),
            x_offset: first_glyph_x(run.glyphs),
            width: run.line_w,
            height: run.line_height,
            baseline: run.line_y,
        });
    }

    TextLayoutResult {
        size: Size::new(max_width, max_height),
        line_count,
        elided: layout_elided(request, &lines),
        first_baseline: lines.first().map(|line| line.baseline),
        lines,
    }
}

fn layout_elided(request: &TextLayoutRequest<'_>, lines: &[TextLayoutLine]) -> bool {
    if request.layout_style.text_overflow != TextOverflow::Ellipsis {
        return false;
    }
    let Some(last) = lines.last() else {
        return false;
    };
    let all_layout_chars = request.text.layout_text().chars().count();
    request
        .layout_style
        .max_lines
        .is_some_and(|max_lines| lines.len() >= max_lines && last.layout_end < all_layout_chars)
}

fn first_glyph_x(glyphs: &[cosmic_text::LayoutGlyph]) -> f32 {
    glyphs.first().map(|glyph| glyph.x).unwrap_or(0.0)
}

fn line_byte_to_layout_index(text: &str, line_index: usize, line_byte_index: usize) -> usize {
    let mut byte_start = 0usize;
    for (index, line) in text.split_inclusive('\n').enumerate() {
        if index == line_index {
            let local_byte = line_byte_index.min(line.len());
            return text[..byte_start + local_byte].chars().count();
        }
        byte_start += line.len();
    }
    text.chars().count()
}

fn cosmic_wrap(request: &TextLayoutRequest<'_>) -> Wrap {
    match request.layout_style.text_wrap_mode {
        TextWrapMode::NoWrap => Wrap::None,
        TextWrapMode::Wrap => {
            if matches!(
                request.layout_style.overflow_wrap,
                des_document::OverflowWrap::Anywhere
            ) || matches!(
                request.layout_style.word_break,
                des_document::WordBreak::BreakAll
            ) {
                Wrap::Glyph
            } else {
                Wrap::Word
            }
        }
    }
}

fn cosmic_align(text_align: TextAlign, direction: Direction) -> Align {
    match text_align {
        TextAlign::Start if direction == Direction::Rtl => Align::Right,
        TextAlign::Start => Align::Left,
        TextAlign::Center => Align::Center,
        TextAlign::End if direction == Direction::Rtl => Align::Left,
        TextAlign::End => Align::Right,
        TextAlign::Justify => Align::Justified,
    }
}

fn shaping_for(request: &TextLayoutRequest<'_>) -> Shaping {
    if request.direction == Direction::Ltr
        && request
            .text
            .layout_text()
            .chars()
            .all(|character| character.is_ascii())
    {
        Shaping::Basic
    } else {
        Shaping::Advanced
    }
}

fn cosmic_attrs(
    style: &InlineTextStyle,
    inherited_font_size: f32,
    inherited_color: Color,
    inherited_line_height: Option<f32>,
    scale: f32,
    metadata: usize,
) -> Attrs<'static> {
    let color = style.color.unwrap_or(inherited_color);
    let font_size = style.font_size.unwrap_or(inherited_font_size).max(1.0) * scale;
    let line_height = style
        .line_height
        .or(inherited_line_height)
        .unwrap_or(font_size / scale * 1.2)
        .max(1.0)
        * scale;
    let family = match style.font_family.as_deref() {
        Some("monospace") | Some("mono") | Some(JETBRAINS_MONO_FAMILY) => {
            Family::Name(JETBRAINS_MONO_FAMILY)
        }
        Some(INTER_FAMILY) | None => Family::Name(INTER_FAMILY),
        Some(_) => Family::SansSerif,
    };
    let mut attrs = Attrs::new()
        .family(family)
        .metrics(Metrics::new(font_size, line_height))
        .metadata(metadata)
        .color(cosmic_color(color));
    if let Some(weight) = style.font_weight {
        attrs = attrs.weight(Weight(weight.value()));
    }
    if let Some(stretch) = style.font_stretch {
        attrs = attrs.stretch(cosmic_stretch(stretch.value()));
    }
    if matches!(
        style.font_style,
        Some(FontStyle::Italic | FontStyle::Oblique)
    ) {
        attrs = attrs.style(Style::Italic);
    }
    if let Some(letter_spacing) = style.letter_spacing {
        attrs = attrs.letter_spacing((letter_spacing / font_size).max(0.0));
    }
    if let Some(decoration) = style.text_decoration {
        let decoration_color = cosmic_color(decoration.stroke_color(color));
        if decoration.underline {
            attrs = attrs
                .underline(UnderlineStyle::Single)
                .underline_color(decoration_color);
        }
        if decoration.overline {
            attrs = attrs.overline().overline_color(decoration_color);
        }
        if decoration.line_through {
            attrs = attrs.strikethrough().strikethrough_color(decoration_color);
        }
    }
    attrs
}

fn cosmic_stretch(percent: f32) -> Stretch {
    let percent = percent.clamp(
        des_document::FontStretch::MIN_PERCENT,
        des_document::FontStretch::MAX_PERCENT,
    );
    if percent <= 56.25 {
        Stretch::UltraCondensed
    } else if percent <= 68.75 {
        Stretch::ExtraCondensed
    } else if percent <= 81.25 {
        Stretch::Condensed
    } else if percent <= 93.75 {
        Stretch::SemiCondensed
    } else if percent < 106.25 {
        Stretch::Normal
    } else if percent < 118.75 {
        Stretch::SemiExpanded
    } else if percent < 137.5 {
        Stretch::Expanded
    } else if percent < 175.0 {
        Stretch::ExtraExpanded
    } else {
        Stretch::UltraExpanded
    }
}

fn run_backgrounds(text: &des_document::NormalizedText) -> Vec<Option<Color>> {
    let mut backgrounds = Vec::with_capacity(text.runs().len() + 1);
    backgrounds.push(None);
    backgrounds.extend(text.runs().iter().map(|run| run.style.background));
    backgrounds
}

fn run_baseline_shifts(
    text: &des_document::NormalizedText,
    inherited_font_size: f32,
    scale: f32,
) -> Vec<f32> {
    let mut shifts = Vec::with_capacity(text.runs().len() + 1);
    shifts.push(0.0);
    shifts.extend(text.runs().iter().map(|run| {
        let font_size = run.style.font_size.unwrap_or(inherited_font_size).max(1.0) * scale;
        match run
            .style
            .vertical_align
            .unwrap_or(des_document::TextVerticalAlign::Baseline)
        {
            des_document::TextVerticalAlign::Super => font_size * 0.35,
            des_document::TextVerticalAlign::Sub => -(font_size * 0.2),
            des_document::TextVerticalAlign::Baseline
            | des_document::TextVerticalAlign::Top
            | des_document::TextVerticalAlign::Middle
            | des_document::TextVerticalAlign::Bottom => 0.0,
        }
    }));
    shifts
}

fn collect_run_backgrounds(
    glyphs: &[cosmic_text::LayoutGlyph],
    line_top: f32,
    line_height: f32,
    backgrounds: &[Option<Color>],
    baseline_shifts: &[f32],
    output: &mut Vec<TextGlyphRect>,
) {
    let mut active: Option<(usize, f32, f32, f32, Color)> = None;
    for glyph in glyphs {
        let color = backgrounds
            .get(glyph.metadata)
            .and_then(|background| *background);
        let baseline_shift = baseline_shifts.get(glyph.metadata).copied().unwrap_or(0.0);
        match (active, color) {
            (Some((metadata, min_x, max_x, active_shift, active_color)), Some(color))
                if metadata == glyph.metadata
                    && active_shift == baseline_shift
                    && active_color == color =>
            {
                active = Some((
                    metadata,
                    min_x.min(glyph.x),
                    max_x.max(glyph.x + glyph.w),
                    active_shift,
                    color,
                ));
            }
            (Some((_, min_x, max_x, active_shift, active_color)), next_color) => {
                push_text_rect(
                    output,
                    min_x,
                    line_top - active_shift,
                    max_x - min_x,
                    line_height,
                    active_color,
                );
                active = next_color.map(|color| {
                    (
                        glyph.metadata,
                        glyph.x,
                        glyph.x + glyph.w,
                        baseline_shift,
                        color,
                    )
                });
            }
            (None, Some(color)) => {
                active = Some((
                    glyph.metadata,
                    glyph.x,
                    glyph.x + glyph.w,
                    baseline_shift,
                    color,
                ));
            }
            (None, None) => {}
        }
    }
    if let Some((_, min_x, max_x, baseline_shift, color)) = active {
        push_text_rect(
            output,
            min_x,
            line_top - baseline_shift,
            max_x - min_x,
            line_height,
            color,
        );
    }
}

fn push_text_rect(
    output: &mut Vec<TextGlyphRect>,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    color: Color,
) {
    let x_px = x.floor() as i32;
    let y_px = y.floor() as i32;
    let right_px = (x + width).ceil() as i32;
    let bottom_px = (y + height).ceil() as i32;
    let width_px = (right_px - x_px).max(0) as u32;
    let height_px = (bottom_px - y_px).max(0) as u32;
    if width_px > 0 && height_px > 0 {
        output.push(TextGlyphRect {
            x_px,
            y_px,
            width_px,
            height_px,
            color,
        });
    }
}

struct DecorationCollector<'a> {
    rects: &'a mut Vec<TextGlyphRect>,
}

impl Renderer for DecorationCollector<'_> {
    fn rectangle(&mut self, x: i32, y: i32, w: u32, h: u32, color: CosmicColor) {
        if w > 0 && h > 0 {
            self.rects.push(TextGlyphRect {
                x_px: x,
                y_px: y,
                width_px: w,
                height_px: h,
                color: cosmic_to_document_color(color),
            });
        }
    }

    fn glyph(&mut self, _physical_glyph: PhysicalGlyph, _color: CosmicColor) {}
}

fn cosmic_color(color: Color) -> CosmicColor {
    CosmicColor::rgba(color.r, color.g, color.b, color.a)
}

fn cosmic_to_document_color(color: CosmicColor) -> Color {
    let (r, g, b, a) = color.as_rgba_tuple();
    Color::rgba(r, g, b, a)
}

struct RasterizedSurface {
    surface: TextSurface,
    glyph_rects: usize,
}

fn rasterize_surface(
    buffer: &mut cosmic_text::BorrowedWithFontSystem<'_, Buffer>,
    swash_cache: &mut SwashCache,
    color: Color,
    scale: f32,
    layout: &TextLayoutResult,
    visible_rect: Option<Rect>,
) -> RasterizedSurface {
    let layout_min_x = layout
        .lines
        .iter()
        .map(|line| line.x_offset)
        .fold(f32::INFINITY, f32::min)
        .min(0.0)
        .floor() as i32;
    let layout_max_x = layout
        .lines
        .iter()
        .map(|line| line.x_offset + line.width)
        .fold(0.0_f32, f32::max)
        .ceil() as i32;
    let layout_min_y = 0_i32;
    let layout_max_y = layout.size.height.ceil() as i32;
    let (clip_min_x, clip_min_y, clip_max_x, clip_max_y) = visible_rect
        .map(|rect| {
            (
                (rect.origin.x * scale).floor() as i32,
                (rect.origin.y * scale).floor() as i32,
                ((rect.origin.x + rect.size.width) * scale).ceil() as i32,
                ((rect.origin.y + rect.size.height) * scale).ceil() as i32,
            )
        })
        .unwrap_or((layout_min_x, layout_min_y, layout_max_x, layout_max_y));
    let min_x = layout_min_x.max(clip_min_x);
    let min_y = layout_min_y.max(clip_min_y);
    let max_x = layout_max_x.min(clip_max_x);
    let max_y = layout_max_y.min(clip_max_y);
    let width_px = (max_x - min_x).clamp(1, 16_384) as u32;
    let height_px = (max_y - min_y).clamp(1, 16_384) as u32;
    if layout.lines.is_empty() || min_x >= max_x || min_y >= max_y {
        return RasterizedSurface {
            surface: TextSurface {
                size: Size::default(),
                offset: Point::ZERO,
                pixels_per_point: scale,
                width_px: 0,
                height_px: 0,
                rgba: Vec::new(),
            },
            glyph_rects: 0,
        };
    }

    let mut rgba = vec![0; width_px.saturating_mul(height_px).saturating_mul(4) as usize];
    let mut glyph_rects = 0usize;
    buffer.draw(swash_cache, cosmic_color(color), |x, y, w, h, color| {
        glyph_rects += 1;
        blend_rect(
            &mut rgba,
            width_px,
            height_px,
            x - min_x,
            y - min_y,
            w,
            h,
            color,
        );
    });

    RasterizedSurface {
        surface: TextSurface {
            size: Size::new(width_px as f32 / scale, height_px as f32 / scale),
            offset: Point::new(min_x as f32 / scale, min_y as f32 / scale),
            pixels_per_point: scale,
            width_px,
            height_px,
            rgba,
        },
        glyph_rects,
    }
}

fn blend_rect(
    rgba: &mut [u8],
    width_px: u32,
    height_px: u32,
    x: i32,
    y: i32,
    w: u32,
    h: u32,
    color: CosmicColor,
) {
    let (r, g, b, a) = color.as_rgba_tuple();
    let min_x = x.max(0) as u32;
    let min_y = y.max(0) as u32;
    let max_x = (x + w as i32).clamp(0, width_px as i32) as u32;
    let max_y = (y + h as i32).clamp(0, height_px as i32) as u32;
    for py in min_y..max_y {
        for px in min_x..max_x {
            let index = ((py * width_px + px) * 4) as usize;
            alpha_blend(&mut rgba[index..index + 4], [r, g, b, a]);
        }
    }
}

fn alpha_blend(dst: &mut [u8], src: [u8; 4]) {
    if src[3] == 0 {
        return;
    }
    if dst[3] == 0 || src[3] == 255 {
        dst.copy_from_slice(&src);
        return;
    }
    let src_a = src[3] as f32 / 255.0;
    let dst_a = dst[3] as f32 / 255.0;
    let out_a = src_a + dst_a * (1.0 - src_a);
    if out_a <= f32::EPSILON {
        dst.copy_from_slice(&[0, 0, 0, 0]);
        return;
    }
    for channel in 0..3 {
        let src_c = src[channel] as f32 / 255.0;
        let dst_c = dst[channel] as f32 / 255.0;
        dst[channel] = (((src_c * src_a + dst_c * dst_a * (1.0 - src_a)) / out_a) * 255.0)
            .round()
            .clamp(0.0, 255.0) as u8;
    }
    dst[3] = (out_a * 255.0).round().clamp(0.0, 255.0) as u8;
}

#[cfg(test)]
mod tests {
    use super::*;
    use des_document::{NormalizedText, TextContent, TextDecoration, TextLayoutStyle, TextRun};

    const INTER: &[u8] = include_bytes!("../../egui/assets/fonts/inter/InterVariable.ttf");
    const JETBRAINS_MONO: &[u8] =
        include_bytes!("../../egui/assets/fonts/jetbrains-mono/JetBrainsMono[wght].ttf");
    const JETBRAINS_MONO_ITALIC: &[u8] =
        include_bytes!("../../egui/assets/fonts/jetbrains-mono/JetBrainsMono-Italic[wght].ttf");

    fn renderer() -> CosmicTextRenderer {
        CosmicTextRenderer::new([
            FontAsset::new(INTER_FAMILY, INTER),
            FontAsset::new(JETBRAINS_MONO_FAMILY, JETBRAINS_MONO),
            FontAsset::new(JETBRAINS_MONO_FAMILY, JETBRAINS_MONO_ITALIC),
        ])
    }

    fn request<'a>(
        normalized: &'a NormalizedText,
        font_size: f32,
        wrap_width: f32,
    ) -> TextLayoutRequest<'a> {
        TextLayoutRequest {
            text: normalized,
            font_size,
            color: Color::rgb(24, 24, 30),
            direction: Direction::Ltr,
            wrap_width,
            layout_style: TextLayoutStyle::default(),
            line_height: Some(font_size * 1.25),
        }
    }

    #[test]
    fn rasterizes_basic_text_surface() {
        let mut renderer = renderer();
        let content = TextContent::plain("Ag 100px");
        let normalized = NormalizedText::from_content(&content, TextLayoutStyle::default());
        let rasterized = renderer.rasterize(request(&normalized, 32.0, 300.0), 2.0);

        assert!(rasterized.surface.width_px > 0);
        assert!(rasterized.surface.height_px > 0);
        assert!(rasterized.diagnostics.glyph_rects > 0);
        assert!(rasterized.surface.rgba.iter().any(|channel| *channel != 0));
    }

    #[test]
    fn measures_layout_without_rasterizing_surface() {
        let mut renderer = renderer();
        let content = TextContent::plain("Alpha beta gamma delta");
        let normalized = NormalizedText::from_content(&content, TextLayoutStyle::default());
        let measured = renderer.measure_text(request(&normalized, 16.0, 90.0));

        assert!(measured.size.width > 0.0);
        assert!(measured.line_count >= 1);
    }

    #[test]
    fn reuses_retained_buffers_for_matching_layout_requests() {
        let mut renderer = renderer();
        let content = TextContent::plain("Alpha beta gamma delta");
        let normalized = NormalizedText::from_content(&content, TextLayoutStyle::default());
        let text_request = request(&normalized, 16.0, 90.0);

        renderer.begin_frame();
        let first = renderer.measure_text(text_request.clone());
        let second = renderer.measure_text(text_request);
        let stats = renderer.buffer_stats();

        assert_eq!(first, second);
        assert_eq!(stats.cache_entries, 1);
        assert_eq!(stats.cache_misses, 1);
        assert_eq!(stats.cache_hits, 1);
    }

    #[test]
    fn hit_testing_returns_interior_text_indices() {
        let mut renderer = renderer();
        let content = TextContent::plain("Alpha beta");
        let normalized = NormalizedText::from_content(&content, TextLayoutStyle::default());
        let text_request = request(&normalized, 24.0, 400.0);
        let measured = renderer.measure_text(text_request.clone());
        let baseline = measured.first_baseline.unwrap_or(20.0);

        let start = renderer.text_index_at(text_request.clone(), Point::new(1.0, baseline));
        let middle = renderer.text_index_at(text_request.clone(), Point::new(72.0, baseline));
        let end = renderer.text_index_at(text_request, Point::new(220.0, baseline));

        assert_eq!(start, 0);
        assert!(
            middle > start && middle < content.semantic_text().chars().count(),
            "expected hit testing to produce an interior semantic index, got {middle}"
        );
        assert_eq!(end, content.semantic_text().chars().count());
    }

    #[test]
    fn exposes_cacheable_glyph_images_for_atlas_rendering() {
        let mut renderer = renderer();
        let content = TextContent::plain("Atlas");
        let normalized = NormalizedText::from_content(&content, TextLayoutStyle::default());
        let glyph_run = renderer.glyphs(request(&normalized, 24.0, 400.0), 2.0, None);

        assert!(!glyph_run.glyphs.is_empty());
        let image = renderer
            .glyph_image(glyph_run.glyphs[0].cache_key)
            .expect("glyph cache key should resolve to a swash image");
        assert!(image.width_px > 0);
        assert!(image.height_px > 0);
        assert_eq!(
            image.rgba.len(),
            image.width_px as usize * image.height_px as usize * 4
        );
    }

    #[test]
    fn exposes_inline_backgrounds_and_decorations_for_atlas_rendering() {
        let mut renderer = renderer();
        let highlight = Color::rgba(234, 221, 255, 180);
        let underline = Color::rgb(103, 80, 164);
        let content = TextContent::new(vec![
            TextRun::styled(
                "under ",
                InlineTextStyle {
                    text_decoration: Some(TextDecoration::UNDERLINE.color(underline)),
                    ..InlineTextStyle::default()
                },
            ),
            TextRun::styled(
                "marked",
                InlineTextStyle {
                    background: Some(highlight),
                    ..InlineTextStyle::default()
                },
            ),
        ]);
        let normalized = NormalizedText::from_content(&content, TextLayoutStyle::default());
        let glyph_run = renderer.glyphs(request(&normalized, 24.0, 400.0), 2.0, None);

        assert!(
            glyph_run
                .backgrounds
                .iter()
                .any(|rect| rect.color == highlight && rect.width_px > 0 && rect.height_px > 0),
            "inline background runs should become paintable rectangles"
        );
        assert!(
            glyph_run
                .decorations
                .iter()
                .any(|rect| rect.color == underline && rect.width_px > 0 && rect.height_px > 0),
            "underline runs should become paintable decoration rectangles"
        );
    }

    #[test]
    fn maps_font_stretch_to_cosmic_width_classes() {
        assert_eq!(cosmic_stretch(50.0), Stretch::UltraCondensed);
        assert_eq!(cosmic_stretch(62.5), Stretch::ExtraCondensed);
        assert_eq!(cosmic_stretch(75.0), Stretch::Condensed);
        assert_eq!(cosmic_stretch(87.5), Stretch::SemiCondensed);
        assert_eq!(cosmic_stretch(100.0), Stretch::Normal);
        assert_eq!(cosmic_stretch(112.5), Stretch::SemiExpanded);
        assert_eq!(cosmic_stretch(125.0), Stretch::Expanded);
        assert_eq!(cosmic_stretch(150.0), Stretch::ExtraExpanded);
        assert_eq!(cosmic_stretch(200.0), Stretch::UltraExpanded);
    }

    #[test]
    fn applies_font_stretch_to_cosmic_attrs() {
        let attrs = cosmic_attrs(
            &InlineTextStyle {
                font_stretch: Some(des_document::FontStretch::EXPANDED),
                ..InlineTextStyle::default()
            },
            16.0,
            Color::rgb(1, 2, 3),
            None,
            1.0,
            0,
        );

        assert_eq!(attrs.stretch, Stretch::Expanded);
    }

    #[test]
    fn positions_subscript_and_superscript_runs_around_the_baseline() {
        let mut renderer = renderer();
        let content = TextContent::new(vec![
            TextRun::plain("H"),
            TextRun::styled(
                "2",
                InlineTextStyle {
                    vertical_align: Some(des_document::TextVerticalAlign::Super),
                    font_size: Some(12.0),
                    ..InlineTextStyle::default()
                },
            ),
            TextRun::plain("O CO"),
            TextRun::styled(
                "2",
                InlineTextStyle {
                    vertical_align: Some(des_document::TextVerticalAlign::Sub),
                    font_size: Some(12.0),
                    ..InlineTextStyle::default()
                },
            ),
        ]);
        let normalized = NormalizedText::from_content(&content, TextLayoutStyle::default());
        let glyph_run = renderer.glyphs(request(&normalized, 20.0, 400.0), 2.0, None);
        let super_two = glyph_run
            .glyphs
            .iter()
            .find(|glyph| glyph.run_index == 1)
            .expect("superscript run should paint one glyph");
        let sub_two = glyph_run
            .glyphs
            .iter()
            .find(|glyph| glyph.run_index == 3)
            .expect("subscript run should paint one glyph");

        assert!(
            super_two.y_px < sub_two.y_px,
            "superscript glyph should paint above subscript glyph"
        );
        assert!(
            sub_two.y_px - super_two.y_px >= 8,
            "baseline shift should be visible in physical glyph placement"
        );
    }
}
