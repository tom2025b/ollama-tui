use std::io;

use anyhow::Context;
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

use crate::Result;

pub type AppTerminal = Terminal<CrosstermBackend<io::Stdout>>;

/// Put the terminal into TUI mode.
pub fn start_terminal() -> Result<AppTerminal> {
    enable_raw_mode().context("failed to enable terminal raw mode")?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).context("failed to enter alternate screen")?;

    let terminal = Terminal::new(CrosstermBackend::new(stdout))
        .context("failed to initialize terminal backend")?;

    Ok(terminal)
}

/// Restore the terminal before exiting.
pub fn stop_terminal(terminal: &mut AppTerminal) -> Result<()> {
    disable_raw_mode().context("failed to disable terminal raw mode")?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)
        .context("failed to leave alternate screen")?;
    terminal
        .show_cursor()
        .context("failed to show terminal cursor")?;

    Ok(())
}

/// Temporarily restore the normal terminal before launching an external tool.
pub fn suspend_terminal(terminal: &mut AppTerminal) -> Result<()> {
    disable_raw_mode().context("failed to disable terminal raw mode")?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)
        .context("failed to leave alternate screen")?;
    terminal
        .show_cursor()
        .context("failed to show terminal cursor")?;

    Ok(())
}

/// Return to TUI mode after an external command exits.
pub fn resume_terminal(terminal: &mut AppTerminal) -> Result<()> {
    enable_raw_mode().context("failed to enable terminal raw mode")?;
    execute!(terminal.backend_mut(), EnterAlternateScreen)
        .context("failed to enter alternate screen")?;
    terminal.clear().context("failed to clear terminal")?;

    Ok(())
}
