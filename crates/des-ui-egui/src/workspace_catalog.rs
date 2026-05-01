use crate::theme;
use des_app::{
    AppCommand, AppSnapshot, FlowGroupSummary, FlowSummary, ProjectSummary, StudioAppState,
};
use eframe::egui;
use egui::{Color32, Rect, RichText, Stroke, TextEdit, scroll_area::ScrollAreaOutput, vec2};
use std::collections::BTreeSet;

const PANEL_WIDTH: f32 = 200.0;
const PANEL_MARGIN: f32 = 8.0;
const PANEL_INNER_MARGIN: f32 = 8.0;

#[derive(Default)]
pub(crate) struct WorkspaceCatalogState {
    project_filter: String,
    flow_filter: String,
    expanded_project_ids: BTreeSet<String>,
    expanded_group_ids: BTreeSet<String>,
    collapsed_project_ids: BTreeSet<String>,
    collapsed_group_ids: BTreeSet<String>,
    selected_flow_anchor: Option<Rect>,
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct WorkspaceCatalogOutput {
    pub(crate) selected_flow_anchor: Option<Rect>,
}

impl WorkspaceCatalogState {
    pub(crate) fn selected_flow_anchor(&self) -> Option<Rect> {
        self.selected_flow_anchor
    }
}

pub(crate) fn panel_rect(graph_rect: Rect) -> Rect {
    let height = (graph_rect.height() - PANEL_MARGIN * 2.0).max(320.0);
    Rect::from_min_size(
        graph_rect.left_top() + vec2(PANEL_MARGIN, PANEL_MARGIN),
        vec2(PANEL_WIDTH, height),
    )
}

pub(crate) fn render(
    ui: &mut egui::Ui,
    rect: Rect,
    state: &mut StudioAppState,
    catalog: &mut WorkspaceCatalogState,
    snapshot: &AppSnapshot,
) -> WorkspaceCatalogOutput {
    let mut output = WorkspaceCatalogOutput::default();

    let layer_id = egui::LayerId::new(
        egui::Order::Middle,
        egui::Id::new("workspace_catalog_panel_layer"),
    );
    ui.scope_builder(
        egui::UiBuilder::new().max_rect(rect).layer_id(layer_id),
        |ui| {
            egui::Frame::new()
                .fill(theme::PANEL)
                .stroke(Stroke::new(1.0, theme::STROKE))
                .corner_radius(8.0)
                .inner_margin(PANEL_INNER_MARGIN)
                .show(ui, |ui| {
                    let content_size = vec2(
                        (rect.width() - PANEL_INNER_MARGIN * 2.0).max(1.0),
                        (rect.height() - PANEL_INNER_MARGIN * 2.0).max(1.0),
                    );
                    ui.set_min_size(content_size);
                    ui.set_max_size(content_size);
                    ui.set_width(content_size.x);
                    ui.spacing_mut().item_spacing = vec2(4.0, 5.0);

                    let scroll_output = egui::ScrollArea::vertical()
                        .id_salt("workspace_catalog_panel_scroll")
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.label(theme::graph_heading_at("Catalog", 0.78));

                            render_root_selector(ui, state, snapshot);
                            ui.separator();
                            output.selected_flow_anchor =
                                render_project_tree(ui, state, snapshot, catalog);
                        });
                    route_touchpad_scroll_to_catalog(ui, rect, &scroll_output);
                });
        },
    );

    catalog.selected_flow_anchor = output.selected_flow_anchor;
    output
}

fn route_touchpad_scroll_to_catalog<R>(
    ui: &mut egui::Ui,
    rect: Rect,
    scroll_output: &ScrollAreaOutput<R>,
) {
    let (pointer_pos, scroll_delta) = ui.input(|input| {
        (
            input
                .pointer
                .hover_pos()
                .or_else(|| input.pointer.interact_pos())
                .or_else(|| input.pointer.latest_pos()),
            input.smooth_scroll_delta(),
        )
    });
    if scroll_delta.y == 0.0 || !pointer_pos.is_some_and(|pos| rect.contains(pos)) {
        return;
    }

    let max_offset = (scroll_output.content_size.y - scroll_output.inner_rect.height()).max(0.0);
    if max_offset <= 0.0 {
        return;
    }

    let mut scroll_state = scroll_output.state;
    let next_offset = (scroll_state.offset.y - scroll_delta.y).clamp(0.0, max_offset);
    if (next_offset - scroll_state.offset.y).abs() <= f32::EPSILON {
        return;
    }

    scroll_state.offset.y = next_offset;
    scroll_state.store(ui.ctx(), scroll_output.id);
    ui.input_mut(|input| {
        input.smooth_scroll_delta.y = 0.0;
    });
    ui.ctx().request_repaint();
}

fn render_root_selector(ui: &mut egui::Ui, state: &mut StudioAppState, snapshot: &AppSnapshot) {
    let selected_root = snapshot
        .home
        .workspace_roots
        .iter()
        .find(|root| Some(root.id.as_str()) == snapshot.selected_root_id.as_deref());
    let selected_label = selected_root
        .map(|root| root.name.as_str())
        .unwrap_or("No root selected");

    egui::ComboBox::from_id_salt("workspace_root_selector")
        .selected_text(selected_label)
        .width(ui.available_width())
        .show_ui(ui, |ui| {
            for root in &snapshot.home.workspace_roots {
                let selected = Some(root.id.as_str()) == snapshot.selected_root_id.as_deref();
                if ui
                    .selectable_label(
                        selected,
                        format!("{}  -  {} workspaces", root.name, root.workspace_count),
                    )
                    .clicked()
                {
                    state.dispatch(AppCommand::SelectWorkspaceRoot {
                        root_id: root.id.clone(),
                    });
                }
            }
        });

    if let Some(root) = selected_root {
        ui.label(theme::metadata_at(&root.path, 0.84));
    }
}

fn render_project_tree(
    ui: &mut egui::Ui,
    state: &mut StudioAppState,
    snapshot: &AppSnapshot,
    catalog: &mut WorkspaceCatalogState,
) -> Option<Rect> {
    section_header(ui, "Projects");
    search_box(
        ui,
        "project_filter",
        "Filter projects",
        &mut catalog.project_filter,
    );
    search_box(ui, "flow_filter", "Filter flows", &mut catalog.flow_filter);

    let filter = catalog.project_filter.trim().to_lowercase();
    let projects: Vec<_> = snapshot
        .home
        .projects
        .iter()
        .filter(|project| {
            Some(project.workspace_id.as_str()) == snapshot.selected_workspace_id.as_deref()
        })
        .filter(|project| {
            filter.is_empty()
                || project.name.to_lowercase().contains(&filter)
                || project.description.to_lowercase().contains(&filter)
                || project.status.to_lowercase().contains(&filter)
        })
        .collect();

    if projects.is_empty() {
        empty_hint(ui, "No projects match this workspace.");
    }
    let mut selected_anchor = None;
    for project in projects {
        if let Some(anchor) = render_project_branch(ui, state, snapshot, catalog, project) {
            selected_anchor = Some(anchor);
        };
    }

    selected_anchor
}

fn render_project_branch(
    ui: &mut egui::Ui,
    state: &mut StudioAppState,
    snapshot: &AppSnapshot,
    catalog: &mut WorkspaceCatalogState,
    project: &ProjectSummary,
) -> Option<Rect> {
    let selected = Some(project.id.as_str()) == snapshot.selected_project_id.as_deref();
    let open = catalog.expanded_project_ids.contains(&project.id)
        || (selected && !catalog.collapsed_project_ids.contains(&project.id));
    let mut selected_anchor = None;
    let response = compact_card_frame(selected, 6.0).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            ui.label(expander(open));
            ui.label(RichText::new(&project.name).size(12.6).strong());
        });
        ui.horizontal(|ui| {
            status_badge(ui, &project.status);
            ui.label(theme::metadata_at(
                format!("{}g / {}f", project.group_count, project.flow_count),
                0.82,
            ));
        });
    });
    if response.response.interact(egui::Sense::click()).clicked() {
        toggle_expansion(
            &mut catalog.expanded_project_ids,
            &mut catalog.collapsed_project_ids,
            &project.id,
            open,
        );
        state.dispatch(AppCommand::SelectProject {
            project_id: project.id.clone(),
        });
    }

    if open {
        ui.indent(ui.id().with((&project.id, "groups")), |ui| {
            ui.spacing_mut().item_spacing = vec2(3.0, 4.0);
            let groups: Vec<_> = snapshot
                .home
                .flow_groups
                .iter()
                .filter(|group| group.project_id.as_str() == project.id.as_str())
                .collect();

            for group in groups {
                if let Some(anchor) = render_group_branch(ui, state, snapshot, catalog, group) {
                    selected_anchor = Some(anchor);
                }
            }
        });
    }

    selected_anchor
}

fn render_group_branch(
    ui: &mut egui::Ui,
    state: &mut StudioAppState,
    snapshot: &AppSnapshot,
    catalog: &mut WorkspaceCatalogState,
    group: &FlowGroupSummary,
) -> Option<Rect> {
    let selected = Some(group.id.as_str()) == snapshot.selected_group_id.as_deref();
    let open = catalog.expanded_group_ids.contains(&group.id)
        || (selected && !catalog.collapsed_group_ids.contains(&group.id));
    let response = group_card_frame(selected).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            ui.label(expander(open));
            ui.label(RichText::new(&group.name).size(11.8).strong());
        });
        ui.horizontal(|ui| {
            ui.label(theme::metadata_at(&group.kind, 0.8));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(theme::metadata_at(format!("{}", group.flow_count), 0.8));
            });
        });
    });
    if response.response.interact(egui::Sense::click()).clicked() {
        toggle_expansion(
            &mut catalog.expanded_group_ids,
            &mut catalog.collapsed_group_ids,
            &group.id,
            open,
        );
        state.dispatch(AppCommand::SelectFlowGroup {
            group_id: group.id.clone(),
        });
    }

    if !open {
        return None;
    }

    let filter = catalog.flow_filter.trim().to_lowercase();
    let flows: Vec<_> = snapshot
        .home
        .flows
        .iter()
        .filter(|flow| flow.group_id.as_str() == group.id.as_str())
        .filter(|flow| {
            filter.is_empty()
                || flow.name.to_lowercase().contains(&filter)
                || flow.description.to_lowercase().contains(&filter)
                || flow.trigger.to_lowercase().contains(&filter)
        })
        .collect();
    let mut selected_anchor = None;

    if !flows.is_empty() {
        ui.indent(ui.id().with((&group.id, "flows")), |ui| {
            for flow in flows {
                let anchor = render_flow_subcard(ui, state, snapshot, flow);
                if anchor.is_some() {
                    selected_anchor = anchor;
                }
            }
        });
    }

    selected_anchor
}

fn render_flow_subcard(
    ui: &mut egui::Ui,
    state: &mut StudioAppState,
    snapshot: &AppSnapshot,
    flow: &FlowSummary,
) -> Option<Rect> {
    let selected = Some(flow.id.as_str()) == snapshot.selected_flow_id.as_deref();
    let response = flow_card_frame(selected).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.add(
            egui::Label::new(RichText::new(&flow.name).size(11.4).strong())
                .wrap()
                .selectable(false),
        );
        ui.horizontal(|ui| {
            ui.label(theme::metadata_at(
                format!("{} nodes", flow.node_count),
                0.78,
            ));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(theme::metadata_at(&flow.trigger, 0.78));
            });
        });
    });
    let response_rect = response.response.rect;
    let response = response
        .response
        .on_hover_text(&flow.description)
        .interact(egui::Sense::click());
    if response.clicked() {
        state.dispatch(AppCommand::SelectFlow {
            flow_id: flow.id.clone(),
        });
    }
    selected.then_some(response_rect)
}

fn toggle_expansion(
    expanded: &mut BTreeSet<String>,
    collapsed: &mut BTreeSet<String>,
    id: &str,
    open: bool,
) {
    if open {
        expanded.remove(id);
        collapsed.insert(id.to_string());
    } else {
        collapsed.remove(id);
        expanded.insert(id.to_string());
    }
}

fn expander(open: bool) -> RichText {
    if open {
        RichText::new("-").size(11.0).color(theme::TEXT_MUTED)
    } else {
        RichText::new("+").size(11.0).color(theme::TEXT_MUTED)
    }
}

fn section_header(ui: &mut egui::Ui, label: &str) {
    ui.label(
        RichText::new(label)
            .size(12.5)
            .color(Color32::from_rgb(226, 232, 238))
            .strong(),
    );
    ui.add_space(2.0);
}

fn search_box(ui: &mut egui::Ui, id: &str, hint: &str, value: &mut String) {
    ui.push_id(id, |ui| {
        ui.add_sized(
            [ui.available_width(), 22.0],
            TextEdit::singleline(value)
                .hint_text(hint)
                .desired_width(f32::INFINITY),
        );
    });
}

fn status_badge(ui: &mut egui::Ui, label: &str) {
    let color = match label {
        "Active" | "Available" => theme::SOURCE_CONNECTOR,
        "Draft" | "Design" => Color32::from_rgb(224, 177, 79),
        _ => theme::TEXT_MUTED,
    };
    ui.label(RichText::new(label).size(10.5).color(color).strong());
}

fn empty_hint(ui: &mut egui::Ui, label: &str) {
    egui::Frame::new()
        .fill(Color32::from_rgb(29, 34, 38))
        .stroke(Stroke::new(1.0, theme::STROKE))
        .corner_radius(6.0)
        .inner_margin(8.0)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(theme::metadata_at(label, 1.0));
        });
}

fn compact_card_frame(selected: bool, margin: f32) -> egui::Frame {
    egui::Frame::new()
        .fill(if selected {
            theme::PANEL_SELECTED
        } else {
            Color32::from_rgb(31, 36, 41)
        })
        .stroke(Stroke::new(
            1.0,
            if selected {
                theme::STROKE_SELECTED
            } else {
                theme::STROKE
            },
        ))
        .corner_radius(6.0)
        .inner_margin(margin)
}

fn group_card_frame(selected: bool) -> egui::Frame {
    egui::Frame::new()
        .fill(if selected {
            Color32::from_rgb(36, 54, 72)
        } else {
            Color32::from_rgb(29, 34, 39)
        })
        .stroke(Stroke::new(
            1.0,
            if selected {
                theme::STROKE_SELECTED
            } else {
                theme::STROKE
            },
        ))
        .corner_radius(5.0)
        .inner_margin(5.0)
}

fn flow_card_frame(selected: bool) -> egui::Frame {
    egui::Frame::new()
        .fill(if selected {
            Color32::from_rgb(35, 64, 91)
        } else {
            Color32::from_rgb(25, 30, 35)
        })
        .stroke(Stroke::new(
            1.0,
            if selected {
                theme::STROKE_SELECTED
            } else {
                Color32::from_rgb(45, 52, 60)
            },
        ))
        .corner_radius(5.0)
        .inner_margin(5.0)
}
