use crate::llm::{ConversationTurn, RouteDecision};

/// Maximum completed exchanges kept in memory and shown in the TUI.
pub(crate) const MAX_STORED_TURNS: usize = 200;

/// Maximum completed exchanges sent back to the next model request.
pub(crate) const MAX_CONTEXT_TURNS: usize = 6;

/// Lightweight ASCII spinner used while the model is active.
pub(crate) const SPINNER_FRAMES: &[&str] = &["-", "\\", "|", "/"];

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

    /// True when this entry was produced locally instead of by a streamed model answer.
    pub is_local_message: bool,
}

impl ChatMessage {
    pub(crate) fn streaming_model_turn(
        prompt: String,
        model_name: String,
        route_reason: String,
    ) -> Self {
        Self {
            prompt,
            model_name,
            route_reason,
            answer: String::new(),
            in_progress: true,
            failed: false,
            include_in_context: true,
            is_local_message: false,
        }
    }

    pub(crate) fn terminal_handoff(
        prompt: String,
        model_name: String,
        route_reason: String,
    ) -> Self {
        Self {
            prompt,
            model_name,
            route_reason,
            answer: "→ Prompt forwarded. Working in terminal app — exit to return here."
                .to_string(),
            in_progress: false,
            failed: false,
            include_in_context: false,
            is_local_message: true,
        }
    }

    pub(crate) fn local_output(command: &str, answer: String) -> Self {
        Self {
            prompt: command.to_string(),
            model_name: "ai-suite".to_string(),
            route_reason: "Local command. Not sent to any model.".to_string(),
            answer,
            in_progress: false,
            failed: false,
            include_in_context: false,
            is_local_message: true,
        }
    }

    pub(crate) fn append_token(&mut self, token: &str) {
        self.answer.push_str(token);
    }

    pub(crate) fn finish_streaming(&mut self) {
        self.in_progress = false;
    }

    pub(crate) fn fail_streaming(&mut self, error: &str) {
        if !self.answer.trim().is_empty() {
            self.answer.push_str("\n\n");
        }
        self.answer.push_str(error);
        self.in_progress = false;
        self.failed = true;
    }

    pub(crate) fn remember_for_context(&mut self) {
        self.include_in_context = true;
    }

    pub(crate) fn forget_context(&mut self) {
        self.include_in_context = false;
    }

    pub(crate) fn is_model_turn(&self) -> bool {
        !self.is_local_message
    }

    pub(crate) fn is_finished_model_turn(&self) -> bool {
        self.is_model_turn() && !self.in_progress && !self.failed
    }

    pub(crate) fn has_completed_model_answer(&self) -> bool {
        self.is_finished_model_turn() && !self.answer.trim().is_empty()
    }

    pub(crate) fn is_ready_for_context(&self) -> bool {
        self.include_in_context && self.has_completed_model_answer()
    }

    pub(crate) fn context_turn(&self) -> Option<ConversationTurn> {
        self.is_ready_for_context().then(|| ConversationTurn {
            user: self.prompt.clone(),
            assistant: self.answer.clone(),
        })
    }
}

/// Work item created when the user submits a prompt.
///
/// The TUI loop owns terminal input and drawing. The async task owns the model
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
