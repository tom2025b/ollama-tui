use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event};
use tokio::sync::mpsc;

use crate::app::{App, ModelEvent};
use crate::external::run_external_action;
use crate::keys::handle_key_event;
use crate::terminal::{AppTerminal, start_terminal, stop_terminal};

/// Start the terminal app.
///
/// Tokio is used because talking to models is HTTP work, while terminal input
/// and drawing stay in one small event loop.
pub async fn run() -> Result<()> {
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
        terminal.draw(|frame| crate::ui::draw(frame, &app))?;

        if event::poll(Duration::from_millis(50))?
            && let Event::Key(key_event) = event::read()?
        {
            handle_key_event(key_event, &mut app, model_event_tx.clone());
        }

        while let Some(action) = app.take_external_action() {
            run_external_action(terminal, &mut app, action)?;
        }
    }

    Ok(())
}
