use des_document::{
    Color, Direction, DocumentOutput, FontStyle, InlineTextStyle, Point, Size, TextAlign,
    TextDecoration, TextLayoutLine, TextLayoutRequest, TextLayoutResult, TextMeasurer,
    TextMeasurerKey, TextOverflow, TextVerticalAlign, TextWrapMode,
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
            first_baseline: lines.first().map(|line| line.baseline),
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
                x_offset: row.pos.x,
                width: row.size.x,
                height: row.size.y,
                baseline: row_baseline(row),
            };
            layout_start = layout_end;
            line
        })
        .collect()
}

fn row_baseline(row: &egui::epaint::text::PlacedRow) -> f32 {
    row.glyphs
        .iter()
        .map(|glyph| glyph.pos.y)
        .fold(0.0, f32::max)
        .clamp(0.0, row.size.y)
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
    job.wrap.overflow_character = match request.layout_style.text_overflow {
        TextOverflow::Clip => None,
        TextOverflow::Ellipsis => Some('…'),
    };
    job.halign = egui_text_align(request.layout_style.text_align, request.direction);
    job.justify = request.layout_style.text_align == TextAlign::Justify;
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

fn egui_text_align(text_align: TextAlign, direction: Direction) -> egui::Align {
    match text_align {
        TextAlign::Start | TextAlign::Justify if direction == Direction::Rtl => egui::Align::RIGHT,
        TextAlign::Start | TextAlign::Justify => egui::Align::LEFT,
        TextAlign::Center => egui::Align::Center,
        TextAlign::End if direction == Direction::Rtl => egui::Align::LEFT,
        TextAlign::End => egui::Align::RIGHT,
    }
}

fn text_format(
    style: &InlineTextStyle,
    inherited_font_size: f32,
    inherited_color: Color,
    inherited_line_height: Option<f32>,
) -> egui::TextFormat {
    let color = style.color.unwrap_or(inherited_color);
    let text_decoration = style.text_decoration.unwrap_or(TextDecoration::NONE);
    let family = style
        .font_family
        .as_deref()
        .map(egui_font_family)
        .unwrap_or(egui::FontFamily::Proportional);
    let coords = variation_coords(style);
    egui::TextFormat {
        font_id: egui::FontId::new(style.font_size.unwrap_or(inherited_font_size), family),
        extra_letter_spacing: style.letter_spacing.unwrap_or(0.0).max(0.0),
        color: to_egui_color(color),
        coords,
        italics: matches!(
            style.font_style,
            Some(FontStyle::Italic | FontStyle::Oblique)
        ),
        strikethrough: if text_decoration.line_through {
            text_decoration_stroke(text_decoration, color)
        } else {
            egui::Stroke::NONE
        },
        underline: if text_decoration.underline {
            text_decoration_stroke(text_decoration, color)
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
        valign: egui_vertical_align(style.vertical_align.unwrap_or(TextVerticalAlign::Baseline)),
        ..Default::default()
    }
}

fn egui_font_family(css_family: &str) -> egui::FontFamily {
    for family in css_family.split(',').map(clean_css_font_family) {
        match family.to_ascii_lowercase().as_str() {
            "sans-serif" | "system-ui" | "ui-sans-serif" => {
                return egui::FontFamily::Proportional;
            }
            "monospace" | "ui-monospace" => return egui::FontFamily::Monospace,
            "serif" | "ui-serif" => continue,
            "" => continue,
            "inter" | "aptos" => return egui::FontFamily::Proportional,
            "jetbrains mono" => return egui::FontFamily::Monospace,
            _ => {
                return egui::FontFamily::Name(family.into());
            }
        }
    }
    egui::FontFamily::Proportional
}

fn clean_css_font_family(family: &str) -> &str {
    family.trim().trim_matches('"').trim_matches('\'').trim()
}

fn egui_vertical_align(vertical_align: TextVerticalAlign) -> egui::Align {
    match vertical_align {
        TextVerticalAlign::Top | TextVerticalAlign::Super => egui::Align::TOP,
        TextVerticalAlign::Middle => egui::Align::Center,
        TextVerticalAlign::Baseline | TextVerticalAlign::Bottom | TextVerticalAlign::Sub => {
            egui::Align::BOTTOM
        }
    }
}

fn text_decoration_stroke(decoration: TextDecoration, current_color: Color) -> egui::Stroke {
    egui::Stroke::new(
        decoration.stroke_thickness(),
        to_egui_color(decoration.stroke_color(current_color)),
    )
}

fn variation_coords(style: &InlineTextStyle) -> egui::epaint::text::VariationCoords {
    let mut coords = egui::epaint::text::VariationCoords::default();
    if let Some(weight) = style.font_weight {
        coords.push(b"wght", weight.value() as f32);
    }
    if let Some(stretch) = style.font_stretch {
        coords.push(b"wdth", stretch.value());
    }
    coords
}

fn to_egui_color(color: Color) -> egui::Color32 {
    egui::Color32::from_rgba_premultiplied(color.r, color.g, color.b, color.a)
}

#[cfg(test)]
mod tests {
    use super::*;
    use des_document::{
        FontStretch, FontStyle, FontWeight, InlineTextStyle, NormalizedText, TextAlign,
        TextContent, TextDecoration, TextLayoutStyle, TextOverflow, TextRun, TextVerticalAlign,
        WhiteSpace,
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
                direction: Direction::Ltr,
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
    fn egui_font_family_accepts_css_family_lists() {
        assert_eq!(
            egui_font_family("Aptos, Inter, sans-serif"),
            egui::FontFamily::Proportional
        );
        assert_eq!(
            egui_font_family("'JetBrains Mono', ui-monospace, monospace"),
            egui::FontFamily::Monospace
        );
        assert_eq!(
            egui_font_family("IconFlow"),
            egui::FontFamily::Name("IconFlow".into())
        );
    }

    #[test]
    fn layout_job_max_lines_forces_single_breakable_row() {
        let text = TextContent::plain("truncate me");
        let style = TextLayoutStyle {
            max_lines: Some(1),
            ..Default::default()
        };
        let normalized = NormalizedText::from_content(&text, style);
        let job = layout_job(
            TextLayoutRequest {
                text: &normalized,
                font_size: 14.0,
                color: Color::rgb(255, 255, 255),
                direction: Direction::Ltr,
                wrap_width: 0.0,
                layout_style: style,
                line_height: None,
            },
            egui::Color32::WHITE,
        );

        assert_eq!(job.wrap.max_width, 1.0);
        assert_eq!(job.wrap.max_rows, 1);
        assert!(job.wrap.break_anywhere);
        assert_eq!(job.wrap.overflow_character, None);
    }

    #[test]
    fn layout_job_maps_text_overflow_ellipsis() {
        let text = TextContent::plain("truncate me");
        let style = TextLayoutStyle {
            max_lines: Some(1),
            text_overflow: TextOverflow::Ellipsis,
            ..Default::default()
        };
        let normalized = NormalizedText::from_content(&text, style);
        let job = layout_job(
            TextLayoutRequest {
                text: &normalized,
                font_size: 14.0,
                color: Color::rgb(255, 255, 255),
                direction: Direction::Ltr,
                wrap_width: 60.0,
                layout_style: style,
                line_height: None,
            },
            egui::Color32::WHITE,
        );

        assert_eq!(job.wrap.overflow_character, Some('…'));
    }

    #[test]
    fn layout_job_maps_text_alignment() {
        let text = TextContent::plain("aligned");
        let style = TextLayoutStyle {
            text_align: TextAlign::End,
            ..Default::default()
        };
        let normalized = NormalizedText::from_content(&text, style);
        let job = layout_job(
            TextLayoutRequest {
                text: &normalized,
                font_size: 14.0,
                color: Color::rgb(255, 255, 255),
                direction: Direction::Ltr,
                wrap_width: 120.0,
                layout_style: style,
                line_height: None,
            },
            egui::Color32::WHITE,
        );

        assert_eq!(job.halign, egui::Align::RIGHT);
        assert!(!job.justify);
    }

    #[test]
    fn layout_job_maps_start_and_end_alignment_through_direction() {
        let text = TextContent::plain("aligned");
        let normalized = NormalizedText::from_content(&text, TextLayoutStyle::default());

        let rtl_start = layout_job(
            TextLayoutRequest {
                text: &normalized,
                font_size: 14.0,
                color: Color::rgb(255, 255, 255),
                direction: Direction::Rtl,
                wrap_width: 120.0,
                layout_style: TextLayoutStyle::default(),
                line_height: None,
            },
            egui::Color32::WHITE,
        );

        let style = TextLayoutStyle {
            text_align: TextAlign::End,
            ..Default::default()
        };
        let rtl_end = layout_job(
            TextLayoutRequest {
                text: &normalized,
                font_size: 14.0,
                color: Color::rgb(255, 255, 255),
                direction: Direction::Rtl,
                wrap_width: 120.0,
                layout_style: style,
                line_height: None,
            },
            egui::Color32::WHITE,
        );

        assert_eq!(rtl_start.halign, egui::Align::RIGHT);
        assert_eq!(rtl_end.halign, egui::Align::LEFT);
    }

    #[test]
    fn layout_job_maps_justified_text_alignment() {
        let text = TextContent::plain("justified text");
        let style = TextLayoutStyle {
            text_align: TextAlign::Justify,
            ..Default::default()
        };
        let normalized = NormalizedText::from_content(&text, style);
        let job = layout_job(
            TextLayoutRequest {
                text: &normalized,
                font_size: 14.0,
                color: Color::rgb(255, 255, 255),
                direction: Direction::Ltr,
                wrap_width: 120.0,
                layout_style: style,
                line_height: None,
            },
            egui::Color32::WHITE,
        );

        assert_eq!(job.halign, egui::Align::LEFT);
        assert!(job.justify);
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
                    letter_spacing: Some(1.5),
                    font_weight: Some(FontWeight::BOLD),
                    font_stretch: Some(FontStretch::CONDENSED),
                    font_style: Some(FontStyle::Italic),
                    text_decoration: Some(
                        TextDecoration::lines(true, false, true)
                            .color(Color::rgb(0, 255, 0))
                            .thickness(2.0),
                    ),
                    vertical_align: Some(TextVerticalAlign::Super),
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
                direction: Direction::Ltr,
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
        assert_eq!(format.extra_letter_spacing, 1.5);
        assert!(format.italics);
        assert_eq!(format.valign, egui::Align::TOP);
        assert_eq!(format.underline.width, 2.0);
        assert_eq!(format.underline.color, egui::Color32::from_rgb(0, 255, 0));
        assert_eq!(format.strikethrough.width, 2.0);
        assert_eq!(
            format.strikethrough.color,
            egui::Color32::from_rgb(0, 255, 0)
        );
        assert_eq!(format.background, egui::Color32::from_rgb(0, 0, 255));
        assert_eq!(
            format.coords.as_ref(),
            &[
                (egui::epaint::text::Tag::new(b"wght"), 700.0),
                (egui::epaint::text::Tag::new(b"wdth"), 75.0)
            ]
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
                direction: Direction::Ltr,
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
    fn layout_job_maps_font_stretch_to_variation_axis() {
        let text = TextContent::new(vec![TextRun::styled(
            "stretched",
            InlineTextStyle {
                font_stretch: Some(FontStretch::percent(137.5)),
                ..InlineTextStyle::default()
            },
        )]);
        let normalized = NormalizedText::from_content(&text, TextLayoutStyle::default());

        let job = layout_job(
            TextLayoutRequest {
                text: &normalized,
                font_size: 14.0,
                color: Color::rgb(255, 255, 255),
                direction: Direction::Ltr,
                wrap_width: 240.0,
                layout_style: TextLayoutStyle::default(),
                line_height: None,
            },
            egui::Color32::WHITE,
        );

        assert_eq!(
            job.sections[0].format.coords.as_ref(),
            &[(egui::epaint::text::Tag::new(b"wdth"), 137.5)]
        );
    }

    #[test]
    fn layout_job_maps_oblique_font_style_to_italics() {
        let text = TextContent::new(vec![TextRun::styled(
            "oblique",
            InlineTextStyle {
                font_style: Some(FontStyle::Oblique),
                ..InlineTextStyle::default()
            },
        )]);
        let normalized = NormalizedText::from_content(&text, TextLayoutStyle::default());

        let job = layout_job(
            TextLayoutRequest {
                text: &normalized,
                font_size: 14.0,
                color: Color::rgb(255, 255, 255),
                direction: Direction::Ltr,
                wrap_width: 240.0,
                layout_style: TextLayoutStyle::default(),
                line_height: None,
            },
            egui::Color32::WHITE,
        );

        assert!(job.sections[0].format.italics);
    }

    #[test]
    fn layout_job_maps_vertical_align_keywords() {
        let text = TextContent::new(vec![TextRun::styled(
            "sub",
            InlineTextStyle {
                vertical_align: Some(TextVerticalAlign::Sub),
                ..InlineTextStyle::default()
            },
        )]);
        let normalized = NormalizedText::from_content(&text, TextLayoutStyle::default());

        let job = layout_job(
            TextLayoutRequest {
                text: &normalized,
                font_size: 14.0,
                color: Color::rgb(255, 255, 255),
                direction: Direction::Ltr,
                wrap_width: 240.0,
                layout_style: TextLayoutStyle::default(),
                line_height: None,
            },
            egui::Color32::WHITE,
        );

        assert_eq!(job.sections[0].format.valign, egui::Align::BOTTOM);
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
