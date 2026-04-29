use std::env;

use crate::{
    anthropic,
    llm::{LanguageModel, Provider, RouteDecision},
    openai, xai,
};

/// Primary local model.
pub const PRIMARY_OLLAMA_MODEL: &str = "llama3";

/// Environment variable for the small local Ollama model.
pub const FAST_OLLAMA_MODEL_ENV: &str = "OLLAMA_FAST_MODEL";

/// Default fast local model.
///
/// This defaults to the same installed Llama 3 tag used by the primary local
/// fallback, so short prompts work on a fresh setup. Set `OLLAMA_FAST_MODEL` to
/// a smaller installed model if you want lower latency.
pub const DEFAULT_FAST_OLLAMA_MODEL: &str = "llama3:latest";

/// Explicit user instructions that require local/private handling.
const LOCAL_ONLY_KEYWORDS: &[&str] =
    &["private", "privacy", "offline", "local only", "do not send"];

/// Sensitive phrases that should never be sent to a cloud provider automatically.
const SENSITIVE_PHRASES: &[&str] = &[
    "api key",
    "account number",
    "bank account",
    "client data",
    "credit card",
    "email address",
    "employment contract",
    "home address",
    "legal contract",
    "medical record",
    "medical records",
    "patient record",
    "personal data",
    "personal note",
    "phone number",
    "private key",
    "routing number",
    "secret key",
    "social security",
    "tax return",
];

/// Sensitive single-word markers. These are matched as words, not substrings.
const SENSITIVE_WORDS: &[&str] = &[
    "1099",
    "attorney",
    "banking",
    "birthdate",
    "contract",
    "credential",
    "credentials",
    "diagnosis",
    "diagnoses",
    "divorce",
    "dob",
    "insurance",
    "lawsuit",
    "lawyer",
    "medical",
    "medication",
    "passport",
    "password",
    "patient",
    "payroll",
    "prescription",
    "salary",
    "secret",
    "ssn",
    "tax",
    "taxes",
    "therapy",
    "therapist",
    "token",
    "w2",
];

/// Common interface for anything that can choose a backend for a prompt.
///
/// The rest of the app should call this trait instead of depending on the
/// current rule-based implementation directly.
///
/// Future dynamic router will implement this same trait.
pub trait Router {
    /// Route one prompt to one backend/model.
    fn route(&self, prompt: &str) -> RouteDecision;

    /// Return every model the router knows about.
    ///
    /// The TUI uses this for visibility into enabled and disabled backends.
    fn models(&self) -> &[LanguageModel];
}

/// Chooses the best available model for a prompt.
///
/// The router is intentionally readable. It classifies the prompt with simple
/// rules, then selects the best configured model for that class. If a cloud API
/// key is missing, the router falls back to a local Ollama model instead of
/// choosing a backend that cannot run.
pub struct ModelRouter {
    /// Ordered list of models.
    ///
    /// The first model is treated as the safest default.
    models: Vec<LanguageModel>,
}

impl ModelRouter {
    /// Create the router with all supported backends.
    ///
    /// Cloud models are listed even when disabled so the TUI can show what must
    /// be configured.
    pub fn new() -> Self {
        let fast_ollama_model = env::var(FAST_OLLAMA_MODEL_ENV)
            .unwrap_or_else(|_| DEFAULT_FAST_OLLAMA_MODEL.to_string());

        Self {
            models: vec![
                LanguageModel::ollama(
                    PRIMARY_OLLAMA_MODEL,
                    &[
                        "primary local model",
                        "private/offline prompts",
                        "reliable fallback",
                    ],
                ),
                LanguageModel::ollama(
                    &fast_ollama_model,
                    &["fast local model", "short/simple prompts", "low latency"],
                ),
                LanguageModel::anthropic(
                    &anthropic::configured_model_name(),
                    &["deep coding", "careful reasoning", "long structured work"],
                    anthropic::is_configured(),
                    disabled_reason(
                        anthropic::is_configured(),
                        anthropic::missing_configuration_reason(),
                    ),
                ),
                LanguageModel::openai(
                    &openai::configured_model_name(),
                    &["balanced cloud model", "general tasks", "creative drafting"],
                    openai::is_configured(),
                    disabled_reason(
                        openai::is_configured(),
                        openai::missing_configuration_reason(),
                    ),
                ),
                LanguageModel::xai(
                    &xai::configured_model_name(),
                    &[
                        "Grok reasoning",
                        "public-discourse questions",
                        "fresh-context style prompts",
                    ],
                    xai::is_configured(),
                    disabled_reason(xai::is_configured(), xai::missing_configuration_reason()),
                ),
            ],
        }
    }

    /// Internal implementation for the current rule-based router.
    fn route_with_rules(&self, prompt: &str) -> RouteDecision {
        let profile = PromptProfile::from_prompt(prompt);

        if profile.needs_privacy {
            return self.choose_with_fallback(
                &[Provider::Ollama],
                "The prompt contains privacy instructions or sensitive data markers, so I kept it on Ollama.",
            );
        }

        if profile.needs_current_context {
            return self.choose_with_fallback(
                &[Provider::Xai, Provider::OpenAi, Provider::Anthropic, Provider::Ollama],
                "This asks for current or public-context reasoning, so I preferred Grok and then fell back by availability.",
            );
        }

        if profile.needs_deep_reasoning_or_code {
            return self.choose_with_fallback(
                &[Provider::Anthropic, Provider::OpenAi, Provider::Xai, Provider::Ollama],
                "This looks like coding, debugging, planning, or deep reasoning, so I preferred Claude and then fell back by availability.",
            );
        }

        if profile.is_simple {
            return self.choose_specific_model(
                &Provider::Ollama,
                &self.fast_ollama_model_name(),
                "This is short/simple, so I chose the fast local Ollama model.",
            );
        }

        if profile.is_creative_or_general_cloud {
            return self.choose_with_fallback(
                &[Provider::OpenAi, Provider::Anthropic, Provider::Xai, Provider::Ollama],
                "This is a general or creative prompt, so I preferred GPT-4o and then fell back by availability.",
            );
        }

        self.choose_with_fallback(
            &[Provider::OpenAi, Provider::Anthropic, Provider::Ollama],
            "No special rule matched, so I chose the best configured general-purpose model.",
        )
    }

    /// Choose the first enabled provider from a preference list.
    fn choose_with_fallback(&self, providers: &[Provider], reason: &str) -> RouteDecision {
        for provider in providers {
            if let Some(model) = self.first_enabled_provider(provider) {
                return RouteDecision {
                    model,
                    reason: reason.to_string(),
                };
            }
        }

        RouteDecision {
            model: self.primary_ollama_model(),
            reason: "No preferred cloud backend is configured, so I used the primary local Ollama model.".to_string(),
        }
    }

    /// Choose one exact model when it is enabled, otherwise use primary Llama3.
    fn choose_specific_model(
        &self,
        provider: &Provider,
        model_name: &str,
        reason: &str,
    ) -> RouteDecision {
        if let Some(model) = self
            .models
            .iter()
            .find(|model| model.enabled && &model.provider == provider && model.name == model_name)
            .cloned()
        {
            return RouteDecision {
                model,
                reason: reason.to_string(),
            };
        }

        RouteDecision {
            model: self.primary_ollama_model(),
            reason: "The preferred exact model is not enabled, so I used the primary local Ollama model.".to_string(),
        }
    }

    /// Get the first enabled model for one provider.
    fn first_enabled_provider(&self, provider: &Provider) -> Option<LanguageModel> {
        self.models
            .iter()
            .find(|model| model.enabled && &model.provider == provider)
            .cloned()
    }

    /// Get the configured primary Ollama model.
    fn primary_ollama_model(&self) -> LanguageModel {
        self.models
            .iter()
            .find(|model| model.provider == Provider::Ollama && model.name == PRIMARY_OLLAMA_MODEL)
            .expect("router always contains primary Ollama model")
            .clone()
    }

    /// Get the configured fast Ollama model name.
    fn fast_ollama_model_name(&self) -> String {
        self.models
            .iter()
            .find(|model| model.provider == Provider::Ollama && model.name != PRIMARY_OLLAMA_MODEL)
            .expect("router always contains fast Ollama model")
            .name
            .clone()
    }
}

impl Router for ModelRouter {
    /// Choose a model for the user's prompt.
    ///
    /// The rule order matters. More specific prompt classes are checked before
    /// broad general-purpose routing.
    fn route(&self, prompt: &str) -> RouteDecision {
        self.route_with_rules(prompt)
    }

    /// Return every model known to the router.
    fn models(&self) -> &[LanguageModel] {
        &self.models
    }
}

/// Return a disabled reason only when the model is disabled.
fn disabled_reason(enabled: bool, reason: String) -> Option<String> {
    if enabled { None } else { Some(reason) }
}

/// Simple prompt features used by the router.
///
/// This struct keeps classification readable. Each boolean answers one direct
/// question about the prompt, and routing decisions combine those booleans in a
/// predictable order.
#[derive(Debug, Default)]
struct PromptProfile {
    /// The prompt explicitly asks to stay local/private/offline.
    needs_privacy: bool,

    /// The prompt is short or asks for a quick/simple answer.
    is_simple: bool,

    /// The prompt asks about current events, latest information, news, or public
    /// discourse.
    needs_current_context: bool,

    /// The prompt is about coding, debugging, architecture, planning, analysis,
    /// or other careful reasoning.
    needs_deep_reasoning_or_code: bool,

    /// The prompt is broad, creative, or general enough for the balanced cloud
    /// model to be a good first choice.
    is_creative_or_general_cloud: bool,
}

impl PromptProfile {
    /// Build a prompt profile from plain text.
    fn from_prompt(prompt: &str) -> Self {
        let prompt_lowercase = prompt.to_lowercase();
        let word_count = prompt.split_whitespace().count();

        Self {
            needs_privacy: contains_any(&prompt_lowercase, LOCAL_ONLY_KEYWORDS)
                || contains_sensitive_keyword(&prompt_lowercase),
            is_simple: word_count <= 20
                || contains_any(
                    &prompt_lowercase,
                    &[
                        "quick",
                        "brief",
                        "one line",
                        "one-line",
                        "short answer",
                        "simple",
                    ],
                ),
            needs_current_context: contains_any(
                &prompt_lowercase,
                &[
                    "latest",
                    "today",
                    "right now",
                    "current",
                    "news",
                    "trending",
                    "recent",
                    "this week",
                    "public debate",
                    "x/twitter",
                ],
            ),
            needs_deep_reasoning_or_code: contains_any(
                &prompt_lowercase,
                &[
                    "code",
                    "rust",
                    "python",
                    "javascript",
                    "typescript",
                    "debug",
                    "error",
                    "stack trace",
                    "architecture",
                    "refactor",
                    "plan",
                    "analyze",
                    "reason",
                    "tradeoff",
                    "security",
                ],
            ) || word_count >= 120,
            is_creative_or_general_cloud: contains_any(
                &prompt_lowercase,
                &[
                    "write",
                    "draft",
                    "rewrite",
                    "brainstorm",
                    "summarize",
                    "explain",
                    "email",
                    "story",
                ],
            ),
        }
    }
}

/// True when any keyword appears in the prompt.
fn contains_any(prompt: &str, keywords: &[&str]) -> bool {
    keywords.iter().any(|keyword| prompt.contains(keyword))
}

/// True when the prompt appears to include sensitive personal, financial, legal,
/// medical, or credential material.
fn contains_sensitive_keyword(prompt: &str) -> bool {
    contains_any(prompt, SENSITIVE_PHRASES)
        || prompt
            .split(|character: char| !character.is_ascii_alphanumeric())
            .filter(|word| !word.is_empty())
            .any(|word| SENSITIVE_WORDS.contains(&word))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn enabled_model(provider: Provider, name: &str) -> LanguageModel {
        match provider {
            Provider::Ollama => LanguageModel::ollama(name, &["test"]),
            Provider::Anthropic => LanguageModel::anthropic(name, &["test"], true, None),
            Provider::OpenAi => LanguageModel::openai(name, &["test"], true, None),
            Provider::Xai => LanguageModel::xai(name, &["test"], true, None),
        }
    }

    fn disabled_model(provider: Provider, name: &str) -> LanguageModel {
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

    fn router_with_models(models: Vec<LanguageModel>) -> ModelRouter {
        ModelRouter { models }
    }

    #[test]
    fn simple_prompt_chooses_fast_ollama_model() {
        let router = router_with_models(vec![
            enabled_model(Provider::Ollama, PRIMARY_OLLAMA_MODEL),
            enabled_model(Provider::Ollama, DEFAULT_FAST_OLLAMA_MODEL),
        ]);

        let decision = router.route("quick summary");

        assert_eq!(decision.model.name, DEFAULT_FAST_OLLAMA_MODEL);
    }

    #[test]
    fn default_fast_model_uses_installed_llama3_latest_tag() {
        assert_eq!(DEFAULT_FAST_OLLAMA_MODEL, "llama3:latest");
    }

    #[test]
    fn code_prompt_prefers_claude_when_enabled() {
        let router = router_with_models(vec![
            enabled_model(Provider::Ollama, PRIMARY_OLLAMA_MODEL),
            enabled_model(Provider::Ollama, DEFAULT_FAST_OLLAMA_MODEL),
            enabled_model(Provider::Anthropic, anthropic::DEFAULT_ANTHROPIC_MODEL),
            enabled_model(Provider::OpenAi, openai::DEFAULT_OPENAI_MODEL),
        ]);

        let decision = router.route("debug this Rust compile error and explain the fix");

        assert_eq!(decision.model.provider, Provider::Anthropic);
    }

    #[test]
    fn code_prompt_falls_back_when_claude_is_disabled() {
        let router = router_with_models(vec![
            enabled_model(Provider::Ollama, PRIMARY_OLLAMA_MODEL),
            enabled_model(Provider::Ollama, DEFAULT_FAST_OLLAMA_MODEL),
            disabled_model(Provider::Anthropic, anthropic::DEFAULT_ANTHROPIC_MODEL),
            enabled_model(Provider::OpenAi, openai::DEFAULT_OPENAI_MODEL),
        ]);

        let decision = router.route("debug this Rust compile error and explain the fix");

        assert_eq!(decision.model.provider, Provider::OpenAi);
    }

    #[test]
    fn current_context_prompt_prefers_grok_when_enabled() {
        let router = router_with_models(vec![
            enabled_model(Provider::Ollama, PRIMARY_OLLAMA_MODEL),
            enabled_model(Provider::Ollama, DEFAULT_FAST_OLLAMA_MODEL),
            enabled_model(Provider::Xai, xai::DEFAULT_XAI_MODEL),
            enabled_model(Provider::OpenAi, openai::DEFAULT_OPENAI_MODEL),
        ]);

        let decision = router.route("what is the latest public debate around AI policy");

        assert_eq!(decision.model.provider, Provider::Xai);
    }

    #[test]
    fn privacy_prompt_stays_on_primary_ollama() {
        let router = router_with_models(vec![
            enabled_model(Provider::Ollama, PRIMARY_OLLAMA_MODEL),
            enabled_model(Provider::Ollama, DEFAULT_FAST_OLLAMA_MODEL),
            enabled_model(Provider::OpenAi, openai::DEFAULT_OPENAI_MODEL),
        ]);

        let decision = router.route("private local only: summarize this personal note");

        assert_eq!(decision.model.name, PRIMARY_OLLAMA_MODEL);
    }

    #[test]
    fn sensitive_medical_prompt_stays_on_primary_ollama_even_when_cloud_is_enabled() {
        let router = router_with_models(vec![
            enabled_model(Provider::Ollama, PRIMARY_OLLAMA_MODEL),
            enabled_model(Provider::Ollama, DEFAULT_FAST_OLLAMA_MODEL),
            enabled_model(Provider::Anthropic, anthropic::DEFAULT_ANTHROPIC_MODEL),
            enabled_model(Provider::OpenAi, openai::DEFAULT_OPENAI_MODEL),
            enabled_model(Provider::Xai, xai::DEFAULT_XAI_MODEL),
        ]);

        let decision = router.route("summarize these medical records and draft an email");

        assert_eq!(decision.model.provider, Provider::Ollama);
        assert_eq!(decision.model.name, PRIMARY_OLLAMA_MODEL);
    }

    #[test]
    fn sensitive_credential_prompt_stays_on_primary_ollama_even_for_code() {
        let router = router_with_models(vec![
            enabled_model(Provider::Ollama, PRIMARY_OLLAMA_MODEL),
            enabled_model(Provider::Ollama, DEFAULT_FAST_OLLAMA_MODEL),
            enabled_model(Provider::Anthropic, anthropic::DEFAULT_ANTHROPIC_MODEL),
            enabled_model(Provider::OpenAi, openai::DEFAULT_OPENAI_MODEL),
        ]);

        let decision = router.route("debug this Python error; it includes my API key");

        assert_eq!(decision.model.provider, Provider::Ollama);
        assert_eq!(decision.model.name, PRIMARY_OLLAMA_MODEL);
    }
}
