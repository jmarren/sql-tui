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
    EnterVisualMode,
    Move(Focus, MoveDirection),
    SetFocus(Focus),
    TODO,
}




impl From<(&Focus, &Mode, KeyEvent)> for Command {
    fn from(value: (&Focus, &Mode,  KeyEvent)) -> Self {
        match value {
            // Esc to exit
            (_, _,  KeyEvent{ code: KeyCode::Esc,  .. }) => {
                Self::Exit
            },
            // Editor
            (Focus::Editor, mode, key) => {
                match (mode, key)  {
                    // Editor Visual Mode
                    (Mode::Visual, key) => {
                        match key { 
                            // Ctrl-S to execute Query
                            KeyEvent{ code: KeyCode::Char('s'), modifiers: KeyModifiers::CONTROL, .. } => {
                                Self::ExecuteQuery
                            }
                            KeyEvent{ code: KeyCode::Char('j'), modifiers: KeyModifiers::CONTROL, .. } => {
                                Self::SetFocus(Focus::Results)
                            },
                            KeyEvent{ code: KeyCode::Char('h'), modifiers: KeyModifiers::CONTROL, .. } => {
                                Self::SetFocus(Focus::SideTab)
                            },
                            // i for insert mode
                            KeyEvent{ code: KeyCode::Char('i'), .. } => {
                                Self::EnterInsertMode
                            },
                            // j for down
                            KeyEvent{ code: KeyCode::Char('j'), .. } => {
                                Self::Move(Focus::Editor, MoveDirection::Down)
                            },
                            // h for left
                            KeyEvent{ code: KeyCode::Char('h'), .. } => {
                                Self::Move(Focus::Editor, MoveDirection::Left)
                            },
                            // k for Up
                            KeyEvent{ code: KeyCode::Char('k'), .. } => {
                                Self::Move(Focus::Editor, MoveDirection::Up)
                            },
                            // l for Right
                            KeyEvent{ code: KeyCode::Char('l'), .. } => {
                                Self::Move(Focus::Editor, MoveDirection::Right)
                            }, 
                            _ => Self::TODO
                        }
                    },
                    // Editor Insert Mode
                    (Mode::Insert, key) => {
                        match key {
                            // Ctrl-C to enter visual mode
                            KeyEvent{ code: KeyCode::Char('c'), modifiers: KeyModifiers::CONTROL, .. } => {
                                Self::EnterVisualMode
                            },
                            // otherwise insert key
                            _ => {
                                Self::InsertKey(key)
                            },
                        }
                    }
                }
            },
            // Side Tab
            (Focus::SideTab, _, key) =>  {
                match key {
                        KeyEvent{ code: KeyCode::Char('l'), modifiers: KeyModifiers::CONTROL, .. } => {
                            Self::SetFocus(Focus::Editor)
                        },
                        // j to scroll down
                        KeyEvent{ code: KeyCode::Char('j'), .. } => {
                            Self::Move(Focus::SideTab, MoveDirection::Down)
                        },
                        // k to scroll up
                        KeyEvent{ code: KeyCode::Char('k'), .. } => {
                            Self::Move(Focus::SideTab, MoveDirection::Up)
                        },
                        _ => Self::TODO
                }
            },
            // Results
            (Focus::Results, _, key) =>  {
                match key {
                        KeyEvent{ code: KeyCode::Char('k'), modifiers: KeyModifiers::CONTROL, .. } => {
                            Self::SetFocus(Focus::Editor)
                        },
                        KeyEvent{ code: KeyCode::Char('h'), modifiers: KeyModifiers::CONTROL, .. } => {
                            Self::SetFocus(Focus::SideTab)
                        },
                        // j to scroll down
                        KeyEvent{ code: KeyCode::Char('j'), .. } => {
                            Self::Move(Focus::Results, MoveDirection::Down)
                        },
                        // h to scroll left
                        KeyEvent{ code: KeyCode::Char('h'), .. } => {
                            Self::Move(Focus::Results, MoveDirection::Left)
                        },
                        // k to scroll up
                        KeyEvent{ code: KeyCode::Char('k'), .. } => {
                            Self::Move(Focus::Results, MoveDirection::Up)
                        },
                        // k to scroll right
                        KeyEvent{ code: KeyCode::Char('l'), .. } => {
                            Self::Move(Focus::Results, MoveDirection::Right)
                        }, 
                        _ => Self::TODO,
                }
            },

            (Focus::Tables, _, key) =>  {
                match key {
                        // j to scroll down
                        KeyEvent{ code: KeyCode::Char('j'), .. } => {
                            Self::Move(Focus::Tables, MoveDirection::Down)
                        },
                        // k to scroll up
                        KeyEvent{ code: KeyCode::Char('k'), .. } => {
                            Self::Move(Focus::Tables, MoveDirection::Up)
                        },
                        _ => Self::TODO
                }
            },
        }
    }
}

            //     KeyEvent{ code, modifiers, ..} => {
            //         match (code, modifiers) {
            //             (KeyCode::Char('k'), KeyModifiers::CONTROL) => {
            //                 // move focus up to editor
            //                 self.focus = Section::Upper;   
            //             },
            //             (KeyCode::Char('k'), _) => {
            //                 // scroll results up
            //                 let first = self.results.remove(0);
            //                 self.results.push(first);
            //             },
            //             (KeyCode::Char('j'), _) => {
            //                 // scroll results down
            //                 if let Some(last) = self.results.pop() {
            //                     let curr = self.results.clone();
            //                     self.results = Vec::new();
            //                     self.results.push(last);
            //                     self.results.extend(curr);
            //                 }
            //             },
            //             (KeyCode::Char('w'), _) => {
            //                 // scroll results forward (horizontally)
            //                 let mut new_rows = Vec::new();
            //                 for row in &mut self.results  {
            //                     let mut new_row = Vec::new();
            //                     if let Some(last) = row.pop() {
            //                         new_row.push(last);   
            //                     }
            //                     new_row.extend(row.clone());
            //                     new_rows.push(new_row);
            //                 }
            //                 self.results = new_rows;
            //             },
            //             (KeyCode::Char('b'), _) => {
            //                 // scroll results backward (horizontally)
            //                 for row in &mut self.results  {
            //                     let first = row.remove(0);
            //                     row.push(first);
            //                 }
            //             },
            //             _ => {}
            //         }
            //     }
            // }
            //
            //
            //
    // async fn editor_handle_key(&mut self,  key: KeyEvent) {
    //         match (&self.mode, key) {
    //             (Mode::Insert, KeyEvent{ code: KeyCode::Esc,  .. }) => {
    //                 // set mode to visual
    //                 self.mode = Mode::Visual;
    //                 // set cursor style to reversed
    //                 self.editor.textarea.set_cursor_style(Style::default().reversed());
    //             },
    //             (_, KeyEvent{ code: KeyCode::Char('s'), modifiers: KeyModifiers::CONTROL, .. }) => {
    //                 // get content and perform query
    //                  self.user_query(self.editor.content()).await;
    //             },
    //             (Mode::Insert, _) => {
    //                 // insert character
    //                 // input the key
    //                 self.editor.textarea.input(key);
    //                 self.editor.highlighter.highlight(self.editor.textarea.lines().join("\n"));
    //             },
    //             (Mode::Visual, KeyEvent{ code: KeyCode::Char('i'), ..}) => {
    //                 // set mode to insert
    //                 self.mode = Mode::Insert;  
    //                 self.editor.textarea.set_cursor_style(Style::new().not_reversed());
    //             },
    //             (Mode::Visual, KeyEvent{ code, modifiers,  .. }) => {
    //                 match (code, modifiers) {
    //                     (KeyCode::Char('k'), _) => {
    //                         // cursor up
    //                         self.editor.textarea.move_cursor(CursorMove::Up);
    //                     },
    //                     (KeyCode::Char('j'), KeyModifiers::CONTROL) => {
    //                         // move focus to results section
    //                         self.focus = Section::Lower;   
    //                     },
    //                     (KeyCode::Char('j'), _) => {
    //                         // cursor down
    //                         self.editor.textarea.move_cursor(CursorMove::Down);
    //                     },
    //                     (KeyCode::Char('l'), _) => {
    //                         // cursor forward
    //                         self.editor.textarea.move_cursor(CursorMove::Forward);
    //                     },
    //                     (KeyCode::Char('h'), _) => {
    //                         // cursor back
    //                         self.editor.textarea.move_cursor(CursorMove::Back);
    //                     },
    //                     _ => {}
    //                 }
    //             }
    //         }
    // }
    //
