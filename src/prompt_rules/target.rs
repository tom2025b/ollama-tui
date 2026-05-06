/// Which rules file the user wants to edit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RulesTarget {
    /// User-wide rules loaded from `~/.config/ollama-me/rules.md`.
    Global,
    /// Project-local rules loaded from `<project-root>/.ollama-me/rules.md`.
    Project,
}

impl RulesTarget {
    /// Human-readable label used in command output.
    pub fn label(self) -> &'static str {
        match self {
            RulesTarget::Global => "global rules",
            RulesTarget::Project => "project rules",
        }
    }
}
