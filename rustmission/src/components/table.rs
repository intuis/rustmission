use ratatui::widgets::TableState;

pub struct GenericTable<T> {
    pub state: TableState,
    pub items: Vec<T>,
}

impl<T> GenericTable<T> {
    pub fn new(items: Vec<T>) -> Self {
        Self {
            state: TableState::new().with_selected(Some(0)),
            items,
        }
    }

    pub fn update_items(&mut self, items: Vec<T>) {
        self.items = items;
    }

    pub fn next(&mut self) {
        if let Some(curr) = self.state.selected() {
            if curr == self.items.len() {
                self.state.select(Some(0));
            } else {
                self.state.select(Some(curr + 1));
            }
        }
    }

    pub fn previous(&mut self) {
        if let Some(curr) = self.state.selected() {
            if curr == 0 {
                self.state.select(Some(self.items.len()));
            } else {
                self.state.select(Some(curr - 1));
            }
        }
    }
}
