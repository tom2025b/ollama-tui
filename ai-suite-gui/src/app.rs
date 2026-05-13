mod actions;

use std::time::Duration;

use ai_suite::{ModelInfo, available_models};
use egui::{Key, Modifiers};
use tokio::{runtime::Handle, sync::mpsc};

use crate::backend::BackendEvent;
use crate::commands;
use crate::message::{ChatMessage, conversation_context};
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
