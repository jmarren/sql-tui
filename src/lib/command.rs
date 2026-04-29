use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::lib::{Focus, Mode};


#[derive(Debug)]
pub enum MoveDirection {
    Up, 
    Down,
    Left,
    Right,
}

#[derive(Debug)]
pub enum Command {
    // exit the app
    Exit,
    // insert a key
    InsertKey(KeyEvent),
    // execute a user query
    ExecuteQuery,
    // enter insert mode
    EnterInsertMode,
    ExpandTable,
    EnterVisualMode,
    MoveCursor(MoveDirection),
    MoveFocus(MoveDirection),
    TODO,
}




impl From<(&Focus, &Mode, KeyEvent)> for Command {
    fn from(value: (&Focus, &Mode,  KeyEvent)) -> Self {
        match value {
            (_, _,  KeyEvent{ code: KeyCode::Esc,  .. }) => Self::Exit,
            (_, _, KeyEvent{ code: KeyCode::Char('j'), modifiers: KeyModifiers::CONTROL, .. }) => Self::MoveFocus(MoveDirection::Down),
            (_, _, KeyEvent{ code: KeyCode::Char('h'), modifiers: KeyModifiers::CONTROL, .. }) => Self::MoveFocus(MoveDirection::Left),
            (_, _, KeyEvent{ code: KeyCode::Char('l'), modifiers: KeyModifiers::CONTROL, .. }) => Self::MoveFocus(MoveDirection::Right),
            (_, _, KeyEvent{ code: KeyCode::Char('k'), modifiers: KeyModifiers::CONTROL, .. }) => Self::MoveFocus(MoveDirection::Up),
            (Focus::Editor, Mode::Insert, KeyEvent{ code: KeyCode::Char('c'), modifiers: KeyModifiers::CONTROL, ..}) => Self::EnterVisualMode,
            (Focus::Editor, Mode::Insert, key) => Self::InsertKey(key),
            (Focus::Editor, Mode::Visual, KeyEvent{ code: KeyCode::Char('i'), .. }) => Self::EnterInsertMode,
            (Focus::Editor, Mode::Visual, KeyEvent{ code: KeyCode::Char('s'), .. }) => Self::ExecuteQuery,
            (Focus::Tables, _, KeyEvent{ code: KeyCode::Enter, .. }) => Self::ExpandTable,
            (_, _, KeyEvent{ code: KeyCode::Char('j'), .. }) => Self::MoveCursor(MoveDirection::Down),
            (_, _, KeyEvent{ code: KeyCode::Char('h'), .. }) => Self::MoveCursor(MoveDirection::Left),
            (_, _, KeyEvent{ code: KeyCode::Char('k'), .. }) => Self::MoveCursor(MoveDirection::Up),
            (_, _, KeyEvent{ code: KeyCode::Char('l'), .. }) => Self::MoveCursor(MoveDirection::Right),
            _ => Self::TODO,
        }
    }
}

