use std::collections::BTreeMap;

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
};

use rm_config::{
    keymap::{actions::UserAction, Keybinding},
    CONFIG,
};
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

const KEYS_DELIMITER: &str = ", ";

pub struct HelpPopup {
    ctx: app::Ctx,
    scroll: Option<Scroll>,
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
        Self { ctx, scroll: None }
    }

    fn get_keybindings<T: Into<Action> + UserAction + Ord>(
        keybindings: &[Keybinding<T>],
        max_keycode_len: &mut usize,
        max_line_len: &mut usize,
    ) -> Vec<(String, &'static str)> {
        let mut keys: BTreeMap<&T, Vec<String>> = BTreeMap::new();

        for keybinding in keybindings {
            if !keybinding.show_in_help {
                continue;
            }

            let keycode = keybinding.keycode_string();
            if keycode.len() > *max_keycode_len {
                *max_keycode_len = keycode.chars().count();
            }

            keys.entry(&keybinding.action)
                .or_insert_with(Vec::new)
                .push(keybinding.keycode_string());
        }

        for keycodes in keys.values() {
            let mut keycodes_total_len = 0;
            let delimiter_len = if keycodes.len() >= 2 {
                (keycodes.len() - 1) * 3
            } else {
                0
            };

            for keycode in keycodes {
                keycodes_total_len += keycode.chars().count();
            }

            if keycodes_total_len + delimiter_len > *max_keycode_len {
                *max_keycode_len = keycodes_total_len + delimiter_len;
            }
        }

        let mut new_keys = vec![];

        for (action, keycodes) in keys {
            new_keys.push((action, keycodes));
        }

        let mut res = vec![];
        let mut skip_next_loop = false;
        for (idx, (action, keycodes)) in new_keys.iter().enumerate() {
            if skip_next_loop {
                skip_next_loop = false;
                continue;
            }

            if let Some(next_key) = new_keys.get(idx + 1) {
                if action.is_mergable_with(next_key.0) {
                    skip_next_loop = true;
                    let keys = format!(
                        "{} / {}",
                        keycodes.join(KEYS_DELIMITER),
                        next_key.1.join(KEYS_DELIMITER)
                    );

                    if keys.chars().count() > *max_keycode_len {
                        *max_keycode_len = keys.chars().count();
                    }

                    let desc = action.merged_desc(next_key.0).unwrap();

                    let line_len = keys.chars().count() + desc.chars().count() + 3;
                    if line_len > *max_line_len {
                        *max_line_len = line_len;
                    }

                    res.push((keys, desc));

                    continue;
                }
            }

            let keycode_string = keycodes.join(KEYS_DELIMITER);
            if keycode_string.chars().count() > *max_keycode_len {
                *max_keycode_len = keycode_string.chars().count();
            }
            let desc = action.desc();
            let line_len = keycode_string.chars().count() + desc.chars().count() + 3;
            if line_len > *max_line_len {
                *max_line_len = line_len;
            }
            res.push((keycode_string, desc));
        }

        res
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

        let mut max_len = 0;
        let mut max_line_len = 0;
        let mut global_keys = Self::get_keybindings(
            &CONFIG.keybindings.general.keybindings,
            &mut max_len,
            &mut max_line_len,
        );
        let mut torrents_keys = Self::get_keybindings(
            &CONFIG.keybindings.torrents_tab.keybindings,
            &mut max_len,
            &mut max_line_len,
        );
        let mut search_keys = Self::get_keybindings(
            &CONFIG.keybindings.search_tab.keybindings,
            &mut max_len,
            &mut max_line_len,
        );
        debug_assert!(max_len > 0);
        let to_pad_additionally = (text_rect
            .width
            .saturating_sub(max_line_len.try_into().unwrap())
            / 2)
        .saturating_sub(6);
        max_len += usize::from(to_pad_additionally);

        let pad_keys = |keys: &mut Vec<(String, &'static str)>| {
            for key in keys {
                let mut how_much_to_pad = max_len.saturating_sub(key.0.chars().count());
                while how_much_to_pad > 0 {
                    key.0.insert(0, ' ');
                    how_much_to_pad -= 1;
                }
            }
        };
        pad_keys(&mut global_keys);
        pad_keys(&mut torrents_keys);
        pad_keys(&mut search_keys);

        let mut lines = vec![];

        let insert_keys = |lines: &mut Vec<Line>, keys: Vec<(String, &'static str)>| {
            for (keycode, desc) in keys {
                add_line!(lines, keycode, desc);
            }
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

        insert_keys(&mut lines, torrents_keys);

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
