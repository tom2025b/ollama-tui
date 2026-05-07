use std::time::Duration;

use ai_suite::{ModelInfo, available_models, route_prompt};
use egui::{Key, Modifiers};
use tokio::{runtime::Handle, sync::mpsc};

use crate::backend::{BackendEvent, spawn_request};
use crate::commands::{self, ParsedCommand};
use crate::message::{ChatMessage, Role, conversation_context};
use crate::settings::GuiPreferences;

pub struct App {
    pub(crate) messages: Vec<ChatMessage>,
    pub(crate) input: String,
    pub(crate) models: Vec<ModelInfo>,
    pub(crate) selected_model_id: Option<String>,
    pub(crate) last_model_label: Option<String>,
    pub(crate) status: String,
    pub(crate) show_help: bool,
    pub(crate) show_model_picker: bool,
    pub(crate) debug_errors: bool,
    pub(crate) text_scale: f32,
    pub(crate) command_suggestion_index: usize,
    command_suggestions_dismissed_for: Option<String>,
    pub(crate) streaming: bool,
    pub(crate) rx: Option<mpsc::UnboundedReceiver<BackendEvent>>,
    handle: Handle,
}

impl App {
    pub fn new(handle: Handle) -> Self {
        let preferences = GuiPreferences::load();
        Self {
            messages: Vec::new(),
            input: String::new(),
            models: available_models(),
            selected_model_id: None,
            last_model_label: None,
            status: "Ready".to_string(),
            show_help: false,
            show_model_picker: false,
            debug_errors: false,
            text_scale: preferences.text_scale,
            command_suggestion_index: 0,
            command_suggestions_dismissed_for: None,
            streaming: false,
            rx: None,
            handle,
        }
    }

    pub(crate) fn selected_model_label(&self) -> String {
        self.selected_model()
            .map(|model| model.label.clone())
            .unwrap_or_else(|| "Auto Router".to_string())
    }

    pub(crate) fn selected_model(&self) -> Option<&ModelInfo> {
        let selected = self.selected_model_id.as_ref()?;
        self.models.iter().find(|model| &model.id == selected)
    }

    pub(crate) fn send_current_input(&mut self, ctx: &egui::Context) {
        let prompt = self.input.trim().to_string();
        if prompt.is_empty() || self.streaming {
            return;
        }

        if prompt == "?" {
            self.input.clear();
            self.open_help(ctx);
            return;
        }

        if let Some(command) = commands::parse_slash_command(&prompt) {
            self.input.clear();
            self.execute_command(command, ctx);
            return;
        }

        self.input.clear();
        self.send_prompt(prompt, ctx);
    }

    pub(crate) fn consume_enter(&self, ctx: &egui::Context, input_id: egui::Id) -> bool {
        let had_focus = ctx.memory(|memory| memory.has_focus(input_id));
        had_focus && ctx.input_mut(|input| input.consume_key(Modifiers::NONE, Key::Enter))
    }

    pub(crate) fn command_suggestions(&self) -> Vec<&'static commands::CommandSpec> {
        if self
            .command_suggestions_dismissed_for
            .as_deref()
            .map(|dismissed| dismissed == self.input)
            .unwrap_or(false)
        {
            return Vec::new();
        }
        commands::suggestions(&self.input)
    }

    pub(crate) fn input_id() -> egui::Id {
        egui::Id::new("main_input")
    }

    pub(crate) fn request_input_focus(&self, ctx: &egui::Context) {
        ctx.memory_mut(|memory| memory.request_focus(Self::input_id()));
    }

    pub(crate) fn on_input_changed(&mut self) {
        self.command_suggestion_index = 0;
        if self.command_suggestions_dismissed_for.as_deref() != Some(self.input.as_str()) {
            self.command_suggestions_dismissed_for = None;
        }
        if self.input.trim() == "?" {
            self.input.clear();
            self.show_help = true;
            self.status = "Help opened".to_string();
        }
    }

    pub(crate) fn suggestion_index(&self) -> usize {
        let suggestions = self.command_suggestions();
        if suggestions.is_empty() {
            0
        } else {
            self.command_suggestion_index.min(suggestions.len() - 1)
        }
    }

    pub(crate) fn select_next_suggestion(&mut self) {
        let total = self.command_suggestions().len();
        if total == 0 {
            self.command_suggestion_index = 0;
        } else {
            self.command_suggestion_index = (self.suggestion_index() + 1) % total;
        }
    }

    pub(crate) fn select_previous_suggestion(&mut self) {
        let total = self.command_suggestions().len();
        if total == 0 {
            self.command_suggestion_index = 0;
        } else {
            self.command_suggestion_index = if self.suggestion_index() == 0 {
                total - 1
            } else {
                self.suggestion_index() - 1
            };
        }
    }

    pub(crate) fn dismiss_command_suggestions(&mut self) {
        if !self.command_suggestions().is_empty() {
            self.command_suggestions_dismissed_for = Some(self.input.clone());
        }
        self.command_suggestion_index = 0;
    }

    pub(crate) fn complete_command_suggestion(&mut self, command: &commands::CommandSpec) {
        self.input = format!("{} ", command.name);
        self.command_suggestion_index = 0;
        self.command_suggestions_dismissed_for = None;
    }

    pub(crate) fn handle_suggestion_enter(&mut self, ctx: &egui::Context) -> bool {
        let suggestions = self.command_suggestions();
        let Some(command) = suggestions.get(self.suggestion_index()).copied() else {
            return false;
        };

        let exact = self.input.trim().eq_ignore_ascii_case(command.name);
        if exact && !command.needs_argument() {
            let raw = command.name.to_string();
            self.input.clear();
            if let Some(parsed) = commands::parse_slash_command(&raw) {
                self.execute_command(parsed, ctx);
            }
        } else {
            self.complete_command_suggestion(command);
        }
        true
    }

    pub(crate) fn append_local_message(&mut self, command: &str, body: String) {
        self.messages
            .push(ChatMessage::local(format!("{command}\n\n{body}")));
    }

    pub(crate) fn completed_turn_count(&self) -> usize {
        conversation_context(&self.messages).len()
    }

    fn send_prompt(&mut self, prompt: String, ctx: &egui::Context) {
        let context = conversation_context(&self.messages);
        let model_id = self.selected_model_id.clone();
        let model_label = self.selected_model_label();

        self.messages.push(ChatMessage::user(prompt.clone()));
        self.messages.push(ChatMessage::assistant(model_label));

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

    fn execute_command(&mut self, command: ParsedCommand, ctx: &egui::Context) {
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
                self.debug_errors = !self.debug_errors;
                let state = if self.debug_errors { "on" } else { "off" };
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

    fn open_help(&mut self, ctx: &egui::Context) {
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

    fn drain_channel(&mut self) {
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

    fn error_text(&self, error: &str) -> String {
        if self.debug_errors {
            format!("Error: {error}")
        } else {
            format!("The backend could not complete the request.\n\n{error}")
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        crate::theme::apply(ctx, self.text_scale);
        self.drain_channel();

        if self.streaming {
            ctx.request_repaint_after(Duration::from_millis(100));
        }

        self.render_ui(ctx);
    }
}
