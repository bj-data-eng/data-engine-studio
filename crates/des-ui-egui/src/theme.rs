use eframe::egui;
use egui::Color32;

pub(crate) const BACKGROUND: Color32 = Color32::from_rgb(19, 22, 25);
pub(crate) const PANEL: Color32 = Color32::from_rgb(27, 31, 35);
pub(crate) const PANEL_SELECTED: Color32 = Color32::from_rgb(35, 56, 78);
pub(crate) const STROKE: Color32 = Color32::from_rgb(61, 68, 76);
pub(crate) const STROKE_SELECTED: Color32 = Color32::from_rgb(88, 157, 230);
pub(crate) const TEXT_MUTED: Color32 = Color32::from_rgb(156, 166, 176);
pub(crate) const CONNECTOR: Color32 = Color32::from_rgb(94, 162, 230);
pub(crate) const SOURCE_CONNECTOR: Color32 = Color32::from_rgb(95, 204, 140);

pub(crate) fn apply_visuals(context: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = BACKGROUND;
    visuals.window_fill = BACKGROUND;
    visuals.widgets.inactive.bg_fill = Color32::from_rgb(35, 39, 43);
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(48, 57, 66);
    visuals.selection.bg_fill = Color32::from_rgb(45, 100, 165);
    context.set_visuals(visuals);
}
