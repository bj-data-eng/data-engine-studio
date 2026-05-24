use crate::{
    AlignContent, AlignItems, Color, ComputedStyle, CornerRadii, Display, Document, DocumentEngine,
    DocumentEventKind, DocumentInput, Easing, Element, ElementId, ElementSpec, ElementState,
    ElementStateSelector, FlexDirection, FlexWrap, FloatingAxisOffset, FloatingPlacement,
    FloatingShift, GridAutoFlow, GridPlacement, GridPlacementLine, GridTemplateArea,
    GridTemplateComponent, GridTemplateRepetition, GridTrack, Insets, JustifyContent, Length,
    NthChildFormula, Overflow, Point, PointerInput, Rect, RepetitionCount, ScrollAxis, Shadow,
    Size, Style, StyleSelector, StyleSheet, TableCellSpec, TableColumnSpec, TableSpec,
    TableTrackSize, TextLayoutRequest, TextLayoutResult, TextMeasurer, TextMeasurerKey,
    TextWrapMode, Transition,
};
use des_layout::prelude::{
    AlignContent as LayoutAlignContent, AlignItems as LayoutAlignItems, Dimension,
    FlexDirection as LayoutFlexDirection, JustifyContent as LayoutJustifyContent,
    LengthPercentageAuto, Size as LayoutSize, fr, length, line, percent,
};
use des_layout::style::Overflow as LayoutOverflow;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

#[test]
fn nth_child_formula_matches_css_like_positions() {
    assert!(NthChildFormula::odd().matches(1));
    assert!(!NthChildFormula::odd().matches(2));
    assert!(NthChildFormula::even().matches(2));
    assert!(!NthChildFormula::even().matches(3));
    assert!(NthChildFormula::new(3, 2).matches(5));
    assert!(!NthChildFormula::new(3, 2).matches(4));
    assert!(NthChildFormula::new(4, 0).matches(8));
    assert!(!NthChildFormula::new(4, 0).matches(6));
    assert!(NthChildFormula::new(0, 3).matches(3));
    assert!(!NthChildFormula::new(0, 3).matches(4));
}

#[derive(Default)]
struct RecordingTextMeasurer {
    requests: Vec<(String, f32)>,
}

impl TextMeasurer for RecordingTextMeasurer {
    fn cache_key(&self) -> TextMeasurerKey {
        TextMeasurerKey::new("recording")
    }

    fn measure_text(&mut self, request: TextLayoutRequest<'_>) -> TextLayoutResult {
        self.requests
            .push((request.text.semantic_text().to_string(), request.wrap_width));
        TextLayoutResult::new(Size::new(64.0, 18.0), 1, false)
    }
}

static COMPUTE_TEXT_MEASURE_COUNT: AtomicUsize = AtomicUsize::new(0);

#[derive(Default)]
struct CountingTextMeasurer;

impl TextMeasurer for CountingTextMeasurer {
    fn cache_key(&self) -> TextMeasurerKey {
        TextMeasurerKey::new("counting")
    }

    fn measure_text(&mut self, request: TextLayoutRequest<'_>) -> TextLayoutResult {
        if request.text.semantic_text() == "Measured" {
            COMPUTE_TEXT_MEASURE_COUNT.fetch_add(1, Ordering::SeqCst);
        }
        TextLayoutResult::new(Size::new(64.0, 18.0), 1, false)
    }
}

#[derive(Default)]
struct SlotTextMeasurer {
    hit_requests: usize,
}

impl TextMeasurer for SlotTextMeasurer {
    fn cache_key(&self) -> TextMeasurerKey {
        TextMeasurerKey::new("slot")
    }

    fn measure_text(&mut self, _request: TextLayoutRequest<'_>) -> TextLayoutResult {
        TextLayoutResult::new(Size::new(120.0, 18.0), 1, false)
    }

    fn text_index_at(&mut self, _request: TextLayoutRequest<'_>, point: Point) -> usize {
        self.hit_requests += 1;
        (point.x / 20.0).floor().max(0.0) as usize
    }
}

fn hover_input(position: Point) -> DocumentInput {
    DocumentInput {
        pointer: Some(PointerInput {
            position,
            primary_delta: Point::ZERO,
            primary_down: false,
            primary_pressed: false,
            primary_clicked: false,
            primary_click_count: 0,
            secondary_clicked: false,
            time_seconds: 0.0,
        }),
        scroll_delta: Point::ZERO,
    }
}

fn drag_text_input(position: Point, primary_pressed: bool) -> DocumentInput {
    DocumentInput {
        pointer: Some(PointerInput {
            position,
            primary_delta: Point::ZERO,
            primary_down: true,
            primary_pressed,
            primary_clicked: primary_pressed,
            primary_click_count: u8::from(primary_pressed),
            secondary_clicked: false,
            time_seconds: 0.0,
        }),
        scroll_delta: Point::ZERO,
    }
}

#[test]
fn document_reparents_existing_element_without_reallocating_layout_node() {
    let mut document = Document::new(Size::new(800.0, 600.0));
    document
        .append_element("root", "panel", ElementSpec::new(Element::Div))
        .unwrap();
    document
        .append_text(
            "panel",
            "label",
            ElementSpec::new(Element::Text),
            "Retained text",
        )
        .unwrap();

    let original_layout_node = document.layout_node("label").unwrap();

    document.reparent("label", "root").unwrap();

    assert_eq!(document.layout_node("label"), Some(original_layout_node));
    assert_eq!(
        document.parent("label").unwrap(),
        Some(ElementId::new("root"))
    );
    assert!(document.children("panel").unwrap().is_empty());
    assert_eq!(
        document.children("root").unwrap(),
        vec![ElementId::new("panel"), ElementId::new("label")]
    );
}

#[test]
fn document_remove_prunes_descendants_from_model_and_layout_graph() {
    let mut document = Document::new(Size::new(800.0, 600.0));
    document
        .append_element("root", "panel", ElementSpec::new(Element::Div))
        .unwrap();
    document
        .append_text(
            "panel",
            "label",
            ElementSpec::new(Element::Text),
            "Retained text",
        )
        .unwrap();

    document.remove("panel").unwrap();

    assert_eq!(document.children("root").unwrap(), Vec::<ElementId>::new());
    assert!(document.layout_node("panel").is_none());
    assert!(document.layout_node("label").is_none());
    assert!(document.parent("label").is_err());
}

#[test]
fn document_applies_document_style_to_existing_layout_node() {
    let mut document = Document::new(Size::new(800.0, 600.0));
    document
        .append_element("root", "panel", ElementSpec::new(Element::Div))
        .unwrap();
    let original_layout_node = document.layout_node("panel").unwrap();

    let mut style = ComputedStyle::default();
    style.display = Display::Grid;
    style.flex_direction = FlexDirection::Row;
    style.flex_wrap = FlexWrap::Wrap;
    style.flex_basis = Length::Px(88.0);
    style.flex_grow = 2.0;
    style.flex_shrink = 0.5;
    style.align_content = AlignContent::SpaceAround;
    style.align_items = AlignItems::FlexEnd;
    style.align_self = Some(AlignItems::Baseline);
    style.justify_items = Some(AlignItems::Center);
    style.justify_self = Some(AlignItems::End);
    style.justify_content = JustifyContent::SpaceEvenly;
    style.gap = Length::Px(12.0);
    style.row_gap = Length::Px(10.0);
    style.column_gap = Length::Px(14.0);
    style.grid_template_rows = vec![GridTemplateComponent::Repeat(GridTemplateRepetition {
        count: RepetitionCount::Count(2),
        tracks: vec![fr::<_, GridTrack>(1.0)],
        line_names: vec![vec!["row".to_string()]],
    })];
    style.grid_template_columns = vec![
        GridTemplateComponent::Single(length::<_, GridTrack>(96.0)),
        GridTemplateComponent::Single(fr::<_, GridTrack>(1.0)),
    ];
    style.grid_auto_rows = vec![length::<_, GridTrack>(24.0)];
    style.grid_auto_columns = vec![fr::<_, GridTrack>(2.0)];
    style.grid_auto_flow = GridAutoFlow::ColumnDense;
    style.grid_template_areas = vec![GridTemplateArea {
        name: "main".to_string(),
        row_start: 0,
        row_end: 1,
        column_start: 0,
        column_end: 2,
    }];
    style.grid_template_column_names = vec![vec!["start".to_string()], vec!["end".to_string()]];
    style.grid_template_row_names = vec![vec!["top".to_string()], vec!["bottom".to_string()]];
    style.grid_row = GridPlacementLine {
        start: line::<GridPlacement>(1),
        end: GridPlacement::Span(2),
    };
    style.grid_column = GridPlacementLine {
        start: GridPlacement::NamedLine("content".to_string(), 0),
        end: GridPlacement::NamedSpan("content".to_string(), 2),
    };
    style.margin = Insets::symmetric(8.0, 4.0);
    style.padding = Insets::all(6.0);
    style.width = Length::Percent(0.5);
    style.height = Length::Px(240.0);
    style.min_size = Size::new(120.0, 80.0);
    style.max_size = Size::new(640.0, 480.0);
    style.overflow_x = Overflow::Scroll;

    document.apply_computed_style("panel", &style).unwrap();

    assert_eq!(document.layout_node("panel"), Some(original_layout_node));
    let layout_style = document.layout_style("panel").unwrap();
    assert_eq!(layout_style.display, Display::Grid);
    assert_eq!(layout_style.flex_direction, LayoutFlexDirection::Row);
    assert_eq!(layout_style.flex_wrap, des_layout::prelude::FlexWrap::Wrap);
    assert_eq!(layout_style.flex_basis, length::<_, Dimension>(88.0));
    assert_eq!(layout_style.flex_grow, 2.0);
    assert_eq!(layout_style.flex_shrink, 0.5);
    assert_eq!(
        layout_style.align_content,
        Some(LayoutAlignContent::SpaceAround)
    );
    assert_eq!(layout_style.align_items, Some(LayoutAlignItems::FlexEnd));
    assert_eq!(layout_style.align_self, Some(LayoutAlignItems::Baseline));
    assert_eq!(layout_style.justify_items, Some(LayoutAlignItems::Center));
    assert_eq!(layout_style.justify_self, Some(LayoutAlignItems::End));
    assert_eq!(
        layout_style.justify_content,
        Some(LayoutJustifyContent::SpaceEvenly)
    );
    assert_eq!(
        layout_style.gap,
        LayoutSize {
            width: length(14.0),
            height: length(10.0),
        }
    );
    assert_eq!(layout_style.grid_template_rows, style.grid_template_rows);
    assert_eq!(
        layout_style.grid_template_columns,
        style.grid_template_columns
    );
    assert_eq!(layout_style.grid_auto_rows, style.grid_auto_rows);
    assert_eq!(layout_style.grid_auto_columns, style.grid_auto_columns);
    assert_eq!(layout_style.grid_auto_flow, GridAutoFlow::ColumnDense);
    assert_eq!(layout_style.grid_template_areas, style.grid_template_areas);
    assert_eq!(
        layout_style.grid_template_column_names,
        style.grid_template_column_names
    );
    assert_eq!(
        layout_style.grid_template_row_names,
        style.grid_template_row_names
    );
    assert_eq!(layout_style.grid_row, style.grid_row);
    assert_eq!(layout_style.grid_column, style.grid_column);
    assert_eq!(
        layout_style.margin,
        des_layout::prelude::Rect {
            left: length::<_, LengthPercentageAuto>(8.0),
            right: length::<_, LengthPercentageAuto>(8.0),
            top: length::<_, LengthPercentageAuto>(4.0),
            bottom: length::<_, LengthPercentageAuto>(4.0),
        }
    );
    assert_eq!(layout_style.padding, des_layout::prelude::Rect::length(6.0));
    assert_eq!(
        layout_style.size,
        LayoutSize {
            width: percent::<_, Dimension>(0.5),
            height: length::<_, Dimension>(240.0),
        }
    );
    assert_eq!(
        layout_style.min_size,
        LayoutSize {
            width: length::<_, Dimension>(120.0),
            height: length::<_, Dimension>(80.0),
        }
    );
    assert_eq!(
        layout_style.max_size,
        LayoutSize {
            width: length::<_, Dimension>(640.0),
            height: length::<_, Dimension>(480.0),
        }
    );
    assert_eq!(layout_style.overflow.x, LayoutOverflow::Scroll);
    assert_eq!(layout_style.overflow.y, LayoutOverflow::Hidden);
}

#[test]
fn document_fill_size_does_not_imply_flex_layout_style() {
    let mut document = Document::new(Size::new(800.0, 600.0));
    document
        .append_element("root", "panel", ElementSpec::new(Element::Div))
        .unwrap();

    let mut style = ComputedStyle::default();
    style.width = Length::Fill;
    style.height = Length::Fill;

    document.apply_computed_style("panel", &style).unwrap();

    let layout_style = document.layout_style("panel").unwrap();
    assert_eq!(layout_style.align_self, None);
    assert_eq!(layout_style.flex_grow, 0.0);
}

#[test]
fn document_stylesheet_applies_grid_style_surface() {
    let mut document = Document::new(Size::new(320.0, 200.0));
    document
        .append_element("root", "grid", ElementSpec::new(Element::Div))
        .unwrap();
    document
        .append_element("grid", "cell", ElementSpec::new(Element::Div))
        .unwrap();
    let columns = vec![
        GridTemplateComponent::Single(length::<_, GridTrack>(48.0)),
        GridTemplateComponent::Single(fr::<_, GridTrack>(1.0)),
    ];
    let row = GridPlacementLine {
        start: line::<GridPlacement>(1),
        end: GridPlacement::Span(2),
    };
    let column = GridPlacementLine {
        start: line::<GridPlacement>(2),
        end: line::<GridPlacement>(3),
    };
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("grid"),
            Style::default()
                .display(Display::Grid)
                .grid_template_columns(columns.clone())
                .grid_auto_rows(vec![length::<_, GridTrack>(32.0)])
                .grid_auto_flow(GridAutoFlow::RowDense)
                .justify_items(AlignItems::Stretch),
        )
        .rule(
            StyleSelector::id("cell"),
            Style::default()
                .grid_row_line(row.clone())
                .grid_column_line(column.clone())
                .justify_self(AlignItems::Center),
        );

    let states = HashMap::new();
    document.apply_stylesheet(&stylesheet, &states).unwrap();

    let grid_style = document.layout_style("grid").unwrap();
    assert_eq!(grid_style.display, Display::Grid);
    assert_eq!(grid_style.grid_template_columns, columns);
    assert_eq!(
        grid_style.grid_auto_rows,
        vec![length::<_, GridTrack>(32.0)]
    );
    assert_eq!(grid_style.grid_auto_flow, GridAutoFlow::RowDense);
    assert_eq!(grid_style.justify_items, Some(LayoutAlignItems::Stretch));

    let cell_style = document.layout_style("cell").unwrap();
    assert_eq!(cell_style.grid_row, row);
    assert_eq!(cell_style.grid_column, column);
    assert_eq!(cell_style.justify_self, Some(LayoutAlignItems::Center));
}

#[test]
fn document_computes_layout_rects_from_retained_graph() {
    let mut document = Document::new(Size::new(800.0, 600.0));
    document
        .append_element("root", "panel", ElementSpec::new(Element::Div))
        .unwrap();

    let mut style = ComputedStyle::default();
    style.width = Length::Px(200.0);
    style.height = Length::Px(100.0);
    document.apply_computed_style("panel", &style).unwrap();

    document.compute_layout().unwrap();

    assert_eq!(
        document.layout_rect("root").unwrap(),
        Rect::new(0.0, 0.0, 800.0, 600.0)
    );
    assert_eq!(
        document.layout_rect("panel").unwrap(),
        Rect::new(0.0, 0.0, 200.0, 100.0)
    );
}

#[test]
fn document_resolves_stylesheet_over_retained_elements() {
    let mut document = Document::new(Size::new(800.0, 600.0));
    document
        .append_element(
            "root",
            "first",
            ElementSpec::new(Element::Div).class("primary"),
        )
        .unwrap();
    document
        .append_element("root", "second", ElementSpec::new(Element::Div))
        .unwrap();
    let first_node = document.layout_node("first").unwrap();
    let second_node = document.layout_node("second").unwrap();

    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::Element(Element::Root),
            Style::default().flex_direction(FlexDirection::Row),
        )
        .rule(
            StyleSelector::class("primary"),
            Style::default().width(Length::Px(120.0)),
        )
        .rule(
            StyleSelector::first_child(),
            Style::default().height(Length::Px(40.0)),
        )
        .rule(
            StyleSelector::id_state("second", ElementStateSelector::Hovered),
            Style::default().width(Length::Px(240.0)),
        );
    let mut states = HashMap::new();
    let mut second_state = ElementState::default();
    second_state.hovered = true;
    states.insert(ElementId::new("second"), second_state);

    document.apply_stylesheet(&stylesheet, &states).unwrap();

    assert_eq!(document.layout_node("first"), Some(first_node));
    assert_eq!(document.layout_node("second"), Some(second_node));
    assert_eq!(
        document.layout_style("root").unwrap().flex_direction,
        LayoutFlexDirection::Row
    );
    assert_eq!(
        document.layout_style("root").unwrap().size,
        LayoutSize {
            width: length::<_, Dimension>(800.0),
            height: length::<_, Dimension>(600.0),
        }
    );
    assert_eq!(
        document.layout_style("first").unwrap().size,
        LayoutSize {
            width: length::<_, Dimension>(120.0),
            height: length::<_, Dimension>(40.0),
        }
    );
    assert_eq!(
        document.layout_style("second").unwrap().size.width,
        length::<_, Dimension>(240.0)
    );
}

#[test]
fn css_stylesheet_parser_resolves_supported_selectors_and_properties() {
    let mut document = Document::new(Size::new(800.0, 600.0));
    document
        .append_text(
            "root",
            "title",
            ElementSpec::new(Element::Text).class("section-title"),
            "Parsed CSS",
        )
        .unwrap();
    let stylesheet = StyleSheet::parse_css(
        r#"
        text.section-title {
            width: 100%;
            height: auto;
            color: #6750a4;
            font-size: 18px;
            flex-basis: calc(50% - 8px);
            text-wrap: wrap;
            max-lines: 2;
            text-selection-background: #6750a4dc;
            text-selection-color: #fffbfe;
            box-shadow: 0px 4px 10px 0px #0000003a, 0px 1px 2px rgba(103, 80, 164, 0.4);
            transition: all 0.24s ease-out;
            scrollbar-expanded-width: 10px;
            scrollbar-handle-color: rgba(103, 80, 164, 0.46);
            scrollbar-track-color: rgba(103, 80, 164, 0.11);
            scrollbar-hover-track-color: rgba(103, 80, 164, 0.11);
            scrollbar-pressed-track-color: rgba(103, 80, 164, 0.11);
            scrollbar-pressed-handle-color: rgba(103, 80, 164, 0.69);
            scrollbar-pressed-handle-border-color: rgba(255, 251, 254, 0.71);
            scrollbar-pressed-handle-border-width: 1px;
            scrollbar-radius: 6px;
            animate-size: false;
            animate-paint: false;
            animate-shadows: false;
            padding: 4px 8px;
        }

        #title:hover {
            background: #eee5ff;
            border: 1px solid #6750a4;
            border-left-width: 4px;
            border-bottom-width: 3px;
            border-top-right-radius: 12px;
            border-bottom-left-radius: 2px;
        }
        "#,
    )
    .unwrap();
    let mut states = HashMap::new();
    let mut title_state = ElementState::default();
    title_state.hovered = true;
    states.insert(ElementId::new("title"), title_state);

    document.apply_stylesheet(&stylesheet, &states).unwrap();
    document.compute_layout().unwrap();
    let title = document
        .resolved_layout()
        .unwrap()
        .find("title")
        .unwrap()
        .clone();

    assert_eq!(stylesheet.rule_count(), 2);
    assert_eq!(title.style.width, Length::Fill);
    assert_eq!(title.style.height, Length::Auto);
    assert_eq!(title.style.text_color, Color::rgb(103, 80, 164));
    assert_eq!(title.style.flex_basis, Length::calc(0.5, -8.0));
    assert_eq!(title.style.text_layout.max_lines, Some(2));
    assert_eq!(
        title.style.text_selection_background,
        Color::rgba(103, 80, 164, 220)
    );
    assert_eq!(title.style.text_selection_color, Color::rgb(255, 251, 254));
    assert_eq!(
        title.style.shadows,
        vec![
            Shadow {
                offset: Point::new(0.0, 4.0),
                blur: 10.0,
                spread: 0.0,
                color: Color::rgba(0, 0, 0, 58),
            },
            Shadow {
                offset: Point::new(0.0, 1.0),
                blur: 2.0,
                spread: 0.0,
                color: Color::rgba(103, 80, 164, 102),
            },
        ]
    );
    assert_eq!(title.style.transition, Some(Transition::ease_out(0.24)));
    assert_eq!(title.style.scrollbar_expanded_width, 10.0);
    assert_eq!(
        title.style.scrollbar_handle_color,
        Color::rgba(103, 80, 164, 117)
    );
    assert_eq!(
        title.style.scrollbar_track_color,
        Some(Color::rgba(103, 80, 164, 28))
    );
    assert_eq!(
        title.style.scrollbar_hover_track_color,
        Some(Color::rgba(103, 80, 164, 28))
    );
    assert_eq!(
        title.style.scrollbar_pressed_handle_color,
        Some(Color::rgba(103, 80, 164, 176))
    );
    assert_eq!(
        title.style.scrollbar_pressed_handle_border_color,
        Some(Color::rgba(255, 251, 254, 181))
    );
    assert_eq!(title.style.scrollbar_pressed_handle_border_width, Some(1.0));
    assert_eq!(title.style.scrollbar_radius, 6.0);
    assert!(!title.style.animate_size);
    assert!(!title.style.animate_paint);
    assert!(!title.style.animate_shadows);
    assert_eq!(title.style.background, Some(Color::rgb(238, 229, 255)));
    assert_eq!(title.style.border, Some(Color::rgb(103, 80, 164)));
    assert_eq!(title.style.border_width.top, 1.0);
    assert_eq!(title.style.border_width.left, 4.0);
    assert_eq!(title.style.border_width.bottom, 3.0);
    assert_eq!(title.style.radius.top_right, 12.0);
    assert_eq!(title.style.radius.bottom_left, 2.0);
    assert_eq!(title.style.padding.left, 8.0);
}

#[test]
fn css_stylesheet_parser_resolves_descendant_selectors() {
    let mut document = Document::new(Size::new(800.0, 600.0));
    document
        .append_element(
            "root",
            "section",
            ElementSpec::new(Element::Section).class("section"),
        )
        .unwrap();
    document
        .append_element(
            "section",
            "panel",
            ElementSpec::new(Element::Div).class("panel"),
        )
        .unwrap();
    document
        .append_text(
            "panel",
            "nested-title",
            ElementSpec::new(Element::Text).class("title"),
            "Nested title",
        )
        .unwrap();
    document
        .append_text(
            "root",
            "loose-title",
            ElementSpec::new(Element::Text).class("title"),
            "Loose title",
        )
        .unwrap();

    let stylesheet = StyleSheet::parse_css(
        r#"
        .section .panel text.title {
            width: 240px;
        }

        .section:hover text.title {
            color: #6750a4;
        }
        "#,
    )
    .unwrap();
    let mut states = HashMap::new();
    let mut section_state = ElementState::default();
    section_state.hovered = true;
    states.insert(ElementId::new("section"), section_state);

    document.apply_stylesheet(&stylesheet, &states).unwrap();

    assert_eq!(
        document.layout_style("nested-title").unwrap().size.width,
        length::<_, Dimension>(240.0)
    );
    assert_eq!(
        document
            .resolved_layout()
            .unwrap()
            .find("nested-title")
            .unwrap()
            .style
            .text_color,
        Color::rgb(103, 80, 164)
    );
    assert_eq!(
        document.layout_style("loose-title").unwrap().size.width,
        Dimension::auto()
    );
}

#[test]
fn css_stylesheet_parser_resolves_child_combinators() {
    let mut document = Document::new(Size::new(800.0, 600.0));
    document
        .append_element(
            "root",
            "panel",
            ElementSpec::new(Element::Div).class("panel"),
        )
        .unwrap();
    document
        .append_text(
            "panel",
            "direct-title",
            ElementSpec::new(Element::Text).class("title"),
            "Direct title",
        )
        .unwrap();
    document
        .append_element(
            "panel",
            "nested-wrap",
            ElementSpec::new(Element::Div).class("wrap"),
        )
        .unwrap();
    document
        .append_text(
            "nested-wrap",
            "nested-title",
            ElementSpec::new(Element::Text).class("title"),
            "Nested title",
        )
        .unwrap();

    let stylesheet = StyleSheet::parse_css(
        r#"
        .panel > text.title {
            width: 220px;
        }

        .panel > .wrap text.title {
            height: 32px;
        }
        "#,
    )
    .unwrap();

    document
        .apply_stylesheet(&stylesheet, &HashMap::new())
        .unwrap();

    assert_eq!(
        document.layout_style("direct-title").unwrap().size.width,
        length::<_, Dimension>(220.0)
    );
    assert_eq!(
        document.layout_style("nested-title").unwrap().size.width,
        Dimension::auto()
    );
    assert_eq!(
        document.layout_style("nested-title").unwrap().size.height,
        length::<_, Dimension>(32.0)
    );
}

#[test]
fn css_stylesheet_parser_accepts_box_shadow_none() {
    let stylesheet = StyleSheet::parse_css(".panel { box-shadow: none; }").unwrap();
    let mut document = Document::new(Size::new(800.0, 600.0));
    document
        .append_element(
            "root",
            "panel",
            ElementSpec::new(Element::Div).class("panel"),
        )
        .unwrap();

    document
        .apply_stylesheet(&stylesheet, &HashMap::new())
        .unwrap();

    assert_eq!(
        document
            .resolved_layout()
            .unwrap()
            .find("panel")
            .unwrap()
            .style
            .shadows,
        Vec::<Shadow>::new()
    );
}

#[test]
fn css_stylesheet_parser_accepts_transition_variants() {
    let stylesheet = StyleSheet::parse_css(
        r#"
        .linear { transition: all 120ms linear; }
        .implicit-easing { transition: 0.5s; }
        .disabled { transition: none; }
        "#,
    )
    .unwrap();

    let mut document = Document::new(Size::new(800.0, 600.0));
    for class in ["linear", "implicit-easing", "disabled"] {
        document
            .append_element("root", class, ElementSpec::new(Element::Div).class(class))
            .unwrap();
    }

    document
        .apply_stylesheet(&stylesheet, &HashMap::new())
        .unwrap();

    let layout = document.resolved_layout().unwrap();
    assert_eq!(
        layout.find("linear").unwrap().style.transition,
        Some(Transition::linear(0.12))
    );
    assert_eq!(
        layout.find("implicit-easing").unwrap().style.transition,
        Some(Transition {
            step: 0.5,
            easing: Easing::Linear,
        })
    );
    assert_eq!(layout.find("disabled").unwrap().style.transition, None);
}

#[test]
fn css_stylesheet_parser_accepts_anchor_and_floating_metadata() {
    let stylesheet = StyleSheet::parse_css(
        r#"
        #legacy {
            position: absolute-parent;
            anchor-bottom-start: anchor 0px -1px;
        }

        #popover {
            floating-to: anchor;
            floating-placement: top-start;
            floating-offset: 0px 8px;
            floating-boundary-to: boundary;
            floating-shift: cross;
            floating-flip: true;
            floating-alignment-axis-offset: floating-width(-1);
        }
        "#,
    )
    .unwrap();

    let mut document = Document::new(Size::new(800.0, 600.0));
    for id in ["anchor", "boundary", "legacy", "popover"] {
        document
            .append_element("root", id, ElementSpec::new(Element::Div))
            .unwrap();
    }
    document
        .apply_stylesheet(&stylesheet, &HashMap::new())
        .unwrap();

    let layout = document.resolved_layout().unwrap();
    let legacy_anchor = layout
        .find("legacy")
        .unwrap()
        .style
        .anchor
        .as_ref()
        .unwrap();
    assert_eq!(legacy_anchor.target, ElementId::new("anchor"));
    assert_eq!(
        legacy_anchor.options.placement,
        FloatingPlacement::BottomStart
    );
    assert_eq!(legacy_anchor.options.offset.main_axis.px, -1.0);

    let popover_anchor = layout
        .find("popover")
        .unwrap()
        .style
        .anchor
        .as_ref()
        .unwrap();
    assert_eq!(popover_anchor.target, ElementId::new("anchor"));
    assert_eq!(
        popover_anchor.boundary_target,
        Some(ElementId::new("boundary"))
    );
    assert_eq!(
        popover_anchor.options.placement,
        FloatingPlacement::TopStart
    );
    assert_eq!(popover_anchor.options.offset.cross_axis.px, 8.0);
    assert_eq!(
        popover_anchor.options.shift,
        Some(FloatingShift::new(false, true))
    );
    assert!(popover_anchor.options.flip);
    assert_eq!(
        popover_anchor.options.offset.alignment_axis,
        Some(FloatingAxisOffset::floating_width(-1.0))
    );
}

#[test]
fn css_stylesheet_parser_resolves_media_rules_and_universal_selectors() {
    let css = r#"
        * {
            color: #111111;
        }

        .panel {
            width: 320px;
            height: 48px;
        }

        @media screen and (max-width: 700px) {
            .panel {
                width: 180px;
                background: #cdf0dd;
            }

            @media (min-height: 400px) {
                #panel {
                    height: 72px;
                }
            }
        }
    "#;
    let stylesheet = StyleSheet::parse_css(css).unwrap();
    let mut narrow_document = Document::new(Size::new(640.0, 480.0));
    narrow_document
        .append_element(
            "root",
            "panel",
            ElementSpec::new(Element::Div).class("panel"),
        )
        .unwrap();
    narrow_document
        .apply_stylesheet(&stylesheet, &HashMap::new())
        .unwrap();
    narrow_document.compute_layout().unwrap();

    let narrow_layout = narrow_document.resolved_layout().unwrap();
    let panel = narrow_layout.find("panel").unwrap();
    assert_eq!(stylesheet.rule_count(), 4);
    assert_eq!(panel.style.text_color, Color::rgb(17, 17, 17));
    assert_eq!(panel.style.background, Some(Color::rgb(205, 240, 221)));
    assert_eq!(
        narrow_document.layout_rect("panel").unwrap().size,
        Size::new(180.0, 72.0)
    );

    let mut wide_document = Document::new(Size::new(900.0, 480.0));
    wide_document
        .append_element(
            "root",
            "panel",
            ElementSpec::new(Element::Div).class("panel"),
        )
        .unwrap();
    wide_document
        .apply_stylesheet(&stylesheet, &HashMap::new())
        .unwrap();
    wide_document.compute_layout().unwrap();

    let wide_layout = wide_document.resolved_layout().unwrap();
    let panel = wide_layout.find("panel").unwrap();
    assert_eq!(panel.style.text_color, Color::rgb(17, 17, 17));
    assert_eq!(panel.style.background, None);
    assert_eq!(
        wide_document.layout_rect("panel").unwrap().size,
        Size::new(320.0, 48.0)
    );
}

#[test]
fn css_stylesheet_parser_resolves_container_rules() {
    let stylesheet = StyleSheet::parse_css(
        r#"
        .parent {
            width: 300px;
            height: 80px;
        }

        .child {
            width: 80px;
            height: 24px;
        }

        @container (max-width: 350px) and (min-height: 40px) {
            .child {
                width: 180px;
                background: #cdf0dd;
            }
        }
        "#,
    )
    .unwrap();
    let mut document = Document::new(Size::new(800.0, 600.0));
    document
        .append_element(
            "root",
            "parent",
            ElementSpec::new(Element::Div).class("parent"),
        )
        .unwrap();
    document
        .append_element(
            "parent",
            "child",
            ElementSpec::new(Element::Div).class("child"),
        )
        .unwrap();

    let output = DocumentEngine::default().update(&mut document, &stylesheet);
    let child = output.layout.find("child").unwrap();
    assert_eq!(stylesheet.rule_count(), 3);
    assert_eq!(child.rect.size, Size::new(180.0, 24.0));
    assert_eq!(child.style.background, Some(Color::rgb(205, 240, 221)));
}

#[test]
fn css_stylesheet_parser_reports_unclosed_comments_and_blocks() {
    assert!(StyleSheet::parse_css("/* unclosed").is_err());
    assert!(StyleSheet::parse_css(".panel { width: 100px;").is_err());
    assert!(StyleSheet::parse_css("@media (max-width: 700px) { .panel { width: 100px; }").is_err());
}

#[test]
fn document_engine_invalidates_cached_layout_when_stylesheet_key_changes() {
    let mut document = Document::new(Size::new(800.0, 600.0));
    document
        .append_element(
            "root",
            "panel",
            ElementSpec::new(Element::Div).class("panel"),
        )
        .unwrap();
    let mut stylesheet = StyleSheet::parse_css(".panel { width: 120px; height: 40px; }").unwrap();
    let mut engine = DocumentEngine::default();

    let first = engine.update(&mut document, &stylesheet);
    let warm = engine.update(&mut document, &stylesheet);

    assert!(first.metrics.style_nodes_visited > 0);
    assert!(warm.metrics.reused_cached_layout);
    assert_eq!(warm.metrics.style_nodes_visited, 0);

    stylesheet.push_rule(
        StyleSelector::id("panel"),
        Style::default().width(Length::Px(220.0)),
    );
    let changed = engine.update(&mut document, &stylesheet);

    assert!(!changed.metrics.reused_cached_layout);
    assert!(changed.metrics.style_nodes_visited > 0);
    assert_eq!(
        changed.layout.find("panel").unwrap().rect.size,
        Size::new(220.0, 40.0)
    );
}

#[test]
fn css_stylesheet_scales_to_many_rules_on_default_stack() {
    let mut css = String::new();
    for index in 0..10_000 {
        css.push_str(&format!(
            ".rule-{index} {{ width: 100%; height: auto; font-size: 13px; }}\n"
        ));
    }
    css.push_str(".target { width: 240px; height: 32px; background: #cdf0dd; }\n");
    let stylesheet = StyleSheet::parse_css(&css).unwrap();

    let mut document = Document::new(Size::new(1024.0, 768.0));
    for index in 0..1_000 {
        let class = if index == 777 {
            "target".to_string()
        } else {
            format!("node-{index}")
        };
        document
            .append_element(
                "root",
                format!("node-{index}"),
                ElementSpec::new(Element::Div).class(class),
            )
            .unwrap();
    }

    document
        .apply_stylesheet(&stylesheet, &HashMap::new())
        .unwrap();
    document.compute_layout().unwrap();

    assert_eq!(stylesheet.rule_count(), 10_001);
    assert_eq!(
        document.layout_rect("node-777").unwrap().size,
        Size::new(240.0, 32.0)
    );
}

#[test]
fn document_does_not_dirty_layout_graph_when_resolved_layout_style_is_unchanged() {
    let mut document = Document::new(Size::new(800.0, 600.0));
    document
        .append_element("root", "panel", ElementSpec::new(Element::Div))
        .unwrap();
    let stylesheet = StyleSheet::new().rule(
        StyleSelector::id("panel"),
        Style::default()
            .width(Length::Px(120.0))
            .height(Length::Px(40.0)),
    );

    let first_report = document
        .apply_stylesheet(&stylesheet, &HashMap::new())
        .unwrap();
    document.compute_layout().unwrap();
    assert!(first_report.layout_changed);
    assert_eq!(first_report.visited, 2);
    assert!(!document.layout_dirty("root").unwrap());
    assert!(!document.layout_dirty("panel").unwrap());

    let second_report = document
        .apply_stylesheet(&stylesheet, &HashMap::new())
        .unwrap();

    assert!(!second_report.changed());
    assert_eq!(second_report.visited, 2);
    assert!(!document.layout_dirty("root").unwrap());
    assert!(!document.layout_dirty("panel").unwrap());
}

#[test]
fn document_paint_only_style_update_does_not_dirty_layout_graph() {
    let mut document = Document::new(Size::new(800.0, 600.0));
    document
        .append_element("root", "panel", ElementSpec::new(Element::Div))
        .unwrap();
    let layout_stylesheet = StyleSheet::new().rule(
        StyleSelector::id("panel"),
        Style::default()
            .width(Length::Px(120.0))
            .height(Length::Px(40.0)),
    );
    document
        .apply_stylesheet(&layout_stylesheet, &HashMap::new())
        .unwrap();
    document.compute_layout().unwrap();

    let paint_stylesheet = layout_stylesheet.clone().rule(
        StyleSelector::id("panel"),
        Style::default().background(Color::rgb(16, 24, 32)),
    );
    let report = document
        .apply_stylesheet(&paint_stylesheet, &HashMap::new())
        .unwrap();

    assert!(report.paint_changed);
    assert!(!report.layout_changed);
    assert!(!document.layout_dirty("root").unwrap());
    assert!(!document.layout_dirty("panel").unwrap());
    assert_eq!(
        document
            .resolved_layout()
            .unwrap()
            .find("panel")
            .unwrap()
            .style
            .background,
        Some(Color::rgb(16, 24, 32))
    );
}

#[test]
fn document_layout_style_update_dirties_changed_node_and_ancestors() {
    let mut document = Document::new(Size::new(800.0, 600.0));
    document
        .append_element("root", "panel", ElementSpec::new(Element::Div))
        .unwrap();
    let stylesheet = StyleSheet::new().rule(
        StyleSelector::id("panel"),
        Style::default()
            .width(Length::Px(120.0))
            .height(Length::Px(40.0)),
    );
    document
        .apply_stylesheet(&stylesheet, &HashMap::new())
        .unwrap();
    document.compute_layout().unwrap();

    let changed_stylesheet = StyleSheet::new().rule(
        StyleSelector::id("panel"),
        Style::default()
            .width(Length::Px(240.0))
            .height(Length::Px(40.0)),
    );
    document
        .apply_stylesheet(&changed_stylesheet, &HashMap::new())
        .unwrap();

    assert!(document.layout_dirty("root").unwrap());
    assert!(document.layout_dirty("panel").unwrap());
}

#[test]
fn document_emits_resolved_element_tree_from_retained_layout() {
    let mut document = Document::new(Size::new(800.0, 600.0));
    document
        .append_element(
            "root",
            "panel",
            ElementSpec::new(Element::Div)
                .class("primary")
                .interactive(),
        )
        .unwrap();
    document
        .append_text(
            "panel",
            "label",
            ElementSpec::new(Element::Text).selectable_text(),
            "Retained text",
        )
        .unwrap();

    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("panel"),
            Style::default()
                .size(200.0, 100.0)
                .padding(Insets::all(10.0)),
        )
        .rule(
            StyleSelector::id("label"),
            Style::default().size(80.0, 20.0),
        );

    document
        .apply_stylesheet(&stylesheet, &HashMap::new())
        .unwrap();
    document.compute_layout().unwrap();

    let root = document.resolved_layout().unwrap();
    let panel = root.find("panel").unwrap();
    let label = root.find("label").unwrap();

    assert_eq!(root.rect, Rect::new(0.0, 0.0, 800.0, 600.0));
    assert_eq!(panel.element, Element::Div);
    assert_eq!(panel.classes, vec!["primary".into()]);
    assert_eq!(panel.rect, Rect::new(0.0, 0.0, 200.0, 100.0));
    assert_eq!(panel.style.padding, Insets::all(10.0));
    assert!(panel.interactive);
    assert_eq!(
        label.text.as_ref().map(|text| text.semantic_text()),
        Some("Retained text")
    );
    assert_eq!(label.rect, Rect::new(10.0, 10.0, 80.0, 20.0));
    assert!(label.selectable_text);
    assert!(label.copyable_text);
}

#[test]
fn document_emits_text_layout_from_retained_layout() {
    let mut document = Document::new(Size::new(800.0, 600.0));
    document
        .append_text(
            "root",
            "label",
            ElementSpec::new(Element::Text),
            "Retained text",
        )
        .unwrap();
    let stylesheet = StyleSheet::new().rule(
        StyleSelector::id("label"),
        Style::default()
            .size(120.0, 40.0)
            .padding(Insets::all(8.0))
            .border_width(2.0)
            .text_wrap_mode(TextWrapMode::Wrap),
    );
    let mut text_measurer = RecordingTextMeasurer::default();

    document
        .apply_stylesheet(&stylesheet, &HashMap::new())
        .unwrap();
    document.compute_layout().unwrap();
    let root = document
        .resolved_layout_with_text_measurer(&mut text_measurer)
        .unwrap();
    let label = root.find("label").unwrap();

    assert_eq!(
        label.text_layout,
        Some(TextLayoutResult::new(Size::new(64.0, 18.0), 1, false))
    );
    assert_eq!(
        label
            .normalized_text
            .as_ref()
            .map(|text| text.semantic_text()),
        Some("Retained text")
    );
    assert_eq!(
        text_measurer.requests,
        vec![("Retained text".to_string(), 100.0)]
    );
}

#[test]
fn document_uses_text_measurement_for_auto_sized_text_layout() {
    let mut document = Document::new(Size::new(800.0, 600.0));
    document
        .append_text("root", "label", ElementSpec::new(Element::Text), "Measured")
        .unwrap();
    let stylesheet = StyleSheet::new().rule(
        StyleSelector::id("label"),
        Style::default().padding(Insets::all(8.0)).border_width(2.0),
    );
    let mut text_measurer = RecordingTextMeasurer::default();

    document
        .apply_stylesheet(&stylesheet, &HashMap::new())
        .unwrap();
    document
        .compute_layout_with_text_measurer(&mut text_measurer)
        .unwrap();
    let label = document
        .resolved_layout_with_text_measurer(&mut text_measurer)
        .unwrap()
        .find("label")
        .unwrap()
        .clone();

    assert_eq!(label.rect, Rect::new(0.0, 0.0, 84.0, 38.0));
}

#[test]
fn document_resolves_styles_computes_layout_and_emits_tree_in_one_pass() {
    let mut document = Document::new(Size::new(800.0, 600.0));
    document
        .append_element("root", "panel", ElementSpec::new(Element::Div))
        .unwrap();
    let stylesheet = StyleSheet::new().rule(
        StyleSelector::id("panel"),
        Style::default().size(320.0, 180.0),
    );

    let root = document
        .resolve_layout(&stylesheet, &HashMap::new())
        .unwrap();

    assert_eq!(
        root.find("panel").unwrap().rect,
        Rect::new(0.0, 0.0, 320.0, 180.0)
    );
}

#[test]
fn document_resolve_layout_skips_compute_when_layout_graph_is_clean() {
    COMPUTE_TEXT_MEASURE_COUNT.store(0, Ordering::SeqCst);
    let mut document = Document::new(Size::new(800.0, 600.0));
    document
        .append_text("root", "label", ElementSpec::new(Element::Text), "Measured")
        .unwrap();
    let stylesheet = StyleSheet::new().rule(
        StyleSelector::id("label"),
        Style::default().width(Length::Px(100.0)),
    );
    let mut text_measurer = CountingTextMeasurer;

    document
        .resolve_layout_with_text_measurer(&stylesheet, &HashMap::new(), &mut text_measurer)
        .unwrap();
    let first_resolve_count = COMPUTE_TEXT_MEASURE_COUNT.load(Ordering::SeqCst);
    assert!(first_resolve_count > 1);

    document
        .resolve_layout_with_text_measurer(&stylesheet, &HashMap::new(), &mut text_measurer)
        .unwrap();

    assert_eq!(
        COMPUTE_TEXT_MEASURE_COUNT.load(Ordering::SeqCst),
        first_resolve_count + 1
    );
}

#[test]
fn document_engine_can_update_from_retained_document() {
    let mut document = Document::new(Size::new(800.0, 600.0));
    document
        .append_element("root", "panel", ElementSpec::new(Element::Div))
        .unwrap();
    let stylesheet = StyleSheet::new().rule(
        StyleSelector::id("panel"),
        Style::default().size(320.0, 180.0),
    );
    let mut engine = DocumentEngine::default();

    let output = engine.update(&mut document, &stylesheet);

    assert_eq!(
        output.layout.find("panel").unwrap().rect,
        Rect::new(0.0, 0.0, 320.0, 180.0)
    );
    assert_eq!(
        output.changes.created,
        vec![ElementId::new("panel"), ElementId::new("root")]
    );
    assert_eq!(output.metrics.element_count, 2);
    assert_eq!(engine.element_state("panel"), Some(&Default::default()));
}

#[test]
fn document_engine_does_not_reuse_cached_layout_for_distinct_document_with_same_revision() {
    let stylesheet = StyleSheet::new().rule(
        StyleSelector::id("label"),
        Style::default().size(160.0, 24.0),
    );
    let mut engine = DocumentEngine::default();
    let mut first = Document::build(Size::new(320.0, 200.0), |ui| {
        ui.text_element("label", ElementSpec::new(Element::Text), "first");
    });
    let mut second = Document::build(Size::new(320.0, 200.0), |ui| {
        ui.text_element("label", ElementSpec::new(Element::Text), "second");
    });

    let first_output = engine.update(&mut first, &stylesheet);
    let second_output = engine.update(&mut second, &stylesheet);

    assert_eq!(
        first_output
            .layout
            .find("label")
            .unwrap()
            .text
            .as_ref()
            .map(|text| text.semantic_text()),
        Some("first")
    );
    assert_eq!(
        second_output
            .layout
            .find("label")
            .unwrap()
            .text
            .as_ref()
            .map(|text| text.semantic_text()),
        Some("second")
    );
}

#[test]
fn document_engine_update_reports_scroll_chrome_for_overflow() {
    let mut document = Document::new(Size::new(800.0, 600.0));
    document
        .append_element("root", "scroll", ElementSpec::new(Element::Div))
        .unwrap();
    document
        .append_element("scroll", "content", ElementSpec::new(Element::Div))
        .unwrap();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("scroll"),
            Style::default()
                .size(100.0, 100.0)
                .overflow_y(Overflow::Scroll),
        )
        .rule(
            StyleSelector::id("content"),
            Style::default().size(100.0, 300.0),
        );
    let mut engine = DocumentEngine::default();

    let output = engine.update(&mut document, &stylesheet);

    assert_eq!(output.scroll_chrome.len(), 1);
    assert_eq!(output.scroll_chrome[0].element_id, ElementId::new("scroll"));
    assert_eq!(output.scroll_chrome[0].max_scroll, 200.0);
    assert_eq!(output.metrics.scroll_chrome_count, 1);
}

#[test]
fn scroll_limits_use_inner_content_viewport_after_border_and_padding() {
    let mut document = Document::new(Size::new(800.0, 600.0));
    document
        .append_element("root", "scroll", ElementSpec::new(Element::Div))
        .unwrap();
    document
        .append_element("scroll", "content", ElementSpec::new(Element::Div))
        .unwrap();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("scroll"),
            Style::default()
                .size(100.0, 100.0)
                .padding(Insets::all(10.0))
                .border_width(2.0)
                .overflow_y(Overflow::Scroll),
        )
        .rule(
            StyleSelector::id("content"),
            Style::default().size(76.0, 160.0),
        );
    let mut engine = DocumentEngine::default();

    engine.update(&mut document, &stylesheet);
    let output = engine.update_with_input(
        &mut document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(50.0, 50.0),
                primary_delta: Point::ZERO,
                primary_down: false,
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: false,
                time_seconds: 0.0,
            }),
            scroll_delta: Point::new(0.0, -1000.0),
        },
    );
    let scroll = output.layout.find("scroll").unwrap();
    let content = output.layout.find("content").unwrap();
    let content_viewport = scroll
        .rect
        .inset(scroll.style.border_width)
        .inset(scroll.style.padding);

    assert_eq!(engine.element_state("scroll").unwrap().scroll_y, 84.0);
    assert_eq!(output.scroll_chrome[0].max_scroll, 84.0);
    assert_eq!(content.rect.bottom(), content_viewport.bottom());
}

#[test]
fn document_engine_update_with_input_clicks_interactive_element() {
    let mut document = Document::new(Size::new(800.0, 600.0));
    document
        .append_element(
            "root",
            "button",
            ElementSpec::new(Element::Button).interactive(),
        )
        .unwrap();
    let stylesheet = StyleSheet::new().rule(
        StyleSelector::id("button"),
        Style::default().size(120.0, 40.0),
    );
    let mut engine = DocumentEngine::default();

    let output = engine.update_with_input(
        &mut document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(10.0, 10.0),
                primary_delta: Point::ZERO,
                primary_down: true,
                primary_pressed: true,
                primary_clicked: true,
                primary_click_count: 1,
                secondary_clicked: false,
                time_seconds: 0.0,
            }),
            scroll_delta: Point::ZERO,
        },
    );

    assert_eq!(output.hit_id, Some(ElementId::new("button")));
    assert!(engine.element_state("button").unwrap().hovered);
    assert!(engine.element_state("button").unwrap().pressed);
    assert!(
        output
            .events
            .iter()
            .any(|event| event.target == ElementId::new("button")
                && event.kind == DocumentEventKind::Clicked)
    );
}

#[test]
fn document_engine_active_text_selection_ignores_pointer_moves_inside_same_index() {
    let mut document = Document::new(Size::new(240.0, 120.0));
    document
        .append_text(
            "root",
            "label",
            ElementSpec::new(Element::Text).selectable_text(),
            "Selectable text",
        )
        .unwrap();
    let stylesheet = StyleSheet::new().rule(
        StyleSelector::id("label"),
        Style::default().size(160.0, 40.0),
    );
    let mut engine = DocumentEngine::default();
    let mut text_measurer = SlotTextMeasurer::default();

    let start = engine.update_with_input_and_text_measurer(
        &mut document,
        &stylesheet,
        drag_text_input(Point::new(4.0, 8.0), true),
        &mut text_measurer,
    );
    assert!(start.metrics.input_changed_state);

    let changed = engine.update_with_input_and_text_measurer(
        &mut document,
        &stylesheet,
        drag_text_input(Point::new(45.0, 8.0), false),
        &mut text_measurer,
    );
    let changed_selection = changed
        .text_selection
        .clone()
        .expect("dragging selectable text should update selection");
    assert!(changed.metrics.input_changed_state);
    assert_eq!(changed_selection.focus_index, 2);

    let same_index = engine.update_with_input_and_text_measurer(
        &mut document,
        &stylesheet,
        drag_text_input(Point::new(48.0, 8.0), false),
        &mut text_measurer,
    );
    let same_selection = same_index
        .text_selection
        .expect("selection should remain active while dragging");

    assert!(
        !same_index.metrics.input_changed_state,
        "moving inside the same text index should not mark input changed"
    );
    assert_eq!(same_selection.anchor_index, changed_selection.anchor_index);
    assert_eq!(same_selection.focus_index, changed_selection.focus_index);
    assert_eq!(same_selection.focus, changed_selection.focus);
    assert!(text_measurer.hit_requests >= 3);
}

#[test]
fn document_engine_update_treats_primary_click_count_as_control_click() {
    let mut document = Document::new(Size::new(800.0, 600.0));
    document
        .append_element(
            "root",
            "button",
            ElementSpec::new(Element::Button).interactive(),
        )
        .unwrap();
    let stylesheet = StyleSheet::new().rule(
        StyleSelector::id("button"),
        Style::default().size(120.0, 40.0),
    );
    let mut engine = DocumentEngine::default();

    let output = engine.update_with_input(
        &mut document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(10.0, 10.0),
                primary_delta: Point::ZERO,
                primary_down: false,
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 2,
                secondary_clicked: false,
                time_seconds: 0.0,
            }),
            scroll_delta: Point::ZERO,
        },
    );

    assert_eq!(output.hit_id, Some(ElementId::new("button")));
    assert_eq!(engine.element_state("button").unwrap().click_count, 1);
    assert!(
        output
            .events
            .iter()
            .any(|event| event.target == ElementId::new("button")
                && event.kind == DocumentEventKind::Clicked)
    );
}

#[test]
fn document_engine_update_eases_transitioned_paint_styles() {
    let mut document = Document::new(Size::new(320.0, 200.0));
    document
        .append_element("root", "card", ElementSpec::new(Element::Div).interactive())
        .unwrap();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::Element(Element::Div),
            Style::default()
                .size(100.0, 40.0)
                .background(Color::rgb(20, 20, 20))
                .transition(Transition::ease_out(0.24)),
        )
        .rule(
            StyleSelector::State(ElementStateSelector::Hovered),
            Style::default().background(Color::rgb(40, 70, 95)),
        );
    let mut engine = DocumentEngine::default();
    engine.update(&mut document, &stylesheet);

    let output = engine.update_with_input(
        &mut document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(2.0, 2.0),
                primary_delta: Point::ZERO,
                primary_down: false,
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: false,
                time_seconds: 0.0,
            }),
            scroll_delta: Point::ZERO,
        },
    );
    let card = output.layout.find("card").unwrap();

    assert_eq!(card.style.background, Some(Color::rgb(31, 48, 62)));
    assert!(output.animating);
    assert!(output.metrics.animation_changed_style);
    assert!(!output.metrics.animation_changed_layout);
    assert!(output.metrics.animation_changed_paint);

    let output = engine.update_with_input(
        &mut document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(2.0, 2.0),
                primary_delta: Point::ZERO,
                primary_down: false,
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: false,
                time_seconds: 0.0,
            }),
            scroll_delta: Point::ZERO,
        },
    );
    let card = output.layout.find("card").unwrap();

    assert!(card.style.background.unwrap().r > 31);
    assert!(output.animating);
    assert!(output.metrics.animation_changed_paint);
}

#[test]
fn document_engine_update_transitioned_paint_styles_settle_to_target() {
    let mut document = Document::new(Size::new(320.0, 200.0));
    document
        .append_element("root", "card", ElementSpec::new(Element::Div).interactive())
        .unwrap();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::Element(Element::Div),
            Style::default()
                .size(100.0, 40.0)
                .background(Color::rgb(20, 20, 20))
                .transition(Transition::ease_out(0.24)),
        )
        .rule(
            StyleSelector::State(ElementStateSelector::Hovered),
            Style::default().background(Color::rgb(40, 70, 95)),
        );
    let hover_input = hover_input(Point::new(2.0, 2.0));
    let mut engine = DocumentEngine::default();

    engine.update(&mut document, &stylesheet);
    let output = (0..30)
        .map(|_| engine.update_with_input(&mut document, &stylesheet, hover_input))
        .last()
        .unwrap();
    let card = output.layout.find("card").unwrap();

    assert_eq!(card.style.background, Some(Color::rgb(40, 70, 95)));
    assert!(!output.animating);
    assert!(!output.metrics.animation_changed_style);

    let output = engine.update_with_input(&mut document, &stylesheet, hover_input);

    assert!(!output.animating);
    assert!(!output.metrics.animation_changed_style);
}

#[test]
fn document_engine_update_eases_transitioned_layout_styles() {
    let mut document = Document::new(Size::new(320.0, 200.0));
    document
        .append_element("root", "card", ElementSpec::new(Element::Div).interactive())
        .unwrap();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::Element(Element::Div),
            Style::default()
                .size(100.0, 40.0)
                .transition(Transition::linear(0.25)),
        )
        .rule(
            StyleSelector::State(ElementStateSelector::Hovered),
            Style::default().size(140.0, 80.0),
        );
    let mut engine = DocumentEngine::default();
    engine.update(&mut document, &stylesheet);

    let output = engine.update_with_input(
        &mut document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(4.0, 4.0),
                primary_delta: Point::ZERO,
                primary_down: false,
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: false,
                time_seconds: 0.0,
            }),
            scroll_delta: Point::ZERO,
        },
    );
    let card = output.layout.find("card").unwrap();

    assert_eq!(card.rect.size, Size::new(110.0, 50.0));
    assert!(output.animating);
    assert!(output.metrics.animation_changed_layout);
}

#[test]
fn document_engine_update_eases_full_box_model_layout_styles() {
    let mut document = Document::new(Size::new(320.0, 200.0));
    document
        .append_element("root", "card", ElementSpec::new(Element::Div).interactive())
        .unwrap();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::Element(Element::Div),
            Style::default()
                .size(100.0, 40.0)
                .min_size(20.0, 20.0)
                .max_size(180.0, 120.0)
                .padding(Insets::all(4.0))
                .margin(Insets::all(2.0))
                .gap(4.0)
                .border_width(2.0)
                .radius(4.0)
                .font_size(12.0)
                .transition(Transition::linear(0.25)),
        )
        .rule(
            StyleSelector::State(ElementStateSelector::Hovered),
            Style::default()
                .size(140.0, 80.0)
                .min_size(40.0, 60.0)
                .max_size(220.0, 160.0)
                .padding(Insets::all(12.0))
                .margin(Insets::all(10.0))
                .gap(20.0)
                .border_width(10.0)
                .radius(20.0)
                .font_size(20.0),
        );
    let mut engine = DocumentEngine::default();
    engine.update(&mut document, &stylesheet);

    let output = engine.update_with_input(
        &mut document,
        &stylesheet,
        hover_input(Point::new(4.0, 4.0)),
    );
    let card = output.layout.find("card").unwrap();

    assert_eq!(card.rect.size, Size::new(110.0, 50.0));
    assert_eq!(card.style.min_size, Size::new(25.0, 30.0));
    assert_eq!(card.style.max_size, Size::new(190.0, 130.0));
    assert_eq!(card.style.padding, Insets::all(6.0));
    assert_eq!(card.style.margin, Insets::all(4.0));
    assert_eq!(card.style.gap, Length::Px(8.0));
    assert_eq!(card.style.border_width, Insets::all(4.0));
    assert_eq!(card.style.radius, CornerRadii::all(8.0));
    assert_eq!(card.style.font_size, 14.0);
    assert!(!output.metrics.reused_input_layout);
    assert!(output.metrics.animation_changed_style);
    assert!(output.metrics.animation_changed_layout);
    assert!(output.metrics.animation_changed_paint);

    let output = engine.update_with_input(
        &mut document,
        &stylesheet,
        hover_input(Point::new(4.0, 4.0)),
    );

    assert!(!output.metrics.reused_input_layout);
    assert!(output.metrics.animation_changed_layout);
}

#[test]
fn document_engine_update_untransitioned_hover_color_reuses_layout_and_updates_paint() {
    let mut document = Document::new(Size::new(320.0, 200.0));
    document
        .append_element("root", "card", ElementSpec::new(Element::Div).interactive())
        .unwrap();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::Element(Element::Div),
            Style::default()
                .size(100.0, 40.0)
                .background(Color::rgb(20, 20, 20)),
        )
        .rule(
            StyleSelector::State(ElementStateSelector::Hovered),
            Style::default().background(Color::rgb(40, 70, 95)),
        );
    let mut engine = DocumentEngine::default();

    engine.update(&mut document, &stylesheet);
    let output = engine.update_with_input(
        &mut document,
        &stylesheet,
        hover_input(Point::new(2.0, 2.0)),
    );
    let card = output.layout.find("card").unwrap();

    assert_eq!(card.style.background, Some(Color::rgb(40, 70, 95)));
    assert!(output.metrics.reused_input_layout);
    assert!(output.metrics.input_changed_state);
    assert!(output.metrics.animation_changed_style);
    assert!(!output.metrics.animation_changed_layout);
    assert!(output.metrics.animation_changed_paint);
}

#[test]
fn document_engine_repeated_hover_on_same_target_is_noop() {
    let mut document = Document::new(Size::new(320.0, 200.0));
    document
        .append_element("root", "card", ElementSpec::new(Element::Div).interactive())
        .unwrap();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::Element(Element::Div),
            Style::default()
                .size(100.0, 40.0)
                .background(Color::rgb(20, 20, 20)),
        )
        .rule(
            StyleSelector::State(ElementStateSelector::Hovered),
            Style::default().background(Color::rgb(40, 70, 95)),
        );
    let mut engine = DocumentEngine::default();
    let input = hover_input(Point::new(2.0, 2.0));

    engine.update(&mut document, &stylesheet);
    let first = engine.update_with_input(&mut document, &stylesheet, input);
    let second = engine.update_with_input(&mut document, &stylesheet, input);

    assert!(first.metrics.input_changed_state);
    assert_eq!(
        second.layout.find("card").unwrap().style.background,
        Some(Color::rgb(40, 70, 95))
    );
    assert!(!second.metrics.input_changed_state);
    assert!(second.metrics.reused_input_layout);
    assert!(!second.metrics.animation_changed_style);
    assert!(!second.metrics.animation_changed_layout);
    assert!(!second.metrics.animation_changed_paint);
}

#[test]
fn document_engine_update_untransitioned_hover_layout_change_rebuilds_layout() {
    let mut document = Document::new(Size::new(320.0, 200.0));
    document
        .append_element("root", "card", ElementSpec::new(Element::Div).interactive())
        .unwrap();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::Element(Element::Div),
            Style::default().size(100.0, 40.0),
        )
        .rule(
            StyleSelector::State(ElementStateSelector::Hovered),
            Style::default().size(140.0, 40.0),
        );
    let mut engine = DocumentEngine::default();

    engine.update(&mut document, &stylesheet);
    let output = engine.update_with_input(
        &mut document,
        &stylesheet,
        hover_input(Point::new(2.0, 2.0)),
    );
    let card = output.layout.find("card").unwrap();

    assert_eq!(card.rect.size, Size::new(140.0, 40.0));
    assert!(!output.metrics.reused_input_layout);
    assert!(output.metrics.animation_changed_style);
    assert!(output.metrics.animation_changed_layout);
}

#[test]
fn document_engine_update_snap_element_animation_clears_rendered_style() {
    let mut document = Document::new(Size::new(320.0, 200.0));
    document
        .append_element("root", "card", ElementSpec::new(Element::Div).interactive())
        .unwrap();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::Element(Element::Div),
            Style::default()
                .size(100.0, 40.0)
                .background(Color::rgb(20, 20, 20))
                .transition(Transition::ease_out(0.24)),
        )
        .rule(
            StyleSelector::State(ElementStateSelector::Hovered),
            Style::default().background(Color::rgb(40, 70, 95)),
        );
    let mut engine = DocumentEngine::default();

    engine.update(&mut document, &stylesheet);
    engine.update_with_input(
        &mut document,
        &stylesheet,
        hover_input(Point::new(2.0, 2.0)),
    );

    assert!(engine.snap_element_animation("card"));

    let output = engine.update_with_input(
        &mut document,
        &stylesheet,
        hover_input(Point::new(2.0, 2.0)),
    );
    let card = output.layout.find("card").unwrap();

    assert_eq!(card.style.background, Some(Color::rgb(40, 70, 95)));
    assert!(!output.animating);
}

#[test]
fn document_engine_update_scrollbar_hover_transition_reuses_layout() {
    let mut document = Document::new(Size::new(180.0, 140.0));
    document
        .append_element("root", "scroll-panel", ElementSpec::new(Element::Div))
        .unwrap();
    document
        .append_element("scroll-panel", "content", ElementSpec::new(Element::Div))
        .unwrap();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("scroll-panel"),
            Style::default()
                .size(70.0, 70.0)
                .overflow(Overflow::Scroll)
                .scrollbar_width(2.0)
                .scrollbar_expanded_width(10.0)
                .transition(Transition::ease_out(0.25)),
        )
        .rule(
            StyleSelector::id("content"),
            Style::default().size(70.0, 140.0),
        );
    let mut engine = DocumentEngine::default();

    engine.update(&mut document, &stylesheet);
    let output = engine.update_with_input(
        &mut document,
        &stylesheet,
        hover_input(Point::new(64.0, 20.0)),
    );
    let vertical = output
        .scroll_chrome
        .iter()
        .find(|chrome| {
            chrome.element_id == ElementId::new("scroll-panel")
                && chrome.axis == ScrollAxis::Vertical
        })
        .unwrap();

    assert!(vertical.expanded);
    assert!(vertical.handle_rect.size.width > 2.0);
    assert!(vertical.handle_rect.size.width < 10.0);
    assert!(output.animating);
    assert!(output.metrics.reused_input_layout);
    assert!(!output.metrics.animation_changed_layout);

    let output = engine.update_with_input(
        &mut document,
        &stylesheet,
        hover_input(Point::new(64.0, 20.0)),
    );

    assert!(output.metrics.reused_input_layout);
    assert!(!output.metrics.animation_changed_layout);
}

#[test]
fn document_engine_update_uses_document_text_layout() {
    let mut document = Document::new(Size::new(800.0, 600.0));
    document
        .append_text(
            "root",
            "label",
            ElementSpec::new(Element::Text),
            "Retained text",
        )
        .unwrap();
    let stylesheet = StyleSheet::new().rule(
        StyleSelector::id("label"),
        Style::default().size(120.0, 40.0),
    );
    let mut engine = DocumentEngine::default();
    let mut text_measurer = RecordingTextMeasurer::default();

    let output = engine.update_with_input_and_text_measurer(
        &mut document,
        &stylesheet,
        DocumentInput::default(),
        &mut text_measurer,
    );

    assert_eq!(
        output
            .layout
            .find("label")
            .unwrap()
            .text_layout
            .as_ref()
            .unwrap()
            .size,
        Size::new(64.0, 18.0)
    );
}

#[test]
fn document_engine_update_uses_text_measurement_for_auto_text_size() {
    let mut document = Document::new(Size::new(800.0, 600.0));
    document
        .append_text("root", "label", ElementSpec::new(Element::Text), "Measured")
        .unwrap();
    let stylesheet = StyleSheet::new();
    let mut engine = DocumentEngine::default();
    let mut text_measurer = RecordingTextMeasurer::default();

    let output = engine.update_with_input_and_text_measurer(
        &mut document,
        &stylesheet,
        DocumentInput::default(),
        &mut text_measurer,
    );

    assert_eq!(
        output.layout.find("label").unwrap().rect,
        Rect::new(0.0, 0.0, 64.0, 18.0)
    );
}

#[test]
fn document_engine_update_resolves_table_column_tracks() {
    let table = TableSpec::new(vec![
        TableColumnSpec::new("customer", "Customer").width(TableTrackSize::px(120.0)),
        TableColumnSpec::new("country", "Country").width(TableTrackSize::px(100.0)),
        TableColumnSpec::new("orders", "Orders").width(TableTrackSize::px(80.0)),
    ])
    .header_height(28.0)
    .row_height(26.0);
    let mut document = Document::new(Size::new(320.0, 220.0));
    document
        .append_element(
            "root",
            "customers",
            ElementSpec::new(Element::Table).table(table),
        )
        .unwrap();
    document
        .append_element(
            "customers",
            "customers-header",
            ElementSpec::new(Element::Thead),
        )
        .unwrap();
    document
        .append_text(
            "customers-header",
            "customers-header-customer",
            ElementSpec::new(Element::Td).table_cell(TableCellSpec::new("customer")),
            "Customer",
        )
        .unwrap();
    document
        .append_text(
            "customers-header",
            "customers-header-country",
            ElementSpec::new(Element::Td).table_cell(TableCellSpec::new("country")),
            "Country",
        )
        .unwrap();
    document
        .append_text(
            "customers-header",
            "customers-header-orders",
            ElementSpec::new(Element::Td).table_cell(TableCellSpec::new("orders")),
            "Orders",
        )
        .unwrap();
    document
        .append_element(
            "customers",
            "customers-row-0",
            ElementSpec::new(Element::Tr),
        )
        .unwrap();
    document
        .append_text(
            "customers-row-0",
            "customers-row-0-customer",
            ElementSpec::new(Element::Td).table_cell(TableCellSpec::new("customer")),
            "Acme",
        )
        .unwrap();
    document
        .append_text(
            "customers-row-0",
            "customers-row-0-country",
            ElementSpec::new(Element::Td).table_cell(TableCellSpec::new("country")),
            "US",
        )
        .unwrap();
    document
        .append_text(
            "customers-row-0",
            "customers-row-0-orders",
            ElementSpec::new(Element::Td).table_cell(TableCellSpec::new("orders")),
            "42",
        )
        .unwrap();

    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("customers"),
            Style::default()
                .width(Length::Px(240.0))
                .overflow_x(Overflow::Scroll),
        )
        .rule(
            StyleSelector::Element(Element::Td),
            Style::default().border_width(1.0),
        );
    let mut engine = DocumentEngine::default();

    let output = engine.update(&mut document, &stylesheet);
    let header_customer = output.layout.find("customers-header-customer").unwrap();
    let row_customer = output.layout.find("customers-row-0-customer").unwrap();
    let header_orders = output.layout.find("customers-header-orders").unwrap();
    let row_orders = output.layout.find("customers-row-0-orders").unwrap();

    assert_eq!(header_customer.element, Element::Td);
    assert_eq!(
        header_customer.rect.size.width,
        row_customer.rect.size.width
    );
    assert_eq!(header_orders.rect.origin.x, row_orders.rect.origin.x);
    assert_eq!(header_orders.rect.size.width, 80.0);
    assert!(row_customer.rect.origin.y > header_customer.rect.origin.y);
    assert!(output.scroll_chrome.iter().any(|chrome| {
        chrome.element_id == ElementId::new("customers")
            && chrome.axis == ScrollAxis::Horizontal
            && chrome.max_scroll == 60.0
    }));
}

#[test]
fn document_engine_update_anchors_absolute_viewport_elements_to_window() {
    let mut document = Document::new(Size::new(320.0, 200.0));
    document
        .append_element("root", "panel", ElementSpec::new(Element::Div))
        .unwrap();
    document
        .append_element("panel", "absolute-child", ElementSpec::new(Element::Div))
        .unwrap();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("panel"),
            Style::default()
                .size(120.0, 80.0)
                .padding(Insets::all(10.0)),
        )
        .rule(
            StyleSelector::id("absolute-child"),
            Style::default()
                .absolute_viewport()
                .right(Length::Px(8.0))
                .bottom(Length::Px(9.0))
                .size(40.0, 20.0),
        );
    let mut engine = DocumentEngine::default();

    let output = engine.update(&mut document, &stylesheet);
    let absolute_child = output.layout.find("absolute-child").unwrap();

    assert_eq!(absolute_child.rect.origin, Point::new(272.0, 171.0));
}

#[test]
fn document_engine_update_hits_absolute_viewport_child_outside_parent() {
    let mut document = Document::new(Size::new(320.0, 200.0));
    document
        .append_element("root", "panel", ElementSpec::new(Element::Div))
        .unwrap();
    document
        .append_element(
            "panel",
            "absolute-child",
            ElementSpec::new(Element::Div).interactive(),
        )
        .unwrap();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("panel"),
            Style::default().size(60.0, 40.0),
        )
        .rule(
            StyleSelector::id("absolute-child"),
            Style::default()
                .absolute_viewport()
                .left(Length::Px(140.0))
                .top(Length::Px(80.0))
                .size(40.0, 20.0),
        );
    let mut engine = DocumentEngine::default();

    let output = engine.update_with_input(
        &mut document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(150.0, 90.0),
                primary_delta: Point::ZERO,
                primary_down: true,
                primary_pressed: false,
                primary_clicked: true,
                primary_click_count: 1,
                secondary_clicked: false,
                time_seconds: 0.0,
            }),
            scroll_delta: Point::ZERO,
        },
    );

    assert_eq!(output.hit_id, Some(ElementId::new("absolute-child")));
    assert!(engine.element_state("absolute-child").unwrap().pressed);
}

#[test]
fn document_engine_update_positions_absolute_parent_without_flow_measurement() {
    let mut document = Document::new(Size::new(320.0, 200.0));
    document
        .append_element("root", "panel", ElementSpec::new(Element::Div))
        .unwrap();
    document
        .append_element("panel", "absolute-child", ElementSpec::new(Element::Div))
        .unwrap();
    document
        .append_element("panel", "flow-child", ElementSpec::new(Element::Div))
        .unwrap();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("panel"),
            Style::default()
                .width(Length::Auto)
                .height(Length::Auto)
                .padding(Insets::all(10.0))
                .border_width(2.0),
        )
        .rule(
            StyleSelector::id("flow-child"),
            Style::default().size(50.0, 20.0),
        )
        .rule(
            StyleSelector::id("absolute-child"),
            Style::default()
                .absolute_parent()
                .left(Length::Px(7.0))
                .top(Length::Px(5.0))
                .size(40.0, 20.0),
        );
    let mut engine = DocumentEngine::default();

    let output = engine.update(&mut document, &stylesheet);
    let panel = output.layout.find("panel").unwrap();
    let absolute_child = output.layout.find("absolute-child").unwrap();
    let flow_child = output.layout.find("flow-child").unwrap();

    assert_eq!(panel.rect.size, Size::new(74.0, 44.0));
    assert_eq!(flow_child.rect.origin, Point::new(12.0, 12.0));
    assert_eq!(absolute_child.rect.origin, Point::new(19.0, 17.0));
}

#[test]
fn document_engine_update_positions_absolute_anchor_after_target_layout() {
    let mut document = Document::new(Size::new(320.0, 200.0));
    document
        .append_element("root", "panel", ElementSpec::new(Element::Div))
        .unwrap();
    document
        .append_element("panel", "popover", ElementSpec::new(Element::Div))
        .unwrap();
    document
        .append_element("panel", "anchor", ElementSpec::new(Element::Div))
        .unwrap();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("panel"),
            Style::default()
                .width(Length::Auto)
                .height(Length::Auto)
                .padding(Insets::all(10.0))
                .border_width(2.0),
        )
        .rule(
            StyleSelector::id("anchor"),
            Style::default().size(80.0, 30.0),
        )
        .rule(
            StyleSelector::id("popover"),
            Style::default()
                .absolute_parent()
                .anchor_bottom_start("anchor", 0.0, -1.0)
                .size(60.0, 20.0),
        );
    let mut engine = DocumentEngine::default();

    let output = engine.update(&mut document, &stylesheet);
    let panel = output.layout.find("panel").unwrap();
    let anchor = output.layout.find("anchor").unwrap();
    let popover = output.layout.find("popover").unwrap();

    assert_eq!(panel.rect.size, Size::new(104.0, 54.0));
    assert_eq!(anchor.rect.origin, Point::new(12.0, 12.0));
    assert_eq!(popover.rect.origin, Point::new(12.0, 41.0));
    assert_eq!(popover.rect.size, Size::new(60.0, 20.0));
}

#[test]
fn document_engine_update_wraps_row_children_and_expands_height() {
    let mut document = Document::new(Size::new(240.0, 160.0));
    document
        .append_element("root", "row", ElementSpec::new(Element::Div))
        .unwrap();
    for index in 0..3 {
        document
            .append_element(
                "row",
                format!("item-{index}"),
                ElementSpec::new(Element::Div).class("item"),
            )
            .unwrap();
    }
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("row"),
            Style::default()
                .flex_direction(FlexDirection::Row)
                .flex_wrap(FlexWrap::Wrap)
                .width(Length::Px(120.0))
                .height(Length::Auto)
                .gap(10.0),
        )
        .rule(
            StyleSelector::class("item"),
            Style::default().size(50.0, 20.0),
        );
    let mut engine = DocumentEngine::default();

    let output = engine.update(&mut document, &stylesheet);
    let row = output.layout.find("row").unwrap();
    let item_0 = output.layout.find("item-0").unwrap();
    let item_1 = output.layout.find("item-1").unwrap();
    let item_2 = output.layout.find("item-2").unwrap();

    assert_eq!(row.rect.size, Size::new(120.0, 50.0));
    assert_eq!(item_0.rect.origin, Point::new(0.0, 0.0));
    assert_eq!(item_1.rect.origin, Point::new(60.0, 0.0));
    assert_eq!(item_2.rect.origin, Point::new(0.0, 30.0));
}

#[test]
fn document_engine_resolves_percent_column_gap_against_flex_container_width() {
    let mut document = Document::new(Size::new(320.0, 200.0));
    document
        .append_element("root", "row", ElementSpec::new(Element::Div))
        .unwrap();
    for index in 0..2 {
        document
            .append_element(
                "row",
                format!("item-{index}"),
                ElementSpec::new(Element::Div).class("item"),
            )
            .unwrap();
    }
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("row"),
            Style::default()
                .flex_direction(FlexDirection::Row)
                .width(Length::Px(200.0))
                .height(Length::Auto)
                .column_gap_percent(0.05),
        )
        .rule(
            StyleSelector::class("item"),
            Style::default().size(50.0, 20.0),
        );
    let mut engine = DocumentEngine::default();

    let output = engine.update(&mut document, &stylesheet);
    let item_0 = output.layout.find("item-0").unwrap();
    let item_1 = output.layout.find("item-1").unwrap();

    assert_eq!(item_0.rect.origin, Point::new(0.0, 0.0));
    assert_eq!(item_1.rect.origin, Point::new(60.0, 0.0));
}

#[test]
fn document_engine_resolves_percent_row_gap_against_flex_container_height() {
    let mut document = Document::new(Size::new(320.0, 200.0));
    document
        .append_element("root", "column", ElementSpec::new(Element::Div))
        .unwrap();
    for index in 0..2 {
        document
            .append_element(
                "column",
                format!("item-{index}"),
                ElementSpec::new(Element::Div).class("item"),
            )
            .unwrap();
    }
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("column"),
            Style::default()
                .flex_direction(FlexDirection::Column)
                .width(Length::Px(200.0))
                .height(Length::Px(200.0))
                .row_gap_percent(0.05),
        )
        .rule(
            StyleSelector::class("item"),
            Style::default().size(50.0, 20.0),
        );
    let mut engine = DocumentEngine::default();

    let output = engine.update(&mut document, &stylesheet);
    let item_0 = output.layout.find("item-0").unwrap();
    let item_1 = output.layout.find("item-1").unwrap();

    assert_eq!(item_0.rect.origin, Point::new(0.0, 0.0));
    assert_eq!(item_1.rect.origin, Point::new(0.0, 30.0));
}

#[test]
fn document_engine_resolves_calc_width_with_fixed_gap() {
    let mut document = Document::new(Size::new(320.0, 200.0));
    document
        .append_element("root", "row", ElementSpec::new(Element::Div))
        .unwrap();
    for index in 0..2 {
        document
            .append_element(
                "row",
                format!("item-{index}"),
                ElementSpec::new(Element::Div).class("item"),
            )
            .unwrap();
    }
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("row"),
            Style::default()
                .flex_direction(FlexDirection::Row)
                .width(Length::Px(200.0))
                .height(Length::Auto)
                .column_gap(10.0),
        )
        .rule(
            StyleSelector::class("item"),
            Style::default()
                .width(Length::calc(0.5, -5.0))
                .height(Length::Px(20.0)),
        );
    let mut engine = DocumentEngine::default();

    let output = engine.update(&mut document, &stylesheet);
    let item_0 = output.layout.find("item-0").unwrap();
    let item_1 = output.layout.find("item-1").unwrap();

    assert_eq!(item_0.rect.size, Size::new(95.0, 20.0));
    assert_eq!(item_0.rect.origin, Point::new(0.0, 0.0));
    assert_eq!(item_1.rect.size, Size::new(95.0, 20.0));
    assert_eq!(item_1.rect.origin, Point::new(105.0, 0.0));
}

#[test]
fn document_engine_update_fill_width_uses_parent_content_width_after_box_model() {
    let mut document = Document::new(Size::new(320.0, 200.0));
    document
        .append_element("root", "panel", ElementSpec::new(Element::Div))
        .unwrap();
    document
        .append_element("panel", "row", ElementSpec::new(Element::Div))
        .unwrap();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("panel"),
            Style::default()
                .size(200.0, 120.0)
                .border_width(2.0)
                .padding(Insets::symmetric(12.0, 8.0)),
        )
        .rule(
            StyleSelector::id("row"),
            Style::default()
                .width_fill()
                .height(Length::Px(24.0))
                .margin(Insets::symmetric(3.0, 0.0)),
        );
    let mut engine = DocumentEngine::default();

    let output = engine.update(&mut document, &stylesheet);
    let row = output.layout.find("row").unwrap();

    assert_eq!(row.rect.origin, Point::new(17.0, 10.0));
    assert_eq!(row.rect.size, Size::new(172.0, 24.0));
}

#[test]
fn document_engine_update_fill_size_does_not_inflate_auto_parent() {
    let mut document = Document::new(Size::new(240.0, 160.0));
    document
        .append_element("root", "panel", ElementSpec::new(Element::Div))
        .unwrap();
    document
        .append_element("panel", "child", ElementSpec::new(Element::Div))
        .unwrap();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("panel"),
            Style::default().width(Length::Auto).height(Length::Auto),
        )
        .rule(
            StyleSelector::id("child"),
            Style::default()
                .width_fill()
                .height_fill()
                .min_size(24.0, 24.0),
        );
    let mut engine = DocumentEngine::default();

    let output = engine.update(&mut document, &stylesheet);
    let panel = output.layout.find("panel").unwrap();

    assert_eq!(panel.rect.size, Size::new(24.0, 24.0));
}

#[test]
fn document_engine_update_max_size_clamps_auto_explicit_and_fill_sizes() {
    let mut document = Document::new(Size::new(260.0, 180.0));
    document
        .append_element("root", "panel", ElementSpec::new(Element::Div))
        .unwrap();
    document
        .append_element("panel", "auto-child", ElementSpec::new(Element::Div))
        .unwrap();
    document
        .append_element(
            "auto-child",
            "wide-child",
            ElementSpec::new(Element::Div).class("wide"),
        )
        .unwrap();
    document
        .append_element("panel", "fixed-child", ElementSpec::new(Element::Div))
        .unwrap();
    document
        .append_element("panel", "fill-child", ElementSpec::new(Element::Div))
        .unwrap();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("panel"),
            Style::default()
                .size(200.0, 120.0)
                .padding(Insets::all(10.0)),
        )
        .rule(
            StyleSelector::id("auto-child"),
            Style::default()
                .width(Length::Auto)
                .height(Length::Auto)
                .max_size(40.0, 30.0),
        )
        .rule(
            StyleSelector::id("fixed-child"),
            Style::default().size(96.0, 70.0).max_size(42.0, 28.0),
        )
        .rule(
            StyleSelector::id("fill-child"),
            Style::default()
                .width_fill()
                .height_fill()
                .max_size(50.0, 34.0),
        )
        .rule(
            StyleSelector::class("wide"),
            Style::default().size(80.0, 20.0),
        );
    let mut engine = DocumentEngine::default();

    let output = engine.update(&mut document, &stylesheet);

    assert_eq!(
        output.layout.find("auto-child").unwrap().rect.size,
        Size::new(40.0, 20.0)
    );
    assert_eq!(
        output.layout.find("fixed-child").unwrap().rect.size,
        Size::new(42.0, 28.0)
    );
    assert_eq!(
        output.layout.find("fill-child").unwrap().rect.size,
        Size::new(50.0, 34.0)
    );
}

#[test]
fn document_engine_update_with_input_scrolls_overflow_container() {
    let mut document = Document::new(Size::new(800.0, 600.0));
    document
        .append_element("root", "scroll", ElementSpec::new(Element::Div))
        .unwrap();
    document
        .append_element("scroll", "content", ElementSpec::new(Element::Div))
        .unwrap();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("scroll"),
            Style::default()
                .size(100.0, 100.0)
                .overflow_y(Overflow::Scroll),
        )
        .rule(
            StyleSelector::id("content"),
            Style::default().size(100.0, 300.0),
        );
    let mut engine = DocumentEngine::default();

    let output = engine.update_with_input(
        &mut document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(50.0, 50.0),
                primary_delta: Point::ZERO,
                primary_down: false,
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: false,
                time_seconds: 0.0,
            }),
            scroll_delta: Point::new(0.0, -40.0),
        },
    );

    assert_eq!(engine.element_state("scroll").unwrap().scroll_y, 40.0);
    assert!(
        output
            .events
            .iter()
            .any(|event| event.target == ElementId::new("scroll")
                && event.kind == DocumentEventKind::Scrolled(ScrollAxis::Vertical))
    );
    assert!(output.metrics.input_changed_state);
}

#[test]
fn document_engine_applies_initial_scroll_position_on_first_layout() {
    let mut document = Document::new(Size::new(800.0, 600.0));
    document
        .append_element(
            "root",
            "scroll",
            ElementSpec::new(Element::Div)
                .initial_scroll_x(30.0)
                .initial_scroll_y(40.0),
        )
        .unwrap();
    document
        .append_element("scroll", "content", ElementSpec::new(Element::Div))
        .unwrap();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("scroll"),
            Style::default()
                .size(100.0, 100.0)
                .overflow_x(Overflow::Scroll)
                .overflow_y(Overflow::Scroll),
        )
        .rule(
            StyleSelector::id("content"),
            Style::default().size(300.0, 300.0),
        );
    let mut engine = DocumentEngine::default();

    let output = engine.update(&mut document, &stylesheet);

    let state = engine.element_state("scroll").unwrap();
    assert_eq!(state.scroll_x, 30.0);
    assert_eq!(state.scroll_y, 40.0);
    assert_eq!(
        output.layout.find("content").unwrap().rect,
        Rect::new(-30.0, -40.0, 300.0, 300.0)
    );
}

#[test]
fn document_engine_clamps_initial_scroll_position_to_scroll_range() {
    let mut document = Document::new(Size::new(800.0, 600.0));
    document
        .append_element(
            "root",
            "scroll",
            ElementSpec::new(Element::Div).initial_scroll(Point::new(500.0, 500.0)),
        )
        .unwrap();
    document
        .append_element("scroll", "content", ElementSpec::new(Element::Div))
        .unwrap();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("scroll"),
            Style::default()
                .size(100.0, 100.0)
                .overflow_x(Overflow::Scroll)
                .overflow_y(Overflow::Scroll),
        )
        .rule(
            StyleSelector::id("content"),
            Style::default().size(160.0, 180.0),
        );
    let mut engine = DocumentEngine::default();

    let output = engine.update(&mut document, &stylesheet);

    let state = engine.element_state("scroll").unwrap();
    assert_eq!(state.scroll_x, 60.0);
    assert_eq!(state.scroll_y, 80.0);
    assert_eq!(
        output.layout.find("content").unwrap().rect,
        Rect::new(-60.0, -80.0, 160.0, 180.0)
    );
}

#[test]
fn document_engine_scroll_element_to_repositions_existing_scroll_state() {
    let mut document = Document::new(Size::new(800.0, 600.0));
    document
        .append_element(
            "root",
            "scroll",
            ElementSpec::new(Element::Div).initial_scroll_y(40.0),
        )
        .unwrap();
    document
        .append_element("scroll", "content", ElementSpec::new(Element::Div))
        .unwrap();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("scroll"),
            Style::default()
                .size(100.0, 100.0)
                .overflow_y(Overflow::Scroll),
        )
        .rule(
            StyleSelector::id("content"),
            Style::default().size(100.0, 300.0),
        );
    let mut engine = DocumentEngine::default();
    engine.update(&mut document, &stylesheet);

    assert!(engine.scroll_element_to("scroll", Point::new(0.0, 500.0)));
    let output = engine.update(&mut document, &stylesheet);

    assert_eq!(engine.element_state("scroll").unwrap().scroll_y, 200.0);
    assert_eq!(
        output.layout.find("content").unwrap().rect,
        Rect::new(0.0, -200.0, 100.0, 300.0)
    );
}

#[test]
fn document_engine_reports_current_scroll_position() {
    let mut document = Document::new(Size::new(800.0, 600.0));
    document
        .append_element("root", "scroll", ElementSpec::new(Element::Div))
        .unwrap();
    document
        .append_element("scroll", "content", ElementSpec::new(Element::Div))
        .unwrap();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("scroll"),
            Style::default()
                .size(100.0, 100.0)
                .overflow_x(Overflow::Scroll)
                .overflow_y(Overflow::Scroll),
        )
        .rule(
            StyleSelector::id("content"),
            Style::default().size(300.0, 300.0),
        );
    let mut engine = DocumentEngine::default();
    engine.update(&mut document, &stylesheet);

    assert!(engine.scroll_element_to("scroll", Point::new(24.0, 48.0)));

    assert_eq!(
        engine.scroll_position("scroll"),
        Some(Point::new(24.0, 48.0))
    );
    assert_eq!(engine.scroll_position("missing"), None);
}

#[test]
fn document_engine_initial_scroll_does_not_overwrite_retained_scroll_state() {
    let mut document = Document::new(Size::new(800.0, 600.0));
    document
        .append_element(
            "root",
            "scroll",
            ElementSpec::new(Element::Div).initial_scroll_y(80.0),
        )
        .unwrap();
    document
        .append_element("scroll", "content", ElementSpec::new(Element::Div))
        .unwrap();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("scroll"),
            Style::default()
                .size(100.0, 100.0)
                .overflow_y(Overflow::Scroll),
        )
        .rule(
            StyleSelector::id("content"),
            Style::default().size(100.0, 300.0),
        );
    let mut engine = DocumentEngine::default();
    engine.update(&mut document, &stylesheet);

    assert!(engine.scroll_element_to("scroll", Point::new(0.0, 20.0)));
    let output = engine.update(&mut document, &stylesheet);

    assert_eq!(engine.element_state("scroll").unwrap().scroll_y, 20.0);
    assert_eq!(
        output.layout.find("content").unwrap().rect,
        Rect::new(0.0, -20.0, 100.0, 300.0)
    );
}

#[test]
fn document_engine_update_scroll_only_final_pass_skips_style_resolution() {
    let mut document = Document::new(Size::new(800.0, 600.0));
    document
        .append_element("root", "scroll", ElementSpec::new(Element::Div))
        .unwrap();
    document
        .append_element("scroll", "content", ElementSpec::new(Element::Div))
        .unwrap();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("scroll"),
            Style::default()
                .size(100.0, 100.0)
                .overflow_y(Overflow::Scroll),
        )
        .rule(
            StyleSelector::id("content"),
            Style::default().size(100.0, 300.0),
        );
    let mut engine = DocumentEngine::default();
    let hover_input = DocumentInput {
        pointer: Some(PointerInput {
            position: Point::new(50.0, 50.0),
            primary_delta: Point::ZERO,
            primary_down: false,
            primary_pressed: false,
            primary_clicked: false,
            primary_click_count: 0,
            secondary_clicked: false,
            time_seconds: 0.0,
        }),
        scroll_delta: Point::ZERO,
    };
    engine.update_with_input(&mut document, &stylesheet, hover_input);

    let output = engine.update_with_input(
        &mut document,
        &stylesheet,
        DocumentInput {
            scroll_delta: Point::new(0.0, -40.0),
            ..hover_input
        },
    );

    assert_eq!(output.metrics.style_nodes_visited, 0);
    assert!(!output.metrics.reused_input_layout);
    assert_eq!(
        output.layout.find("content").unwrap().rect.origin,
        Point::new(0.0, -40.0)
    );
}

#[test]
fn document_engine_update_with_input_offsets_scrolled_child_rects() {
    let mut document = Document::new(Size::new(800.0, 600.0));
    document
        .append_element("root", "scroll", ElementSpec::new(Element::Div))
        .unwrap();
    document
        .append_element("scroll", "content", ElementSpec::new(Element::Div))
        .unwrap();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("scroll"),
            Style::default()
                .size(100.0, 100.0)
                .overflow_y(Overflow::Scroll),
        )
        .rule(
            StyleSelector::id("content"),
            Style::default().size(100.0, 300.0),
        );
    let mut engine = DocumentEngine::default();

    let output = engine.update_with_input(
        &mut document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(50.0, 50.0),
                primary_delta: Point::ZERO,
                primary_down: false,
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: false,
                time_seconds: 0.0,
            }),
            scroll_delta: Point::new(0.0, -40.0),
        },
    );

    assert_eq!(
        output.layout.find("content").unwrap().rect,
        Rect::new(0.0, -40.0, 100.0, 300.0)
    );
}

#[test]
fn document_engine_update_with_input_hit_tests_scrolled_child_position() {
    let mut document = Document::new(Size::new(800.0, 600.0));
    document
        .append_element("root", "scroll", ElementSpec::new(Element::Div))
        .unwrap();
    document
        .append_element("scroll", "content", ElementSpec::new(Element::Div))
        .unwrap();
    document
        .append_element("content", "spacer", ElementSpec::new(Element::Div))
        .unwrap();
    document
        .append_element(
            "content",
            "target",
            ElementSpec::new(Element::Button).interactive(),
        )
        .unwrap();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("scroll"),
            Style::default()
                .size(100.0, 100.0)
                .overflow_y(Overflow::Scroll),
        )
        .rule(
            StyleSelector::id("content"),
            Style::default().size(100.0, 300.0),
        )
        .rule(
            StyleSelector::id("spacer"),
            Style::default().size(100.0, 100.0),
        )
        .rule(
            StyleSelector::id("target"),
            Style::default().size(100.0, 30.0),
        );
    let mut engine = DocumentEngine::default();
    engine.update_with_input(
        &mut document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(50.0, 50.0),
                primary_delta: Point::ZERO,
                primary_down: false,
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: false,
                time_seconds: 0.0,
            }),
            scroll_delta: Point::new(0.0, -40.0),
        },
    );

    let output = engine.update_with_input(
        &mut document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(50.0, 70.0),
                primary_delta: Point::ZERO,
                primary_down: true,
                primary_pressed: true,
                primary_clicked: true,
                primary_click_count: 1,
                secondary_clicked: false,
                time_seconds: 0.1,
            }),
            scroll_delta: Point::ZERO,
        },
    );

    assert_eq!(output.hit_id, Some(ElementId::new("target")));
    assert!(engine.element_state("target").unwrap().pressed);
}

#[test]
fn document_engine_update_with_input_rerenders_state_styles() {
    let mut document = Document::new(Size::new(800.0, 600.0));
    document
        .append_element(
            "root",
            "button",
            ElementSpec::new(Element::Button).interactive(),
        )
        .unwrap();
    let stylesheet = StyleSheet::new()
        .rule(
            StyleSelector::id("button"),
            Style::default().size(120.0, 40.0),
        )
        .rule(
            StyleSelector::id_state("button", ElementStateSelector::Hovered),
            Style::default().size(160.0, 40.0),
        );
    let mut engine = DocumentEngine::default();

    let output = engine.update_with_input(
        &mut document,
        &stylesheet,
        DocumentInput {
            pointer: Some(PointerInput {
                position: Point::new(10.0, 10.0),
                primary_delta: Point::ZERO,
                primary_down: false,
                primary_pressed: false,
                primary_clicked: false,
                primary_click_count: 0,
                secondary_clicked: false,
                time_seconds: 0.0,
            }),
            scroll_delta: Point::ZERO,
        },
    );

    assert_eq!(
        output.layout.find("button").unwrap().rect,
        Rect::new(0.0, 0.0, 160.0, 40.0)
    );
    assert!(!output.metrics.reused_input_layout);
}
