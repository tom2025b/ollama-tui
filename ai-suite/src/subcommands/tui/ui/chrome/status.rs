use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Wrap},
};

use crate::subcommands::tui::app::App;

use super::super::theme;

/// Draw compact status for medium and narrow layouts.
pub(in crate::subcommands::tui::ui) fn draw_status_strip(frame: &mut Frame, app: &App, area: Rect) {
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
