mod anthropic;
mod app;
mod llm;
mod ollama;
mod openai;
mod openai_compatible;
mod router;
mod ui;
mod xai;

use std::{io, time::Duration};

use anyhow::Result;
use app::{App, ModelEvent, PendingRequest};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
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
    }

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
    if app.show_help {
        match key_event.code {
            KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => app.quit(),
            KeyCode::Esc | KeyCode::Char('?') => app.hide_help(),
            _ => {}
        }
        return;
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
