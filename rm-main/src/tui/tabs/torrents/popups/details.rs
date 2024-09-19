use ratatui::{
    prelude::*,
    widgets::{block::Title, Block, BorderType, Clear, Paragraph, Wrap},
};
use rm_config::{keymap::TorrentsAction, CONFIG};
use rm_shared::{action::Action, utils::bytes_to_human_format};
use style::Styled;

use crate::tui::{
    app,
    components::{keybinding_style, popup_close_button_highlight, Component, ComponentAction},
    main_window::centered_rect,
    tabs::torrents::rustmission_torrent::{CategoryType, RustmissionTorrent},
};

pub struct DetailsPopup {
    ctx: app::Ctx,
    torrent: RustmissionTorrent,
}

impl DetailsPopup {
    pub fn new(ctx: app::Ctx, torrent: RustmissionTorrent) -> Self {
        Self { ctx, torrent }
    }
}

impl Component for DetailsPopup {
    fn handle_actions(&mut self, action: Action) -> ComponentAction {
        match action {
            _ if action.is_soft_quit() => ComponentAction::Quit,
            Action::Confirm => ComponentAction::Quit,
            Action::Delete => {
                self.ctx.send_action(Action::Delete);
                ComponentAction::Quit
            }
            Action::ShowFiles => {
                self.ctx.send_action(Action::ShowFiles);
                ComponentAction::Quit
            }
            Action::ChangeCategory => {
                self.ctx.send_action(Action::ChangeCategory);
                ComponentAction::Quit
            }
            Action::MoveTorrent => {
                self.ctx.send_action(Action::MoveTorrent);
                ComponentAction::Quit
            }
            _ => ComponentAction::Nothing,
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let popup_rect = centered_rect(rect, 50, 50);
        let block_rect = popup_rect.inner(Margin::new(1, 1));
        let text_rect = block_rect.inner(Margin::new(3, 2));

        let title_style = Style::default().fg(CONFIG.general.accent_color);
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(Title::from(" Details ".set_style(title_style)))
            .title(popup_close_button_highlight());

        let mut lines = vec![];

        let name_line = Line::from(format!("Name: {}", self.torrent.torrent_name));

        let directory_line = Line::from(format!("Directory: {}", self.torrent.download_dir));

        let uploaded_line = Line::from(format!("Total uploaded: {}", self.torrent.uploaded_ever));

        let peers_line = Line::from(format!("Peers connected: {}", self.torrent.peers_connected));

        let ratio = Line::from(format!("Ratio: {}", self.torrent.upload_ratio));

        let size_line = Line::from(format!(
            "Size: {}",
            bytes_to_human_format(self.torrent.size_when_done)
        ));

        let activity_line = Line::from(format!("Last activity: {}", self.torrent.activity_date));

        let added_line = Line::from(format!("Added: {}", self.torrent.added_date));

        let mut show_files_line = Line::default();
        show_files_line.push_span(Span::raw("Show files: "));
        show_files_line.push_span(Span::styled(
            CONFIG
                .keybindings
                .torrents_tab
                .get_keys_for_action_joined(TorrentsAction::ShowFiles)
                .unwrap_or_default(),
            keybinding_style(),
        ));

        let mut move_location_line = Line::default();
        move_location_line.push_span(Span::raw("Move location: "));
        move_location_line.push_span(Span::styled(
            CONFIG
                .keybindings
                .torrents_tab
                .get_keys_for_action_joined(TorrentsAction::MoveTorrent)
                .unwrap_or_default(),
            keybinding_style(),
        ));

        let mut delete_line = Line::default();
        delete_line.push_span(Span::raw("Delete: "));
        delete_line.push_span(Span::styled(
            CONFIG
                .keybindings
                .torrents_tab
                .get_keys_for_action_joined(TorrentsAction::Delete)
                .unwrap_or_default(),
            keybinding_style(),
        ));

        let mut change_category_line = Line::default();
        change_category_line.push_span(Span::raw("Change category: "));
        change_category_line.push_span(Span::styled(
            CONFIG
                .keybindings
                .torrents_tab
                .get_keys_for_action_joined(TorrentsAction::ChangeCategory)
                .unwrap_or_default(),
            keybinding_style(),
        ));

        let padding_line = Line::default();

        lines.push(name_line);

        if let Some(error) = &self.torrent.error {
            lines.push(Line::from(format!("Error: {error}")).red());
        }

        if let Some(category) = &self.torrent.category {
            let mut category_line = Line::from("Category: ");
            let mut category_span = Span::raw(category.name());

            if let CategoryType::Config(category) = category {
                category_span = category_span.set_style(Style::default().fg(category.color))
            }

            category_line.push_span(category_span);

            lines.push(category_line);
        }

        lines.push(directory_line);
        lines.push(size_line);
        lines.push(padding_line.clone());
        lines.push(peers_line);
        lines.push(uploaded_line);
        lines.push(ratio);
        lines.push(padding_line.clone());
        lines.push(added_line);
        lines.push(activity_line);
        lines.push(padding_line);
        lines.push(delete_line);
        lines.push(show_files_line);
        lines.push(move_location_line);
        lines.push(change_category_line);

        let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });

        f.render_widget(Clear, popup_rect);
        f.render_widget(block, block_rect);
        f.render_widget(paragraph, text_rect);
    }
}
