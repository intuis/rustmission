use crate::{
    transmission::{self, TorrentAction},
    tui::components::Component,
};

use intuitils::Terminal;
use rm_config::CONFIG;
use rm_shared::action::{Action, UpdateAction};

use anyhow::Result;
use crossterm::event::{Event, KeyCode, KeyModifiers};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

use super::main_window::{CurrentTab, MainWindow};

#[derive(Clone)]
pub struct Ctx {
    action_tx: UnboundedSender<Action>,
    update_tx: UnboundedSender<UpdateAction>,
    trans_tx: UnboundedSender<TorrentAction>,
}

impl Ctx {
    fn new(
        action_tx: UnboundedSender<Action>,
        update_tx: UnboundedSender<UpdateAction>,
        trans_tx: UnboundedSender<TorrentAction>,
    ) -> Self {
        Self {
            action_tx,
            update_tx,
            trans_tx,
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

        let client = transmission::utils::new_client();

        let (trans_tx, trans_rx) = mpsc::unbounded_channel();
        let ctx = Ctx::new(action_tx.clone(), update_tx.clone(), trans_tx);

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

    pub async fn run(mut self) -> Result<()> {
        let mut terminal = Terminal::new()?;

        terminal.init()?;

        self.render(&mut terminal)?;

        self.main_loop(&mut terminal).await?;

        terminal.exit()?;
        Ok(())
    }

    async fn main_loop(mut self, terminal: &mut Terminal) -> Result<()> {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(250));
        loop {
            let tui_event = terminal.next();
            let action = self.action_rx.recv();
            let update_action = self.update_rx.recv();
            let tick_action = interval.tick();

            let current_tab = self.main_window.tabs.current();

            tokio::select! {
                _ = tick_action => self.tick(),

                event = tui_event => {
                    event_to_action(&self.ctx, self.mode, current_tab, event.unwrap());
                },

                update_action = update_action => self.handle_update_action(update_action.unwrap()).await,

                action = action => {
                    if let Some(action) = action {
                        if action.is_render() {
                            tokio::task::block_in_place(|| self.render(terminal).unwrap() );
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

    fn render(&mut self, terminal: &mut Terminal) -> Result<()> {
        terminal.draw(|f| {
            self.main_window.render(f, f.area());
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
        Event::Mouse(mouse_event) => match mouse_event.kind {
            crossterm::event::MouseEventKind::ScrollDown => {
                ctx.send_action(Action::ScrollDownBy(3))
            }
            crossterm::event::MouseEventKind::ScrollUp => ctx.send_action(Action::ScrollUpBy(3)),
            _ => (),
        },
        Event::Key(key) => {
            let keymaps = match current_tab {
                CurrentTab::Torrents => [
                    &CONFIG.keybindings.general.map,
                    &CONFIG.keybindings.torrents_tab.map,
                ],
                CurrentTab::Search => [
                    &CONFIG.keybindings.general.map,
                    &CONFIG.keybindings.search_tab.map,
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
