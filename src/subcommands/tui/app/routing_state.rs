use crate::llm::LanguageModel;
use crate::router::ModelRouter;

/// Routing state for prompt dispatch and the optional `/model` pin.
pub(super) struct RoutingState {
    pub(super) router: ModelRouter,
    pub(super) pinned_model: Option<LanguageModel>,
}

impl RoutingState {
    pub(super) fn new() -> Self {
        Self {
            router: ModelRouter::new(),
            pinned_model: None,
        }
    }
}
