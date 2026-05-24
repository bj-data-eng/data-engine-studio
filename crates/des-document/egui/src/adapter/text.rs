use des_document::{
    Color, DocumentOutput, Point, Size, TextLayoutRequest, TextLayoutResult, TextMeasurer,
    TextMeasurerKey, TextWrapMode,
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
            .fonts_mut(|fonts| fonts.layout_job(layout_job(request, egui::Color32::WHITE)));
        let size = galley.size();
        TextLayoutResult {
            size: Size::new(size.x, size.y),
            line_count: galley.rows.len(),
            elided: galley.elided,
        }
    }

    fn text_index_at(&mut self, request: TextLayoutRequest<'_>, point: Point) -> usize {
        let galley = self
            .ctx
            .fonts_mut(|fonts| fonts.layout_job(layout_job(request, egui::Color32::WHITE)));
        galley.cursor_from_pos(egui::vec2(point.x, point.y)).index
    }
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
    color: egui::Color32,
) -> egui::text::LayoutJob {
    let wrap_width = if request.wrap_mode == TextWrapMode::Extend {
        f32::INFINITY
    } else {
        request.wrap_width.max(1.0)
    };
    let mut job = egui::text::LayoutJob::simple(
        request.text.to_owned(),
        egui::FontId::proportional(request.font_size),
        color,
        wrap_width,
    );
    job.wrap.max_width = wrap_width;
    if request.wrap_mode == TextWrapMode::Truncate {
        job.wrap.max_rows = 1;
        job.wrap.break_anywhere = true;
    }
    if request.wrap_mode != TextWrapMode::Truncate
        && let Some(max_lines) = request.max_lines
    {
        job.wrap.max_rows = max_lines.max(1);
        if job.wrap.max_rows == 1 {
            job.wrap.break_anywhere = true;
        }
    }
    if let Some(line_height) = request.line_height {
        if let Some(section) = job.sections.first_mut() {
            section.format.line_height = Some(line_height.max(1.0));
        }
    }
    job
}

fn to_egui_color(color: Color) -> egui::Color32 {
    egui::Color32::from_rgba_premultiplied(color.r, color.g, color.b, color.a)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_job_extends_without_wrapping() {
        let job = layout_job(
            TextLayoutRequest {
                text: "A line that should not wrap",
                font_size: 14.0,
                wrap_width: 10.0,
                wrap_mode: TextWrapMode::Extend,
                max_lines: Some(1),
                line_height: Some(18.0),
            },
            egui::Color32::WHITE,
        );

        assert_eq!(job.wrap.max_width, f32::INFINITY);
        assert_eq!(job.wrap.max_rows, 1);
        assert_eq!(job.sections[0].format.line_height, Some(18.0));
    }

    #[test]
    fn layout_job_truncate_forces_single_breakable_row() {
        let job = layout_job(
            TextLayoutRequest {
                text: "truncate me",
                font_size: 14.0,
                wrap_width: 0.0,
                wrap_mode: TextWrapMode::Truncate,
                max_lines: Some(3),
                line_height: None,
            },
            egui::Color32::WHITE,
        );

        assert_eq!(job.wrap.max_width, 1.0);
        assert_eq!(job.wrap.max_rows, 1);
        assert!(job.wrap.break_anywhere);
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
