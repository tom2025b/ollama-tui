use egui::{Frame, Key, Label, Margin, RichText, Rounding, ScrollArea, Stroke, TextEdit};

use crate::app::App;
use crate::theme;

impl App {
    pub(super) fn render_composer(&mut self, ctx: &egui::Context) {
        let colors = theme::palette();
        egui::TopBottomPanel::bottom("composer_panel")
            .frame(Frame::none().fill(colors.panel).inner_margin(Margin {
                left: 22.0,
                right: 22.0,
                top: 14.0,
                bottom: 16.0,
            }))
            .show(ctx, |ui| {
                self.render_command_suggestions(ui);
                ui.horizontal(|ui| {
                    let input_id = Self::input_id();
                    let suggestions_visible = !self.command_suggestions().is_empty();
                    let enter = self.consume_enter(ctx, input_id);
                    let tab = suggestions_visible
                        && ctx.memory(|memory| memory.has_focus(input_id))
                        && ctx
                            .input_mut(|input| input.consume_key(egui::Modifiers::NONE, Key::Tab));
                    let down = suggestions_visible
                        && ctx.memory(|memory| memory.has_focus(input_id))
                        && ctx.input_mut(|input| {
                            input.consume_key(egui::Modifiers::NONE, Key::ArrowDown)
                        });
                    let up = suggestions_visible
                        && ctx.memory(|memory| memory.has_focus(input_id))
                        && ctx.input_mut(|input| {
                            input.consume_key(egui::Modifiers::NONE, Key::ArrowUp)
                        });
                    let esc = suggestions_visible
                        && ctx.memory(|memory| memory.has_focus(input_id))
                        && ctx.input_mut(|input| {
                            input.consume_key(egui::Modifiers::NONE, Key::Escape)
                        });
                    let focused = ctx.memory(|memory| memory.has_focus(input_id));

                    if down {
                        self.select_next_suggestion();
                    }
                    if up {
                        self.select_previous_suggestion();
                    }
                    if esc {
                        self.dismiss_command_suggestions();
                    }
                    if tab {
                        let suggestions = self.command_suggestions();
                        if let Some(command) = suggestions.get(self.suggestion_index()) {
                            self.complete_command_suggestion(command);
                        }
                    }
                    if enter && suggestions_visible {
                        self.handle_suggestion_enter(ctx);
                    }

                    Frame::none()
                        .fill(colors.panel_alt)
                        .stroke(Stroke::new(
                            1.0,
                            if focused {
                                colors.accent_strong
                            } else {
                                colors.border
                            },
                        ))
                        .rounding(Rounding::same(8.0))
                        .inner_margin(Margin {
                            left: 14.0,
                            right: 14.0,
                            top: 11.0,
                            bottom: 11.0,
                        })
                        .show(ui, |ui| {
                            ui.set_min_width((ui.available_width() - 106.0).max(260.0));
                            let response = ui.add(
                                TextEdit::multiline(&mut self.input)
                                    .id(input_id)
                                    .hint_text("Message or slash command")
                                    .desired_rows(2)
                                    .desired_width(f32::INFINITY)
                                    .frame(false)
                                    .text_color(colors.text),
                            );
                            if response.changed() {
                                self.on_input_changed();
                                self.request_input_focus(ctx);
                            }
                        });

                    let can_send = !self.streaming && !self.input.trim().is_empty();
                    let label = if self.streaming { "Streaming" } else { "Send" };
                    let send = egui::Button::new(RichText::new(label).strong())
                        .min_size(egui::vec2(92.0, 54.0))
                        .fill(if can_send {
                            colors.accent_strong
                        } else {
                            colors.panel_soft
                        })
                        .stroke(Stroke::new(
                            1.0,
                            if can_send {
                                colors.accent
                            } else {
                                colors.border_strong
                            },
                        ))
                        .rounding(Rounding::same(8.0));

                    if ui.add_enabled(can_send, send).clicked()
                        || (enter && can_send && !suggestions_visible)
                    {
                        self.send_current_input(ctx);
                        self.request_input_focus(ctx);
                    }
                });
            });
    }

    fn render_command_suggestions(&mut self, ui: &mut egui::Ui) {
        let suggestions = self.command_suggestions();
        if suggestions.is_empty() {
            return;
        }

        let colors = theme::palette();
        Frame::none()
            .fill(colors.panel_alt)
            .stroke(Stroke::new(1.0, colors.border))
            .rounding(Rounding::same(8.0))
            .inner_margin(Margin::same(10.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Commands")
                            .small()
                            .strong()
                            .color(colors.text_muted),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            RichText::new("Tab/Enter complete  Up/Down select")
                                .small()
                                .color(colors.text_subtle),
                        );
                    });
                });
                ui.add_space(6.0);
                ScrollArea::vertical()
                    .id_source("command_palette_scroll")
                    .max_height(260.0 * self.text_scale)
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        for (index, command) in suggestions.iter().enumerate() {
                            let selected = index == self.suggestion_index();
                            let fill = if selected {
                                colors.accent_soft
                            } else {
                                colors.panel
                            };
                            let stroke = if selected {
                                Stroke::new(1.0, colors.accent_strong)
                            } else {
                                Stroke::new(1.0, colors.border)
                            };
                            let response = Frame::none()
                                .fill(fill)
                                .stroke(stroke)
                                .rounding(Rounding::same(8.0))
                                .inner_margin(Margin::same(10.0))
                                .show(ui, |ui| {
                                    ui.set_width(ui.available_width());
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            RichText::new(command.name)
                                                .monospace()
                                                .strong()
                                                .color(colors.accent),
                                        );
                                        ui.label(
                                            RichText::new(command.hint).strong().color(colors.text),
                                        );
                                    });
                                    ui.add(
                                        Label::new(
                                            RichText::new(command.detail)
                                                .small()
                                                .color(colors.text_subtle),
                                        )
                                        .wrap(),
                                    );
                                })
                                .response
                                .interact(egui::Sense::click())
                                .on_hover_cursor(egui::CursorIcon::PointingHand);

                            if response.clicked() {
                                self.command_suggestion_index = index;
                                self.complete_command_suggestion(command);
                                self.request_input_focus(ui.ctx());
                            }
                            ui.add_space(5.0);
                        }
                    });
            });
        ui.add_space(8.0);
    }
}
