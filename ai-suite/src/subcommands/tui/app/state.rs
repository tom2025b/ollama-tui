use std::{collections::VecDeque, ffi::OsStr};

use crate::Result;
use crate::llm::LanguageModel;
use crate::prompt_rules::RulesState;
use crate::routing::{ModelRouter, RouteExplanation};
use crate::runtime::{Runtime, RuntimeConfig};
use crate::subcommands::tui::slash_commands::{CommandRegistry, ExternalAction};

use super::ChatMessage;
use super::settings::{LayoutMode, UiTheme};

/// Slash-command state, including suggestions and queued command side effects.
pub(crate) struct CommandState {
    pub(crate) registry: CommandRegistry,
    external_actions: VecDeque<ExternalAction>,
    pub(super) suggestion_index: usize,
    pub(super) suggestions_dismissed: bool,

    /// Prompt that a slash command produced for the next model turn, if any.
    /// Drained by `submit_prompt` so commands like /fix actually reach a model.
    staged_prompt: Option<String>,
}

impl CommandState {
    pub(super) fn new() -> Self {
        Self {
            registry: CommandRegistry::default(),
            external_actions: VecDeque::new(),
            suggestion_index: 0,
            suggestions_dismissed: false,
            staged_prompt: None,
        }
    }

    pub(super) fn reset_suggestions(&mut self) {
        self.suggestion_index = 0;
        self.suggestions_dismissed = false;
    }

    pub(super) fn dismiss_suggestions(&mut self) {
        self.suggestions_dismissed = true;
    }

    pub(crate) fn queue_external_action(&mut self, action: ExternalAction) {
        self.external_actions.push_back(action);
    }

    pub(crate) fn take_external_action(&mut self) -> Option<ExternalAction> {
        self.external_actions.pop_front()
    }

    pub(crate) fn stage_prompt(&mut self, prompt: String) {
        self.staged_prompt = Some(prompt);
    }

    pub(crate) fn take_staged_prompt(&mut self) -> Option<String> {
        self.staged_prompt.take()
    }
}

/// Routing state for prompt dispatch and the optional `/model` pin.
pub(crate) struct RoutingState {
    pub(super) router: ModelRouter,
    pub(super) pinned_model: Option<LanguageModel>,
}

impl RoutingState {
    pub(super) fn new(config: &RuntimeConfig) -> Self {
        Self {
            router: ModelRouter::new(config.models()),
            pinned_model: None,
        }
    }
}

/// Conversation state for the current TUI session.
pub(crate) struct SessionState {
    /// Text currently being typed in the prompt box.
    pub(crate) input: String,
    pub(crate) input_cursor: usize,

    /// Visible conversation entries, including in-progress model responses and
    /// local command output.
    pub(crate) history: Vec<ChatMessage>,

    /// True while a request is running.
    pub(crate) waiting_for_model: bool,

    pub(crate) active_model_name: Option<String>,
    pub(crate) activity_tick: usize,
}

impl SessionState {
    pub(super) fn new() -> Self {
        Self {
            input: String::new(),
            input_cursor: 0,
            history: Vec::new(),
            waiting_for_model: false,
            active_model_name: None,
            activity_tick: 0,
        }
    }
}

/// Presentation state for overlays, scrolling, and visual preferences.
pub(crate) struct UiState {
    /// Short status line shown near the input box.
    pub(crate) status: String,

    /// True when the help overlay should be shown.
    pub(crate) show_help: bool,

    /// How many lines the user has scrolled up from the bottom of the chat
    /// history. Zero means pinned to the newest content.
    pub(crate) scroll_offset: usize,

    /// True while the interactive `/model` picker overlay is visible.
    pub(crate) show_models_picker: bool,

    pub(in crate::subcommands::tui::app) models_picker_index: usize,
    pub(in crate::subcommands::tui::app) theme: UiTheme,
    pub(in crate::subcommands::tui::app) layout_mode: LayoutMode,
}

impl UiState {
    pub(super) fn new() -> Self {
        Self {
            status: "Type a prompt. Press Enter to send. Press Esc or Ctrl-C to quit.".to_string(),
            show_help: false,
            scroll_offset: 0,
            show_models_picker: false,
            models_picker_index: 0,
            theme: UiTheme::Dark,
            layout_mode: LayoutMode::Normal,
        }
    }
}

/// Top-level application coordinator.
///
/// Domain-specific state lives in smaller structs. `App` keeps the event loop
/// simple by grouping those structs with process-level lifecycle state.
pub struct App {
    pub(crate) runtime: Runtime,
    pub(crate) routing: RoutingState,
    pub(crate) commands: CommandState,
    pub(crate) session: SessionState,
    pub(crate) ui: UiState,

    /// Set to true when the user wants to leave the app.
    pub(crate) should_quit: bool,

    pub(crate) rules: RulesState,
}

impl App {
    pub(crate) fn with_runtime(runtime: Runtime) -> Self {
        let routing = RoutingState::new(runtime.config());
        let rules = RulesState::load(runtime.paths());

        Self {
            runtime,
            routing,
            commands: CommandState::new(),
            session: SessionState::new(),
            ui: UiState::new(),
            should_quit: false,
            rules,
        }
    }

    pub(crate) fn editor_command(&self) -> &OsStr {
        self.runtime.paths().editor()
    }

    /// Run the router for `prompt` without dispatching to a model. Used by
    /// `/route` to introspect routing behavior.
    pub(crate) fn explain_route(&self, prompt: &str) -> Result<RouteExplanation> {
        self.routing.router.explain(prompt)
    }
}
