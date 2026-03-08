use ratatui::{
    layout::{Constraint, Flex, Layout, Margin, Rect},
    style::{Color, Style, Stylize},
    text::Text,
    widgets::{List, ListState},
    Frame,
};
use rm_config::CONFIG;
use rm_shared::action::Action;
use transmission_rpc::types::{Id, TorrentSetArgs};

use crate::{
    transmission::TorrentAction,
    tui::{
        components::{popup_block, Component, ComponentAction},
        ctx::CTX,
    },
};

pub struct PriorityPopup {
    torrent_id: Id,
    files: Vec<usize>,
    list_state: ListState,
}

impl PriorityPopup {
    pub fn new(torrent_id: Id, files: Vec<usize>) -> Self {
        Self {
            torrent_id,
            files,
            list_state: ListState::default().with_selected(Some(1)),
        }
    }
}

impl Component for PriorityPopup {
    fn handle_actions(&mut self, action: Action) -> ComponentAction {
        if action.is_soft_quit() {
            return ComponentAction::Quit;
        }

        match action {
            Action::Up => {
                self.list_state.select_previous();
                CTX.send_action(Action::Render);
                return ComponentAction::Nothing;
            }
            Action::Down => {
                self.list_state.select_next();
                CTX.send_action(Action::Render);
                return ComponentAction::Nothing;
            }
            Action::Confirm => {
                let args = match self.list_state.selected().unwrap() {
                    0 => TorrentSetArgs::new().priority_low(self.files.clone()),
                    1 => TorrentSetArgs::new().priority_normal(self.files.clone()),
                    2 => TorrentSetArgs::new().priority_high(self.files.clone()),
                    _ => unreachable!(),
                };

                CTX.send_torrent_action(TorrentAction::SetArgs(
                    Box::new(args),
                    Some(vec![self.torrent_id.clone()]),
                ));

                CTX.send_action(Action::Render);
                return ComponentAction::Quit;
            }
            _ => return ComponentAction::Nothing,
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let [block_rect] = Layout::horizontal([Constraint::Length(20)])
            .flex(Flex::Center)
            .areas(rect);
        let [block_rect] = Layout::vertical([Constraint::Length(5)])
            .flex(Flex::Center)
            .areas(block_rect);

        let block = popup_block(" Priority ");

        let list_rect = block_rect.inner(Margin {
            horizontal: 1,
            vertical: 1,
        });
        let list = List::new([
            Text::raw("Low").centered(),
            Text::raw("Normal").centered(),
            Text::raw("High").centered(),
        ])
        .highlight_style(
            Style::default()
                .fg(CONFIG.general.accent_color)
                .bg(Color::Black)
                .bold(),
        );

        f.render_widget(block, block_rect);
        f.render_stateful_widget(list, list_rect, &mut self.list_state);
    }
}
