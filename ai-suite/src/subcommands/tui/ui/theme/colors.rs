use ratatui::style::Color;

use crate::subcommands::tui::app::App;

pub(super) fn accent(app: &App) -> Color {
    match app.theme_name() {
        "light" => Color::Rgb(37, 99, 235),
        "mono" => Color::White,
        _ => Color::Rgb(34, 211, 238),
    }
}

pub(super) fn secondary(app: &App) -> Color {
    match app.theme_name() {
        "light" => Color::Rgb(124, 58, 237),
        "mono" => Color::Gray,
        _ => Color::Rgb(167, 139, 250),
    }
}

pub(super) fn success(app: &App) -> Color {
    match app.theme_name() {
        "light" => Color::Rgb(22, 163, 74),
        "mono" => Color::White,
        _ => Color::Rgb(52, 211, 153),
    }
}

pub(super) fn warning(app: &App) -> Color {
    match app.theme_name() {
        "light" => Color::Rgb(180, 83, 9),
        "mono" => Color::Gray,
        _ => Color::Rgb(251, 191, 36),
    }
}

pub(super) fn error(app: &App) -> Color {
    match app.theme_name() {
        "mono" => Color::White,
        "light" => Color::Rgb(220, 38, 38),
        _ => Color::Rgb(248, 113, 113),
    }
}

pub(super) fn muted(app: &App) -> Color {
    match app.theme_name() {
        "light" => Color::Rgb(100, 116, 139),
        "mono" => Color::Gray,
        _ => Color::Rgb(129, 145, 166),
    }
}

pub(super) fn highlight_bg(app: &App) -> Color {
    match app.theme_name() {
        "light" => accent(app),
        "mono" => Color::White,
        _ => Color::Rgb(34, 211, 238),
    }
}

pub(super) fn highlight_fg(app: &App) -> Color {
    match app.theme_name() {
        "light" => Color::White,
        "mono" => Color::Black,
        _ => Color::Black,
    }
}

pub(super) fn active_edge(app: &App) -> Color {
    match app.theme_name() {
        "light" => Color::Rgb(59, 130, 246),
        "mono" => Color::White,
        _ => Color::Rgb(45, 212, 191),
    }
}

pub(super) fn text(app: &App) -> Color {
    match app.theme_name() {
        "light" => Color::Rgb(15, 23, 42),
        "mono" => Color::White,
        _ => Color::Rgb(232, 238, 247),
    }
}

pub(super) fn background(app: &App) -> Color {
    match app.theme_name() {
        "light" => Color::Rgb(248, 250, 252),
        "mono" => Color::Black,
        _ => Color::Rgb(5, 7, 11),
    }
}

pub(super) fn surface(app: &App) -> Color {
    match app.theme_name() {
        "light" => Color::Rgb(255, 255, 255),
        "mono" => Color::Black,
        _ => Color::Rgb(14, 18, 26),
    }
}

pub(super) fn surface_raised(app: &App) -> Color {
    match app.theme_name() {
        "light" => Color::Rgb(255, 255, 255),
        "mono" => Color::Black,
        _ => Color::Rgb(18, 24, 34),
    }
}

pub(super) fn surface_sunken(app: &App) -> Color {
    match app.theme_name() {
        "light" => Color::Rgb(241, 245, 249),
        "mono" => Color::Black,
        _ => Color::Rgb(8, 12, 18),
    }
}

pub(super) fn border(app: &App) -> Color {
    match app.theme_name() {
        "light" => Color::Rgb(203, 213, 225),
        "mono" => Color::Gray,
        _ => Color::Rgb(50, 63, 81),
    }
}

pub(super) fn chip_bg(app: &App) -> Color {
    match app.theme_name() {
        "light" => Color::Rgb(226, 232, 240),
        "mono" => Color::Black,
        _ => Color::Rgb(24, 33, 45),
    }
}

pub(super) fn chip_fg(app: &App) -> Color {
    match app.theme_name() {
        "light" => Color::Rgb(51, 65, 85),
        "mono" => Color::White,
        _ => Color::Rgb(194, 205, 220),
    }
}

pub(super) fn badge_fg(_app: &App) -> Color {
    Color::Black
}
