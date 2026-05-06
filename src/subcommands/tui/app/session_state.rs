use super::ChatMessage;

/// Conversation state for the current TUI session.
pub(crate) struct SessionState {
    /// Text currently being typed in the prompt box.
    pub(crate) input: String,

    /// Visible conversation entries, including in-progress model responses and
    /// local command output.
    pub(crate) history: Vec<ChatMessage>,

    /// True while a request is running.
    pub(crate) waiting_for_model: bool,

    pub(super) active_model_name: Option<String>,
    pub(super) activity_tick: usize,
}

impl SessionState {
    pub(super) fn new() -> Self {
        Self {
            input: String::new(),
            history: Vec::new(),
            waiting_for_model: false,
            active_model_name: None,
            activity_tick: 0,
        }
    }
}
