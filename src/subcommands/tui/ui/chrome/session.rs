use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Wrap},
};

use crate::subcommands::tui::app::{App, MAX_CONTEXT_TURNS};

use super::super::theme;

/// Draw the right-side session intelligence rail.
pub(in crate::subcommands::tui::ui) fn draw_session_intel(
    frame: &mut Frame,
    app: &App,
    area: Rect,
) {
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
    let routing_mode = if app.has_pinned_model() {
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

fn status_style(app: &App) -> Style {
    if app.session.waiting_for_model {
        theme::warning_style(app).add_modifier(Modifier::BOLD)
    } else {
        theme::success_style(app).add_modifier(Modifier::BOLD)
    }
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
