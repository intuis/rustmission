use ratatui::{
    prelude::*,
    widgets::{
        block::{Position, Title},
        Block, Clear, Paragraph,
    },
};

use crate::{
    action::Action,
    app,
    ui::{centered_rect, components::Component},
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

        let title_style = Style::new().fg(self.ctx.config.general.accent_color.as_ratatui());
        let block = Block::bordered()
            .border_set(symbols::border::ROUNDED)
            .title(
                Title::from(
                    " [ CLOSE ] "
                        .fg(self.ctx.config.general.accent_color.as_ratatui())
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

        add_line!(lines, "? / F1", "show/hide help");
        add_line!(lines, "q", "quit Rustmission / a popup");
        add_line!(lines, "ESC", "close a popup / task");
        add_line!(lines, "1", "switch to torrents tab");
        add_line!(lines, "2", "switch to search tab");
        add_line!(lines, "/", "search or filter");
        add_line!(lines, "TAB", "switch focus");
        add_line!(lines, "Enter", "confirm");
        add_line!(lines, "CTRL-d", "scroll page down");
        add_line!(lines, "CTRL-u", "scroll page up");
        add_line!(lines, "Home", "scroll to the beginning");
        add_line!(lines, "End", "scroll to the end");
        add_line!(lines, "j / ↓", "move down");
        add_line!(lines, "k / ↑", "move up");

        lines.push(
            Line::from(vec![Span::styled(
                "Torrents Tab",
                Style::default().bold().underlined(),
            )])
            .centered(),
        );

        add_line!(lines, "a", "add a magnet url");
        add_line!(lines, "p", "pause/unpause a torrent");
        add_line!(lines, "d", "delete a torrent without files");
        add_line!(lines, "D", "delete a torrent with files");
        add_line!(lines, "f", "show files of a torrent");
        add_line!(lines, "s", "show statistics");

        let help_text = Text::from(lines);
        let help_paragraph = Paragraph::new(help_text);

        f.render_widget(Clear, centered_rect);
        f.render_widget(block, popup_rect);
        f.render_widget(help_paragraph, text_rect);
    }
}
