use crate::llm::{ConversationTurn, LanguageModel, Provider, RouteDecision};
use crate::router::{ModelRouter, Router};

/// Maximum completed exchanges kept in memory and shown in the TUI.
const MAX_STORED_TURNS: usize = 12;

/// Maximum completed exchanges sent back to the next model request.
const MAX_CONTEXT_TURNS: usize = 6;

/// Lightweight ASCII spinner used while the model is active.
const SPINNER_FRAMES: &[&str] = &["-", "\\", "|", "/"];

/// One prompt-and-answer exchange shown in the TUI history.
///
/// The route explanation is stored with the prompt so you can later see what
/// the router decided for each individual request.
#[derive(Clone, Debug)]
pub struct ChatMessage {
    /// Text typed by the user.
    pub prompt: String,

    /// Model selected by the router.
    pub model_name: String,

    /// Human-readable route explanation.
    pub route_reason: String,

    /// The model answer, or an error message if the request failed.
    pub answer: String,

    /// True while tokens are still arriving for this exchange.
    pub in_progress: bool,

    /// True when the backend failed before completing the response.
    pub failed: bool,

    /// True when this exchange should be sent as future model context.
    pub include_in_context: bool,
}

/// Work item created when the user submits a prompt.
///
/// The TUI loop owns terminal input and drawing. The async task owns the Ollama
/// HTTP call. This struct is the clean handoff between those two parts.
#[derive(Clone, Debug)]
pub struct PendingRequest {
    /// The prompt to send.
    pub prompt: String,

    /// The router decision for this prompt.
    pub route: RouteDecision,

    /// Bounded previous conversation included with this request.
    pub context: Vec<ConversationTurn>,
}

/// Incremental result sent back from an async model task to the TUI loop.
#[derive(Clone, Debug)]
pub enum ModelEvent {
    /// One streamed piece of assistant text.
    Token(String),

    /// The current response completed successfully.
    Finished,

    /// The current response failed.
    Failed(String),
}

/// All state needed to draw and update the application.
///
/// Keeping app state in one struct makes the TUI loop in `main.rs` simple: read
/// input, update this struct, draw this struct, repeat.
pub struct App {
    /// The router that chooses a model for each prompt.
    router: ModelRouter,

    /// Text currently being typed at the bottom of the screen.
    pub input: String,

    /// Previous completed requests.
    pub history: Vec<ChatMessage>,

    /// True while a request is running.
    ///
    /// The first version sends one request at a time. That makes the UI easier
    /// to understand and prevents overlapping local model jobs.
    pub waiting_for_model: bool,

    /// Set to true when the user wants to leave the app.
    pub should_quit: bool,

    /// Short status line shown near the input box.
    pub status: String,

    /// Label for the active model, if one is currently streaming.
    active_model_name: Option<String>,

    /// Counter used to animate the waiting status.
    activity_tick: usize,

    /// True when the help overlay should be shown.
    pub show_help: bool,

    /// How many lines the user has scrolled up from the bottom of the chat
    /// history. Zero means pinned to the newest content.
    pub scroll_offset: usize,
}

impl App {
    /// Build a fresh app with the default router.
    pub fn new() -> Self {
        Self {
            router: ModelRouter::new(),
            input: String::new(),
            history: Vec::new(),
            waiting_for_model: false,
            should_quit: false,
            status: "Type a prompt. Press Enter to send. Press Esc or Ctrl-C to quit.".to_string(),
            active_model_name: None,
            activity_tick: 0,
            show_help: false,
            scroll_offset: 0,
        }
    }

    /// Return the list of models that can be displayed by the UI.
    pub fn models(&self) -> &[LanguageModel] {
        self.router.models()
    }

    /// Add one typed character to the input buffer.
    pub fn push_input_char(&mut self, character: char) {
        self.input.push(character);
    }

    /// Remove the most recent typed character.
    pub fn backspace(&mut self) {
        self.input.pop();
    }

    /// Clear the current input without submitting it.
    pub fn clear_input(&mut self) {
        self.input.clear();
        self.status = "Input cleared.".to_string();
    }

    /// Move the chat history view upward by the given number of lines.
    pub fn scroll_up(&mut self, lines: usize) {
        self.scroll_offset = self.scroll_offset.saturating_add(lines);
    }

    /// Move the chat history view downward (toward newest messages).
    /// Clamps at zero, which means "pinned to the bottom."
    pub fn scroll_down(&mut self, lines: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(lines);
    }

    /// Jump to the very top of the chat history.
    /// The UI will clamp this to the actual maximum during rendering.
    pub fn scroll_to_top(&mut self) {
        self.scroll_offset = usize::MAX;
    }

    /// Jump back to the newest messages at the bottom.
    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = 0;
    }

    /// Show or hide the help overlay.
    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
        self.status = if self.show_help {
            "Help is open. Press Esc or ? to close it.".to_string()
        } else {
            "Help closed.".to_string()
        };
    }

    /// Hide the help overlay.
    pub fn hide_help(&mut self) {
        self.show_help = false;
        self.status = "Help closed.".to_string();
    }

    /// Mark the app as ready to close.
    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    /// Try to submit the current prompt.
    ///
    /// If the prompt is empty or a model is already working, nothing is sent.
    /// Otherwise this returns a `PendingRequest` for `main.rs` to run
    /// asynchronously.
    pub fn submit_prompt(&mut self) -> Option<PendingRequest> {
        let prompt = self.input.trim().to_string();

        if prompt.is_empty() {
            self.status = "Write a prompt before pressing Enter.".to_string();
            return None;
        }

        if prompt.starts_with('/') {
            self.input.clear();
            self.handle_command(&prompt);
            return None;
        }

        if self.waiting_for_model {
            self.status = "A model is already answering. Wait for it to finish.".to_string();
            return None;
        }

        let route = self.router.route(&prompt);
        let context = self.conversation_context();
        let model_name = route.model.display_label();
        let route_reason = route.reason.clone();

        self.input.clear();
        self.waiting_for_model = true;
        self.active_model_name = Some(model_name.clone());
        self.activity_tick = 0;
        // Snap to the bottom so the user sees the new exchange arrive.
        self.scroll_offset = 0;
        self.status = format!("Sent to {model_name}. Waiting for first token...");

        self.history.push(ChatMessage {
            prompt: prompt.clone(),
            model_name,
            route_reason,
            answer: String::new(),
            in_progress: true,
            failed: false,
            include_in_context: true,
        });
        self.trim_history();

        Some(PendingRequest {
            prompt,
            route,
            context,
        })
    }

    /// Apply one streamed model event to the visible conversation.
    pub fn handle_model_event(&mut self, event: ModelEvent) {
        match event {
            ModelEvent::Token(token) => {
                if let Some(message) = self.active_message_mut() {
                    message.answer.push_str(&token);
                }
            }
            ModelEvent::Finished => {
                if let Some(message) = self.active_message_mut() {
                    message.in_progress = false;
                }

                let model_name = self
                    .active_model_name
                    .take()
                    .unwrap_or_else(|| "the selected model".to_string());
                self.waiting_for_model = false;
                self.status = format!("Last answer came from {model_name}.");
            }
            ModelEvent::Failed(error) => {
                if let Some(message) = self.active_message_mut() {
                    if !message.answer.trim().is_empty() {
                        message.answer.push_str("\n\n");
                    }
                    message.answer.push_str(&error);
                    message.in_progress = false;
                    message.failed = true;
                }

                self.active_model_name = None;
                self.waiting_for_model = false;
                self.status = "Model request failed.".to_string();
            }
        }
    }

    /// Advance lightweight UI activity while a request is in progress.
    pub fn tick(&mut self) {
        if !self.waiting_for_model {
            return;
        }

        self.activity_tick = self.activity_tick.wrapping_add(1);
        let spinner = SPINNER_FRAMES[self.activity_tick % SPINNER_FRAMES.len()];
        let model_name = self
            .active_model_name
            .as_deref()
            .unwrap_or("the selected model");
        self.status = format!("{spinner} Streaming from {model_name}...");
    }

    /// Return the model currently in use, or the most recent model if idle.
    pub fn current_model_label(&self) -> String {
        if let Some(model_name) = &self.active_model_name {
            return model_name.clone();
        }

        self.history
            .iter()
            .rev()
            .find(|message| message.include_in_context)
            .map(|message| message.model_name.clone())
            .unwrap_or_else(|| "none".to_string())
    }

    /// Return the bounded context to include with the next request.
    fn conversation_context(&self) -> Vec<ConversationTurn> {
        let mut turns = self
            .history
            .iter()
            .rev()
            .filter(|message| {
                message.include_in_context
                    && !message.in_progress
                    && !message.failed
                    && !message.answer.trim().is_empty()
            })
            .take(MAX_CONTEXT_TURNS)
            .map(|message| ConversationTurn {
                user: message.prompt.clone(),
                assistant: message.answer.clone(),
            })
            .collect::<Vec<_>>();

        turns.reverse();
        turns
    }

    /// Keep the visible and retained history bounded.
    fn trim_history(&mut self) {
        let overflow = self.history.len().saturating_sub(MAX_STORED_TURNS);
        if overflow > 0 {
            self.history.drain(0..overflow);
        }
    }

    /// Get the current in-progress message, if there is one.
    fn active_message_mut(&mut self) -> Option<&mut ChatMessage> {
        self.history
            .iter_mut()
            .rev()
            .find(|message| message.in_progress)
    }

    /// Execute a slash command entered in the prompt box.
    fn handle_command(&mut self, input: &str) {
        let command = input.split_whitespace().next().unwrap_or_default();

        match command {
            "/clear" => self.clear_conversation_command(),
            "/models" => {
                let report = self.models_report();
                self.append_local_message("/models", report);
                self.status = "Listed configured models.".to_string();
            }
            "/backends" => {
                let report = self.backends_report();
                self.append_local_message("/backends", report);
                self.status = "Listed backend status.".to_string();
            }
            "/help" => {
                self.show_help = true;
                self.status = "Help is open. Press Esc or ? to close it.".to_string();
            }
            _ => {
                self.append_local_message(
                    input,
                    "Unknown command. Available commands: /clear, /models, /backends.".to_string(),
                );
                self.status = "Unknown command.".to_string();
            }
        }
    }

    /// Clear completed conversation state from a local command.
    fn clear_conversation_command(&mut self) {
        if self.waiting_for_model {
            self.status = "Cannot clear while a model is answering.".to_string();
            return;
        }

        self.history.clear();
        self.active_model_name = None;
        self.status = "Conversation cleared.".to_string();
    }

    /// Add a local command result to the visible history without sending it later.
    fn append_local_message(&mut self, command: &str, answer: String) {
        self.history.push(ChatMessage {
            prompt: command.to_string(),
            model_name: "ollama-me".to_string(),
            route_reason: "Local command. Not sent to any model.".to_string(),
            answer,
            in_progress: false,
            failed: false,
            include_in_context: false,
        });
        self.trim_history();
    }

    /// Build a human-readable list of configured models.
    fn models_report(&self) -> String {
        self.models()
            .iter()
            .map(|model| {
                let status = if model.enabled { "enabled" } else { "disabled" };
                let setup = model
                    .disabled_reason
                    .as_deref()
                    .unwrap_or("ready for routing");

                format!(
                    "{} [{}]\n  strengths: {}\n  setup: {}",
                    model.display_label(),
                    status,
                    model.strengths.join(", "),
                    setup
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    /// Build a human-readable backend readiness summary.
    fn backends_report(&self) -> String {
        [
            Provider::Ollama,
            Provider::Anthropic,
            Provider::OpenAi,
            Provider::Xai,
        ]
        .iter()
        .map(|provider| {
            let models = self
                .models()
                .iter()
                .filter(|model| model.provider == *provider)
                .collect::<Vec<_>>();
            let enabled_count = models.iter().filter(|model| model.enabled).count();
            let status = if enabled_count > 0 {
                "available"
            } else {
                "not configured"
            };
            let notes = models
                .iter()
                .filter_map(|model| model.disabled_reason.as_deref())
                .collect::<Vec<_>>();
            let note = if notes.is_empty() {
                "ready".to_string()
            } else {
                notes.join("; ")
            };

            format!(
                "{}: {} ({}/{}) - {}",
                provider.label(),
                status,
                enabled_count,
                models.len(),
                note
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn completed_message(number: usize) -> ChatMessage {
        ChatMessage {
            prompt: format!("prompt {number}"),
            model_name: "Ollama llama3".to_string(),
            route_reason: "test route".to_string(),
            answer: format!("answer {number}"),
            in_progress: false,
            failed: false,
            include_in_context: true,
        }
    }

    #[test]
    fn conversation_context_is_bounded_to_recent_completed_turns() {
        let mut app = App::new();
        for number in 0..10 {
            app.history.push(completed_message(number));
        }

        let context = app.conversation_context();

        assert_eq!(context.len(), MAX_CONTEXT_TURNS);
        assert_eq!(
            context.first().expect("first context turn").user,
            "prompt 4"
        );
        assert_eq!(context.last().expect("last context turn").user, "prompt 9");
    }

    #[test]
    fn trim_history_keeps_recent_turns_only() {
        let mut app = App::new();
        for number in 0..15 {
            app.history.push(completed_message(number));
        }

        app.trim_history();

        assert_eq!(app.history.len(), MAX_STORED_TURNS);
        assert_eq!(
            app.history.first().expect("first stored turn").prompt,
            "prompt 3"
        );
        assert_eq!(
            app.history.last().expect("last stored turn").prompt,
            "prompt 14"
        );
    }

    #[test]
    fn token_events_update_active_message() {
        let mut app = App::new();
        app.history.push(ChatMessage {
            prompt: "hello".to_string(),
            model_name: "Ollama llama3".to_string(),
            route_reason: "test route".to_string(),
            answer: String::new(),
            in_progress: true,
            failed: false,
            include_in_context: true,
        });

        app.handle_model_event(ModelEvent::Token("hi".to_string()));
        app.handle_model_event(ModelEvent::Token(" there".to_string()));
        app.handle_model_event(ModelEvent::Finished);

        let message = app.history.last().expect("streamed message");
        assert_eq!(message.answer, "hi there");
        assert!(!message.in_progress);
    }

    #[test]
    fn clear_command_clears_history_without_model_request() {
        let mut app = App::new();
        app.history.push(completed_message(1));
        app.input = "/clear".to_string();

        let request = app.submit_prompt();

        assert!(request.is_none());
        assert!(app.history.is_empty());
        assert_eq!(app.status, "Conversation cleared.");
    }

    #[test]
    fn models_command_adds_local_message_not_context() {
        let mut app = App::new();
        app.input = "/models".to_string();

        let request = app.submit_prompt();

        assert!(request.is_none());
        let message = app.history.last().expect("local command message");
        assert_eq!(message.model_name, "ollama-me");
        assert!(!message.include_in_context);
        assert!(message.answer.contains("Ollama llama3"));
    }

    #[test]
    fn command_messages_are_not_sent_as_context() {
        let mut app = App::new();
        app.history.push(completed_message(1));
        app.append_local_message("/models", "local output".to_string());

        let context = app.conversation_context();

        assert_eq!(context.len(), 1);
        assert_eq!(context[0].user, "prompt 1");
    }
}
