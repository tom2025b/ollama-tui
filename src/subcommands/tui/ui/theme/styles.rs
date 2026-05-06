use ratatui::style::{Modifier, Style};

use crate::subcommands::tui::app::App;

use super::colors::{
    accent, background, badge_fg, chip_bg, chip_fg, error, highlight_bg, highlight_fg, muted,
    secondary, success, surface_raised, surface_sunken, text, warning,
};

pub(in crate::subcommands::tui::ui) fn background_style(app: &App) -> Style {
    Style::default().fg(text(app)).bg(background(app))
}

pub(in crate::subcommands::tui::ui) fn raised_style(app: &App) -> Style {
    Style::default().fg(text(app)).bg(surface_raised(app))
}

pub(in crate::subcommands::tui::ui) fn sunken_style(app: &App) -> Style {
    Style::default().fg(text(app)).bg(surface_sunken(app))
}

pub(in crate::subcommands::tui::ui) fn panel_style(app: &App) -> Style {
    Style::default().fg(text(app))
}

pub(in crate::subcommands::tui::ui) fn body_style(app: &App) -> Style {
    panel_style(app)
}

pub(in crate::subcommands::tui::ui) fn label_style(app: &App) -> Style {
    panel_style(app).fg(muted(app)).add_modifier(Modifier::BOLD)
}

pub(super) fn deck_title_style(app: &App) -> Style {
    panel_style(app)
        .fg(accent(app))
        .add_modifier(Modifier::BOLD)
}

pub(in crate::subcommands::tui::ui) fn muted_style(app: &App) -> Style {
    panel_style(app).fg(muted(app))
}

pub(in crate::subcommands::tui::ui) fn accent_style(app: &App) -> Style {
    panel_style(app)
        .fg(accent(app))
        .add_modifier(Modifier::BOLD)
}

pub(in crate::subcommands::tui::ui) fn success_style(app: &App) -> Style {
    panel_style(app).fg(success(app))
}

pub(in crate::subcommands::tui::ui) fn warning_style(app: &App) -> Style {
    panel_style(app).fg(warning(app))
}

pub(in crate::subcommands::tui::ui) fn error_style(app: &App) -> Style {
    panel_style(app).fg(error(app))
}

pub(in crate::subcommands::tui::ui) fn secondary_style(app: &App) -> Style {
    panel_style(app)
        .fg(secondary(app))
        .add_modifier(Modifier::BOLD)
}

pub(in crate::subcommands::tui::ui) fn user_badge_style(app: &App) -> Style {
    Style::default()
        .fg(badge_fg(app))
        .bg(accent(app))
        .add_modifier(Modifier::BOLD)
}

pub(in crate::subcommands::tui::ui) fn assistant_badge_style(app: &App) -> Style {
    Style::default()
        .fg(badge_fg(app))
        .bg(success(app))
        .add_modifier(Modifier::BOLD)
}

pub(in crate::subcommands::tui::ui) fn chip_style(app: &App) -> Style {
    Style::default().fg(chip_fg(app)).bg(chip_bg(app))
}

pub(in crate::subcommands::tui::ui) fn chip_accent_style(app: &App) -> Style {
    chip_style(app).fg(accent(app)).add_modifier(Modifier::BOLD)
}

pub(in crate::subcommands::tui::ui) fn composer_hint_style(app: &App) -> Style {
    panel_style(app).fg(muted(app))
}

pub(in crate::subcommands::tui::ui) fn selection_style(app: &App) -> Style {
    Style::default()
        .fg(highlight_fg(app))
        .bg(highlight_bg(app))
        .add_modifier(Modifier::BOLD)
}
