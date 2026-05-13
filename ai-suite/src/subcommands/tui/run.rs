use std::time::Duration;

use crossterm::event::{self, Event};
use tokio::sync::mpsc;

use crate::runtime::Runtime;
use crate::subcommands::tui::app::{App, ModelEvent};
use crate::subcommands::tui::external::run_external_action;
use crate::subcommands::tui::input::handle_key_event;
use crate::subcommands::tui::terminal::{AppTerminal, start_terminal, stop_terminal};
use crate::{Error, Result};

/// Start the terminal app.
///
/// Tokio is used because talking to models is HTTP work, while terminal input
/// and drawing stay in one small event loop.
pub async fn run(runtime: &Runtime) -> Result<()> {
    let mut terminal = start_terminal()?;
    let app_result = run_app(&mut terminal, runtime).await;
    stop_terminal(&mut terminal)?;

    app_result
}

/// Main application loop.
async fn run_app(terminal: &mut AppTerminal, runtime: &Runtime) -> Result<()> {
    let mut app = App::with_runtime(runtime.clone());
    let (model_event_tx, mut model_event_rx) = mpsc::unbounded_channel::<ModelEvent>();

    while !app.should_quit {
        while let Ok(event) = model_event_rx.try_recv() {
            app.handle_model_event(event);
        }

        app.tick();
        terminal
            .draw(|frame| crate::subcommands::tui::ui::draw(frame, &app))
            .map_err(|source| Error::io_operation("draw terminal frame", source))?;

        if event::poll(Duration::from_millis(50))
            .map_err(|source| Error::io_operation("poll terminal events", source))?
            && let Event::Key(key_event) = event::read()
                .map_err(|source| Error::io_operation("read terminal event", source))?
        {
            handle_key_event(key_event, &mut app, model_event_tx.clone());
        }

        while let Some(action) = app.take_external_action() {
            run_external_action(terminal, &mut app, action)?;
        }
    }

    Ok(())
}
