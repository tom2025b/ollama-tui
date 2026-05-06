use ratatui::{
    Frame,
    layout::Rect,
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::subcommands::tui::app::App;

use super::super::theme;

/// Draw the prompt composer.
pub(in crate::subcommands::tui::ui) fn draw_composer(frame: &mut Frame, app: &App, area: Rect) {
    let command_mode = app.session.input.starts_with('/');
    let title = if command_mode { "Command" } else { "Prompt" };
    let block = theme::composer_block(app, title, command_mode || !app.session.input.is_empty());
    let inner = block.inner(area);

    frame.render_widget(block, area);

    if inner.height == 0 {
        return;
    }

    let prefix = if command_mode { "$ " } else { "> " };
    let input_line = if app.session.input.is_empty() {
        Line::from(vec![
            Span::styled(prefix, theme::accent_style(app)),
            Span::styled(
                "Ask anything. Type / for commands.",
                theme::composer_hint_style(app),
            ),
        ])
    } else {
        Line::from(vec![
            Span::styled(prefix, theme::accent_style(app)),
            Span::styled(app.session.input.clone(), theme::sunken_style(app)),
        ])
    };

    frame.render_widget(
        Paragraph::new(input_line).style(theme::sunken_style(app)),
        Rect {
            x: inner.x,
            y: inner.y,
            width: inner.width,
            height: 1,
        },
    );

    if inner.height > 1 {
        render_footer(frame, app, inner);
    }

    let cursor_base_x = inner.x + prefix.len() as u16;
    let cursor_x = cursor_base_x
        + input_cursor_offset(
            &app.session.input,
            inner.width.saturating_sub(prefix.len() as u16),
        );
    frame.set_cursor_position((cursor_x, inner.y));
}

fn render_footer(frame: &mut Frame, app: &App, inner: Rect) {
    let footer = Paragraph::new(Line::from(vec![
        Span::styled("Enter", theme::chip_accent_style(app)),
        Span::styled(" send  ", theme::composer_hint_style(app)),
        Span::styled("/model", theme::chip_accent_style(app)),
        Span::styled(" pin  ", theme::composer_hint_style(app)),
        Span::styled("/theme", theme::chip_accent_style(app)),
        Span::styled(" switch  ", theme::composer_hint_style(app)),
        Span::styled("PgUp/PgDn", theme::chip_accent_style(app)),
        Span::styled(" history", theme::composer_hint_style(app)),
    ]))
    .style(theme::sunken_style(app));

    frame.render_widget(
        footer,
        Rect {
            x: inner.x,
            y: inner.y + inner.height - 1,
            width: inner.width,
            height: 1,
        },
    );
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
