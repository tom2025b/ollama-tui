use ai_suite::ConversationTurn;
use egui::{Color32, Frame, Key, Layout, Margin, Modifiers, RichText, Rounding, ScrollArea};
use tokio::{runtime::Handle, sync::mpsc};

use crate::backend::{BackendEvent, spawn_request};

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

// Derives PartialEq so we can compare roles in conversation_context().
#[derive(PartialEq)]
enum Role {
    User,
    Assistant,
}

struct ChatMessage {
    role: Role,
    content: String,
    /// False while the assistant is still streaming this message.
    complete: bool,
    /// True if this message contains an error from the backend.
    is_error: bool,
}

// ---------------------------------------------------------------------------
// App state
// ---------------------------------------------------------------------------

pub struct App {
    messages: Vec<ChatMessage>,
    input: String,
    /// Shown in the top-right bar. "Ready" until the first response arrives.
    current_model: String,
    rx: Option<mpsc::UnboundedReceiver<BackendEvent>>,
    streaming: bool,
    handle: Handle,
}

impl App {
    pub fn new(handle: Handle) -> Self {
        Self {
            messages: Vec::new(),
            input: String::new(),
            current_model: "Ready".to_string(),
            rx: None,
            streaming: false,
            handle,
        }
    }

    // -----------------------------------------------------------------------
    // Logic
    // -----------------------------------------------------------------------

    /// Build context from all completed user/assistant pairs so far.
    fn conversation_context(&self) -> Vec<ConversationTurn> {
        let complete: Vec<&ChatMessage> = self
            .messages
            .iter()
            .filter(|m| m.complete && !m.is_error)
            .collect();

        // Pair consecutive user+assistant messages into ConversationTurn.
        complete
            .chunks(2)
            .filter_map(|pair| {
                if pair.len() == 2
                    && pair[0].role == Role::User
                    && pair[1].role == Role::Assistant
                {
                    Some(ConversationTurn {
                        user: pair[0].content.clone(),
                        assistant: pair[1].content.clone(),
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    /// Validate, build context, push placeholder messages, fire backend task.
    fn send_message(&mut self, ctx: &egui::Context) {
        let prompt = self.input.trim().to_string();
        if prompt.is_empty() || self.streaming {
            return;
        }

        // Snapshot context BEFORE adding the new user message so the current
        // prompt isn't included as prior context.
        let context = self.conversation_context();

        self.messages.push(ChatMessage {
            role: Role::User,
            content: prompt.clone(),
            complete: true,
            is_error: false,
        });

        // Empty placeholder — streaming tokens will fill this in.
        self.messages.push(ChatMessage {
            role: Role::Assistant,
            content: String::new(),
            complete: false,
            is_error: false,
        });

        let (tx, rx) = mpsc::unbounded_channel();
        self.rx = Some(rx);
        self.streaming = true;
        self.input.clear();

        spawn_request(prompt, context, tx, ctx.clone(), self.handle.clone());
    }

    /// Drain all pending backend events into the message list.
    fn drain_channel(&mut self) {
        loop {
            // Borrow self.rx briefly to pull one event out by value, then
            // drop the borrow so match arms can freely mutate other fields
            // (including setting self.rx = None on Done/Error).
            let event = match self.rx.as_mut() {
                Some(rx) => match rx.try_recv() {
                    Ok(event) => event,
                    Err(_) => break,
                },
                None => break,
            };

            match event {
                BackendEvent::Token(token) => {
                    if let Some(msg) = self.messages.last_mut() {
                        msg.content.push_str(&token);
                    }
                }
                BackendEvent::Done { full_text, model_name } => {
                    if let Some(msg) = self.messages.last_mut() {
                        msg.content = full_text;
                        msg.complete = true;
                    }
                    self.current_model = model_name;
                    self.streaming = false;
                    self.rx = None;
                    break;
                }
                BackendEvent::Error(e) => {
                    if let Some(msg) = self.messages.last_mut() {
                        msg.content = format!("Error: {e}");
                        msg.complete = true;
                        msg.is_error = true;
                    }
                    self.streaming = false;
                    self.rx = None;
                    break;
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // Rendering
    // -----------------------------------------------------------------------

    fn render_top_bar(&self, ctx: &egui::Context) {
        let text_col = Color32::from_rgb(0xcd, 0xd6, 0xf4);
        let accent = Color32::from_rgb(0x89, 0xb4, 0xfa);

        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.add_space(4.0);
                ui.label(RichText::new("ai-suite").strong().color(text_col).size(15.0));
                ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(4.0);
                    ui.label(RichText::new(&self.current_model).color(accent).size(13.0));
                });
            });
            ui.add_space(2.0);
        });
    }

    fn render_input_bar(&mut self, ctx: &egui::Context) {
        let text_col = Color32::from_rgb(0xcd, 0xd6, 0xf4);
        let input_bg = Color32::from_rgb(0x18, 0x18, 0x25);
        let btn_col = Color32::from_rgb(0x89, 0xb4, 0xfa);
        let input_id = egui::Id::new("main_input");

        egui::TopBottomPanel::bottom("input_panel").show(ctx, |ui| {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                // Consume Enter (without Shift) before TextEdit sees it so it
                // triggers a send rather than inserting a newline character.
                let had_focus = ctx.memory(|m| m.has_focus(input_id));
                let enter_hit = had_focus
                    && ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Enter));

                let text_edit = egui::TextEdit::multiline(&mut self.input)
                    .id(input_id)
                    .hint_text("Type a message…  (Enter to send, Shift+Enter for newline)")
                    .desired_rows(1)
                    .desired_width(ui.available_width() - 52.0)
                    .text_color(text_col)
                    .frame(false);

                Frame::none()
                    .fill(input_bg)
                    .rounding(Rounding::same(6.0))
                    .inner_margin(Margin { left: 10.0, right: 10.0, top: 6.0, bottom: 6.0 })
                    .show(ui, |ui| {
                        ui.add(text_edit);
                    });

                let send_btn = ui.add_enabled(
                    !self.streaming,
                    egui::Button::new(RichText::new("▶").color(btn_col))
                        .min_size([40.0, 32.0].into()),
                );

                if (enter_hit || send_btn.clicked()) && !self.streaming {
                    self.send_message(ctx);
                    ctx.memory_mut(|m| m.request_focus(input_id));
                }
            });
            ui.add_space(8.0);
        });
    }

    fn render_messages(&self, ctx: &egui::Context) {
        let bg = Color32::from_rgb(0x1e, 0x1e, 0x2e);
        let text_col = Color32::from_rgb(0xcd, 0xd6, 0xf4);
        let user_bg = Color32::from_rgb(0x45, 0x47, 0x8a);
        let asst_bg = Color32::from_rgb(0x31, 0x32, 0x44);
        let err_bg = Color32::from_rgb(0x7a, 0x25, 0x25);

        egui::CentralPanel::default()
            .frame(Frame::none().fill(bg))
            .show(ctx, |ui| {
                ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        ui.add_space(10.0);

                        for msg in &self.messages {
                            let bubble_bg = if msg.is_error {
                                err_bg
                            } else if msg.role == Role::User {
                                user_bg
                            } else {
                                asst_bg
                            };

                            // Append blinking cursor while the response streams in.
                            let text = if !msg.complete {
                                format!("{}▌", msg.content)
                            } else {
                                msg.content.clone()
                            };

                            let bubble = Frame::none()
                                .fill(bubble_bg)
                                .rounding(Rounding::same(8.0))
                                .inner_margin(Margin {
                                    left: 12.0,
                                    right: 12.0,
                                    top: 8.0,
                                    bottom: 8.0,
                                });

                            if msg.role == Role::User {
                                ui.with_layout(
                                    Layout::right_to_left(egui::Align::TOP),
                                    |ui| {
                                        bubble.show(ui, |ui| {
                                            ui.set_max_width(ui.available_width() * 0.72);
                                            ui.label(
                                                RichText::new(&text).color(text_col).size(14.0),
                                            );
                                        });
                                    },
                                );
                            } else {
                                bubble.show(ui, |ui| {
                                    ui.set_max_width(ui.available_width() * 0.88);
                                    ui.label(RichText::new(&text).color(text_col).size(14.0));
                                });
                            }

                            ui.add_space(6.0);
                        }

                        ui.add_space(4.0);
                    });
            });
    }
}

// ---------------------------------------------------------------------------
// eframe integration
// ---------------------------------------------------------------------------

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply backend events before rendering so new tokens are visible this frame.
        self.drain_channel();

        // Keep repainting while streaming so the cursor blinks and tokens appear.
        if self.streaming {
            ctx.request_repaint();
        }

        ctx.set_visuals(egui::Visuals::dark());

        self.render_top_bar(ctx);
        self.render_input_bar(ctx);
        self.render_messages(ctx);
    }
}
