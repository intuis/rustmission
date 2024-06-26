use std::collections::BTreeMap;

use ratatui::{
    prelude::*,
    widgets::{
        block::{Position, Title},
        Block, Clear, Paragraph,
    },
};

use crate::{
    app,
    ui::{centered_rect, components::Component},
};
use rm_config::keymap::{GeneralAction, TorrentsAction, UserAction};
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
}

impl HelpPopup {
    pub const fn new(ctx: app::Ctx) -> Self {
        Self { ctx }
    }
}

impl Component for HelpPopup {
    fn handle_actions(&mut self, action: Action) -> Option<Action> {
        match action {
            action if action.is_soft_quit() => Some(Action::SoftQuit),
            Action::Confirm | Action::ShowHelp => Some(Action::SoftQuit),
            _ => None,
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let centered_rect = centered_rect(rect, 75, 75);
        let popup_rect = centered_rect.inner(&Margin::new(1, 1));
        let text_rect = popup_rect.inner(&Margin::new(3, 2));

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

        let mut general_keys: BTreeMap<GeneralAction, Vec<String>> = BTreeMap::new();

        for keybinding in &self.ctx.config.keybindings.general.keybindings {
            general_keys
                .entry(keybinding.action)
                .or_insert_with(Vec::new)
                .push(keybinding.keycode_string());
        }

        for (action, keycodes) in general_keys {
            let keycode_string = keycodes.join(" / ");
            add_line!(lines, keycode_string, action.desc());
        }

        lines.push(
            Line::from(vec![Span::styled(
                "Torrents Tab",
                Style::default().bold().underlined(),
            )])
            .centered(),
        );

        let mut torrent_keys: BTreeMap<TorrentsAction, Vec<String>> = BTreeMap::new();

        for keybinding in &self.ctx.config.keybindings.torrents_tab.keybindings {
            torrent_keys
                .entry(keybinding.action)
                .or_insert_with(Vec::new)
                .push(keybinding.keycode_string());
        }

        for (action, keycodes) in torrent_keys {
            let keycode_string = keycodes.join(" / ");
            add_line!(lines, keycode_string, action.desc());
        }

        let help_text = Text::from(lines);
        let help_paragraph = Paragraph::new(help_text);

        f.render_widget(Clear, centered_rect);
        f.render_widget(block, popup_rect);
        f.render_widget(help_paragraph, text_rect);
    }
}
