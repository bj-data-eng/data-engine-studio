use cosmic_text::{
    Align, Attrs, Buffer, Color as CosmicColor, Ellipsize, EllipsizeHeightLimit, Family,
    FontSystem, Metrics, Shaping, Style, SwashCache, Weight, Wrap,
};
use des_document::{
    Color, Direction, FontStyle, InlineTextStyle, Point, Size, TextAlign, TextLayoutLine,
    TextLayoutRequest, TextLayoutResult, TextMeasurer, TextMeasurerKey, TextOverflow, TextWrapMode,
};
use std::sync::Arc;

pub const INTER_FAMILY: &str = "Inter";
pub const JETBRAINS_MONO_FAMILY: &str = "JetBrains Mono";

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
        }
    }

    pub fn rasterize(
        &mut self,
        request: TextLayoutRequest<'_>,
        pixels_per_point: f32,
    ) -> RasterizedText {
        let scale = pixels_per_point.max(1.0);
        let (layout, rgba, glyph_rects, width_px, height_px) =
            self.with_buffer(request.clone(), scale, |buffer, swash_cache| {
                let layout = layout_result(&request, buffer);
                let width_px = surface_width_px(
                    &layout,
                    request.wrap_width,
                    request.layout_style.text_wrap_mode,
                    scale,
                );
                let height_px = finite_surface_extent(layout.size.height).ceil() as u32;
                let mut rgba =
                    vec![0; width_px.saturating_mul(height_px).saturating_mul(4) as usize];
                let mut glyph_rects = 0usize;
                buffer.draw(
                    swash_cache,
                    cosmic_color(request.color),
                    |x, y, w, h, color| {
                        glyph_rects += 1;
                        blend_rect(&mut rgba, width_px, height_px, x, y, w, h, color);
                    },
                );
                (layout, rgba, glyph_rects, width_px, height_px)
            });

        RasterizedText {
            surface: TextSurface {
                size: Size::new(width_px as f32 / scale, height_px as f32 / scale),
                pixels_per_point: scale,
                width_px,
                height_px,
                rgba,
            },
            layout: TextLayoutResult {
                size: Size::new(layout.size.width / scale, layout.size.height / scale),
                ..layout.scale_lines(1.0 / scale)
            },
            diagnostics: TextDiagnostics {
                backend: "cosmic-text",
                proportional_family: INTER_FAMILY,
                monospace_family: JETBRAINS_MONO_FAMILY,
                pixels_per_point: scale,
                width_px,
                height_px,
                glyph_rects,
            },
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
        let metrics = Metrics::new(
            request.font_size.max(1.0) * scale,
            request
                .line_height
                .unwrap_or(request.font_size * 1.2)
                .max(1.0)
                * scale,
        );
        let wrap_width = match request.layout_style.text_wrap_mode {
            TextWrapMode::NoWrap => None,
            TextWrapMode::Wrap if request.wrap_width.is_finite() && request.wrap_width > 1.0 => {
                Some((request.wrap_width * scale).max(1.0))
            }
            TextWrapMode::Wrap => None,
        };
        let height = request
            .layout_style
            .max_lines
            .map(|lines| lines.max(1) as f32 * metrics.line_height);
        let mut buffer = Buffer::new(&mut self.font_system, metrics);
        let mut buffer = buffer.borrow_with(&mut self.font_system);
        buffer.set_wrap(cosmic_wrap(&request));
        buffer.set_size(wrap_width, height);
        if request.layout_style.text_overflow == TextOverflow::Ellipsis {
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
        );
        let spans = request.text.runs().iter().map(|run| {
            (
                run.text.as_str(),
                cosmic_attrs(
                    &run.style,
                    request.font_size,
                    request.color,
                    request.line_height,
                    scale,
                ),
            )
        });
        buffer.set_rich_text(
            spans,
            &default_attrs,
            Shaping::Advanced,
            Some(cosmic_align(
                request.layout_style.text_align,
                request.direction,
            )),
        );

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

fn surface_width_px(
    layout: &TextLayoutResult,
    wrap_width: f32,
    wrap_mode: TextWrapMode,
    scale: f32,
) -> u32 {
    let width = match wrap_mode {
        TextWrapMode::NoWrap => layout.size.width.max(1.0),
        TextWrapMode::Wrap if wrap_width.is_finite() && wrap_width > 1.0 => {
            (wrap_width * scale).max(layout.size.width).max(1.0)
        }
        TextWrapMode::Wrap => layout.size.width.max(1.0),
    };
    finite_surface_extent(width).ceil() as u32
}

fn finite_surface_extent(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(1.0, 16_384.0)
    } else {
        16_384.0
    }
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

fn cosmic_attrs(
    style: &InlineTextStyle,
    inherited_font_size: f32,
    inherited_color: Color,
    inherited_line_height: Option<f32>,
    scale: f32,
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
        .color(cosmic_color(color));
    if let Some(weight) = style.font_weight {
        attrs = attrs.weight(Weight(weight.value()));
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
    attrs
}

fn cosmic_color(color: Color) -> CosmicColor {
    CosmicColor::rgba(color.r, color.g, color.b, color.a)
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
    use des_document::{NormalizedText, TextContent, TextLayoutStyle};

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
}
