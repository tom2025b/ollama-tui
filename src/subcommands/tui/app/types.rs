use crate::llm::{ConversationTurn, RouteDecision};

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

    /// True when this entry was produced by a local slash command, not a model.
    pub is_local_message: bool,
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
