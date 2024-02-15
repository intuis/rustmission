use rm_config::Config;
use std::sync::Arc;

use crate::{
    action::{event_to_action, Action, Mode},
    transmission,
    tui::Tui,
    ui::{components::Component, MainWindow},
};

use anyhow::Result;
use tokio::sync::{
    mpsc::{self, UnboundedReceiver, UnboundedSender},
    Mutex,
};
use transmission_rpc::{types::BasicAuth, TransClient};

pub struct App {
    should_quit: bool,
    action_tx: UnboundedSender<Action>,
    action_rx: UnboundedReceiver<Action>,
    // TODO: change trans_tx to something else than Action
    trans_tx: UnboundedSender<Action>,
    main_window: MainWindow,
    mode: Mode,
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
        tokio::spawn(transmission::action_handler(
            client,
            trans_rx,
            action_tx.clone(),
        ));

        Self {
            should_quit: false,
            action_tx,
            action_rx,
            main_window: MainWindow::new(trans_tx.clone()),
            trans_tx,
            mode: Mode::Normal,
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut tui = Tui::new()?;

        tui.enter()?;

        loop {
            let event = tui.next().await.unwrap();

            if let Some(action) = event_to_action(self.mode, event) {
                if let Some(action) = self.update(action) {
                    self.action_tx.send(action)?;
                }
            }

            // For actions that come from somewhere else
            while let Ok(action) = self.action_rx.try_recv() {
                if action.is_render() {
                    self.render(&mut tui)?;
                }

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
            self.main_window.render(f, f.size());
        })?;
        Ok(())
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
                Some(Action::Render)
            }

            Action::SwitchToNormalMode => {
                self.mode = Mode::Normal;
                Some(Action::Render)
            }

            Action::TorrentAdd(_) => {
                self.trans_tx.send(action).unwrap();
                None
            }

            _ => self.main_window.handle_events(action),
        }
    }
}
