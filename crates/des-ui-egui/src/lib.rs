use des_app::StudioAppState;
use des_core::{APP_INTERNAL_ID, APP_NAME, StudioResult};
use eframe::egui;
use egui::{Align, Color32, Layout, RichText, Stroke};

const MIN_WINDOW_WIDTH: f32 = 1080.0;
const MIN_WINDOW_HEIGHT: f32 = 680.0;

#[derive(Clone, Debug)]
pub struct NativeLaunchOptions {
    pub title: String,
}

impl Default for NativeLaunchOptions {
    fn default() -> Self {
        Self {
            title: APP_NAME.to_string(),
        }
    }
}

pub fn run_native(options: NativeLaunchOptions) -> StudioResult<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(options.title.clone())
            .with_app_id(APP_INTERNAL_ID)
            .with_inner_size([1320.0, 780.0])
            .with_min_inner_size([MIN_WINDOW_WIDTH, MIN_WINDOW_HEIGHT]),
        persist_window: false,
        centered: true,
        ..Default::default()
    };

    eframe::run_native(
        &options.title,
        native_options,
        Box::new(|creation_context| {
            apply_visuals(&creation_context.egui_ctx);
            Ok(Box::<StudioEguiApp>::default())
        }),
    )
    .map_err(|error| des_core::StudioError::new(error.to_string()))
}

#[derive(Default)]
struct StudioEguiApp {
    state: StudioAppState,
}

impl eframe::App for StudioEguiApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::top("top_bar")
            .exact_size(48.0)
            .show_inside(ui, |ui| self.render_top_bar(ui));

        egui::Panel::left("flow_list")
            .resizable(true)
            .default_size(300.0)
            .size_range(240.0..=420.0)
            .show_inside(ui, |ui| self.render_flow_list(ui));

        egui::Panel::bottom("status_bar")
            .exact_size(28.0)
            .show_inside(ui, |ui| self.render_status_bar(ui));

        egui::CentralPanel::default().show_inside(ui, |ui| self.render_flow_workspace(ui));
    }
}

impl StudioEguiApp {
    fn render_top_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            ui.heading(RichText::new(APP_NAME).strong());
            ui.separator();
            let _ = ui.button("New Flow");
            let _ = ui.button("Open");
            let _ = ui.button("Save");
            ui.separator();
            let _ = ui.button("Validate");
            let _ = ui.button("Run");
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(format!("v{}", self.state.home().app_info.version));
            });
        });
    }

    fn render_flow_list(&mut self, ui: &mut egui::Ui) {
        ui.add_space(8.0);
        ui.heading("Flows");
        ui.label(
            RichText::new("Primary workspace list")
                .small()
                .color(Color32::GRAY),
        );
        ui.add_space(8.0);

        let selected_id = self.state.selected_flow_id().map(str::to_string);
        let flows = self.state.home().flows.clone();
        for flow in flows {
            let selected = selected_id.as_deref() == Some(flow.id.as_str());
            let response = egui::Frame::new()
                .fill(if selected {
                    Color32::from_rgb(38, 57, 78)
                } else {
                    Color32::from_rgb(28, 31, 34)
                })
                .stroke(Stroke::new(
                    1.0,
                    if selected {
                        Color32::from_rgb(85, 155, 230)
                    } else {
                        Color32::from_rgb(55, 60, 66)
                    },
                ))
                .corner_radius(6.0)
                .inner_margin(10.0)
                .show(ui, |ui| {
                    ui.label(RichText::new(&flow.name).strong());
                    ui.label(RichText::new(&flow.description).small());
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(format!("{} nodes", flow.node_count)).small());
                        ui.separator();
                        ui.label(RichText::new(&flow.trigger).small());
                    });
                })
                .response;

            if response.interact(egui::Sense::click()).clicked() {
                self.state.select_flow(flow.id);
            }
            ui.add_space(8.0);
        }
    }

    fn render_flow_workspace(&mut self, ui: &mut egui::Ui) {
        ui.add_space(12.0);
        let selected = self
            .state
            .home()
            .flows
            .iter()
            .find(|flow| Some(flow.id.as_str()) == self.state.selected_flow_id());

        if let Some(flow) = selected {
            ui.heading(&flow.name);
            ui.label("Expanded flow editor placeholder");
            ui.add_space(12.0);
            egui::Frame::canvas(ui.style())
                .stroke(Stroke::new(1.0, Color32::from_rgb(65, 70, 76)))
                .show(ui, |ui| {
                    ui.set_min_height(520.0);
                    ui.centered_and_justified(|ui| {
                        ui.label(
                            RichText::new("Node graph expansion view will open here.")
                                .size(18.0)
                                .color(Color32::from_rgb(190, 198, 207)),
                        );
                    });
                });
        }
    }

    fn render_status_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            ui.label("Ready");
            ui.separator();
            for diagnostic in &self.state.home().diagnostics {
                ui.label(&diagnostic.message);
            }
        });
    }
}

fn apply_visuals(context: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = Color32::from_rgb(22, 24, 27);
    visuals.window_fill = Color32::from_rgb(25, 28, 31);
    visuals.widgets.inactive.bg_fill = Color32::from_rgb(35, 39, 43);
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(48, 57, 66);
    visuals.selection.bg_fill = Color32::from_rgb(45, 100, 165);
    context.set_visuals(visuals);
}
