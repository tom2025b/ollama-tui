use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};

use crate::app::{App, ChatMessage};

/// Draw the entire terminal interface.
///
/// The app uses four vertical areas:
/// 1. a model list at the top,
/// 2. the conversation history,
/// 3. a status line,
/// 4. the input box.
pub fn draw(frame: &mut Frame, app: &App) {
    let page = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),
            Constraint::Min(8),
            Constraint::Length(4),
            Constraint::Length(3),
        ])
        .split(frame.area());

    draw_models(frame, app, page[0]);
    draw_history(frame, app, page[1]);
    draw_status(frame, app, page[2]);
    draw_input(frame, app, page[3]);

    if app.show_help {
        draw_help(frame, frame.area());
    }
}

/// Draw the models that the router can currently use.
fn draw_models(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
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
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
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

/// Draw previous prompt/answer pairs with user-controlled scrolling.
///
/// `scroll_offset` in App tracks how many lines the user has scrolled up
/// from the bottom. Zero means pinned to the newest content. The offset is
/// clamped here so it can never exceed the actual scrollable range.
fn draw_history(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let history_text = if app.history.is_empty() {
        vec![Line::from("No prompts yet.")]
    } else {
        history_as_lines(&app.history)
    };

    // Height inside the border (top + bottom border = 2 rows).
    let visible_height = area.height.saturating_sub(2) as usize;

    // The farthest the view can scroll down (i.e., when pinned to bottom).
    let max_scroll = history_text.len().saturating_sub(visible_height);

    // Clamp the user's offset so it never exceeds the available range.
    let clamped_offset = app.scroll_offset.min(max_scroll);

    // Convert "lines from bottom" into the absolute row Ratatui expects.
    let scroll = max_scroll
        .saturating_sub(clamped_offset)
        .min(u16::MAX as usize) as u16;

    // Show scroll position in the title when the user isn't at the bottom.
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

/// Convert chat history into styled lines for Ratatui.
///
/// This keeps `draw_history` small and keeps all history formatting in one
/// place.
fn history_as_lines(history: &[ChatMessage]) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    for message in history {
        push_labeled_multiline(
            &mut lines,
            "You: ".to_string(),
            Style::default().add_modifier(Modifier::BOLD),
            &message.prompt,
        );

        let answer_style = if message.failed {
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
        } else if message.in_progress {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(Color::Green)
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
            Span::styled("Route: ", Style::default().fg(Color::Yellow)),
            Span::raw(message.route_reason.clone()),
        ]));

        lines.push(Line::from(""));
    }

    lines
}

/// Push text that may contain newlines while keeping the label on the first line.
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

/// Draw the current application status.
fn draw_status(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let status_color = if app.waiting_for_model {
        Color::Yellow
    } else {
        Color::Gray
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
    ])
    .style(Style::default().fg(status_color))
    .block(Block::default().title("Status").borders(Borders::ALL));

    frame.render_widget(status, area);
}

/// Draw the input box.
fn draw_input(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let input = Paragraph::new(app.input.clone())
        .block(Block::default().title("Prompt").borders(Borders::ALL));

    frame.render_widget(input, area);

    // Place the terminal cursor after the current input text.
    //
    // Saturating math avoids edge-case underflow if the terminal is extremely
    // narrow.
    let cursor_x = area.x + 1 + app.input.len().min(area.width.saturating_sub(2) as usize) as u16;
    let cursor_y = area.y + 1;
    frame.set_cursor_position((cursor_x, cursor_y));
}

/// Draw the keyboard and command help overlay.
fn draw_help(frame: &mut Frame, area: Rect) {
    let popup = centered_rect(72, 62, area);
    let help_lines = vec![
        Line::from(vec![Span::styled(
            "ollama-me help",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from("Enter sends the current prompt."),
        Line::from("? opens or closes this help when the prompt is empty."),
        Line::from("Esc closes help; from the main screen it quits."),
        Line::from("Ctrl-C quits. Ctrl-U clears the prompt input."),
        Line::from(""),
        Line::from("Up/Down scrolls the chat history one line at a time."),
        Line::from("PageUp/PageDown scrolls by half a screen."),
        Line::from("Home/End jumps to the top/bottom of the history."),
        Line::from(""),
        Line::from("/clear clears the visible conversation."),
        Line::from("/models lists configured models and setup notes."),
        Line::from("/backends lists backend readiness."),
    ];
    let help = Paragraph::new(help_lines)
        .block(Block::default().title("Help").borders(Borders::ALL))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: false });

    frame.render_widget(Clear, popup);
    frame.render_widget(help, popup);
}

/// Return a centered rectangle sized as a percentage of the available area.
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
