#![allow(dead_code)]

use eframe::egui;
use egui::{FontData, FontDefinitions, FontFamily, FontId, TextStyle, vec2};
use std::{collections::BTreeMap, fs, path::PathBuf, sync::Arc};

const INTER_FONT_NAME: &str = "studio-inter";
const INTER_VARIABLE_FONT: &[u8] = include_bytes!("../assets/fonts/inter/InterVariable.ttf");

pub fn apply_host_configuration(context: &egui::Context) {
    apply_fonts(context);
    apply_text_metrics(context);
}

fn apply_fonts(context: &egui::Context) {
    let mut definitions = FontDefinitions::default();

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

    definitions.font_data.insert(
        INTER_FONT_NAME.to_string(),
        Arc::new(FontData::from_static(INTER_VARIABLE_FONT)),
    );
    definitions
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, INTER_FONT_NAME.to_string());

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

    let fallback_fonts: Vec<String> = definitions.font_data.keys().cloned().collect();
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

fn apply_text_metrics(context: &egui::Context) {
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
        style.visuals.text_options.font_hinting = false;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_configuration_disables_font_hinting_for_document_text() {
        let context = egui::Context::default();

        apply_host_configuration(&context);

        assert!(!context.global_style().visuals.text_options.font_hinting);
    }
}
