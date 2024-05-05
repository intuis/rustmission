use self::stats::StatisticsPopup;
use crate::{action::Action, ui::components::Component};

use ratatui::prelude::*;

pub mod info;
pub mod stats;

pub struct PopupManager {
    current_popup: Option<CurrentPopup>,
}

pub enum CurrentPopup {
    Stats(StatisticsPopup),
}

impl PopupManager {
    pub fn new() -> Self {
        Self {
            current_popup: None,
        }
    }

    pub fn is_showing_popup(&self) -> bool {
        self.current_popup.is_some()
    }

    pub fn show_popup(&mut self, popup: CurrentPopup) {
        self.current_popup = Some(popup);
    }

    pub fn close_popup(&mut self) {
        self.current_popup = None
    }
}

impl Component for PopupManager {
    #[must_use]
    fn handle_actions(&mut self, action: Action) -> Option<Action> {
        if let Some(current_popup) = &mut self.current_popup {
            match current_popup {
                CurrentPopup::Stats(popup) => {
                    if let Some(Action::Quit) = popup.handle_actions(action) {
                        self.close_popup();
                        return Some(Action::Render);
                    };
                }
            }

            return None;
        }
        None
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        if let Some(current_popup) = &mut self.current_popup {
            match current_popup {
                CurrentPopup::Stats(popup) => {
                    popup.render(f, rect);
                }
            }
        }
    }
}
