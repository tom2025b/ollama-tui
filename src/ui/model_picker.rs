use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem},
};

use crate::app::App;

use super::theme;

/// Draw the interactive `/models` picker.
///
/// Anchored above the input box and grown upward, just like the slash
/// autocomplete popup, so the two overlays feel like the same control. The
/// first row is always the synthetic "Auto" entry that clears the pin; the
/// remaining rows mirror `App::pickable_models` in order.
pub(super) fn draw_models_picker(frame: &mut Frame, app: &App, input_area: Rect) {
    const MAX_VISIBLE: usize = 8;

    let entries = app.pickable_models();
    let total_rows = entries.len() + 1;
    let selected = app.models_picker_index();

    let visible_count = total_rows.min(MAX_VISIBLE);
    let scroll_start = if selected >= MAX_VISIBLE {
        selected + 1 - MAX_VISIBLE
    } else {
        0
    };

    let height = visible_count as u16 + 2;
    let width = 60.min(input_area.width);
    let area = Rect {
        x: input_area.x,
        y: input_area.y.saturating_sub(height),
        width,
        height,
    };

    let mut items: Vec<ListItem> = Vec::with_capacity(visible_count);
    for offset in 0..visible_count {
        let row_index = scroll_start + offset;
        let highlighted = row_index == selected;

        let row_style = if highlighted {
            Style::default()
                .bg(theme::highlight_bg(app))
                .fg(theme::highlight_fg(app))
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        let hint_style = if highlighted {
            row_style
        } else {
            Style::default().fg(theme::muted(app))
        };

        let (label, hint) = if row_index == 0 {
            (
                "Auto".to_string(),
                "Let the router pick per prompt".to_string(),
            )
        } else {
            let model = entries[row_index - 1];
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
