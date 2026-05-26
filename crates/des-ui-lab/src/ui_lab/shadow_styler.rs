use des_document::{
    Color, DocumentBuilder, DocumentWidget, Element, ElementSpec, ElementStateSelector, Glyph,
    Point, Shadow, Style, StyleSelector, StyleSheet,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct ShadowStyler {
    base: ShadowTuneState,
    hover: ShadowTuneState,
    shadow_color: Color,
}

impl ShadowStyler {
    pub(super) fn new(base: ShadowTuneState, hover: ShadowTuneState) -> Self {
        Self {
            base,
            hover,
            shadow_color: Color::rgb(0, 0, 0),
        }
    }

    pub(super) fn shadow_color(mut self, color: Color) -> Self {
        self.shadow_color = color;
        self
    }

    pub(super) fn action_for_command(command: &str) -> Option<ShadowStylerAction> {
        let rest = command.strip_prefix("shadow-tune-")?;
        let (target, rest) = if let Some(rest) = rest.strip_prefix("base-") {
            (ShadowTuneTarget::Base, rest)
        } else if let Some(rest) = rest.strip_prefix("hover-") {
            (ShadowTuneTarget::Hover, rest)
        } else {
            return None;
        };
        if let Some(layer) = rest
            .strip_prefix("layer-")
            .and_then(|value| value.strip_suffix("-toggle"))
            .and_then(|value| value.parse::<usize>().ok())
        {
            return Some(ShadowStylerAction::ToggleLayer { target, layer });
        }

        let mut parts = rest.split('-');
        let layer = parts.next()?.strip_prefix("l")?.parse::<usize>().ok()?;
        let field = match parts.next()? {
            "x" => ShadowTuneField::X,
            "y" => ShadowTuneField::Y,
            "blur" => ShadowTuneField::Blur,
            "spread" => ShadowTuneField::Spread,
            "alpha" => ShadowTuneField::Alpha,
            _ => return None,
        };
        let direction = match parts.next()? {
            "dec" => -1,
            "inc" => 1,
            _ => return None,
        };
        if parts.next().is_some() {
            return None;
        }
        Some(ShadowStylerAction::Adjust {
            target,
            layer,
            field,
            direction,
        })
    }

    pub(super) fn render(&self, ui: &mut DocumentBuilder) {
        ui.text_element(
            "shadow-tune-title",
            ElementSpec::new(Element::Text).class("section-title"),
            "Shadow Tuner",
        );
        ui.text_element(
            "shadow-tune-copy",
            ElementSpec::new(Element::Text)
                .class("muted")
                .class("param-description"),
            "Tune base and hover shadows by eye, then copy the numbers into the elevation recipe.",
        );
        ui.element(
            "shadow-tune-panel",
            ElementSpec::new(Element::Div).class("shadow-tune-panel"),
            |ui| {
                ui.element(
                    "shadow-tune-preview",
                    ElementSpec::new(Element::Div).class("shadow-tune-preview"),
                    |ui| {
                        self.preview_card(ui, "shadow-tune-preview-card-1", "base row 01");
                        self.preview_card(ui, "shadow-tune-preview-card-2", "hover row 02");
                        self.preview_card(ui, "shadow-tune-preview-card-3", "hover row 03");
                    },
                );
                ui.element(
                    "shadow-tune-controls",
                    ElementSpec::new(Element::Div).class("shadow-tune-controls"),
                    |ui| {
                        self.group(ui, ShadowTuneTarget::Base, self.base);
                        self.group(ui, ShadowTuneTarget::Hover, self.hover);
                        ui.text_element(
                            "shadow-tune-output",
                            ElementSpec::new(Element::Text)
                                .class("shadow-tune-output")
                                .selectable_text(),
                            self.output(),
                        );
                    },
                );
            },
        );
    }

    fn preview_card(&self, ui: &mut DocumentBuilder, id: &'static str, label: &'static str) {
        ui.element(
            id,
            ElementSpec::new(Element::Div)
                .class("shadow-tune-preview-card")
                .interactive(),
            |ui| {
                ui.text_element(
                    format!("{id}-label"),
                    ElementSpec::new(Element::Text).class("shadow-web-label"),
                    label,
                );
                ui.element(
                    format!("{id}-handle"),
                    ElementSpec::new(Element::Icon)
                        .class("shadow-web-handle")
                        .glyph(Glyph::DragHandle),
                    |_| {},
                );
            },
        );
    }

    fn group(&self, ui: &mut DocumentBuilder, target: ShadowTuneTarget, state: ShadowTuneState) {
        ui.element(
            format!("shadow-tune-{}-group", target.id_prefix()),
            ElementSpec::new(Element::Div).class("shadow-tune-group"),
            |ui| {
                ui.text_element(
                    format!("shadow-tune-{}-group-title", target.id_prefix()),
                    ElementSpec::new(Element::Text).class("section-title"),
                    format!("{} shadow", target.label()),
                );
                self.layer(ui, target, 0, state.layers[0]);
                self.layer(ui, target, 1, state.layers[1]);
            },
        );
    }

    fn layer(
        &self,
        ui: &mut DocumentBuilder,
        target: ShadowTuneTarget,
        layer_index: usize,
        layer: ShadowTuneLayer,
    ) {
        let target_id = target.id_prefix();
        let layer_id = format!("shadow-tune-{target_id}-layer-{layer_index}");
        ui.element(
            layer_id,
            ElementSpec::new(Element::Div).class("shadow-tune-layer"),
            |ui| {
                ui.element(
                    format!("shadow-tune-{target_id}-layer-{layer_index}-header"),
                    ElementSpec::new(Element::Div).class("shadow-tune-header"),
                    |ui| {
                        ui.text_element(
                            format!("shadow-tune-{target_id}-layer-{layer_index}-title"),
                            ElementSpec::new(Element::Text).class("card-title"),
                            format!(
                                "Layer {} ({})",
                                layer_index + 1,
                                if layer.enabled { "on" } else { "off" }
                            ),
                        );
                        ui.element(
                            format!("shadow-tune-{target_id}-layer-{layer_index}-toggle"),
                            ElementSpec::new(Element::Button)
                                .class("shadow-tune-toggle")
                                .on_click(format!(
                                    "shadow-tune-{target_id}-layer-{layer_index}-toggle"
                                )),
                            |ui| {
                                ui.text_element(
                                    format!(
                                        "shadow-tune-{target_id}-layer-{layer_index}-toggle-label"
                                    ),
                                    ElementSpec::new(Element::Text).class("button-label"),
                                    if layer.enabled { "Disable" } else { "Enable" },
                                );
                            },
                        );
                    },
                );
                self.control(ui, target, layer_index, "x", "x", format!("{:.0}", layer.x));
                self.control(ui, target, layer_index, "y", "y", format!("{:.0}", layer.y));
                self.control(
                    ui,
                    target,
                    layer_index,
                    "blur",
                    "blur",
                    format!("{:.0}", layer.blur),
                );
                self.control(
                    ui,
                    target,
                    layer_index,
                    "spread",
                    "spread",
                    format!("{:.0}", layer.spread),
                );
                self.control(
                    ui,
                    target,
                    layer_index,
                    "alpha",
                    "alpha",
                    layer.alpha.to_string(),
                );
            },
        );
    }

    fn control(
        &self,
        ui: &mut DocumentBuilder,
        target: ShadowTuneTarget,
        layer_index: usize,
        field: &'static str,
        label: &'static str,
        value: String,
    ) {
        let target_id = target.id_prefix();
        let row_id = format!("shadow-tune-{target_id}-l{layer_index}-{field}-row");
        ui.element(
            row_id,
            ElementSpec::new(Element::Div).class("shadow-tune-row"),
            |ui| {
                ui.text_element(
                    format!("shadow-tune-{target_id}-l{layer_index}-{field}-label"),
                    ElementSpec::new(Element::Text).class("shadow-tune-label"),
                    label,
                );
                self.button(ui, target, layer_index, field, "dec", "-");
                ui.text_element(
                    format!("shadow-tune-{target_id}-l{layer_index}-{field}-value"),
                    ElementSpec::new(Element::Text).class("shadow-tune-value"),
                    value,
                );
                self.button(ui, target, layer_index, field, "inc", "+");
            },
        );
    }

    fn button(
        &self,
        ui: &mut DocumentBuilder,
        target: ShadowTuneTarget,
        layer_index: usize,
        field: &'static str,
        direction: &'static str,
        label: &'static str,
    ) {
        let button_id = format!(
            "shadow-tune-{}-l{layer_index}-{field}-{direction}",
            target.id_prefix()
        );
        ui.element(
            button_id.clone(),
            ElementSpec::new(Element::Button)
                .class("shadow-tune-button")
                .on_click(button_id.clone()),
            |ui| {
                ui.text_element(
                    format!("{button_id}-label"),
                    ElementSpec::new(Element::Text).class("button-label"),
                    label,
                );
            },
        );
    }

    fn output(&self) -> String {
        let layer = |state: ShadowTuneState, index: usize| {
            let layer = state.layers[index];
            format!(
                "{}: x {:.0}, y {:.0}, blur {:.0}, spread {:.0}, alpha {}",
                if layer.enabled { "on" } else { "off" },
                layer.x,
                layer.y,
                layer.blur,
                layer.spread,
                layer.alpha
            )
        };
        format!(
            "base L1 {}; base L2 {}; hover L1 {}; hover L2 {}",
            layer(self.base, 0),
            layer(self.base, 1),
            layer(self.hover, 0),
            layer(self.hover, 1)
        )
    }
}

impl DocumentWidget for ShadowStyler {
    fn render(&self, ui: &mut DocumentBuilder) {
        ShadowStyler::render(self, ui);
    }

    fn push_styles(&self, stylesheet: &mut StyleSheet) {
        stylesheet.push_rule(
            StyleSelector::class("shadow-tune-preview-card"),
            Style::default().shadows(self.base.shadows(self.shadow_color)),
        );
        stylesheet.push_rule(
            StyleSelector::class_state("shadow-tune-preview-card", ElementStateSelector::Hovered),
            Style::default().shadows(self.hover.shadows(self.shadow_color)),
        );
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum ShadowStylerAction {
    Adjust {
        target: ShadowTuneTarget,
        layer: usize,
        field: ShadowTuneField,
        direction: i8,
    },
    ToggleLayer {
        target: ShadowTuneTarget,
        layer: usize,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ShadowTuneTarget {
    Base,
    Hover,
}

impl ShadowTuneTarget {
    fn id_prefix(self) -> &'static str {
        match self {
            Self::Base => "base",
            Self::Hover => "hover",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Base => "Base",
            Self::Hover => "Hover",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ShadowTuneField {
    X,
    Y,
    Blur,
    Spread,
    Alpha,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct ShadowTuneState {
    layers: [ShadowTuneLayer; 2],
}

impl Default for ShadowTuneState {
    fn default() -> Self {
        Self {
            layers: [
                ShadowTuneLayer {
                    enabled: true,
                    x: 0.0,
                    y: 0.0,
                    blur: 7.0,
                    spread: -7.0,
                    alpha: 80,
                },
                ShadowTuneLayer {
                    enabled: false,
                    x: 0.0,
                    y: 0.0,
                    blur: 0.0,
                    spread: 0.0,
                    alpha: 0,
                },
            ],
        }
    }
}

impl ShadowTuneState {
    pub(super) fn hover_default() -> Self {
        Self {
            layers: [
                ShadowTuneLayer {
                    enabled: true,
                    x: 0.0,
                    y: 5.0,
                    blur: 20.0,
                    spread: -15.0,
                    alpha: 80,
                },
                ShadowTuneLayer {
                    enabled: false,
                    x: 10.0,
                    y: 20.0,
                    blur: 15.0,
                    spread: -15.0,
                    alpha: 10,
                },
            ],
        }
    }

    pub(super) fn adjust(&mut self, layer: usize, field: ShadowTuneField, direction: i8) {
        let Some(layer) = self.layers.get_mut(layer) else {
            return;
        };
        let sign = if direction < 0 { -1.0 } else { 1.0 };
        match field {
            ShadowTuneField::X => layer.x = (layer.x + sign).clamp(-80.0, 80.0),
            ShadowTuneField::Y => layer.y = (layer.y + sign).clamp(-80.0, 80.0),
            ShadowTuneField::Blur => layer.blur = (layer.blur + sign).clamp(0.0, 120.0),
            ShadowTuneField::Spread => layer.spread = (layer.spread + sign).clamp(-40.0, 40.0),
            ShadowTuneField::Alpha => {
                let next = layer.alpha as i16 + if direction < 0 { -1 } else { 1 };
                layer.alpha = next.clamp(0, 255) as u8;
            }
        }
    }

    pub(super) fn toggle(&mut self, layer: usize) {
        if let Some(layer) = self.layers.get_mut(layer) {
            layer.enabled = !layer.enabled;
        }
    }

    fn shadows(self, color: Color) -> Vec<Shadow> {
        self.layers
            .into_iter()
            .filter(|layer| layer.enabled && layer.alpha > 0)
            .map(|layer| Shadow {
                offset: Point::new(layer.x, layer.y),
                blur: layer.blur,
                spread: layer.spread,
                color: Color {
                    a: layer.alpha,
                    ..color
                },
            })
            .collect()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct ShadowTuneLayer {
    enabled: bool,
    x: f32,
    y: f32,
    blur: f32,
    spread: f32,
    alpha: u8,
}
