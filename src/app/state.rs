use crate::command::{CommandDispatcher, CommandRegistry, ExternalAction};
use crate::llm::LanguageModel;
use crate::router::ModelRouter;
use crate::rules::RulesState;

use super::ChatMessage;

/// All state needed to draw and update the application.
///
/// Keeping app state in one struct makes the TUI loop in `main.rs` simple:
/// read input, update this struct, draw this struct, repeat.
pub struct App {
    pub(super) router: ModelRouter,
    pub(super) command_dispatcher: CommandDispatcher,

    /// Text currently being typed at the bottom of the screen.
    pub input: String,

    /// Previous completed requests.
    pub history: Vec<ChatMessage>,

    /// True while a request is running.
    pub waiting_for_model: bool,

    /// Set to true when the user wants to leave the app.
    pub should_quit: bool,

    /// Short status line shown near the input box.
    pub status: String,

    pub(super) active_model_name: Option<String>,
    pub(super) activity_tick: usize,

    /// True when the help overlay should be shown.
    pub show_help: bool,

    /// How many lines the user has scrolled up from the bottom of the chat
    /// history. Zero means pinned to the newest content.
    pub scroll_offset: usize,

    pub(super) rules: RulesState,
    pub(super) pending_external_action: Option<ExternalAction>,
    pub(super) suggestion_index: usize,
    pub(super) suggestions_dismissed: bool,

    /// True while the interactive `/models` picker overlay is visible.
    pub show_models_picker: bool,

    pub(super) models_picker_index: usize,
    pub(super) pinned_model: Option<LanguageModel>,
}

impl App {
    /// Build a fresh app with the default router.
    pub fn new() -> Self {
        Self {
            router: ModelRouter::new(),
            command_dispatcher: CommandDispatcher::new(CommandRegistry::default()),
            input: String::new(),
            history: Vec::new(),
            waiting_for_model: false,
            should_quit: false,
            status: "Type a prompt. Press Enter to send. Press Esc or Ctrl-C to quit.".to_string(),
            active_model_name: None,
            activity_tick: 0,
            show_help: false,
            scroll_offset: 0,
            rules: RulesState::load(),
            pending_external_action: None,
            suggestion_index: 0,
            suggestions_dismissed: false,
            show_models_picker: false,
            models_picker_index: 0,
            pinned_model: None,
        }
    }
}
