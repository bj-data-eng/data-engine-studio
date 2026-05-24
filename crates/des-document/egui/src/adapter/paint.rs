use super::text::{layout_job, paint_document_text_selection};
use des_document::{
    BorderStyle, ClipRect, Color, CornerRadii, DocumentTextSelection, FloatingPlacement, Glyph,
    Insets, Rect, ResolvedElement, ScrollChrome, Shadow, TextLayoutRequest, TextLayoutResult,
    TextMeasurer, TextMeasurerKey, TextWrapMode,
};
use des_text::{
    CosmicTextRenderer, TextGlyph, TextGlyphCacheKey, TextGlyphImage, TextGlyphImageContent,
    TextGlyphRect, TextPaintRunId,
};
use eframe::egui;
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

pub struct CosmicTextPaintResources {
    renderer: CosmicTextRenderer,
    atlas: TextGlyphAtlas,
    mesh_cache: TextGlyphMeshCache,
    stats: TextPaintStats,
}

struct TextGlyphAtlas {
    pages: Vec<TextGlyphAtlasPage>,
    entries: HashMap<TextGlyphCacheKey, TextGlyphAtlasEntry>,
    size: usize,
}

struct TextGlyphAtlasPage {
    texture: egui::TextureHandle,
    cursor_x: usize,
    cursor_y: usize,
    row_height: usize,
}

struct TextGlyphMeshCache {
    entries: HashMap<TextGlyphMeshCacheKey, Vec<Arc<egui::epaint::Mesh>>>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct TextGlyphMeshCacheKey {
    run_id: TextPaintRunId,
    position_x: u32,
    position_y: u32,
    selection_start: Option<usize>,
    selection_end: Option<usize>,
    selection_color: [u8; 4],
}

#[derive(Clone, Copy, Debug)]
struct TextGlyphAtlasEntry {
    texture_id: egui::TextureId,
    uv: egui::Rect,
    size_px: [u32; 2],
    placement_px: [i32; 2],
    color_content: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TextPaintStats {
    pub measure_requests: usize,
    pub measure_time: Duration,
    pub hit_test_requests: usize,
    pub hit_test_time: Duration,
    pub paint_text_requests: usize,
    pub glyph_run_time: Duration,
    pub glyph_atlas_time: Duration,
    pub glyph_image_time: Duration,
    pub glyph_upload_time: Duration,
    pub glyph_paint_time: Duration,
    pub glyphs_painted: usize,
    pub glyph_meshes: usize,
    pub glyph_mesh_cache_hits: usize,
    pub glyph_mesh_cache_misses: usize,
    pub glyph_mesh_cache_entries: usize,
    pub glyph_cache_hits: usize,
    pub rasterizations: usize,
    pub cached_glyphs: usize,
    pub atlas_pages: usize,
    pub uploaded_pixels: u64,
    pub layout_cache_hits: usize,
    pub layout_cache_misses: usize,
    pub layout_cache_entries: usize,
    pub paint_run_cache_hits: usize,
    pub paint_run_cache_misses: usize,
    pub paint_run_cache_entries: usize,
}

impl CosmicTextPaintResources {
    pub fn new(renderer: CosmicTextRenderer) -> Self {
        Self {
            renderer,
            atlas: TextGlyphAtlas::new(),
            mesh_cache: TextGlyphMeshCache::new(),
            stats: TextPaintStats::default(),
        }
    }

    pub fn begin_frame(&mut self) {
        self.renderer.begin_frame();
        self.stats = TextPaintStats {
            cached_glyphs: self.atlas.entries.len(),
            atlas_pages: self.atlas.pages.len(),
            glyph_mesh_cache_entries: self.mesh_cache.entries.len(),
            layout_cache_entries: self.renderer.buffer_stats().cache_entries,
            paint_run_cache_entries: self.renderer.buffer_stats().paint_run_cache_entries,
            ..TextPaintStats::default()
        };
    }

    pub fn stats(&self) -> TextPaintStats {
        let layout_stats = self.renderer.buffer_stats();
        TextPaintStats {
            cached_glyphs: self.atlas.entries.len(),
            atlas_pages: self.atlas.pages.len(),
            glyph_mesh_cache_entries: self.mesh_cache.entries.len(),
            layout_cache_hits: layout_stats.cache_hits,
            layout_cache_misses: layout_stats.cache_misses,
            layout_cache_entries: layout_stats.cache_entries,
            paint_run_cache_hits: layout_stats.paint_run_cache_hits,
            paint_run_cache_misses: layout_stats.paint_run_cache_misses,
            paint_run_cache_entries: layout_stats.paint_run_cache_entries,
            ..self.stats
        }
    }
}

impl TextGlyphAtlas {
    #[cfg(not(test))]
    const SIZE: usize = 2048;
    #[cfg(test)]
    const SIZE: usize = 256;
    const PADDING: usize = 1;

    fn new() -> Self {
        Self {
            pages: Vec::new(),
            entries: HashMap::new(),
            size: Self::SIZE,
        }
    }

    fn add_page(&mut self, ctx: &egui::Context) -> usize {
        let page_index = self.pages.len();
        self.pages
            .push(TextGlyphAtlasPage::new(ctx, self.size, page_index));
        page_index
    }

    fn entry(
        &mut self,
        ctx: &egui::Context,
        renderer: &mut CosmicTextRenderer,
        cache_key: TextGlyphCacheKey,
        stats: &mut TextPaintStats,
    ) -> Option<TextGlyphAtlasEntry> {
        if let Some(entry) = self.entries.get(&cache_key).copied() {
            stats.glyph_cache_hits += 1;
            return Some(entry);
        }

        let image_start = Instant::now();
        let image = renderer.glyph_image(cache_key)?;
        stats.glyph_image_time += image_start.elapsed();
        let entry = self.insert(ctx, cache_key, &image, stats)?;
        Some(entry)
    }

    fn insert(
        &mut self,
        ctx: &egui::Context,
        cache_key: TextGlyphCacheKey,
        image: &TextGlyphImage,
        stats: &mut TextPaintStats,
    ) -> Option<TextGlyphAtlasEntry> {
        let entry = self.insert_image(ctx, image, stats)?;
        self.entries.insert(cache_key, entry);
        Some(entry)
    }

    fn insert_image(
        &mut self,
        ctx: &egui::Context,
        image: &TextGlyphImage,
        stats: &mut TextPaintStats,
    ) -> Option<TextGlyphAtlasEntry> {
        let width = image.width_px as usize;
        let height = image.height_px as usize;
        if width == 0 || height == 0 || width + Self::PADDING * 2 > self.size {
            return None;
        }

        if self.pages.is_empty() {
            self.add_page(ctx);
        }

        let mut page_index = self.pages.len() - 1;
        let allocation =
            if let Some(allocation) = self.pages[page_index].allocate(width, height, self.size) {
                allocation
            } else {
                page_index = self.add_page(ctx);
                self.pages[page_index].allocate(width, height, self.size)?
            };
        let (x, y) = allocation;
        let upload_start = Instant::now();
        self.pages[page_index].upload([x, y], [width, height], &image.rgba);
        stats.glyph_upload_time += upload_start.elapsed();

        let uv = egui::Rect::from_min_max(
            egui::pos2(x as f32 / self.size as f32, y as f32 / self.size as f32),
            egui::pos2(
                (x + width) as f32 / self.size as f32,
                (y + height) as f32 / self.size as f32,
            ),
        );
        let entry = TextGlyphAtlasEntry {
            texture_id: self.pages[page_index].texture.id(),
            uv,
            size_px: [image.width_px, image.height_px],
            placement_px: [image.left_px, image.top_px],
            color_content: image.content == TextGlyphImageContent::Color,
        };
        stats.rasterizations += 1;
        stats.uploaded_pixels += image.width_px as u64 * image.height_px as u64;
        Some(entry)
    }
}

impl TextGlyphAtlasPage {
    fn new(ctx: &egui::Context, size: usize, page_index: usize) -> Self {
        let image =
            egui::ColorImage::new([size, size], vec![egui::Color32::TRANSPARENT; size * size]);
        let texture = ctx.load_texture(
            format!("des-cosmic-glyph-atlas-{page_index}"),
            image,
            texture_options(),
        );
        Self {
            texture,
            cursor_x: TextGlyphAtlas::PADDING,
            cursor_y: TextGlyphAtlas::PADDING,
            row_height: 0,
        }
    }

    fn allocate(&mut self, width: usize, height: usize, size: usize) -> Option<(usize, usize)> {
        if self.cursor_x + width + TextGlyphAtlas::PADDING > size {
            self.cursor_x = TextGlyphAtlas::PADDING;
            self.cursor_y += self.row_height + TextGlyphAtlas::PADDING;
            self.row_height = 0;
        }
        if self.cursor_y + height + TextGlyphAtlas::PADDING > size {
            return None;
        }

        let x = self.cursor_x;
        let y = self.cursor_y;
        self.cursor_x += width + TextGlyphAtlas::PADDING;
        self.row_height = self.row_height.max(height);
        Some((x, y))
    }

    fn upload(&mut self, origin: [usize; 2], size: [usize; 2], rgba: &[u8]) {
        let color_image = egui::ColorImage::from_rgba_unmultiplied(size, rgba);
        self.texture
            .set_partial(origin, color_image, texture_options());
    }
}

impl TextGlyphMeshCache {
    const MAX_ENTRIES: usize = 1024;

    fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    fn get(&self, key: &TextGlyphMeshCacheKey) -> Option<&[Arc<egui::epaint::Mesh>]> {
        self.entries.get(key).map(Vec::as_slice)
    }

    fn insert(&mut self, key: TextGlyphMeshCacheKey, meshes: Vec<Arc<egui::epaint::Mesh>>) {
        if self.entries.len() > Self::MAX_ENTRIES {
            self.entries.clear();
        }
        self.entries.insert(key, meshes);
    }
}

impl TextGlyphMeshCacheKey {
    fn new(
        run_id: TextPaintRunId,
        position: egui::Pos2,
        selection: Option<&std::ops::Range<usize>>,
        selection_color: Color,
    ) -> Self {
        Self {
            run_id,
            position_x: f32_key(position.x),
            position_y: f32_key(position.y),
            selection_start: selection.map(|selection| selection.start),
            selection_end: selection.map(|selection| selection.end),
            selection_color: selection.map_or([0; 4], |_| color_key(selection_color)),
        }
    }
}

fn texture_options() -> egui::TextureOptions {
    egui::TextureOptions::LINEAR
}

impl TextMeasurer for CosmicTextPaintResources {
    fn cache_key(&self) -> TextMeasurerKey {
        self.renderer.cache_key()
    }

    fn measure_text(&mut self, request: TextLayoutRequest<'_>) -> TextLayoutResult {
        self.stats.measure_requests += 1;
        let start = Instant::now();
        let result = self.renderer.measure_text(request);
        self.stats.measure_time += start.elapsed();
        result
    }

    fn text_index_at(
        &mut self,
        request: TextLayoutRequest<'_>,
        point: des_document::Point,
    ) -> usize {
        self.stats.hit_test_requests += 1;
        let start = Instant::now();
        let result = self.renderer.text_index_at(request, point);
        self.stats.hit_test_time += start.elapsed();
        result
    }
}
pub fn paint_frame(
    ui: &mut egui::Ui,
    origin: egui::Pos2,
    frame: &ResolvedElement,
    text_selection: Option<&DocumentTextSelection>,
) {
    paint_frame_clipped(ui, origin, frame, ui.clip_rect(), text_selection, None);
}

pub fn paint_frame_with_text_resources(
    ui: &mut egui::Ui,
    origin: egui::Pos2,
    frame: &ResolvedElement,
    text_selection: Option<&DocumentTextSelection>,
    text_resources: &mut CosmicTextPaintResources,
) {
    paint_frame_clipped(
        ui,
        origin,
        frame,
        ui.clip_rect(),
        text_selection,
        Some(text_resources),
    );
}

pub fn paint_frame_with_text_renderer(
    ui: &mut egui::Ui,
    origin: egui::Pos2,
    frame: &ResolvedElement,
    text_selection: Option<&DocumentTextSelection>,
    text_renderer: &mut CosmicTextRenderer,
) {
    let mut resources = CosmicTextPaintResources::new(std::mem::replace(
        text_renderer,
        crate::document_text_renderer(),
    ));
    paint_frame_with_text_resources(ui, origin, frame, text_selection, &mut resources);
    *text_renderer = resources.renderer;
}

fn paint_frame_clipped(
    ui: &mut egui::Ui,
    origin: egui::Pos2,
    frame: &ResolvedElement,
    host_clip_rect: egui::Rect,
    text_selection: Option<&DocumentTextSelection>,
    mut text_resources: Option<&mut CosmicTextPaintResources>,
) {
    let clip_rect = apply_document_clip(origin, host_clip_rect, frame.clip_rect);
    let painter = ui.painter().with_clip_rect(clip_rect);
    if frame.id.as_str() != "root" {
        let rect = frame_rect(origin, frame);
        let self_visible = clip_rect.intersects(rect);

        if self_visible {
            if let Some(arrow) = floating_arrow(frame, rect) {
                paint_floating_surface(&painter, rect, frame, arrow);
            } else {
                paint_shadows(&painter, rect, frame.style.radius, &frame.style.shadows);

                if let Some(color) = frame.style.background {
                    painter.rect_filled(
                        rect,
                        to_egui_radius(frame.style.radius),
                        to_egui_color(color),
                    );
                }

                if let Some(color) = frame.style.border {
                    paint_border(
                        &painter,
                        rect,
                        frame.style.radius,
                        frame.style.border_width,
                        frame.style.border_style,
                        color,
                    );
                }
            }
        }

        if self_visible && let Some(normalized) = &frame.normalized_text {
            let text_rect = frame_content_rect(rect, frame);
            let request = TextLayoutRequest {
                text: normalized,
                font_size: frame.style.font_size,
                color: frame.style.text_color,
                direction: frame.style.direction,
                wrap_width: match frame.style.text_layout.text_wrap_mode {
                    TextWrapMode::NoWrap => f32::INFINITY,
                    TextWrapMode::Wrap => text_rect.width(),
                },
                layout_style: frame.style.text_layout,
                line_height: frame.style.line_height,
            };
            if let Some(resources) = text_resources.as_deref_mut() {
                if let Some(visible_rect) = visible_local_rect(clip_rect, text_rect) {
                    let selection = text_selection.and_then(|selection| {
                        (frame.selectable_text && selection.target == frame.id).then(|| {
                            let anchor =
                                normalized.semantic_to_layout_index(selection.anchor_index);
                            let focus = normalized.semantic_to_layout_index(selection.focus_index);
                            anchor.min(focus)..anchor.max(focus)
                        })
                    });
                    paint_atlas_text(
                        &painter,
                        text_rect.min,
                        visible_rect,
                        request,
                        selection,
                        frame.style.text_selection_background,
                        frame.style.text_selection_color,
                        ui.ctx().pixels_per_point(),
                        &mut resources.renderer,
                        &mut resources.atlas,
                        &mut resources.mesh_cache,
                        &mut resources.stats,
                    );
                }
            } else {
                let color = to_egui_color(frame.style.text_color);
                let mut galley = painter.layout_job(layout_job(request, color));
                if frame.selectable_text
                    && let Some(selection) = text_selection
                    && selection.target == frame.id
                {
                    let anchor_index = normalized.semantic_to_layout_index(selection.anchor_index);
                    let focus_index = normalized.semantic_to_layout_index(selection.focus_index);
                    let cursor_range = egui::text_selection::CCursorRange::two(
                        egui::text::CCursor::new(anchor_index),
                        egui::text::CCursor::new(focus_index),
                    );
                    paint_document_text_selection(
                        &mut galley,
                        &cursor_range,
                        frame.style.text_selection_background,
                        frame.style.text_selection_color,
                    );
                }
                painter.galley(text_rect.min, galley, color);
            }
        }

        if self_visible && let Some(glyph) = frame.glyph {
            paint_glyph(
                &painter,
                rect,
                glyph,
                frame.style.text_color,
                frame.style.font_size,
            );
        }
    }

    paint_children_clipped(
        ui,
        origin,
        frame,
        host_clip_rect,
        text_selection,
        text_resources,
    );
}

fn paint_children_clipped(
    ui: &mut egui::Ui,
    origin: egui::Pos2,
    frame: &ResolvedElement,
    host_clip_rect: egui::Rect,
    text_selection: Option<&DocumentTextSelection>,
    mut text_resources: Option<&mut CosmicTextPaintResources>,
) {
    match frame.children.as_slice() {
        [] => {}
        [child] => paint_frame_clipped(
            ui,
            origin,
            child,
            host_clip_rect,
            text_selection,
            text_resources,
        ),
        children => {
            if children_need_z_sort(children) {
                let mut sorted: Vec<_> = children.iter().collect();
                sorted.sort_by_key(|child| child.style.z_index);
                for child in sorted {
                    paint_frame_clipped(
                        ui,
                        origin,
                        child,
                        host_clip_rect,
                        text_selection,
                        text_resources.as_deref_mut(),
                    );
                }
            } else {
                for child in children {
                    paint_frame_clipped(
                        ui,
                        origin,
                        child,
                        host_clip_rect,
                        text_selection,
                        text_resources.as_deref_mut(),
                    );
                }
            }
        }
    }
}

fn children_need_z_sort(children: &[ResolvedElement]) -> bool {
    children
        .windows(2)
        .any(|pair| pair[0].style.z_index > pair[1].style.z_index)
}

fn paint_atlas_text(
    painter: &egui::Painter,
    position: egui::Pos2,
    visible_rect: Rect,
    request: TextLayoutRequest<'_>,
    selection: Option<std::ops::Range<usize>>,
    selection_background: Color,
    selection_color: Color,
    pixels_per_point: f32,
    renderer: &mut CosmicTextRenderer,
    atlas: &mut TextGlyphAtlas,
    mesh_cache: &mut TextGlyphMeshCache,
    stats: &mut TextPaintStats,
) {
    stats.paint_text_requests += 1;
    let glyph_run_start = Instant::now();
    let glyph_run = renderer.paint_glyphs(request.clone(), pixels_per_point, Some(visible_rect));
    stats.glyph_run_time += glyph_run_start.elapsed();
    let scale = pixels_per_point.max(1.0);
    if glyph_run.glyphs.is_empty() {
        return;
    }
    for background in &glyph_run.backgrounds {
        paint_text_rect(painter, position, *background, scale);
    }
    let selection = selection.filter(|selection| selection.start < selection.end);
    if let Some(selection) = selection.clone() {
        for rect in renderer.selection_rects(
            request.clone(),
            pixels_per_point,
            selection,
            selection_background,
        ) {
            paint_text_rect(painter, position, rect, scale);
        }
    }
    let atlas_start = Instant::now();
    let paint_start = Instant::now();
    paint_text_glyph_meshes(
        painter,
        position,
        glyph_run.id,
        &glyph_run.glyphs,
        selection.as_ref(),
        selection_color,
        scale,
        renderer,
        atlas,
        mesh_cache,
        stats,
    );
    stats.glyph_paint_time += paint_start.elapsed();
    stats.glyph_atlas_time += atlas_start.elapsed();
    for decoration in &glyph_run.decorations {
        paint_text_rect(painter, position, *decoration, scale);
    }
}

fn paint_text_glyph_meshes(
    painter: &egui::Painter,
    position: egui::Pos2,
    run_id: TextPaintRunId,
    glyphs: &[TextGlyph],
    selection: Option<&std::ops::Range<usize>>,
    selection_color: Color,
    scale: f32,
    renderer: &mut CosmicTextRenderer,
    atlas: &mut TextGlyphAtlas,
    mesh_cache: &mut TextGlyphMeshCache,
    stats: &mut TextPaintStats,
) {
    let cache_key = TextGlyphMeshCacheKey::new(run_id, position, selection, selection_color);
    if let Some(meshes) = mesh_cache.get(&cache_key) {
        stats.glyph_mesh_cache_hits += 1;
        stats.glyphs_painted += glyphs.len();
        stats.glyph_meshes += meshes.len();
        paint_cached_text_meshes(painter, meshes);
        return;
    }

    stats.glyph_mesh_cache_misses += 1;
    let mut meshes = Vec::new();
    let mut batch = TextGlyphBatch::default();
    for glyph in glyphs.iter().copied() {
        let Some(entry) = atlas.entry(painter.ctx(), renderer, glyph.cache_key, stats) else {
            continue;
        };
        stats.glyphs_painted += 1;
        let selected = selection.as_ref().is_some_and(|selection| {
            glyph.layout_start < selection.end && glyph.layout_end > selection.start
        });
        let glyph = if selected {
            TextGlyph {
                color: selection_color,
                ..glyph
            }
        } else {
            glyph
        };
        batch.push(position, glyph, entry, scale, &mut meshes);
    }
    batch.flush(&mut meshes);
    stats.glyph_meshes += meshes.len();
    paint_cached_text_meshes(painter, &meshes);
    mesh_cache.insert(cache_key, meshes);
}

fn paint_cached_text_meshes(painter: &egui::Painter, meshes: &[Arc<egui::epaint::Mesh>]) {
    for mesh in meshes {
        painter.add(egui::epaint::Shape::Mesh(mesh.clone()));
    }
}

fn visible_local_rect(clip_rect: egui::Rect, text_rect: egui::Rect) -> Option<Rect> {
    let visible = clip_rect.intersect(text_rect);
    (visible.width() > 0.0 && visible.height() > 0.0).then(|| {
        Rect::new(
            visible.left() - text_rect.left(),
            visible.top() - text_rect.top(),
            visible.width(),
            visible.height(),
        )
    })
}

#[derive(Default)]
struct TextGlyphBatch {
    mesh: Option<egui::epaint::Mesh>,
}

impl TextGlyphBatch {
    fn push(
        &mut self,
        text_position: egui::Pos2,
        glyph: TextGlyph,
        entry: TextGlyphAtlasEntry,
        scale: f32,
        meshes: &mut Vec<Arc<egui::epaint::Mesh>>,
    ) {
        let texture_id = entry.texture_id;
        if self
            .mesh
            .as_ref()
            .is_some_and(|mesh| mesh.texture_id != texture_id)
        {
            self.flush(meshes);
        }
        let mesh = self
            .mesh
            .get_or_insert_with(|| egui::epaint::Mesh::with_texture(texture_id));
        let min = egui::pos2(
            text_position.x + (glyph.x_px + entry.placement_px[0]) as f32 / scale,
            text_position.y + (glyph.y_px - entry.placement_px[1]) as f32 / scale,
        );
        let size = egui::vec2(
            entry.size_px[0] as f32 / scale,
            entry.size_px[1] as f32 / scale,
        );
        let tint = if entry.color_content {
            egui::Color32::WHITE
        } else {
            to_egui_color(glyph.color)
        };
        mesh.add_rect_with_uv(egui::Rect::from_min_size(min, size), entry.uv, tint);
    }

    fn flush(&mut self, meshes: &mut Vec<Arc<egui::epaint::Mesh>>) {
        let Some(mesh) = self.mesh.take().filter(|mesh| !mesh.is_empty()) else {
            return;
        };
        meshes.push(Arc::new(mesh));
    }
}

fn paint_text_rect(
    painter: &egui::Painter,
    text_position: egui::Pos2,
    rect: TextGlyphRect,
    scale: f32,
) {
    let min = egui::pos2(
        text_position.x + rect.x_px as f32 / scale,
        text_position.y + rect.y_px as f32 / scale,
    );
    let size = egui::vec2(rect.width_px as f32 / scale, rect.height_px as f32 / scale);
    painter.rect_filled(
        egui::Rect::from_min_size(min, size),
        0.0,
        to_egui_color(rect.color),
    );
}

fn apply_document_clip(
    origin: egui::Pos2,
    base_clip: egui::Rect,
    document_clip: ClipRect,
) -> egui::Rect {
    let min_x = document_clip.left.map_or(base_clip.left(), |left| {
        base_clip.left().max(origin.x + left)
    });
    let min_y = document_clip
        .top
        .map_or(base_clip.top(), |top| base_clip.top().max(origin.y + top));
    let max_x = document_clip
        .right
        .map_or(base_clip.right(), |right| {
            base_clip.right().min(origin.x + right)
        })
        .max(min_x);
    let max_y = document_clip
        .bottom
        .map_or(base_clip.bottom(), |bottom| {
            base_clip.bottom().min(origin.y + bottom)
        })
        .max(min_y);

    egui::Rect::from_min_max(egui::pos2(min_x, min_y), egui::pos2(max_x, max_y))
}

#[cfg(test)]
mod clip_tests {
    use super::*;

    #[test]
    fn document_clip_can_constrain_one_axis_without_clipping_the_other() {
        let base = egui::Rect::from_min_max(egui::pos2(10.0, 20.0), egui::pos2(210.0, 220.0));

        let clipped = apply_document_clip(
            egui::pos2(10.0, 20.0),
            base,
            ClipRect {
                left: Some(0.0),
                right: Some(80.0),
                top: None,
                bottom: None,
            },
        );

        assert_eq!(
            clipped,
            egui::Rect::from_min_max(egui::pos2(10.0, 20.0), egui::pos2(90.0, 220.0))
        );
    }

    #[test]
    fn document_clip_collapses_empty_intersections() {
        let base = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(100.0, 100.0));

        let clipped = apply_document_clip(
            egui::pos2(0.0, 0.0),
            base,
            ClipRect {
                left: Some(120.0),
                right: Some(140.0),
                top: None,
                bottom: None,
            },
        );

        assert_eq!(clipped.width(), 0.0);
        assert_eq!(clipped.height(), 100.0);
    }
}

fn frame_rect(origin: egui::Pos2, frame: &ResolvedElement) -> egui::Rect {
    document_rect_to_egui(origin, frame.rect)
}

pub fn paint_surface(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    radius: CornerRadii,
    shadows: &[Shadow],
    background: Color,
    border: Option<Color>,
    border_width: Insets,
) {
    let painter = ui.painter();
    paint_shadows(painter, rect, radius, shadows);
    painter.rect_filled(rect, to_egui_radius(radius), to_egui_color(background));
    if let Some(border) = border {
        paint_border(
            painter,
            rect,
            radius,
            border_width,
            BorderStyle::Solid,
            border,
        );
    }
}

fn frame_content_rect(rect: egui::Rect, frame: &ResolvedElement) -> egui::Rect {
    let min = egui::pos2(
        rect.left() + frame.style.border_width.left + frame.style.padding.left,
        rect.top() + frame.style.border_width.top + frame.style.padding.top,
    );
    let max = egui::pos2(
        (rect.right() - frame.style.border_width.right - frame.style.padding.right).max(min.x),
        (rect.bottom() - frame.style.border_width.bottom - frame.style.padding.bottom).max(min.y),
    );
    egui::Rect::from_min_max(min, max)
}

fn paint_glyph(painter: &egui::Painter, rect: egui::Rect, glyph: Glyph, color: Color, size: f32) {
    let color = to_egui_color(color);
    let stroke = egui::Stroke::new((size / 8.0).clamp(1.25, 2.5), color);
    let center = rect.center();
    let half = (size.min(rect.width()).min(rect.height()) / 2.0).max(1.0);
    match glyph {
        Glyph::Check => {
            let a = egui::pos2(center.x - half * 0.55, center.y - half * 0.05);
            let b = egui::pos2(center.x - half * 0.15, center.y + half * 0.38);
            let c = egui::pos2(center.x + half * 0.58, center.y - half * 0.42);
            painter.line_segment([a, b], stroke);
            painter.line_segment([b, c], stroke);
        }
        Glyph::ChevronDown => {
            let a = egui::pos2(center.x - half * 0.5, center.y - half * 0.2);
            let b = egui::pos2(center.x, center.y + half * 0.32);
            let c = egui::pos2(center.x + half * 0.5, center.y - half * 0.2);
            painter.line_segment([a, b], stroke);
            painter.line_segment([b, c], stroke);
        }
        Glyph::ChevronUp => {
            let a = egui::pos2(center.x - half * 0.5, center.y + half * 0.2);
            let b = egui::pos2(center.x, center.y - half * 0.32);
            let c = egui::pos2(center.x + half * 0.5, center.y + half * 0.2);
            painter.line_segment([a, b], stroke);
            painter.line_segment([b, c], stroke);
        }
        Glyph::DragHandle => {
            let radius = (size / 18.0).clamp(1.0, 1.7);
            let spacing_x = (size * 0.18).max(3.0);
            let spacing_y = (size * 0.24).max(4.0);
            for column in [-0.5_f32, 0.5] {
                for row in [-1.0_f32, 0.0, 1.0] {
                    painter.circle_filled(
                        egui::pos2(center.x + spacing_x * column, center.y + spacing_y * row),
                        radius,
                        color,
                    );
                }
            }
        }
    }
}

fn paint_border(
    painter: &egui::Painter,
    rect: egui::Rect,
    radius: CornerRadii,
    widths: Insets,
    style: BorderStyle,
    color: Color,
) {
    let color = to_egui_color(color);
    if style != BorderStyle::Solid {
        paint_segmented_border(painter, rect, widths, style, color);
        return;
    }
    if widths.is_uniform() {
        if widths.top > 0.0 {
            painter.rect_stroke(
                rect,
                to_egui_radius(radius),
                egui::Stroke::new(widths.top, color),
                egui::StrokeKind::Inside,
            );
        }
        return;
    }

    if widths.top > 0.0 {
        painter.rect_filled(
            egui::Rect::from_min_max(
                rect.left_top(),
                egui::pos2(rect.right(), rect.top() + widths.top),
            ),
            0.0,
            color,
        );
    }
    if widths.right > 0.0 {
        painter.rect_filled(
            egui::Rect::from_min_max(
                egui::pos2(rect.right() - widths.right, rect.top()),
                rect.right_bottom(),
            ),
            0.0,
            color,
        );
    }
    if widths.bottom > 0.0 {
        painter.rect_filled(
            egui::Rect::from_min_max(
                egui::pos2(rect.left(), rect.bottom() - widths.bottom),
                rect.right_bottom(),
            ),
            0.0,
            color,
        );
    }
    if widths.left > 0.0 {
        painter.rect_filled(
            egui::Rect::from_min_max(
                rect.left_top(),
                egui::pos2(rect.left() + widths.left, rect.bottom()),
            ),
            0.0,
            color,
        );
    }
}

fn paint_segmented_border(
    painter: &egui::Painter,
    rect: egui::Rect,
    widths: Insets,
    style: BorderStyle,
    color: egui::Color32,
) {
    let width = widths
        .top
        .max(widths.right)
        .max(widths.bottom)
        .max(widths.left);
    if width <= 0.0 {
        return;
    }
    let stroke = egui::Stroke::new(width, color);
    match style {
        BorderStyle::Solid => {}
        BorderStyle::Dashed => {
            let dash = (width * 3.0).max(6.0);
            let gap = (width * 2.0).max(4.0);
            paint_corner_preserved_dashed_border(painter, rect, dash, gap, stroke);
        }
        BorderStyle::Dotted => {
            let radius = (width * 0.5).max(1.0);
            let gap = (width * 3.0).max(6.0);
            paint_corner_preserved_dotted_border(painter, rect, radius, gap, color);
        }
    }
}

fn paint_corner_preserved_dashed_border(
    painter: &egui::Painter,
    rect: egui::Rect,
    dash: f32,
    gap: f32,
    stroke: egui::Stroke,
) {
    let corner_x = dash.min(rect.width() * 0.5);
    let corner_y = dash.min(rect.height() * 0.5);
    paint_dashed_corner_segments(painter, rect, corner_x, corner_y, stroke);
    paint_distributed_dashes(
        painter,
        rect.left_top() + egui::vec2(corner_x + gap, 0.0),
        rect.right_top() - egui::vec2(corner_x + gap, 0.0),
        dash,
        gap,
        stroke,
    );
    paint_distributed_dashes(
        painter,
        rect.right_top() + egui::vec2(0.0, corner_y + gap),
        rect.right_bottom() - egui::vec2(0.0, corner_y + gap),
        dash,
        gap,
        stroke,
    );
    paint_distributed_dashes(
        painter,
        rect.right_bottom() - egui::vec2(corner_x + gap, 0.0),
        rect.left_bottom() + egui::vec2(corner_x + gap, 0.0),
        dash,
        gap,
        stroke,
    );
    paint_distributed_dashes(
        painter,
        rect.left_bottom() - egui::vec2(0.0, corner_y + gap),
        rect.left_top() + egui::vec2(0.0, corner_y + gap),
        dash,
        gap,
        stroke,
    );
}

fn paint_dashed_corner_segments(
    painter: &egui::Painter,
    rect: egui::Rect,
    corner_x: f32,
    corner_y: f32,
    stroke: egui::Stroke,
) {
    for (corner, horizontal, vertical) in [
        (
            rect.left_top(),
            egui::vec2(corner_x, 0.0),
            egui::vec2(0.0, corner_y),
        ),
        (
            rect.right_top(),
            egui::vec2(-corner_x, 0.0),
            egui::vec2(0.0, corner_y),
        ),
        (
            rect.right_bottom(),
            egui::vec2(-corner_x, 0.0),
            egui::vec2(0.0, -corner_y),
        ),
        (
            rect.left_bottom(),
            egui::vec2(corner_x, 0.0),
            egui::vec2(0.0, -corner_y),
        ),
    ] {
        painter.add(egui::Shape::line(
            vec![corner + horizontal, corner, corner + vertical],
            stroke,
        ));
    }
}

fn paint_corner_preserved_dotted_border(
    painter: &egui::Painter,
    rect: egui::Rect,
    radius: f32,
    gap: f32,
    color: egui::Color32,
) {
    for corner in [
        rect.left_top(),
        rect.right_top(),
        rect.right_bottom(),
        rect.left_bottom(),
    ] {
        painter.circle_filled(corner, radius, color);
    }
    let corner_gap = gap.max(radius * 3.0);
    paint_dotted_segment(
        painter,
        rect.left_top() + egui::vec2(corner_gap, 0.0),
        rect.right_top() - egui::vec2(corner_gap, 0.0),
        radius,
        gap,
        color,
    );
    paint_dotted_segment(
        painter,
        rect.right_top() + egui::vec2(0.0, corner_gap),
        rect.right_bottom() - egui::vec2(0.0, corner_gap),
        radius,
        gap,
        color,
    );
    paint_dotted_segment(
        painter,
        rect.right_bottom() - egui::vec2(corner_gap, 0.0),
        rect.left_bottom() + egui::vec2(corner_gap, 0.0),
        radius,
        gap,
        color,
    );
    paint_dotted_segment(
        painter,
        rect.left_bottom() - egui::vec2(0.0, corner_gap),
        rect.left_top() + egui::vec2(0.0, corner_gap),
        radius,
        gap,
        color,
    );
}

fn paint_distributed_dashes(
    painter: &egui::Painter,
    start: egui::Pos2,
    end: egui::Pos2,
    preferred_dash: f32,
    preferred_gap: f32,
    stroke: egui::Stroke,
) {
    let vector = end - start;
    let length = vector.length();
    if length <= f32::EPSILON {
        return;
    }
    let pattern = distributed_dash_pattern(length, preferred_dash, preferred_gap);
    let direction = vector / length;
    let mut cursor = pattern.leading_gap;
    for _ in 0..pattern.count {
        painter.line_segment(
            [
                start + direction * cursor,
                start + direction * (cursor + pattern.dash),
            ],
            stroke,
        );
        cursor += pattern.dash + pattern.gap;
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct DistributedDashPattern {
    count: usize,
    dash: f32,
    gap: f32,
    leading_gap: f32,
}

fn distributed_dash_pattern(
    length: f32,
    preferred_dash: f32,
    preferred_gap: f32,
) -> DistributedDashPattern {
    if length <= preferred_dash {
        return DistributedDashPattern {
            count: 1,
            dash: length.max(0.0),
            gap: 0.0,
            leading_gap: 0.0,
        };
    }
    let pattern = preferred_dash + preferred_gap;
    let count = ((length + preferred_gap) / pattern).floor().max(1.0) as usize;
    if count == 1 {
        return DistributedDashPattern {
            count,
            dash: preferred_dash.min(length),
            gap: 0.0,
            leading_gap: ((length - preferred_dash).max(0.0)) * 0.5,
        };
    }
    let used_dash = preferred_dash * count as f32;
    let remaining = (length - used_dash).max(0.0);
    DistributedDashPattern {
        count,
        dash: preferred_dash,
        gap: remaining / (count - 1) as f32,
        leading_gap: 0.0,
    }
}

fn paint_dotted_segment(
    painter: &egui::Painter,
    start: egui::Pos2,
    end: egui::Pos2,
    radius: f32,
    gap: f32,
    color: egui::Color32,
) {
    let vector = end - start;
    let length = vector.length();
    if length <= f32::EPSILON {
        return;
    }
    let direction = vector / length;
    let mut cursor = 0.0;
    while cursor <= length {
        painter.circle_filled(start + direction * cursor, radius, color);
        cursor += gap;
    }
}

fn paint_shadows(
    painter: &egui::Painter,
    rect: egui::Rect,
    radius: CornerRadii,
    shadows: &[Shadow],
) {
    for shadow in shadows.iter().rev().copied() {
        paint_shadow(painter, rect, radius, shadow);
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct FloatingArrowPaint {
    points: [egui::Pos2; 3],
}

fn floating_arrow(frame: &ResolvedElement, rect: egui::Rect) -> Option<FloatingArrowPaint> {
    let floating = frame.floating?;
    let offset = floating.arrow_offset?;
    let size = floating.arrow_size?;
    Some(FloatingArrowPaint {
        points: floating_arrow_points(
            rect,
            floating.placement,
            offset.x,
            offset.y,
            size.width,
            size.height,
        ),
    })
}

fn floating_arrow_points(
    rect: egui::Rect,
    placement: FloatingPlacement,
    offset_x: f32,
    offset_y: f32,
    width: f32,
    height: f32,
) -> [egui::Pos2; 3] {
    match placement {
        FloatingPlacement::Center => {
            let center = rect.center();
            [center, center, center]
        }
        FloatingPlacement::Bottom
        | FloatingPlacement::BottomStart
        | FloatingPlacement::BottomEnd => {
            let left = rect.left() + offset_x;
            let center = left + width * 0.5;
            [
                egui::pos2(left, rect.top()),
                egui::pos2(left + width, rect.top()),
                egui::pos2(center, rect.top() - height),
            ]
        }
        FloatingPlacement::Top | FloatingPlacement::TopStart | FloatingPlacement::TopEnd => {
            let left = rect.left() + offset_x;
            let center = left + width * 0.5;
            [
                egui::pos2(left + width, rect.bottom()),
                egui::pos2(left, rect.bottom()),
                egui::pos2(center, rect.bottom() + height),
            ]
        }
        FloatingPlacement::Right | FloatingPlacement::RightStart | FloatingPlacement::RightEnd => {
            let top = rect.top() + offset_y;
            let center = top + height * 0.5;
            [
                egui::pos2(rect.left(), top + height),
                egui::pos2(rect.left(), top),
                egui::pos2(rect.left() - width, center),
            ]
        }
        FloatingPlacement::Left | FloatingPlacement::LeftStart | FloatingPlacement::LeftEnd => {
            let top = rect.top() + offset_y;
            let center = top + height * 0.5;
            [
                egui::pos2(rect.right(), top),
                egui::pos2(rect.right(), top + height),
                egui::pos2(rect.right() + width, center),
            ]
        }
    }
}

fn paint_floating_surface(
    painter: &egui::Painter,
    rect: egui::Rect,
    frame: &ResolvedElement,
    arrow: FloatingArrowPaint,
) {
    paint_shadows_with_arrow(
        painter,
        rect,
        frame.style.radius,
        &frame.style.shadows,
        arrow,
    );
    if let Some(color) = frame.style.background {
        painter.rect_filled(
            rect,
            to_egui_radius(frame.style.radius),
            to_egui_color(color),
        );
        painter.add(egui::Shape::convex_polygon(
            arrow.points.to_vec(),
            to_egui_color(color),
            egui::Stroke::NONE,
        ));
    }
    if let Some(color) = frame.style.border {
        paint_border(
            painter,
            rect,
            frame.style.radius,
            frame.style.border_width,
            frame.style.border_style,
            color,
        );
        paint_arrow_border(painter, arrow, frame.style.border_width, color);
    }
}

fn paint_shadows_with_arrow(
    painter: &egui::Painter,
    rect: egui::Rect,
    radius: CornerRadii,
    shadows: &[Shadow],
    arrow: FloatingArrowPaint,
) {
    for shadow in shadows.iter().rev().copied() {
        paint_shadow(painter, rect, radius, shadow);
        paint_arrow_shadow(painter, arrow, shadow);
    }
}

fn paint_arrow_shadow(painter: &egui::Painter, arrow: FloatingArrowPaint, shadow: Shadow) {
    if shadow.color.a == 0 {
        return;
    }
    let translated = translate_arrow(arrow, shadow.offset.x, shadow.offset.y);
    if shadow.blur <= 0.0 {
        painter.add(egui::Shape::convex_polygon(
            expanded_arrow(translated, shadow.spread).points.to_vec(),
            to_egui_color(shadow.color),
            egui::Stroke::NONE,
        ));
        return;
    }

    let sigma = shadow.blur * 0.5;
    let max_blur_extent = sigma * 3.0;
    let steps = max_blur_extent.ceil().clamp(10.0, 36.0) as usize;
    for step in (0..steps).rev() {
        let outer_distance = max_blur_extent * (step + 1) as f32 / steps as f32;
        let inner_distance = max_blur_extent * step as f32 / steps as f32;
        let outer_alpha = gaussian_alpha(outer_distance, sigma);
        let inner_alpha = gaussian_alpha(inner_distance, sigma);
        let alpha_portion = (inner_alpha - outer_alpha).max(0.0);
        let alpha = (shadow.color.a as f32 * alpha_portion * 0.86)
            .round()
            .clamp(0.0, 255.0) as u8;
        if alpha == 0 {
            continue;
        }
        let color = Color {
            a: alpha,
            ..shadow.color
        };
        painter.add(egui::Shape::convex_polygon(
            expanded_arrow(translated, shadow.spread + outer_distance)
                .points
                .to_vec(),
            to_egui_color(color),
            egui::Stroke::NONE,
        ));
    }
}

fn translate_arrow(arrow: FloatingArrowPaint, x: f32, y: f32) -> FloatingArrowPaint {
    FloatingArrowPaint {
        points: arrow.points.map(|point| point + egui::vec2(x, y)),
    }
}

fn expanded_arrow(arrow: FloatingArrowPaint, amount: f32) -> FloatingArrowPaint {
    let center = egui::pos2(
        (arrow.points[0].x + arrow.points[1].x + arrow.points[2].x) / 3.0,
        (arrow.points[0].y + arrow.points[1].y + arrow.points[2].y) / 3.0,
    );
    FloatingArrowPaint {
        points: arrow.points.map(|point| {
            let vector = point - center;
            let length = vector.length();
            if length <= f32::EPSILON {
                point
            } else {
                center + vector * ((length + amount).max(0.0) / length)
            }
        }),
    }
}

fn paint_arrow_border(
    painter: &egui::Painter,
    arrow: FloatingArrowPaint,
    widths: Insets,
    color: Color,
) {
    let width = widths
        .top
        .max(widths.right)
        .max(widths.bottom)
        .max(widths.left);
    if width <= 0.0 {
        return;
    }
    painter.add(egui::Shape::closed_line(
        arrow.points.to_vec(),
        egui::Stroke::new(width, to_egui_color(color)),
    ));
}

fn paint_shadow(painter: &egui::Painter, rect: egui::Rect, radius: CornerRadii, shadow: Shadow) {
    if shadow.color.a == 0 {
        return;
    }

    let base_rect = expand_rect_safely(
        rect.translate(egui::vec2(shadow.offset.x, shadow.offset.y)),
        shadow.spread,
    );
    if shadow.blur <= 0.0 {
        painter.rect_filled(
            base_rect,
            to_egui_radius(expand_radius(radius, shadow.spread)),
            to_egui_color(shadow.color),
        );
        return;
    }

    let sigma = shadow.blur * 0.5;
    let max_blur_extent = sigma * 3.0;
    let steps = max_blur_extent.ceil().clamp(10.0, 36.0) as usize;
    for step in (0..steps).rev() {
        let outer_distance = max_blur_extent * (step + 1) as f32 / steps as f32;
        let inner_distance = max_blur_extent * step as f32 / steps as f32;
        let outer_alpha = gaussian_alpha(outer_distance, sigma);
        let inner_alpha = gaussian_alpha(inner_distance, sigma);
        let alpha_portion = (inner_alpha - outer_alpha).max(0.0);
        let alpha = (shadow.color.a as f32 * alpha_portion * 0.86)
            .round()
            .clamp(0.0, 255.0) as u8;
        if alpha == 0 {
            continue;
        }
        let color = Color {
            a: alpha,
            ..shadow.color
        };
        let expansion = shadow.spread + outer_distance;
        painter.rect_filled(
            expand_rect_safely(
                rect.translate(egui::vec2(shadow.offset.x, shadow.offset.y)),
                expansion,
            ),
            to_egui_radius(expand_radius(radius, expansion)),
            to_egui_color(color),
        );
    }
}

fn expand_radius(radius: CornerRadii, amount: f32) -> CornerRadii {
    CornerRadii {
        top_left: (radius.top_left + amount).max(0.0),
        top_right: (radius.top_right + amount).max(0.0),
        bottom_right: (radius.bottom_right + amount).max(0.0),
        bottom_left: (radius.bottom_left + amount).max(0.0),
    }
}

fn expand_rect_safely(rect: egui::Rect, amount: f32) -> egui::Rect {
    if amount >= 0.0 {
        return rect.expand(amount);
    }
    let inset = (-amount).min(rect.width() * 0.5).min(rect.height() * 0.5);
    rect.shrink(inset)
}

fn gaussian_alpha(distance: f32, sigma: f32) -> f32 {
    if sigma <= 0.0 {
        return 1.0;
    }
    (-0.5 * (distance / sigma).powi(2)).exp()
}

pub fn paint_scroll_chrome(ui: &mut egui::Ui, origin: egui::Pos2, chromes: &[ScrollChrome]) {
    let painter = ui.painter();
    for chrome in chromes {
        if !chrome.visible {
            continue;
        }

        let track = document_rect_to_egui(origin, chrome.track_rect);
        let handle = document_rect_to_egui(origin, chrome.handle_rect);
        if let Some(track_color) = chrome.track_color {
            painter.rect_filled(track, chrome.radius, to_egui_color(track_color));
        }
        painter.rect_filled(handle, chrome.radius, to_egui_color(chrome.handle_color));
        if let Some(border_color) = chrome.handle_border_color
            && chrome.handle_border_width > 0.0
        {
            painter.rect_stroke(
                handle,
                chrome.radius,
                egui::Stroke::new(chrome.handle_border_width, to_egui_color(border_color)),
                egui::StrokeKind::Inside,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use des_document::{ComputedStyle, Element, ElementId};

    fn resolved_frame(id: &str, z_index: i32, background: Color) -> ResolvedElement {
        let mut style = ComputedStyle {
            background: Some(background),
            z_index,
            ..ComputedStyle::default()
        };
        style.radius = CornerRadii::ZERO;
        ResolvedElement {
            id: ElementId::new(id),
            element: Element::Div,
            classes: Vec::new(),
            rect: Rect::new(0.0, 0.0, 20.0, 20.0),
            clip_rect: ClipRect::from_rect(Rect::new(0.0, 0.0, 20.0, 20.0)),
            style,
            text: None,
            normalized_text: None,
            text_layout: None,
            selectable_text: false,
            copyable_text: false,
            value: None,
            glyph: None,
            interactive: false,
            floating: None,
            children: Vec::new(),
        }
    }

    #[test]
    fn floating_arrow_points_attach_to_opposite_side_of_placement() {
        let rect = egui::Rect::from_min_size(egui::pos2(20.0, 30.0), egui::vec2(80.0, 40.0));

        let bottom = floating_arrow_points(rect, FloatingPlacement::Bottom, 30.0, 0.0, 12.0, 6.0);
        assert_eq!(bottom[0], egui::pos2(50.0, 30.0));
        assert_eq!(bottom[1], egui::pos2(62.0, 30.0));
        assert_eq!(bottom[2], egui::pos2(56.0, 24.0));

        let right = floating_arrow_points(rect, FloatingPlacement::Right, 0.0, 10.0, 6.0, 12.0);
        assert_eq!(right[0], egui::pos2(20.0, 52.0));
        assert_eq!(right[1], egui::pos2(20.0, 40.0));
        assert_eq!(right[2], egui::pos2(14.0, 46.0));
    }

    #[test]
    fn expanded_arrow_keeps_center_and_moves_points_outward() {
        let arrow = FloatingArrowPaint {
            points: [
                egui::pos2(0.0, 0.0),
                egui::pos2(10.0, 0.0),
                egui::pos2(5.0, -5.0),
            ],
        };

        let expanded = expanded_arrow(arrow, 2.0);

        assert!(expanded.points[0].x < arrow.points[0].x);
        assert!(expanded.points[1].x > arrow.points[1].x);
        assert!(expanded.points[2].y < arrow.points[2].y);
    }

    #[test]
    fn distributed_dash_pattern_avoids_cutoff_dash() {
        let pattern = distributed_dash_pattern(52.0, 8.0, 5.0);

        assert_eq!(pattern.count, 4);
        assert_eq!(pattern.dash, 8.0);
        assert_eq!(pattern.gap, (52.0 - 32.0) / 3.0);
        assert_eq!(
            pattern.dash * pattern.count as f32 + pattern.gap * (pattern.count - 1) as f32,
            52.0
        );
    }

    #[test]
    fn distributed_dash_pattern_centers_single_dash() {
        let pattern = distributed_dash_pattern(18.0, 12.0, 8.0);

        assert_eq!(
            pattern,
            DistributedDashPattern {
                count: 1,
                dash: 12.0,
                gap: 0.0,
                leading_gap: 3.0,
            }
        );
    }

    #[test]
    fn glyph_atlas_adds_pages_instead_of_dropping_when_full() {
        let ctx = egui::Context::default();
        let mut atlas = TextGlyphAtlas::new();
        let mut stats = TextPaintStats::default();
        let image = TextGlyphImage {
            width_px: 80,
            height_px: 80,
            left_px: 0,
            top_px: 0,
            content: TextGlyphImageContent::Mask,
            rgba: vec![255; 80 * 80 * 4],
        };

        for _ in 0..16 {
            assert!(
                atlas.insert_image(&ctx, &image, &mut stats).is_some(),
                "atlas should allocate another page instead of dropping a fitting glyph"
            );
        }

        assert!(
            atlas.pages.len() > 1,
            "test-sized atlas should grow to multiple pages"
        );
        assert_eq!(stats.rasterizations, 16);
    }

    #[test]
    fn child_paint_order_skips_z_sort_when_already_ordered() {
        let children = vec![
            resolved_frame("back", -1, Color::rgb(255, 0, 0)),
            resolved_frame("middle", 0, Color::rgb(0, 0, 255)),
            resolved_frame("front", 10, Color::rgb(0, 255, 0)),
        ];

        assert!(!children_need_z_sort(&children));
    }

    #[test]
    fn child_paint_order_detects_unordered_z_index() {
        let children = vec![
            resolved_frame("front", 10, Color::rgb(0, 255, 0)),
            resolved_frame("back", -1, Color::rgb(255, 0, 0)),
        ];

        assert!(children_need_z_sort(&children));
    }

    #[test]
    fn paint_frame_emits_children_in_z_index_order() {
        let mut root = resolved_frame("root", 0, Color::rgb(0, 0, 0));
        root.children = vec![
            resolved_frame("front", 10, Color::rgb(0, 255, 0)),
            resolved_frame("back", -1, Color::rgb(255, 0, 0)),
        ];
        let ctx = egui::Context::default();

        let output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(40.0, 40.0),
                )),
                ..Default::default()
            },
            |ui| paint_frame(ui, egui::Pos2::ZERO, &root, None),
        );

        let fills: Vec<_> = output
            .shapes
            .iter()
            .filter_map(|shape| match &shape.shape {
                egui::epaint::Shape::Rect(rect) if rect.fill != egui::Color32::TRANSPARENT => {
                    Some(rect.fill)
                }
                _ => None,
            })
            .collect();
        assert_eq!(
            fills,
            vec![
                egui::Color32::from_rgb(255, 0, 0),
                egui::Color32::from_rgb(0, 255, 0),
            ]
        );
    }
}

fn document_rect_to_egui(origin: egui::Pos2, rect: Rect) -> egui::Rect {
    egui::Rect::from_min_size(
        egui::pos2(origin.x + rect.origin.x, origin.y + rect.origin.y),
        egui::vec2(rect.size.width, rect.size.height),
    )
}

fn to_egui_color(color: Color) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(color.r, color.g, color.b, color.a)
}

fn color_key(color: Color) -> [u8; 4] {
    [color.r, color.g, color.b, color.a]
}

fn f32_key(value: f32) -> u32 {
    if value == 0.0 {
        0
    } else if value.is_nan() {
        f32::NAN.to_bits()
    } else {
        value.to_bits()
    }
}

fn to_egui_radius(radius: CornerRadii) -> egui::CornerRadius {
    egui::CornerRadius {
        nw: radius.top_left.round().clamp(0.0, 255.0) as u8,
        ne: radius.top_right.round().clamp(0.0, 255.0) as u8,
        se: radius.bottom_right.round().clamp(0.0, 255.0) as u8,
        sw: radius.bottom_left.round().clamp(0.0, 255.0) as u8,
    }
}
