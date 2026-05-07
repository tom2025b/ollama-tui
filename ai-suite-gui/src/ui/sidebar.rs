use egui::{
    ComboBox, CursorIcon, Frame, Label, Margin, RichText, Rounding, ScrollArea, Stroke, Ui,
};

use crate::app::App;
use crate::commands;
use crate::theme;

use super::status_pill;

impl App {
    pub(super) fn render_sidebar(&mut self, ctx: &egui::Context) {
        let colors = theme::palette();
        egui::SidePanel::left("sidebar")
            .exact_width(292.0)
            .resizable(false)
            .frame(Frame::none().fill(colors.sidebar_bg).inner_margin(Margin {
                left: 16.0,
                right: 16.0,
                top: 16.0,
                bottom: 16.0,
            }))
            .show(ctx, |ui| {
                ScrollArea::vertical()
                    .id_source("sidebar_scroll")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        self.render_model_selector(ui);
                        ui.add_space(20.0);
                        self.render_model_list(ui);
                        ui.add_space(20.0);
                        self.render_command_list(ui);
                    });
            });
    }

    fn render_model_selector(&mut self, ui: &mut Ui) {
        let colors = theme::palette();
        section_label(ui, "Current Model");

        let before = self.selected_model_id.clone();
        Frame::none()
            .fill(colors.panel)
            .stroke(Stroke::new(1.0, colors.border))
            .rounding(Rounding::same(8.0))
            .inner_margin(Margin::same(12.0))
            .show(ui, |ui| {
                ui.label(
                    RichText::new(self.selected_model_label())
                        .size(self.text_size(15.0))
                        .strong()
                        .color(colors.text),
                );
                ui.add_space(6.0);
                ComboBox::from_id_source("model_selector")
                    .width(ui.available_width())
                    .selected_text("Change model")
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.selected_model_id, None, "Auto Router");
                        for model in &self.models {
                            ui.add_enabled_ui(model.enabled, |ui| {
                                ui.selectable_value(
                                    &mut self.selected_model_id,
                                    Some(model.id.clone()),
                                    &model.label,
                                );
                            });
                        }
                    });
                ui.add_space(8.0);
                let mode = if self.selected_model_id.is_some() {
                    "Pinned"
                } else {
                    "Auto Router"
                };
                status_pill(ui, "Mode", mode, colors.accent);
            });

        if before != self.selected_model_id {
            self.status = format!("Selected {}", self.selected_model_label());
        }
    }

    fn render_model_list(&mut self, ui: &mut Ui) {
        let colors = theme::palette();
        section_label(ui, "Backends");
        let mut clicked_model: Option<(String, String)> = None;

        for model in &self.models {
            let selected = self
                .selected_model_id
                .as_ref()
                .map(|id| id == &model.id)
                .unwrap_or(false);
            let border = if selected {
                colors.accent
            } else {
                colors.border
            };
            let dot = if model.enabled {
                colors.success
            } else {
                colors.warning
            };

            let response = Frame::none()
                .fill(if selected {
                    colors.accent_soft
                } else {
                    colors.panel
                })
                .stroke(Stroke::new(1.0, border))
                .rounding(Rounding::same(8.0))
                .inner_margin(Margin::same(11.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let (rect, _) =
                            ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::hover());
                        ui.painter().rect_filled(rect, Rounding::same(4.0), dot);
                        ui.vertical(|ui| {
                            ui.label(
                                RichText::new(&model.label)
                                    .size(self.text_size(13.5))
                                    .strong()
                                    .color(colors.text),
                            );
                            let detail = if model.enabled {
                                model.strengths.join(", ")
                            } else {
                                model
                                    .disabled_reason
                                    .clone()
                                    .unwrap_or_else(|| "unavailable".to_string())
                            };
                            ui.add(
                                Label::new(
                                    RichText::new(detail)
                                        .size(self.text_size(11.5))
                                        .color(colors.text_subtle),
                                )
                                .wrap(),
                            );
                        });
                    });
                })
                .response;

            if model.enabled {
                let response = response
                    .interact(egui::Sense::click())
                    .on_hover_cursor(CursorIcon::PointingHand);
                if response.clicked() {
                    clicked_model = Some((model.id.clone(), model.label.clone()));
                }
            }
            ui.add_space(6.0);
        }

        if let Some((id, label)) = clicked_model {
            self.selected_model_id = Some(id);
            self.status = format!("Selected {label}");
        }
    }

    fn render_command_list(&mut self, ui: &mut Ui) {
        let colors = theme::palette();
        section_label(ui, "Commands");

        for command in commands::COMMANDS {
            ui.horizontal(|ui| {
                if ui
                    .small_button(RichText::new(command.name).monospace().color(colors.accent))
                    .clicked()
                {
                    self.input = command.name.to_string();
                }
                ui.add(
                    Label::new(
                        RichText::new(command.hint)
                            .size(self.text_size(12.0))
                            .color(colors.text_subtle),
                    )
                    .wrap(),
                );
            });
            ui.add_space(2.0);
        }
    }
}

fn section_label(ui: &mut Ui, label: &str) {
    let colors = theme::palette();
    ui.label(
        RichText::new(label)
            .small()
            .strong()
            .color(colors.text_muted),
    );
    ui.add_space(7.0);
}
