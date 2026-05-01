use super::super::App;
use crate::command::CommandRegistry;

#[test]
fn command_suggestions_empty_for_normal_input() {
    let mut app = App::new();
    app.input = "hello".to_string();
    assert!(app.command_suggestions().is_empty());
}

#[test]
fn command_suggestions_show_all_commands_for_slash_alone() {
    let mut app = App::new();
    app.input = "/".to_string();
    let suggestions = app.command_suggestions();
    assert_eq!(suggestions, CommandRegistry::default().suggestions("/"));
}

#[test]
fn command_suggestions_filter_by_prefix() {
    let mut app = App::new();
    app.input = "/mo".to_string();
    let names: Vec<&str> = app
        .command_suggestions()
        .into_iter()
        .map(|suggestion| suggestion.name)
        .collect();
    assert_eq!(names, vec!["/model", "/models"]);
}

#[test]
fn command_suggestions_hidden_after_whitespace() {
    let mut app = App::new();
    app.input = "/rules ".to_string();
    assert!(app.command_suggestions().is_empty());
}

#[test]
fn accept_suggestion_replaces_input_and_appends_space() {
    let mut app = App::new();
    app.input = "/h".to_string();
    let accepted = app.accept_suggestion();
    assert!(accepted);
    assert_eq!(app.input, "/help ");
    assert!(app.command_suggestions().is_empty());
}

#[test]
fn select_next_wraps_around_match_list() {
    let mut app = App::new();
    app.input = "/mo".to_string();
    app.select_next_suggestion();
    assert_eq!(app.suggestion_index(), 1);
    app.select_next_suggestion();
    assert_eq!(app.suggestion_index(), 0);
}

#[test]
fn select_previous_wraps_to_end() {
    let mut app = App::new();
    app.input = "/mo".to_string();
    app.select_previous_suggestion();
    assert_eq!(app.suggestion_index(), 1);
}

#[test]
fn select_next_is_noop_without_suggestions() {
    let mut app = App::new();
    app.input = "hello".to_string();
    app.select_next_suggestion();
    assert_eq!(app.suggestion_index(), 0);
}

#[test]
fn dismiss_suggestions_hides_popup_until_input_changes() {
    let mut app = App::new();
    app.input = "/".to_string();
    assert!(!app.command_suggestions().is_empty());

    app.dismiss_suggestions();
    assert!(app.command_suggestions().is_empty());

    app.push_input_char('h');
    assert!(!app.command_suggestions().is_empty());
}

#[test]
fn suggestion_index_clamps_when_match_list_shrinks() {
    let mut app = App::new();
    app.input = "/mo".to_string();
    app.select_next_suggestion();
    assert_eq!(app.suggestion_index(), 1);

    app.input = "/model".to_string();
    assert_eq!(app.command_suggestions().len(), 2);
    app.input = "/models".to_string();
    assert_eq!(app.command_suggestions().len(), 1);
    assert_eq!(app.suggestion_index(), 0);
}
