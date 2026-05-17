//! Renderer-neutral text layout contracts.
//!
//! This crate is the document UI text boundary. It owns text layout, hit
//! testing, selection geometry, and glyph-run extraction without depending on
//! a host renderer such as egui.

use fontique::{
    Blob, Collection, CollectionOptions, FontInfoOverride, FontStyle, FontWeight, FontWidth,
    GenericFamily, SourceCache,
};
use parley::{
    Alignment, AlignmentOptions, Cursor, FontContext, FontFamily, Layout, LayoutContext,
    LineHeight, PositionedLayoutItem, Selection, StyleProperty, layout::Affinity,
};
use std::{fmt, ops::Range};

pub const DEFAULT_FONT_FAMILY: &str = "Inter";

const INTER_VARIABLE: &[u8] = include_bytes!("../assets/fonts/inter/InterVariable.ttf");
const INTER_VARIABLE_ITALIC: &[u8] =
    include_bytes!("../assets/fonts/inter/InterVariable-Italic.ttf");

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FontOrigin {
    BuiltIn,
    Document,
    User,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum FontSlant {
    #[default]
    Normal,
    Italic,
    Oblique,
}

impl FontSlant {
    fn to_fontique(self) -> FontStyle {
        match self {
            Self::Normal => FontStyle::Normal,
            Self::Italic => FontStyle::Italic,
            Self::Oblique => FontStyle::Oblique(None),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FontFaceAttributes {
    pub weight: Option<u16>,
    pub width: Option<u16>,
    pub slant: FontSlant,
}

impl FontFaceAttributes {
    pub fn regular() -> Self {
        Self::default()
    }

    pub fn italic() -> Self {
        Self {
            slant: FontSlant::Italic,
            ..Self::default()
        }
    }

    pub fn weight(mut self, weight: u16) -> Self {
        self.weight = Some(weight);
        self
    }

    pub fn width(mut self, width: u16) -> Self {
        self.width = Some(width);
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct FontFaceId(usize);

impl FontFaceId {
    pub fn index(self) -> usize {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FontFace {
    pub id: FontFaceId,
    pub family: String,
    pub origin: FontOrigin,
    pub attributes: FontFaceAttributes,
    bytes: Blob<u8>,
}

impl FontFace {
    pub fn bytes(&self) -> &[u8] {
        self.bytes.as_ref()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FontRegistryError {
    EmptyFamily,
    EmptyFont,
    InvalidFont { family: String },
}

impl fmt::Display for FontRegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyFamily => f.write_str("font family cannot be empty"),
            Self::EmptyFont => f.write_str("font data cannot be empty"),
            Self::InvalidFont { family } => {
                write!(f, "font data did not register any faces for {family:?}")
            }
        }
    }
}

impl std::error::Error for FontRegistryError {}

#[derive(Clone, Debug, PartialEq)]
pub struct FontRegistry {
    default_family: String,
    faces: Vec<FontFace>,
}

impl FontRegistry {
    pub fn empty(default_family: impl Into<String>) -> Self {
        Self {
            default_family: default_family.into(),
            faces: Vec::new(),
        }
    }

    pub fn with_builtin_fonts() -> Self {
        let mut registry = Self::empty(DEFAULT_FONT_FAMILY);
        registry
            .add_font_bytes(
                FontOrigin::BuiltIn,
                DEFAULT_FONT_FAMILY,
                INTER_VARIABLE,
                FontFaceAttributes::regular(),
            )
            .expect("bundled Inter regular font must be valid");
        registry
            .add_font_bytes(
                FontOrigin::BuiltIn,
                DEFAULT_FONT_FAMILY,
                INTER_VARIABLE_ITALIC,
                FontFaceAttributes::italic(),
            )
            .expect("bundled Inter italic font must be valid");
        registry
    }

    pub fn default_family(&self) -> &str {
        &self.default_family
    }

    pub fn set_default_family(&mut self, family: impl Into<String>) {
        self.default_family = family.into();
    }

    pub fn faces(&self) -> &[FontFace] {
        &self.faces
    }

    pub fn contains_family(&self, family: &str) -> bool {
        self.faces.iter().any(|face| face.family == family)
    }

    pub fn add_builtin_font(
        &mut self,
        family: impl Into<String>,
        bytes: impl AsRef<[u8]>,
        attributes: FontFaceAttributes,
    ) -> Result<FontFaceId, FontRegistryError> {
        self.add_font_bytes(FontOrigin::BuiltIn, family, bytes, attributes)
    }

    pub fn add_document_font(
        &mut self,
        family: impl Into<String>,
        bytes: impl AsRef<[u8]>,
        attributes: FontFaceAttributes,
    ) -> Result<FontFaceId, FontRegistryError> {
        self.add_font_bytes(FontOrigin::Document, family, bytes, attributes)
    }

    pub fn add_user_font(
        &mut self,
        family: impl Into<String>,
        bytes: impl AsRef<[u8]>,
        attributes: FontFaceAttributes,
    ) -> Result<FontFaceId, FontRegistryError> {
        self.add_font_bytes(FontOrigin::User, family, bytes, attributes)
    }

    pub fn add_font_bytes(
        &mut self,
        origin: FontOrigin,
        family: impl Into<String>,
        bytes: impl AsRef<[u8]>,
        attributes: FontFaceAttributes,
    ) -> Result<FontFaceId, FontRegistryError> {
        let family = family.into();
        if family.trim().is_empty() {
            return Err(FontRegistryError::EmptyFamily);
        }
        let bytes = bytes.as_ref();
        if bytes.is_empty() {
            return Err(FontRegistryError::EmptyFont);
        }
        let bytes: Blob<u8> = bytes.to_vec().into();
        validate_font_face(&family, bytes.clone(), attributes)?;

        let id = FontFaceId(self.faces.len());
        self.faces.push(FontFace {
            id,
            family,
            origin,
            attributes,
            bytes,
        });
        Ok(id)
    }

    fn to_parley_font_context(&self) -> FontContext {
        let mut collection = Collection::new(CollectionOptions {
            shared: false,
            system_fonts: false,
        });
        for face in &self.faces {
            register_face(&mut collection, face);
        }
        configure_generic_families(&mut collection, &self.default_family);
        FontContext {
            collection,
            source_cache: SourceCache::default(),
        }
    }
}

impl Default for FontRegistry {
    fn default() -> Self {
        Self::with_builtin_fonts()
    }
}

fn validate_font_face(
    family: &str,
    bytes: Blob<u8>,
    attributes: FontFaceAttributes,
) -> Result<(), FontRegistryError> {
    let mut collection = Collection::new(CollectionOptions {
        shared: false,
        system_fonts: false,
    });
    let registered = collection.register_fonts(bytes, Some(font_info_override(family, attributes)));
    if registered.is_empty() {
        Err(FontRegistryError::InvalidFont {
            family: family.to_owned(),
        })
    } else {
        Ok(())
    }
}

fn register_face(collection: &mut Collection, face: &FontFace) {
    collection.register_fonts(
        face.bytes.clone(),
        Some(font_info_override(&face.family, face.attributes)),
    );
}

fn font_info_override(family: &str, attributes: FontFaceAttributes) -> FontInfoOverride<'_> {
    FontInfoOverride {
        family_name: Some(family),
        width: attributes
            .width
            .map(|width| FontWidth::from_percentage(width as f32)),
        style: Some(attributes.slant.to_fontique()),
        weight: attributes
            .weight
            .map(|weight| FontWeight::new(weight as f32)),
        axes: None,
    }
}

fn configure_generic_families(collection: &mut Collection, family: &str) {
    if let Some(id) = collection.family_id(family) {
        collection.set_generic_families(GenericFamily::SansSerif, [id].into_iter());
        collection.set_generic_families(GenericFamily::SystemUi, [id].into_iter());
    }
}

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
    pub font_family: String,
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
            font_family: DEFAULT_FONT_FAMILY.to_owned(),
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

    pub fn font_family(mut self, family: impl Into<String>) -> Self {
        self.font_family = family.into();
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

pub struct TextEngine {
    font_registry: FontRegistry,
    font_context: FontContext,
    layout_context: LayoutContext<TextColor>,
}

impl TextEngine {
    pub fn new() -> Self {
        Self::with_font_registry(FontRegistry::default())
    }

    pub fn with_font_registry(font_registry: FontRegistry) -> Self {
        let font_context = font_registry.to_parley_font_context();
        Self {
            font_registry,
            font_context,
            layout_context: LayoutContext::default(),
        }
    }

    pub fn font_registry(&self) -> &FontRegistry {
        &self.font_registry
    }

    pub fn layout(&mut self, request: &TextLayoutRequest) -> TextLayout {
        let mut builder = self.layout_context.ranged_builder(
            &mut self.font_context,
            &request.text,
            request.display_scale,
            true,
        );
        builder.push_default(StyleProperty::FontFamily(FontFamily::named(
            &request.font_family,
        )));
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
        assert_eq!(engine.font_registry().default_family(), DEFAULT_FONT_FAMILY);

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

    #[test]
    fn font_registry_uses_bundled_inter_by_default() {
        let registry = FontRegistry::default();

        assert_eq!(registry.default_family(), DEFAULT_FONT_FAMILY);
        assert!(registry.contains_family(DEFAULT_FONT_FAMILY));
        assert_eq!(registry.faces().len(), 2);
        assert!(
            registry
                .faces()
                .iter()
                .all(|face| face.origin == FontOrigin::BuiltIn)
        );
    }

    #[test]
    fn font_registry_accepts_document_font_bytes() {
        let mut registry = FontRegistry::empty("Report Sans");
        let id = registry
            .add_document_font("Report Sans", INTER_VARIABLE, FontFaceAttributes::regular())
            .expect("known valid bundled font can also be registered as a document font");

        assert_eq!(id.index(), 0);
        assert_eq!(registry.faces()[0].origin, FontOrigin::Document);
        assert!(registry.contains_family("Report Sans"));
    }

    #[test]
    fn font_registry_rejects_invalid_font_bytes() {
        let mut registry = FontRegistry::empty(DEFAULT_FONT_FAMILY);

        let error = registry
            .add_user_font("Broken", b"not a font", FontFaceAttributes::regular())
            .expect_err("invalid font bytes should not enter the registry");

        assert_eq!(
            error,
            FontRegistryError::InvalidFont {
                family: "Broken".to_owned()
            }
        );
        assert!(registry.faces().is_empty());
    }
}
