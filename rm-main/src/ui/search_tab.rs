use std::{
    borrow::Cow,
    sync::{Arc, Mutex},
};

use crossterm::event::KeyCode;
use magnetease::{magnetease::Magnetease, Magnet};
use ratatui::{
    layout::Flex,
    prelude::*,
    widgets::{Cell, Paragraph, Row, Table},
};
use tokio::sync::mpsc::{self, UnboundedSender};
use tui_input::Input;

use crate::action::Action;

use super::{
    bytes_to_human_format,
    components::{table::GenericTable, Component},
    to_input_request,
};

enum SearchFocus {
    Search,
    List,
}

pub(super) struct SearchTab {
    search_focus: SearchFocus,
    input: Input,
    req_sender: UnboundedSender<String>,
    table: Arc<Mutex<GenericTable<Magnet>>>,
}

impl SearchTab {
    pub(super) fn new(action_tx: UnboundedSender<Action>) -> Self {
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
            Cell::from(Cow::Owned(magnet.seeders.to_string())).light_green(),
            Cell::from(Cow::Borrowed(&*magnet.title)),
            Cell::from(Cow::Owned(size)),
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
