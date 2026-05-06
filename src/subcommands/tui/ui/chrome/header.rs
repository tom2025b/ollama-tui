use ratatui::{
    Frame,
    layout::Rect,
    style::Modifier,
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::subcommands::tui::app::App;

use super::super::theme;

/// Draw the top command deck header.
pub(in crate::subcommands::tui::ui) fn draw_header(frame: &mut Frame, app: &App, area: Rect) {
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
