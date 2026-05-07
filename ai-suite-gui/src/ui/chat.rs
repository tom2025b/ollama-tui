use egui::{
    Align, Color32, Frame, Label, Layout, Margin, RichText, Rounding, ScrollArea, Stroke, Ui,
};

use crate::app::App;
use crate::message::{ChatMessage, Role};
use crate::theme;

use super::status_pill;

impl App {
    pub(super) fn render_chat(&self, ctx: &egui::Context) {
        let colors = theme::palette();
        egui::CentralPanel::default()
            .frame(Frame::none().fill(colors.chat_bg).inner_margin(Margin {
                left: 24.0,
                right: 24.0,
                top: 20.0,
                bottom: 20.0,
            }))
            .show(ctx, |ui| {
                if self.messages.is_empty() {
                    self.render_empty_state(ui);
                    return;
                }

                ScrollArea::vertical()
                    .id_source("messages_scroll")
                    .auto_shrink([false, false])
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        ui.add_space(4.0);
                        for message in &self.messages {
                            self.render_message_row(ui, ctx, message);
                            ui.add_space(14.0);
                        }
                        ui.add_space(8.0);
                    });
            });
    }

    fn render_empty_state(&self, ui: &mut Ui) {
        let colors = theme::palette();
        ui.with_layout(Layout::top_down(Align::Center), |ui| {
            ui.add_space(ui.available_height() * 0.30);
            Frame::none()
                .fill(colors.panel)
                .stroke(Stroke::new(1.0, colors.border))
                .rounding(Rounding::same(8.0))
                .inner_margin(Margin {
                    left: 28.0,
                    right: 28.0,
                    top: 24.0,
                    bottom: 24.0,
                })
                .show(ui, |ui| {
                    ui.with_layout(Layout::top_down(Align::Center), |ui| {
                        ui.label(
                            RichText::new("New conversation")
                                .size(self.text_size(27.0))
                                .strong()
                                .color(colors.text),
                        );
                        ui.add_space(8.0);
                        status_pill(ui, "Model", &self.selected_model_label(), colors.accent);
                    });
                });
        });
    }

    fn render_message_row(&self, ui: &mut Ui, ctx: &egui::Context, message: &ChatMessage) {
        match message.role {
            Role::User => {
                ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
                    let max_width = ui.available_width() * 0.66;
                    message_bubble(ui, ctx, message, max_width, self.text_scale);
                });
            }
            Role::Assistant => {
                ui.with_layout(Layout::left_to_right(Align::TOP), |ui| {
                    let max_width = ui.available_width() * 0.82;
                    message_bubble(ui, ctx, message, max_width, self.text_scale);
                });
            }
            Role::Local => {
                ui.with_layout(Layout::top_down(Align::Center), |ui| {
                    let max_width = ui.available_width() * 0.76;
                    message_bubble(ui, ctx, message, max_width, self.text_scale);
                });
            }
        };
    }
}

fn message_bubble(
    ui: &mut Ui,
    ctx: &egui::Context,
    message: &ChatMessage,
    max_width: f32,
    text_scale: f32,
) {
    let colors = theme::palette();
    let (fill, stroke) = bubble_colors(message, &colors);
    let accent = bubble_accent(message, &colors);
    let text = message_text(ctx, message);

    let response = Frame::none()
        .fill(fill)
        .stroke(stroke)
        .rounding(Rounding::same(8.0))
        .inner_margin(Margin {
            left: 15.0,
            right: 15.0,
            top: 12.0,
            bottom: 13.0,
        })
        .show(ui, |ui| {
            ui.set_max_width(max_width.clamp(260.0, 820.0));
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(message.role.label())
                        .small()
                        .strong()
                        .color(accent),
                );
                if let Some(model) = &message.model_label {
                    ui.label(RichText::new(model).small().color(colors.text_subtle));
                }
            });
            ui.add_space(6.0);
            ui.add(
                Label::new(
                    RichText::new(text)
                        .size(theme::scaled_size(14.5, text_scale))
                        .color(colors.text),
                )
                .wrap()
                .selectable(true),
            );
        })
        .response;

    let rect = response.rect;
    let x = if message.role == Role::User {
        rect.right()
    } else {
        rect.left()
    };
    ui.painter().line_segment(
        [
            egui::pos2(x, rect.top() + 9.0),
            egui::pos2(x, rect.bottom() - 9.0),
        ],
        Stroke::new(2.0, accent),
    );
}

fn bubble_colors(message: &ChatMessage, colors: &theme::Palette) -> (Color32, Stroke) {
    if message.is_error {
        return (
            colors.error_bubble,
            Stroke::new(1.0, Color32::from_rgb(151, 69, 78)),
        );
    }

    match message.role {
        Role::User => (
            colors.user_bubble,
            Stroke::new(1.0, Color32::from_rgb(74, 132, 151)),
        ),
        Role::Assistant => (colors.assistant_bubble, Stroke::new(1.0, colors.border)),
        Role::Local => (colors.local_bubble, Stroke::new(1.0, colors.border_strong)),
    }
}

fn bubble_accent(message: &ChatMessage, colors: &theme::Palette) -> Color32 {
    if message.is_error {
        return colors.warning;
    }

    match message.role {
        Role::User => colors.accent,
        Role::Assistant => colors.success,
        Role::Local => colors.accent_warm,
    }
}

fn message_text(ctx: &egui::Context, message: &ChatMessage) -> String {
    if message.complete {
        return message.content.clone();
    }

    if message.content.is_empty() {
        let dots = ((ctx.input(|input| input.time) * 4.0) as usize % 4).min(3);
        format!("Thinking{}", ".".repeat(dots))
    } else if ((ctx.input(|input| input.time) * 2.0) as usize).is_multiple_of(2) {
        format!("{} |", message.content)
    } else {
        message.content.clone()
    }
}
