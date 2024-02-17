use std::cell::RefCell;

use ratatui::widgets::TableState;

pub struct GenericTable<T> {
    pub state: RefCell<TableState>,
    pub items: Vec<T>,
}

impl<T> GenericTable<T> {
    pub fn new(items: Vec<T>) -> Self {
        Self {
            state: RefCell::new(TableState::new().with_selected(Some(0))),
            items,
        }
    }

    pub fn set_items(&mut self, items: Vec<T>) {
        self.items = items;
    }

    pub fn current_item(&self) -> Option<&T> {
        Some(&self.items[self.state.borrow().selected()?])
    }

    pub fn next(&mut self) {
        let mut state = self.state.borrow_mut();
        if let Some(curr) = state.selected() {
            if curr == self.items.len() {
                state.select(Some(0));
            } else {
                state.select(Some(curr + 1));
            }
        }
    }

    pub fn previous(&mut self) {
        let mut state = self.state.borrow_mut();

        if let Some(curr) = state.selected() {
            if curr == 0 {
                state.select(Some(self.items.len()));
            } else {
                state.select(Some(curr - 1));
            }
        }
    }
}
