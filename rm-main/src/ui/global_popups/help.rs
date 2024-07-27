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
    scroll: u16,
    scroll_max: u16,
    scroll_state: ScrollbarState,
}

impl HelpPopup {
    pub fn new(ctx: app::Ctx) -> Self {
        Self {
            ctx,
            scroll: 0,
            scroll_state: ScrollbarState::default(),
            scroll_max: 0,
        }
    }

    fn write_keybindings<T: Into<Action> + UserAction + Ord>(
        keybindings: &[Keybinding<T>],
        lines: &mut Vec<Line>,
    ) {
        let mut keys = BTreeMap::new();

        for keybinding in keybindings {
            keys.entry(&keybinding.action)
                .or_insert_with(Vec::new)
                .push(keybinding.keycode_string());
        }

        for (action, keycodes) in keys {
            let keycode_string = keycodes.join(" / ");
            add_line!(lines, keycode_string, action.desc());
        }
    }

    fn scroll_down(&mut self) -> ComponentAction {
        if self.scroll >= self.scroll_max {
            return ComponentAction::Nothing;
        }

        self.scroll = self.scroll.saturating_add(1);
        self.scroll_state.next();
        self.ctx.send_action(Action::Render);
        ComponentAction::Nothing
    }

    fn scroll_up(&mut self) -> ComponentAction {
        self.scroll = self.scroll.saturating_sub(1);
        self.scroll_state.prev();
        self.ctx.send_action(Action::Render);
        ComponentAction::Nothing
    }

    fn scroll_to_end(&mut self) -> ComponentAction {
        self.scroll = self.scroll_max;
        self.scroll_state.last();
        self.ctx.send_action(Action::Render);
        ComponentAction::Nothing
    }

    fn scroll_to_home(&mut self) -> ComponentAction {
        self.scroll = 0;
        self.scroll_state.first();
        self.ctx.send_action(Action::Render);
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

        let help_text = Text::from(lines);

        if text_rect.height < 5 {
            self.scroll_max = u16::try_from(help_text.lines.len()).unwrap();
        } else {
            self.scroll_max = u16::try_from(help_text.lines.len() - 5).unwrap();
        }

        self.scroll_state = self
            .scroll_state
            .content_length(self.scroll_max.into())
            .viewport_content_length(text_rect.height as usize);

        let help_paragraph = Paragraph::new(help_text)
            .scroll((self.scroll, 0))
            .block(Block::new().borders(Borders::RIGHT));

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);

        f.render_widget(Clear, centered_rect);
        f.render_widget(block, popup_rect);
        f.render_widget(help_paragraph, text_rect);
        f.render_stateful_widget(
            scrollbar,
            text_rect.inner(Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut self.scroll_state,
        )
    }
}
