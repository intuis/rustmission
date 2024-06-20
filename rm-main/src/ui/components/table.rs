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

    pub fn get_len(&self) -> usize {
        self.overwritten_len
            .borrow()
            .map_or(self.items.len(), |len| len)
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
            let last_idx = self.get_len() - 1;
            if curr == last_idx {
                state.select(Some(0));
            } else {
                state.select(Some(curr + 1));
            }
        }
    }

    pub fn previous(&mut self) {
        let mut state = self.state.borrow_mut();

        if let Some(curr) = state.selected() {
            let last_idx = self.get_len() - 1;
            if curr == 0 {
                state.select(Some(last_idx));
            } else {
                state.select(Some(curr - 1));
            }
        }
    }

    pub fn scroll_down_by(&mut self, amount: usize) {
        if self.items.is_empty() {
            return;
        }

        let mut state = self.state.borrow_mut();
        let new_selection = state.selected().unwrap_or_default() + amount;

        if new_selection > self.get_len() {
            state.select(Some(self.get_len() - 1));
        } else {
            state.select(Some(new_selection));
        };
    }

    pub fn scroll_up_by(&mut self, amount: usize) {
        let mut state = self.state.borrow_mut();
        let selected = state.selected().unwrap_or_default();

        if amount >= selected {
            state.select(Some(0));
        } else {
            state.select(Some(selected - amount));
        }
    }

    pub fn scroll_to_home(&mut self) {
        let mut state = self.state.borrow_mut();
        if !self.items.is_empty() {
            state.select(Some(0));
        }
    }

    pub fn scroll_to_end(&mut self) {
        if self.items.is_empty() {
            return;
        }

        let mut state = self.state.borrow_mut();
        state.select(Some(self.items.len() - 1));
    }
}
