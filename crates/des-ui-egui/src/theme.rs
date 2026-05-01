#![allow(dead_code)]

use eframe::egui;
use egui::{Color32, FontData, FontDefinitions, FontFamily, FontId, TextStyle, vec2};
use iconflow::{Pack, Size, Style, try_icon};
use std::{collections::BTreeMap, fs, path::PathBuf, sync::Arc};

pub(crate) const BACKGROUND: Color32 = Color32::from_rgb(19, 22, 25);
pub(crate) const PANEL: Color32 = Color32::from_rgb(27, 31, 35);
pub(crate) const PANEL_SELECTED: Color32 = Color32::from_rgb(35, 56, 78);
pub(crate) const MENU_BAR: Color32 = Color32::from_rgb(23, 26, 30);
pub(crate) const STROKE: Color32 = Color32::from_rgb(61, 68, 76);
pub(crate) const STROKE_SELECTED: Color32 = Color32::from_rgb(88, 157, 230);
pub(crate) const TEXT_MUTED: Color32 = Color32::from_rgb(156, 166, 176);
pub(crate) const CONNECTOR: Color32 = Color32::from_rgb(94, 162, 230);
pub(crate) const SOURCE_CONNECTOR: Color32 = Color32::from_rgb(95, 204, 140);

pub(crate) fn apply_theme(context: &egui::Context) {
    apply_fonts(context);
    apply_visuals(context);
    apply_text_styles(context);
}

fn apply_visuals(context: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = BACKGROUND;
    visuals.window_fill = BACKGROUND;
    visuals.widgets.inactive.bg_fill = Color32::from_rgb(35, 39, 43);
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(48, 57, 66);
    visuals.selection.bg_fill = Color32::from_rgb(45, 100, 165);
    context.set_visuals(visuals);
}

fn apply_fonts(context: &egui::Context) {
    let mut definitions = FontDefinitions::default();
    let fallback_fonts: Vec<String> = definitions.font_data.keys().cloned().collect();

    if let Some(font) = load_first_font(&system_ui_font_candidates()) {
        definitions
            .font_data
            .insert("studio-system-ui".to_string(), Arc::new(font));
        definitions
            .families
            .entry(FontFamily::Proportional)
            .or_default()
            .insert(0, "studio-system-ui".to_string());
    }

    if let Some(font) = load_first_font(&monospace_font_candidates()) {
        definitions
            .font_data
            .insert("studio-monospace".to_string(), Arc::new(font));
        definitions
            .families
            .entry(FontFamily::Monospace)
            .or_default()
            .insert(0, "studio-monospace".to_string());
    }

    for font in iconflow::fonts() {
        let family_name = font.family.to_string();
        definitions.font_data.insert(
            family_name.clone(),
            Arc::new(FontData::from_static(font.bytes)),
        );
        let family = definitions
            .families
            .entry(FontFamily::Name(font.family.into()))
            .or_default();
        family.insert(0, family_name.clone());
        for fallback in &fallback_fonts {
            if fallback != &family_name {
                family.push(fallback.clone());
            }
        }
    }

    context.set_fonts(definitions);
}

fn apply_text_styles(context: &egui::Context) {
    let text_styles: BTreeMap<_, _> = [
        (
            TextStyle::Small,
            FontId::new(11.0, FontFamily::Proportional),
        ),
        (TextStyle::Body, FontId::new(14.0, FontFamily::Proportional)),
        (
            TextStyle::Button,
            FontId::new(14.0, FontFamily::Proportional),
        ),
        (
            TextStyle::Heading,
            FontId::new(22.0, FontFamily::Proportional),
        ),
        (
            TextStyle::Monospace,
            FontId::new(13.0, FontFamily::Monospace),
        ),
    ]
    .into();

    context.all_styles_mut(move |style| {
        style.text_styles = text_styles.clone();
        style.spacing.item_spacing = vec2(8.0, 6.0);
        style.spacing.button_padding = vec2(10.0, 5.0);
        style.spacing.interact_size = vec2(36.0, 24.0);
    });
}

pub(crate) fn app_title_font() -> FontId {
    FontId::new(22.0, FontFamily::Proportional)
}

pub(crate) fn app_subtitle_font() -> FontId {
    FontId::new(12.5, FontFamily::Proportional)
}

pub(crate) fn node_title_at(text: impl Into<String>, scale: f32) -> egui::RichText {
    egui::RichText::new(text.into())
        .size(18.0 * scale)
        .color(Color32::from_rgb(224, 230, 236))
}

pub(crate) fn metadata_at(text: impl Into<String>, scale: f32) -> egui::RichText {
    egui::RichText::new(text.into())
        .size(11.0 * scale)
        .color(TEXT_MUTED)
}

pub(crate) fn graph_heading_at(text: impl Into<String>, scale: f32) -> egui::RichText {
    egui::RichText::new(text.into())
        .size(22.0 * scale)
        .color(Color32::from_rgb(228, 234, 240))
        .strong()
}

pub(crate) fn icon(name: &str, size: f32) -> egui::RichText {
    match try_icon(Pack::Lucide, name, Style::Regular, Size::Regular) {
        Ok(icon) => {
            let glyph = char::from_u32(icon.codepoint).unwrap_or('?');
            egui::RichText::new(glyph.to_string())
                .font(FontId::new(size, FontFamily::Name(icon.family.into())))
                .color(Color32::from_rgb(218, 226, 234))
        }
        Err(_) => egui::RichText::new("?").size(size).color(TEXT_MUTED),
    }
}

fn load_first_font(paths: &[PathBuf]) -> Option<FontData> {
    paths
        .iter()
        .find_map(|path| fs::read(path).ok().map(FontData::from_owned))
}

fn system_ui_font_candidates() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Some(windir) = std::env::var_os("WINDIR") {
        paths.push(PathBuf::from(&windir).join("Fonts").join("segoeui.ttf"));
        paths.push(PathBuf::from(&windir).join("Fonts").join("SegoeUI-VF.ttf"));
    }
    paths.push(PathBuf::from("/System/Library/Fonts/SFNS.ttf"));
    paths.push(PathBuf::from(
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
    ));
    paths
}

fn monospace_font_candidates() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Some(windir) = std::env::var_os("WINDIR") {
        paths.push(
            PathBuf::from(&windir)
                .join("Fonts")
                .join("CascadiaMono.ttf"),
        );
        paths.push(
            PathBuf::from(&windir)
                .join("Fonts")
                .join("CascadiaCode.ttf"),
        );
        paths.push(PathBuf::from(&windir).join("Fonts").join("consola.ttf"));
    }
    paths.push(PathBuf::from(
        "/System/Library/Fonts/MonacoSupplemental.ttf",
    ));
    paths.push(PathBuf::from(
        "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf",
    ));
    paths
}
