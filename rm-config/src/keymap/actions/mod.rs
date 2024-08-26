use rm_shared::action::Action;

pub mod general;
pub mod search_tab;
pub mod torrents_tab;

pub trait UserAction: Into<Action> {
    fn desc(&self) -> &'static str;
    fn merged_desc(&self, other: &Self) -> Option<&'static str> {
        None
    }
    fn is_mergable_with(&self, other: &Self) -> bool {
        false
    }
}
