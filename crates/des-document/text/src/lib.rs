use cosmic_text::{
    Align, Attrs, Buffer, CacheKey, Color as CosmicColor, Ellipsize, EllipsizeHeightLimit, Family,
    FontSystem, Metrics, PhysicalGlyph, Renderer, Shaping, Stretch, Style, SwashCache,
    SwashContent, UnderlineStyle, Weight, Wrap, render_decoration,
};
use des_document::{
    Color, Direction, FontStyle, InlineTextStyle, Point, Rect, Size, TextAlign, TextLayoutLine,
    TextLayoutRequest, TextLayoutResult, TextMeasurer, TextMeasurerKey, TextOverflow, TextWrapMode,
};
use std::{
    collections::{HashMap, HashSet, hash_map::DefaultHasher},
    hash::{Hash, Hasher},
    ops::Range,
    sync::Arc,
};

pub const INTER_FAMILY: &str = "Inter";
pub const JETBRAINS_MONO_FAMILY: &str = "JetBrains Mono";
pub type TextGlyphCacheKey = CacheKey;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SystemFontLoading {
    BundledOnly,
    IncludeSystemFonts,
}

impl SystemFontLoading {
    pub const fn label(self) -> &'static str {
        match self {
            Self::BundledOnly => "bundled-only",
            Self::IncludeSystemFonts => "system-fallbacks",
        }
    }
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

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextPaintGlyphRun {
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
    pub layout_start: usize,
    pub layout_end: usize,
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
    pub font_loading: &'static str,
    pub pixels_per_point: f32,
    pub width_px: u32,
    pub height_px: u32,
    pub glyph_rects: usize,
}

pub struct CosmicTextRenderer {
    font_system: FontSystem,
    swash_cache: SwashCache,
    system_font_loading: SystemFontLoading,
    font_families: FontFamilyResolver,
    buffers: HashMap<TextBufferKey, Buffer>,
    paint_runs: HashMap<TextPaintGlyphRunKey, TextPaintGlyphRun>,
    buffer_cache_hits: usize,
    buffer_cache_misses: usize,
    paint_run_cache_hits: usize,
    paint_run_cache_misses: usize,
}

const MIN_TEXT_PAINT_CACHE_TILE: f32 = 256.0;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TextBufferStats {
    pub cache_entries: usize,
    pub cache_hits: usize,
    pub cache_misses: usize,
    pub paint_run_cache_entries: usize,
    pub paint_run_cache_hits: usize,
    pub paint_run_cache_misses: usize,
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
        let font_families = FontFamilyResolver::from_db(&db);

        let locale = std::env::var("LANG").unwrap_or_else(|_| "en-US".to_string());
        Self {
            font_system: FontSystem::new_with_locale_and_db(locale, db),
            swash_cache: SwashCache::new(),
            system_font_loading,
            font_families,
            buffers: HashMap::new(),
            paint_runs: HashMap::new(),
            buffer_cache_hits: 0,
            buffer_cache_misses: 0,
            paint_run_cache_hits: 0,
            paint_run_cache_misses: 0,
        }
    }

    pub fn begin_frame(&mut self) {
        self.buffer_cache_hits = 0;
        self.buffer_cache_misses = 0;
        self.paint_run_cache_hits = 0;
        self.paint_run_cache_misses = 0;
    }

    pub fn buffer_stats(&self) -> TextBufferStats {
        TextBufferStats {
            cache_entries: self.buffers.len(),
            cache_hits: self.buffer_cache_hits,
            cache_misses: self.buffer_cache_misses,
            paint_run_cache_entries: self.paint_runs.len(),
            paint_run_cache_hits: self.paint_run_cache_hits,
            paint_run_cache_misses: self.paint_run_cache_misses,
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
            let layout = layout_result(&request, buffer, scale);
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
            font_loading: self.system_font_loading.label(),
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
            layout_result(&request, buffer, scale)
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
        let (layout, paint_run) = self.with_buffer(request.clone(), scale, |buffer, _| {
            let layout = layout_result(&request, buffer, scale);
            let paint_run = collect_text_paint_glyph_run(&request, buffer, scale, visible_rect);
            (layout, paint_run)
        });
        TextGlyphRun {
            layout: TextLayoutResult {
                size: Size::new(layout.size.width / scale, layout.size.height / scale),
                ..layout.scale_lines(1.0 / scale)
            },
            glyphs: paint_run.glyphs,
            backgrounds: paint_run.backgrounds,
            decorations: paint_run.decorations,
        }
    }

    pub fn paint_glyphs(
        &mut self,
        request: TextLayoutRequest<'_>,
        pixels_per_point: f32,
        visible_rect: Option<Rect>,
    ) -> TextPaintGlyphRun {
        let scale = pixels_per_point.max(1.0);
        let buffer_key = TextBufferKey::new(&request, scale);
        let cache_rect = visible_rect.map(text_paint_cache_rect);
        let paint_key = TextPaintGlyphRunKey::new(buffer_key.clone(), cache_rect);
        if let Some(run) = self.paint_runs.get(&paint_key).cloned() {
            self.paint_run_cache_hits += 1;
            return run;
        }
        self.paint_run_cache_misses += 1;
        if self.paint_runs.len() > 1024 {
            self.paint_runs.clear();
        }
        let run = self.with_buffer_key(buffer_key, request.clone(), scale, |buffer, _| {
            collect_text_paint_glyph_run(&request, buffer, scale, cache_rect)
        });
        self.paint_runs.insert(paint_key, run.clone());
        run
    }

    pub fn selection_rects(
        &mut self,
        request: TextLayoutRequest<'_>,
        pixels_per_point: f32,
        selected_layout: Range<usize>,
        color: Color,
    ) -> Vec<TextGlyphRect> {
        if selected_layout.start >= selected_layout.end {
            return Vec::new();
        }
        let scale = pixels_per_point.max(1.0);
        self.with_buffer(request.clone(), scale, |buffer, _| {
            let mut rects = Vec::new();
            for run in buffer.layout_runs() {
                let line_start =
                    line_byte_to_layout_index(request.text.layout_text(), run.line_i, 0);
                let line_end = line_start + run.text.chars().count();
                let start = selected_layout.start.max(line_start);
                let end = selected_layout.end.min(line_end);
                if start >= end {
                    continue;
                }
                let left = x_for_layout_index(request.text.layout_text(), &run, start);
                let right = x_for_layout_index(request.text.layout_text(), &run, end);
                let min_x = left.min(right);
                let max_x = left.max(right);
                push_text_rect(
                    &mut rects,
                    min_x,
                    run.line_top,
                    max_x - min_x,
                    run.line_height,
                    color,
                );
            }
            rects
        })
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
        self.with_buffer_key(key, request, scale, f)
    }

    fn with_buffer_key<R>(
        &mut self,
        key: TextBufferKey,
        request: TextLayoutRequest<'_>,
        scale: f32,
        f: impl FnOnce(&mut cosmic_text::BorrowedWithFontSystem<'_, Buffer>, &mut SwashCache) -> R,
    ) -> R {
        if self.buffers.len() > 512 && !self.buffers.contains_key(&key) {
            self.buffers.clear();
            self.paint_runs.clear();
        }
        if self.buffers.contains_key(&key) {
            self.buffer_cache_hits += 1;
        } else {
            self.buffer_cache_misses += 1;
        }
        let font_system = &mut self.font_system;
        if !self.buffers.contains_key(&key) {
            let metrics = buffer_metrics(&request, scale);
            let mut buffer = Buffer::new_empty(metrics);
            let mut borrowed = buffer.borrow_with(font_system);
            configure_buffer(&mut borrowed, &request, scale, &self.font_families);
            self.buffers.insert(key.clone(), buffer);
        }
        let buffer = self
            .buffers
            .get_mut(&key)
            .expect("text buffer should exist after insertion");
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
    text: TextContentFingerprint,
    font_size: u32,
    color: [u8; 4],
    direction: DirectionKey,
    wrap_width: u32,
    layout_style: TextLayoutStyleKey,
    line_height: Option<u32>,
    scale: u32,
    runs: Vec<InlineStyleKey>,
}

impl TextBufferKey {
    fn new(request: &TextLayoutRequest<'_>, scale: f32) -> Self {
        Self {
            text: TextContentFingerprint::new(request),
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
                .map(|run| InlineStyleKey::from(&run.style))
                .collect(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct TextContentFingerprint {
    layout_bytes: usize,
    layout_chars: usize,
    run_count: usize,
    hash_a: u64,
    hash_b: u64,
}

impl TextContentFingerprint {
    fn new(request: &TextLayoutRequest<'_>) -> Self {
        let mut primary = DefaultHasher::new();
        let mut secondary = DefaultHasher::new();
        "des-text-layout-v1".hash(&mut primary);
        "des-text-runs-v1".hash(&mut secondary);
        request.text.layout_text().hash(&mut primary);
        request.text.layout_text().hash(&mut secondary);
        for run in request.text.runs() {
            run.text.hash(&mut primary);
            run.text.len().hash(&mut secondary);
            run.text.hash(&mut secondary);
        }

        Self {
            layout_bytes: request.text.layout_text().len(),
            layout_chars: request.text.layout_text().chars().count(),
            run_count: request.text.runs().len(),
            hash_a: primary.finish(),
            hash_b: secondary.finish(),
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct TextPaintGlyphRunKey {
    buffer: TextBufferKey,
    visible_rect: Option<RectKey>,
}

impl TextPaintGlyphRunKey {
    fn new(buffer: TextBufferKey, visible_rect: Option<Rect>) -> Self {
        Self {
            buffer,
            visible_rect: visible_rect.map(RectKey::from),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct RectKey {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

impl From<Rect> for RectKey {
    fn from(rect: Rect) -> Self {
        Self {
            x: f32_key(rect.origin.x),
            y: f32_key(rect.origin.y),
            width: f32_key(rect.size.width),
            height: f32_key(rect.size.height),
        }
    }
}

fn text_paint_cache_rect(rect: Rect) -> Rect {
    if !rect.origin.x.is_finite()
        || !rect.origin.y.is_finite()
        || !rect.size.width.is_finite()
        || !rect.size.height.is_finite()
    {
        return rect;
    }

    // Cache a snapped coverage rect, not the exact clip. egui still owns final clipping;
    // this just prevents small scroll deltas from forcing a new cosmic paint run.
    let tile_width = rect.size.width.max(MIN_TEXT_PAINT_CACHE_TILE).ceil();
    let tile_height = rect.size.height.max(MIN_TEXT_PAINT_CACHE_TILE).ceil();
    let origin_x = (rect.origin.x / tile_width).floor() * tile_width;
    let origin_y = (rect.origin.y / tile_height).floor() * tile_height;

    Rect::new(origin_x, origin_y, tile_width * 2.0, tile_height * 2.0)
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
    font_families: &FontFamilyResolver,
) {
    let metrics = buffer_metrics(request, scale);
    buffer.set_metrics(metrics);
    buffer.set_tab_width(request.layout_style.tab_size.max(1));
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

    let default_style = InlineTextStyle::default();
    let default_attrs = cosmic_attrs(
        &default_style,
        request.font_size,
        request.color,
        request.line_height,
        scale,
        0,
        font_families,
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
                font_families,
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
    if request.wrap_width.is_finite() && request.wrap_width > 1.0 {
        Some((request.wrap_width * scale).max(1.0))
    } else {
        None
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
    scale: f32,
) -> TextLayoutResult {
    let mut lines = Vec::new();
    let mut max_width: f32 = 0.0;
    let mut max_height: f32 = 0.0;
    let mut line_count = 0usize;
    for run in buffer.layout_runs() {
        line_count += 1;
        max_width = max_width.max(run.line_w);
        max_height = max_height.max(run.line_top + run.line_height);
        let layout_start = run
            .glyphs
            .first()
            .map(|glyph| {
                line_byte_to_layout_index(request.text.layout_text(), run.line_i, glyph.start)
            })
            .unwrap_or_else(|| {
                line_byte_to_layout_index(request.text.layout_text(), run.line_i, 0)
            });
        let layout_end = run
            .glyphs
            .last()
            .map(|glyph| {
                line_byte_to_layout_index(request.text.layout_text(), run.line_i, glyph.end)
            })
            .unwrap_or(layout_start);
        lines.push(TextLayoutLine {
            layout_start,
            layout_end,
            semantic_start: request.text.layout_to_semantic_index(layout_start),
            semantic_end: request.text.layout_to_semantic_index(layout_end),
            x_offset: glyph_span_min_x(run.glyphs),
            width: run.line_w,
            height: run.line_height,
            baseline: run.line_y,
        });
    }

    if lines.is_empty() {
        let metrics = buffer_metrics(request, scale);
        let baseline = (metrics.font_size * 0.8).clamp(0.0, metrics.line_height);
        lines.push(TextLayoutLine {
            layout_start: 0,
            layout_end: 0,
            semantic_start: 0,
            semantic_end: 0,
            x_offset: 0.0,
            width: 0.0,
            height: metrics.line_height,
            baseline,
        });
        max_height = metrics.line_height;
        line_count = 1;
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

fn glyph_span_min_x(glyphs: &[cosmic_text::LayoutGlyph]) -> f32 {
    if glyphs.is_empty() {
        0.0
    } else {
        glyphs
            .iter()
            .map(|glyph| glyph.x.min(glyph.x + glyph.w))
            .fold(f32::INFINITY, f32::min)
    }
}

fn x_for_layout_index(text: &str, run: &cosmic_text::LayoutRun<'_>, layout_index: usize) -> f32 {
    if run.glyphs.is_empty() {
        return 0.0;
    }
    let line_start = line_byte_to_layout_index(text, run.line_i, 0);
    let local_index = layout_index.saturating_sub(line_start);
    let min_x = run
        .glyphs
        .iter()
        .map(|glyph| glyph.x.min(glyph.x + glyph.w))
        .fold(f32::INFINITY, f32::min);
    let max_x = run
        .glyphs
        .iter()
        .map(|glyph| glyph.x.max(glyph.x + glyph.w))
        .fold(0.0_f32, f32::max);

    for glyph in run.glyphs {
        let glyph_start =
            line_byte_to_layout_index(text, run.line_i, glyph.start).saturating_sub(line_start);
        let glyph_end =
            line_byte_to_layout_index(text, run.line_i, glyph.end).saturating_sub(line_start);
        if local_index <= glyph_start {
            return glyph.x;
        }
        if local_index < glyph_end {
            let span = glyph_end.saturating_sub(glyph_start).max(1) as f32;
            let progress = (local_index - glyph_start) as f32 / span;
            return glyph.x + glyph.w * progress.clamp(0.0, 1.0);
        }
    }

    if run.rtl { min_x } else { max_x }
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
                des_document::OverflowWrap::Anywhere | des_document::OverflowWrap::BreakWord
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

fn cosmic_attrs<'a>(
    style: &'a InlineTextStyle,
    inherited_font_size: f32,
    inherited_color: Color,
    inherited_line_height: Option<f32>,
    scale: f32,
    metadata: usize,
    font_families: &FontFamilyResolver,
) -> Attrs<'a> {
    let color = style.color.unwrap_or(inherited_color);
    let font_size = style.font_size.unwrap_or(inherited_font_size).max(1.0) * scale;
    let line_height = style
        .line_height
        .or(inherited_line_height)
        .unwrap_or(font_size / scale * 1.2)
        .max(1.0)
        * scale;
    let family = font_families.cosmic_family(style.font_family.as_deref());
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
    match style.font_style {
        Some(FontStyle::Italic) => attrs = attrs.style(Style::Italic),
        Some(FontStyle::Oblique) => attrs = attrs.style(Style::Oblique),
        Some(FontStyle::Normal) | None => {}
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

#[derive(Clone, Debug, Default)]
struct FontFamilyResolver {
    available_names: HashSet<String>,
}

impl FontFamilyResolver {
    fn from_db(db: &fontdb::Database) -> Self {
        let mut resolver = Self::default();
        for face in db.faces() {
            for (family, _) in &face.families {
                resolver.available_names.insert(family.to_lowercase());
            }
        }
        resolver
    }

    fn cosmic_family<'a>(&self, declaration: Option<&'a str>) -> Family<'a> {
        let Some(declaration) = declaration else {
            return Family::Name(INTER_FAMILY);
        };
        let mut first_named = None;
        for family in parse_font_family_list(declaration) {
            if family.eq_ignore_ascii_case("monospace")
                || family.eq_ignore_ascii_case("mono")
                || family.eq_ignore_ascii_case(JETBRAINS_MONO_FAMILY)
            {
                return Family::Name(JETBRAINS_MONO_FAMILY);
            }
            if family.eq_ignore_ascii_case("serif") {
                return Family::Serif;
            }
            if family.eq_ignore_ascii_case("sans-serif") {
                return Family::SansSerif;
            }
            if family.eq_ignore_ascii_case("cursive") {
                return Family::Cursive;
            }
            if family.eq_ignore_ascii_case("fantasy") {
                return Family::Fantasy;
            }
            first_named.get_or_insert(family);
            if family.eq_ignore_ascii_case(INTER_FAMILY) || self.has_family(family) {
                return if family.eq_ignore_ascii_case(INTER_FAMILY) {
                    Family::Name(INTER_FAMILY)
                } else {
                    Family::Name(family)
                };
            }
        }
        first_named.map_or(Family::Name(INTER_FAMILY), Family::Name)
    }

    fn has_family(&self, family: &str) -> bool {
        self.available_names.contains(&family.to_lowercase())
    }

    #[cfg(test)]
    fn from_names(names: impl IntoIterator<Item = &'static str>) -> Self {
        Self {
            available_names: names
                .into_iter()
                .map(str::to_lowercase)
                .collect::<HashSet<_>>(),
        }
    }
}

fn parse_font_family_list(declaration: &str) -> Vec<&str> {
    let mut families = Vec::new();
    let mut start = 0;
    let mut quote = None;
    for (index, character) in declaration.char_indices() {
        match (quote, character) {
            (Some(active), next) if next == active => quote = None,
            (None, '\'' | '"') => quote = Some(character),
            (None, ',') => {
                push_font_family_candidate(&mut families, &declaration[start..index]);
                start = index + character.len_utf8();
            }
            _ => {}
        }
    }
    push_font_family_candidate(&mut families, &declaration[start..]);
    families
}

fn push_font_family_candidate<'a>(families: &mut Vec<&'a str>, candidate: &'a str) {
    let candidate = strip_font_family_quotes(candidate.trim());
    if !candidate.is_empty() {
        families.push(candidate);
    }
}

fn strip_font_family_quotes(candidate: &str) -> &str {
    if candidate.len() >= 2 {
        let bytes = candidate.as_bytes();
        let first = bytes[0];
        let last = bytes[bytes.len() - 1];
        if (first == b'\'' && last == b'\'') || (first == b'"' && last == b'"') {
            return &candidate[1..candidate.len() - 1];
        }
    }
    candidate
}

#[cfg(test)]
fn cosmic_family(family: Option<&str>) -> Family<'_> {
    match family.map(str::trim) {
        Some(family)
            if family.eq_ignore_ascii_case("monospace")
                || family.eq_ignore_ascii_case("mono")
                || family.eq_ignore_ascii_case(JETBRAINS_MONO_FAMILY) =>
        {
            Family::Name(JETBRAINS_MONO_FAMILY)
        }
        Some(family) if family.eq_ignore_ascii_case("serif") => Family::Serif,
        Some(family) if family.eq_ignore_ascii_case("sans-serif") => Family::SansSerif,
        Some(family) if family.eq_ignore_ascii_case("cursive") => Family::Cursive,
        Some(family) if family.eq_ignore_ascii_case("fantasy") => Family::Fantasy,
        Some(family) if family.eq_ignore_ascii_case(INTER_FAMILY) => Family::Name(INTER_FAMILY),
        None => Family::Name(INTER_FAMILY),
        Some(family) => Family::Name(family),
    }
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

fn collect_text_paint_glyph_run(
    request: &TextLayoutRequest<'_>,
    buffer: &mut cosmic_text::BorrowedWithFontSystem<'_, Buffer>,
    scale: f32,
    visible_rect: Option<Rect>,
) -> TextPaintGlyphRun {
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
            let baseline_shift = baseline_shifts.get(glyph.metadata).copied().unwrap_or(0.0);
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
                layout_start: line_byte_to_layout_index(
                    request.text.layout_text(),
                    run.line_i,
                    glyph.start,
                ),
                layout_end: line_byte_to_layout_index(
                    request.text.layout_text(),
                    run.line_i,
                    glyph.end,
                ),
            });
        }
        let mut collector = DecorationCollector {
            rects: &mut decorations,
        };
        render_decoration(&mut collector, &run, cosmic_color(request.color));
    }
    TextPaintGlyphRun {
        glyphs,
        backgrounds,
        decorations,
    }
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

    fn renderer_with_system_font_loading(
        system_font_loading: SystemFontLoading,
    ) -> CosmicTextRenderer {
        CosmicTextRenderer::with_system_font_loading(
            [
                FontAsset::new(INTER_FAMILY, INTER),
                FontAsset::new(JETBRAINS_MONO_FAMILY, JETBRAINS_MONO),
                FontAsset::new(JETBRAINS_MONO_FAMILY, JETBRAINS_MONO_ITALIC),
            ],
            system_font_loading,
        )
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
        assert_eq!(rasterized.diagnostics.font_loading, "bundled-only");
        assert!(rasterized.diagnostics.glyph_rects > 0);
        assert!(rasterized.surface.rgba.iter().any(|channel| *channel != 0));
    }

    #[test]
    fn diagnostics_report_system_font_loading_mode() {
        let content = TextContent::plain("Hello");
        let normalized = NormalizedText::from_content(&content, TextLayoutStyle::default());
        let mut bundled = renderer_with_system_font_loading(SystemFontLoading::BundledOnly);
        let mut system = renderer_with_system_font_loading(SystemFontLoading::IncludeSystemFonts);

        assert_eq!(
            bundled
                .rasterize(request(&normalized, 16.0, 300.0), 1.0)
                .diagnostics
                .font_loading,
            "bundled-only"
        );
        assert_eq!(
            system
                .rasterize(request(&normalized, 16.0, 300.0), 1.0)
                .diagnostics
                .font_loading,
            "system-fallbacks"
        );
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
    fn measures_break_word_as_wrappable_long_token() {
        let mut renderer = renderer();
        let content = TextContent::plain("supercalifragilisticexpialidocious");
        let mut style = TextLayoutStyle::default();
        style.overflow_wrap = des_document::OverflowWrap::BreakWord;
        let normalized = NormalizedText::from_content(&content, style);

        let measured = renderer.measure_text(TextLayoutRequest {
            text: &normalized,
            font_size: 16.0,
            color: Color::rgb(24, 24, 30),
            direction: Direction::Ltr,
            wrap_width: 70.0,
            layout_style: style,
            line_height: Some(20.0),
        });

        assert!(
            measured.line_count > 1,
            "overflow-wrap: break-word should break long unspaced text"
        );
    }

    #[test]
    fn measures_empty_text_as_a_line_box() {
        let mut renderer = renderer();
        let content = TextContent::plain("");
        let normalized = NormalizedText::from_content(&content, TextLayoutStyle::default());
        let measured = renderer.measure_text(request(&normalized, 16.0, 90.0));

        assert_eq!(measured.line_count, 1);
        assert_eq!(measured.lines.len(), 1);
        assert_eq!(measured.lines[0].layout_start, 0);
        assert_eq!(measured.lines[0].layout_end, 0);
        assert_eq!(measured.lines[0].semantic_start, 0);
        assert_eq!(measured.lines[0].semantic_end, 0);
        assert_eq!(measured.size.width, 0.0);
        assert!(measured.size.height > 0.0);
        assert!(measured.first_baseline.is_some());
        assert!(measured.lines[0].baseline > 0.0);
        assert!(measured.lines[0].baseline <= measured.lines[0].height);
    }

    #[test]
    fn reports_single_line_ellipsis_as_elided() {
        let mut renderer = renderer();
        let content = TextContent::plain("A compact field title should elide when too wide.");
        let mut style = TextLayoutStyle::default();
        style.max_lines = Some(1);
        style.text_overflow = TextOverflow::Ellipsis;
        let normalized = NormalizedText::from_content(&content, style);
        let measured = renderer.measure_text(TextLayoutRequest {
            text: &normalized,
            font_size: 16.0,
            color: Color::rgb(24, 24, 30),
            direction: Direction::Ltr,
            wrap_width: 90.0,
            layout_style: style,
            line_height: Some(20.0),
        });

        assert_eq!(measured.line_count, 1);
        assert!(measured.elided);
    }

    #[test]
    fn preserves_blank_lines_in_preformatted_text() {
        let mut renderer = renderer();
        let content = TextContent::plain("a\n\nb");
        let style = TextLayoutStyle::white_space(des_document::WhiteSpace::Pre);
        let normalized = NormalizedText::from_content(&content, style);
        let measured = renderer.measure_text(TextLayoutRequest {
            text: &normalized,
            font_size: 16.0,
            color: Color::rgb(24, 24, 30),
            direction: Direction::Ltr,
            wrap_width: 400.0,
            layout_style: style,
            line_height: Some(20.0),
        });

        assert_eq!(normalized.layout_text(), "a\n\nb");
        assert_eq!(measured.line_count, 3);
        assert_eq!(measured.lines.len(), 3);
        assert_eq!(measured.lines[0].layout_start, 0);
        assert_eq!(measured.lines[0].layout_end, 1);
        assert_eq!(measured.lines[1].layout_start, 2);
        assert_eq!(measured.lines[1].layout_end, 2);
        assert_eq!(measured.lines[2].layout_start, 3);
        assert_eq!(measured.lines[2].layout_end, 4);
        assert!(measured.size.height >= 60.0);
    }

    #[test]
    fn measures_normalized_css_tab_size_spacing() {
        let mut renderer = renderer();
        let content = TextContent::plain("a\tb");
        let mut compact_style = TextLayoutStyle::white_space(des_document::WhiteSpace::Pre);
        compact_style.tab_size = 2;
        let mut wide_style = TextLayoutStyle::white_space(des_document::WhiteSpace::Pre);
        wide_style.tab_size = 8;
        let compact = NormalizedText::from_content(&content, compact_style);
        let wide = NormalizedText::from_content(&content, wide_style);

        let compact_measured = renderer.measure_text(TextLayoutRequest {
            text: &compact,
            font_size: 16.0,
            color: Color::rgb(24, 24, 30),
            direction: Direction::Ltr,
            wrap_width: 400.0,
            layout_style: compact_style,
            line_height: Some(20.0),
        });
        let wide_measured = renderer.measure_text(TextLayoutRequest {
            text: &wide,
            font_size: 16.0,
            color: Color::rgb(24, 24, 30),
            direction: Direction::Ltr,
            wrap_width: 400.0,
            layout_style: wide_style,
            line_height: Some(20.0),
        });

        assert_eq!(compact.layout_text(), "a  b");
        assert_eq!(wide.layout_text(), "a        b");
        assert!(
            wide_measured.size.width > compact_measured.size.width,
            "larger CSS tab-size should widen preserved tab stops"
        );
    }

    #[test]
    fn resolves_start_and_end_alignment_against_direction() {
        let mut renderer = renderer();
        let content = TextContent::plain("abcd");
        let mut style = TextLayoutStyle::white_space(des_document::WhiteSpace::Pre);
        style.text_align = TextAlign::Start;
        let normalized = NormalizedText::from_content(&content, style);

        let rtl_start = renderer.measure_text(TextLayoutRequest {
            text: &normalized,
            font_size: 16.0,
            color: Color::rgb(24, 24, 30),
            direction: Direction::Rtl,
            wrap_width: 120.0,
            layout_style: style,
            line_height: Some(20.0),
        });

        style.text_align = TextAlign::End;
        let ltr_end = renderer.measure_text(TextLayoutRequest {
            text: &normalized,
            font_size: 16.0,
            color: Color::rgb(24, 24, 30),
            direction: Direction::Ltr,
            wrap_width: 120.0,
            layout_style: style,
            line_height: Some(20.0),
        });
        let rtl_end = renderer.measure_text(TextLayoutRequest {
            text: &normalized,
            font_size: 16.0,
            color: Color::rgb(24, 24, 30),
            direction: Direction::Rtl,
            wrap_width: 120.0,
            layout_style: style,
            line_height: Some(20.0),
        });

        assert!(
            rtl_start.lines[0].x_offset > 0.0,
            "RTL start alignment should place the line at the right edge"
        );
        assert!(
            ltr_end.lines[0].x_offset > 0.0,
            "LTR end alignment should place the line at the right edge"
        );
        assert!(
            (rtl_start.lines[0].x_offset - ltr_end.lines[0].x_offset).abs() < 0.5,
            "RTL start and LTR end should resolve to matching physical alignment"
        );
        assert!(
            rtl_end.lines[0].x_offset.abs() < 0.5,
            "RTL end alignment should place the line at the left edge"
        );
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
    fn rtl_hit_testing_tracks_visual_start_and_end() {
        let mut renderer = renderer();
        let content = TextContent::plain("אבגד");
        let mut style = TextLayoutStyle::white_space(des_document::WhiteSpace::Pre);
        style.text_align = TextAlign::Start;
        let normalized = NormalizedText::from_content(&content, style);
        let text_request = TextLayoutRequest {
            text: &normalized,
            font_size: 24.0,
            color: Color::rgb(24, 24, 30),
            direction: Direction::Rtl,
            wrap_width: 240.0,
            layout_style: style,
            line_height: Some(32.0),
        };
        let measured = renderer.measure_text(text_request.clone());
        let line = measured.lines[0];
        let y = line.baseline;

        let visual_start = renderer.text_index_at(
            text_request.clone(),
            Point::new(line.x_offset + line.width - 1.0, y),
        );
        let visual_end = renderer.text_index_at(text_request, Point::new(line.x_offset + 1.0, y));

        assert_eq!(visual_start, 0);
        assert_eq!(visual_end, content.semantic_text().chars().count());
    }

    #[test]
    fn rtl_selection_rects_cover_right_aligned_visual_range() {
        let mut renderer = renderer();
        let content = TextContent::plain("אבגד");
        let mut style = TextLayoutStyle::white_space(des_document::WhiteSpace::Pre);
        style.text_align = TextAlign::Start;
        let normalized = NormalizedText::from_content(&content, style);
        let text_request = TextLayoutRequest {
            text: &normalized,
            font_size: 24.0,
            color: Color::rgb(24, 24, 30),
            direction: Direction::Rtl,
            wrap_width: 240.0,
            layout_style: style,
            line_height: Some(32.0),
        };
        let measured = renderer.measure_text(text_request.clone());
        let rects = renderer.selection_rects(
            text_request,
            1.0,
            0..normalized.layout_text().chars().count(),
            Color::rgba(234, 221, 255, 190),
        );

        assert_eq!(rects.len(), 1);
        assert!(rects[0].width_px > 0);
        assert!(
            rects[0].x_px as f32 >= measured.lines[0].x_offset.floor(),
            "RTL selection should be positioned inside the right-aligned line"
        );
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
    fn paint_glyphs_match_full_glyph_run_without_rebuilding_layout() {
        let mut renderer = renderer();
        let highlight = Color::rgba(234, 221, 255, 180);
        let content = TextContent::new(vec![
            TextRun::plain("Paint "),
            TextRun::styled(
                "glyphs",
                InlineTextStyle {
                    background: Some(highlight),
                    ..InlineTextStyle::default()
                },
            ),
        ]);
        let normalized = NormalizedText::from_content(&content, TextLayoutStyle::default());
        let request = request(&normalized, 24.0, 400.0);
        let full_run = renderer.glyphs(request.clone(), 2.0, None);
        let paint_run = renderer.paint_glyphs(request, 2.0, None);

        assert_eq!(paint_run.glyphs, full_run.glyphs);
        assert_eq!(paint_run.backgrounds, full_run.backgrounds);
        assert_eq!(paint_run.decorations, full_run.decorations);
        assert!(
            paint_run
                .backgrounds
                .iter()
                .any(|rect| rect.color == highlight && rect.width_px > 0 && rect.height_px > 0),
            "paint-only glyph runs should preserve inline background output"
        );
    }

    #[test]
    fn paint_glyphs_reuse_cached_visible_runs() {
        let mut renderer = renderer();
        let content = TextContent::plain("Paint cache keeps warm frames cheap");
        let normalized = NormalizedText::from_content(&content, TextLayoutStyle::default());
        let request = request(&normalized, 24.0, 400.0);
        let visible = Some(Rect::new(0.0, 0.0, 240.0, 80.0));

        let cold = renderer.paint_glyphs(request.clone(), 2.0, visible);
        assert!(!cold.glyphs.is_empty());
        renderer.begin_frame();
        let warm = renderer.paint_glyphs(request, 2.0, visible);
        let stats = renderer.buffer_stats();

        assert_eq!(warm, cold);
        assert_eq!(stats.paint_run_cache_hits, 1);
        assert_eq!(stats.paint_run_cache_misses, 0);
        assert!(stats.paint_run_cache_entries > 0);
    }

    #[test]
    fn paint_glyphs_reuse_cached_runs_across_nearby_visible_rects() {
        let mut renderer = renderer();
        let content = TextContent::plain("Nearby scroll offsets should not reshape warm text");
        let normalized = NormalizedText::from_content(&content, TextLayoutStyle::default());
        let request = request(&normalized, 24.0, 420.0);
        let first_visible = Some(Rect::new(0.0, 0.0, 240.0, 80.0));
        let nearby_visible = Some(Rect::new(0.0, 32.0, 240.0, 80.0));

        let cold = renderer.paint_glyphs(request.clone(), 2.0, first_visible);
        assert!(!cold.glyphs.is_empty());
        renderer.begin_frame();
        let nearby = renderer.paint_glyphs(request, 2.0, nearby_visible);
        let stats = renderer.buffer_stats();

        assert_eq!(nearby, cold);
        assert_eq!(stats.paint_run_cache_hits, 1);
        assert_eq!(stats.paint_run_cache_misses, 0);
    }

    #[test]
    fn text_buffer_cache_key_distinguishes_run_boundaries_without_storing_run_text() {
        let mut renderer = renderer();
        let first_content = TextContent::new(vec![TextRun::plain("ab"), TextRun::plain("c")]);
        let second_content = TextContent::new(vec![TextRun::plain("a"), TextRun::plain("bc")]);
        let first = NormalizedText::from_content(&first_content, TextLayoutStyle::default());
        let second = NormalizedText::from_content(&second_content, TextLayoutStyle::default());

        assert_eq!(first.layout_text(), second.layout_text());

        renderer.measure_text(request(&first, 16.0, 240.0));
        renderer.measure_text(request(&second, 16.0, 240.0));
        let stats = renderer.buffer_stats();

        assert_eq!(stats.cache_misses, 2);
        assert_eq!(stats.cache_hits, 0);
        assert_eq!(stats.cache_entries, 2);
    }

    #[test]
    fn text_paint_cache_rect_covers_adjacent_scroll_offsets() {
        let first = text_paint_cache_rect(Rect::new(0.0, 0.0, 240.0, 80.0));
        let nearby = text_paint_cache_rect(Rect::new(0.0, 32.0, 240.0, 80.0));
        let distant = text_paint_cache_rect(Rect::new(0.0, 300.0, 240.0, 80.0));

        assert_eq!(nearby, first);
        assert_ne!(distant, first);
        assert!(first.size.width >= 480.0);
        assert!(first.size.height >= 512.0);
    }

    #[test]
    fn exposes_selection_rectangles_for_layout_ranges() {
        let mut renderer = renderer();
        let content = TextContent::plain("select me");
        let normalized = NormalizedText::from_content(&content, TextLayoutStyle::default());
        let rects = renderer.selection_rects(
            request(&normalized, 20.0, 400.0),
            2.0,
            0..6,
            Color::rgba(234, 221, 255, 190),
        );

        assert_eq!(rects.len(), 1);
        assert_eq!(rects[0].color, Color::rgba(234, 221, 255, 190));
        assert!(rects[0].width_px > 0);
        assert!(rects[0].height_px > 0);
    }

    #[test]
    fn layout_lines_use_character_indices_for_unicode_text() {
        let mut renderer = renderer();
        let content = TextContent::plain("aé🙂b");
        let normalized = NormalizedText::from_content(&content, TextLayoutStyle::default());
        let measured = renderer.measure_text(request(&normalized, 24.0, 400.0));
        let line = measured
            .lines
            .first()
            .expect("unicode text should produce one measured line");

        assert_eq!(normalized.layout_text().chars().count(), 4);
        assert_eq!(normalized.layout_text().len(), 8);
        assert_eq!(line.layout_start, 0);
        assert_eq!(line.layout_end, 4);
        assert_eq!(line.semantic_start, 0);
        assert_eq!(line.semantic_end, 4);
        assert!(
            measured
                .lines
                .iter()
                .all(|line| line.layout_end <= normalized.layout_text().chars().count()),
            "layout line ranges must never expose UTF-8 byte offsets"
        );
    }

    #[test]
    fn glyph_ranges_use_character_indices_for_unicode_text() {
        let mut renderer = renderer();
        let content = TextContent::plain("aé🙂b");
        let normalized = NormalizedText::from_content(&content, TextLayoutStyle::default());
        let glyph_run = renderer.glyphs(request(&normalized, 24.0, 400.0), 2.0, None);
        let char_count = normalized.layout_text().chars().count();

        assert!(!glyph_run.glyphs.is_empty());
        assert!(
            glyph_run
                .glyphs
                .iter()
                .all(|glyph| glyph.layout_start <= char_count && glyph.layout_end <= char_count),
            "glyph ranges must stay in document layout character coordinates"
        );
        assert!(
            glyph_run
                .glyphs
                .iter()
                .any(|glyph| glyph.layout_start < 2 && glyph.layout_end > 1),
            "accented character should have a paintable glyph range"
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
        let style = InlineTextStyle {
            font_stretch: Some(des_document::FontStretch::EXPANDED),
            ..InlineTextStyle::default()
        };
        let resolver = FontFamilyResolver::from_names([INTER_FAMILY, JETBRAINS_MONO_FAMILY]);
        let attrs = cosmic_attrs(&style, 16.0, Color::rgb(1, 2, 3), None, 1.0, 0, &resolver);

        assert_eq!(attrs.stretch, Stretch::Expanded);
    }

    #[test]
    fn maps_css_font_family_names_to_cosmic_attrs() {
        let declared_family = "Aptos";
        assert_eq!(cosmic_family(Some(declared_family)), Family::Name("Aptos"));
        assert_eq!(cosmic_family(Some("serif")), Family::Serif);
        assert_eq!(cosmic_family(Some("sans-serif")), Family::SansSerif);
        assert_eq!(cosmic_family(Some("cursive")), Family::Cursive);
        assert_eq!(cosmic_family(Some("fantasy")), Family::Fantasy);
        assert_eq!(
            cosmic_family(Some("monospace")),
            Family::Name(JETBRAINS_MONO_FAMILY)
        );

        let style = InlineTextStyle {
            font_family: Some(declared_family.to_string()),
            ..InlineTextStyle::default()
        };
        let resolver = FontFamilyResolver::from_names([INTER_FAMILY, JETBRAINS_MONO_FAMILY]);
        let attrs = cosmic_attrs(&style, 16.0, Color::rgb(1, 2, 3), None, 1.0, 0, &resolver);

        assert_eq!(attrs.family, Family::Name("Aptos"));
    }

    #[test]
    fn maps_css_font_family_lists_to_first_available_cosmic_family() {
        let resolver = FontFamilyResolver::from_names([INTER_FAMILY, JETBRAINS_MONO_FAMILY]);

        assert_eq!(
            parse_font_family_list("\"Aptos\", Inter, sans-serif"),
            vec!["Aptos", "Inter", "sans-serif"]
        );
        assert_eq!(
            resolver.cosmic_family(Some("Aptos, Inter, sans-serif")),
            Family::Name(INTER_FAMILY)
        );
        assert_eq!(resolver.cosmic_family(Some("Aptos, serif")), Family::Serif);
        assert_eq!(
            resolver.cosmic_family(Some("\"JetBrains Mono\", monospace")),
            Family::Name(JETBRAINS_MONO_FAMILY)
        );
        assert_eq!(
            resolver.cosmic_family(Some("Aptos, Unknown Sans")),
            Family::Name("Aptos")
        );

        let style = InlineTextStyle {
            font_family: Some("Aptos, Inter, sans-serif".to_string()),
            ..InlineTextStyle::default()
        };
        let attrs = cosmic_attrs(&style, 16.0, Color::rgb(1, 2, 3), None, 1.0, 0, &resolver);

        assert_eq!(attrs.family, Family::Name(INTER_FAMILY));
    }

    #[test]
    fn maps_font_style_keywords_to_cosmic_style() {
        let resolver = FontFamilyResolver::from_names([INTER_FAMILY, JETBRAINS_MONO_FAMILY]);
        let italic = InlineTextStyle {
            font_style: Some(FontStyle::Italic),
            ..InlineTextStyle::default()
        };
        let oblique = InlineTextStyle {
            font_style: Some(FontStyle::Oblique),
            ..InlineTextStyle::default()
        };
        let italic_attrs =
            cosmic_attrs(&italic, 16.0, Color::rgb(1, 2, 3), None, 1.0, 0, &resolver);
        let oblique_attrs =
            cosmic_attrs(&oblique, 16.0, Color::rgb(1, 2, 3), None, 1.0, 0, &resolver);

        assert_eq!(italic_attrs.style, Style::Italic);
        assert_eq!(oblique_attrs.style, Style::Oblique);
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
