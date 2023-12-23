use std::time::Duration;

use crate::{
    components::{Component, Components},
    tui::{self, Event},
};

use anyhow::Result;
use crossterm::event::KeyCode;
use ratatui::{widgets::Paragraph, Frame};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use transmission_rpc::{types::BasicAuth, TransClient};

pub struct App {
    pub counter: u64,
    pub should_quit: bool,
    pub action_tx: UnboundedSender<Action>,
    pub action_rx: UnboundedReceiver<Action>,
    pub torrent_info: Option<Vec<String>>,
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
    TorrentListUpdate(Vec<String>),
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
                let res = client.torrent_get(None, None).await.unwrap();
                let names: Vec<String> = res
                    .arguments
                    .torrents
                    .into_iter()
                    .map(|it| it.name.unwrap())
                    .collect();
                action_tx.send(Action::TorrentListUpdate(names)).unwrap();

                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        });
        App {
            counter: 0,
            should_quit: false,
            action_tx,
            action_rx,
            torrent_info: None,
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
        self.components.tabs.render(f, f.size());
        // tabs.render(self, f, f.size());
        // self.components.tabs.render(f, f.size());
        // if let Some(torrent) = &self.torrent_info {
        //     f.render_widget(
        //         Paragraph::new(format!("Got first torrent: {}", torrent[0])),
        //         f.size(),
        //     );
        // };
    }

    fn update(&mut self, action: Action) -> Option<Action> {
        match action {
            Action::Quit => self.should_quit = true,
            Action::Increment => self.counter += 1,
            Action::Decrement if self.counter > 0 => self.counter -= 1,
            Action::TorrentListUpdate(names) => self.torrent_info = Some(names),
            Action::Tick => {}
            _ => {}
        };
        None
    }
}
