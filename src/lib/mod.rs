mod log;
mod tables;
mod results;
mod command;
mod pgtype;
mod highlight;
mod db;
mod tabs;
mod styles;
mod editor;

use ratatui_textarea::{CursorMove};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{DefaultTerminal, Frame, layout::{Constraint, Direction, Layout}, style::Style, widgets::{Block, Borders, Row, Table}};

use crate::lib::{command::{Command, MoveDirection}, editor::Editor, results::Results, tabs::{TabKind, Tabs}};

pub enum Mode {
    Insert,
    Visual,
}

enum Section {
    Upper,
    Lower,
}

#[derive(Debug)]
pub enum Focus {
    SideTab,
    Editor,
    Results,
    Tables,
}

// impl Focus {
//     fn take_focus(self) {
//
//     }
// }


// struct SideTabs 
struct App<'a> {
    db: db::Db,
    term: &'a mut DefaultTerminal,
    results: Results<'a>,
    mode: Mode,
    should_quit: bool,
    focused: Focus,
    tabs: Tabs<'a>,
    tables: tables::Tables<'a>,
    outer_layout: Layout,
    inner_layout: Layout,
    results_block: Block<'a>,
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
            tables: tables::Tables::new(table_names),
            tabs: Tabs::new(),
            outer_layout: make_outer(),
            inner_layout: make_inner(),
            results_block: make_border_title_block("results"),
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

        match *self.tabs.active_tab()  {
            TabKind::Editor => {
                frame.render_widget(&self.editor.block, layout[0]);
                frame.render_widget(self.editor.line(), self.editor.block.inner(layout[0]));
            },
            TabKind::Tables => {
                frame.render_widget(&self.tables.block, layout[0]);
                frame.render_widget(&self.tables.paragraph, self.tables.block.inner(layout[0]));
            },
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
            Command::Move(focus,direction) => {
                match (focus,direction) {
                    (Focus::Editor, MoveDirection::Up) => {
                            self.editor.textarea.move_cursor(CursorMove::Up);
                    },
                    (Focus::Editor, MoveDirection::Down) => {
                            self.editor.textarea.move_cursor(CursorMove::Down);
                    },
                    (Focus::Editor, MoveDirection::Left) => {
                            self.editor.textarea.move_cursor(CursorMove::Back);
                    },
                    (Focus::Editor, MoveDirection::Right) => {
                            self.editor.textarea.move_cursor(CursorMove::Forward);
                    },
                    (Focus::SideTab, MoveDirection::Down) => {
                            self.tabs.scroll();
                    },
                    (Focus::Results, MoveDirection::Up) => {
                            self.results.scroll_up();
                    },
                    (Focus::Results, MoveDirection::Down) => {
                            self.results.scroll_down();
                    },
                    (Focus::Results, MoveDirection::Right) => {
                            self.results.scroll_right();
                    },
                    (Focus::Results, MoveDirection::Left) => {
                            self.results.scroll_left();
                    },
                    _ => {},

                }
            },
            Command::SetFocus(focus) => {
                // lose focus on current
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
                    _ => {}
                }
                
                self.focused = focus;

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
                    _ => {}
                }
            }

            _ => {}
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


