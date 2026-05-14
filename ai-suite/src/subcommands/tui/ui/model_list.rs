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
    let title = format!("Models {}/{} ready", models.len(), models.len());

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
        let pin = if app.is_pinned(model) { " PINNED" } else { "" };

        lines.push(Line::from(vec![
            Span::styled(
                "READY  ",
                theme::success_style(app).add_modifier(Modifier::BOLD),
            ),
            Span::styled("Ollama", theme::label_style(app)),
            Span::styled(pin, theme::chip_accent_style(app)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("        ", theme::body_style(app)),
            Span::styled(
                model.name.clone(),
                theme::body_style(app).add_modifier(Modifier::BOLD),
            ),
        ]));

        let note = model.strengths.join(", ");
        lines.push(Line::from(vec![
            Span::styled("        ", theme::body_style(app)),
            Span::styled(note, theme::muted_style(app)),
        ]));
        lines.push(Line::from(""));
    }

    let rail = Paragraph::new(lines).style(theme::body_style(app));
    frame.render_widget(rail, inner);
}

/// Draw the medium-layout model ribbon.
pub(super) fn draw_model_ribbon(frame: &mut Frame, app: &App, area: Rect) {
    let models = app.models();
    let title = format!("Route Deck {}/{} online", models.len(), models.len());

    let block = theme::rail_block(app, title);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height == 0 {
        return;
    }

    let mut model_spans = Vec::new();
    for model in models {
        model_spans.push(Span::styled(
            "READY ",
            theme::success_style(app).add_modifier(Modifier::BOLD),
        ));
        model_spans.push(Span::styled(model.name.clone(), theme::body_style(app)));
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
