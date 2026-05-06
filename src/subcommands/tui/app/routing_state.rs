use crate::llm::LanguageModel;
use crate::routing::ModelRouter;
use crate::runtime::RuntimeConfig;

/// Routing state for prompt dispatch and the optional `/model` pin.
pub(super) struct RoutingState {
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
