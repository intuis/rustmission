use ratatui::{
    layout::{Margin, Rect},
    style::{Style, Styled, Stylize},
    text::Line,
    widgets::{block::Title, Block, BorderType},
};
use rm_config::CONFIG;

use crate::tui::main_window::centered_rect;

pub fn popup_close_button_highlight() -> Line<'static> {
    Line::from(" [ CLOSE ] ".fg(CONFIG.general.accent_color).bold()).right_aligned()
}

pub fn popup_close_button() -> Line<'static> {
    Line::from(" [CLOSE] ".bold()).right_aligned()
}

pub fn popup_block(title: &str) -> Block {
    let title_style = Style::default().fg(CONFIG.general.accent_color);
    Block::bordered()
        .border_type(BorderType::Rounded)
        .title(Title::from(title.set_style(title_style)))
}

pub fn popup_block_with_close_highlight(title: &str) -> Block {
    popup_block(title).title(popup_close_button_highlight())
}

pub fn popup_rects(rect: Rect, percent_x: u16, percent_y: u16) -> (Rect, Rect, Rect) {
    let popup_rect = centered_rect(rect, percent_x, percent_y);
    let block_rect = popup_rect.inner(Margin::new(1, 1));
    let text_rect = block_rect.inner(Margin::new(3, 2));
    (popup_rect, block_rect, text_rect)
}

pub fn keybinding_style() -> Style {
    Style::default()
        .underlined()
        .underline_color(CONFIG.general.accent_color)
}
