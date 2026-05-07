use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Clear, Paragraph, Wrap},
};

use crate::subcommands::tui::app::App;
use crate::subcommands::tui::slash_commands::CommandHelp;

use super::theme;

/// Draw the keyboard and command help overlay.
pub(super) fn draw_help(frame: &mut Frame, area: Rect, app: &App) {
    let popup = centered_rect(82, 84, area);
    let content_width = theme::panel_inner_width(popup.width) as usize;
    let mut help_lines = vec![
        Line::from(vec![
            Span::styled("COMMAND DECK", theme::accent_style(app)),
            Span::styled("  terminal routing workspace", theme::muted_style(app)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Enter", theme::chip_accent_style(app)),
            Span::styled(" send   ", theme::muted_style(app)),
            Span::styled("Ctrl-U", theme::chip_accent_style(app)),
            Span::styled(" clear input   ", theme::muted_style(app)),
            Span::styled("?", theme::chip_accent_style(app)),
            Span::styled(" toggle help", theme::muted_style(app)),
        ]),
        Line::from(vec![
            Span::styled("Up/Down", theme::chip_accent_style(app)),
            Span::styled(" scroll   ", theme::muted_style(app)),
            Span::styled("PageUp/PageDown", theme::chip_accent_style(app)),
            Span::styled(" page   ", theme::muted_style(app)),
            Span::styled("Home/End", theme::chip_accent_style(app)),
            Span::styled(" jump", theme::muted_style(app)),
        ]),
        Line::from(vec![
            Span::styled("Esc/q/Ctrl-C", theme::chip_accent_style(app)),
            Span::styled(" close or quit from the main deck", theme::muted_style(app)),
        ]),
        Line::from(""),
        Line::from(Span::styled("Commands:", theme::label_style(app))),
    ];

    help_lines.extend(command_help_lines(
        &app.command_help_entries(),
        content_width,
    ));

    help_lines.push(Line::from(""));
    help_lines.push(Line::from(vec![
        Span::styled("Model picker: ", theme::label_style(app)),
        Span::styled(
            "Up/Down navigate, Enter pins, Esc cancels.",
            theme::muted_style(app),
        ),
    ]));

    let help = Paragraph::new(help_lines)
        .style(theme::raised_style(app))
        .block(theme::overlay_block(app, "Command Reference"))
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

fn command_help_lines(entries: &[CommandHelp], content_width: usize) -> Vec<Line<'static>> {
    const COLUMN_GAP: usize = 2;
    const MIN_TWO_COLUMN_WIDTH: usize = 56;

    if content_width < MIN_TWO_COLUMN_WIDTH {
        return entries
            .iter()
            .map(|entry| Line::from(command_help_cell(entry, content_width)))
            .collect();
    }

    let column_width = content_width.saturating_sub(COLUMN_GAP) / 2;
    let right_start = entries.len().div_ceil(2);

    (0..right_start)
        .map(|index| {
            let left = command_help_cell(&entries[index], column_width);
            match entries.get(index + right_start) {
                Some(right_entry) => {
                    let right = command_help_cell(right_entry, column_width);
                    Line::from(format!("{left:<column_width$}  {right}"))
                }
                None => Line::from(left),
            }
        })
        .collect()
}

fn command_help_cell(entry: &CommandHelp, width: usize) -> String {
    let text = format!("{:<10} {}", entry.name, entry.hint);
    text.chars().take(width).collect()
}

#[cfg(test)]
mod tests {
    use super::{command_help_cell, command_help_lines};
    use crate::subcommands::tui::slash_commands::CommandHelp;

    #[test]
    fn command_help_cell_fits_requested_width() {
        let entry = CommandHelp {
            name: "/bookmark",
            hint: "Remember latest turn",
            detail: "Add or remove the latest completed turn from future context.",
        };

        let cell = command_help_cell(&entry, 12);

        assert_eq!(cell.len(), 12);
        assert!(cell.starts_with("/bookmark"));
    }

    #[test]
    fn command_help_lines_uses_two_columns_when_there_is_room() {
        let entries = [
            CommandHelp {
                name: "/one",
                hint: "First",
                detail: "First command.",
            },
            CommandHelp {
                name: "/two",
                hint: "Second",
                detail: "Second command.",
            },
            CommandHelp {
                name: "/three",
                hint: "Third",
                detail: "Third command.",
            },
        ];

        assert_eq!(command_help_lines(&entries, 80).len(), 2);
        assert_eq!(command_help_lines(&entries, 40).len(), 3);
    }
}
