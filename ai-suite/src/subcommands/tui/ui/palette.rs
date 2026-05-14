use ratatui::{
    Frame,
    layout::Rect,
    text::{Line, Span},
    widgets::{Clear, List, ListItem},
};

use crate::subcommands::tui::app::App;
use crate::subcommands::tui::slash_commands::CommandSuggestion;

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
    let selected = app.suggestion_index();
    let max_visible = input_area.y.saturating_sub(2).max(1) as usize;
    let (scroll_start, visible_count) =
        visible_suggestion_window(suggestions.len(), selected, max_visible);
    let height = visible_count as u16 + 2;
    let width = 84.min(input_area.width);
    let area = Rect {
        x: input_area.x + input_area.width.saturating_sub(width) / 2,
        y: input_area.y.saturating_sub(height),
        width,
        height,
    };

    let command_width = suggestions
        .iter()
        .map(|suggestion| suggestion.name.len())
        .max()
        .unwrap_or(0);

    let items: Vec<ListItem> = suggestions
        .iter()
        .skip(scroll_start)
        .take(visible_count)
        .enumerate()
        .map(|(offset, suggestion)| {
            let highlighted = scroll_start + offset == selected;
            let row_style = if highlighted {
                theme::selection_style(app)
            } else {
                theme::raised_style(app)
            };
            let command_style = if highlighted {
                row_style
            } else {
                theme::accent_style(app)
            };
            let hint_style = if highlighted {
                row_style
            } else {
                theme::muted_style(app)
            };
            let padding = " ".repeat(command_width.saturating_sub(suggestion.name.len()) + 2);

            let lines = vec![Line::from(vec![
                Span::styled(format!(" {}", suggestion.name), command_style),
                Span::styled(padding, row_style),
                Span::styled(format!("{} ", suggestion.hint), hint_style),
            ])];

            ListItem::new(lines).style(row_style)
        })
        .collect();

    let popup = List::new(items).block(theme::overlay_block(app, "Command Palette").title_bottom(
        Line::from(Span::styled(
            " Tab/Enter accept | Esc dismiss ",
            theme::muted_style(app),
        )),
    ));

    frame.render_widget(Clear, area);
    frame.render_widget(popup, area);
}

fn visible_suggestion_window(
    suggestion_count: usize,
    selected: usize,
    max_visible: usize,
) -> (usize, usize) {
    if suggestion_count == 0 || max_visible == 0 {
        return (0, 0);
    }

    let visible_count = suggestion_count.min(max_visible);
    let selected = selected.min(suggestion_count - 1);
    let scroll_start = if selected >= visible_count {
        selected + 1 - visible_count
    } else {
        0
    };

    (scroll_start, visible_count)
}
