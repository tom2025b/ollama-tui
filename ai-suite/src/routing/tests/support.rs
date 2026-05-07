use super::super::*;

pub(super) fn enabled_model(provider: Provider, name: &str) -> LanguageModel {
    match provider {
        Provider::Ollama => LanguageModel::ollama(name, &["test"]),
        Provider::Anthropic => LanguageModel::anthropic(name, &["test"], true, None),
        Provider::OpenAi => LanguageModel::openai(name, &["test"], true, None),
        Provider::Xai => LanguageModel::xai(name, &["test"], true, None),
    }
}

pub(super) fn disabled_model(provider: Provider, name: &str) -> LanguageModel {
    match provider {
        Provider::Ollama => LanguageModel::ollama(name, &["test"]),
        Provider::Anthropic => {
            LanguageModel::anthropic(name, &["test"], false, Some("missing key".to_string()))
        }
        Provider::OpenAi => {
            LanguageModel::openai(name, &["test"], false, Some("missing key".to_string()))
        }
        Provider::Xai => {
            LanguageModel::xai(name, &["test"], false, Some("missing key".to_string()))
        }
    }
}

pub(super) fn router_with_models(models: Vec<LanguageModel>) -> ModelRouter {
    ModelRouter { models }
}
