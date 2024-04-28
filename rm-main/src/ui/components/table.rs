use std::cell::RefCell;

use ratatui::widgets::TableState;

pub struct GenericTable<T: Clone> {
    pub state: RefCell<TableState>,
    pub items: Vec<T>,
    pub overwritten_len: RefCell<Option<usize>>,
}

impl<T: Clone> GenericTable<T> {
    pub fn new(items: Vec<T>) -> Self {
        Self {
            state: RefCell::new(TableState::new().with_selected(Some(0))),
            items,
            overwritten_len: RefCell::new(None),
        }
    }

    fn get_len(&self) -> usize {
        if let Some(len) = *self.overwritten_len.borrow() {
            len
        } else {
            self.items.len()
        }
    }

    pub fn overwrite_len(&self, len: usize) {
        *self.overwritten_len.borrow_mut() = Some(len);
    }

    pub fn set_items(&mut self, items: Vec<T>) {
        self.items = items;
    }

    pub fn current_item(&self) -> Option<T> {
        let items = &self.items;
        let selected = self.state.borrow().selected()?;
        items.get(selected).cloned()
    }

    pub fn next(&mut self) {
        let mut state = self.state.borrow_mut();
        if let Some(curr) = state.selected() {
            if curr == self.get_len() {
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
                state.select(Some(self.get_len()));
            } else {
                state.select(Some(curr - 1));
            }
        }
    }
}
