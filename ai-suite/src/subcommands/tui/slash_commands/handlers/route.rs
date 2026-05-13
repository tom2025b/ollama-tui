use crate::subcommands::tui::app::App;
use crate::subcommands::tui::slash_commands::parser::ParsedCommand;

/// `/route <prompt>` (alias `/route test <prompt>`): run the prompt through the
/// router and print the trace. No model is called.
pub fn route_command(app: &mut App, command: &ParsedCommand) {
    let prompt = extract_prompt(command);
    if prompt.is_empty() {
        app.append_local_message(
            command.raw(),
            "Usage: /route <prompt>\n       /route test <prompt>\n\n\
             Runs the router and shows which provider it would pick (no model call)."
                .to_string(),
        );
        app.ui.status = "Provide a prompt to /route.".to_string();
        return;
    }

    match app.explain_route(&prompt) {
        Ok(report) => {
            app.append_local_message(command.raw(), report.format());
            app.ui.status = "Routed prompt (no model called).".to_string();
        }
        Err(error) => {
            app.append_local_message(command.raw(), format!("Routing failed: {error}"));
            app.ui.status = "Routing failed.".to_string();
        }
    }
}

/// Reconstruct the prompt text from the parsed command, stripping an optional
/// leading `test` subcommand and any wrapping quotes.
fn extract_prompt(command: &ParsedCommand) -> String {
    let args = command.args();
    let body = if args
        .first()
        .map(|first| first.eq_ignore_ascii_case("test"))
        .unwrap_or(false)
    {
        &args[1..]
    } else {
        args
    };

    let joined = body.join(" ");
    let trimmed = joined.trim();

    // Strip a single pair of wrapping quotes if present.
    let unquoted = if trimmed.len() >= 2
        && ((trimmed.starts_with('"') && trimmed.ends_with('"'))
            || (trimmed.starts_with('\'') && trimmed.ends_with('\'')))
    {
        &trimmed[1..trimmed.len() - 1]
    } else {
        trimmed
    };

    unquoted.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::subcommands::tui::slash_commands::parser::{ParseResult, parse_slash_command};

    fn parse(input: &str) -> ParsedCommand {
        match parse_slash_command(input) {
            ParseResult::Command(command) => command,
            ParseResult::NotCommand => panic!("expected a parsed command"),
        }
    }

    #[test]
    fn extracts_quoted_prompt_after_test_keyword() {
        let command = parse(r#"/route test "what is 2+2""#);
        assert_eq!(extract_prompt(&command), "what is 2+2");
    }

    #[test]
    fn extracts_unquoted_prompt_without_test_keyword() {
        let command = parse("/route refactor this code please");
        assert_eq!(extract_prompt(&command), "refactor this code please");
    }

    #[test]
    fn empty_when_no_args() {
        let command = parse("/route");
        assert_eq!(extract_prompt(&command), "");
    }
}
