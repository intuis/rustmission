use rm_config::Config;
use std::sync::Arc;

use crate::{
    action::{event_to_action, Action, Mode, TorrentAction},
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

#[derive(Clone)]
pub(crate) struct Ctx {
    pub(crate) client: Arc<Mutex<TransClient>>,
    pub(crate) config: Arc<Config>,
    action_tx: UnboundedSender<Action>,
    trans_tx: UnboundedSender<TorrentAction>,
}

impl Ctx {
    fn new(
        client: Arc<Mutex<TransClient>>,
        config: Config,
        action_tx: UnboundedSender<Action>,
        trans_tx: UnboundedSender<TorrentAction>,
    ) -> Self {
        Ctx {
            client,
            config: Arc::new(config),
            action_tx,
            trans_tx,
        }
    }

    pub(crate) fn send_action(&self, action: Action) {
        self.action_tx.send(action).unwrap();
    }

    pub(crate) fn send_torrent_action(&self, action: TorrentAction) {
        self.trans_tx.send(action).unwrap();
    }
}

pub struct App {
    should_quit: bool,
    ctx: Ctx,
    action_rx: UnboundedReceiver<Action>,
    main_window: MainWindow,
    mode: Mode,
}

impl App {
    pub fn new(config: Config) -> Self {
        let user = config
            .connection
            .username
            .as_ref()
            .unwrap_or(&"".to_string())
            .clone();
        let password = config
            .connection
            .password
            .as_ref()
            .unwrap_or(&"".to_string())
            .clone();
        let url = config.connection.url.parse().unwrap();

        let auth = BasicAuth { user, password };

        let (action_tx, action_rx) = mpsc::unbounded_channel();

        let client = Arc::new(Mutex::new(TransClient::with_auth(url, auth)));

        let (trans_tx, trans_rx) = mpsc::unbounded_channel();
        let ctx = Ctx::new(client, config, action_tx, trans_tx);

        tokio::spawn(transmission::action_handler(ctx.clone(), trans_rx));

        Self {
            should_quit: false,
            main_window: MainWindow::new(ctx.clone()),
            action_rx,
            ctx,
            mode: Mode::Normal,
        }
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
        loop {
            let tui_event = tui.next();
            let action = self.action_rx.recv();

            tokio::select! {
                event = tui_event => {
                    if let Some(action) = event_to_action(self.mode, event.unwrap()) {
                        if let Some(action) = self.update(action).await {
                            self.ctx.action_tx.send(action).unwrap();
                        }
                    };
                },

                action = action => {
                    if let Some(action) = action {
                        if action.is_render() {
                            self.render(tui)?;
                        } else if let Some(action) = self.update(action).await {
                            self.ctx.action_tx.send(action).unwrap();
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
    async fn update(&mut self, action: Action) -> Option<Action> {
        use Action as A;
        match &action {
            A::Render => Some(A::Render),

            A::Quit => {
                self.should_quit = true;
                None
            }

            A::SwitchToInputMode => {
                self.mode = Mode::Input;
                Some(A::Render)
            }

            A::SwitchToNormalMode => {
                self.mode = Mode::Normal;
                Some(A::Render)
            }

            _ => self.main_window.handle_actions(action),
        }
    }
}
