use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Wrap},
};

use crate::app::{App, ChatMessage};

use super::theme;

/// Draw previous prompt/answer pairs with user-controlled scrolling.
///
/// `scroll_offset` tracks how many lines the user has scrolled up
/// from the bottom. Zero means pinned to the newest content. The offset is
/// clamped here so it can never exceed the actual scrollable range.
pub(super) fn draw_history(frame: &mut Frame, app: &App, area: Rect) {
    let block_title = if app.ui.scroll_offset > 0 {
        format!(
            "Conversation Canvas  scrolled {} lines",
            app.ui.scroll_offset
        )
    } else {
        "Conversation Canvas".to_string()
    };
    let block = theme::canvas_block(app, block_title);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height == 0 {
        return;
    }

    let history_text = if app.session.history.is_empty() {
        empty_state_lines(app)
    } else {
        history_as_lines(app, &app.session.history)
    };

    let visible_height = inner.height as usize;
    let max_scroll = history_text.len().saturating_sub(visible_height);
    let clamped_offset = app.ui.scroll_offset.min(max_scroll);
    let scroll = max_scroll
        .saturating_sub(clamped_offset)
        .min(u16::MAX as usize) as u16;

    let history = Paragraph::new(history_text)
        .style(theme::body_style(app))
        .scroll((scroll, 0))
        .wrap(Wrap { trim: false });

    frame.render_widget(history, inner);
}

fn history_as_lines(app: &App, history: &[ChatMessage]) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    for (index, message) in history.iter().enumerate() {
        let turn_label = format!("TURN {:02}", index + 1);
        lines.push(Line::from(vec![
            Span::styled(turn_label, theme::label_style(app)),
            Span::styled("  ", theme::body_style(app)),
            Span::styled(" YOU ", theme::user_badge_style(app)),
        ]));
        push_multiline(
            &mut lines,
            "  ".to_string(),
            theme::body_style(app),
            &message.prompt,
        );
        lines.push(Line::from(""));

        let answer_style = if message.failed {
            theme::error_style(app).add_modifier(Modifier::BOLD)
        } else if message.in_progress {
            theme::warning_style(app).add_modifier(Modifier::BOLD)
        } else {
            theme::assistant_badge_style(app)
        };
        let answer_label = if message.in_progress {
            " STREAMING "
        } else if message.failed {
            " FAILED "
        } else {
            " ANSWER "
        };
        let answer_text = if message.in_progress && message.answer.is_empty() {
            "Waiting for first token...".to_string()
        } else {
            message.answer.clone()
        };

        lines.push(Line::from(vec![
            Span::styled("        ", theme::body_style(app)),
            Span::styled(answer_label, answer_style),
            Span::styled(" ", theme::body_style(app)),
            Span::styled(message.model_name.clone(), theme::secondary_style(app)),
        ]));
        push_multiline(
            &mut lines,
            "  ".to_string(),
            theme::body_style(app),
            &answer_text,
        );

        lines.push(Line::from(vec![
            Span::styled("  route ", theme::chip_accent_style(app)),
            Span::styled(message.route_reason.clone(), theme::chip_style(app)),
        ]));

        lines.push(Line::from(""));
    }

    lines
}

fn empty_state_lines(app: &App) -> Vec<Line<'static>> {
    vec![
        Line::from(vec![
            Span::styled(
                "READY",
                theme::success_style(app).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "  Command Deck is waiting for a prompt.",
                theme::muted_style(app),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Try ", theme::muted_style(app)),
            Span::styled("/model", theme::chip_accent_style(app)),
            Span::styled(
                " to pin a backend, or ask a question below.",
                theme::muted_style(app),
            ),
        ]),
    ]
}

fn push_multiline(lines: &mut Vec<Line<'static>>, indent: String, style: Style, text: &str) {
    let mut text_lines = text.lines();

    if let Some(first_line) = text_lines.next() {
        lines.push(Line::from(vec![
            Span::styled(indent.clone(), style),
            Span::styled(first_line.to_string(), style),
        ]));
    } else {
        lines.push(Line::from(Span::styled(indent.clone(), style)));
    }

    for line in text_lines {
        lines.push(Line::from(vec![
            Span::styled(indent.clone(), style),
            Span::styled(line.to_string(), style),
        ]));
    }
}
