mod chrome;
mod help;
mod history;
mod model_list;
mod model_picker;
mod palette;
mod theme;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
};

use crate::app::App;

use chrome::{draw_input, draw_status};
use help::draw_help;
use history::draw_history;
use model_list::draw_models;
use model_picker::draw_models_picker;
use palette::draw_command_palette;

/// Draw the entire terminal interface.
///
/// The app uses four vertical areas:
/// 1. a model list at the top,
/// 2. the conversation history,
/// 3. a status line,
/// 4. the input box.
pub fn draw(frame: &mut Frame, app: &App) {
    let page = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(app.model_panel_height()),
            Constraint::Min(8),
            Constraint::Length(app.status_panel_height()),
            Constraint::Length(3),
        ])
        .split(frame.area());

    draw_models(frame, app, page[0]);
    draw_history(frame, app, page[1]);
    draw_status(frame, app, page[2]);
    draw_input(frame, app, page[3]);

    let suggestions = app.command_suggestions();
    if !suggestions.is_empty() {
        draw_command_palette(frame, app, page[3], &suggestions);
    }

    if app.show_models_picker {
        draw_models_picker(frame, app, page[3]);
    }

    if app.show_help {
        draw_help(frame, frame.area(), app);
    }
}
