use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::app::App;

use super::theme;

/// Draw the current application status.
pub(super) fn draw_status(frame: &mut Frame, app: &App, area: Rect) {
    let status_color = if app.waiting_for_model {
        theme::warning(app)
    } else {
        theme::muted(app)
    };

    let status = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("Status: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(app.status.clone()),
        ]),
        Line::from(vec![
            Span::styled("Model: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(app.current_model_label()),
        ]),
        Line::from(vec![
            Span::styled("Rules: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(app.rules_status_line()),
        ]),
    ])
    .style(Style::default().fg(status_color))
    .block(Block::default().title("Status").borders(Borders::ALL));

    frame.render_widget(status, area);
}

/// Draw the input box.
pub(super) fn draw_input(frame: &mut Frame, app: &App, area: Rect) {
    let input = Paragraph::new(app.input.clone())
        .block(Block::default().title("Prompt").borders(Borders::ALL));

    frame.render_widget(input, area);

    let cursor_x = area.x + 1 + input_cursor_offset(&app.input, area.width.saturating_sub(2));
    let cursor_y = area.y + 1;
    frame.set_cursor_position((cursor_x, cursor_y));
}

fn input_cursor_offset(input: &str, max_width: u16) -> u16 {
    Line::from(input).width().min(max_width as usize) as u16
}

#[cfg(test)]
mod tests {
    use super::input_cursor_offset;

    #[test]
    fn input_cursor_offset_uses_terminal_width() {
        assert_eq!(input_cursor_offset("\u{00e9}", 10), 1);
        assert_eq!(input_cursor_offset("\u{1f600}", 10), 2);
        assert_eq!(input_cursor_offset("abcdef", 3), 3);
    }
}
