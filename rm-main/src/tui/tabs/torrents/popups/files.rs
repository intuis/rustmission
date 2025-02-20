use std::{collections::BTreeMap, time::Duration};

use ratatui::{
    prelude::*,
    style::Styled,
    widgets::{
        block::{Position, Title},
        Clear, Paragraph,
    },
};
use rm_config::{keymap::GeneralAction, CONFIG};
use tokio::{sync::oneshot, task::JoinHandle};
use transmission_rpc::types::{Id, Torrent, TorrentSetArgs};
use tui_tree_widget::{Tree, TreeItem, TreeState};

use crate::{
    transmission::TorrentAction,
    tui::{
        app::CTX,
        components::{
            keybinding_style, popup_block, popup_close_button, popup_close_button_highlight,
            popup_rects, Component, ComponentAction,
        },
    },
};
use rm_shared::{
    action::{Action, ErrorMessage, UpdateAction},
    status_task::StatusTask,
    utils::{bytes_to_human_format, bytes_to_short_human_format},
};

pub struct FilesPopup {
    torrent: Option<Torrent>,
    torrent_id: Id,
    tree_state: TreeState<String>,
    tree: Node,
    current_focus: CurrentFocus,
    switched_after_fetched_data: bool,
    torrent_info_task_handle: JoinHandle<()>,
}

async fn fetch_new_files(torrent_id: Id) {
    loop {
        let (torrent_tx, torrent_rx) = oneshot::channel();
        CTX.send_torrent_action(TorrentAction::GetTorrentsById(
            vec![torrent_id.clone()],
            torrent_tx,
        ));

        match torrent_rx.await.unwrap() {
            Ok(mut torrents) => {
                CTX.send_update_action(UpdateAction::UpdateCurrentTorrent(Box::new(
                    torrents.pop().unwrap(),
                )));
            }
            Err(err_message) => {
                CTX.send_update_action(UpdateAction::Error(err_message));
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
    pub fn new(torrent_id: Id) -> Self {
        let torrent = None;
        let tree_state = TreeState::default();
        let tree = Node::new();

        let torrent_info_task_handle = tokio::task::spawn(fetch_new_files(torrent_id.clone()));

        Self {
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
                CTX.send_action(A::Render);
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
                        CTX.send_action(A::Render);
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

                    CTX.send_torrent_action(TorrentAction::SetArgs(
                        Box::new(args),
                        Some(vec![self.torrent_id.clone()]),
                    ));

                    CTX.send_action(Action::Render);
                }
            }

            (A::Up | A::ScrollUpBy(_), CurrentFocus::Files) => {
                self.tree_state.key_up();
                CTX.send_action(Action::Render);
            }
            (A::Down | A::ScrollDownBy(_), CurrentFocus::Files) => {
                self.tree_state.key_down();
                CTX.send_action(Action::Render);
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
                        Ok(()) => CTX.send_update_action(UpdateAction::StatusTaskSetSuccess(
                            StatusTask::new_open(&path),
                        )),
                        Err(err) => {
                            let desc =
                                format!("An error occured while trying to open \"{}\"", path);
                            let err_msg =
                                ErrorMessage::new("Failed to open a file", desc, Box::new(err));
                            CTX.send_update_action(UpdateAction::Error(Box::new(err_msg)));
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
        let (popup_rect, block_rect, text_rect) = popup_rects(rect, 75, 75);

        let highlight_style = Style::default().fg(CONFIG.general.accent_color);
        let bold_highlight_style = highlight_style.on_black().bold();

        let block = popup_block(" Files ");

        if self.tree_state.selected().is_empty() {
            self.tree_state.select_first();
        }

        if let Some(torrent) = &self.torrent {
            if !self.switched_after_fetched_data {
                self.current_focus = CurrentFocus::Files;
                self.switched_after_fetched_data = true;
            }

            let close_button = {
                match self.current_focus {
                    CurrentFocus::CloseButton => popup_close_button_highlight(),
                    CurrentFocus::Files => popup_close_button(),
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
                if CONFIG.general.beginner_mode {
                    let mut keys = vec![];

                    if let Some(key) = CONFIG
                        .keybindings
                        .general
                        .get_keys_for_action_joined(GeneralAction::Select)
                    {
                        keys.push(Span::raw(" "));
                        keys.push(Span::styled(key, keybinding_style()));
                        keys.push(Span::raw(" - toggle | "));
                    }

                    if let Some(key) = CONFIG
                        .keybindings
                        .general
                        .get_keys_for_action_joined(GeneralAction::XdgOpen)
                    {
                        keys.push(Span::styled(key, keybinding_style()));
                        keys.push(Span::raw(" - xdg_open "));
                    }

                    Line::from(keys)
                } else {
                    Line::from("")
                }
            };

            let block = block
                .title_top(
                    format!(" {} ", download_dir)
                        .set_style(highlight_style)
                        .into_right_aligned_line(),
                )
                .title(close_button)
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
            let block = block.title(popup_close_button_highlight());
            f.render_widget(Clear, popup_rect);
            f.render_widget(paragraph, text_rect);
            f.render_widget(block, block_rect);
        }
    }
}

struct TransmissionFile {
    name: String,
    id: usize,
    wanted: bool,
    length: i64,
    bytes_completed: i64,
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
                length: file.length,
                bytes_completed: file.bytes_completed,
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
            let mut name = Line::default();
            let progress: f64 = if transmission_file.length != 0 {
                transmission_file.bytes_completed as f64 / transmission_file.length as f64
            } else {
                0.0
            };
            let mut progress_percent = format!("{}% ", (progress * 100f64).ceil());

            if progress_percent.len() == 3 {
                progress_percent.push(' ');
            }

            if transmission_file.wanted {
                name.push_span(Span::raw("󰄲 "));
            } else {
                name.push_span(Span::raw(" "));
            }

            name.push_span(Span::raw("| "));

            if progress != 1.0 {
                name.push_span(Span::styled(
                    progress_percent,
                    Style::new().fg(CONFIG.general.accent_color),
                ));

                name.push_span(Span::raw("["));
                name.push_span(Span::styled(
                    bytes_to_short_human_format(transmission_file.bytes_completed),
                    Style::new().fg(CONFIG.general.accent_color),
                ));
                name.push_span(Span::raw("/"));
                name.push_span(Span::raw(bytes_to_short_human_format(
                    transmission_file.length,
                )));
                name.push_span(Span::raw("] "));
            } else {
                name.push_span(Span::raw("["));
                name.push_span(bytes_to_human_format(transmission_file.length));
                name.push_span(Span::raw("] "));
            }

            name.push_span(Span::raw(transmission_file.name.as_str()));

            tree_items.push(TreeItem::new_leaf(transmission_file.id.to_string(), name));
        }

        for (key, value) in &self.directories {
            tree_items.push(TreeItem::new(key.clone(), key.clone(), value.make_tree()).unwrap());
        }
        tree_items
    }
}
