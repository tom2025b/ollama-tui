use super::super::*;

pub(super) fn ollama_model(name: &str) -> LanguageModel {
    LanguageModel::ollama(name, &["test"])
}

pub(super) fn router_with_models(models: Vec<LanguageModel>) -> ModelRouter {
    ModelRouter { models }
}
