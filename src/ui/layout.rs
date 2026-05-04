use ratatui::layout::{Constraint, Direction, Layout, Rect};

use crate::app::App;

pub(super) enum CommandDeckLayout {
    Wide {
        header: Rect,
        model_rail: Rect,
        conversation: Rect,
        session_rail: Rect,
        composer: Rect,
    },
    Medium {
        header: Rect,
        model_ribbon: Rect,
        conversation: Rect,
        status_strip: Rect,
        composer: Rect,
    },
    Compact {
        header: Rect,
        conversation: Rect,
        status_strip: Rect,
        composer: Rect,
    },
}

pub(super) fn command_deck(area: Rect, app: &App) -> CommandDeckLayout {
    let surface = surface_area(area);
    let focus = app.layout_mode_name() == "focus";

    if !focus && surface.width >= 120 && surface.height >= 24 {
        wide_layout(surface)
    } else if !focus && surface.width >= 84 && surface.height >= 22 {
        medium_layout(surface)
    } else {
        compact_layout(surface)
    }
}

fn wide_layout(surface: Rect) -> CommandDeckLayout {
    let vertical_gap = gutter(surface.height, 30);
    let shell = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(vertical_gap),
            Constraint::Min(12),
            Constraint::Length(vertical_gap),
            Constraint::Length(composer_height(surface.height)),
        ])
        .split(surface);

    let horizontal_gap = gutter(surface.width, 120);
    let model_rail_width = 30.min(surface.width.saturating_sub(78));
    let session_rail_width = 28.min(surface.width.saturating_sub(80));
    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(model_rail_width),
            Constraint::Length(horizontal_gap),
            Constraint::Min(48),
            Constraint::Length(horizontal_gap),
            Constraint::Length(session_rail_width),
        ])
        .split(shell[2]);

    CommandDeckLayout::Wide {
        header: shell[0],
        model_rail: body[0],
        conversation: body[2],
        session_rail: body[4],
        composer: shell[4],
    }
}

fn medium_layout(surface: Rect) -> CommandDeckLayout {
    let vertical_gap = gutter(surface.height, 28);
    let shell = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(vertical_gap),
            Constraint::Length(5),
            Constraint::Length(vertical_gap),
            Constraint::Min(8),
            Constraint::Length(vertical_gap),
            Constraint::Length(3),
            Constraint::Length(vertical_gap),
            Constraint::Length(composer_height(surface.height)),
        ])
        .split(surface);

    CommandDeckLayout::Medium {
        header: shell[0],
        model_ribbon: shell[2],
        conversation: shell[4],
        status_strip: shell[6],
        composer: shell[8],
    }
}

fn compact_layout(surface: Rect) -> CommandDeckLayout {
    let vertical_gap = gutter(surface.height, 24);
    let status_height = if surface.height >= 16 { 3 } else { 1 };
    let shell = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(compact_header_height(surface.height)),
            Constraint::Length(vertical_gap),
            Constraint::Min(5),
            Constraint::Length(vertical_gap),
            Constraint::Length(status_height),
            Constraint::Length(vertical_gap),
            Constraint::Length(compact_composer_height(surface.height)),
        ])
        .split(surface);

    CommandDeckLayout::Compact {
        header: shell[0],
        conversation: shell[2],
        status_strip: shell[4],
        composer: shell[6],
    }
}

fn surface_area(area: Rect) -> Rect {
    if area.width >= 72 && area.height >= 20 {
        Rect {
            x: area.x + 1,
            y: area.y + 1,
            width: area.width.saturating_sub(2),
            height: area.height.saturating_sub(2),
        }
    } else {
        area
    }
}

fn composer_height(height: u16) -> u16 {
    if height >= 28 { 5 } else { 4 }
}

fn compact_header_height(height: u16) -> u16 {
    if height >= 12 { 3 } else { 1 }
}

fn compact_composer_height(height: u16) -> u16 {
    if height >= 14 { 4 } else { 3 }
}

fn gutter(size: u16, roomy_at: u16) -> u16 {
    if size >= roomy_at { 1 } else { 0 }
}
