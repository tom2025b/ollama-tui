use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Wrap},
};

use crate::app::{App, MAX_CONTEXT_TURNS};

use super::theme;

/// Draw the top command deck header.
pub(super) fn draw_header(frame: &mut Frame, app: &App, area: Rect) {
    if area.height == 0 {
        return;
    }

    let active_label = if app.session.waiting_for_model {
        "STREAMING"
    } else {
        "READY"
    };
    let active_style = if app.session.waiting_for_model {
        theme::warning_style(app).add_modifier(Modifier::BOLD)
    } else {
        theme::success_style(app).add_modifier(Modifier::BOLD)
    };

    if area.height < 3 {
        let line = Paragraph::new(Line::from(vec![
            Span::styled("ollama-me ", theme::accent_style(app)),
            Span::styled(active_label, active_style),
            Span::styled("  ", theme::raised_style(app)),
            Span::styled(app.current_model_label(), theme::muted_style(app)),
        ]))
        .style(theme::raised_style(app));
        frame.render_widget(line, area);
        return;
    }

    let block = theme::header_block(app);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let header = Paragraph::new(Line::from(vec![
        Span::styled("COMMAND DECK", theme::accent_style(app)),
        Span::styled("  ", theme::raised_style(app)),
        Span::styled(
            format!("layout {}", app.layout_mode_name()),
            theme::muted_style(app),
        ),
        Span::styled("  ", theme::raised_style(app)),
        Span::styled(active_label, active_style),
        Span::styled("  ", theme::raised_style(app)),
        Span::styled("model ", theme::muted_style(app)),
        Span::styled(app.current_model_label(), theme::secondary_style(app)),
        Span::styled("  ", theme::raised_style(app)),
        Span::styled(app.rules_status_line(), theme::muted_style(app)),
        Span::styled("  ? help", theme::muted_style(app)),
    ]))
    .style(theme::raised_style(app));

    frame.render_widget(header, inner);
}

/// Draw the right-side session intelligence rail.
pub(super) fn draw_session_intel(frame: &mut Frame, app: &App, area: Rect) {
    let block = theme::intel_block(app, "Session Intel");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height == 0 {
        return;
    }

    let context_turns = context_turn_count(app);
    let active_state = if app.session.waiting_for_model {
        "streaming"
    } else {
        "idle"
    };
    let routing_mode = if app.current_model_label().contains("(pinned)") {
        "pinned"
    } else {
        "auto"
    };

    let rows = vec![
        metric_line(app, "state", active_state.to_string(), status_style(app)),
        metric_line(
            app,
            "model",
            app.current_model_label(),
            theme::secondary_style(app),
        ),
        metric_line(
            app,
            "routing",
            routing_mode.to_string(),
            theme::accent_style(app),
        ),
        metric_line(
            app,
            "context",
            format!("{context_turns}/{MAX_CONTEXT_TURNS}"),
            theme::success_style(app).add_modifier(Modifier::BOLD),
        ),
        metric_line(
            app,
            "history",
            app.session.history.len().to_string(),
            theme::body_style(app),
        ),
        Line::from(""),
        Line::from(Span::styled("STATUS", theme::label_style(app))),
        Line::from(Span::styled(app.ui.status.clone(), status_style(app))),
        Line::from(""),
        Line::from(Span::styled("RULES", theme::label_style(app))),
        Line::from(Span::styled(
            app.rules_status_line(),
            theme::muted_style(app),
        )),
    ];

    let intel = Paragraph::new(rows)
        .style(theme::body_style(app))
        .wrap(Wrap { trim: false });
    frame.render_widget(intel, inner);
}

/// Draw compact status for medium and narrow layouts.
pub(super) fn draw_status_strip(frame: &mut Frame, app: &App, area: Rect) {
    if area.height == 0 {
        return;
    }

    if area.height < 3 {
        let status = Paragraph::new(status_line(app)).style(theme::sunken_style(app));
        frame.render_widget(status, area);
        return;
    }

    let block = theme::status_block(app, "Live Status");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height == 0 {
        return;
    }

    if inner.width >= 72 {
        draw_status_columns(frame, app, inner);
    } else {
        let status = Paragraph::new(status_line(app))
            .style(theme::sunken_style(app))
            .wrap(Wrap { trim: true });
        frame.render_widget(status, inner);
    }
}

fn status_style(app: &App) -> Style {
    if app.session.waiting_for_model {
        theme::warning_style(app).add_modifier(Modifier::BOLD)
    } else {
        theme::success_style(app).add_modifier(Modifier::BOLD)
    }
}

fn draw_status_columns(frame: &mut Frame, app: &App, area: Rect) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(34),
            Constraint::Length(1),
            Constraint::Percentage(33),
            Constraint::Length(1),
            Constraint::Percentage(33),
        ])
        .split(area);

    render_status_cell(
        frame,
        app,
        columns[0],
        "STATUS",
        app.ui.status.clone(),
        status_style(app),
    );
    render_status_cell(
        frame,
        app,
        columns[2],
        "MODEL",
        app.current_model_label(),
        theme::accent_style(app),
    );
    render_status_cell(
        frame,
        app,
        columns[4],
        "RULES",
        app.rules_status_line(),
        theme::body_style(app),
    );
}

fn render_status_cell(
    frame: &mut Frame,
    app: &App,
    area: Rect,
    label: &'static str,
    value: String,
    value_style: Style,
) {
    let cell = Paragraph::new(vec![
        Line::from(Span::styled(label, theme::label_style(app))),
        Line::from(Span::styled(value, value_style)),
    ])
    .style(theme::body_style(app));

    frame.render_widget(cell, area);
}

/// Draw the prompt composer.
pub(super) fn draw_composer(frame: &mut Frame, app: &App, area: Rect) {
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

    let cursor_base_x = inner.x + prefix.len() as u16;
    let cursor_x = cursor_base_x
        + input_cursor_offset(
            &app.session.input,
            inner.width.saturating_sub(prefix.len() as u16),
        );
    let cursor_y = inner.y;
    frame.set_cursor_position((cursor_x, cursor_y));
}

fn status_line(app: &App) -> Line<'static> {
    Line::from(vec![
        Span::styled("status ", theme::label_style(app)),
        Span::styled(app.ui.status.clone(), status_style(app)),
        Span::styled("  model ", theme::label_style(app)),
        Span::styled(app.current_model_label(), theme::secondary_style(app)),
        Span::styled("  rules ", theme::label_style(app)),
        Span::styled(app.rules_status_line(), theme::muted_style(app)),
    ])
}

fn metric_line(app: &App, label: &'static str, value: String, value_style: Style) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{label:<8}"), theme::label_style(app)),
        Span::styled(value, value_style),
    ])
}

fn context_turn_count(app: &App) -> usize {
    app.session
        .history
        .iter()
        .filter(|message| message.include_in_context)
        .count()
        .min(MAX_CONTEXT_TURNS)
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
