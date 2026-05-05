mod chrome;
mod help;
mod history;
mod layout;
mod model_list;
mod model_picker;
mod palette;
mod theme;

use ratatui::{Frame, widgets::Block};

use crate::app::App;

use chrome::{draw_composer, draw_header, draw_session_intel, draw_status_strip};
use help::draw_help;
use history::draw_history;
use layout::CommandDeckLayout;
use model_list::{draw_model_rail, draw_model_ribbon};
use model_picker::draw_models_picker;
use palette::draw_command_palette;

/// Draw the entire terminal interface.
pub fn draw(frame: &mut Frame, app: &App) {
    let frame_area = frame.area();
    frame.render_widget(
        Block::default().style(theme::background_style(app)),
        frame_area,
    );

    let composer_area = match layout::command_deck(frame_area, app) {
        CommandDeckLayout::Wide {
            header,
            model_rail,
            conversation,
            session_rail,
            composer,
        } => {
            draw_header(frame, app, header);
            draw_model_rail(frame, app, model_rail);
            draw_history(frame, app, conversation);
            draw_session_intel(frame, app, session_rail);
            draw_composer(frame, app, composer);
            composer
        }
        CommandDeckLayout::Medium {
            header,
            model_ribbon,
            conversation,
            status_strip,
            composer,
        } => {
            draw_header(frame, app, header);
            draw_model_ribbon(frame, app, model_ribbon);
            draw_history(frame, app, conversation);
            draw_status_strip(frame, app, status_strip);
            draw_composer(frame, app, composer);
            composer
        }
        CommandDeckLayout::Compact {
            header,
            conversation,
            status_strip,
            composer,
        } => {
            draw_header(frame, app, header);
            draw_history(frame, app, conversation);
            draw_status_strip(frame, app, status_strip);
            draw_composer(frame, app, composer);
            composer
        }
    };

    let suggestions = app.command_suggestions();
    if !suggestions.is_empty() {
        draw_command_palette(frame, app, composer_area, &suggestions);
    }

    if app.ui.show_models_picker {
        draw_models_picker(frame, app, composer_area);
    }

    if app.ui.show_help {
        draw_help(frame, frame_area, app);
    }

}
