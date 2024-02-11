use ratatui::prelude::*;
use rm_config::Config;
use std::sync::Arc;

use crate::{
    action::{event_to_action, Action, Mode},
    components::{tabcomponent::TabComponent, torrent_tab::TorrentsTab, Component},
    transmission,
    tui::Tui,
};

use anyhow::Result;
use tokio::sync::{
    mpsc::{self, UnboundedReceiver, UnboundedSender},
    Mutex,
};
use transmission_rpc::{types::BasicAuth, TransClient};

pub struct App {
    pub should_quit: bool,
    action_tx: UnboundedSender<Action>,
    pub action_rx: UnboundedReceiver<Action>,
    // TODO: change trans_tx to something else than Action
    trans_tx: UnboundedSender<Action>,
    pub components: Components,
    pub current_tab: Tab,
    mode: Mode,
}

#[derive(Clone, Copy)]
pub enum Tab {
    Torrents,
    Search,
    Settings,
}

impl App {
    pub fn new(config: &Config) -> Self {
        let user = config.connection.username.clone();
        let password = config.connection.password.clone();
        let url = config.connection.url.clone().parse().unwrap();

        let auth = BasicAuth { user, password };

        let (action_tx, action_rx) = mpsc::unbounded_channel();

        let client = Arc::new(Mutex::new(TransClient::with_auth(url, auth)));
        transmission::spawn_fetchers(client.clone(), action_tx.clone());

        let (trans_tx, trans_rx) = mpsc::unbounded_channel();
        tokio::spawn(transmission::action_handler(client, trans_rx));

        Self {
            should_quit: false,
            action_tx,
            action_rx,
            components: Components::new(trans_tx.clone()),
            trans_tx,
            current_tab: Tab::Torrents,
            mode: Mode::Normal,
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut tui = Tui::new()?;

        tui.enter()?;

        loop {
            let event = tui.next().await.unwrap();

            if let Some(action) = event_to_action(self.mode, event) {
                if action.is_render() {
                    self.render(&mut tui)?;
                } else if let Some(action) = self.update(action) {
                    self.action_tx.send(action)?;
                }
            }

            // For actions that come from somewhere else
            while let Ok(action) = self.action_rx.try_recv() {
                if let Some(action) = self.update(action) {
                    self.action_tx.send(action)?;
                }
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

    #[must_use]
    fn update(&mut self, action: Action) -> Option<Action> {
        match &action {
            Action::Quit => {
                self.should_quit = true;
                None
            }

            Action::SwitchToInputMode => {
                self.mode = Mode::Input;
                None
            }

            Action::SwitchToNormalMode => {
                self.mode = Mode::Normal;
                None
            }

            Action::TorrentAdd(_) => {
                self.trans_tx.send(action).unwrap();
                None
            }

            _ if matches!(self.current_tab, Tab::Torrents) => {
                self.components.torrents_tab.handle_events(action)
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
    pub fn new(trans_tx: UnboundedSender<Action>) -> Self {
        Self {
            tabs: TabComponent::new(),
            torrents_tab: TorrentsTab::new(trans_tx),
        }
    }
}
