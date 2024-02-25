use std::{
    cell::RefCell,
    sync::{Arc, Mutex},
};

use ratatui::widgets::TableState;

pub struct GenericTable<T: Clone> {
    pub state: RefCell<TableState>,
    pub items: Arc<Mutex<Vec<T>>>,
}

impl<T: Clone> GenericTable<T> {
    pub fn new(items: Vec<T>) -> Self {
        let items = Arc::new(Mutex::new(items));

        Self {
            state: RefCell::new(TableState::new().with_selected(Some(0))),
            items,
        }
    }

    pub fn set_items(&mut self, items: Vec<T>) {
        *self.items.lock().unwrap() = items;
    }

    pub fn current_item(&self) -> Option<T> {
        let items = self.items.lock().unwrap();
        let selected = self.state.borrow().selected()?;
        Some(items[selected].clone())
    }

    pub fn next(&mut self) {
        let mut state = self.state.borrow_mut();
        if let Some(curr) = state.selected() {
            if curr == self.items.lock().unwrap().len() {
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
                state.select(Some(self.items.lock().unwrap().len()));
            } else {
                state.select(Some(curr - 1));
            }
        }
    }
}
