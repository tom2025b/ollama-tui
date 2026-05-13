use std::io;

use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

use crate::{Error, Result};

pub type AppTerminal = Terminal<CrosstermBackend<io::Stdout>>;

/// Put the terminal into TUI mode.
pub fn start_terminal() -> Result<AppTerminal> {
    enable_raw_mode().map_err(|source| Error::io_operation("enable terminal raw mode", source))?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)
        .map_err(|source| Error::io_operation("enter alternate screen", source))?;

    let terminal = Terminal::new(CrosstermBackend::new(stdout))
        .map_err(|source| Error::io_operation("initialize terminal backend", source))?;

    Ok(terminal)
}

/// Restore the terminal before exiting.
pub fn stop_terminal(terminal: &mut AppTerminal) -> Result<()> {
    disable_raw_mode()
        .map_err(|source| Error::io_operation("disable terminal raw mode", source))?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)
        .map_err(|source| Error::io_operation("leave alternate screen", source))?;
    terminal
        .show_cursor()
        .map_err(|source| Error::io_operation("show terminal cursor", source))?;

    Ok(())
}

/// Temporarily restore the normal terminal before launching an external tool.
pub fn suspend_terminal(terminal: &mut AppTerminal) -> Result<()> {
    disable_raw_mode()
        .map_err(|source| Error::io_operation("disable terminal raw mode", source))?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)
        .map_err(|source| Error::io_operation("leave alternate screen", source))?;
    terminal
        .show_cursor()
        .map_err(|source| Error::io_operation("show terminal cursor", source))?;

    Ok(())
}

/// Return to TUI mode after an external command exits.
pub fn resume_terminal(terminal: &mut AppTerminal) -> Result<()> {
    enable_raw_mode().map_err(|source| Error::io_operation("enable terminal raw mode", source))?;
    execute!(terminal.backend_mut(), EnterAlternateScreen)
        .map_err(|source| Error::io_operation("enter alternate screen", source))?;
    terminal
        .clear()
        .map_err(|source| Error::io_operation("clear terminal", source))?;

    Ok(())
}
