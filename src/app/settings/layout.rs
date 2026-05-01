use super::super::App;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::app) enum LayoutMode {
    Compact,
    Normal,
    Focus,
}

impl LayoutMode {
    fn label(self) -> &'static str {
        match self {
            Self::Compact => "compact",
            Self::Normal => "normal",
            Self::Focus => "focus",
        }
    }

    fn next(self) -> Self {
        match self {
            Self::Compact => Self::Normal,
            Self::Normal => Self::Focus,
            Self::Focus => Self::Compact,
        }
    }

    fn parse(value: &str) -> Option<Self> {
        match value {
            "compact" => Some(Self::Compact),
            "normal" | "reset" => Some(Self::Normal),
            "focus" | "chat" => Some(Self::Focus),
            _ => None,
        }
    }
}

impl App {
    pub fn model_panel_height(&self) -> u16 {
        match self.layout_mode {
            LayoutMode::Compact => 4,
            LayoutMode::Normal => 7,
            LayoutMode::Focus => 3,
        }
    }

    pub fn status_panel_height(&self) -> u16 {
        5
    }

    pub fn layout_report(&self) -> String {
        format!(
            "Layout: {}\nAvailable layouts: compact, normal, focus.",
            self.layout_mode.label()
        )
    }

    pub fn set_layout_mode(&mut self, requested: Option<&str>) -> Result<String, String> {
        self.layout_mode = match requested.map(str::to_ascii_lowercase).as_deref() {
            None | Some("next") => self.layout_mode.next(),
            Some("show") | Some("status") => self.layout_mode,
            Some(value) => LayoutMode::parse(value)
                .ok_or_else(|| "Usage: /resize [compact|normal|focus|next|status]".to_string())?,
        };

        self.scroll_to_bottom();
        Ok(format!("Layout set to {}.", self.layout_mode.label()))
    }
}
