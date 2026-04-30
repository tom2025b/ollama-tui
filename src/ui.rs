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
            Constraint::Length(5),
            Constraint::Length(3),
        ])
        .split(frame.area());

    draw_models(frame, app, page[0]);
    draw_history(frame, app, page[1]);
    draw_status(frame, app, page[2]);
    draw_input(frame, app, page[3]);

    // The autocomplete popup is drawn after the input box so it can overlap
    // the status panel without us having to reserve dedicated layout space.
    let suggestions = app.command_suggestions();
    if !suggestions.is_empty() {
        draw_command_palette(frame, app, page[3], &suggestions);
    }

    // The /models picker uses the same anchor as the slash-command popup so
    // the two overlays share a visual style. Drawing it last means it sits
    // on top of any other popup in the rare case both would be visible.
    if app.show_models_picker {
        draw_models_picker(frame, app, page[3]);
    }

    if app.show_help {
        draw_help(frame, frame.area());
    }
}

/// Draw the slash-command autocomplete popup just above the input box.
///
/// The popup is anchored to the input area's top edge and grows upward, so
/// the user can see both their typed prefix and the available commands at the
/// same time. `Clear` is rendered first so the popup is visually opaque over
/// whatever was below it.
fn draw_command_palette(
    frame: &mut Frame,
    app: &App,
    input_area: Rect,
    suggestions: &[(&'static str, &'static str)],
) {
    // Cap the popup height so a long match list cannot push off the screen.
    const MAX_VISIBLE: usize = 8;

    let visible_count = suggestions.len().min(MAX_VISIBLE);
    let height = visible_count as u16 + 2; // top + bottom borders
    let width = 50.min(input_area.width);
    let x = input_area.x;
    let y = input_area.y.saturating_sub(height);

    let area = Rect {
        x,
        y,
        width,
        height,
    };

    let selected = app.suggestion_index();

    // Pad each command name to the same width so the hint column lines up.
    let command_width = suggestions
        .iter()
        .map(|(command, _)| command.len())
        .max()
        .unwrap_or(0);

    let items: Vec<ListItem> = suggestions
        .iter()
        .take(MAX_VISIBLE)
        .enumerate()
        .map(|(index, (command, hint))| {
            let highlighted = index == selected;
            let row_style = if highlighted {
                Style::default()
                    .bg(Color::Cyan)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            let hint_style = if highlighted {
                row_style
            } else {
                Style::default().fg(Color::DarkGray)
            };

            let padding = " ".repeat(command_width.saturating_sub(command.len()) + 2);

            ListItem::new(Line::from(vec![
                Span::styled(format!(" {command}"), row_style),
                Span::styled(padding, row_style),
                Span::styled(format!("{hint} "), hint_style),
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

/// Draw the interactive `/models` picker.
///
/// Anchored above the input box and grown upward, just like the slash
/// autocomplete popup, so the two overlays feel like the same control. The
/// first row is always the synthetic "Auto" entry that clears the pin; the
/// remaining rows mirror `App::pickable_models` in order.
fn draw_models_picker(frame: &mut Frame, app: &App, input_area: Rect) {
    // Cap visible rows so a long model list cannot overflow the screen. If
    // the highlighted row is below the cap, scroll the visible window down.
    const MAX_VISIBLE: usize = 8;

    let entries = app.pickable_models();
    let total_rows = entries.len() + 1; // +1 for the leading "Auto" row.
    let selected = app.models_picker_index();

    let visible_count = total_rows.min(MAX_VISIBLE);
    let scroll_start = if selected >= MAX_VISIBLE {
        // Keep the highlight on the last visible row when the user scrolls
        // past the cap. Simple bottom-anchored windowing — good enough for
        // the small list sizes this app expects.
        selected + 1 - MAX_VISIBLE
    } else {
        0
    };

    let height = visible_count as u16 + 2; // +2 for the popup border.
    let width = 60.min(input_area.width);
    let x = input_area.x;
    let y = input_area.y.saturating_sub(height);
    let area = Rect {
        x,
        y,
        width,
        height,
    };

    // Build the visible slice of rows. Each row is built independently so
    // styling for the highlight is straightforward.
    let mut items: Vec<ListItem> = Vec::with_capacity(visible_count);
    for offset in 0..visible_count {
        let row_index = scroll_start + offset;
        let highlighted = row_index == selected;

        let row_style = if highlighted {
            Style::default()
                .bg(Color::Cyan)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        let hint_style = if highlighted {
            row_style
        } else {
            Style::default().fg(Color::DarkGray)
        };

        // Row 0 is the synthetic "Auto" entry; everything else maps to a
        // model in the pickable list with a small marker for the active pin.
        let (label, hint) = if row_index == 0 {
            (
                "Auto".to_string(),
                "Let the router pick per prompt".to_string(),
            )
        } else {
            let model = entries[row_index - 1];
            // A trailing star marks the row that is currently pinned, so the
            // user can visually confirm the active selection without reading
            // the status panel.
            let pin_marker = if app.is_pinned(model) { " *" } else { "" };
            (
                format!("{}{}", model.display_label(), pin_marker),
                model.strengths.join(", "),
            )
        };

        items.push(ListItem::new(Line::from(vec![
            Span::styled(format!(" {label}"), row_style),
            Span::styled("  ".to_string(), row_style),
            Span::styled(format!("{hint} "), hint_style),
        ])));
    }

    let popup = List::new(items).block(
        Block::default()
            .title("Models - ↑/↓ navigate, Enter pin, Esc cancel")
            .borders(Borders::ALL),
    );

    frame.render_widget(Clear, area);
    frame.render_widget(popup, area);
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
    let popup = centered_rect(78, 90, area);
    let help_lines = vec![
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
        Line::from("/clear clears the visible conversation."),
        Line::from("/models opens an interactive picker to pin a model."),
        Line::from("    Up/Down navigate, Enter pins, Esc cancels."),
        Line::from("    Pick \"Auto\" to hand routing back to the router."),
        Line::from("/backends lists backend readiness."),
        Line::from("/rules opens project rules; /rules global edits global rules."),
        Line::from("/rules off, /rules on, and /rules toggle control rule loading."),
        Line::from("/history shows history; /history save and /history email export it."),
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
