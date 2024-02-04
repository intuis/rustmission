use ratatui::prelude::*;
use rm_config::Config;
use std::{sync::Arc, time::Duration};

use crate::{
    components::{tabcomponent::TabComponent, torrent_tab::TorrentsTab, Component},
    tui::{self, Event},
};

use anyhow::Result;
use crossterm::event::KeyCode;
use ratatui::Frame;
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
    TorrentListUpdate(Vec<Torrent>),
    StatsUpdate(SessionStats),
}

const fn get_action(_app: &App, event: Event) -> Option<Action> {
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
        let stats = client.lock().await.session_stats().await.unwrap().arguments;
        sender.send(Action::StatsUpdate(stats)).unwrap();
        tokio::time::sleep(Duration::from_secs(3)).await;
    }
}

async fn transmission_torrent_fetch(
    client: Arc<Mutex<TransClient>>,
    sender: UnboundedSender<Action>,
) {
    loop {
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
        let res = client
            .lock()
            .await
            .torrent_get(Some(fields), None)
            .await
            .unwrap();
        let torrents = res.arguments.torrents;
        sender.send(Action::TorrentListUpdate(torrents)).unwrap();

        tokio::time::sleep(Duration::from_secs(3)).await;
    }
}

impl App {
    pub fn new(config: &Config) -> Self {
        // Get some values from config
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
        let mut tui = tui::Tui::new()?;

        tui.enter()?;

        loop {
            let e = tui.next().await.unwrap();
            match e {
                Event::Quit => self.action_tx.send(Action::Quit)?,
                Event::Error => todo!(),
                Event::Tick => self.action_tx.send(Action::Tick)?,
                Event::Render => self.action_tx.send(Action::Render)?,
                Event::Key(_) => {
                    let action = get_action(self, e);
                    if let Some(action) = action {
                        self.action_tx.send(action.clone()).unwrap();
                    }
                }
            }

            while let Ok(action) = self.action_rx.try_recv() {
                self.update(action.clone());
                if matches!(action, Action::Render) {
                    tui.draw(|f| {
                        self.ui(f);
                    })?;
                }
            }

            if self.should_quit {
                break;
            }
        }

        tui.exit()?;
        Ok(())
    }

    fn ui(&mut self, f: &mut Frame) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Percentage(100)])
            .split(f.size());
        self.components.tabs.render(f, layout[0]);
        self.components.torrents_tab.render(f, layout[1]);
    }

    fn update(&mut self, action: Action) -> Option<Action> {
        match (self.current_tab, &action) {
            (_, Action::Quit) => self.should_quit = true,
            (Tab::Torrents, _) => return self.components.torrents_tab.handle_events(action),
            _ => {}
        };
        None
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
