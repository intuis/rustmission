mod add_magnet;
mod change_category;
mod default;
mod delete_torrent;
mod filter;
mod move_torrent;
mod selection;
mod sort;
mod status;

pub use add_magnet::AddMagnet;
pub use change_category::ChangeCategory;
pub use default::Default;
pub use delete_torrent::Delete;
pub use filter::Filter;
pub use move_torrent::Move;
pub use selection::Selection;
pub use sort::Sort;
pub use status::{CurrentTaskState, Status};
