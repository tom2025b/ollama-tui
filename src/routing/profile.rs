mod keywords;

use keywords::{
    CREATIVE_OR_GENERAL_CLOUD_KEYWORDS, CURRENT_CONTEXT_KEYWORDS, DEEP_REASONING_OR_CODE_KEYWORDS,
    LOCAL_ONLY_KEYWORDS, SENSITIVE_PHRASES, SENSITIVE_WORDS, SIMPLE_KEYWORDS,
};

/// Simple prompt features used by the router.
#[derive(Debug, Default)]
pub(super) struct PromptProfile {
    pub(super) needs_privacy: bool,
    pub(super) is_simple: bool,
    pub(super) needs_current_context: bool,
    pub(super) needs_deep_reasoning_or_code: bool,
    pub(super) is_creative_or_general_cloud: bool,
}

impl PromptProfile {
    /// Build a prompt profile from plain text.
    pub(super) fn from_prompt(prompt: &str) -> Self {
        let prompt_lowercase = prompt.to_lowercase();
        let word_count = prompt.split_whitespace().count();

        Self {
            needs_privacy: contains_any(&prompt_lowercase, LOCAL_ONLY_KEYWORDS)
                || contains_sensitive_keyword(&prompt_lowercase),
            is_simple: word_count <= 20 || contains_any(&prompt_lowercase, SIMPLE_KEYWORDS),
            needs_current_context: contains_any(&prompt_lowercase, CURRENT_CONTEXT_KEYWORDS),
            needs_deep_reasoning_or_code: contains_any(
                &prompt_lowercase,
                DEEP_REASONING_OR_CODE_KEYWORDS,
            ) || word_count >= 120,
            is_creative_or_general_cloud: contains_any(
                &prompt_lowercase,
                CREATIVE_OR_GENERAL_CLOUD_KEYWORDS,
            ),
        }
    }
}

fn contains_any(prompt: &str, keywords: &[&str]) -> bool {
    keywords.iter().any(|keyword| prompt.contains(keyword))
}

fn contains_sensitive_keyword(prompt: &str) -> bool {
    contains_any(prompt, SENSITIVE_PHRASES)
        || prompt
            .split(|character: char| !character.is_ascii_alphanumeric())
            .filter(|word| !word.is_empty())
            .any(|word| SENSITIVE_WORDS.contains(&word))
}
