use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

use crate::app::App;

/// Draw the keyboard and command help overlay.
pub(super) fn draw_help(frame: &mut Frame, area: Rect, app: &App) {
    let popup = centered_rect(78, 90, area);
    let mut help_lines = vec![
        Line::from(vec![Span::styled(
            "ollama-me help",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from("Enter sends the current prompt."),
        Line::from("? opens or closes this help when the prompt is empty."),
        Line::from("Esc, q, ?, or Ctrl-C closes help."),
        Line::from("Main screen: Esc or Ctrl-C quits. Ctrl-U clears input."),
        Line::from(""),
        Line::from("Up/Down scrolls the chat history one line at a time."),
        Line::from("PageUp/PageDown scrolls by half a screen."),
        Line::from("Home/End jumps to the top/bottom of the history."),
        Line::from(""),
        Line::from("Commands:"),
    ];

    for entry in app.command_help_entries() {
        help_lines.push(Line::from(format!("{} {}", entry.name, entry.detail)));
    }

    help_lines.push(Line::from(""));
    help_lines.push(Line::from(
        "Model picker: Up/Down navigate, Enter pins, Esc cancels.",
    ));

    let help = Paragraph::new(help_lines)
        .block(Block::default().title("Help").borders(Borders::ALL))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: false });

    frame.render_widget(Clear, popup);
    frame.render_widget(help, popup);
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1]);

    horizontal[1]
}
