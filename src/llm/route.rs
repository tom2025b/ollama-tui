use super::model::LanguageModel;

/// The router's final decision for a prompt.
#[derive(Clone, Debug)]
pub struct RouteDecision {
    /// The model that should answer the prompt.
    pub model: LanguageModel,
    /// Short explanation written for a human.
    pub reason: String,
}
