use std::sync::{Arc, OnceLock};

use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Clear, Paragraph},
};
use ratatui_macros::constraints;
use transmission_rpc::types::{Id, Torrent};

use crate::{
    action::{Action, TorrentAction},
    app,
    ui::{centered_rect, components::Component},
};

pub struct InfoPopup {
    ctx: app::Ctx,
    // It could be probably replaced with a OnceLock
    torrent_info: Arc<OnceLock<Torrent>>,
}

impl InfoPopup {
    pub fn new(ctx: app::Ctx, torrent_id: Id) -> Self {
        let torrent_info = Arc::new(OnceLock::new());
        ctx.send_torrent_action(TorrentAction::GetTorrentInfo(
            torrent_id,
            Arc::clone(&torrent_info),
        ));

        Self { ctx, torrent_info }
    }
}

impl Component for InfoPopup {
    fn handle_actions(&mut self, action: Action) -> Option<Action> {
        if let Action::Confirm = action {
            return Some(Action::Quit);
        }
        None
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let popup_rect = centered_rect(rect, 50, 50);
        let block_rect = popup_rect.inner(&Margin::new(1, 1));
        let text_rect = block_rect.inner(&Margin::new(3, 2));
        let button_rect = Layout::vertical(constraints![==100%, ==1]).split(text_rect)[1];

        let title_style = Style::default().fg(self.ctx.config.general.accent_color.as_ratatui());
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(" Info ")
            .title_style(title_style);

        let button = Paragraph::new("[ OK ]").bold().right_aligned();

        let paragraph;
        if let Some(torrent_info) = self.torrent_info.get() {
            let download_dir = torrent_info.download_dir.as_ref().expect("Requested");
            let info = format!("Download dir: {download_dir}");
            paragraph = Paragraph::new(info);
        } else {
            paragraph = Paragraph::new("Loading...");
        }

        f.render_widget(Clear, popup_rect);
        f.render_widget(paragraph, text_rect);
        f.render_widget(block, block_rect);
        f.render_widget(button, button_rect);
    }
}
