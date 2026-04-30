use std::{fmt::Write as _, path::PathBuf};

use crate::history;
use crate::llm::{ConversationTurn, LanguageModel, Provider, RouteDecision};
use crate::router::{ModelRouter, Router};
use crate::rules::{RulesState, RulesTarget};

/// Maximum completed exchanges kept in memory and shown in the TUI.
const MAX_STORED_TURNS: usize = 200;

/// Maximum completed exchanges sent back to the next model request.
const MAX_CONTEXT_TURNS: usize = 6;

/// Lightweight ASCII spinner used while the model is active.
const SPINNER_FRAMES: &[&str] = &["-", "\\", "|", "/"];

/// Slash commands offered by the autocomplete popup, paired with a short hint.
///
/// The list is shown in this exact order, so keep it sorted alphabetically for
/// a predictable popup. Aliases (`/model` and `/models`, `/quit` and `/exit`)
/// are listed individually so users can complete whichever feels natural.
const SLASH_COMMANDS: &[(&str, &str)] = &[
    ("/backend", "List backend readiness"),
    ("/backends", "List backend readiness"),
    ("/clear", "Clear visible conversation"),
    ("/exit", "Quit the app"),
    ("/help", "Open help overlay"),
    ("/history", "Show, save, or email history"),
    ("/model", "Pick a model to pin"),
    ("/models", "Pick a model to pin"),
    ("/quit", "Quit the app"),
    ("/rules", "Edit or toggle rules"),
];

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

/// External work that must happen outside the TUI raw-mode loop.
#[derive(Clone, Debug)]
pub enum ExternalAction {
    /// Open a rules file in nano, then reload rules when nano exits.
    EditRules {
        /// Which rules file is being edited.
        target: RulesTarget,

        /// File path to open.
        path: PathBuf,
    },
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

    /// Loaded global and project rules.
    rules: RulesState,

    /// Pending external editor action requested by a slash command.
    pending_external_action: Option<ExternalAction>,

    /// Index of the currently highlighted item in the slash-command popup.
    ///
    /// Only meaningful while the popup is actually visible; otherwise the
    /// value is unused. Readers go through `suggestion_index()` so a stale
    /// index gets clamped to the live match list automatically.
    suggestion_index: usize,

    /// True after the user dismisses the popup (for example with Esc) until
    /// the input changes again. Without this flag, Esc would have no way to
    /// hide the popup while the input still starts with a slash.
    suggestions_dismissed: bool,

    /// True while the interactive `/models` picker overlay is visible.
    ///
    /// The picker lets the user pin a specific model so the router stops
    /// choosing per prompt. Esc closes the picker; Enter pins the highlight.
    pub show_models_picker: bool,

    /// Index of the currently highlighted entry in the `/models` picker.
    ///
    /// Index 0 is always the synthetic "Auto" entry that clears the pin and
    /// hands routing back to `ModelRouter`. Indices 1.. correspond, in order,
    /// to the slice returned by `pickable_models`.
    models_picker_index: usize,

    /// Optional override that forces every new prompt to use one specific
    /// model regardless of what the rule-based router would otherwise pick.
    ///
    /// Cleared by selecting "Auto" inside the `/models` picker.
    pinned_model: Option<LanguageModel>,
}

impl App {
    /// Build a fresh app with the default router.
    pub fn new() -> Self {
        let rules = RulesState::load();

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
            rules,
            pending_external_action: None,
            suggestion_index: 0,
            suggestions_dismissed: false,
            show_models_picker: false,
            models_picker_index: 0,
            pinned_model: None,
        }
    }

    /// Return the list of models that can be displayed by the UI.
    pub fn models(&self) -> &[LanguageModel] {
        self.router.models()
    }

    /// Add one typed character to the input buffer.
    ///
    /// Typing always restarts the suggestion popup so filtering feels
    /// responsive: any new keystroke resets the highlight to the first match
    /// and clears a previous Esc dismissal.
    pub fn push_input_char(&mut self, character: char) {
        self.input.push(character);
        self.suggestion_index = 0;
        self.suggestions_dismissed = false;
    }

    /// Remove the most recent typed character.
    pub fn backspace(&mut self) {
        self.input.pop();
        // Editing the input revives the popup if it was previously dismissed,
        // and resets the highlight so the user does not land on a stale row.
        self.suggestion_index = 0;
        self.suggestions_dismissed = false;
    }

    /// Clear the current input without submitting it.
    pub fn clear_input(&mut self) {
        self.input.clear();
        self.status = "Input cleared.".to_string();
        self.suggestion_index = 0;
        self.suggestions_dismissed = false;
    }

    /// Slash-command suggestions that match the current input.
    ///
    /// Returns an empty list when the popup should not be shown. The popup is
    /// hidden when:
    /// - the input does not start with `/`,
    /// - the input already contains whitespace (the user has moved on to
    ///   typing a subcommand or argument),
    /// - the user dismissed the popup with Esc since the last edit, or
    /// - nothing in the command list matches the current prefix.
    pub fn command_suggestions(&self) -> Vec<(&'static str, &'static str)> {
        if self.suggestions_dismissed {
            return Vec::new();
        }
        if !self.input.starts_with('/') {
            return Vec::new();
        }
        if self.input.chars().any(char::is_whitespace) {
            return Vec::new();
        }

        SLASH_COMMANDS
            .iter()
            .copied()
            .filter(|(command, _)| command.starts_with(&self.input))
            .collect()
    }

    /// Currently highlighted suggestion index, clamped to the live match list.
    ///
    /// Going through this method keeps callers from worrying about stale
    /// indices: if the match list shrinks because the user typed more
    /// characters, the highlight automatically snaps to the last valid row.
    pub fn suggestion_index(&self) -> usize {
        let suggestions = self.command_suggestions();
        if suggestions.is_empty() {
            0
        } else {
            self.suggestion_index.min(suggestions.len() - 1)
        }
    }

    /// Move the popup highlight to the previous item, wrapping to the bottom.
    pub fn select_previous_suggestion(&mut self) {
        let suggestions = self.command_suggestions();
        if suggestions.is_empty() {
            return;
        }
        let current = self.suggestion_index.min(suggestions.len() - 1);
        self.suggestion_index = if current == 0 {
            suggestions.len() - 1
        } else {
            current - 1
        };
    }

    /// Move the popup highlight to the next item, wrapping to the top.
    pub fn select_next_suggestion(&mut self) {
        let suggestions = self.command_suggestions();
        if suggestions.is_empty() {
            return;
        }
        let current = self.suggestion_index.min(suggestions.len() - 1);
        self.suggestion_index = (current + 1) % suggestions.len();
    }

    /// Replace the input with the highlighted suggestion plus a trailing space.
    ///
    /// The trailing space lets the user type subcommands like `/rules show`
    /// without having to reach for the spacebar, and it also hides the popup
    /// naturally because `command_suggestions` filters on whitespace.
    /// Returns `true` when a suggestion was actually applied.
    pub fn accept_suggestion(&mut self) -> bool {
        let suggestions = self.command_suggestions();
        if suggestions.is_empty() {
            return false;
        }
        let index = self.suggestion_index.min(suggestions.len() - 1);
        let (selected, _hint) = suggestions[index];

        self.input.clear();
        self.input.push_str(selected);
        self.input.push(' ');
        self.suggestion_index = 0;
        // The trailing space already hides the popup, but setting this flag
        // keeps things consistent with explicit dismissal.
        self.suggestions_dismissed = true;
        true
    }

    /// Hide the suggestion popup until the input is edited again.
    pub fn dismiss_suggestions(&mut self) {
        self.suggestions_dismissed = true;
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
            "Help is open. Press q, Esc, ?, or Ctrl-C to close it.".to_string()
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

    /// All models the picker is allowed to offer for pinning.
    ///
    /// Disabled cloud models are filtered out because pinning to one would
    /// just produce request failures at send time (no API key, etc.).
    /// `/backends` already exists for inspecting unconfigured backends.
    pub fn pickable_models(&self) -> Vec<&LanguageModel> {
        self.router
            .models()
            .iter()
            .filter(|model| model.enabled)
            .collect()
    }

    /// Total number of rows shown by the picker, including the leading "Auto"
    /// entry. Always at least 1.
    pub fn models_picker_total(&self) -> usize {
        // The +1 is the synthetic "Auto" entry; see `show_models_picker` docs.
        self.pickable_models().len() + 1
    }

    /// Currently highlighted picker row, clamped to the live entry list.
    ///
    /// Going through this method protects callers from a stale index in the
    /// rare case the underlying model list shrinks while the picker is open.
    pub fn models_picker_index(&self) -> usize {
        let total = self.models_picker_total();
        if total == 0 {
            0
        } else {
            self.models_picker_index.min(total - 1)
        }
    }

    /// True when the given model is the one currently pinned, used by the UI
    /// to mark the active row with a small indicator.
    pub fn is_pinned(&self, model: &LanguageModel) -> bool {
        match &self.pinned_model {
            Some(pinned) => pinned.provider == model.provider && pinned.name == model.name,
            None => false,
        }
    }

    /// Open the interactive `/models` picker overlay.
    ///
    /// The highlight starts on the row that matches the existing pin (so
    /// reopening the picker shows the user what is currently active), or on
    /// the "Auto" row when nothing is pinned.
    pub fn open_models_picker(&mut self) {
        self.show_models_picker = true;
        // Slash-command popup must not stay visible behind the modal picker.
        self.suggestions_dismissed = true;
        // Pre-select the row representing the current state so the user can
        // confirm with Enter or quickly nudge to a different choice.
        self.models_picker_index = match &self.pinned_model {
            None => 0,
            Some(pinned) => self
                .pickable_models()
                .iter()
                .position(|model| model.provider == pinned.provider && model.name == pinned.name)
                .map(|i| i + 1) // +1 because index 0 is "Auto".
                .unwrap_or(0),
        };
        self.status =
            "Pick a model. Up/Down to navigate, Enter to select, Esc to cancel.".to_string();
    }

    /// Close the picker without applying any change.
    pub fn close_models_picker(&mut self) {
        if !self.show_models_picker {
            return;
        }
        self.show_models_picker = false;
        self.status = "Model picker cancelled.".to_string();
    }

    /// Move the picker highlight up by one row, wrapping at the top.
    pub fn select_previous_model(&mut self) {
        let total = self.models_picker_total();
        if total == 0 {
            return;
        }
        let current = self.models_picker_index.min(total - 1);
        self.models_picker_index = if current == 0 { total - 1 } else { current - 1 };
    }

    /// Move the picker highlight down by one row, wrapping at the bottom.
    pub fn select_next_model(&mut self) {
        let total = self.models_picker_total();
        if total == 0 {
            return;
        }
        let current = self.models_picker_index.min(total - 1);
        self.models_picker_index = (current + 1) % total;
    }

    /// Apply the highlighted picker row.
    ///
    /// Selecting "Auto" clears the pin and hands routing back to the rule-
    /// based `ModelRouter`. Selecting a specific model pins it for every
    /// future prompt until the user picks something else.
    pub fn accept_model_selection(&mut self) {
        let total = self.models_picker_total();
        if total == 0 {
            return;
        }
        let index = self.models_picker_index.min(total - 1);

        if index == 0 {
            self.pinned_model = None;
            self.status = "Routing reset to Auto. Router will pick per prompt.".to_string();
        } else {
            // Clone the chosen model out of the borrowed list before we touch
            // self.pinned_model — Rust will not allow us to hold the borrow
            // and the mutation at the same time.
            let chosen = self.pickable_models()[index - 1].clone();
            let label = chosen.display_label();
            self.pinned_model = Some(chosen);
            self.status = format!("Pinned to {label}. New prompts will skip the router.");
        }

        self.show_models_picker = false;
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

        // A pinned model bypasses the rule-based router entirely. The route
        // reason still gets a human-readable explanation so the conversation
        // panel makes the override visible.
        let route = if let Some(pinned) = &self.pinned_model {
            RouteDecision {
                model: pinned.clone(),
                reason: format!(
                    "Pinned to {} via /models picker. Router skipped.",
                    pinned.display_label()
                ),
            }
        } else {
            self.router.route(&prompt)
        };
        let context = self.conversation_context();
        let prompt_for_model = self.rules.prompt_with_rules(&prompt);
        let model_name = route.model.display_label();
        let route_reason = if let Some(rules_summary) = self.rules.application_summary() {
            format!("{} {}", route.reason, rules_summary)
        } else {
            route.reason.clone()
        };

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
            prompt: prompt_for_model,
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
    ///
    /// When the user has pinned a model through `/models`, the label is
    /// suffixed with `(pinned)` so the status panel makes the override
    /// obvious at a glance.
    pub fn current_model_label(&self) -> String {
        if let Some(model_name) = &self.active_model_name {
            return model_name.clone();
        }

        if let Some(pinned) = &self.pinned_model {
            return format!("{} (pinned)", pinned.display_label());
        }

        self.history
            .iter()
            .rev()
            .find(|message| message.include_in_context)
            .map(|message| message.model_name.clone())
            .unwrap_or_else(|| "none".to_string())
    }

    /// Current rules status for the status panel.
    pub fn rules_status_line(&self) -> String {
        self.rules.status_line()
    }

    /// Take the next external action requested by a local command.
    pub fn take_external_action(&mut self) -> Option<ExternalAction> {
        self.pending_external_action.take()
    }

    /// Update app state after nano exits from a `/rules` edit.
    pub fn complete_rules_edit(
        &mut self,
        target: RulesTarget,
        path: PathBuf,
        editor_result: Result<(), String>,
    ) {
        match editor_result {
            Ok(()) => {
                let rules_were_enabled = self.rules.enabled();
                self.rules = RulesState::load().with_enabled(rules_were_enabled);
                self.append_local_message(
                    "/rules",
                    format!(
                        "Reloaded {} from {}.\nRules: {}",
                        target.label(),
                        path.display(),
                        self.rules.status_line()
                    ),
                );
                self.status = format!("Reloaded {}.", target.label());
            }
            Err(error) => {
                self.append_local_message(
                    "/rules",
                    format!(
                        "Could not edit {} at {}.\n{}",
                        target.label(),
                        path.display(),
                        error
                    ),
                );
                self.status = format!("Failed to edit {}.", target.label());
            }
        }
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
        let command = input
            .split_whitespace()
            .next()
            .unwrap_or_default()
            .to_ascii_lowercase();

        match command.as_str() {
            "/clear" => self.clear_conversation_command(),
            "/model" | "/models" => {
                // Replaces the old text dump with an interactive picker that
                // reuses the slash-command popup style. The text report is
                // still reachable via `/backends` for the broader summary.
                self.open_models_picker();
            }
            "/backend" | "/backends" => {
                let report = self.backends_report();
                self.append_local_message(input, report);
                self.status = "Listed backend status.".to_string();
            }
            "/rules" => self.handle_rules_command(input),
            "/history" => self.handle_history_command(input),
            "/help" => {
                self.show_help = true;
                self.status = "Help is open. Press q, Esc, ?, or Ctrl-C to close it.".to_string();
            }
            "/quit" | "/exit" | "/q" => self.quit(),
            _ => {
                self.append_local_message(
                    input,
                    "Unknown command. Available commands: /clear, /model, /backend, /rules, /history, /help, /quit."
                        .to_string(),
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

    /// Handle the rules command family.
    fn handle_rules_command(&mut self, input: &str) {
        let mut args = input.split_whitespace().skip(1);
        let subcommand = args.next().map(|arg| arg.to_ascii_lowercase());

        match subcommand.as_deref() {
            None => {
                let target = if self.rules.project_root().is_some() {
                    RulesTarget::Project
                } else {
                    RulesTarget::Global
                };
                self.queue_rules_edit(input, target);
            }
            Some("global") => self.queue_rules_edit(input, RulesTarget::Global),
            Some("project") => self.queue_rules_edit(input, RulesTarget::Project),
            Some("show") | Some("status") => {
                self.append_local_message(input, self.rules.report());
                self.status = "Displayed rules status.".to_string();
            }
            Some("off") | Some("disable") => {
                self.rules.set_enabled(false);
                self.append_local_message(input, "All rules are off for new prompts.".to_string());
                self.status = "Rules turned off.".to_string();
            }
            Some("on") | Some("enable") => {
                self.rules = RulesState::load().with_enabled(true);
                self.append_local_message(input, self.rules.report());
                self.status = "Rules turned on and reloaded.".to_string();
            }
            Some("toggle") => {
                let enabled = !self.rules.enabled();
                if enabled {
                    self.rules = RulesState::load().with_enabled(true);
                } else {
                    self.rules.set_enabled(false);
                }
                self.append_local_message(
                    input,
                    format!(
                        "Rules are now {}.\nRules: {}",
                        if enabled { "on" } else { "off" },
                        self.rules.status_line()
                    ),
                );
                self.status = if enabled {
                    "Rules turned on.".to_string()
                } else {
                    "Rules turned off.".to_string()
                };
            }
            _ => {
                self.append_local_message(
                    input,
                    "Usage: /rules [global|project|show|off|on|toggle]".to_string(),
                );
                self.status = "Unknown /rules command.".to_string();
            }
        }
    }

    /// Queue a nano edit for a rules file.
    fn queue_rules_edit(&mut self, input: &str, target: RulesTarget) {
        if self.waiting_for_model {
            self.append_local_message(
                input,
                "Wait for the current model response to finish before editing rules.".to_string(),
            );
            self.status = "Cannot edit rules while a model is answering.".to_string();
            return;
        }

        match self.rules.prepare_edit(target) {
            Ok(path) => {
                self.pending_external_action = Some(ExternalAction::EditRules {
                    target,
                    path: path.clone(),
                });
                self.status = format!("Opening nano for {} at {}.", target.label(), path.display());
            }
            Err(error) => {
                self.append_local_message(
                    input,
                    format!("Could not prepare {}: {error}", target.label()),
                );
                self.status = format!("Failed to prepare {}.", target.label());
            }
        }
    }

    /// Handle `/history` display, save, and email commands.
    fn handle_history_command(&mut self, input: &str) {
        let mut args = input.split_whitespace().skip(1);
        let subcommand = args.next().map(|arg| arg.to_ascii_lowercase());

        match subcommand.as_deref() {
            None | Some("show") => {
                self.append_local_message(input, self.history_report());
                self.status = "Displayed history.".to_string();
            }
            Some("save") => {
                let requested_path = args.next();
                let report = self.history_report();

                match history::save_report(&report, requested_path) {
                    Ok(path) => {
                        self.append_local_message(
                            input,
                            format!("Saved history to {}.", path.display()),
                        );
                        self.status = "Saved history.".to_string();
                    }
                    Err(error) => {
                        self.append_local_message(
                            input,
                            format!("Could not save history: {error}"),
                        );
                        self.status = "Failed to save history.".to_string();
                    }
                }
            }
            Some("email") | Some("mail") => {
                let subject = args.collect::<Vec<_>>().join(" ");
                let subject = if subject.trim().is_empty() {
                    "ollama-me history"
                } else {
                    subject.trim()
                };
                let report = self.history_report();

                match history::email_report(&report, subject) {
                    Ok(()) => {
                        self.append_local_message(
                            input,
                            format!("Emailed history with subject: {subject}"),
                        );
                        self.status = "Emailed history.".to_string();
                    }
                    Err(error) => {
                        self.append_local_message(
                            input,
                            format!("Could not email history through send-report: {error}"),
                        );
                        self.status = "Failed to email history.".to_string();
                    }
                }
            }
            _ => {
                self.append_local_message(
                    input,
                    "Usage: /history [show|save [path]|email [subject]]".to_string(),
                );
                self.status = "Unknown /history command.".to_string();
            }
        }
    }

    /// Build a plain-text transcript for `/history`.
    fn history_report(&self) -> String {
        let conversation = self
            .history
            .iter()
            .filter(|message| message.include_in_context)
            .collect::<Vec<_>>();

        let mut report = String::new();
        let _ = writeln!(report, "ollama-me history");
        let _ = writeln!(report, "Rules: {}", self.rules.status_line());

        if let Some(project_root) = self.rules.project_root() {
            let _ = writeln!(report, "Project: {}", project_root.display());
        }

        let _ = writeln!(report);

        if conversation.is_empty() {
            report.push_str("No model conversation history yet.\n");
            return report;
        }

        for (index, message) in conversation.iter().enumerate() {
            let _ = writeln!(report, "## Turn {}", index + 1);
            let _ = writeln!(report, "Model: {}", message.model_name);
            let _ = writeln!(report, "Route: {}", message.route_reason);

            if message.failed {
                let _ = writeln!(report, "Status: failed");
            } else if message.in_progress {
                let _ = writeln!(report, "Status: streaming");
            }

            let _ = writeln!(report);
            let _ = writeln!(report, "User:");
            let _ = writeln!(report, "{}", message.prompt);
            let _ = writeln!(report);
            let _ = writeln!(report, "Assistant:");
            let answer = if message.answer.trim().is_empty() {
                "(no answer yet)"
            } else {
                &message.answer
            };
            let _ = writeln!(report, "{answer}");
            let _ = writeln!(report);
        }

        report
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
        for number in 0..(MAX_STORED_TURNS + 3) {
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
            format!("prompt {}", MAX_STORED_TURNS + 2)
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
    fn models_command_opens_picker_without_model_request() {
        let mut app = App::new();
        app.input = "/models".to_string();

        let request = app.submit_prompt();

        // The picker is an interactive overlay, not a text dump, so no message
        // should land in history and no model request should be queued.
        assert!(request.is_none());
        assert!(app.show_models_picker);
        assert!(app.history.is_empty());
    }

    #[test]
    fn singular_model_command_opens_picker_without_model_request() {
        let mut app = App::new();
        app.input = "/model".to_string();

        let request = app.submit_prompt();

        assert!(request.is_none());
        assert!(app.show_models_picker);
        assert!(app.history.is_empty());
    }

    #[test]
    fn models_picker_navigates_and_pins_selection() {
        let mut app = App::new();
        app.open_models_picker();
        // Snapshot the first real model before mutation; index 0 is "Auto".
        let expected = app.pickable_models()[0].clone();
        app.select_next_model();
        app.accept_model_selection();

        assert!(!app.show_models_picker);
        assert!(app.is_pinned(&expected));
        assert!(app.current_model_label().contains("(pinned)"));
        assert!(
            app.current_model_label()
                .contains(&expected.display_label())
        );
    }

    #[test]
    fn models_picker_auto_entry_clears_pin() {
        let mut app = App::new();
        // Pin first…
        app.open_models_picker();
        let first = app.pickable_models()[0].clone();
        app.select_next_model();
        app.accept_model_selection();
        assert!(app.is_pinned(&first));

        // …then reopen and pick "Auto" (row 0) to clear the pin.
        app.open_models_picker();
        app.models_picker_index = 0;
        app.accept_model_selection();

        assert!(!app.is_pinned(&first));
        assert!(!app.current_model_label().contains("(pinned)"));
    }

    #[test]
    fn pinned_model_overrides_router_for_new_prompts() {
        let mut app = App::new();
        // Pick the first non-Auto entry so we know a pin is in effect.
        app.open_models_picker();
        let pinned = app.pickable_models()[0].clone();
        app.select_next_model();
        app.accept_model_selection();

        app.input = "what is the latest news today".to_string();
        let request = app.submit_prompt().expect("submitted request");

        // Without the pin, this prompt would be routed as "current context"
        // and likely land on Grok if configured. With the pin, the chosen
        // model must match what the user pinned.
        assert_eq!(request.route.model.display_label(), pinned.display_label());
        assert!(request.route.reason.contains("Pinned"));
    }

    #[test]
    fn esc_on_picker_cancels_without_changing_pin() {
        let mut app = App::new();
        app.open_models_picker();
        let candidate = app.pickable_models()[0].clone();
        app.select_next_model();
        app.close_models_picker();

        assert!(!app.show_models_picker);
        assert!(!app.is_pinned(&candidate));
    }

    #[test]
    fn select_next_model_wraps_around_picker() {
        let mut app = App::new();
        app.open_models_picker();
        let total = app.models_picker_total();
        // Step through every row exactly once and confirm we land back at
        // the start; this proves the wrap-around behavior.
        for _ in 0..total {
            app.select_next_model();
        }
        assert_eq!(app.models_picker_index(), 0);
    }

    #[test]
    fn singular_backend_command_adds_local_message_without_model_request() {
        let mut app = App::new();
        app.input = "/backend".to_string();

        let request = app.submit_prompt();

        assert!(request.is_none());
        let message = app.history.last().expect("local command message");
        assert_eq!(message.prompt, "/backend");
        assert_eq!(message.model_name, "ollama-me");
        assert!(!message.include_in_context);
        assert!(message.answer.contains("Ollama"));
    }

    #[test]
    fn help_command_opens_help_without_model_request() {
        let mut app = App::new();
        app.input = "/help".to_string();

        let request = app.submit_prompt();

        assert!(request.is_none());
        assert!(app.show_help);
        assert!(app.history.is_empty());
    }

    #[test]
    fn quit_command_exits_without_model_request() {
        let mut app = App::new();
        app.input = "/quit".to_string();

        let request = app.submit_prompt();

        assert!(request.is_none());
        assert!(app.should_quit);
        assert!(app.history.is_empty());
    }

    #[test]
    fn command_messages_are_not_sent_as_context() {
        let mut app = App::new();
        app.history.push(completed_message(1));
        // Use any local-message-producing command; /models is now interactive
        // so we exercise the same path through `append_local_message` directly.
        app.append_local_message("/help", "local output".to_string());

        let context = app.conversation_context();

        assert_eq!(context.len(), 1);
        assert_eq!(context[0].user, "prompt 1");
    }

    #[test]
    fn command_suggestions_empty_for_normal_input() {
        let mut app = App::new();
        app.input = "hello".to_string();
        assert!(app.command_suggestions().is_empty());
    }

    #[test]
    fn command_suggestions_show_all_commands_for_slash_alone() {
        let mut app = App::new();
        app.input = "/".to_string();
        let suggestions = app.command_suggestions();
        assert_eq!(suggestions.len(), SLASH_COMMANDS.len());
    }

    #[test]
    fn command_suggestions_filter_by_prefix() {
        let mut app = App::new();
        app.input = "/m".to_string();
        let names: Vec<&str> = app
            .command_suggestions()
            .into_iter()
            .map(|(command, _)| command)
            .collect();
        assert_eq!(names, vec!["/model", "/models"]);
    }

    #[test]
    fn command_suggestions_hidden_after_whitespace() {
        let mut app = App::new();
        app.input = "/rules ".to_string();
        assert!(app.command_suggestions().is_empty());
    }

    #[test]
    fn accept_suggestion_replaces_input_and_appends_space() {
        let mut app = App::new();
        app.input = "/h".to_string();
        // /h matches /help and /history; first match is /help (alphabetical).
        let accepted = app.accept_suggestion();
        assert!(accepted);
        assert_eq!(app.input, "/help ");
        // Trailing space hides the popup naturally.
        assert!(app.command_suggestions().is_empty());
    }

    #[test]
    fn select_next_wraps_around_match_list() {
        let mut app = App::new();
        app.input = "/m".to_string();
        // /model is at index 0, /models at index 1.
        app.select_next_suggestion();
        assert_eq!(app.suggestion_index(), 1);
        app.select_next_suggestion();
        assert_eq!(app.suggestion_index(), 0);
    }

    #[test]
    fn select_previous_wraps_to_end() {
        let mut app = App::new();
        app.input = "/m".to_string();
        app.select_previous_suggestion();
        assert_eq!(app.suggestion_index(), 1);
    }

    #[test]
    fn select_next_is_noop_without_suggestions() {
        let mut app = App::new();
        app.input = "hello".to_string();
        app.select_next_suggestion();
        assert_eq!(app.suggestion_index(), 0);
    }

    #[test]
    fn dismiss_suggestions_hides_popup_until_input_changes() {
        let mut app = App::new();
        app.input = "/".to_string();
        assert!(!app.command_suggestions().is_empty());

        app.dismiss_suggestions();
        assert!(app.command_suggestions().is_empty());

        app.push_input_char('h');
        assert!(!app.command_suggestions().is_empty());
    }

    #[test]
    fn suggestion_index_clamps_when_match_list_shrinks() {
        let mut app = App::new();
        app.input = "/m".to_string();
        // Set raw index to 1 (= /models).
        app.select_next_suggestion();
        assert_eq!(app.suggestion_index(), 1);

        // Mutate the input directly so the bookkeeping in `push_input_char`
        // doesn't reset the index. This simulates a stale index and proves the
        // clamp inside `suggestion_index` works.
        app.input.push('o');
        let suggestions = app.command_suggestions();
        assert_eq!(suggestions.len(), 2);
        app.input = "/model".to_string();
        let suggestions = app.command_suggestions();
        assert_eq!(suggestions.len(), 2);
        app.input = "/models".to_string();
        let suggestions = app.command_suggestions();
        assert_eq!(suggestions.len(), 1);
        assert_eq!(app.suggestion_index(), 0);
    }
}
