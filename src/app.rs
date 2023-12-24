use ratatui::prelude::*;
use std::time::Duration;

use crate::{
    components::{Component, Components},
    tui::{self, Event},
};

use anyhow::Result;
use crossterm::event::KeyCode;
use ratatui::{widgets::Paragraph, Frame};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use transmission_rpc::{
    types::{BasicAuth, Torrent, TorrentGetField},
    TransClient,
};

pub struct App {
    pub counter: u64,
    pub should_quit: bool,
    pub action_tx: UnboundedSender<Action>,
    pub action_rx: UnboundedReceiver<Action>,
    pub components: Components,
    pub current_tab: Tab,
}

enum Tab {
    Torrents,
    Search,
    Settings,
}

#[derive(Debug, Clone)]
pub enum Action {
    Tick,
    Increment,
    Decrement,
    Quit,
    Render,
    TorrentListUpdate(Vec<Torrent>),
}

fn get_action(_app: &App, event: Event) -> Option<Action> {
    if let Event::Key(key) = event {
        return match key.code {
            KeyCode::Char('j') => Some(Action::Increment),
            KeyCode::Char('k') => Some(Action::Decrement),
            KeyCode::Char('q') => Some(Action::Quit),
            _ => None,
        };
    }
    None
}

impl App {
    pub fn new() -> Self {
        let (action_tx, action_rx) = mpsc::unbounded_channel();
        let action_tx2 = action_tx.clone();
        let transmission_task = tokio::task::spawn(async move {
            let action_tx = action_tx2;
            let auth = BasicAuth {
                user: "erth".into(),
                password: "password".into(),
            };
            let mut client = TransClient::with_auth(
                "http://192.168.1.2:9091/transmission/rpc".parse().unwrap(),
                auth,
            );
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
                let res = client.torrent_get(Some(fields), None).await.unwrap();
                let torrents = res.arguments.torrents;
                action_tx.send(Action::TorrentListUpdate(torrents)).unwrap();

                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        });
        App {
            counter: 0,
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
                    let action = get_action(&self, e);
                    if let Some(action) = action {
                        self.action_tx.send(action.clone()).unwrap();
                    }
                }
            }

            while let Ok(action) = self.action_rx.try_recv() {
                self.update(action.clone());
                if let Action::Render = action {
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
        match action {
            Action::Quit => self.should_quit = true,
            Action::Increment => self.counter += 1,
            Action::Decrement if self.counter > 0 => self.counter -= 1,
            Action::TorrentListUpdate(torrents) => self.components.torrents_tab.torrents = torrents,
            Action::Tick => {}
            _ => {}
        };
        None
    }
}
