use super::super::App;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::app) enum UiTheme {
    Dark,
    Light,
    Mono,
}

impl UiTheme {
    fn label(self) -> &'static str {
        match self {
            Self::Dark => "dark",
            Self::Light => "light",
            Self::Mono => "mono",
        }
    }

    fn next(self) -> Self {
        match self {
            Self::Dark => Self::Light,
            Self::Light => Self::Mono,
            Self::Mono => Self::Dark,
        }
    }

    fn parse(value: &str) -> Option<Self> {
        match value {
            "dark" | "default" => Some(Self::Dark),
            "light" => Some(Self::Light),
            "mono" | "monochrome" => Some(Self::Mono),
            _ => None,
        }
    }
}

impl App {
    pub fn theme_name(&self) -> &'static str {
        self.ui.theme.label()
    }

    pub fn theme_report(&self) -> String {
        format!(
            "Theme: {}\nAvailable themes: dark, light, mono.",
            self.ui.theme.label()
        )
    }

    pub fn set_theme(&mut self, requested: Option<&str>) -> Result<String, String> {
        self.ui.theme = match requested.map(str::to_ascii_lowercase).as_deref() {
            None | Some("next") => self.ui.theme.next(),
            Some("show") | Some("status") => self.ui.theme,
            Some(value) => UiTheme::parse(value)
                .ok_or_else(|| "Usage: /theme [dark|light|mono|next|status]".to_string())?,
        };

        Ok(format!("Theme set to {}.", self.ui.theme.label()))
    }
}
