use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
};

use crate::app::App;

use super::theme;

/// Draw the models that the router can currently use.
pub(super) fn draw_models(frame: &mut Frame, app: &App, area: Rect) {
    let model_lines: Vec<ListItem> = app
        .models()
        .iter()
        .map(|model| {
            let strengths = model.strengths.join(", ");
            let setup_note = model
                .disabled_reason
                .as_ref()
                .map(|reason| format!(" ({reason})"))
                .unwrap_or_default();
            let model_style = if model.enabled {
                Style::default()
                    .fg(theme::accent(app))
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme::muted(app))
            };

            ListItem::new(Line::from(vec![
                Span::styled(model.display_label(), model_style),
                Span::raw(format!(" - {strengths}{setup_note}")),
            ]))
        })
        .collect();

    let model_list = List::new(model_lines).block(
        Block::default()
            .title("Available Models")
            .borders(Borders::ALL),
    );

    frame.render_widget(model_list, area);
}
