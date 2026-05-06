use ratatui::{
    Frame,
    layout::Rect,
    style::Modifier,
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::subcommands::tui::app::App;

use super::theme;

/// Draw the wide-layout model and routing rail.
pub(super) fn draw_model_rail(frame: &mut Frame, app: &App, area: Rect) {
    let models = app.models();
    let enabled_count = models.iter().filter(|model| model.enabled).count();
    let title = format!("Models {enabled_count}/{} ready", models.len());

    let block = theme::rail_block(app, title);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height == 0 {
        return;
    }

    let mut lines = vec![
        Line::from(vec![
            Span::styled("MODE     ", theme::label_style(app)),
            Span::styled(app.routing_mode_label(), theme::accent_style(app)),
        ]),
        Line::from(vec![
            Span::styled("CURRENT  ", theme::label_style(app)),
            Span::styled(app.current_model_label(), theme::secondary_style(app)),
        ]),
        Line::from(""),
    ];

    for model in models {
        let status_label = if model.enabled { "READY" } else { "OFFLINE" };
        let status_style = if model.enabled {
            theme::success_style(app).add_modifier(Modifier::BOLD)
        } else {
            theme::warning_style(app).add_modifier(Modifier::BOLD)
        };
        let model_style = if model.enabled {
            theme::body_style(app).add_modifier(Modifier::BOLD)
        } else {
            theme::muted_style(app)
        };
        let pin = if app.is_pinned(model) { " PINNED" } else { "" };

        lines.push(Line::from(vec![
            Span::styled(format!("{status_label:<7}"), status_style),
            Span::styled(model.provider.label(), theme::label_style(app)),
            Span::styled(pin, theme::chip_accent_style(app)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("        ", theme::body_style(app)),
            Span::styled(model.name.clone(), model_style),
        ]));

        let note = model
            .disabled_reason
            .clone()
            .unwrap_or_else(|| model.strengths.join(", "));
        let note_style = if model.enabled {
            theme::muted_style(app)
        } else {
            theme::warning_style(app)
        };
        lines.push(Line::from(vec![
            Span::styled("        ", theme::body_style(app)),
            Span::styled(note, note_style),
        ]));
        lines.push(Line::from(""));
    }

    let rail = Paragraph::new(lines).style(theme::body_style(app));
    frame.render_widget(rail, inner);
}

/// Draw the medium-layout model ribbon.
pub(super) fn draw_model_ribbon(frame: &mut Frame, app: &App, area: Rect) {
    let models = app.models();
    let enabled_count = models.iter().filter(|model| model.enabled).count();
    let title = format!("Route Deck {enabled_count}/{} online", models.len());

    let block = theme::rail_block(app, title);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height == 0 {
        return;
    }

    let mut model_spans = Vec::new();
    for model in models {
        let style = if model.enabled {
            theme::success_style(app).add_modifier(Modifier::BOLD)
        } else {
            theme::warning_style(app)
        };
        let marker = if model.enabled { "READY" } else { "OFFLINE" };
        model_spans.push(Span::styled(format!("{marker} "), style));
        model_spans.push(Span::styled(model.provider.label(), theme::body_style(app)));
        model_spans.push(Span::styled("  ", theme::body_style(app)));
    }

    let lines = vec![
        Line::from(vec![
            Span::styled("mode ", theme::label_style(app)),
            Span::styled(app.routing_mode_label(), theme::accent_style(app)),
            Span::styled("  current ", theme::label_style(app)),
            Span::styled(app.current_model_label(), theme::secondary_style(app)),
        ]),
        Line::from(model_spans),
    ];

    let ribbon = Paragraph::new(lines).style(theme::body_style(app));
    frame.render_widget(ribbon, inner);
}
