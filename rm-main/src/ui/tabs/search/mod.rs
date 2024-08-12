mod bottom_bar;
mod popups;

use std::{
    borrow::{BorrowMut, Cow},
    sync::Arc,
};

use bottom_bar::BottomBar;
use crossterm::event::{KeyCode, KeyEvent};
use futures::{stream::FuturesUnordered, StreamExt};
use magnetease::{Magnet, MagneteaseErrorKind, WhichProvider};
use popups::{CurrentPopup, PopupManager};
use ratatui::{
    layout::Flex,
    prelude::*,
    widgets::{Cell, Paragraph, Row, Table},
};
use reqwest::Client;
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
    action::{Action, UpdateAction},
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
    popup_manager: PopupManager,
    configured_providers: Vec<ConfiguredProvider>,
    bottom_bar: BottomBar,
    currently_displaying_no: u16,
    ctx: app::Ctx,
}

impl SearchTab {
    pub(crate) fn new(ctx: app::Ctx) -> Self {
        let (search_query_tx, mut search_query_rx) = mpsc::unbounded_channel::<String>();
        let table = GenericTable::new(vec![]);

        let providers = vec![WhichProvider::Knaben, WhichProvider::Nyaa];

        let mut configured_providers = vec![];

        for provider in providers.clone() {
            configured_providers.push(ConfiguredProvider::new(provider));
        }

        let bottom_bar = BottomBar::new(ctx.clone(), &configured_providers);

        tokio::task::spawn({
            let ctx = ctx.clone();
            async move {
                let client = Client::new();
                while let Some(phrase) = search_query_rx.recv().await {
                    ctx.send_update_action(UpdateAction::SearchStarted);
                    let mut futures = FuturesUnordered::new();
                    for provider in &providers {
                        futures.push(provider.search(&client, &phrase));
                    }

                    while let Some(result) = futures.next().await {
                        match result {
                            Ok(response) => {
                                ctx.send_update_action(UpdateAction::ProviderResult(response))
                            }
                            Err(e) => ctx.send_update_action(UpdateAction::ProviderError(e)),
                        }
                    }
                    ctx.send_update_action(UpdateAction::SearchFinished);
                }
            }
        });

        Self {
            focus: SearchTabFocus::List,
            input: Input::default(),
            table,
            bottom_bar,
            search_query_rx: search_query_tx,
            currently_displaying_no: 0,
            popup_manager: PopupManager::new(ctx.clone()),
            ctx,
            configured_providers,
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

    fn show_providers_info(&mut self) {
        self.popup_manager
            .show_providers_info_popup(self.configured_providers.clone());
        self.ctx.send_action(Action::Render);
    }

    fn providers_searching(&mut self) {
        for configured_provider in &mut self.configured_providers {
            configured_provider.provider_state = ProviderState::Searching;
        }
    }

    fn provider_state_success(&mut self, provider: WhichProvider, results_count: u16) {
        for configured_provider in &mut self.configured_providers {
            if configured_provider.provider == provider {
                configured_provider.provider_state = ProviderState::Found(results_count);
                break;
            }
        }
    }

    fn provider_state_error(&mut self, provider: WhichProvider, error: MagneteaseErrorKind) {
        for configured_provider in &mut self.configured_providers {
            if configured_provider.provider == provider {
                configured_provider.provider_state = ProviderState::Error(Arc::new(error));
                break;
            }
        }
    }

    fn update_providers_popup(&mut self) {
        if let Some(CurrentPopup::Providers(popup)) = &mut self.popup_manager.current_popup {
            popup.update_providers(self.configured_providers.clone());
        }
    }
}

impl Component for SearchTab {
    fn handle_actions(&mut self, action: Action) -> ComponentAction {
        use Action as A;

        if self.popup_manager.is_showing_popup() {
            self.popup_manager.handle_actions(action);
            return ComponentAction::Nothing;
        }

        if action.is_quit() {
            self.ctx.send_action(Action::HardQuit);
        }

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
            A::ShowProvidersInfo => self.show_providers_info(),

            _ => (),
        };
        ComponentAction::Nothing
    }

    fn handle_update_action(&mut self, action: UpdateAction) {
        match action {
            UpdateAction::SearchStarted => {
                self.providers_searching();

                self.table.items.drain(..);
                self.table.state.borrow_mut().select(None);

                self.bottom_bar
                    .handle_update_action(UpdateAction::SearchStarted);
                self.update_providers_popup();
            }
            UpdateAction::ProviderResult(response) => {
                self.provider_state_success(
                    response.provider,
                    u16::try_from(response.magnets.len()).unwrap(),
                );

                self.bottom_bar
                    .search_state
                    .update_counts(&self.configured_providers);
                self.update_providers_popup();

                self.table.items.extend(response.magnets);
                self.table.items.sort_by(|a, b| b.seeders.cmp(&a.seeders));

                let mut state = self.table.state.borrow_mut();
                if !self.table.items.is_empty() && state.selected().is_none() {
                    state.select(Some(0))
                }
            }
            UpdateAction::ProviderError(e) => {
                self.provider_state_error(e.provider, e.kind);

                self.bottom_bar
                    .search_state
                    .update_counts(&self.configured_providers);
                self.update_providers_popup();
            }
            UpdateAction::SearchFinished => {
                if self.table.items.is_empty() {
                    self.bottom_bar.search_state.not_found();
                } else {
                    self.bottom_bar.search_state.found(self.table.items.len());
                }
            }
            _ => (),
        }
    }

    fn tick(&mut self) {
        self.bottom_bar.tick();
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

        self.bottom_bar.render(f, bottom_line);
        self.popup_manager.render(f, f.size());
    }
}

#[derive(Clone)]
pub struct ConfiguredProvider {
    provider: WhichProvider,
    provider_state: ProviderState,
    enabled: bool,
}

impl ConfiguredProvider {
    fn new(provider: WhichProvider) -> Self {
        Self {
            provider,
            provider_state: ProviderState::Idle,
            enabled: true,
        }
    }
}

#[derive(Clone)]
enum ProviderState {
    Idle,
    Searching,
    Found(u16),
    Error(Arc<MagneteaseErrorKind>),
}
