mod log;
mod list;
mod results;
mod command;
mod pgtype;
mod highlight;
mod db;
mod tabs;
mod styles;
mod editor;

use crossterm::event::{Event};
use ratatui::{DefaultTerminal, Frame, layout::{Constraint, Direction, Layout}, style::Style, widgets::{Block, Borders}};

use crate::lib::{command::{Command, MoveDirection}, editor::Editor, results::Results };

pub enum Mode {
    Insert,
    Visual,
}


#[derive(Debug, Clone, Copy)]
pub enum Focus {
    SideTab,
    Editor,
    Results,
    Tables,
}



// struct SideTabs 
struct App<'a> {
    db: db::Db,
    term: &'a mut DefaultTerminal,
    results: Results<'a>,
    mode: Mode,
    should_quit: bool,
    focused: Focus,
    tabs: list::List<'a>,
    tables: list::List<'a>,
    outer_layout: Layout,
    inner_layout: Layout,
    editor: Editor<'a>,
}




impl<'a> App<'a> {
    async fn new(terminal: &'a mut DefaultTerminal) -> App<'a> {
    
        let mut db = db::Db::new().await;
    
        let table_names = db.query_table_names().await;
            
        let mut editor = Editor::new();
        editor.take_focus();

        App{
            db: db,
            term: terminal,
            results: Results::new(),
            mode: Mode::Visual,
            should_quit: false,
            focused: Focus::Editor,
            tables: list::List::new("tables", table_names),
            tabs: list::List::new("", vec![" editor ".to_string(), " tables ".to_string()]),
            outer_layout: make_outer(),
            inner_layout: make_inner(),
            editor: editor,
        }
    }

    // perform the provided query and set result_columns and results
    async fn user_query(&mut self, query: String) {
        let (cols, vals) = self.db.query(query.as_str()).await;
        self.results.set_results(cols, vals);
    }
    

    fn draw(&mut self) {
        let _ = self.term.draw(| frame: &mut Frame |  {

        // outer horizontal split: narrow tab bar on left, main content on right
        let h_layout = self.outer_layout.split(frame.area());

        // render side tab bar
        let tab_inner = self.tabs.block.inner(h_layout[0]);
        frame.render_widget(&self.tabs.block, h_layout[0]);
        frame.render_widget(&self.tabs.paragraph, tab_inner);

        // create layout
        let layout = self.inner_layout.split(h_layout[1]);

        if self.tabs.active_item() == " editor ".to_string()  {
                frame.render_widget(&self.editor.block, layout[0]);
                frame.render_widget(self.editor.line(), self.editor.block.inner(layout[0]));
        } else {
                frame.render_widget(&self.tables.block, layout[0]);
                frame.render_widget(&self.tables.paragraph, self.tables.block.inner(layout[0]));
        }
        frame.render_widget(&self.results.block, layout[1]);
        frame.render_widget(&self.results.table, self.results.block.inner(layout[1]));

        });
    }

    async fn run_command(&mut self, cmd: command::Command) {
        match cmd {
            Command::Exit => {
                    self.should_quit = true;
            },
            Command::EnterInsertMode => {
                    self.mode = Mode::Insert;  
                    self.editor.textarea.set_cursor_style(Style::new().not_reversed());
            }, 
            Command::EnterVisualMode => {
                    self.mode = Mode::Visual;
                    self.editor.textarea.set_cursor_style(Style::default().reversed());
            },
            Command::InsertKey(key) => {
                    self.editor.textarea.input(key);
                    self.editor.highlighter.highlight(self.editor.textarea.lines().join("\n"));
            },
            Command::ExecuteQuery => {
                    // get content and perform query
                     self.user_query(self.editor.content()).await;
            },
            Command::MoveCursor(direction) => {
                self.move_cursor(direction);
            },
            Command::MoveFocus(direction) => {
                self.move_focus(direction);
            }

            _ => {}
        }
    }

    fn move_cursor(&mut self, direction: MoveDirection) {
                match (self.focused,direction) {
                    (Focus::Editor, dir) => {
                            self.editor.move_cursor(dir);
                    },
                    (Focus::SideTab, dir) => {
                            self.tabs.scroll(dir);
                    },
                    (Focus::Results, dir) => {
                            self.results.scroll(dir);
                    },
                    (Focus::Tables, dir) => {
                            self.tables.scroll(dir);
                    },
                }
    }

    fn move_focus(&mut self, direction: MoveDirection) {
                match self.focused {
                    Focus::Results => {
                        self.results.lose_focus();
                    },
                    Focus::Editor => {
                        self.editor.lose_focus();
                    },
                    Focus::SideTab => {
                        self.tabs.lose_focus();
                    },
                    Focus::Tables => {
                        self.tables.lose_focus();
                    }
                }

                // set new focus
                match (&self.focused, direction) {
                    (Focus::Results, MoveDirection::Up) => {
                        if self.tabs.active_item() == " editor " {
                            self.focused = Focus::Editor;
                        } else {
                            self.focused = Focus::Tables;
                        }
                    },
                    (Focus::Results, MoveDirection::Left) => {
                        self.focused = Focus::SideTab;
                    }
                    (Focus::SideTab, MoveDirection::Right) => {
                        if self.tabs.active_item() == " editor " {
                            self.focused = Focus::Editor;
                        } else {
                            self.focused = Focus::Tables;
                        }
                    },
                    (Focus::Editor, MoveDirection::Left) => {
                        self.focused = Focus::SideTab;
                    }
                    (Focus::Editor, MoveDirection::Down) => {
                        self.focused = Focus::Results;
                    },
                    (Focus::Tables, MoveDirection::Down) => {
                        self.focused = Focus::Results;
                    },
                    (Focus::Tables, MoveDirection::Left) => {
                        self.focused = Focus::SideTab;
                    }
                    _ => {}
                }
                
                // take focus on current
                match self.focused {
                    Focus::Results => {
                        self.results.take_focus();
                    },
                    Focus::Editor => {
                        self.editor.take_focus();
                    },
                    Focus::SideTab => {
                        self.tabs.take_focus();
                    },
                    Focus::Tables => {
                        self.tables.take_focus();
                    }
                }
    }




    async fn handle_event(&mut self)  {
        if let Ok(event) = crossterm::event::read() {
            let _ = match event {
                    Event::Key(key) => {
                        let input = (&self.focused, &self.mode, key);
                        let cmd = Command::from(input);
                        self.run_command(cmd).await;
                    },
                    _ => {}
                };
        }
    }

    async fn run(&mut self) -> Result<(), anyhow::Error> {
        loop {
            self.draw();
            self.handle_event().await;
            if self.should_quit {
                break Ok(());
            }
        }
    }

}


fn make_outer() -> Layout {
    Layout::default()
            .direction(Direction::Horizontal)
            .margin(2)
            .constraints(vec![
                Constraint::Length(10),
                Constraint::Min(0),
            ])
}

fn make_inner() -> Layout {
    Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Percentage(50),
                Constraint::Percentage(50)
            ])
}

fn make_border_title_block<'a>(title: &'a str) -> Block<'a> {
    Block::default()
            .title(title)
            .borders(Borders::ALL)
}

#[tokio::main]
pub async fn app(terminal: &mut DefaultTerminal) -> Result<(), anyhow::Error> {
    
    let mut app = App::new(terminal).await;
    
    app.run().await?;
    Ok(())
}


//
// enum SideTab<'a> {
//     Active(Line<'a>),
//     Inactive(Line<'a>),
// }
//
// enum SideTabs<'a> {
//     Editor(SideTab<'a>),
//     Tables(SideTab<'a>),
// }
//
// impl <'a>SideTab<'a> {
//     fn style(self) -> Style {
//         match self {
//             SideTab::Active(_) =>  Style::default().fg(Color::Black).bg(Color::White).add_modifier(Modifier::BOLD),
//             SideTab::Inactive(_) =>  Style::default().fg(Color::DarkGray),
//         }
//     }
//
//     fn line(self, text: String) -> Line {
//         match self {
//             SideTab::Active(l) => {
//                 l.style(self.style());
//             }
//         }
//     }
//
// }
//

// static SIDE_TABS: [SideTab; 2] = [SideTab::Editor, SideTab::Tables];
//
// impl <'a>SideTabs<'a> {
//
//     fn to_string(self) -> String {
//         match self {
//             SideTabs::Editor(_)  => " editor ".to_string(),
//             SideTabs::Tables(_) => " tables ".to_string(), 
//         }
//     }
//
//     fn set_active(&mut self) {
//
//      // Use 'if let' to gain a mutable reference to the internal fields
//         // if let self::SideTab(ref mut x, ref mut y) = msg {
//         //     *x += 5; // Changes 10 to 15
//         //     *y = 50; // Changes 20 to 50
//         // }
//
//     }
//
//     // fn to_line(self) -> Line<'a> {
//     //
//     //     // match self {
//     //     //     // SideTab::E
//     //     // }
//     //
//     // }
//
//
// }


