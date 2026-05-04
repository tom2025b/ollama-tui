use super::*;
use crate::command::parser::{ParseResult, parse_slash_command};

fn parse(input: &str) -> ParsedCommand {
    match parse_slash_command(input) {
        ParseResult::Command(command) => command,
        ParseResult::NotCommand => panic!("expected command"),
    }
}

fn assert_unique(names: &[&str]) {
    for (index, name) in names.iter().enumerate() {
        assert!(
            !names[..index].contains(name),
            "duplicate command name: {name}"
        );
    }
}

#[test]
fn registry_resolves_aliases_to_same_command_id() {
    let registry = CommandRegistry::default();

    assert_eq!(
        registry.resolve(&parse("/backend")).unwrap().id,
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

    assert_eq!(names, vec!["/model"]);
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
fn registry_suggests_each_command_once() {
    let registry = CommandRegistry::default();
    let suggestions = registry.suggestions("/");
    let help_names = registry
        .help_entries()
        .into_iter()
        .map(|entry| entry.name)
        .collect::<Vec<_>>();
    let names = suggestions
        .iter()
        .map(|suggestion| suggestion.name)
        .collect::<Vec<_>>();

    assert_eq!(names, help_names);
    assert_unique(&names);
    assert!(!names.contains(&"/models"));
    assert!(!names.contains(&"/backends"));
}

#[test]
fn registry_does_not_resolve_removed_plural_aliases() {
    let registry = CommandRegistry::default();

    assert!(registry.resolve(&parse("/models")).is_none());
    assert!(registry.resolve(&parse("/backends")).is_none());
}

#[test]
fn registry_help_uses_current_command_definitions() {
    let registry = CommandRegistry::default();
    let names = registry
        .help_entries()
        .into_iter()
        .map(|entry| entry.name)
        .collect::<Vec<_>>();

    assert_eq!(
        names,
        vec![
            "/clear",
            "/model",
            "/backend",
            "/cost",
            "/rules",
            "/help",
            "/history",
            "/quit",
            "/context",
            "/tokens",
            "/bookmark",
            "/memory",
            "/summary",
            "/export",
            "/theme",
            "/resize",
        ]
    );
    assert_unique(&names);
}

#[test]
fn registry_builds_available_commands_from_definitions() {
    let registry = CommandRegistry::default();

    assert_eq!(
        registry.available_commands(),
        "/clear, /model, /backend, /cost, /rules, /help, /history, /quit, /context, /tokens, /bookmark, /memory, /summary, /export, /theme, /resize"
    );
}
