use ratatui::style::Color;

use crate::app::App;

pub(super) fn accent(app: &App) -> Color {
    match app.theme_name() {
        "light" => Color::Blue,
        "mono" => Color::White,
        _ => Color::Cyan,
    }
}

pub(super) fn success(app: &App) -> Color {
    match app.theme_name() {
        "light" => Color::Green,
        "mono" => Color::White,
        _ => Color::Green,
    }
}

pub(super) fn warning(app: &App) -> Color {
    match app.theme_name() {
        "light" => Color::Magenta,
        "mono" => Color::Gray,
        _ => Color::Yellow,
    }
}

pub(super) fn error(app: &App) -> Color {
    match app.theme_name() {
        "mono" => Color::White,
        _ => Color::Red,
    }
}

pub(super) fn muted(app: &App) -> Color {
    match app.theme_name() {
        "light" => Color::DarkGray,
        "mono" => Color::Gray,
        _ => Color::DarkGray,
    }
}

pub(super) fn highlight_bg(app: &App) -> Color {
    match app.theme_name() {
        "light" => Color::Blue,
        "mono" => Color::White,
        _ => Color::Cyan,
    }
}

pub(super) fn highlight_fg(app: &App) -> Color {
    match app.theme_name() {
        "mono" => Color::Black,
        _ => Color::Black,
    }
}
