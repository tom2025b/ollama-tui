use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use tokio::sync::mpsc;

use crate::subcommands::tui::app::{App, ModelEvent};
use crate::subcommands::tui::model_task::spawn_model_request;

/// Apply one keyboard event to the app.
///
/// Most keys only change local app state. Pressing Enter may create a
/// `PendingRequest`, which is then sent to a model in a background task.
pub fn handle_key_event(
    key_event: KeyEvent,
    app: &mut App,
    model_event_tx: mpsc::UnboundedSender<ModelEvent>,
) {
    if key_event.kind != KeyEventKind::Press {
        return;
    }

    if app.ui.show_help {
        match key_event.code {
            KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                app.hide_help()
            }
            KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('q') | KeyCode::Char('Q') => {
                app.hide_help()
            }
            _ => {}
        }
        return;
    }

    if app.ui.show_models_picker {
        match key_event.code {
            KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                app.close_models_picker();
            }
            KeyCode::Esc => app.close_models_picker(),
            KeyCode::Up => app.select_previous_model(),
            KeyCode::Down => app.select_next_model(),
            KeyCode::Enter => app.accept_model_selection(),
            _ => {}
        }
        return;
    }

    if !app.command_suggestions().is_empty() {
        match key_event.code {
            KeyCode::Tab => {
                app.accept_suggestion();
                return;
            }
            KeyCode::Up => {
                app.select_previous_suggestion();
                return;
            }
            KeyCode::Down => {
                app.select_next_suggestion();
                return;
            }
            KeyCode::Esc => {
                app.dismiss_suggestions();
                return;
            }
            KeyCode::Enter => {
                app.accept_suggestion();
                if let Some(request) = app.submit_prompt() {
                    spawn_model_request(request, model_event_tx);
                }
                return;
            }
            _ => {}
        }
    }

    const PAGE_LINES: usize = 10;

    match key_event.code {
        KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => app.quit(),
        KeyCode::Esc => app.quit(),
        KeyCode::Backspace => app.backspace(),
        KeyCode::Enter => {
            if let Some(request) = app.submit_prompt() {
                spawn_model_request(request, model_event_tx);
            }
        }
        KeyCode::Char('u') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
            app.clear_input();
        }
        KeyCode::Up => app.scroll_up(1),
        KeyCode::Down => app.scroll_down(1),
        KeyCode::PageUp => app.scroll_up(PAGE_LINES),
        KeyCode::PageDown => app.scroll_down(PAGE_LINES),
        KeyCode::Home => app.scroll_to_top(),
        KeyCode::End => app.scroll_to_bottom(),
        KeyCode::Char('?') if app.session.input.is_empty() => app.toggle_help(),
        KeyCode::Char(character) => app.push_input_char(character),
        _ => {}
    }
}

#[cfg(test)]
mod tests;
