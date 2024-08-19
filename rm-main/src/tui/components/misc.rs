use ratatui::{
    layout::Alignment,
    style::Stylize,
    widgets::block::{Position, Title},
};
use rm_config::CONFIG;

pub fn popup_close_button_highlight() -> Title<'static> {
    Title::from(" [ CLOSE ] ".fg(CONFIG.general.accent_color).bold())
        .alignment(Alignment::Right)
        .position(Position::Bottom)
}

pub fn popup_close_button() -> Title<'static> {
    Title::from(" [CLOSE] ".bold())
        .alignment(Alignment::Right)
        .position(Position::Bottom)
}
