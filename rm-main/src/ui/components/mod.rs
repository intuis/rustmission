pub mod tabcomponent;
pub mod table;
pub mod torrent_tab;

use ratatui::{prelude::Rect, Frame};

use crate::action::Action;
pub use tabcomponent::TabComponent;
pub use torrent_tab::TorrentsTab;

pub trait Component {
    fn handle_events(&mut self, _action: Action) -> Option<Action> {
        None
    }
    fn render(&mut self, _f: &mut Frame, _rect: Rect) {}
}
