mod anthropic;
mod app;
mod command;
mod external;
mod history;
mod keys;
mod llm;
mod model_task;
mod ollama;
mod openai;
mod openai_compatible;
mod router;
mod rules;
mod terminal;
mod ui;
mod xai;

use std::time::Duration;

use anyhow::Result;
use app::{App, ModelEvent};
use crossterm::event::{self, Event};
use external::run_external_action;
use keys::handle_key_event;
use terminal::{AppTerminal, start_terminal, stop_terminal};
use tokio::sync::mpsc;

/// Start the terminal app.
///
/// Tokio is used because talking to models is HTTP work, while terminal input
/// and drawing stay in one small event loop.
#[tokio::main]
async fn main() -> Result<()> {
    let mut terminal = start_terminal()?;
    let app_result = run_app(&mut terminal).await;
    stop_terminal(&mut terminal)?;

    app_result
}

/// Main application loop.
async fn run_app(terminal: &mut AppTerminal) -> Result<()> {
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
