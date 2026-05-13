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
    let max_input_width = inner.width.saturating_sub(prefix.len() as u16);
    let (visible_input, cursor_offset) =
        visible_input_window(&app.session.input, app.input_cursor(), max_input_width);
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
            Span::styled(visible_input, theme::sunken_style(app)),
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
    let cursor_x = cursor_base_x + cursor_offset;
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

#[cfg(test)]
fn input_cursor_offset(input: &str, max_width: u16) -> u16 {
    Line::from(input).width().min(max_width as usize) as u16
}

fn visible_input_window(input: &str, cursor: usize, max_width: u16) -> (String, u16) {
    if input.is_empty() || max_width == 0 {
        return (String::new(), 0);
    }

    let cursor = clamp_cursor(input, cursor);
    let max_width = max_width as usize;
    let mut end = cursor;
    let mut visible_width = 0;
    while let Some(character) = input[end..].chars().next() {
        let width = character_width(character);
        if visible_width + width > max_width {
            break;
        }
        end += character.len_utf8();
        visible_width += width;
    }

    let mut start = cursor;
    let mut cursor_offset = 0;
    while let Some((index, character)) = previous_character(input, start) {
        let width = character_width(character);
        if visible_width + width > max_width {
            break;
        }
        start = index;
        cursor_offset += width;
        visible_width += width;
    }

    (input[start..end].to_string(), cursor_offset as u16)
}

fn clamp_cursor(input: &str, cursor: usize) -> usize {
    let mut cursor = cursor.min(input.len());
    while cursor > 0 && !input.is_char_boundary(cursor) {
        cursor -= 1;
    }
    cursor
}

fn previous_character(input: &str, cursor: usize) -> Option<(usize, char)> {
    input[..cursor].char_indices().next_back()
}

fn character_width(character: char) -> usize {
    let mut buffer = [0; 4];
    let encoded: &str = character.encode_utf8(&mut buffer);
    Line::from(encoded).width()
}

#[cfg(test)]
mod tests {
    use super::{input_cursor_offset, visible_input_window};

    #[test]
    fn input_cursor_offset_uses_terminal_width() {
        assert_eq!(input_cursor_offset("\u{00e9}", 10), 1);
        assert_eq!(input_cursor_offset("\u{1f600}", 10), 2);
        assert_eq!(input_cursor_offset("abcdef", 3), 3);
    }

    #[test]
    fn visible_input_window_keeps_cursor_visible_near_end() {
        assert_eq!(visible_input_window("abcdef", 6, 3), ("def".to_string(), 3));
    }

    #[test]
    fn visible_input_window_keeps_room_for_text_after_cursor() {
        assert_eq!(
            visible_input_window("abcdef", 4, 4),
            ("cdef".to_string(), 2)
        );
    }
}
