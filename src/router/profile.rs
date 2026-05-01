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
