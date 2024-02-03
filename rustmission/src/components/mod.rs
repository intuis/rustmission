pub mod tabcomponent;
mod table;
pub mod torrent_tab;

use ratatui::{prelude::Rect, Frame};

use crate::app::Action;

pub trait Component {
    fn handle_events(&mut self, _action: Action) -> Option<Action> {
        None
    }
    fn render(&mut self, _f: &mut Frame, _rect: Rect) {}
}
