use std::{
    io::stdout,
    panic::{set_hook, take_hook},
};

use crate::{
    transmission::{self, TorrentAction},
    tui::{app_key_event::AppKeyEvent, components::Component},
};

use intuitils::Terminal;
use rm_shared::action::{Action, UpdateAction};

use color_eyre::{
    eyre::{self},
    Result, Section,
};
use crossterm::{
    cursor::Show,
    event::DisableMouseCapture,
    execute,
    terminal::{disable_raw_mode, LeaveAlternateScreen},
};
use tokio::sync::{mpsc::UnboundedReceiver, oneshot};

use super::{
    ctx::{CTX, CTX_RAW},
    main_window::MainWindow,
    tabs::torrents::SESSION_GET,
};

pub struct App {
    should_quit: bool,
    action_rx: UnboundedReceiver<Action>,
    update_rx: UnboundedReceiver<UpdateAction>,
    main_window: MainWindow,
    mode: Mode,
}

impl App {
    pub async fn new() -> Result<Self> {
        let client = transmission::utils::new_client();

        let (action_rx, update_rx, torrent_rx) = CTX_RAW
            .1
            .lock()
            .unwrap()
            .take()
            .expect("it wasn't taken before");

        tokio::spawn(transmission::action_handler(
            client,
            torrent_rx,
            CTX.update_tx.clone(),
        ));

        tokio::spawn(async move {
            let (sess_tx, sess_rx) = oneshot::channel();

            CTX.send_torrent_action(TorrentAction::GetSessionGet(sess_tx));
            match sess_rx.await.unwrap() {
                Ok(sess_get) => SESSION_GET.set(sess_get).unwrap(),
                Err(e) => CTX.send_update_action(UpdateAction::UnrecoverableError(Box::new(
                    eyre::eyre!(e.source).wrap_err("error connecting to transmission daemon")
                        .suggestion("Check if the transmission daemon IP address is correct and ensure you have an internet connection."),
                ))),
            }
        });

        Ok(Self {
            should_quit: false,
            main_window: MainWindow::new(),
            action_rx,
            update_rx,
            mode: Mode::Normal,
        })
    }

    pub async fn run(mut self) -> Result<()> {
        let mut terminal = Terminal::new()?;

        let original_hook = take_hook();

        set_hook(Box::new(move |panic_info| {
            let _ = disable_raw_mode();
            let _ = execute!(stdout(), LeaveAlternateScreen, Show, DisableMouseCapture);
            original_hook(panic_info);
        }));

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

            let current_window = self.main_window.current_window();

            tokio::select! {
                _ = tick_action => self.tick(),

                event = tui_event => {
                    let event = event.unwrap();

                    use crossterm::event::{Event, MouseEventKind};
                    match event {
                        Event::Key(key_event) => {
                            let app_key_event = AppKeyEvent::from(key_event);
                            if app_key_event.is_ctrl_c() {
                                self.should_quit = true;
                            } else if self.mode == Mode::Input {
                                self.handle_user_action(Action::Input(key_event));
                            } else if let Some(action) = app_key_event.to_action(current_window) {
                                self.handle_user_action(action);
                            }
                        },
                        Event::Mouse(mouse_event) => match mouse_event.kind {
                            MouseEventKind::ScrollDown => self.handle_user_action(Action::ScrollDownBy(3)),
                            MouseEventKind::ScrollUp => self.handle_user_action(Action::ScrollUpBy(3)),
                            _ => (),
                        },
                        Event::Resize(_, _) => self.render(terminal).unwrap(),
                        _ => (),
                    }
                },

                update_action = update_action => self.handle_update_action(update_action.unwrap()).await?,

                action = action => {
                    if let Some(action) = action {
                        if action.is_render() {
                            self.render(terminal)?;
                        } else {
                            self.handle_user_action(action);
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
        tokio::task::block_in_place(|| {
            terminal
                .draw(|f| {
                    self.main_window.render(f, f.area());
                })
                .unwrap();
        });
        Ok(())
    }

    fn handle_user_action(&mut self, action: Action) {
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

    async fn handle_update_action(&mut self, action: UpdateAction) -> Result<()> {
        match action {
            UpdateAction::UnrecoverableError(report) => return Err(*report),
            UpdateAction::SwitchToInputMode => {
                self.mode = Mode::Input;
            }
            UpdateAction::SwitchToNormalMode => {
                self.mode = Mode::Normal;
            }

            _ => self.main_window.handle_update_action(action),
        };
        CTX.send_action(Action::Render);
        Ok(())
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
