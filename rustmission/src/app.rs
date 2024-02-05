use ratatui::prelude::*;
use rm_config::Config;
use std::{pin::Pin, sync::Arc, time::Duration};

use crate::{
    components::{tabcomponent::TabComponent, torrent_tab::TorrentsTab, Component},
    tui::{Event, Tui},
};

use anyhow::Result;
use crossterm::event::KeyCode;
use ratatui::Frame;
use static_assertions::const_assert;
use tokio::sync::{
    mpsc::{self, UnboundedReceiver, UnboundedSender},
    Mutex,
};
use transmission_rpc::{
    types::{BasicAuth, SessionStats, Torrent, TorrentGetField},
    TransClient,
};

pub struct App {
    pub should_quit: bool,
    action_tx: UnboundedSender<Action>,
    pub action_rx: UnboundedReceiver<Action>,
    pub components: Components,
    pub current_tab: Tab,
}

#[derive(Clone, Copy)]
pub enum Tab {
    Torrents,
    Search,
    Settings,
}

#[derive(Debug, Clone)]
pub enum Action {
    Tick,
    Quit,
    Render,
    Up,
    Down,
    TorrentListUpdate(Box<Vec<Torrent>>),
    StatsUpdate(Pin<Box<SessionStats>>),
}

const_assert!(std::mem::size_of::<Action>() <= 16);

const fn event_to_action(event: Event) -> Option<Action> {
    match event {
        Event::Quit => Some(Action::Quit),
        Event::Error => todo!(),
        Event::Tick => Some(Action::Tick),
        Event::Render => Some(Action::Render),
        Event::Key(_) => keycode_to_action(event),
    }
}

const fn keycode_to_action(event: Event) -> Option<Action> {
    if let Event::Key(key) = event {
        return match key.code {
            KeyCode::Char('j') => Some(Action::Down),
            KeyCode::Char('k') => Some(Action::Up),
            KeyCode::Char('q') => Some(Action::Quit),
            _ => None,
        };
    }
    None
}

async fn transmission_stats_fetch(
    client: Arc<Mutex<TransClient>>,
    sender: UnboundedSender<Action>,
) {
    loop {
        let stats = Box::pin(client.lock().await.session_stats().await.unwrap().arguments);
        sender.send(Action::StatsUpdate(stats)).unwrap();
        tokio::time::sleep(Duration::from_secs(4)).await;
    }
}

async fn transmission_torrent_fetch(
    client: Arc<Mutex<TransClient>>,
    sender: UnboundedSender<Action>,
) {
    loop {
        // TODO: talk to rustmission-rpc's authors to tell them that torrent_get shouldnt
        // take an ownership of this vec, or check the documentation (maybe there's a function that
        // takes a reference who knows)
        let fields = vec![
            TorrentGetField::Id,
            TorrentGetField::Name,
            TorrentGetField::IsFinished,
            TorrentGetField::IsStalled,
            TorrentGetField::PercentDone,
            TorrentGetField::UploadRatio,
            TorrentGetField::SizeWhenDone,
            TorrentGetField::Eta,
            TorrentGetField::RateUpload,
            TorrentGetField::RateDownload,
        ];
        let rpc_response = client
            .lock()
            .await
            .torrent_get(Some(fields), None)
            .await
            .unwrap();
        let torrents = rpc_response.arguments.torrents;
        sender
            .send(Action::TorrentListUpdate(Box::new(torrents)))
            .unwrap();

        tokio::time::sleep(Duration::from_secs(4)).await;
    }
}

impl App {
    pub fn new(config: &Config) -> Self {
        let user = config.connection.username.clone();
        let password = config.connection.password.clone();
        let url = config.connection.url.clone().parse().unwrap();

        let auth = BasicAuth { user, password };

        let (action_tx, action_rx) = mpsc::unbounded_channel();
        let client = Arc::new(Mutex::new(TransClient::with_auth(url, auth)));
        tokio::spawn(transmission_torrent_fetch(
            Arc::clone(&client),
            action_tx.clone(),
        ));
        tokio::spawn(transmission_stats_fetch(client, action_tx.clone()));
        Self {
            should_quit: false,
            action_tx,
            action_rx,
            components: Components::new(),
            current_tab: Tab::Torrents,
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut tui = Tui::new()?;

        tui.enter()?;

        loop {
            let event = tui.next().await.unwrap();

            if let Some(action) = event_to_action(event) {
                if matches!(action, Action::Render) {
                    self.render(&mut tui)?;
                } else {
                    self.update(action);
                }
            }

            // For actions that come from somewhere else
            while let Ok(action) = self.action_rx.try_recv() {
                self.update(action);
            }

            if self.should_quit {
                break;
            }
        }

        tui.exit()?;
        Ok(())
    }

    fn render(&mut self, tui: &mut Tui) -> Result<()> {
        tui.draw(|f| {
            self.draw_main_ui(f);
        })?;
        Ok(())
    }

    fn draw_main_ui(&mut self, f: &mut Frame) {
        let [top_bar, main_window] =
            Layout::vertical([Constraint::Length(1), Constraint::Percentage(100)]).areas(f.size());

        self.components.tabs.render(f, top_bar);
        self.components.torrents_tab.render(f, main_window);
    }

    fn update(&mut self, action: Action) -> Option<Action> {
        match &action {
            Action::Quit => {
                self.should_quit = true;
                None
            }

            _ if matches!(self.current_tab, Tab::Torrents) => {
                return self.components.torrents_tab.handle_events(action)
            }

            _ => None,
        }
    }
}

pub struct Components {
    pub tabs: TabComponent,
    pub torrents_tab: TorrentsTab,
}

impl Components {
    pub fn new() -> Self {
        Self {
            tabs: TabComponent::new(),
            torrents_tab: TorrentsTab::new(),
        }
    }
}
