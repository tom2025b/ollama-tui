use super::super::*;

pub(super) fn enabled_model(provider: Provider, name: &str) -> LanguageModel {
    match provider {
        Provider::Ollama => LanguageModel::ollama(name, &["test"]),
        Provider::ClaudeCode => LanguageModel::claude_code(name, &["test"], true, None),
        Provider::Codex => LanguageModel::codex(name, &["test"], true, None),
    }
}

pub(super) fn disabled_model(provider: Provider, name: &str) -> LanguageModel {
    match provider {
        Provider::Ollama => LanguageModel::ollama(name, &["test"]),
        Provider::ClaudeCode => {
            LanguageModel::claude_code(name, &["test"], false, Some("disabled".to_string()))
        }
        Provider::Codex => {
            LanguageModel::codex(name, &["test"], false, Some("disabled".to_string()))
        }
    }
}

pub(super) fn router_with_models(models: Vec<LanguageModel>) -> ModelRouter {
    ModelRouter { models }
}
