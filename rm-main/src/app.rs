use rm_config::Config;
use rm_shared::action::event_to_action;
use rm_shared::action::Action;
use rm_shared::action::Mode;
use rm_shared::action::UpdateAction;
use std::sync::Arc;

use crate::{
    transmission::{self, TorrentAction},
    tui::Tui,
    ui::{components::Component, MainWindow},
};

use anyhow::{Error, Result};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use transmission_rpc::{types::SessionGet, TransClient};

#[derive(Clone)]
pub struct Ctx {
    pub config: Arc<Config>,
    pub session_info: Arc<SessionGet>,
    pub action_tx: UnboundedSender<Action>,
    update_tx: UnboundedSender<UpdateAction>,
    trans_tx: UnboundedSender<TorrentAction>,
}

impl Ctx {
    async fn new(
        client: &mut TransClient,
        config: Config,
        action_tx: UnboundedSender<Action>,
        update_tx: UnboundedSender<UpdateAction>,
        trans_tx: UnboundedSender<TorrentAction>,
    ) -> Result<Self> {
        let response = client.session_get().await;
        match response {
            Ok(res) => {
                let session_info = Arc::new(res.arguments);
                Ok(Self {
                    config: Arc::new(config),
                    action_tx,
                    trans_tx,
                    update_tx,
                    session_info,
                })
            }
            Err(e) => {
                let config_path = config.directories.main_path;
                Err(Error::msg(format!(
                    "{e}\nIs the connection info in {:?} correct?",
                    config_path
                )))
            }
        }
    }

    pub(crate) fn send_action(&self, action: Action) {
        self.action_tx.send(action).unwrap();
    }

    pub(crate) fn send_torrent_action(&self, action: TorrentAction) {
        self.trans_tx.send(action).unwrap();
    }

    pub(crate) fn send_update_action(&self, action: UpdateAction) {
        self.update_tx.send(action).unwrap();
    }
}

pub struct App {
    should_quit: bool,
    ctx: Ctx,
    action_rx: UnboundedReceiver<Action>,
    update_rx: UnboundedReceiver<UpdateAction>,
    main_window: MainWindow,
    mode: Mode,
}

impl App {
    pub async fn new(config: Config) -> Result<Self> {
        let (action_tx, action_rx) = mpsc::unbounded_channel();
        let (update_tx, update_rx) = mpsc::unbounded_channel();

        let mut client = transmission::utils::client_from_config(&config);

        let (trans_tx, trans_rx) = mpsc::unbounded_channel();
        let ctx = Ctx::new(
            &mut client,
            config,
            action_tx.clone(),
            update_tx.clone(),
            trans_tx,
        )
        .await?;

        tokio::spawn(transmission::action_handler(client, trans_rx, update_tx));

        Ok(Self {
            should_quit: false,
            main_window: MainWindow::new(ctx.clone()),
            action_rx,
            update_rx,
            ctx,
            mode: Mode::Normal,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut tui = Tui::new()?;

        tui.enter()?;

        self.render(&mut tui)?;

        self.main_loop(&mut tui).await?;

        tui.exit()?;
        Ok(())
    }

    async fn main_loop(&mut self, tui: &mut Tui) -> Result<()> {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(250));
        loop {
            let tui_event = tui.next();
            let action = self.action_rx.recv();
            let update_action = self.update_rx.recv();
            let tick_action = interval.tick();

            tokio::select! {
                _ = tick_action => self.tick(),

                event = tui_event => {
                    event_to_action(self.mode, event.unwrap(), &self.ctx.action_tx, &self.ctx.config.keybindings.keymap);
                },

                update_action = update_action => self.handle_update_action(update_action.unwrap()).await,

                action = action => {
                    if let Some(action) = action {
                        if action.is_render() {
                            self.render(tui)?;
                        } else if action.is_quit() {
                            self.should_quit = true;
                        } else {
                            self.handle_user_action(action).await
                        }
                    }
                }
            }

            if self.should_quit {
                break Ok(());
            }
        }
    }

    fn render(&mut self, tui: &mut Tui) -> Result<()> {
        tui.terminal.draw(|f| {
            self.main_window.render(f, f.size());
        })?;
        Ok(())
    }

    #[must_use]
    async fn handle_user_action(&mut self, action: Action) {
        use Action as A;
        match &action {
            A::HardQuit => {
                self.should_quit = true;
            }

            _ => {
                self.main_window.handle_actions(action);
            }
        }
    }

    async fn handle_update_action(&mut self, action: UpdateAction) {
        match action {
            UpdateAction::SwitchToInputMode => {
                self.mode = Mode::Input;
            }
            UpdateAction::SwitchToNormalMode => {
                self.mode = Mode::Normal;
            }

            _ => self.main_window.handle_update_action(action),
        };
        self.ctx.send_action(Action::Render);
    }

    fn tick(&mut self) {
        self.main_window.tick();
    }
}
