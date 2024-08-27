use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
};

use rm_config::CONFIG;
use rm_shared::action::Action;

use crate::tui::{
    app,
    components::{popup_block_with_close_highlight, popup_rects, Component, ComponentAction},
};

macro_rules! add_line {
    ($lines:expr, $key:expr, $description:expr) => {
        $lines.push(Line::from(vec![
            Span::styled($key, Style::default().bold()),
            " - ".into(),
            $description.into(),
        ]));
    };
}

pub struct HelpPopup {
    ctx: app::Ctx,
    scroll: Option<Scroll>,
    global_keys: Vec<(String, &'static str)>,
    torrent_keys: Vec<(String, &'static str)>,
    search_keys: Vec<(String, &'static str)>,
    max_key_len: usize,
    max_line_len: usize,
}

struct Scroll {
    state: ScrollbarState,
    position: u16,
    position_max: u16,
}

impl Scroll {
    fn new() -> Self {
        Self {
            state: ScrollbarState::default(),
            position: 0,
            position_max: 0,
        }
    }
}

impl HelpPopup {
    pub fn new(ctx: app::Ctx) -> Self {
        let mut max_key_len = 0;
        let mut max_line_len = 0;
        let global_keys = CONFIG.keybindings.general.get_help_repr();
        let torrent_keys = CONFIG.keybindings.torrents_tab.get_help_repr();
        let search_keys = CONFIG.keybindings.search_tab.get_help_repr();

        let mut calc_max_lens = |keys: &[(String, &'static str)]| {
            for (keycode, desc) in keys {
                let key_len = keycode.chars().count();
                let desc_len = desc.chars().count();
                let line_len = key_len + desc_len + 3;
                if key_len > max_key_len {
                    max_key_len = key_len;
                }

                if line_len > max_line_len {
                    max_line_len = line_len;
                }
            }
        };

        calc_max_lens(&global_keys);
        calc_max_lens(&torrent_keys);
        calc_max_lens(&search_keys);

        debug_assert!(max_key_len > 0);
        debug_assert!(max_line_len > 0);
        Self {
            ctx,
            scroll: None,
            global_keys,
            torrent_keys,
            search_keys,
            max_key_len,
            max_line_len,
        }
    }

    fn scroll_down(&mut self) -> ComponentAction {
        if let Some(scroll) = &mut self.scroll {
            if scroll.position >= scroll.position_max {
                return ComponentAction::Nothing;
            }

            scroll.position = scroll.position.saturating_add(1);
            scroll.state.next();
            self.ctx.send_action(Action::Render);
        }
        ComponentAction::Nothing
    }

    fn scroll_up(&mut self) -> ComponentAction {
        if let Some(scroll) = &mut self.scroll {
            scroll.position = scroll.position.saturating_sub(1);
            scroll.state.prev();
            self.ctx.send_action(Action::Render);
        }
        ComponentAction::Nothing
    }

    fn scroll_to_end(&mut self) -> ComponentAction {
        if let Some(scroll) = &mut self.scroll {
            scroll.position = scroll.position_max;
            scroll.state.last();
            self.ctx.send_action(Action::Render);
        }
        ComponentAction::Nothing
    }

    fn scroll_to_home(&mut self) -> ComponentAction {
        if let Some(scroll) = &mut self.scroll {
            scroll.position = 0;
            scroll.state.first();
            self.ctx.send_action(Action::Render);
        }
        ComponentAction::Nothing
    }
}

impl Component for HelpPopup {
    fn handle_actions(&mut self, action: Action) -> ComponentAction {
        match action {
            action if action.is_soft_quit() => ComponentAction::Quit,
            Action::Confirm | Action::ShowHelp => ComponentAction::Quit,
            Action::Up | Action::ScrollUpBy(_) => self.scroll_up(),
            Action::Down | Action::ScrollDownBy(_) => self.scroll_down(),
            Action::ScrollUpPage | Action::Home => self.scroll_to_home(),
            Action::ScrollDownPage | Action::End => self.scroll_to_end(),
            _ => ComponentAction::Nothing,
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let (popup_rect, block_rect, text_rect) = popup_rects(rect, 75, 75);

        let block = popup_block_with_close_highlight(" Help ");

        let to_pad_additionally = (text_rect
            .width
            .saturating_sub(self.max_line_len.try_into().unwrap())
            / 2)
        .saturating_sub(6);

        let pad_amount = usize::from(to_pad_additionally) + self.max_key_len;

        let padded_keys = |keys: &Vec<(String, &'static str)>| -> Vec<(String, &'static str)> {
            let mut new_keys = vec![];
            for key in keys {
                let mut keycode = key.0.clone();
                let mut how_much_to_pad = pad_amount.saturating_sub(key.0.chars().count());
                while how_much_to_pad > 0 {
                    keycode.insert(0, ' ');
                    how_much_to_pad -= 1;
                }
                new_keys.push((keycode, key.1));
            }
            new_keys
        };

        let global_keys = padded_keys(&mut self.global_keys);
        let torrent_keys = padded_keys(&mut self.torrent_keys);
        let search_keys = padded_keys(&mut self.search_keys);

        let mut lines = vec![];

        let insert_keys = |lines: &mut Vec<Line>, keys: Vec<(String, &'static str)>| {
            lines.push(Line::default());
            for (keycode, desc) in keys {
                add_line!(lines, keycode, *desc);
            }
            lines.push(Line::default());
        };

        lines.push(
            Line::from(vec![Span::styled(
                "Global Keybindings",
                Style::default().bold().underlined(),
            )])
            .centered(),
        );

        insert_keys(&mut lines, global_keys);

        lines.push(
            Line::from(vec![Span::styled(
                "Torrents Tab",
                Style::default().bold().underlined(),
            )])
            .centered(),
        );

        insert_keys(&mut lines, torrent_keys);

        lines.push(
            Line::from(vec![Span::styled(
                "Search Tab",
                Style::default().bold().underlined(),
            )])
            .centered(),
        );

        insert_keys(&mut lines, search_keys);

        let help_text = Text::from(lines);

        if text_rect.height <= u16::try_from(help_text.lines.len()).unwrap() {
            if self.scroll.is_none() {
                self.scroll = Some(Scroll::new());
            }
        } else {
            self.scroll = None;
        }

        if let Some(scroll) = &mut self.scroll {
            if text_rect.height < 5 {
                scroll.position_max = u16::try_from(help_text.lines.len()).unwrap();
            } else {
                scroll.position_max = u16::try_from(help_text.lines.len() - 5).unwrap();
            }

            scroll.state = scroll
                .state
                .content_length(scroll.position_max.into())
                .viewport_content_length(text_rect.height as usize);
        }

        let help_paragraph = {
            let paragraph = Paragraph::new(help_text);
            if let Some(scroll) = &self.scroll {
                paragraph
                    .scroll((scroll.position, 0))
                    .block(Block::new().borders(Borders::RIGHT))
            } else {
                paragraph
            }
        };

        f.render_widget(Clear, popup_rect);
        f.render_widget(block, block_rect);
        f.render_widget(help_paragraph, text_rect);

        if let Some(scroll) = &mut self.scroll {
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .thumb_style(Style::default().fg(CONFIG.general.accent_color));

            f.render_stateful_widget(
                scrollbar,
                text_rect.inner(Margin {
                    vertical: 1,
                    horizontal: 0,
                }),
                &mut scroll.state,
            )
        }
    }
}
