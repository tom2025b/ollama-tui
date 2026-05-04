use ratatui::{
    layout::Alignment,
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Padding},
};

use crate::app::App;

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

pub(super) fn background_style(app: &App) -> Style {
    Style::default().fg(text(app)).bg(background(app))
}

pub(super) fn raised_style(app: &App) -> Style {
    Style::default().fg(text(app)).bg(surface_raised(app))
}

pub(super) fn sunken_style(app: &App) -> Style {
    Style::default().fg(text(app)).bg(surface_sunken(app))
}

pub(super) fn panel_style(app: &App) -> Style {
    Style::default().fg(text(app))
}

pub(super) fn body_style(app: &App) -> Style {
    panel_style(app)
}

pub(super) fn label_style(app: &App) -> Style {
    panel_style(app).fg(muted(app)).add_modifier(Modifier::BOLD)
}

pub(super) fn deck_title_style(app: &App) -> Style {
    panel_style(app)
        .fg(accent(app))
        .add_modifier(Modifier::BOLD)
}

pub(super) fn muted_style(app: &App) -> Style {
    panel_style(app).fg(muted(app))
}

pub(super) fn accent_style(app: &App) -> Style {
    panel_style(app)
        .fg(accent(app))
        .add_modifier(Modifier::BOLD)
}

pub(super) fn success_style(app: &App) -> Style {
    panel_style(app).fg(success(app))
}

pub(super) fn warning_style(app: &App) -> Style {
    panel_style(app).fg(warning(app))
}

pub(super) fn error_style(app: &App) -> Style {
    panel_style(app).fg(error(app))
}

pub(super) fn secondary_style(app: &App) -> Style {
    panel_style(app)
        .fg(secondary(app))
        .add_modifier(Modifier::BOLD)
}

pub(super) fn user_badge_style(app: &App) -> Style {
    Style::default()
        .fg(badge_fg(app))
        .bg(accent(app))
        .add_modifier(Modifier::BOLD)
}

pub(super) fn assistant_badge_style(app: &App) -> Style {
    Style::default()
        .fg(badge_fg(app))
        .bg(success(app))
        .add_modifier(Modifier::BOLD)
}

pub(super) fn chip_style(app: &App) -> Style {
    Style::default().fg(chip_fg(app)).bg(chip_bg(app))
}

pub(super) fn chip_accent_style(app: &App) -> Style {
    chip_style(app).fg(accent(app)).add_modifier(Modifier::BOLD)
}

pub(super) fn composer_hint_style(app: &App) -> Style {
    panel_style(app).fg(muted(app))
}

pub(super) fn selection_style(app: &App) -> Style {
    Style::default()
        .fg(highlight_fg(app))
        .bg(highlight_bg(app))
        .add_modifier(Modifier::BOLD)
}

pub(super) fn header_block(app: &App) -> Block<'static> {
    Block::bordered()
        .title(" ollama-me ")
        .title_alignment(Alignment::Left)
        .title_style(deck_title_style(app))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(raised_style(app).fg(active_edge(app)))
        .padding(Padding::horizontal(PANEL_PADDING_X))
        .style(raised_style(app))
}

pub(super) fn rail_block(app: &App, title: impl Into<String>) -> Block<'static> {
    deck_block(app, title, surface_raised(app), border(app))
}

pub(super) fn canvas_block(app: &App, title: impl Into<String>) -> Block<'static> {
    deck_block(app, title, surface(app), active_edge(app))
}

pub(super) fn intel_block(app: &App, title: impl Into<String>) -> Block<'static> {
    deck_block(app, title, surface_raised(app), border(app))
}

pub(super) fn status_block(app: &App, title: impl Into<String>) -> Block<'static> {
    deck_block(app, title, surface_sunken(app), border(app))
}

pub(super) fn composer_block(app: &App, title: impl Into<String>, active: bool) -> Block<'static> {
    let edge = if active {
        accent(app)
    } else {
        active_edge(app)
    };
    Block::bordered()
        .title(format!(" {} ", title.into()))
        .title_alignment(Alignment::Left)
        .title_style(sunken_style(app).fg(edge).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(sunken_style(app).fg(edge))
        .padding(Padding::horizontal(PANEL_PADDING_X))
        .style(sunken_style(app))
}

pub(super) fn overlay_block(app: &App, title: impl Into<String>) -> Block<'static> {
    Block::bordered()
        .title(format!(" {} ", title.into()))
        .title_alignment(Alignment::Left)
        .title_style(deck_title_style(app))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(raised_style(app).fg(accent(app)))
        .padding(Padding::horizontal(PANEL_PADDING_X))
        .style(raised_style(app))
}

pub(super) fn panel_inner_width(area_width: u16) -> u16 {
    area_width.saturating_sub(2 + PANEL_PADDING_X * 2)
}

const PANEL_PADDING_X: u16 = 1;

fn deck_block(
    app: &App,
    title: impl Into<String>,
    background: Color,
    edge: Color,
) -> Block<'static> {
    let style = Style::default().fg(text(app)).bg(background);
    Block::bordered()
        .title(format!(" {} ", title.into()))
        .title_alignment(Alignment::Left)
        .title_style(style.fg(accent(app)).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(style.fg(edge))
        .padding(Padding::horizontal(PANEL_PADDING_X))
        .style(style)
}

fn active_edge(app: &App) -> Color {
    match app.theme_name() {
        "light" => Color::Rgb(59, 130, 246),
        "mono" => Color::White,
        _ => Color::Rgb(45, 212, 191),
    }
}

fn text(app: &App) -> Color {
    match app.theme_name() {
        "light" => Color::Rgb(15, 23, 42),
        "mono" => Color::White,
        _ => Color::Rgb(232, 238, 247),
    }
}

fn background(app: &App) -> Color {
    match app.theme_name() {
        "light" => Color::Rgb(248, 250, 252),
        "mono" => Color::Black,
        _ => Color::Rgb(5, 7, 11),
    }
}

fn surface(app: &App) -> Color {
    match app.theme_name() {
        "light" => Color::Rgb(255, 255, 255),
        "mono" => Color::Black,
        _ => Color::Rgb(14, 18, 26),
    }
}

fn surface_raised(app: &App) -> Color {
    match app.theme_name() {
        "light" => Color::Rgb(255, 255, 255),
        "mono" => Color::Black,
        _ => Color::Rgb(18, 24, 34),
    }
}

fn surface_sunken(app: &App) -> Color {
    match app.theme_name() {
        "light" => Color::Rgb(241, 245, 249),
        "mono" => Color::Black,
        _ => Color::Rgb(8, 12, 18),
    }
}

fn border(app: &App) -> Color {
    match app.theme_name() {
        "light" => Color::Rgb(203, 213, 225),
        "mono" => Color::Gray,
        _ => Color::Rgb(50, 63, 81),
    }
}

fn chip_bg(app: &App) -> Color {
    match app.theme_name() {
        "light" => Color::Rgb(226, 232, 240),
        "mono" => Color::Black,
        _ => Color::Rgb(24, 33, 45),
    }
}

fn chip_fg(app: &App) -> Color {
    match app.theme_name() {
        "light" => Color::Rgb(51, 65, 85),
        "mono" => Color::White,
        _ => Color::Rgb(194, 205, 220),
    }
}

fn badge_fg(app: &App) -> Color {
    match app.theme_name() {
        "mono" => Color::Black,
        _ => Color::Black,
    }
}
