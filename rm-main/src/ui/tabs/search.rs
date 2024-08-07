use std::borrow::Cow;

use crossterm::event::{KeyCode, KeyEvent};
use futures::{stream::FuturesUnordered, StreamExt};
use magnetease::{Magnet, Magnetease, Provider};
use ratatui::{
    layout::Flex,
    prelude::*,
    widgets::{Cell, Paragraph, Row, Table},
};
use reqwest::Client;
use throbber_widgets_tui::ThrobberState;
use tokio::sync::mpsc::{self, UnboundedSender};
use tui_input::Input;

use crate::{
    app,
    transmission::TorrentAction,
    ui::{
        components::{table::GenericTable, Component, ComponentAction},
        to_input_request,
    },
};
use rm_shared::{
    action::{Action, ErrorMessage, UpdateAction},
    utils::bytes_to_human_format,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum SearchTabFocus {
    Search,
    List,
}

pub(crate) struct SearchTab {
    focus: SearchTabFocus,
    input: Input,
    search_query_rx: UnboundedSender<String>,
    table: GenericTable<Magnet>,
    // TODO: Change it to enum, and combine table with search_result_info
    search_result_info: SearchResultState,
    currently_displaying_no: u16,
    ctx: app::Ctx,
}

impl SearchTab {
    pub(crate) fn new(ctx: app::Ctx) -> Self {
        let (search_query_tx, mut search_query_rx) = mpsc::unbounded_channel::<String>();
        let table = GenericTable::new(vec![]);
        let search_result_info = SearchResultState::new(ctx.clone());

        tokio::task::spawn({
            let ctx = ctx.clone();
            async move {
                let knaben = magnetease::providers::Knaben;
                let nyaa = magnetease::providers::Nyaa;
                let client = Client::new();
                while let Some(phrase) = search_query_rx.recv().await {
                    ctx.send_update_action(UpdateAction::SearchStarted);
                    let mut futures = FuturesUnordered::new();
                    futures.push(knaben.search(&client, &phrase));
                    futures.push(nyaa.search(&client, &phrase));

                    while let Some(result) = futures.next().await {
                        match result {
                            Ok(magnets) => {
                                ctx.send_update_action(UpdateAction::SearchResults(magnets))
                            }
                            Err(e) => {
                                let e_title = "Failed to search for magnets";
                                let e_desc = "Provider error";
                                let e_msg = ErrorMessage::new(e_title, e_desc, Box::new(e));
                                ctx.send_update_action(UpdateAction::Error(Box::new(e_msg)));
                            }
                        }
                    }
                    ctx.send_update_action(UpdateAction::SearchFinished);
                }

                let magnetease = Magnetease::new();
                while let Some(search_phrase) = search_query_rx.recv().await {
                    ctx.send_update_action(UpdateAction::SearchStarted);
                    let magnets = magnetease.search(&search_phrase).await.unwrap();
                    ctx.send_update_action(UpdateAction::SearchResults(magnets));
                }
            }
        });

        Self {
            focus: SearchTabFocus::List,
            input: Input::default(),
            table,
            search_result_info,
            search_query_rx: search_query_tx,
            currently_displaying_no: 0,
            ctx,
        }
    }

    fn magnet_to_row(magnet: &Magnet) -> Row {
        let size = bytes_to_human_format(magnet.bytes as i64);
        Row::new([
            Cell::from(Cow::Owned(magnet.seeders.to_string())).light_green(),
            Cell::from(Cow::Borrowed(&*magnet.title)),
            Cell::from(Cow::Owned(size)),
        ])
    }

    fn change_focus(&mut self) {
        if self.focus == SearchTabFocus::Search {
            self.focus = SearchTabFocus::List;
        } else {
            self.focus = SearchTabFocus::Search;
        }
        self.ctx.send_action(Action::Render);
    }

    fn add_torrent(&mut self) {
        let magnet_url = self.table.current_item().map(|magnet| magnet.url);
        if let Some(magnet_url) = magnet_url {
            self.ctx
                .send_torrent_action(TorrentAction::Add(magnet_url, None));
        }
    }

    fn handle_input(&mut self, input: KeyEvent) {
        use Action as A;

        match input.code {
            KeyCode::Enter => {
                self.search_query_rx.send(self.input.to_string()).unwrap();
                self.focus = SearchTabFocus::List;
                self.ctx
                    .send_update_action(UpdateAction::SwitchToNormalMode);
            }
            KeyCode::Esc => {
                self.focus = SearchTabFocus::List;
                self.ctx
                    .send_update_action(UpdateAction::SwitchToNormalMode);
            }
            _ => {
                if let Some(req) = to_input_request(input) {
                    self.input.handle(req);
                    self.ctx.send_action(A::Render);
                }
            }
        }
    }

    fn start_search(&mut self) {
        self.focus = SearchTabFocus::Search;
        self.ctx.send_update_action(UpdateAction::SwitchToInputMode);
    }

    fn next_torrent(&mut self) {
        self.table.next();
        self.ctx.send_action(Action::Render);
    }

    fn previous_torrent(&mut self) {
        self.table.previous();
        self.ctx.send_action(Action::Render);
    }

    fn scroll_down_page(&mut self) {
        self.table
            .scroll_down_by(self.currently_displaying_no as usize);
        self.ctx.send_action(Action::Render);
    }

    fn scroll_up_page(&mut self) {
        self.table
            .scroll_up_by(self.currently_displaying_no as usize);
        self.ctx.send_action(Action::Render);
    }

    fn scroll_to_end(&mut self) {
        self.table.scroll_to_end();
        self.ctx.send_action(Action::Render);
    }

    fn scroll_to_home(&mut self) {
        self.table.scroll_to_home();
        self.ctx.send_action(Action::Render);
    }

    fn xdg_open(&mut self) {
        if let Some(magnet) = self.table.current_item() {
            let _ = open::that_detached(&magnet.url);
        }
    }
}

impl Component for SearchTab {
    fn handle_actions(&mut self, action: Action) -> ComponentAction {
        use Action as A;
        match action {
            A::Quit => self.ctx.send_action(Action::Quit),
            A::Search => self.start_search(),
            A::ChangeFocus => self.change_focus(),
            A::Input(input) => self.handle_input(input),
            A::Down => self.next_torrent(),
            A::Up => self.previous_torrent(),
            A::ScrollDownPage => self.scroll_down_page(),
            A::ScrollUpPage => self.scroll_up_page(),
            A::Home => self.scroll_to_home(),
            A::End => self.scroll_to_end(),
            A::Confirm => self.add_torrent(),
            A::XdgOpen => self.xdg_open(),

            _ => (),
        };
        ComponentAction::Nothing
    }

    fn handle_update_action(&mut self, action: UpdateAction) {
        match action {
            UpdateAction::SearchStarted => {
                self.search_result_info.searching(ThrobberState::default());
                self.table.items.drain(..);
            }
            UpdateAction::SearchResults(magnets) => {
                self.table.items.extend(magnets);
                self.table.items.sort_by(|a, b| b.seeders.cmp(&a.seeders));
            }
            UpdateAction::SearchFinished => {
                if self.table.items.is_empty() {
                    self.search_result_info.not_found();
                } else {
                    self.search_result_info.found(self.table.items.len());
                }
            }
            _ => (),
        }
    }

    fn tick(&mut self) {
        self.search_result_info.tick();
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let [top_line, rest, bottom_line] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Percentage(100),
            Constraint::Length(1),
        ])
        .areas(rect);

        self.currently_displaying_no = rest.height;

        let search_rect = Layout::horizontal([
            Constraint::Percentage(25),
            Constraint::Percentage(50),
            Constraint::Percentage(25),
        ])
        .flex(Flex::Center)
        .split(top_line)[1];

        let input = {
            if self.input.value().is_empty() && self.focus != SearchTabFocus::Search {
                "press / to search"
            } else {
                self.input.value()
            }
        };

        let search_style = {
            if self.focus == SearchTabFocus::Search {
                Style::default()
                    .underlined()
                    .fg(self.ctx.config.general.accent_color)
            } else {
                Style::default().underlined().gray()
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

        let header = Row::new(["S", "Title", "Size"]);

        let table_items = &self.table.items;

        let longest_title = table_items.iter().map(|magnet| magnet.title.len()).max();
        let items = table_items.iter().map(Self::magnet_to_row);

        let widths = [
            Constraint::Length(5),                                  // Seeders
            Constraint::Length(longest_title.unwrap_or(10) as u16), // Title
            Constraint::Length(8),                                  // Size
        ];

        let table_higlight_style = Style::default().on_black().bold().fg(self
            .ctx
            .config
            .general
            .accent_color);

        let table = {
            let table = Table::new(items, widths).highlight_style(table_higlight_style);
            if !self.ctx.config.general.headers_hide {
                table.header(header)
            } else {
                table
            }
        };

        f.render_stateful_widget(table, rest, &mut self.table.state.borrow_mut());

        self.search_result_info.render(f, bottom_line);
    }
}

#[derive(Clone)]
enum SearchResultStatus {
    Nothing,
    NoResults,
    Searching(ThrobberState),
    Found(usize),
}

#[derive(Clone)]
struct SearchResultState {
    ctx: app::Ctx,
    status: SearchResultStatus,
}

impl SearchResultState {
    fn new(ctx: app::Ctx) -> Self {
        Self {
            ctx,
            status: SearchResultStatus::Nothing,
        }
    }

    fn searching(&mut self, state: ThrobberState) {
        self.status = SearchResultStatus::Searching(state);
    }

    fn not_found(&mut self) {
        self.status = SearchResultStatus::NoResults;
    }

    fn found(&mut self, count: usize) {
        self.status = SearchResultStatus::Found(count);
    }
}

impl Component for SearchResultState {
    fn render(&mut self, f: &mut Frame, rect: Rect) {
        match &mut self.status {
            SearchResultStatus::Nothing => (),
            SearchResultStatus::Searching(ref mut state) => {
                let default_throbber = throbber_widgets_tui::Throbber::default()
                    .label("Searching...")
                    .style(ratatui::style::Style::default().fg(ratatui::style::Color::Yellow));
                f.render_stateful_widget(default_throbber.clone(), rect, state);
            }
            SearchResultStatus::NoResults => {
                let mut line = Line::default();
                line.push_span(Span::styled("", Style::default().red()));
                line.push_span(Span::raw(" No results"));
                let paragraph = Paragraph::new(line);
                f.render_widget(paragraph, rect);
            }
            SearchResultStatus::Found(count) => {
                let mut line = Line::default();
                line.push_span(Span::styled("", Style::default().green()));
                line.push_span(Span::raw(format!(" Found {count}")));
                let paragraph = Paragraph::new(line);
                f.render_widget(paragraph, rect);
            }
        }
    }

    fn tick(&mut self) {
        if let SearchResultStatus::Searching(state) = &mut self.status {
            state.calc_next();
            self.ctx.send_action(Action::Render);
        }
    }
}
