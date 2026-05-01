use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem},
};

use crate::app::App;
use crate::command::CommandSuggestion;

use super::theme;

/// Draw the slash-command autocomplete popup just above the input box.
///
/// The popup is anchored to the input area's top edge and grows upward, so
/// the user can see both their typed prefix and the available commands at the
/// same time. `Clear` is rendered first so the popup is visually opaque over
/// whatever was below it.
pub(super) fn draw_command_palette(
    frame: &mut Frame,
    app: &App,
    input_area: Rect,
    suggestions: &[CommandSuggestion],
) {
    const MAX_VISIBLE: usize = 8;

    let visible_count = suggestions.len().min(MAX_VISIBLE);
    let height = visible_count as u16 + 2;
    let width = 50.min(input_area.width);
    let area = Rect {
        x: input_area.x,
        y: input_area.y.saturating_sub(height),
        width,
        height,
    };

    let selected = app.suggestion_index();
    let command_width = suggestions
        .iter()
        .map(|suggestion| suggestion.name.len())
        .max()
        .unwrap_or(0);

    let items: Vec<ListItem> = suggestions
        .iter()
        .take(MAX_VISIBLE)
        .enumerate()
        .map(|(index, suggestion)| {
            let highlighted = index == selected;
            let row_style = if highlighted {
                Style::default()
                    .bg(theme::highlight_bg(app))
                    .fg(theme::highlight_fg(app))
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            let hint_style = if highlighted {
                row_style
            } else {
                Style::default().fg(theme::muted(app))
            };
            let padding = " ".repeat(command_width.saturating_sub(suggestion.name.len()) + 2);

            ListItem::new(Line::from(vec![
                Span::styled(format!(" {}", suggestion.name), row_style),
                Span::styled(padding, row_style),
                Span::styled(format!("{} ", suggestion.hint), hint_style),
            ]))
        })
        .collect();

    let popup = List::new(items).block(
        Block::default()
            .title("Commands - Tab/Enter accept, Esc dismiss")
            .borders(Borders::ALL),
    );

    frame.render_widget(Clear, area);
    frame.render_widget(popup, area);
}
