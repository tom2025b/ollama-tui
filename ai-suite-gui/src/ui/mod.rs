mod chat;
mod composer;
mod sidebar;

use egui::{Align, Color32, Frame, Layout, Margin, RichText, Rounding, ScrollArea, Stroke, Ui};

use crate::app::App;
use crate::commands;
use crate::settings::{MAX_TEXT_SCALE, MIN_TEXT_SCALE};
use crate::theme;

impl App {
    pub(crate) fn render_ui(&mut self, ctx: &egui::Context) {
        self.render_top_bar(ctx);
        self.render_sidebar(ctx);
        self.render_composer(ctx);
        if self.show_help {
            self.render_help_panel(ctx);
        } else {
            self.render_chat(ctx);
        }
        self.render_model_picker_window(ctx);
    }

    fn render_top_bar(&mut self, ctx: &egui::Context) {
        let colors = theme::palette();
        egui::TopBottomPanel::top("top_bar")
            .frame(Frame::none().fill(colors.panel).inner_margin(Margin {
                left: 22.0,
                right: 22.0,
                top: 12.0,
                bottom: 12.0,
            }))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    Frame::none()
                        .fill(colors.accent_soft)
                        .stroke(Stroke::new(1.0, colors.accent_strong))
                        .rounding(Rounding::same(6.0))
                        .inner_margin(Margin {
                            left: 8.0,
                            right: 8.0,
                            top: 5.0,
                            bottom: 5.0,
                        })
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new("ai")
                                    .strong()
                                    .size(self.text_size(13.0))
                                    .color(colors.accent),
                            );
                        });
                    ui.label(
                        RichText::new("ai-suite")
                            .strong()
                            .size(self.text_size(21.0))
                            .color(colors.text),
                    );
                    ui.label(
                        RichText::new("desktop")
                            .size(self.text_size(12.5))
                            .color(colors.text_subtle),
                    );
                    self.render_active_model_badge(ui);

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if header_button(ui, "Help", self.text_scale) {
                            self.show_help = true;
                            self.request_input_focus(ctx);
                        }
                        if header_button(ui, "Clear", self.text_scale) {
                            self.messages.clear();
                            self.status = "Conversation cleared".to_string();
                            self.request_input_focus(ctx);
                        }
                        if text_size_button(
                            ui,
                            "+",
                            self.text_scale < MAX_TEXT_SCALE,
                            self.text_scale,
                        ) {
                            self.increase_text_size();
                            self.request_input_focus(ctx);
                        }
                        ui.label(
                            RichText::new(self.text_scale_label())
                                .size(self.text_size(13.0))
                                .color(colors.text_muted),
                        );
                        if text_size_button(
                            ui,
                            "-",
                            self.text_scale > MIN_TEXT_SCALE,
                            self.text_scale,
                        ) {
                            self.decrease_text_size();
                            self.request_input_focus(ctx);
                        }
                        ui.label(
                            RichText::new(self.activity_label(ctx))
                                .size(self.text_size(13.0))
                                .color(colors.text_muted),
                        );
                    });
                });
                let rect = ui.max_rect();
                ui.painter().line_segment(
                    [rect.left_bottom(), rect.right_bottom()],
                    Stroke::new(1.0, colors.border),
                );
            });
    }

    fn render_active_model_badge(&self, ui: &mut Ui) {
        let colors = theme::palette();
        Frame::none()
            .fill(colors.accent_soft)
            .stroke(Stroke::new(1.0, colors.accent_strong))
            .rounding(Rounding::same(8.0))
            .inner_margin(Margin {
                left: 12.0,
                right: 12.0,
                top: 6.0,
                bottom: 6.0,
            })
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Active")
                            .small()
                            .strong()
                            .color(colors.text_subtle),
                    );
                    ui.label(
                        RichText::new(self.selected_model_label())
                            .size(self.text_size(13.0))
                            .strong()
                            .color(colors.accent),
                    );
                });
            });
    }

    fn render_help_panel(&mut self, ctx: &egui::Context) {
        let colors = theme::palette();
        egui::CentralPanel::default()
            .frame(Frame::none().fill(colors.chat_bg).inner_margin(Margin {
                left: 24.0,
                right: 24.0,
                top: 20.0,
                bottom: 20.0,
            }))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Command Reference")
                            .heading()
                            .strong()
                            .color(colors.text),
                    );
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if header_button(ui, "Close", self.text_scale) {
                            self.show_help = false;
                            self.status = "Help closed".to_string();
                            self.request_input_focus(ctx);
                        }
                    });
                });
                ui.add_space(6.0);
                ui.horizontal_wrapped(|ui| {
                    status_pill(ui, "Enter", "run exact command", colors.accent);
                    status_pill(ui, "Tab", "complete command", colors.success);
                    status_pill(ui, "Up/Down", "select popup row", colors.accent_warm);
                    status_pill(ui, "Esc", "dismiss popup", colors.warning);
                });
                ui.add_space(14.0);

                ScrollArea::vertical()
                    .id_source("help_scroll")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for command in commands::COMMANDS {
                            help_command_row(ui, command, self.text_scale);
                            ui.add_space(8.0);
                        }
                    });
            });
    }

    fn render_model_picker_window(&mut self, ctx: &egui::Context) {
        if !self.show_model_picker {
            return;
        }

        let colors = theme::palette();
        let mut open = self.show_model_picker;
        let mut picked: Option<Option<String>> = None;

        egui::Window::new("Model Picker")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .default_width(460.0)
            .frame(
                Frame::window(ctx.style().as_ref())
                    .fill(colors.panel)
                    .stroke(Stroke::new(1.0, colors.border_strong))
                    .rounding(Rounding::same(8.0)),
            )
            .show(ctx, |ui| {
                if model_pick_row(
                    ui,
                    "Auto Router",
                    "Let the router choose per prompt",
                    true,
                    self.text_scale,
                ) {
                    picked = Some(None);
                }
                ui.add_space(8.0);
                for model in &self.models {
                    let detail = if model.enabled {
                        model.strengths.join(", ")
                    } else {
                        model
                            .disabled_reason
                            .clone()
                            .unwrap_or_else(|| "unavailable".to_string())
                    };
                    if model_pick_row(ui, &model.label, &detail, model.enabled, self.text_scale) {
                        picked = Some(Some(model.id.clone()));
                    }
                    ui.add_space(6.0);
                }
            });

        if let Some(selection) = picked {
            self.selected_model_id = selection;
            self.status = format!("Selected {}", self.selected_model_label());
            self.request_input_focus(ctx);
            open = false;
        }

        self.show_model_picker = open;
    }

    pub(super) fn activity_label(&self, ctx: &egui::Context) -> String {
        if self.streaming {
            let dots = ((ctx.input(|input| input.time) * 4.0) as usize % 4).min(3);
            format!("Streaming{}", ".".repeat(dots))
        } else {
            self.status.clone()
        }
    }
}

pub(super) fn status_pill(ui: &mut Ui, label: &str, value: &str, accent: Color32) {
    let colors = theme::palette();
    Frame::none()
        .fill(colors.panel_alt)
        .stroke(Stroke::new(1.0, colors.border))
        .rounding(Rounding::same(8.0))
        .inner_margin(Margin {
            left: 11.0,
            right: 11.0,
            top: 6.0,
            bottom: 6.0,
        })
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(label).small().color(colors.text_subtle));
                ui.label(RichText::new(value).small().strong().color(accent));
            });
        });
}

fn header_button(ui: &mut Ui, label: &str, text_scale: f32) -> bool {
    let colors = theme::palette();
    ui.add(
        egui::Button::new(RichText::new(label).color(colors.text_muted))
            .fill(colors.panel_alt)
            .stroke(Stroke::new(1.0, colors.border))
            .rounding(Rounding::same(8.0))
            .min_size(egui::vec2(72.0 * text_scale.min(1.35), 32.0)),
    )
    .clicked()
}

fn text_size_button(ui: &mut Ui, label: &str, enabled: bool, text_scale: f32) -> bool {
    let colors = theme::palette();
    ui.add_enabled(
        enabled,
        egui::Button::new(
            RichText::new(label)
                .size(crate::theme::scaled_size(18.0, text_scale))
                .strong()
                .color(colors.text),
        )
        .fill(colors.panel_alt)
        .stroke(Stroke::new(1.0, colors.border))
        .rounding(Rounding::same(8.0))
        .min_size(egui::vec2(36.0, 32.0)),
    )
    .on_hover_text("Adjust text size")
    .clicked()
}

fn model_pick_row(ui: &mut Ui, label: &str, detail: &str, enabled: bool, text_scale: f32) -> bool {
    let colors = theme::palette();
    let response = ui.add_enabled_ui(enabled, |ui| {
        Frame::none()
            .fill(colors.panel_alt)
            .stroke(Stroke::new(1.0, colors.border))
            .rounding(Rounding::same(8.0))
            .inner_margin(Margin::same(12.0))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(
                    RichText::new(label)
                        .size(theme::scaled_size(14.0, text_scale))
                        .strong()
                        .color(colors.text),
                );
                ui.add(
                    egui::Label::new(
                        RichText::new(detail)
                            .size(theme::scaled_size(12.0, text_scale))
                            .color(colors.text_muted),
                    )
                    .wrap(),
                );
            })
            .response
            .interact(egui::Sense::click())
    });

    response.inner.clicked()
}

fn help_command_row(ui: &mut Ui, command: &commands::CommandSpec, text_scale: f32) {
    let colors = theme::palette();
    Frame::none()
        .fill(colors.panel_alt)
        .stroke(Stroke::new(1.0, colors.border))
        .rounding(Rounding::same(8.0))
        .inner_margin(Margin::same(12.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(command.usage)
                        .monospace()
                        .strong()
                        .color(colors.accent),
                );
                ui.label(RichText::new(command.hint).strong().color(colors.text));
            });
            ui.add(
                egui::Label::new(
                    RichText::new(command.detail)
                        .size(theme::scaled_size(12.0, text_scale))
                        .color(colors.text_muted),
                )
                .wrap(),
            );
        });
}
