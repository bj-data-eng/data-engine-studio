//! Renderer-neutral paint planning for document UI.
//!
//! `des-ui-render` turns resolved document output into a deterministic display
//! list. Backends such as `des-ui-egui` should translate these commands into
//! host-specific drawing calls.

use des_ui_document::{
    BorderStyle, Color, CornerRadii, DocumentOutput, ElementId, FloatingPlacement, Glyph, Insets,
    Overflow, Point, Rect, ResolvedElement, ScrollAxis, ScrollChrome, Shadow, TextWrapMode,
};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DisplayList {
    pub commands: Vec<PaintCommand>,
}

impl DisplayList {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, command: PaintCommand) {
        self.commands.push(command);
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum PaintCommand {
    PushClip(Rect),
    PopClip,
    Surface(SurfacePaint),
    Text(TextPaint),
    Glyph(GlyphPaint),
    ScrollChrome(ScrollChromePaint),
}

#[derive(Clone, Debug, PartialEq)]
pub struct SurfacePaint {
    pub element_id: ElementId,
    pub rect: Rect,
    pub radius: CornerRadii,
    pub shadows: Vec<Shadow>,
    pub background: Option<Color>,
    pub border: Option<BorderPaint>,
    pub floating_arrow: Option<FloatingArrowPaint>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BorderPaint {
    pub color: Color,
    pub widths: Insets,
    pub style: BorderStyle,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FloatingArrowPaint {
    pub points: [Point; 3],
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextPaint {
    pub element_id: ElementId,
    pub rect: Rect,
    pub text: String,
    pub color: Color,
    pub font_size: f32,
    pub wrap_width: f32,
    pub wrap_mode: TextWrapMode,
    pub max_lines: Option<usize>,
    pub line_height: Option<f32>,
    pub selection: Option<TextSelectionPaint>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextSelectionPaint {
    pub anchor_index: usize,
    pub focus_index: usize,
    pub background: Color,
    pub color: Color,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GlyphPaint {
    pub element_id: ElementId,
    pub rect: Rect,
    pub glyph: Glyph,
    pub color: Color,
    pub size: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScrollChromePaint {
    pub element_id: ElementId,
    pub axis: ScrollAxis,
    pub track_rect: Rect,
    pub hit_rect: Rect,
    pub handle_rect: Rect,
    pub handle_color: Color,
    pub track_color: Option<Color>,
    pub handle_border_color: Option<Color>,
    pub handle_border_width: f32,
    pub radius: f32,
    pub visible: bool,
    pub expanded: bool,
    pub hovered: bool,
    pub dragged: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PaintPlanner;

impl PaintPlanner {
    pub fn new() -> Self {
        Self
    }

    pub fn plan_output(&self, output: &DocumentOutput) -> DisplayList {
        let mut list = DisplayList::new();
        self.plan_element(&mut list, &output.layout, None, output);
        for chrome in &output.scroll_chrome {
            list.push(PaintCommand::ScrollChrome(ScrollChromePaint::from(chrome)));
        }
        list
    }

    pub fn plan_element(
        &self,
        list: &mut DisplayList,
        frame: &ResolvedElement,
        clip_rect: Option<Rect>,
        output: &DocumentOutput,
    ) {
        if frame.id.as_str() != "root" {
            list.push(PaintCommand::Surface(surface_paint(frame)));

            if let Some(text) = &frame.text {
                list.push(PaintCommand::Text(text_paint(frame, text, output)));
            }

            if let Some(glyph) = frame.glyph {
                list.push(PaintCommand::Glyph(GlyphPaint {
                    element_id: frame.id.clone(),
                    rect: frame.rect,
                    glyph,
                    color: frame.style.text_color,
                    size: frame.style.font_size,
                }));
            }
        }

        let next_clip = child_clip_rect(frame, clip_rect);
        let pushed_clip = next_clip != clip_rect;
        if let Some(next_clip) = next_clip
            && pushed_clip
        {
            list.push(PaintCommand::PushClip(next_clip));
        }

        let mut children: Vec<_> = frame.children.iter().collect();
        children.sort_by_key(|child| child.style.z_index);
        for child in children {
            self.plan_element(list, child, next_clip, output);
        }

        if pushed_clip {
            list.push(PaintCommand::PopClip);
        }
    }
}

pub fn plan_paint(output: &DocumentOutput) -> DisplayList {
    PaintPlanner::new().plan_output(output)
}

pub fn content_rect(frame: &ResolvedElement) -> Rect {
    frame.rect.inset(Insets {
        top: frame.style.border_width.top + frame.style.padding.top,
        right: frame.style.border_width.right + frame.style.padding.right,
        bottom: frame.style.border_width.bottom + frame.style.padding.bottom,
        left: frame.style.border_width.left + frame.style.padding.left,
    })
}

fn surface_paint(frame: &ResolvedElement) -> SurfacePaint {
    SurfacePaint {
        element_id: frame.id.clone(),
        rect: frame.rect,
        radius: frame.style.radius,
        shadows: frame.style.shadows.clone(),
        background: frame.style.background,
        border: frame.style.border.map(|color| BorderPaint {
            color,
            widths: frame.style.border_width,
            style: frame.style.border_style,
        }),
        floating_arrow: floating_arrow(frame),
    }
}

fn text_paint(frame: &ResolvedElement, text: &str, output: &DocumentOutput) -> TextPaint {
    let rect = content_rect(frame);
    let selection = output.text_selection.as_ref().and_then(|selection| {
        if frame.selectable_text && selection.target == frame.id {
            Some(TextSelectionPaint {
                anchor_index: selection.anchor_index,
                focus_index: selection.focus_index,
                background: frame.style.text_selection_background,
                color: frame.style.text_selection_color,
            })
        } else {
            None
        }
    });

    TextPaint {
        element_id: frame.id.clone(),
        rect,
        text: text.to_owned(),
        color: frame.style.text_color,
        font_size: frame.style.font_size,
        wrap_width: if frame.style.text_wrap == TextWrapMode::Extend {
            f32::INFINITY
        } else {
            rect.size.width
        },
        wrap_mode: frame.style.text_wrap,
        max_lines: frame.style.max_lines,
        line_height: frame.style.line_height,
        selection,
    }
}

fn child_clip_rect(frame: &ResolvedElement, current_clip: Option<Rect>) -> Option<Rect> {
    if frame.style.overflow_x != Overflow::Scroll && frame.style.overflow_y != Overflow::Scroll {
        return current_clip;
    }

    let content = content_rect(frame);
    let base = current_clip.unwrap_or(frame.rect);
    let clip = Rect::new(
        if frame.style.overflow_x == Overflow::Scroll {
            content.origin.x
        } else {
            base.origin.x
        },
        if frame.style.overflow_y == Overflow::Scroll {
            content.origin.y
        } else {
            base.origin.y
        },
        if frame.style.overflow_x == Overflow::Scroll {
            content.size.width
        } else {
            base.size.width
        },
        if frame.style.overflow_y == Overflow::Scroll {
            content.size.height
        } else {
            base.size.height
        },
    );
    base.intersect(clip)
}

fn floating_arrow(frame: &ResolvedElement) -> Option<FloatingArrowPaint> {
    let floating = frame.floating?;
    let offset = floating.arrow_offset?;
    let size = floating.arrow_size?;
    Some(FloatingArrowPaint {
        points: floating_arrow_points(
            frame.rect,
            floating.placement,
            offset.x,
            offset.y,
            size.width,
            size.height,
        ),
    })
}

pub fn floating_arrow_points(
    rect: Rect,
    placement: FloatingPlacement,
    offset_x: f32,
    offset_y: f32,
    width: f32,
    height: f32,
) -> [Point; 3] {
    match placement {
        FloatingPlacement::Center => {
            let center = Point::new(
                rect.origin.x + rect.size.width * 0.5,
                rect.origin.y + rect.size.height * 0.5,
            );
            [center, center, center]
        }
        FloatingPlacement::Bottom
        | FloatingPlacement::BottomStart
        | FloatingPlacement::BottomEnd => {
            let left = rect.origin.x + offset_x;
            let center = left + width * 0.5;
            [
                Point::new(left, rect.origin.y),
                Point::new(left + width, rect.origin.y),
                Point::new(center, rect.origin.y - height),
            ]
        }
        FloatingPlacement::Top | FloatingPlacement::TopStart | FloatingPlacement::TopEnd => {
            let left = rect.origin.x + offset_x;
            let center = left + width * 0.5;
            [
                Point::new(left + width, rect.bottom()),
                Point::new(left, rect.bottom()),
                Point::new(center, rect.bottom() + height),
            ]
        }
        FloatingPlacement::Right | FloatingPlacement::RightStart | FloatingPlacement::RightEnd => {
            let top = rect.origin.y + offset_y;
            let center = top + height * 0.5;
            [
                Point::new(rect.origin.x, top + height),
                Point::new(rect.origin.x, top),
                Point::new(rect.origin.x - width, center),
            ]
        }
        FloatingPlacement::Left | FloatingPlacement::LeftStart | FloatingPlacement::LeftEnd => {
            let top = rect.origin.y + offset_y;
            let center = top + height * 0.5;
            [
                Point::new(rect.right(), top),
                Point::new(rect.right(), top + height),
                Point::new(rect.right() + width, center),
            ]
        }
    }
}

impl From<&ScrollChrome> for ScrollChromePaint {
    fn from(chrome: &ScrollChrome) -> Self {
        Self {
            element_id: chrome.element_id.clone(),
            axis: chrome.axis,
            track_rect: chrome.track_rect,
            hit_rect: chrome.hit_rect,
            handle_rect: chrome.handle_rect,
            handle_color: chrome.handle_color,
            track_color: chrome.track_color,
            handle_border_color: chrome.handle_border_color,
            handle_border_width: chrome.handle_border_width,
            radius: chrome.radius,
            visible: chrome.visible,
            expanded: chrome.expanded,
            hovered: chrome.hovered,
            dragged: chrome.dragged,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use des_ui_document::{
        Color, Document, DocumentEngine, Insets, Size, Style, StyleSelector, StyleSheet,
    };

    #[test]
    fn plans_surface_text_and_children_in_z_order() {
        let mut document = Document::build(Size::new(200.0, 120.0), |ui| {
            ui.div("lower").empty();
            ui.text("label", "Hello");
        });
        let stylesheet = StyleSheet::new()
            .rule(
                StyleSelector::Id("lower".into()),
                Style::default()
                    .size(40.0, 20.0)
                    .background(Color::rgb(10, 20, 30))
                    .z_index(5),
            )
            .rule(
                StyleSelector::Id("label".into()),
                Style::default().size(60.0, 20.0).z_index(1),
            );
        let output = DocumentEngine::default().update(&mut document, &stylesheet);

        let list = plan_paint(&output);
        let ids: Vec<_> = list
            .commands
            .iter()
            .filter_map(|command| match command {
                PaintCommand::Surface(surface) => Some(surface.element_id.as_str().to_owned()),
                PaintCommand::Text(text) => Some(text.element_id.as_str().to_owned()),
                _ => None,
            })
            .collect();

        assert_eq!(ids, vec!["label", "label", "lower"]);
    }

    #[test]
    fn computes_content_rect_from_border_and_padding() {
        let mut document = Document::build(Size::new(200.0, 120.0), |ui| {
            ui.div("box").empty();
        });
        let stylesheet = StyleSheet::new().rule(
            StyleSelector::Id("box".into()),
            Style::default()
                .size(100.0, 80.0)
                .border_widths(Insets::all(2.0))
                .padding(Insets::symmetric(10.0, 6.0)),
        );
        let output = DocumentEngine::default().update(&mut document, &stylesheet);
        let box_frame = output.layout.find("box").expect("box frame");

        assert_eq!(content_rect(box_frame), Rect::new(12.0, 8.0, 76.0, 64.0));
    }

    #[test]
    fn computes_floating_arrow_points_without_backend_types() {
        let points = floating_arrow_points(
            Rect::new(10.0, 20.0, 100.0, 50.0),
            FloatingPlacement::Bottom,
            30.0,
            0.0,
            12.0,
            8.0,
        );

        assert_eq!(
            points,
            [
                Point::new(40.0, 20.0),
                Point::new(52.0, 20.0),
                Point::new(46.0, 12.0),
            ]
        );
    }
}
