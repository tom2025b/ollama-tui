use ratatui::{
    layout::Alignment,
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Padding},
};

use crate::subcommands::tui::app::App;

use super::{
    colors::{accent, active_edge, border, surface, surface_raised, surface_sunken, text},
    styles::{deck_title_style, raised_style, sunken_style},
};

pub(in crate::subcommands::tui::ui) fn header_block(app: &App) -> Block<'static> {
    Block::bordered()
        .title(" ai-suite ")
        .title_alignment(Alignment::Left)
        .title_style(deck_title_style(app))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(raised_style(app).fg(active_edge(app)))
        .padding(Padding::horizontal(PANEL_PADDING_X))
        .style(raised_style(app))
}

pub(in crate::subcommands::tui::ui) fn rail_block(
    app: &App,
    title: impl Into<String>,
) -> Block<'static> {
    deck_block(app, title, surface_raised(app), border(app))
}

pub(in crate::subcommands::tui::ui) fn canvas_block(
    app: &App,
    title: impl Into<String>,
) -> Block<'static> {
    deck_block(app, title, surface(app), active_edge(app))
}

pub(in crate::subcommands::tui::ui) fn intel_block(
    app: &App,
    title: impl Into<String>,
) -> Block<'static> {
    deck_block(app, title, surface_raised(app), border(app))
}

pub(in crate::subcommands::tui::ui) fn status_block(
    app: &App,
    title: impl Into<String>,
) -> Block<'static> {
    deck_block(app, title, surface_sunken(app), border(app))
}

pub(in crate::subcommands::tui::ui) fn composer_block(
    app: &App,
    title: impl Into<String>,
    active: bool,
) -> Block<'static> {
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

pub(in crate::subcommands::tui::ui) fn overlay_block(
    app: &App,
    title: impl Into<String>,
) -> Block<'static> {
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

pub(in crate::subcommands::tui::ui) fn panel_inner_width(area_width: u16) -> u16 {
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
