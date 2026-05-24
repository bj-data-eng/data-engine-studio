use crate::{
    AlignContent, AlignItems, Color, Direction, Display, Easing, Element, ElementStateSelector,
    FlexDirection, FlexWrap, Insets, JustifyContent, Length, Overflow, OverflowWrap, Position,
    Style, StyleCondition, StyleSelector, StyleSheet, TextAlign, TextOverflow, TextTransform,
    TextWrapMode, Transition, ViewportQuery, WhiteSpace,
};
use std::error::Error;
use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssParseError {
    message: String,
}

impl CssParseError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for CssParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for CssParseError {}

pub(crate) fn parse_stylesheet(input: &str) -> Result<StyleSheet, CssParseError> {
    let input = strip_comments(input)?;
    let mut sheet = StyleSheet::new();
    parse_rules_into(&mut sheet, input.as_str(), None)?;
    Ok(sheet)
}

fn strip_comments(input: &str) -> Result<String, CssParseError> {
    let mut output = String::with_capacity(input.len());
    let mut rest = input;
    while let Some(start) = rest.find("/*") {
        output.push_str(&rest[..start]);
        let after_start = &rest[start + 2..];
        let Some(end) = after_start.find("*/") else {
            return Err(CssParseError::new("CSS comment is missing closing `*/`"));
        };
        rest = &after_start[end + 2..];
    }
    output.push_str(rest);
    Ok(output)
}

fn parse_rules_into(
    sheet: &mut StyleSheet,
    input: &str,
    condition: Option<StyleCondition>,
) -> Result<(), CssParseError> {
    let mut rest = input.trim_start();
    while !rest.is_empty() {
        let Some(start) = rest.find('{') else {
            return Err(CssParseError::new(
                "CSS contains trailing text outside a rule",
            ));
        };
        let prelude = rest[..start].trim();
        if prelude.is_empty() {
            return Err(CssParseError::new("CSS rule is missing a selector"));
        }
        let end = matching_block_end(rest, start)?;
        let block = &rest[start + 1..end];
        rest = rest[end + 1..].trim_start();

        if let Some(query) = prelude.strip_prefix("@media") {
            let media_condition = parse_media_query(query.trim())?;
            let condition = merge_conditions(condition, media_condition)?;
            parse_rules_into(sheet, block, Some(condition))?;
            continue;
        }

        if prelude.starts_with('@') {
            return Err(CssParseError::new(format!(
                "unsupported CSS at-rule `{prelude}`"
            )));
        }

        let style = parse_declarations(block)?;
        for selector in prelude.split(',') {
            let selector = parse_selector(selector.trim())?;
            if let Some(condition) = condition {
                sheet.push_conditional_rule(condition, selector, style.clone());
            } else {
                sheet.push_rule(selector, style.clone());
            }
        }
    }
    Ok(())
}

fn matching_block_end(input: &str, open: usize) -> Result<usize, CssParseError> {
    let mut depth = 0usize;
    for (index, ch) in input[open..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Ok(open + index);
                }
            }
            _ => {}
        }
    }
    Err(CssParseError::new("CSS block is missing closing `}`"))
}

fn parse_media_query(input: &str) -> Result<StyleCondition, CssParseError> {
    let mut query = ViewportQuery::new();
    let mut saw_feature = false;
    let mut rest = input.trim();

    if let Some(after_screen) = rest.strip_prefix("screen") {
        rest = after_screen.trim_start();
        if let Some(after_and) = rest.strip_prefix("and") {
            rest = after_and.trim_start();
        }
    }

    while !rest.is_empty() {
        if !rest.starts_with('(') {
            return Err(CssParseError::new(format!(
                "unsupported @media query `{input}`"
            )));
        }
        let Some(end) = rest.find(')') else {
            return Err(CssParseError::new("@media feature is missing `)`"));
        };
        let feature = rest[1..end].trim();
        apply_media_feature(&mut query, feature)?;
        saw_feature = true;

        rest = rest[end + 1..].trim_start();
        if let Some(after_and) = rest.strip_prefix("and") {
            rest = after_and.trim_start();
        } else if !rest.is_empty() {
            return Err(CssParseError::new(format!(
                "unsupported @media query `{input}`"
            )));
        }
    }

    if !saw_feature {
        return Err(CssParseError::new("@media query is missing a feature"));
    }
    Ok(StyleCondition::viewport(query))
}

fn apply_media_feature(query: &mut ViewportQuery, input: &str) -> Result<(), CssParseError> {
    let Some((name, value)) = input.split_once(':') else {
        return Err(CssParseError::new(format!(
            "@media feature `{input}` is missing `:`"
        )));
    };
    let value = parse_px(value.trim())?;
    match name.trim() {
        "min-width" => query.min_width = Some(value),
        "max-width" => query.max_width = Some(value),
        "min-height" => query.min_height = Some(value),
        "max-height" => query.max_height = Some(value),
        _ => {
            return Err(CssParseError::new(format!(
                "unsupported @media feature `{}`",
                name.trim()
            )));
        }
    }
    Ok(())
}

fn merge_conditions(
    existing: Option<StyleCondition>,
    next: StyleCondition,
) -> Result<StyleCondition, CssParseError> {
    match (existing, next) {
        (None, condition) => Ok(condition),
        (Some(StyleCondition::Viewport(mut existing)), StyleCondition::Viewport(next)) => {
            existing.min_width = max_optional(existing.min_width, next.min_width);
            existing.max_width = min_optional(existing.max_width, next.max_width);
            existing.min_height = max_optional(existing.min_height, next.min_height);
            existing.max_height = min_optional(existing.max_height, next.max_height);
            Ok(StyleCondition::viewport(existing))
        }
        _ => Err(CssParseError::new(
            "CSS @media rules can only be nested with viewport media rules",
        )),
    }
}

fn max_optional(left: Option<f32>, right: Option<f32>) -> Option<f32> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left.max(right)),
        (Some(value), None) | (None, Some(value)) => Some(value),
        (None, None) => None,
    }
}

fn min_optional(left: Option<f32>, right: Option<f32>) -> Option<f32> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left.min(right)),
        (Some(value), None) | (None, Some(value)) => Some(value),
        (None, None) => None,
    }
}

fn parse_selector(input: &str) -> Result<StyleSelector, CssParseError> {
    if input.is_empty() {
        return Err(CssParseError::new("CSS selector is empty"));
    }
    let mut selector = StyleSelector::compound();
    let mut cursor = 0;
    let chars: Vec<char> = input.chars().collect();

    if chars[cursor] == '*' {
        cursor += 1;
    } else if chars[cursor].is_ascii_alphabetic() {
        let start = cursor;
        while cursor < chars.len() && is_ident_char(chars[cursor]) {
            cursor += 1;
        }
        selector = selector.element(parse_element(&input[start..cursor])?);
    }

    while cursor < chars.len() {
        match chars[cursor] {
            '.' => {
                cursor += 1;
                let start = cursor;
                while cursor < chars.len() && is_ident_char(chars[cursor]) {
                    cursor += 1;
                }
                if start == cursor {
                    return Err(CssParseError::new("CSS class selector is missing a name"));
                }
                selector = selector.class(&input[start..cursor]);
            }
            '#' => {
                cursor += 1;
                let start = cursor;
                while cursor < chars.len() && is_ident_char(chars[cursor]) {
                    cursor += 1;
                }
                if start == cursor {
                    return Err(CssParseError::new("CSS id selector is missing a name"));
                }
                selector = selector.id(&input[start..cursor]);
            }
            ':' => {
                cursor += 1;
                let start = cursor;
                while cursor < chars.len() && is_ident_char(chars[cursor]) {
                    cursor += 1;
                }
                let name = &input[start..cursor];
                if name == "nth-child" {
                    if chars.get(cursor) != Some(&'(') {
                        return Err(CssParseError::new("nth-child selector is missing `(`"));
                    }
                    cursor += 1;
                    let arg_start = cursor;
                    while cursor < chars.len() && chars[cursor] != ')' {
                        cursor += 1;
                    }
                    if chars.get(cursor) != Some(&')') {
                        return Err(CssParseError::new("nth-child selector is missing `)`"));
                    }
                    selector = parse_nth_child(selector, input[arg_start..cursor].trim())?;
                    cursor += 1;
                } else {
                    selector = match parse_state_selector(name)? {
                        ParsedPseudo::State(state) => selector.state(state),
                        ParsedPseudo::FirstChild => selector.first_child(),
                        ParsedPseudo::LastChild => selector.last_child(),
                    };
                }
            }
            ch if ch.is_whitespace() => {
                return Err(CssParseError::new(
                    "descendant selectors are not supported by this CSS slice yet",
                ));
            }
            _ => {
                return Err(CssParseError::new(format!(
                    "unsupported selector token `{}`",
                    chars[cursor]
                )));
            }
        }
    }

    Ok(selector.selector())
}

enum ParsedPseudo {
    State(ElementStateSelector),
    FirstChild,
    LastChild,
}

fn parse_state_selector(name: &str) -> Result<ParsedPseudo, CssParseError> {
    match name {
        "hover" | "hovered" => Ok(ParsedPseudo::State(ElementStateSelector::Hovered)),
        "active" | "pressed" => Ok(ParsedPseudo::State(ElementStateSelector::Pressed)),
        "dragged" => Ok(ParsedPseudo::State(ElementStateSelector::Dragged)),
        "focused" | "focus" => Ok(ParsedPseudo::State(ElementStateSelector::Focused)),
        "selected" => Ok(ParsedPseudo::State(ElementStateSelector::Selected)),
        "disabled" => Ok(ParsedPseudo::State(ElementStateSelector::Disabled)),
        "first-child" => Ok(ParsedPseudo::FirstChild),
        "last-child" => Ok(ParsedPseudo::LastChild),
        _ => Err(CssParseError::new(format!(
            "unsupported pseudo selector `:{name}`"
        ))),
    }
}

fn parse_nth_child(
    selector: crate::CompoundSelector,
    input: &str,
) -> Result<crate::CompoundSelector, CssParseError> {
    match input {
        "odd" => Ok(selector.nth_child_odd()),
        "even" => Ok(selector.nth_child_even()),
        value => {
            let nth = value.parse::<usize>().map_err(|_| {
                CssParseError::new("nth-child currently accepts odd, even, or a positive integer")
            })?;
            Ok(selector.nth_child(nth))
        }
    }
}

fn parse_declarations(input: &str) -> Result<Style, CssParseError> {
    let mut style = Style::default();
    for declaration in input.split(';') {
        let declaration = declaration.trim();
        if declaration.is_empty() {
            continue;
        }
        let Some((name, value)) = declaration.split_once(':') else {
            return Err(CssParseError::new(format!(
                "CSS declaration `{declaration}` is missing `:`"
            )));
        };
        apply_declaration(&mut style, name.trim(), value.trim())?;
    }
    Ok(style)
}

fn apply_declaration(style: &mut Style, name: &str, value: &str) -> Result<(), CssParseError> {
    match name {
        "display" => style.display = Some(parse_display(value)?),
        "direction" => style.direction = Some(parse_direction(value)?),
        "flex-direction" => style.flex_direction = Some(parse_flex_direction(value)?),
        "flex-wrap" => style.flex_wrap = Some(parse_flex_wrap(value)?),
        "flex-basis" => style.flex_basis = Some(parse_length(value)?),
        "flex-grow" => style.flex_grow = Some(parse_f32(value)?.max(0.0)),
        "flex-shrink" => style.flex_shrink = Some(parse_f32(value)?.max(0.0)),
        "align-items" => style.align_items = Some(parse_align_items(value)?),
        "align-self" => style.align_self = Some(parse_align_items(value)?),
        "justify-content" => style.justify_content = Some(parse_justify_content(value)?),
        "align-content" => style.align_content = Some(parse_align_content(value)?),
        "gap" => style.gap = Some(parse_length(value)?),
        "row-gap" => style.row_gap = Some(parse_length(value)?),
        "column-gap" => style.column_gap = Some(parse_length(value)?),
        "margin" => style.margin = Some(parse_insets(value)?),
        "padding" => style.padding = Some(parse_insets(value)?),
        "width" => style.width = Some(parse_length(value)?),
        "height" => style.height = Some(parse_length(value)?),
        "min-width" => {
            style
                .min_size
                .get_or_insert(crate::Size::new(0.0, 0.0))
                .width = parse_px(value)?;
        }
        "min-height" => {
            style
                .min_size
                .get_or_insert(crate::Size::new(0.0, 0.0))
                .height = parse_px(value)?;
        }
        "max-width" => {
            style
                .max_size
                .get_or_insert(crate::Size::new(f32::INFINITY, f32::INFINITY))
                .width = parse_px(value)?;
        }
        "max-height" => {
            style
                .max_size
                .get_or_insert(crate::Size::new(f32::INFINITY, f32::INFINITY))
                .height = parse_px(value)?;
        }
        "background" | "background-color" => style.background = Some(parse_color(value)?),
        "border-color" => style.border = Some(parse_color(value)?),
        "border" => parse_border(style, value)?,
        "border-width" => style.border_width = crate::EdgeStyle::all(parse_px(value)?),
        "border-style" => style.border_style = Some(parse_border_style(value)?),
        "border-top-width" => style.border_width.top = Some(parse_px(value)?),
        "border-right-width" => style.border_width.right = Some(parse_px(value)?),
        "border-bottom-width" => style.border_width.bottom = Some(parse_px(value)?),
        "border-left-width" => style.border_width.left = Some(parse_px(value)?),
        "border-radius" | "radius" => style.radius = crate::CornerStyle::all(parse_px(value)?),
        "border-top-left-radius" => style.radius.top_left = Some(parse_px(value)?),
        "border-top-right-radius" => style.radius.top_right = Some(parse_px(value)?),
        "border-bottom-right-radius" => style.radius.bottom_right = Some(parse_px(value)?),
        "border-bottom-left-radius" => style.radius.bottom_left = Some(parse_px(value)?),
        "color" | "text-color" => style.text_color = Some(parse_color(value)?),
        "text-selection-background" => style.text_selection_background = Some(parse_color(value)?),
        "text-selection-color" => style.text_selection_color = Some(parse_color(value)?),
        "font-size" => style.font_size = Some(parse_px(value)?),
        "line-height" => style.line_height = Some(parse_px(value)?),
        "white-space" => *style = style.clone().white_space(parse_white_space(value)?),
        "text-wrap" | "text-wrap-mode" => {
            *style = style.clone().text_wrap_mode(parse_text_wrap_mode(value)?);
        }
        "overflow-wrap" => *style = style.clone().overflow_wrap(parse_overflow_wrap(value)?),
        "text-align" => *style = style.clone().text_align(parse_text_align(value)?),
        "text-overflow" => *style = style.clone().text_overflow(parse_text_overflow(value)?),
        "text-transform" => *style = style.clone().text_transform(parse_text_transform(value)?),
        "max-lines" => *style = style.clone().max_lines(parse_usize(value)?),
        "overflow" => {
            let overflow = parse_overflow(value)?;
            style.overflow_x = Some(overflow);
            style.overflow_y = Some(overflow);
        }
        "overflow-x" => style.overflow_x = Some(parse_overflow(value)?),
        "overflow-y" => style.overflow_y = Some(parse_overflow(value)?),
        "scrollbar-width" => style.scrollbar_width = Some(parse_px(value)?.max(0.0)),
        "scrollbar-visible" => style.scrollbar_visible = Some(parse_bool(value)?),
        "position" => style.position = Some(parse_position(value)?),
        "top" => style.inset.top = Some(parse_length(value)?),
        "right" => style.inset.right = Some(parse_length(value)?),
        "bottom" => style.inset.bottom = Some(parse_length(value)?),
        "left" => style.inset.left = Some(parse_length(value)?),
        "z-index" => {
            style.z_index = Some(
                value
                    .parse::<i32>()
                    .map_err(|_| CssParseError::new("z-index must be an integer"))?,
            );
        }
        "transition" => style.transition = parse_transition(value)?,
        _ => {
            return Err(CssParseError::new(format!(
                "unsupported CSS property `{name}`"
            )));
        }
    }
    Ok(())
}

fn parse_element(input: &str) -> Result<Element, CssParseError> {
    match input {
        "root" => Ok(Element::Root),
        "div" => Ok(Element::Div),
        "span" => Ok(Element::Span),
        "main" => Ok(Element::Main),
        "section" => Ok(Element::Section),
        "article" => Ok(Element::Article),
        "header" => Ok(Element::Header),
        "footer" => Ok(Element::Footer),
        "nav" => Ok(Element::Nav),
        "aside" => Ok(Element::Aside),
        "p" => Ok(Element::P),
        "h1" => Ok(Element::H1),
        "h2" => Ok(Element::H2),
        "h3" => Ok(Element::H3),
        "h4" => Ok(Element::H4),
        "h5" => Ok(Element::H5),
        "h6" => Ok(Element::H6),
        "text" => Ok(Element::Text),
        "button" => Ok(Element::Button),
        "input" => Ok(Element::Input),
        "checkbox" => Ok(Element::Checkbox),
        "radio" => Ok(Element::Radio),
        "select" => Ok(Element::Select),
        "option" => Ok(Element::Option),
        "textarea" => Ok(Element::Textarea),
        "label" => Ok(Element::Label),
        "canvas" => Ok(Element::Canvas),
        "icon" => Ok(Element::Icon),
        "table" => Ok(Element::Table),
        "thead" => Ok(Element::Thead),
        "tbody" => Ok(Element::Tbody),
        "tr" => Ok(Element::Tr),
        "th" => Ok(Element::Th),
        "td" => Ok(Element::Td),
        _ => Err(CssParseError::new(format!("unsupported element `{input}`"))),
    }
}

fn parse_display(input: &str) -> Result<Display, CssParseError> {
    match input {
        "block" | "flex" => Ok(Display::Flex),
        "grid" => Ok(Display::Grid),
        "none" => Ok(Display::None),
        _ => Err(CssParseError::new(format!("unsupported display `{input}`"))),
    }
}

fn parse_direction(input: &str) -> Result<Direction, CssParseError> {
    match input {
        "ltr" => Ok(Direction::Ltr),
        "rtl" => Ok(Direction::Rtl),
        _ => Err(CssParseError::new(format!(
            "unsupported direction `{input}`"
        ))),
    }
}

fn parse_flex_direction(input: &str) -> Result<FlexDirection, CssParseError> {
    match input {
        "row" => Ok(FlexDirection::Row),
        "column" => Ok(FlexDirection::Column),
        "row-reverse" => Ok(FlexDirection::RowReverse),
        "column-reverse" => Ok(FlexDirection::ColumnReverse),
        _ => Err(CssParseError::new(format!(
            "unsupported flex-direction `{input}`"
        ))),
    }
}

fn parse_flex_wrap(input: &str) -> Result<FlexWrap, CssParseError> {
    match input {
        "nowrap" | "no-wrap" => Ok(FlexWrap::NoWrap),
        "wrap" => Ok(FlexWrap::Wrap),
        "wrap-reverse" => Ok(FlexWrap::WrapReverse),
        _ => Err(CssParseError::new(format!(
            "unsupported flex-wrap `{input}`"
        ))),
    }
}

fn parse_align_items(input: &str) -> Result<AlignItems, CssParseError> {
    match input {
        "start" => Ok(AlignItems::Start),
        "flex-start" => Ok(AlignItems::FlexStart),
        "center" => Ok(AlignItems::Center),
        "flex-end" => Ok(AlignItems::FlexEnd),
        "end" => Ok(AlignItems::End),
        "baseline" => Ok(AlignItems::Baseline),
        "stretch" => Ok(AlignItems::Stretch),
        _ => Err(CssParseError::new(format!(
            "unsupported alignment `{input}`"
        ))),
    }
}

fn parse_justify_content(input: &str) -> Result<JustifyContent, CssParseError> {
    match input {
        "start" => Ok(JustifyContent::Start),
        "flex-start" => Ok(JustifyContent::FlexStart),
        "center" => Ok(JustifyContent::Center),
        "flex-end" => Ok(JustifyContent::FlexEnd),
        "end" => Ok(JustifyContent::End),
        "stretch" => Ok(JustifyContent::Stretch),
        "space-between" => Ok(JustifyContent::SpaceBetween),
        "space-evenly" => Ok(JustifyContent::SpaceEvenly),
        "space-around" => Ok(JustifyContent::SpaceAround),
        _ => Err(CssParseError::new(format!(
            "unsupported justify-content `{input}`"
        ))),
    }
}

fn parse_align_content(input: &str) -> Result<AlignContent, CssParseError> {
    match input {
        "start" => Ok(AlignContent::Start),
        "flex-start" => Ok(AlignContent::FlexStart),
        "center" => Ok(AlignContent::Center),
        "flex-end" => Ok(AlignContent::FlexEnd),
        "end" => Ok(AlignContent::End),
        "stretch" => Ok(AlignContent::Stretch),
        "space-between" => Ok(AlignContent::SpaceBetween),
        "space-evenly" => Ok(AlignContent::SpaceEvenly),
        "space-around" => Ok(AlignContent::SpaceAround),
        _ => Err(CssParseError::new(format!(
            "unsupported align-content `{input}`"
        ))),
    }
}

fn parse_white_space(input: &str) -> Result<WhiteSpace, CssParseError> {
    match input {
        "normal" => Ok(WhiteSpace::Normal),
        "nowrap" => Ok(WhiteSpace::Nowrap),
        "pre" => Ok(WhiteSpace::Pre),
        "pre-wrap" => Ok(WhiteSpace::PreWrap),
        "pre-line" => Ok(WhiteSpace::PreLine),
        "break-spaces" => Ok(WhiteSpace::BreakSpaces),
        _ => Err(CssParseError::new(format!(
            "unsupported white-space `{input}`"
        ))),
    }
}

fn parse_text_wrap_mode(input: &str) -> Result<TextWrapMode, CssParseError> {
    match input {
        "wrap" => Ok(TextWrapMode::Wrap),
        "nowrap" | "no-wrap" | "extend" => Ok(TextWrapMode::NoWrap),
        _ => Err(CssParseError::new(format!(
            "unsupported text-wrap `{input}`"
        ))),
    }
}

fn parse_overflow_wrap(input: &str) -> Result<OverflowWrap, CssParseError> {
    match input {
        "normal" => Ok(OverflowWrap::Normal),
        "anywhere" => Ok(OverflowWrap::Anywhere),
        "break-word" => Ok(OverflowWrap::BreakWord),
        _ => Err(CssParseError::new(format!(
            "unsupported overflow-wrap `{input}`"
        ))),
    }
}

fn parse_text_align(input: &str) -> Result<TextAlign, CssParseError> {
    match input {
        "start" => Ok(TextAlign::Start),
        "center" => Ok(TextAlign::Center),
        "end" => Ok(TextAlign::End),
        "justify" => Ok(TextAlign::Justify),
        _ => Err(CssParseError::new(format!(
            "unsupported text-align `{input}`"
        ))),
    }
}

fn parse_text_overflow(input: &str) -> Result<TextOverflow, CssParseError> {
    match input {
        "clip" => Ok(TextOverflow::Clip),
        "ellipsis" => Ok(TextOverflow::Ellipsis),
        _ => Err(CssParseError::new(format!(
            "unsupported text-overflow `{input}`"
        ))),
    }
}

fn parse_text_transform(input: &str) -> Result<TextTransform, CssParseError> {
    match input {
        "none" => Ok(TextTransform::None),
        "uppercase" => Ok(TextTransform::Uppercase),
        "lowercase" => Ok(TextTransform::Lowercase),
        "capitalize" => Ok(TextTransform::Capitalize),
        _ => Err(CssParseError::new(format!(
            "unsupported text-transform `{input}`"
        ))),
    }
}

fn parse_overflow(input: &str) -> Result<Overflow, CssParseError> {
    match input {
        "visible" => Ok(Overflow::Visible),
        "clip" => Ok(Overflow::Clip),
        "hidden" => Ok(Overflow::Hidden),
        "auto" => Ok(Overflow::Auto),
        "scroll" => Ok(Overflow::Scroll),
        _ => Err(CssParseError::new(format!(
            "unsupported overflow `{input}`"
        ))),
    }
}

fn parse_position(input: &str) -> Result<Position, CssParseError> {
    match input {
        "flow" | "static" | "relative" => Ok(Position::Flow),
        "absolute" | "absolute-parent" => Ok(Position::AbsoluteParent),
        "fixed" | "absolute-viewport" => Ok(Position::AbsoluteViewport),
        _ => Err(CssParseError::new(format!(
            "unsupported position `{input}`"
        ))),
    }
}

fn parse_border(style: &mut Style, input: &str) -> Result<(), CssParseError> {
    for part in input.split_whitespace() {
        if let Ok(width) = parse_px(part) {
            style.border_width = crate::EdgeStyle::all(width);
        } else if matches!(part, "solid" | "dashed" | "dotted") {
            style.border_style = Some(parse_border_style(part)?);
        } else {
            style.border = Some(parse_color(part)?);
        }
    }
    Ok(())
}

fn parse_border_style(input: &str) -> Result<crate::BorderStyle, CssParseError> {
    match input {
        "solid" => Ok(crate::BorderStyle::Solid),
        "dashed" => Ok(crate::BorderStyle::Dashed),
        "dotted" => Ok(crate::BorderStyle::Dotted),
        _ => Err(CssParseError::new(format!(
            "unsupported border-style `{input}`"
        ))),
    }
}

fn parse_transition(input: &str) -> Result<Option<Transition>, CssParseError> {
    if input == "none" {
        return Ok(None);
    }

    let mut step = None;
    let mut easing = None;
    for part in input.split_whitespace() {
        match part {
            "all" => {}
            "linear" => easing = Some(Easing::Linear),
            "ease-out" | "ease-out-cubic" => easing = Some(Easing::EaseOutCubic),
            value => step = Some(parse_duration_step(value)?),
        }
    }

    Ok(Some(Transition {
        step: step.ok_or_else(|| CssParseError::new("transition is missing a duration"))?,
        easing: easing.unwrap_or(Easing::Linear),
    }))
}

fn parse_duration_step(input: &str) -> Result<f32, CssParseError> {
    if let Some(value) = input.strip_suffix("ms") {
        return Ok(parse_f32(value)? / 1000.0);
    }
    if let Some(value) = input.strip_suffix('s') {
        return parse_f32(value);
    }
    parse_f32(input)
}

fn parse_insets(input: &str) -> Result<Insets, CssParseError> {
    let values = input
        .split_whitespace()
        .map(parse_px)
        .collect::<Result<Vec<_>, _>>()?;
    match values.as_slice() {
        [all] => Ok(Insets::all(*all)),
        [vertical, horizontal] => Ok(Insets::symmetric(*horizontal, *vertical)),
        [top, horizontal, bottom] => Ok(Insets {
            top: *top,
            right: *horizontal,
            bottom: *bottom,
            left: *horizontal,
        }),
        [top, right, bottom, left] => Ok(Insets {
            top: *top,
            right: *right,
            bottom: *bottom,
            left: *left,
        }),
        _ => Err(CssParseError::new("expected 1 to 4 inset values")),
    }
}

fn parse_length(input: &str) -> Result<Length, CssParseError> {
    match input {
        "auto" => Ok(Length::Auto),
        "fill" | "100%" => Ok(Length::Fill),
        value if value.ends_with('%') => {
            let percent = parse_f32(value.trim_end_matches('%'))? / 100.0;
            Ok(Length::Percent(percent))
        }
        value => Ok(Length::Px(parse_px(value)?)),
    }
}

fn parse_px(input: &str) -> Result<f32, CssParseError> {
    let value = input.trim_end_matches("px");
    parse_f32(value)
}

fn parse_f32(input: &str) -> Result<f32, CssParseError> {
    input
        .parse::<f32>()
        .map_err(|_| CssParseError::new(format!("expected number, got `{input}`")))
}

fn parse_usize(input: &str) -> Result<usize, CssParseError> {
    input
        .parse::<usize>()
        .map_err(|_| CssParseError::new(format!("expected positive integer, got `{input}`")))
}

fn parse_bool(input: &str) -> Result<bool, CssParseError> {
    match input {
        "true" | "visible" => Ok(true),
        "false" | "hidden" => Ok(false),
        _ => Err(CssParseError::new(format!(
            "expected boolean, got `{input}`"
        ))),
    }
}

fn parse_color(input: &str) -> Result<Color, CssParseError> {
    match input {
        "transparent" => return Ok(Color::rgba(0, 0, 0, 0)),
        "black" => return Ok(Color::rgb(0, 0, 0)),
        "white" => return Ok(Color::rgb(255, 255, 255)),
        _ => {}
    }
    let hex = input
        .strip_prefix('#')
        .ok_or_else(|| CssParseError::new(format!("unsupported color `{input}`")))?;
    let parse_pair = |range: std::ops::Range<usize>| -> Result<u8, CssParseError> {
        u8::from_str_radix(&hex[range], 16)
            .map_err(|_| CssParseError::new(format!("invalid color `{input}`")))
    };
    match hex.len() {
        6 => Ok(Color::rgb(
            parse_pair(0..2)?,
            parse_pair(2..4)?,
            parse_pair(4..6)?,
        )),
        8 => Ok(Color::rgba(
            parse_pair(0..2)?,
            parse_pair(2..4)?,
            parse_pair(4..6)?,
            parse_pair(6..8)?,
        )),
        _ => Err(CssParseError::new(format!("invalid color `{input}`"))),
    }
}

fn is_ident_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_')
}
