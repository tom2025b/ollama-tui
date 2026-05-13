use ai_suite::{debug_mode_enabled, friendly_error, route_prompt, toggle_debug_mode};
use tokio::sync::mpsc;

use crate::app::App;
use crate::backend::{BackendEvent, spawn_request};
use crate::commands::{self, ParsedCommand};
use crate::message::{Role, conversation_context};

impl App {
    pub(crate) fn send_prompt(&mut self, prompt: String, ctx: &egui::Context) {
        let context = conversation_context(&self.messages);
        let model_id = self.selected_model_id.clone();
        let model_label = self.selected_model_label();

        self.messages
            .push(crate::message::ChatMessage::user(prompt.clone()));
        self.messages
            .push(crate::message::ChatMessage::assistant(model_label));

        let (tx, rx) = mpsc::unbounded_channel();
        self.rx = Some(rx);
        self.streaming = true;
        self.status = "Streaming response".to_string();

        spawn_request(
            prompt,
            context,
            model_id,
            tx,
            ctx.clone(),
            self.handle.clone(),
        );
    }

    pub(crate) fn execute_command(&mut self, command: ParsedCommand, ctx: &egui::Context) {
        match command.name() {
            "/clear" => {
                self.messages.clear();
                self.status = "Conversation cleared".to_string();
            }
            "/help" => {
                self.open_help(ctx);
            }
            "/model" => self.execute_model_command(&command),
            "/models" | "/backend" | "/backends" => {
                self.append_local_message(command.raw(), self.format_models());
                self.status = "Backend readiness listed".to_string();
            }
            "/route" => self.execute_route_command(&command),
            "/summary" => {
                self.append_local_message(command.raw(), self.format_summary());
                self.status = "Session summary generated".to_string();
            }
            "/debug" => {
                let state = if toggle_debug_mode() { "on" } else { "off" };
                self.append_local_message(
                    command.raw(),
                    format!("Verbose backend errors: {state}"),
                );
                self.status = format!("Debug errors {state}");
            }
            "/quit" | "/exit" | "/q" => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            name => {
                let hint = commands::find_command(name)
                    .map(|command| command.usage.to_string())
                    .unwrap_or_else(|| "/help".to_string());
                self.append_local_message(
                    command.raw(),
                    format!("Unknown command `{name}`.\n\nTry `{hint}`."),
                );
                self.status = "Unknown command".to_string();
            }
        }
    }

    pub(crate) fn open_help(&mut self, ctx: &egui::Context) {
        self.show_help = true;
        self.status = "Help opened".to_string();
        self.request_input_focus(ctx);
    }

    fn execute_model_command(&mut self, command: &ParsedCommand) {
        let body = commands::command_body(command);
        if body.trim().is_empty() {
            self.show_model_picker = true;
            self.status = "Model picker opened".to_string();
            return;
        }

        match self.select_model_by_query(&commands::unquote(&body)) {
            Ok(label) => {
                self.append_local_message(command.raw(), format!("Routing mode: {label}"));
                self.status = format!("Selected {label}");
            }
            Err(message) => {
                self.append_local_message(command.raw(), message);
                self.status = "Model selection failed".to_string();
            }
        }
    }

    fn execute_route_command(&mut self, command: &ParsedCommand) {
        let mut args = command.args();
        if args
            .first()
            .map(|first| first.eq_ignore_ascii_case("test"))
            .unwrap_or(false)
        {
            args = &args[1..];
        }

        let prompt = commands::unquote(&args.join(" "));
        if prompt.is_empty() {
            self.append_local_message(command.raw(), "Usage: /route <prompt>".to_string());
            self.status = "Route command needs a prompt".to_string();
            return;
        }

        self.append_local_message(command.raw(), route_prompt(&prompt));
        self.status = "Router trace generated".to_string();
    }

    fn select_model_by_query(&mut self, query: &str) -> Result<String, String> {
        let trimmed = query.trim();
        if trimmed.eq_ignore_ascii_case("auto") {
            self.selected_model_id = None;
            return Ok("Auto Router".to_string());
        }

        let needle = trimmed.to_ascii_lowercase();
        let matches = self
            .models
            .iter()
            .filter(|model| {
                model.enabled
                    && (model.id.to_ascii_lowercase().contains(&needle)
                        || model.name.to_ascii_lowercase().contains(&needle)
                        || model.label.to_ascii_lowercase().contains(&needle))
            })
            .cloned()
            .collect::<Vec<_>>();

        match matches.as_slice() {
            [] => Err(format!("No enabled model matches `{trimmed}`.")),
            [model] => {
                self.selected_model_id = Some(model.id.clone());
                Ok(model.label.clone())
            }
            _ => {
                let labels = matches
                    .iter()
                    .map(|model| format!("- {}", model.label))
                    .collect::<Vec<_>>()
                    .join("\n");
                Err(format!(
                    "Multiple enabled models match `{trimmed}`:\n{labels}"
                ))
            }
        }
    }

    fn format_models(&self) -> String {
        let rows = self
            .models
            .iter()
            .map(|model| {
                let state = if model.enabled {
                    "ready"
                } else {
                    "unavailable"
                };
                let detail = if model.enabled {
                    model.strengths.join(", ")
                } else {
                    model
                        .disabled_reason
                        .clone()
                        .unwrap_or_else(|| "disabled".to_string())
                };
                format!("- [{state}] {}\n  {detail}", model.label)
            })
            .collect::<Vec<_>>()
            .join("\n");

        format!("Routing mode: {}\n\n{rows}", self.selected_model_label())
    }

    fn format_summary(&self) -> String {
        let user_messages = self
            .messages
            .iter()
            .filter(|message| message.role == Role::User)
            .count();
        let assistant_messages = self
            .messages
            .iter()
            .filter(|message| message.role == Role::Assistant && message.complete)
            .count();
        let last_model = self.last_model_label.as_deref().unwrap_or("none yet");

        format!(
            "Messages: {user_messages} user, {assistant_messages} assistant\n\
             Completed context turns: {}\n\
             Routing mode: {}\n\
             Last model: {last_model}",
            self.completed_turn_count(),
            self.selected_model_label(),
        )
    }

    pub(crate) fn drain_channel(&mut self) {
        while let Some(event) = self.try_recv_backend_event() {
            match event {
                BackendEvent::Token(token) => {
                    if let Some(message) = self.messages.last_mut() {
                        message.content.push_str(&token);
                    }
                }
                BackendEvent::Done {
                    full_text,
                    model_name,
                } => {
                    let model_label = self.model_display_label(&model_name);
                    if let Some(message) = self.messages.last_mut() {
                        message.content = full_text;
                        message.complete = true;
                        message.model_label = Some(model_label.clone());
                    }
                    self.last_model_label = Some(model_label.clone());
                    self.status = format!("Completed with {model_label}");
                    self.streaming = false;
                    self.rx = None;
                    break;
                }
                BackendEvent::Error(error) => {
                    let error_text = self.error_text(&error);
                    if let Some(message) = self.messages.last_mut() {
                        message.content = error_text;
                        message.complete = true;
                        message.is_error = true;
                    }
                    self.status = "Backend error".to_string();
                    self.streaming = false;
                    self.rx = None;
                    break;
                }
            }
        }
    }

    fn try_recv_backend_event(&mut self) -> Option<BackendEvent> {
        self.rx.as_mut()?.try_recv().ok()
    }

    fn model_display_label(&self, model_name: &str) -> String {
        self.models
            .iter()
            .find(|model| model.name == model_name)
            .map(|model| model.label.clone())
            .unwrap_or_else(|| model_name.to_string())
    }

    fn error_text(&self, error: &ai_suite::Error) -> String {
        let rendered = friendly_error(error);
        if debug_mode_enabled() {
            format!("Error: {rendered}")
        } else {
            format!("The backend could not complete the request.\n\n{rendered}")
        }
    }
}
