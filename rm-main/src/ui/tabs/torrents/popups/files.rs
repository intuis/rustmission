use std::{collections::BTreeMap, time::Duration};

use ratatui::{
    prelude::*,
    style::Styled,
    widgets::{
        block::{Position, Title},
        Block, BorderType, Clear, Paragraph,
    },
};
use tokio::{sync::oneshot, task::JoinHandle};
use transmission_rpc::types::{Id, Torrent, TorrentSetArgs};
use tui_tree_widget::{Tree, TreeItem, TreeState};

use crate::{
    app,
    transmission::TorrentAction,
    ui::{
        centered_rect,
        components::{Component, ComponentAction},
    },
};
use rm_shared::{
    action::{Action, ErrorMessage, UpdateAction},
    status_task::StatusTask,
};

pub struct FilesPopup {
    ctx: app::Ctx,
    torrent: Option<Torrent>,
    torrent_id: Id,
    tree_state: TreeState<String>,
    tree: Node,
    current_focus: CurrentFocus,
    switched_after_fetched_data: bool,
    torrent_info_task_handle: JoinHandle<()>,
}

async fn fetch_new_files(ctx: app::Ctx, torrent_id: Id) {
    loop {
        let (torrent_tx, torrent_rx) = oneshot::channel();
        ctx.send_torrent_action(TorrentAction::GetTorrentsById(
            vec![torrent_id.clone()],
            torrent_tx,
        ));

        match torrent_rx.await.unwrap() {
            Ok(mut torrents) => {
                ctx.send_update_action(UpdateAction::UpdateCurrentTorrent(Box::new(
                    torrents.pop().unwrap(),
                )));
            }
            Err(err_message) => {
                ctx.send_update_action(UpdateAction::Error(err_message));
            }
        };

        tokio::time::sleep(Duration::from_secs(6)).await;
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CurrentFocus {
    CloseButton,
    Files,
}

impl FilesPopup {
    pub fn new(ctx: app::Ctx, torrent_id: Id) -> Self {
        let torrent = None;
        let tree_state = TreeState::default();
        let tree = Node::new();

        let torrent_info_task_handle =
            tokio::task::spawn(fetch_new_files(ctx.clone(), torrent_id.clone()));

        Self {
            ctx,
            torrent,
            tree_state,
            tree,
            current_focus: CurrentFocus::CloseButton,
            switched_after_fetched_data: false,
            torrent_id,
            torrent_info_task_handle,
        }
    }

    fn switch_focus(&mut self) {
        match self.current_focus {
            CurrentFocus::CloseButton => self.current_focus = CurrentFocus::Files,
            CurrentFocus::Files => self.current_focus = CurrentFocus::CloseButton,
        }
    }

    fn selected_ids(&self) -> Vec<i32> {
        self.tree_state
            .selected()
            .iter()
            .filter_map(|str_id| str_id.parse::<i32>().ok())
            .collect()
    }
}

impl Component for FilesPopup {
    #[must_use]
    fn handle_actions(&mut self, action: Action) -> ComponentAction {
        use Action as A;
        match (action, self.current_focus) {
            (action, _) if action.is_soft_quit() => {
                self.torrent_info_task_handle.abort();
                return ComponentAction::Quit;
            }
            (A::ChangeFocus, _) => {
                self.switch_focus();
                self.ctx.send_action(A::Render);
            }
            (A::Confirm, CurrentFocus::CloseButton) => {
                self.torrent_info_task_handle.abort();
                return ComponentAction::Quit;
            }
            (A::Select | A::Confirm, CurrentFocus::Files) => {
                if self.torrent.is_some() {
                    let mut wanted_ids = self
                        .torrent
                        .as_ref()
                        .unwrap()
                        .wanted
                        .as_ref()
                        .unwrap()
                        .clone();

                    let selected_ids = self.selected_ids();

                    if selected_ids.is_empty() {
                        self.tree_state.toggle_selected();
                        self.ctx.send_action(A::Render);
                        return ComponentAction::Nothing;
                    }

                    let mut wanted_in_selection_no = 0;
                    for selected_id in &selected_ids {
                        if wanted_ids[*selected_id as usize] == 1 {
                            wanted_in_selection_no += 1;
                        } else {
                            wanted_in_selection_no -= 1;
                        }
                    }

                    if wanted_in_selection_no > 0 {
                        for selected_id in &selected_ids {
                            wanted_ids[*selected_id as usize] = 0;
                        }
                    } else {
                        for selected_id in &selected_ids {
                            wanted_ids[*selected_id as usize] = 1;
                        }
                    }

                    let args = {
                        if wanted_in_selection_no > 0 {
                            for transmission_file in self.tree.get_by_ids(&selected_ids) {
                                transmission_file.set_wanted(false);
                            }
                            TorrentSetArgs {
                                files_unwanted: Some(selected_ids),
                                ..Default::default()
                            }
                        } else {
                            for transmission_file in self.tree.get_by_ids(&selected_ids) {
                                transmission_file.set_wanted(true);
                            }
                            TorrentSetArgs {
                                files_wanted: Some(selected_ids),
                                ..Default::default()
                            }
                        }
                    };

                    self.ctx.send_torrent_action(TorrentAction::SetArgs(
                        Box::new(args),
                        Some(vec![self.torrent_id.clone()]),
                    ));

                    self.ctx.send_action(Action::Render);
                }
            }

            (A::Up, CurrentFocus::Files) => {
                self.tree_state.key_up();
                self.ctx.send_action(Action::Render);
            }
            (A::Down, CurrentFocus::Files) => {
                self.tree_state.key_down();
                self.ctx.send_action(Action::Render);
            }
            (A::XdgOpen, CurrentFocus::Files) => {
                if let Some(torrent) = &self.torrent {
                    let mut identifier = self.tree_state.selected().to_vec();

                    if identifier.is_empty() {
                        return ComponentAction::Nothing;
                    }

                    if let Ok(file_id) = identifier.last().unwrap().parse::<i32>() {
                        identifier.pop();
                        identifier
                            .push(self.tree.get_by_ids(&[file_id]).pop().unwrap().name.clone())
                    }

                    let sub_path = identifier.join("/");

                    let path = format!("{}/{}", torrent.download_dir.as_ref().unwrap(), sub_path,);

                    match open::that_detached(&path) {
                        Ok(()) => self.ctx.send_update_action(UpdateAction::TaskSetSuccess(
                            StatusTask::new_open(&path),
                        )),
                        Err(err) => {
                            let desc =
                                format!("An error occured while trying to open \"{}\"", path);
                            let err_msg =
                                ErrorMessage::new("Failed to open a file", desc, Box::new(err));
                            self.ctx
                                .send_update_action(UpdateAction::Error(Box::new(err_msg)));
                        }
                    };
                }
            }

            _ => (),
        }
        ComponentAction::Nothing
    }

    fn handle_update_action(&mut self, action: UpdateAction) {
        if let UpdateAction::UpdateCurrentTorrent(torrent) = action {
            let new_tree = Node::new_from_torrent(&torrent);
            self.torrent = Some(*torrent);
            self.tree = new_tree;
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let popup_rect = centered_rect(rect, 75, 75);
        let block_rect = popup_rect.inner(Margin::new(1, 1));

        let info_text_rect = block_rect.inner(Margin::new(3, 2));

        let highlight_style = Style::default().fg(self.ctx.config.general.accent_color);
        let bold_highlight_style = highlight_style.on_black().bold();

        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(Title::from(" Files ".set_style(highlight_style)).alignment(Alignment::Left));

        if self.tree_state.selected().is_empty() {
            self.tree_state.select_first();
        }

        if let Some(torrent) = &self.torrent {
            if !self.switched_after_fetched_data {
                self.current_focus = CurrentFocus::Files;
                self.switched_after_fetched_data = true;
            }

            let close_button_style = {
                match self.current_focus {
                    CurrentFocus::CloseButton => highlight_style.bold(),
                    CurrentFocus::Files => Style::default(),
                }
            };

            let tree_highlight_style = {
                if self.current_focus == CurrentFocus::Files {
                    bold_highlight_style
                } else {
                    Style::default()
                }
            };

            let download_dir = torrent.download_dir.as_ref().expect("Requested");

            let keybinding_tip = {
                if self.ctx.config.general.beginner_mode {
                    let mut keys = vec![];

                    if let Some(key) = self
                        .ctx
                        .config
                        .keybindings
                        .get_keys_for_action(Action::Select)
                    {
                        keys.push(Span::raw(" "));
                        keys.push(Span::styled(
                            key,
                            Style::new()
                                .fg(self.ctx.config.general.accent_color)
                                .underlined(),
                        ));
                        keys.push(Span::raw(" - toggle | "));
                    }

                    if let Some(key) = self
                        .ctx
                        .config
                        .keybindings
                        .get_keys_for_action(Action::XdgOpen)
                    {
                        keys.push(Span::styled(
                            key,
                            Style::new()
                                .fg(self.ctx.config.general.accent_color)
                                .underlined(),
                        ));
                        keys.push(Span::raw(" - xdg_open "));
                    }

                    Line::from(keys)
                } else {
                    Line::from("")
                }
            };

            let block = block
                .title(
                    Title::from(format!(" {} ", download_dir).set_style(highlight_style))
                        .alignment(Alignment::Right),
                )
                .title(
                    Title::from(" [ CLOSE ] ".set_style(close_button_style))
                        .alignment(Alignment::Right)
                        .position(Position::Bottom),
                )
                .title(
                    Title::from(keybinding_tip)
                        .alignment(Alignment::Left)
                        .position(Position::Bottom),
                );

            let tree_items = self.tree.make_tree();

            let tree_widget = Tree::new(&tree_items)
                .unwrap()
                .block(block)
                .highlight_style(tree_highlight_style);

            f.render_widget(Clear, popup_rect);
            f.render_stateful_widget(tree_widget, block_rect, &mut self.tree_state);
        } else {
            let paragraph = Paragraph::new("Loading...");
            let block = block.title(
                Title::from(" [ CLOSE ] ".set_style(highlight_style.bold()))
                    .alignment(Alignment::Right)
                    .position(Position::Bottom),
            );
            f.render_widget(Clear, popup_rect);
            f.render_widget(paragraph, info_text_rect);
            f.render_widget(block, block_rect);
        }
    }
}

struct TransmissionFile {
    name: String,
    id: usize,
    // TODO: Change to enum
    wanted: bool,
}

impl TransmissionFile {
    fn set_wanted(&mut self, new_wanted: bool) {
        self.wanted = new_wanted;
    }
}

struct Node {
    items: Vec<TransmissionFile>,
    directories: BTreeMap<String, Node>,
}

impl Node {
    fn new() -> Self {
        Self {
            items: vec![],
            directories: BTreeMap::new(),
        }
    }

    fn new_from_torrent(torrent: &Torrent) -> Self {
        let files = torrent.files.as_ref().unwrap();
        let mut root = Self::new();

        for (id, file) in files.iter().enumerate() {
            let path: Vec<String> = file.name.split('/').map(str::to_string).collect();

            let wanted = torrent.wanted.as_ref().unwrap()[id] != 0;

            let file = TransmissionFile {
                id,
                name: path[path.len() - 1].clone(),
                wanted,
            };

            root.add_transmission_file(file, &path);
        }

        root
    }

    fn add_transmission_file(&mut self, file: TransmissionFile, remaining_path: &[String]) {
        if let Some((first, rest)) = remaining_path.split_first() {
            if rest.is_empty() {
                // We've found home for our TransmissionFile! :D
                self.items.push(file);
            } else {
                let child = self
                    .directories
                    .entry(first.to_string())
                    .or_insert_with(Self::new);
                child.add_transmission_file(file, rest);
            }
        }
    }

    fn get_by_ids(&mut self, ids: &[i32]) -> Vec<&mut TransmissionFile> {
        let mut transmission_files = vec![];
        for file in &mut self.items {
            if ids.contains(&(file.id as i32)) {
                transmission_files.push(file);
            }
        }
        for node in self.directories.values_mut() {
            transmission_files.extend(node.get_by_ids(ids))
        }
        transmission_files
    }

    fn make_tree(&self) -> Vec<TreeItem<String>> {
        let mut tree_items = vec![];
        for transmission_file in &self.items {
            let name = {
                if transmission_file.wanted {
                    format!("󰄲 {}", transmission_file.name)
                } else {
                    format!(" {}", transmission_file.name)
                }
            };
            tree_items.push(TreeItem::new_leaf(transmission_file.id.to_string(), name));
        }

        for (key, value) in &self.directories {
            tree_items.push(TreeItem::new(key.clone(), key.clone(), value.make_tree()).unwrap());
        }
        tree_items
    }
}
