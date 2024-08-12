use std::collections::BTreeMap;

use ratatui::{
    prelude::*,
    widgets::{
        block::{Position, Title},
        Block, Borders, Clear, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
    },
};

use crate::{
    app,
    ui::{
        centered_rect,
        components::{Component, ComponentAction},
    },
};
use rm_config::keymap::{actions::UserAction, Keybinding};
use rm_shared::action::Action;

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

    fn write_keybindings<T: Into<Action> + UserAction + Ord>(
        keybindings: &[Keybinding<T>],
        lines: &mut Vec<Line>,
    ) {
        let mut keys = BTreeMap::new();
        let mut max_len = 0;

        for keybinding in keybindings {
            if !keybinding.show_in_help {
                continue;
            }

            let keycode = keybinding.keycode_string();
            if keycode.len() > max_len {
                max_len = keycode.chars().count();
            }

            keys.entry(&keybinding.action)
                .or_insert_with(Vec::new)
                .push(keybinding.keycode_string());
        }

        for (_, keycodes) in &keys {
            let delimiter_len;
            let mut keycodes_total_len = 0;
            if keycodes.len() >= 2 {
                delimiter_len = (keycodes.len() - 1) * 3;
            } else {
                delimiter_len = 0;
            }

            for keycode in keycodes {
                keycodes_total_len += keycode.chars().count();
            }

            if keycodes_total_len + delimiter_len > max_len {
                max_len = keycodes_total_len + delimiter_len;
            }
        }

        for (action, keycodes) in keys {
            let mut keycode_string = keycodes.join(" / ");
            let mut how_much_to_pad = max_len - keycode_string.chars().count();
            while how_much_to_pad > 0 {
                keycode_string.insert(0, ' ');
                how_much_to_pad -= 1;
            }

            add_line!(lines, keycode_string, action.desc());
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
            Action::Up => self.scroll_up(),
            Action::Down => self.scroll_down(),
            Action::ScrollUpPage | Action::Home => self.scroll_to_home(),
            Action::ScrollDownPage | Action::End => self.scroll_to_end(),
            _ => ComponentAction::Nothing,
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let centered_rect = centered_rect(rect, 75, 75);
        let popup_rect = centered_rect.inner(Margin::new(1, 1));
        let text_rect = popup_rect.inner(Margin::new(3, 2));

        let title_style = Style::new().fg(self.ctx.config.general.accent_color);
        let block = Block::bordered()
            .border_set(symbols::border::ROUNDED)
            .title(
                Title::from(
                    " [ CLOSE ] "
                        .fg(self.ctx.config.general.accent_color)
                        .bold(),
                )
                .alignment(Alignment::Right)
                .position(Position::Bottom),
            )
            .title(" Help ")
            .title_style(title_style);

        let mut lines = vec![Line::from(vec![Span::styled(
            "Global Keybindings",
            Style::default().bold().underlined(),
        )])
        .centered()];

        Self::write_keybindings(&self.ctx.config.keybindings.general.keybindings, &mut lines);

        lines.push(
            Line::from(vec![Span::styled(
                "Torrents Tab",
                Style::default().bold().underlined(),
            )])
            .centered(),
        );

        Self::write_keybindings(
            &self.ctx.config.keybindings.torrents_tab.keybindings,
            &mut lines,
        );

        lines.push(
            Line::from(vec![Span::styled(
                "Search Tab",
                Style::default().bold().underlined(),
            )])
            .centered(),
        );

        Self::write_keybindings(
            &self.ctx.config.keybindings.search_tab.keybindings,
            &mut lines,
        );

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

        f.render_widget(Clear, centered_rect);
        f.render_widget(block, popup_rect);
        f.render_widget(help_paragraph, text_rect);

        if let Some(scroll) = &mut self.scroll {
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .thumb_style(Style::default().fg(self.ctx.config.general.accent_color));

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
