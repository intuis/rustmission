use std::sync::Arc;

use crate::{
    transmission::{self, TorrentAction},
    tui::components::Component,
};

use rm_config::CONFIG;
use rm_shared::action::{Action, UpdateAction};

use anyhow::{Error, Result};
use crossterm::event::{Event, KeyCode, KeyModifiers};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use transmission_rpc::{types::SessionGet, TransClient};

use super::{components::tabs::CurrentTab, main_window::MainWindow, terminal::Tui};

#[derive(Clone)]
pub struct Ctx {
    pub session_info: Arc<SessionGet>,
    action_tx: UnboundedSender<Action>,
    update_tx: UnboundedSender<UpdateAction>,
    trans_tx: UnboundedSender<TorrentAction>,
}

impl Ctx {
    async fn new(
        client: &mut TransClient,
        action_tx: UnboundedSender<Action>,
        update_tx: UnboundedSender<UpdateAction>,
        trans_tx: UnboundedSender<TorrentAction>,
    ) -> Result<Self> {
        let response = client.session_get().await;
        match response {
            Ok(res) => {
                let session_info = Arc::new(res.arguments);
                Ok(Self {
                    action_tx,
                    trans_tx,
                    update_tx,
                    session_info,
                })
            }
            Err(e) => {
                let config_path = CONFIG.directories.main_path;
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
    pub async fn new() -> Result<Self> {
        let (action_tx, action_rx) = mpsc::unbounded_channel();
        let (update_tx, update_rx) = mpsc::unbounded_channel();

        let mut client = transmission::utils::new_client();

        let (trans_tx, trans_rx) = mpsc::unbounded_channel();
        let ctx = Ctx::new(&mut client, action_tx.clone(), update_tx.clone(), trans_tx).await?;

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

            let current_tab = self.main_window.tabs.current_tab;

            tokio::select! {
                _ = tick_action => self.tick(),

                event = tui_event => {
                    event_to_action(&self.ctx, self.mode, current_tab, event.unwrap());
                },

                update_action = update_action => self.handle_update_action(update_action.unwrap()).await,

                action = action => {
                    if let Some(action) = action {
                        if action.is_render() {
                            self.render(tui)?;
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

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Input,
    Normal,
}

pub fn event_to_action(ctx: &Ctx, mode: Mode, current_tab: CurrentTab, event: Event) {
    // Handle CTRL+C first
    if let Event::Key(key_event) = event {
        if key_event.modifiers == KeyModifiers::CONTROL
            && (key_event.code == KeyCode::Char('c') || key_event.code == KeyCode::Char('C'))
        {
            ctx.send_action(Action::HardQuit);
        }
    }

    match event {
        Event::Key(key) if mode == Mode::Input => ctx.send_action(Action::Input(key)),
        Event::Key(key) => {
            let keymaps = match current_tab {
                CurrentTab::Torrents => [
                    &CONFIG.keybindings.general_keymap,
                    &CONFIG.keybindings.torrent_keymap,
                ],
                CurrentTab::Search => [
                    &CONFIG.keybindings.general_keymap,
                    &CONFIG.keybindings.search_keymap,
                ],
            };

            let keybinding = match key.code {
                KeyCode::Char(e) => {
                    let modifier = if e.is_uppercase() {
                        KeyModifiers::NONE
                    } else {
                        key.modifiers
                    };
                    (key.code, modifier)
                }
                _ => (key.code, key.modifiers),
            };

            for keymap in keymaps {
                if let Some(action) = keymap.get(&keybinding).cloned() {
                    ctx.send_action(action);
                    return;
                }
            }
        }
        Event::Resize(_, _) => ctx.send_action(Action::Render),
        _ => (),
    }
}
