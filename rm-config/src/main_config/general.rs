use ratatui::style::Color;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct General {
    #[serde(default)]
    pub auto_hide: bool,
    #[serde(default = "default_accent_color")]
    pub accent_color: Color,
    #[serde(default = "default_beginner_mode")]
    pub beginner_mode: bool,
    #[serde(default)]
    pub headers_hide: bool,
}

impl Default for General {
    fn default() -> Self {
        Self {
            auto_hide: false,
            accent_color: default_accent_color(),
            beginner_mode: default_beginner_mode(),
            headers_hide: false,
        }
    }
}

fn default_accent_color() -> Color {
    Color::LightMagenta
}

fn default_beginner_mode() -> bool {
    true
}
