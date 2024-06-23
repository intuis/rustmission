use crossterm::event::KeyEvent;

#[derive(Clone, Debug)]
pub enum Event {
    Quit,
    Error,
    Render,
    Key(KeyEvent),
}
