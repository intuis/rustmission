use chrono::{DateTime, Utc};
use intuitils::popup::popup_rects;
use ratatui::{
    prelude::*,
    widgets::{Clear, Paragraph, Wrap},
};
use rm_config::CONFIG;
use tokio::task::JoinHandle;
use transmission_rpc::types::{Id, TrackerState, TrackerStat};

use rm_shared::action::{Action, UpdateAction};

use crate::{
    tui::{
        components::{popup_block_with_close_highlight, Component, ComponentAction},
        ctx::CTX,
        tabs::torrents::rustmission_torrent::time_to_string,
    },
};

pub struct TrackersPopup {
    torrent_name: Option<String>,
    trackers: Option<Vec<TrackerStat>>,
    scroll: u16,
    fetch_task_handle: JoinHandle<()>,
}

impl TrackersPopup {
    pub fn new(torrent_id: Id) -> Self {
        Self {
            torrent_name: None,
            trackers: None,
            scroll: 0,
            fetch_task_handle: tokio::task::spawn(super::fetch_current_torrent(torrent_id)),
        }
    }
}

impl Component for TrackersPopup {
    fn handle_actions(&mut self, action: Action) -> ComponentAction {
        use Action as A;

        match action {
            _ if action.is_soft_quit() || action == A::Confirm => {
                self.fetch_task_handle.abort();
                ComponentAction::Quit
            }
            A::Up | A::ScrollUpBy(_) => {
                self.scroll = self.scroll.saturating_sub(1);
                CTX.send_action(A::Render);
                ComponentAction::Nothing
            }
            A::Down | A::ScrollDownBy(_) => {
                self.scroll = self.scroll.saturating_add(1);
                CTX.send_action(A::Render);
                ComponentAction::Nothing
            }
            A::Home => {
                self.scroll = 0;
                CTX.send_action(A::Render);
                ComponentAction::Nothing
            }
            _ => ComponentAction::Nothing,
        }
    }

    fn handle_update_action(&mut self, action: UpdateAction) {
        if let UpdateAction::UpdateCurrentTorrent(torrent) = action {
            self.torrent_name = torrent.name;
            self.trackers = torrent.tracker_stats;
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let (popup_rect, block_rect, text_rect) = popup_rects(rect, 75, 75);

        let title = match &self.torrent_name {
            Some(name) => format!(" Trackers - {name} "),
            None => " Trackers ".to_string(),
        };
        let block = popup_block_with_close_highlight(&title);

        let lines = match &self.trackers {
            None => vec![Line::from("Loading...")],
            Some(trackers) if trackers.is_empty() => vec![Line::from("No trackers")],
            Some(trackers) => trackers_to_lines(trackers),
        };

        // Don't let the user scroll past the last line.
        let max_scroll = u16::try_from(lines.len())
            .unwrap_or(u16::MAX)
            .saturating_sub(text_rect.height);
        self.scroll = self.scroll.min(max_scroll);

        let paragraph = Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .scroll((self.scroll, 0));

        f.render_widget(Clear, popup_rect);
        f.render_widget(block, block_rect);
        f.render_widget(paragraph, text_rect);
    }
}

fn trackers_to_lines(trackers: &[TrackerStat]) -> Vec<Line<'static>> {
    let accent = Style::default().fg(CONFIG.general.accent_color).bold();
    let mut lines = vec![];

    for (i, tracker) in trackers.iter().enumerate() {
        if i != 0 {
            lines.push(Line::default());
        }

        let name = if tracker.sitename.is_empty() {
            tracker.host.clone()
        } else {
            tracker.sitename.clone()
        };
        let mut header = Line::from(vec![
            Span::styled(name, accent),
            Span::raw(format!(" (tier {})", tracker.tier)),
        ]);
        if tracker.is_backup {
            header.push_span(Span::raw(" [backup]").dark_gray());
        }
        lines.push(header);

        lines.push(Line::from(format!("  URL: {}", tracker.announce)));
        lines.push(Line::from(format!(
            "  Status: {}",
            tracker_state_name(&tracker.announce_state)
        )));
        lines.push(Line::from(format!(
            "  Seeders: {} | Leechers: {} | Downloads: {}",
            count(tracker.seeder_count),
            count(tracker.leecher_count),
            count(tracker.download_count),
        )));

        if tracker.has_announced {
            lines.push(result_line(
                "Last announce",
                tracker.last_announce_time,
                &tracker.last_announce_result,
                tracker.last_announce_succeeded,
            ));
        } else {
            lines.push(Line::from("  Last announce: never"));
        }

        if matches!(tracker.announce_state, TrackerState::Waiting) {
            lines.push(Line::from(format!(
                "  Next announce: {}",
                time_to_string(tracker.next_announce_time)
            )));
        }

        if tracker.has_scraped {
            lines.push(result_line(
                "Last scrape",
                tracker.last_scrape_time,
                &tracker.last_scrape_result,
                tracker.last_scrape_succeeded,
            ));
        }
    }

    lines
}

fn tracker_state_name(state: &TrackerState) -> &'static str {
    match state {
        TrackerState::Inactive => "Inactive",
        TrackerState::Waiting => "Waiting",
        TrackerState::Queued => "Queued",
        TrackerState::Active => "Announcing",
    }
}

/// Transmission reports `-1` when a tracker hasn't provided the count.
fn count(n: i64) -> String {
    if n < 0 {
        "?".to_string()
    } else {
        n.to_string()
    }
}

fn result_line(
    label: &str,
    time: DateTime<Utc>,
    result: &str,
    succeeded: bool,
) -> Line<'static> {
    let result = if result.is_empty() { "Success" } else { result };
    let line = Line::from(format!("  {label}: {} - {result}", time_to_string(time)));
    if succeeded {
        line
    } else {
        line.red()
    }
}
