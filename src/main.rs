mod anthropic;
mod app;
mod history;
mod llm;
mod ollama;
mod openai;
mod openai_compatible;
mod router;
mod rules;
mod ui;
mod xai;

use std::{io, process::Command, time::Duration};

use anyhow::Result;
use app::{App, ExternalAction, ModelEvent, PendingRequest};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use llm::Provider;
use ratatui::{Terminal, backend::CrosstermBackend};
use tokio::sync::mpsc;

/// Start the terminal app.
///
/// Tokio is used because talking to Ollama is an HTTP request, and HTTP work is
/// naturally asynchronous. The terminal drawing itself stays simple and runs in
/// one loop.
#[tokio::main]
async fn main() -> Result<()> {
    let mut terminal = start_terminal()?;
    let app_result = run_app(&mut terminal).await;
    stop_terminal(&mut terminal)?;

    app_result
}

/// Put the terminal into TUI mode.
///
/// Raw mode lets the app read keys directly. The alternate screen keeps the app
/// from overwriting your normal shell scrollback.
fn start_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;

    Ok(terminal)
}

/// Restore the terminal before exiting.
///
/// This function is kept separate so cleanup is easy to see and easy to reuse
/// if the app grows.
fn stop_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

/// Main application loop.
///
/// Each pass draws the UI, checks for completed model work, and handles any
/// keyboard input. The loop exits when `app.should_quit` becomes true.
async fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    let mut app = App::new();
    let (model_event_tx, mut model_event_rx) = mpsc::unbounded_channel::<ModelEvent>();

    while !app.should_quit {
        while let Ok(event) = model_event_rx.try_recv() {
            app.handle_model_event(event);
        }

        app.tick();
        terminal.draw(|frame| ui::draw(frame, &app))?;

        if event::poll(Duration::from_millis(50))?
            && let Event::Key(key_event) = event::read()?
        {
            handle_key_event(key_event, &mut app, model_event_tx.clone());
        }

        if let Some(action) = app.take_external_action() {
            run_external_action(terminal, &mut app, action)?;
        }
    }

    Ok(())
}

/// Run an external command that cannot happen while the terminal is in TUI mode.
fn run_external_action(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    action: ExternalAction,
) -> Result<()> {
    match action {
        ExternalAction::EditRules { target, path } => {
            suspend_terminal(terminal)?;
            let editor_result = Command::new("nano").arg(&path).status();
            resume_terminal(terminal)?;

            let editor_result = match editor_result {
                Ok(status) if status.success() => Ok(()),
                Ok(status) => Err(format!("nano exited with status {status}")),
                Err(error) => Err(format!("failed to launch nano: {error}")),
            };

            app.complete_rules_edit(target, path, editor_result);
        }
    }

    Ok(())
}

/// Temporarily restore the normal terminal before launching nano.
fn suspend_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

/// Return to TUI mode after an external command exits.
fn resume_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    enable_raw_mode()?;
    execute!(terminal.backend_mut(), EnterAlternateScreen)?;
    terminal.clear()?;

    Ok(())
}

/// Apply one keyboard event to the app.
///
/// Most keys only change local app state. Pressing Enter may create a
/// `PendingRequest`, which is then sent to Ollama in a background task.
fn handle_key_event(
    key_event: KeyEvent,
    app: &mut App,
    model_event_tx: mpsc::UnboundedSender<ModelEvent>,
) {
    if key_event.kind != KeyEventKind::Press {
        return;
    }

    if app.show_help {
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

    // The /models picker is modal: while it is visible, all keys are consumed
    // by the picker so navigation keys do not also scroll history or quit the
    // app. Ctrl-C still closes the overlay rather than killing the process,
    // matching how Esc behaves on the help screen.
    if app.show_models_picker {
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

    // The autocomplete popup intercepts navigation keys when it is visible so
    // the user can pick a suggestion without those keys also affecting history
    // scroll, the input buffer, or the quit shortcut. Anything not listed here
    // (printable characters, Backspace, Ctrl-C, etc.) falls through to the
    // normal handling below.
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
                // Accept the highlighted command, then submit it. The accepted
                // input ends with a space which `submit_prompt` trims, so the
                // local command dispatcher receives the bare command string.
                app.accept_suggestion();
                if let Some(request) = app.submit_prompt() {
                    spawn_model_request(request, model_event_tx);
                }
                return;
            }
            _ => {}
        }
    }

    // Page size for PageUp/PageDown — a reasonable default that works
    // well across common terminal heights (24–50 rows).
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
        // Scroll the chat history line-by-line.
        KeyCode::Up => app.scroll_up(1),
        KeyCode::Down => app.scroll_down(1),
        // Scroll a full page at a time for faster navigation.
        KeyCode::PageUp => app.scroll_up(PAGE_LINES),
        KeyCode::PageDown => app.scroll_down(PAGE_LINES),
        // Jump to the very top or bottom of the history.
        KeyCode::Home => app.scroll_to_top(),
        KeyCode::End => app.scroll_to_bottom(),
        KeyCode::Char('?') if app.input.is_empty() => app.toggle_help(),
        KeyCode::Char(character) => app.push_input_char(character),
        _ => {}
    }
}

/// Run the selected model without blocking the TUI.
///
/// The UI loop should keep drawing while the model is thinking. Spawning a task
/// gives the selected backend time to answer while the terminal stays
/// responsive.
fn spawn_model_request(request: PendingRequest, model_event_tx: mpsc::UnboundedSender<ModelEvent>) {
    tokio::spawn(async move {
        let selected_model = request.route.model.clone();
        let provider_label = selected_model.provider.label();

        let stream_result = match &selected_model.provider {
            Provider::Ollama => {
                ollama::stream(
                    &selected_model.name,
                    &request.context,
                    &request.prompt,
                    |token| {
                        let _ = model_event_tx.send(ModelEvent::Token(token));
                    },
                )
                .await
            }
            Provider::Anthropic => {
                anthropic::stream(
                    &selected_model.name,
                    &request.context,
                    &request.prompt,
                    |token| {
                        let _ = model_event_tx.send(ModelEvent::Token(token));
                    },
                )
                .await
            }
            Provider::OpenAi => {
                openai::stream(
                    &selected_model.name,
                    &request.context,
                    &request.prompt,
                    |token| {
                        let _ = model_event_tx.send(ModelEvent::Token(token));
                    },
                )
                .await
            }
            Provider::Xai => {
                xai::stream(
                    &selected_model.name,
                    &request.context,
                    &request.prompt,
                    |token| {
                        let _ = model_event_tx.send(ModelEvent::Token(token));
                    },
                )
                .await
            }
        };

        let event = match stream_result {
            Ok(_) => ModelEvent::Finished,
            Err(error) => ModelEvent::Failed(format!("{provider_label} request failed: {error:#}")),
        };

        // If the receiver is gone, the app is already shutting down. There is
        // nothing useful left to do with the result.
        let _ = model_event_tx.send(event);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn model_event_sender() -> mpsc::UnboundedSender<ModelEvent> {
        let (sender, _receiver) = mpsc::unbounded_channel();
        sender
    }

    #[test]
    fn q_closes_help_without_quitting() {
        let mut app = App::new();
        app.show_help = true;

        handle_key_event(
            KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE),
            &mut app,
            model_event_sender(),
        );

        assert!(!app.show_help);
        assert!(!app.should_quit);
    }

    #[test]
    fn ctrl_c_closes_help_without_quitting() {
        let mut app = App::new();
        app.show_help = true;

        handle_key_event(
            KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
            &mut app,
            model_event_sender(),
        );

        assert!(!app.show_help);
        assert!(!app.should_quit);
    }

    #[test]
    fn question_mark_release_does_not_reopen_help() {
        let mut app = App::new();
        app.show_help = true;

        handle_key_event(
            KeyEvent::new(KeyCode::Char('?'), KeyModifiers::NONE),
            &mut app,
            model_event_sender(),
        );
        handle_key_event(
            KeyEvent::new_with_kind(
                KeyCode::Char('?'),
                KeyModifiers::NONE,
                KeyEventKind::Release,
            ),
            &mut app,
            model_event_sender(),
        );

        assert!(!app.show_help);
        assert!(!app.should_quit);
    }

    #[test]
    fn ctrl_c_release_after_closing_help_does_not_quit() {
        let mut app = App::new();
        app.show_help = true;

        handle_key_event(
            KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
            &mut app,
            model_event_sender(),
        );
        handle_key_event(
            KeyEvent::new_with_kind(
                KeyCode::Char('c'),
                KeyModifiers::CONTROL,
                KeyEventKind::Release,
            ),
            &mut app,
            model_event_sender(),
        );

        assert!(!app.show_help);
        assert!(!app.should_quit);
    }

    #[test]
    fn tab_with_suggestions_accepts_selection() {
        let mut app = App::new();
        app.input = "/he".to_string();

        handle_key_event(
            KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE),
            &mut app,
            model_event_sender(),
        );

        assert_eq!(app.input, "/help ");
    }

    #[test]
    fn tab_without_suggestions_does_nothing() {
        let mut app = App::new();
        app.input = "hello".to_string();

        handle_key_event(
            KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE),
            &mut app,
            model_event_sender(),
        );

        assert_eq!(app.input, "hello");
    }

    #[test]
    fn down_with_suggestions_moves_highlight() {
        let mut app = App::new();
        app.input = "/m".to_string();

        handle_key_event(
            KeyEvent::new(KeyCode::Down, KeyModifiers::NONE),
            &mut app,
            model_event_sender(),
        );

        assert_eq!(app.suggestion_index(), 1);
    }

    #[test]
    fn esc_with_suggestions_dismisses_without_quitting() {
        let mut app = App::new();
        app.input = "/".to_string();

        handle_key_event(
            KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE),
            &mut app,
            model_event_sender(),
        );

        assert!(!app.should_quit);
        assert!(app.command_suggestions().is_empty());
    }

    #[test]
    fn enter_with_suggestions_runs_local_command() {
        let mut app = App::new();
        app.input = "/he".to_string();

        handle_key_event(
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
            &mut app,
            model_event_sender(),
        );

        // /help is the first match for "/he"; submitting it opens the help
        // overlay and clears the input.
        assert!(app.show_help);
        assert!(app.input.is_empty());
    }

    #[test]
    fn models_picker_consumes_navigation_keys() {
        let mut app = App::new();
        app.open_models_picker();

        // Down should move the picker highlight, not scroll chat history.
        let starting_index = app.models_picker_index();
        handle_key_event(
            KeyEvent::new(KeyCode::Down, KeyModifiers::NONE),
            &mut app,
            model_event_sender(),
        );
        assert_ne!(app.models_picker_index(), starting_index);
        assert_eq!(app.scroll_offset, 0);
    }

    #[test]
    fn esc_in_models_picker_closes_overlay_without_quitting() {
        let mut app = App::new();
        app.open_models_picker();

        handle_key_event(
            KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE),
            &mut app,
            model_event_sender(),
        );

        assert!(!app.show_models_picker);
        assert!(!app.should_quit);
    }

    #[test]
    fn enter_in_models_picker_pins_selection() {
        let mut app = App::new();
        app.open_models_picker();
        let expected = app.pickable_models()[0].clone();

        // Move to the first real model (skip the "Auto" row at index 0).
        handle_key_event(
            KeyEvent::new(KeyCode::Down, KeyModifiers::NONE),
            &mut app,
            model_event_sender(),
        );
        handle_key_event(
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
            &mut app,
            model_event_sender(),
        );

        assert!(!app.show_models_picker);
        assert!(app.is_pinned(&expected));
    }
}
