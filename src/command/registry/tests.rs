use super::*;
use crate::command::parser::{ParseResult, parse_slash_command};

fn parse(input: &str) -> ParsedCommand {
    match parse_slash_command(input) {
        ParseResult::Command(command) => command,
        ParseResult::NotCommand => panic!("expected command"),
    }
}

#[test]
fn registry_resolves_aliases_to_same_command_id() {
    let registry = CommandRegistry::default();

    assert_eq!(
        registry.resolve(&parse("/backend")).unwrap().id,
        CommandId::Backend
    );
    assert_eq!(
        registry.resolve(&parse("/backends")).unwrap().id,
        CommandId::Backend
    );
    assert_eq!(registry.resolve(&parse("/q")).unwrap().id, CommandId::Quit);
}

#[test]
fn registry_filters_visible_suggestions_by_prefix() {
    let registry = CommandRegistry::default();
    let names = registry
        .suggestions("/mo")
        .into_iter()
        .map(|suggestion| suggestion.name)
        .collect::<Vec<_>>();

    assert_eq!(names, vec!["/model", "/models"]);
}

#[test]
fn registry_hides_non_visible_aliases_from_suggestions() {
    let registry = CommandRegistry::default();
    let names = registry
        .suggestions("/")
        .into_iter()
        .map(|suggestion| suggestion.name)
        .collect::<Vec<_>>();

    assert!(!names.contains(&"/q"));
}

#[test]
fn registry_builds_available_commands_from_definitions() {
    let registry = CommandRegistry::default();

    assert_eq!(
        registry.available_commands(),
        "/clear, /models, /backends, /rules, /help, /history, /quit, /context, /tokens, /bookmark, /memory, /summary, /export, /theme, /resize, /voice"
    );
}
