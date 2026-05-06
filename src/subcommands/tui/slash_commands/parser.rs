/// Parsed representation of a slash command entered in the prompt box.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParsedCommand {
    raw: String,
    name: String,
    args: Vec<String>,
}

impl ParsedCommand {
    /// The original command text, trimmed of leading and trailing whitespace.
    pub fn raw(&self) -> &str {
        &self.raw
    }

    /// The normalized command name, including the leading slash.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Command arguments in their original case.
    pub fn args(&self) -> &[String] {
        &self.args
    }
}

/// Result of parsing prompt text as a slash command.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParseResult {
    /// The prompt does not begin with a slash after trimming.
    NotCommand,

    /// The prompt is a syntactically valid slash-command input.
    Command(ParsedCommand),
}

/// Parse prompt text into a slash-command shape.
///
/// This parser intentionally stays lightweight: it separates the command name
/// from whitespace-delimited arguments, lowercases only the command name, and
/// leaves command-specific argument validation to the execution stage.
pub fn parse_slash_command(input: &str) -> ParseResult {
    let raw = input.trim();
    if !raw.starts_with('/') {
        return ParseResult::NotCommand;
    }

    let mut parts = raw.split_whitespace();
    let name = parts.next().unwrap_or_default().to_ascii_lowercase();
    let args = parts.map(str::to_string).collect();

    ParseResult::Command(ParsedCommand {
        raw: raw.to_string(),
        name,
        args,
    })
}

/// Return the current autocomplete prefix when the input is eligible for slash
/// command suggestions.
pub fn suggestion_prefix(input: &str) -> Option<&str> {
    if !input.starts_with('/') {
        return None;
    }
    if input.chars().any(char::is_whitespace) {
        return None;
    }

    Some(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_ignores_non_command_input() {
        assert_eq!(parse_slash_command("hello"), ParseResult::NotCommand);
    }

    #[test]
    fn parser_normalizes_command_name_and_preserves_args() {
        let parsed = match parse_slash_command("  /RULES Show Project  ") {
            ParseResult::Command(command) => command,
            ParseResult::NotCommand => panic!("expected command"),
        };

        assert_eq!(parsed.raw(), "/RULES Show Project");
        assert_eq!(parsed.name(), "/rules");
        assert_eq!(parsed.args(), &["Show".to_string(), "Project".to_string()]);
    }

    #[test]
    fn suggestion_prefix_requires_slash_without_whitespace() {
        assert_eq!(suggestion_prefix("/ru"), Some("/ru"));
        assert_eq!(suggestion_prefix("hello"), None);
        assert_eq!(suggestion_prefix("/rules show"), None);
    }
}
