use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::app::{App, ChatMessage};

use super::theme;

/// Draw previous prompt/answer pairs with user-controlled scrolling.
///
/// `scroll_offset` in App tracks how many lines the user has scrolled up
/// from the bottom. Zero means pinned to the newest content. The offset is
/// clamped here so it can never exceed the actual scrollable range.
pub(super) fn draw_history(frame: &mut Frame, app: &App, area: Rect) {
    let history_text = if app.history.is_empty() {
        vec![Line::from("No prompts yet.")]
    } else {
        history_as_lines(app, &app.history)
    };

    let visible_height = area.height.saturating_sub(2) as usize;
    let max_scroll = history_text.len().saturating_sub(visible_height);
    let clamped_offset = app.scroll_offset.min(max_scroll);
    let scroll = max_scroll
        .saturating_sub(clamped_offset)
        .min(u16::MAX as usize) as u16;

    let title = if clamped_offset > 0 {
        format!(
            "Conversation (recent turns) — ↑ {} lines above ↑",
            clamped_offset
        )
    } else {
        "Conversation (recent turns)".to_string()
    };

    let history = Paragraph::new(history_text)
        .block(Block::default().title(title).borders(Borders::ALL))
        .scroll((scroll, 0))
        .wrap(Wrap { trim: false });

    frame.render_widget(history, area);
}

fn history_as_lines(app: &App, history: &[ChatMessage]) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    for message in history {
        push_labeled_multiline(
            &mut lines,
            "You: ".to_string(),
            Style::default().add_modifier(Modifier::BOLD),
            &message.prompt,
        );

        let answer_style = if message.failed {
            Style::default()
                .fg(theme::error(app))
                .add_modifier(Modifier::BOLD)
        } else if message.in_progress {
            Style::default()
                .fg(theme::warning(app))
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(theme::success(app))
                .add_modifier(Modifier::BOLD)
        };
        let answer_label = if message.in_progress {
            format!("{} (streaming): ", message.model_name)
        } else {
            format!("{}: ", message.model_name)
        };
        let answer_text = if message.in_progress && message.answer.is_empty() {
            "Waiting for first token...".to_string()
        } else {
            message.answer.clone()
        };

        push_labeled_multiline(&mut lines, answer_label, answer_style, &answer_text);

        lines.push(Line::from(vec![
            Span::styled("Route: ", Style::default().fg(theme::warning(app))),
            Span::raw(message.route_reason.clone()),
        ]));

        lines.push(Line::from(""));
    }

    lines
}

fn push_labeled_multiline(
    lines: &mut Vec<Line<'static>>,
    label: String,
    label_style: Style,
    text: &str,
) {
    let mut text_lines = text.lines();

    if let Some(first_line) = text_lines.next() {
        lines.push(Line::from(vec![
            Span::styled(label, label_style),
            Span::raw(first_line.to_string()),
        ]));
    } else {
        lines.push(Line::from(Span::styled(label, label_style)));
    }

    for line in text_lines {
        lines.push(Line::from(Span::raw(line.to_string())));
    }
}
