use des_document::{
    Color, DocumentOutput, InlineTextStyle, Point, Size, TextDecoration, TextLayoutLine,
    TextLayoutRequest, TextLayoutResult, TextMeasurer, TextMeasurerKey, TextWrapMode,
};
use eframe::egui;
use std::{sync::Arc, time::Duration};
pub const TEXT_SELECTION_CLICK_INTERVAL: Duration = Duration::from_millis(800);
pub const TEXT_SELECTION_CLICK_DISTANCE: f32 = 6.0;

pub struct EguiTextMeasurer {
    ctx: egui::Context,
}

impl EguiTextMeasurer {
    pub fn new(ctx: &egui::Context) -> Self {
        Self { ctx: ctx.clone() }
    }
}

impl TextMeasurer for EguiTextMeasurer {
    fn cache_key(&self) -> TextMeasurerKey {
        TextMeasurerKey::new("egui")
    }

    fn measure_text(&mut self, request: TextLayoutRequest<'_>) -> TextLayoutResult {
        let galley = self
            .ctx
            .fonts_mut(|fonts| fonts.layout_job(layout_job(request.clone(), egui::Color32::WHITE)));
        let size = galley.size();
        let lines = galley_lines(&request, &galley);
        TextLayoutResult {
            size: Size::new(size.x, size.y),
            line_count: galley.rows.len(),
            elided: galley.elided,
            lines,
        }
    }

    fn text_index_at(&mut self, request: TextLayoutRequest<'_>, point: Point) -> usize {
        let galley = self
            .ctx
            .fonts_mut(|fonts| fonts.layout_job(layout_job(request.clone(), egui::Color32::WHITE)));
        request
            .text
            .layout_to_semantic_index(galley.cursor_from_pos(egui::vec2(point.x, point.y)).index)
    }
}

fn galley_lines(request: &TextLayoutRequest<'_>, galley: &egui::Galley) -> Vec<TextLayoutLine> {
    let mut layout_start = 0usize;
    galley
        .rows
        .iter()
        .map(|row| {
            let row_len = row.glyphs.len() + usize::from(row.ends_with_newline);
            let layout_end = layout_start + row_len;
            let line = TextLayoutLine {
                layout_start,
                layout_end,
                semantic_start: request.text.layout_to_semantic_index(layout_start),
                semantic_end: request.text.layout_to_semantic_index(layout_end),
                width: row.size.x,
                height: row.size.y,
            };
            layout_start = layout_end;
            line
        })
        .collect()
}
pub fn configure_text_selection_input(context: &egui::Context) {
    context.options_mut(|options| {
        options.input_options.max_click_duration = TEXT_SELECTION_CLICK_INTERVAL.as_secs_f64();
        options.input_options.max_click_dist = TEXT_SELECTION_CLICK_DISTANCE;
    });
}

pub fn copy_selected_text_on_command(ui: &egui::Ui, output: &DocumentOutput) {
    if copy_requested(ui)
        && let Some(text) = output.selected_text()
        && !text.is_empty()
    {
        ui.ctx().copy_text(text);
    }
}

fn copy_requested(ui: &egui::Ui) -> bool {
    ui.ctx().input_mut(|input| {
        input
            .events
            .iter()
            .any(|event| matches!(event, egui::Event::Copy))
            || input.consume_key(egui::Modifiers::COMMAND, egui::Key::C)
    })
}
pub(crate) fn paint_document_text_selection(
    galley: &mut Arc<egui::Galley>,
    cursor_range: &egui::text_selection::CCursorRange,
    background_color: Color,
    text_color: Color,
) {
    if cursor_range.is_empty() {
        return;
    }

    let background_color = to_egui_color(background_color);
    let text_color = to_egui_color(text_color);
    let galley = Arc::make_mut(galley);
    let [min, max] = cursor_range.sorted_cursors();
    let min = galley.layout_from_cursor(min);
    let max = galley.layout_from_cursor(max);

    for row_index in min.row..=max.row {
        let placed_row = &mut galley.rows[row_index];
        let row = Arc::make_mut(&mut placed_row.row);
        let left = if row_index == min.row {
            row.x_offset(min.column)
        } else {
            0.0
        };
        let right = if row_index == max.row {
            row.x_offset(max.column)
        } else {
            let newline_size = if placed_row.ends_with_newline {
                row.height() / 2.0
            } else {
                0.0
            };
            row.size.x + newline_size
        };
        let rect = egui::Rect::from_min_max(egui::pos2(left, 0.0), egui::pos2(right, row.size.y));

        if !row.glyphs.is_empty() {
            let first_glyph_index = if row_index == min.row { min.column } else { 0 };
            let last_glyph_index = if row_index == max.row {
                max.column
            } else {
                row.glyphs.len() - 1
            };
            let first_vertex_index = row
                .glyphs
                .get(first_glyph_index)
                .map_or(row.visuals.glyph_vertex_range.end, |glyph| {
                    glyph.first_vertex as _
                });
            let last_vertex_index = row
                .glyphs
                .get(last_glyph_index)
                .map_or(row.visuals.glyph_vertex_range.end, |glyph| {
                    glyph.first_vertex as _
                });
            for vertex_index in first_vertex_index..last_vertex_index {
                row.visuals.mesh.vertices[vertex_index].color = text_color;
            }
        }

        let mesh = &mut row.visuals.mesh;
        let glyph_index_start = row.visuals.glyph_index_start;
        let num_indices_before = mesh.indices.len();
        mesh.add_colored_rect(rect, background_color);
        let selection_triangles = [
            mesh.indices[num_indices_before],
            mesh.indices[num_indices_before + 1],
            mesh.indices[num_indices_before + 2],
            mesh.indices[num_indices_before + 3],
            mesh.indices[num_indices_before + 4],
            mesh.indices[num_indices_before + 5],
        ];
        for index in (glyph_index_start..num_indices_before).rev() {
            mesh.indices.swap(index, index + 6);
        }
        mesh.indices[glyph_index_start..glyph_index_start + 6]
            .clone_from_slice(&selection_triangles);
        row.visuals.mesh_bounds = mesh.calc_bounds();
    }
}

pub(crate) fn layout_job(
    request: TextLayoutRequest<'_>,
    _color: egui::Color32,
) -> egui::text::LayoutJob {
    let wrap_width = match request.layout_style.text_wrap_mode {
        TextWrapMode::NoWrap => f32::INFINITY,
        TextWrapMode::Wrap => request.wrap_width.max(1.0),
    };
    let mut job = egui::text::LayoutJob::default();
    job.wrap.max_width = wrap_width;
    if let Some(max_lines) = request.layout_style.max_lines {
        job.wrap.max_rows = max_lines.max(1);
        if job.wrap.max_rows == 1 {
            job.wrap.break_anywhere = true;
        }
    }
    job.wrap.break_anywhere = job.wrap.break_anywhere
        || matches!(
            request.layout_style.overflow_wrap,
            des_document::OverflowWrap::Anywhere
        )
        || matches!(
            request.layout_style.word_break,
            des_document::WordBreak::BreakAll
        );
    for run in request.text.runs() {
        job.append(
            &run.text,
            0.0,
            text_format(
                &run.style,
                request.font_size,
                request.color,
                request.line_height,
            ),
        );
    }
    job
}

fn text_format(
    style: &InlineTextStyle,
    inherited_font_size: f32,
    inherited_color: Color,
    inherited_line_height: Option<f32>,
) -> egui::TextFormat {
    let color = style.color.unwrap_or(inherited_color);
    let family = style
        .font_family
        .as_ref()
        .map(|family| egui::FontFamily::Name(family.clone().into()))
        .unwrap_or(egui::FontFamily::Proportional);
    let coords = style
        .font_weight
        .map(|weight| egui::epaint::text::VariationCoords::new([(b"wght", weight.value() as f32)]))
        .unwrap_or_default();
    egui::TextFormat {
        font_id: egui::FontId::new(style.font_size.unwrap_or(inherited_font_size), family),
        color: to_egui_color(color),
        coords,
        italics: style.italic.unwrap_or(false),
        strikethrough: if style
            .text_decoration
            .unwrap_or(TextDecoration::NONE)
            .line_through
        {
            egui::Stroke::new(1.0, to_egui_color(color))
        } else {
            egui::Stroke::NONE
        },
        underline: if style
            .text_decoration
            .unwrap_or(TextDecoration::NONE)
            .underline
        {
            egui::Stroke::new(1.0, to_egui_color(color))
        } else {
            egui::Stroke::NONE
        },
        background: style
            .background
            .map(to_egui_color)
            .unwrap_or(egui::Color32::TRANSPARENT),
        line_height: style
            .line_height
            .or(inherited_line_height)
            .map(|height| height.max(1.0)),
        ..Default::default()
    }
}

fn to_egui_color(color: Color) -> egui::Color32 {
    egui::Color32::from_rgba_premultiplied(color.r, color.g, color.b, color.a)
}

#[cfg(test)]
mod tests {
    use super::*;
    use des_document::{
        FontWeight, InlineTextStyle, NormalizedText, TextContent, TextDecoration, TextLayoutStyle,
        TextRun, WhiteSpace,
    };

    #[test]
    fn layout_job_extends_without_wrapping() {
        let text = TextContent::plain("A line that should not wrap");
        let normalized =
            NormalizedText::from_content(&text, TextLayoutStyle::white_space(WhiteSpace::Pre));
        let job = layout_job(
            TextLayoutRequest {
                text: &normalized,
                font_size: 14.0,
                color: Color::rgb(255, 255, 255),
                wrap_width: 10.0,
                layout_style: TextLayoutStyle::white_space(WhiteSpace::Pre),
                line_height: Some(18.0),
            },
            egui::Color32::WHITE,
        );

        assert_eq!(job.wrap.max_width, f32::INFINITY);
        assert_eq!(job.wrap.max_rows, usize::MAX);
        assert_eq!(job.sections[0].format.line_height, Some(18.0));
    }

    #[test]
    fn layout_job_max_lines_forces_single_breakable_row() {
        let text = TextContent::plain("truncate me");
        let mut style = TextLayoutStyle::default();
        style.max_lines = Some(1);
        let normalized = NormalizedText::from_content(&text, style);
        let job = layout_job(
            TextLayoutRequest {
                text: &normalized,
                font_size: 14.0,
                color: Color::rgb(255, 255, 255),
                wrap_width: 0.0,
                layout_style: style,
                line_height: None,
            },
            egui::Color32::WHITE,
        );

        assert_eq!(job.wrap.max_width, 1.0);
        assert_eq!(job.wrap.max_rows, 1);
        assert!(job.wrap.break_anywhere);
    }

    #[test]
    fn layout_job_preserves_inline_run_visual_style() {
        let text = TextContent::new(vec![
            TextRun::plain("plain "),
            TextRun::styled(
                "bold",
                InlineTextStyle {
                    color: Some(Color::rgb(255, 0, 0)),
                    font_size: Some(18.0),
                    font_weight: Some(FontWeight::BOLD),
                    italic: Some(true),
                    text_decoration: Some(TextDecoration::lines(true, false, true)),
                    background: Some(Color::rgb(0, 0, 255)),
                    ..InlineTextStyle::default()
                },
            ),
        ]);
        let normalized = NormalizedText::from_content(&text, TextLayoutStyle::default());

        let job = layout_job(
            TextLayoutRequest {
                text: &normalized,
                font_size: 14.0,
                color: Color::rgb(255, 255, 255),
                wrap_width: 240.0,
                layout_style: TextLayoutStyle::default(),
                line_height: None,
            },
            egui::Color32::WHITE,
        );

        assert_eq!(job.sections.len(), 2);
        let format = &job.sections[1].format;
        assert_eq!(format.color, egui::Color32::from_rgb(255, 0, 0));
        assert_eq!(format.font_id.size, 18.0);
        assert!(format.italics);
        assert_ne!(format.underline, egui::Stroke::NONE);
        assert_ne!(format.strikethrough, egui::Stroke::NONE);
        assert_eq!(format.background, egui::Color32::from_rgb(0, 0, 255));
        assert_eq!(
            format.coords.as_ref(),
            &[(egui::epaint::text::Tag::new(b"wght"), 700.0)]
        );
    }

    #[test]
    fn layout_job_maps_numeric_font_weight_to_variation_axis() {
        let text = TextContent::new(vec![TextRun::styled(
            "weighted",
            InlineTextStyle {
                font_weight: Some(FontWeight::new(525)),
                ..InlineTextStyle::default()
            },
        )]);
        let normalized = NormalizedText::from_content(&text, TextLayoutStyle::default());

        let job = layout_job(
            TextLayoutRequest {
                text: &normalized,
                font_size: 14.0,
                color: Color::rgb(255, 255, 255),
                wrap_width: 240.0,
                layout_style: TextLayoutStyle::default(),
                line_height: None,
            },
            egui::Color32::WHITE,
        );

        assert_eq!(
            job.sections[0].format.coords.as_ref(),
            &[(egui::epaint::text::Tag::new(b"wght"), 525.0)]
        );
    }

    #[test]
    fn configure_text_selection_input_sets_host_click_thresholds() {
        let ctx = egui::Context::default();

        configure_text_selection_input(&ctx);

        ctx.options(|options| {
            assert_eq!(
                options.input_options.max_click_duration,
                TEXT_SELECTION_CLICK_INTERVAL.as_secs_f64()
            );
            assert_eq!(
                options.input_options.max_click_dist,
                TEXT_SELECTION_CLICK_DISTANCE
            );
        });
    }
}
