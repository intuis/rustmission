use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use ratatui::{
    prelude::*,
    widgets::{
        block::{Position, Title},
        Block, BorderType, Clear, Paragraph,
    },
};
use transmission_rpc::types::{Id, Torrent, TorrentSetArgs};
use tui_tree_widget::{Tree, TreeItem, TreeState};

use crate::{
    action::{Action, TorrentAction},
    app,
    ui::{centered_rect, components::Component},
};

pub struct FilesPopup {
    ctx: app::Ctx,
    torrent: Arc<Mutex<Option<Torrent>>>,
    torrent_id: Id,
    tree_state: TreeState<String>,
    path_tree: Node,
    current_focus: CurrentFocus,
    switched_after_fetched_data: bool,
}

enum CurrentFocus {
    CloseButton,
    Files,
}

impl FilesPopup {
    pub fn new(ctx: app::Ctx, torrent_id: Id) -> Self {
        let torrent = Arc::new(Mutex::new(None));
        let tree_state = TreeState::default();

        ctx.send_torrent_action(TorrentAction::GetTorrentInfo(
            torrent_id.clone(),
            Arc::clone(&torrent),
        ));

        Self {
            ctx,
            torrent,
            tree_state,
            path_tree: Node::new(),
            current_focus: CurrentFocus::CloseButton,
            switched_after_fetched_data: false,
            torrent_id,
        }
    }

    fn switch_focus(&mut self) {
        match self.current_focus {
            CurrentFocus::CloseButton => self.current_focus = CurrentFocus::Files,
            CurrentFocus::Files => self.current_focus = CurrentFocus::CloseButton,
        }
    }
}

impl Component for FilesPopup {
    #[must_use]
    fn handle_actions(&mut self, action: Action) -> Option<Action> {
        match action {
            Action::Confirm => {
                if let CurrentFocus::Files = self.current_focus {
                    self.tree_state.toggle_selected();
                    Some(Action::Render)
                } else {
                    Some(Action::Quit)
                }
            }
            Action::Space => {
                if let Some(torrent) = &*self.torrent.lock().unwrap() {
                    let wanted_ids = torrent.wanted.as_ref().unwrap();

                    let selected_ids: Vec<_> = self
                        .tree_state
                        .selected()
                        .iter()
                        .filter_map(|str_id| str_id.parse::<i32>().ok())
                        .collect();
                    let mut wanted_in_selection_no = 0;
                    for selected_id in &selected_ids {
                        // If the index is out of bounds, that mean's we've got a branch.
                        if let Some(is_wanted) = wanted_ids.get(*selected_id as usize) {
                            if *is_wanted == 1 {
                                wanted_in_selection_no += 1;
                            } else {
                                wanted_in_selection_no -= 1;
                            }
                        }
                    }
                    let args = {
                        if wanted_in_selection_no > 0 {
                            TorrentSetArgs {
                                files_unwanted: Some(selected_ids),
                                ..Default::default()
                            }
                        } else {
                            TorrentSetArgs {
                                files_wanted: Some(selected_ids),
                                ..Default::default()
                            }
                        }
                    };

                    self.ctx.send_torrent_action(TorrentAction::SetArgs(
                        args,
                        Some(vec![self.torrent_id.clone()]),
                    ));

                    // TODO: Move it to a fetcher that you can send requests to, and will update tree by itself.
                    self.ctx.send_torrent_action(TorrentAction::GetTorrentInfo(
                        self.torrent_id.clone(),
                        Arc::clone(&self.torrent),
                    ));
                }

                None
            }

            Action::ChangeFocus => {
                self.switch_focus();
                Some(Action::Render)
            }

            Action::Up => {
                if let CurrentFocus::Files = self.current_focus {
                    self.tree_state.key_up();
                    Some(Action::Render)
                } else {
                    None
                }
            }
            Action::Down => {
                if let CurrentFocus::Files = self.current_focus {
                    self.tree_state.key_down();
                    Some(Action::Render)
                } else {
                    None
                }
            }

            Action::Quit => Some(Action::Quit),
            _ => None,
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let popup_rect = centered_rect(rect, 75, 75);
        let block_rect = popup_rect.inner(&Margin::new(1, 0));

        let info_text_rect = block_rect.inner(&Margin::new(3, 2));

        let highlight_style =
            Style::default().fg(self.ctx.config.general.accent_color.as_ratatui());
        let bold_highlight_style = highlight_style.on_black().bold();

        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(Title::from(" Files ".set_style(highlight_style)).alignment(Alignment::Left));

        if let Some(torrent) = &*self.torrent.lock().unwrap() {
            if !self.switched_after_fetched_data {
                self.current_focus = CurrentFocus::Files;
                self.switched_after_fetched_data = true;
            }

            if self.tree_state.selected().is_empty() {
                self.tree_state.select_first();
            }

            let close_button_style = {
                match self.current_focus {
                    CurrentFocus::CloseButton => highlight_style.bold(),
                    CurrentFocus::Files => Style::default(),
                }
            };

            let tree_highlight_style = {
                if let CurrentFocus::Files = self.current_focus {
                    bold_highlight_style
                } else {
                    Style::default()
                }
            };

            let download_dir = torrent.download_dir.as_ref().expect("Requested");
            let block = block
                .title(Title::from(format!(" {} ", download_dir)).alignment(Alignment::Right))
                .title(
                    Title::from(" [ CLOSE ] ".set_style(close_button_style))
                        .alignment(Alignment::Right)
                        .position(Position::Bottom),
                );

            let tree = Node::new_from_torrent(&torrent);
            let tree_items = tree.make_tree();

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
    full_path: String,
    name: String,
    id: usize,
    wanted: bool,
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
            let full_path = file.name.clone();
            let path: Vec<String> = file.name.split('/').map(str::to_string).collect();

            let wanted;
            if torrent.wanted.as_ref().unwrap()[id] == 0 {
                wanted = false;
            } else {
                wanted = true;
            }

            let file = TransmissionFile {
                full_path,
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
                    .or_insert_with(|| Node::new());
                child.add_transmission_file(file, rest);
            }
        }
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
