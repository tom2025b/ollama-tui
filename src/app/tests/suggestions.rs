use super::super::App;
use crate::command::CommandRegistry;

#[test]
fn command_suggestions_empty_for_normal_input() {
    let mut app = App::new();
    app.session.input = "hello".to_string();
    assert!(app.command_suggestions().is_empty());
}

#[test]
fn command_suggestions_show_all_commands_for_slash_alone() {
    let mut app = App::new();
    app.session.input = "/".to_string();
    let suggestions = app.command_suggestions();
    assert_eq!(suggestions, CommandRegistry::default().suggestions("/"));
}

#[test]
fn command_help_entries_match_current_commands() {
    let app = App::new();
    let names = app
        .command_help_entries()
        .into_iter()
        .map(|entry| entry.name)
        .collect::<Vec<_>>();
    let registry_names = CommandRegistry::default()
        .help_entries()
        .into_iter()
        .map(|entry| entry.name)
        .collect::<Vec<_>>();

    assert_eq!(names, registry_names);
    assert!(names.contains(&"/theme"));
    assert!(names.contains(&"/context"));
    assert!(names.contains(&"/summary"));
    assert!(names.contains(&"/bookmark"));
    assert!(names.contains(&"/memory"));
    assert!(names.contains(&"/export"));
    assert!(names.contains(&"/tokens"));
    assert!(!names.contains(&"/models"));
    assert!(!names.contains(&"/backends"));
}

#[test]
fn command_suggestions_filter_by_prefix() {
    let mut app = App::new();
    app.session.input = "/mo".to_string();
    let names: Vec<&str> = app
        .command_suggestions()
        .into_iter()
        .map(|suggestion| suggestion.name)
        .collect();
    assert_eq!(names, vec!["/model"]);
}

#[test]
fn command_suggestions_hidden_after_whitespace() {
    let mut app = App::new();
    app.session.input = "/rules ".to_string();
    assert!(app.command_suggestions().is_empty());
}

#[test]
fn accept_suggestion_replaces_input_and_appends_space() {
    let mut app = App::new();
    app.session.input = "/h".to_string();
    let accepted = app.accept_suggestion();
    assert!(accepted);
    assert_eq!(app.session.input, "/help ");
    assert!(app.command_suggestions().is_empty());
}

#[test]
fn select_next_wraps_around_match_list() {
    let mut app = App::new();
    app.session.input = "/".to_string();
    let last_index = app.command_suggestions().len() - 1;

    app.select_next_suggestion();
    assert_eq!(app.suggestion_index(), 1);
    for _ in 0..last_index {
        app.select_next_suggestion();
    }
    assert_eq!(app.suggestion_index(), 0);
}

#[test]
fn select_previous_wraps_to_end() {
    let mut app = App::new();
    app.session.input = "/".to_string();
    let last_index = app.command_suggestions().len() - 1;

    app.select_previous_suggestion();
    assert_eq!(app.suggestion_index(), last_index);
}

#[test]
fn select_next_is_noop_without_suggestions() {
    let mut app = App::new();
    app.session.input = "hello".to_string();
    app.select_next_suggestion();
    assert_eq!(app.suggestion_index(), 0);
}

#[test]
fn dismiss_suggestions_hides_popup_until_input_changes() {
    let mut app = App::new();
    app.session.input = "/".to_string();
    assert!(!app.command_suggestions().is_empty());

    app.dismiss_suggestions();
    assert!(app.command_suggestions().is_empty());

    app.push_input_char('h');
    assert!(!app.command_suggestions().is_empty());
}

#[test]
fn suggestion_index_clamps_when_match_list_shrinks() {
    let mut app = App::new();
    app.session.input = "/".to_string();
    app.select_next_suggestion();
    assert_eq!(app.suggestion_index(), 1);

    app.session.input = "/model".to_string();
    assert_eq!(app.command_suggestions().len(), 1);
    app.session.input = "/models".to_string();
    assert!(app.command_suggestions().is_empty());
    assert_eq!(app.suggestion_index(), 0);
}
