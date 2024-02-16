pub mod components;

use std::sync::{Arc, Mutex};

use crossterm::event::KeyCode;
use magnetease::{magnetease::Magnetease, Magnet};
use ratatui::{
    layout::Flex,
    prelude::*,
    widgets::{Block, Cell, Clear, Paragraph, Row, Table, Wrap},
};
use tokio::sync::mpsc::{self, UnboundedSender};
use tui_input::{Input, InputRequest};

use crate::action::Action;

use self::components::{
    tabcomponent::CurrentTab, table::GenericTable, Component, TabComponent, TorrentsTab,
};

fn centered_rect(r: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(r);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(popup_layout[1])[1]
}

#[derive(Default)]
struct Pipup {
    error_popup: Option<ErrorPopup>,
    help_popup: Option<HelpPopup>,
}

impl Pipup {
    fn needs_action(&self) -> bool {
        self.error_popup.is_some() || self.help_popup.is_some()
    }
}

impl Component for Pipup {
    fn handle_events(&mut self, action: Action) -> Option<Action> {
        if let Some(popup) = &mut self.error_popup {
            if let Some(Action::Quit) = popup.handle_events(action) {
                self.error_popup = None;
                return Some(Action::Render);
            }
            None
        } else if let Some(popup) = &mut self.help_popup {
            if let Some(Action::Quit) = popup.handle_events(action) {
                self.help_popup = None;
                return Some(Action::Render);
            }
            None
        } else {
            None
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        if let Some(popup) = &mut self.error_popup {
            popup.render(f, rect)
        } else if let Some(popup) = &mut self.help_popup {
            popup.render(f, rect);
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ErrorPopup {
    // TODO: make sure that title always has padding
    title: String,
    message: String,
}

impl ErrorPopup {
    pub(crate) fn new(title: &'static str, message: String) -> Self {
        Self {
            title: title.to_owned(),
            message,
        }
    }
}

impl Component for ErrorPopup {
    fn handle_events(&mut self, action: Action) -> Option<Action> {
        if let Action::Confirm = action {
            return Some(Action::Quit);
        }
        None
    }

    fn render(&mut self, f: &mut Frame, _rect: Rect) {
        let centered_rect = centered_rect(f.size(), 50, 50);
        let popup_rect = centered_rect.inner(&Margin::new(1, 1));
        let text_rect = popup_rect.inner(&Margin::new(3, 2));
        let button_rect = {
            let temp_rect = Layout::vertical([Constraint::Percentage(100), Constraint::Length(1)])
                .split(text_rect)[1];
            temp_rect
        };

        let button = Paragraph::new("[ OK ]").bold().right_aligned();

        let block = Block::bordered()
            .border_set(symbols::border::ROUNDED)
            .title_style(Style::new().red())
            .title(format!(" {} ", self.title));

        let error_message = Paragraph::new(&*self.message).wrap(Wrap { trim: false });

        f.render_widget(Clear, centered_rect);
        f.render_widget(block, popup_rect);
        f.render_widget(error_message, text_rect);
        f.render_widget(button, button_rect);
    }
}

struct HelpPopup;

impl Component for HelpPopup {
    fn handle_events(&mut self, action: Action) -> Option<Action> {
        if let Action::Quit = action {
            return Some(Action::Quit);
        }
        None
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let centered_rect = centered_rect(rect, 75, 75);
        let popup_rect = centered_rect.inner(&Margin::new(1, 1));
        let text_rect = popup_rect.inner(&Margin::new(3, 2));

        let block = Block::bordered()
            .border_set(symbols::border::ROUNDED)
            .title_style(Style::new().red())
            .title("Help");

        let global_headline =
            Line::styled("GLOBAL KEYBINDINGS", Style::new().bold().underlined()).centered();

        f.render_widget(Clear, centered_rect);
        f.render_widget(block, popup_rect);
        f.render_widget(global_headline, text_rect);
    }
}

pub struct MainWindow {
    tabs: TabComponent,
    torrents_tab: TorrentsTab,
    search_tab: SearchTab,
    popup: Pipup,
}

enum SearchFocus {
    Search,
    List,
}

struct SearchTab {
    search_focus: SearchFocus,
    input: Input,
    req_sender: UnboundedSender<String>,
    table: Arc<Mutex<GenericTable<Magnet>>>,
}

impl SearchTab {
    fn new(action_tx: UnboundedSender<Action>) -> Self {
        let (tx, mut rx) = mpsc::unbounded_channel::<String>();
        let table = Arc::new(Mutex::new(GenericTable::new(vec![])));
        let table_clone = Arc::clone(&table);
        tokio::task::spawn(async move {
            let magnetease = Magnetease::new();
            while let Some(search_phrase) = rx.recv().await {
                let res = magnetease.search(&search_phrase).await;
                let _ = table_clone.lock().unwrap().set_items(res);
                action_tx.send(Action::Render).unwrap();
            }
        });

        Self {
            search_focus: SearchFocus::List,
            input: Input::default(),
            table,
            req_sender: tx,
        }
    }

    fn magnet_to_row(magnet: &Magnet) -> Row {
        let size = bytes_to_human_format(magnet.bytes as i64);
        // TODO: use cow
        Row::new([
            Cell::from(magnet.seeders.to_string()).light_green(),
            Cell::from(magnet.title.to_string()),
            Cell::from(size),
        ])
    }

    fn change_focus(&mut self) {
        if let SearchFocus::Search = self.search_focus {
            self.search_focus = SearchFocus::List;
        } else {
            self.search_focus = SearchFocus::Search;
        }
    }
}

impl Component for SearchTab {
    fn handle_events(&mut self, action: Action) -> Option<Action> {
        match action {
            Action::Search => {
                self.search_focus = SearchFocus::Search;
                Some(Action::SwitchToInputMode)
            }
            Action::ChangeFocus => {
                self.change_focus();
                Some(Action::Render)
            }
            Action::Input(input) => {
                if input.code == KeyCode::Enter {
                    self.req_sender.send(self.input.to_string()).unwrap();
                    self.search_focus = SearchFocus::List;
                    return Some(Action::SwitchToNormalMode);
                }
                if input.code == KeyCode::Esc {
                    self.search_focus = SearchFocus::List;
                    return Some(Action::SwitchToNormalMode);
                }

                if let Some(req) = to_input_request(input.code) {
                    self.input.handle(req);
                    return Some(Action::Render);
                }

                None
            }
            Action::Down => {
                self.table.lock().unwrap().next();
                Some(Action::Render)
            }
            _ => None,
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let [top_line, rest] =
            Layout::vertical([Constraint::Length(1), Constraint::Percentage(100)]).areas(rect);

        let search_rect = Layout::horizontal([
            Constraint::Percentage(25),
            Constraint::Percentage(50),
            Constraint::Percentage(25),
        ])
        .flex(Flex::Center)
        .split(top_line)[1];

        let input = {
            if self.input.value().is_empty() && !matches!(self.search_focus, SearchFocus::Search) {
                "press / to search"
            } else {
                self.input.value()
            }
        };

        let search_style = {
            if let SearchFocus::Search = self.search_focus {
                Style::default().light_magenta().underlined()
            } else {
                Style::default().gray().underlined()
            }
        };

        let paragraph_text = format!(" {input}");
        let prefix_len = paragraph_text.len() - input.len() - 2;
        let paragraph = Paragraph::new(paragraph_text).style(search_style);

        f.render_widget(paragraph, search_rect);

        let cursor_offset = self.input.visual_cursor() + prefix_len;
        f.set_cursor(
            search_rect.x + u16::try_from(cursor_offset).unwrap(),
            search_rect.y,
        );

        let widths = [
            Constraint::Length(5),
            Constraint::Percentage(40),
            Constraint::Length(8),
        ];
        let header = Row::new(["S", "Title", "Size"]);

        let table_lock = self.table.lock().unwrap();
        let table_items = &table_lock.items;
        let items = table_items.iter().map(Self::magnet_to_row);
        let table = Table::new(items, widths)
            .header(header)
            .highlight_style(Style::default().light_magenta().on_black().bold());

        let mut table_state = table_lock.state.clone();

        f.render_stateful_widget(table, rest, &mut table_state)
    }
}

impl MainWindow {
    pub fn new(action_tx: UnboundedSender<Action>, trans_tx: UnboundedSender<Action>) -> Self {
        Self {
            tabs: TabComponent::new(),
            torrents_tab: TorrentsTab::new(trans_tx),
            search_tab: SearchTab::new(action_tx),
            popup: Pipup::default(),
        }
    }
}

impl Component for MainWindow {
    fn handle_events(&mut self, action: Action) -> Option<Action> {
        if let Action::Error(e_popup) = action {
            self.popup.error_popup = Some(*e_popup);
            return Some(Action::Render);
        }

        if let Action::ShowHelp = action {
            self.popup.help_popup = Some(HelpPopup);
            return Some(Action::Render);
        }

        if let Action::ChangeTab(_) = action {
            self.tabs.handle_events(action);
            return Some(Action::Render);
        }

        if self.popup.needs_action() {
            return self.popup.handle_events(action);
        } else {
            match self.tabs.current_tab {
                CurrentTab::Torrents => return self.torrents_tab.handle_events(action),
                CurrentTab::Search => self.search_tab.handle_events(action),
                CurrentTab::Settings => todo!(),
            }
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let [top_bar, main_window] =
            Layout::vertical([Constraint::Length(1), Constraint::Percentage(100)]).areas(rect);

        self.tabs.render(f, top_bar);

        match self.tabs.current_tab {
            CurrentTab::Torrents => self.torrents_tab.render(f, main_window),
            CurrentTab::Search => self.search_tab.render(f, main_window),
            CurrentTab::Settings => todo!(),
        }

        self.popup.render(f, f.size());
    }
}

const fn to_input_request(keycode: KeyCode) -> Option<InputRequest> {
    use InputRequest as R;

    match keycode {
        KeyCode::Backspace => Some(R::DeletePrevChar),
        KeyCode::Delete => Some(R::DeleteNextChar),
        KeyCode::Char(char) => Some(R::InsertChar(char)),
        _ => None,
    }
}

pub fn bytes_to_human_format(bytes: i64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    const TB: f64 = GB * 1024.0;

    if bytes == 0 {
        return "0 B".to_string();
    }

    let (value, unit) = if bytes < (KB - 25f64) as i64 {
        (bytes as f64, "B")
    } else if bytes < (MB - 25f64) as i64 {
        (bytes as f64 / KB, "KB")
    } else if bytes < (GB - 25f64) as i64 {
        (bytes as f64 / MB, "MB")
    } else if bytes < (TB - 25f64) as i64 {
        (bytes as f64 / GB, "GB")
    } else {
        (bytes as f64 / TB, "TB")
    };

    format!("{value:.1} {unit}")
}
