mod keywords;

use keywords::SIMPLE_KEYWORDS;

/// Simple prompt features used by the router.
#[derive(Debug, Default)]
pub(super) struct PromptProfile {
    pub(super) is_simple: bool,
}

impl PromptProfile {
    /// Build a prompt profile from plain text.
    pub(super) fn from_prompt(prompt: &str) -> Self {
        let prompt_lowercase = prompt.to_lowercase();
        let word_count = prompt.split_whitespace().count();

        Self {
            is_simple: word_count <= 20 || contains_any(&prompt_lowercase, SIMPLE_KEYWORDS),
        }
    }
}

fn contains_any(prompt: &str, keywords: &[&str]) -> bool {
    keywords.iter().any(|keyword| prompt.contains(keyword))
}
