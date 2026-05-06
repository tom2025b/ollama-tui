/// Explicit user instructions that require local/private handling.
pub(super) const LOCAL_ONLY_KEYWORDS: &[&str] =
    &["private", "privacy", "offline", "local only", "do not send"];

/// Sensitive phrases that should never be sent to a cloud provider automatically.
pub(super) const SENSITIVE_PHRASES: &[&str] = &[
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
pub(super) const SENSITIVE_WORDS: &[&str] = &[
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

pub(super) const SIMPLE_KEYWORDS: &[&str] = &[
    "quick",
    "brief",
    "one line",
    "one-line",
    "short answer",
    "simple",
];

pub(super) const CURRENT_CONTEXT_KEYWORDS: &[&str] = &[
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
];

pub(super) const DEEP_REASONING_OR_CODE_KEYWORDS: &[&str] = &[
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
];

pub(super) const CREATIVE_OR_GENERAL_CLOUD_KEYWORDS: &[&str] = &[
    "write",
    "draft",
    "rewrite",
    "brainstorm",
    "summarize",
    "explain",
    "email",
    "story",
];
