use des_ui_document::{
    DocumentEngine, DocumentOutput, ElementId, Point, Rect, ScrollAxis, ScrollChrome,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AutoScrollOptions {
    pub acceleration: f32,
    pub threshold_x: f32,
    pub threshold_y: f32,
    pub tolerance: f32,
}

impl Default for AutoScrollOptions {
    fn default() -> Self {
        Self {
            acceleration: 14.0,
            threshold_x: 0.2,
            threshold_y: 0.2,
            tolerance: 10.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AutoScrollAction {
    pub element_id: ElementId,
    pub delta: Point,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AutoScroller {
    options: AutoScrollOptions,
}

impl AutoScroller {
    pub fn new(options: AutoScrollOptions) -> Self {
        Self { options }
    }

    pub fn scroll_drag(
        self,
        engine: &mut DocumentEngine,
        output: &DocumentOutput,
        pointer: Point,
    ) -> Option<AutoScrollAction> {
        self.scroll_drag_with_filter(engine, output, pointer, |_| true)
    }

    pub fn scroll_drag_with_filter(
        self,
        engine: &mut DocumentEngine,
        output: &DocumentOutput,
        pointer: Point,
        mut accepts_element: impl FnMut(&ElementId) -> bool,
    ) -> Option<AutoScrollAction> {
        if let Some(path) = output.snapshot().path_at(pointer) {
            for element in path.iter().rev() {
                if !accepts_element(element.id()) {
                    continue;
                }
                if let Some(action) =
                    self.scroll_element(engine, output, element.id(), element.rect(), pointer)
                {
                    return Some(action);
                }
            }
        }

        for chrome in &output.scroll_chrome {
            if !accepts_element(&chrome.element_id) {
                continue;
            }
            let Some(element) = output.snapshot().find(chrome.element_id.as_str()) else {
                continue;
            };
            if !element.rect().contains(pointer) {
                continue;
            }
            if let Some(action) =
                self.scroll_element(engine, output, &chrome.element_id, element.rect(), pointer)
            {
                return Some(action);
            }
        }

        None
    }

    fn scroll_element(
        self,
        engine: &mut DocumentEngine,
        output: &DocumentOutput,
        element_id: &ElementId,
        rect: Rect,
        pointer: Point,
    ) -> Option<AutoScrollAction> {
        let horizontal = scroll_chrome(output, element_id, ScrollAxis::Horizontal);
        let vertical = scroll_chrome(output, element_id, ScrollAxis::Vertical);
        let delta = Point::new(
            horizontal
                .and_then(|chrome| {
                    axis_delta(
                        pointer.x,
                        pointer.y,
                        rect,
                        chrome,
                        self.options.threshold_x,
                        self.options.acceleration,
                        self.options.tolerance,
                        Axis::Horizontal,
                    )
                })
                .unwrap_or(0.0),
            vertical
                .and_then(|chrome| {
                    axis_delta(
                        pointer.y,
                        pointer.x,
                        rect,
                        chrome,
                        self.options.threshold_y,
                        self.options.acceleration,
                        self.options.tolerance,
                        Axis::Vertical,
                    )
                })
                .unwrap_or(0.0),
        );

        if (delta.x.abs() > f32::EPSILON || delta.y.abs() > f32::EPSILON)
            && engine.scroll_element_by(element_id.as_str(), delta)
        {
            return Some(AutoScrollAction {
                element_id: element_id.clone(),
                delta,
            });
        }

        None
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Axis {
    Horizontal,
    Vertical,
}

fn scroll_chrome<'a>(
    output: &'a DocumentOutput,
    element_id: &ElementId,
    axis: ScrollAxis,
) -> Option<&'a ScrollChrome> {
    output
        .scroll_chrome
        .iter()
        .find(|chrome| chrome.element_id == *element_id && chrome.axis == axis)
}

fn axis_delta(
    main: f32,
    cross: f32,
    rect: Rect,
    chrome: &ScrollChrome,
    threshold_ratio: f32,
    acceleration: f32,
    tolerance: f32,
    axis: Axis,
) -> Option<f32> {
    let threshold_ratio = threshold_ratio.clamp(0.0, 1.0);
    if threshold_ratio <= f32::EPSILON || acceleration <= f32::EPSILON {
        return None;
    }

    let (start, end, cross_start, cross_end, size) = match axis {
        Axis::Horizontal => (
            rect.origin.x,
            rect.right(),
            rect.origin.y,
            rect.bottom(),
            rect.size.width,
        ),
        Axis::Vertical => (
            rect.origin.y,
            rect.bottom(),
            rect.origin.x,
            rect.right(),
            rect.size.height,
        ),
    };
    if cross < cross_start - tolerance || cross > cross_end + tolerance {
        return None;
    }

    let threshold = size * threshold_ratio;
    if threshold <= f32::EPSILON {
        return None;
    }

    let current_scroll = scroll_position(chrome);
    if main <= start + threshold && current_scroll > 0.0 {
        let factor = ((start + threshold - main) / threshold).clamp(0.0, 1.0);
        return Some(-acceleration * factor);
    }

    if main >= end - threshold && current_scroll < chrome.max_scroll {
        let factor = ((main - (end - threshold)) / threshold).clamp(0.0, 1.0);
        return Some(acceleration * factor);
    }

    None
}

fn scroll_position(chrome: &ScrollChrome) -> f32 {
    let track_main = match chrome.axis {
        ScrollAxis::Horizontal => chrome.track_rect.size.width,
        ScrollAxis::Vertical => chrome.track_rect.size.height,
    };
    let handle_main = match chrome.axis {
        ScrollAxis::Horizontal => chrome.handle_rect.size.width,
        ScrollAxis::Vertical => chrome.handle_rect.size.height,
    };
    let handle_offset = match chrome.axis {
        ScrollAxis::Horizontal => chrome.handle_rect.origin.x - chrome.track_rect.origin.x,
        ScrollAxis::Vertical => chrome.handle_rect.origin.y - chrome.track_rect.origin.y,
    };
    let travel = (track_main - handle_main).max(0.0);
    if travel <= f32::EPSILON || chrome.max_scroll <= f32::EPSILON {
        0.0
    } else {
        (handle_offset / travel).clamp(0.0, 1.0) * chrome.max_scroll
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use des_ui_document::{
        DocumentScene, ElementRole, ElementSpec, Overflow, Size, Style, StyleSelector, StyleSheet,
    };

    #[test]
    fn auto_scroller_scrolls_nearest_scrollable_container_near_bottom_edge() {
        let mut engine = DocumentEngine::default();
        let stylesheet = StyleSheet::new()
            .rule(
                StyleSelector::id("scroll-parent"),
                Style::default()
                    .size(120.0, 80.0)
                    .overflow_y(Overflow::Scroll),
            )
            .rule(
                StyleSelector::class("row"),
                Style::default().size(100.0, 32.0),
            );
        let mut scene = DocumentScene::build(Size::new(160.0, 120.0), |ui| {
            ui.element(
                "scroll-parent",
                ElementSpec::new(ElementRole::Panel),
                |ui| {
                    for index in 0..8 {
                        ui.element(
                            format!("row-{index}"),
                            ElementSpec::new(ElementRole::Card).class("row"),
                            |_| {},
                        );
                    }
                },
            );
        });
        let output = engine.update_scene(&mut scene, &stylesheet);
        let pointer = Point::new(40.0, 76.0);

        let action = AutoScroller::new(AutoScrollOptions::default())
            .scroll_drag(&mut engine, &output, pointer)
            .expect("pointer near the bottom should scroll");

        assert_eq!(action.element_id, ElementId::new("scroll-parent"));
        assert!(action.delta.y > 0.0);
        assert!(engine.element_state("scroll-parent").unwrap().scroll_y > 0.0);
    }

    #[test]
    fn auto_scroller_does_not_scroll_when_axis_threshold_is_disabled() {
        let mut engine = DocumentEngine::default();
        let stylesheet = StyleSheet::new()
            .rule(
                StyleSelector::id("scroll-parent"),
                Style::default()
                    .size(120.0, 80.0)
                    .overflow_y(Overflow::Scroll),
            )
            .rule(
                StyleSelector::class("row"),
                Style::default().size(100.0, 32.0),
            );
        let mut scene = DocumentScene::build(Size::new(160.0, 120.0), |ui| {
            ui.element(
                "scroll-parent",
                ElementSpec::new(ElementRole::Panel),
                |ui| {
                    for index in 0..8 {
                        ui.element(
                            format!("row-{index}"),
                            ElementSpec::new(ElementRole::Card).class("row"),
                            |_| {},
                        );
                    }
                },
            );
        });
        let output = engine.update_scene(&mut scene, &stylesheet);

        let action = AutoScroller::new(AutoScrollOptions {
            threshold_y: 0.0,
            ..AutoScrollOptions::default()
        })
        .scroll_drag(&mut engine, &output, Point::new(40.0, 76.0));

        assert!(action.is_none());
        assert_eq!(engine.element_state("scroll-parent").unwrap().scroll_y, 0.0);
    }
}
