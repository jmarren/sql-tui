
use ratatui_textarea::{CursorMove, TextArea};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{DefaultTerminal, Frame, layout::{Constraint, Direction, Layout}, style::{ Style}, widgets::{Block, Borders, Row, Table} };
use sqlx::{Column, Row as SqlxRow};
// use sqlx::Row;
use sqlx_postgres::{ PgPool, PgPoolOptions, PgRow, PgTypeInfo};
use tokio::io::AsyncWriteExt;
use std::{env};

enum Mode {
    Insert,
    Visual,
}

enum Section {
    Editor,
    Results,
}


struct App<'a> {
    _conn: PgPool,
    textarea: TextArea<'a>,
    term: &'a mut DefaultTerminal,
    results: Vec<Row<'a>>,
    mode: Mode,
    should_quit: bool,
    focus: Section,
}


fn is_text(type_info: &PgTypeInfo) -> bool {
    let text_type = PgTypeInfo::with_name("Text");
    type_info.type_eq(&text_type)
}

fn is_int4(type_info: &PgTypeInfo) -> bool {
    let int4_type = PgTypeInfo::with_name("Int4");
    type_info.type_eq(&int4_type)
}


fn stringify(row: &PgRow) -> Vec<String> {

        let mut row_strs: Vec<String> = Vec::new();
        let cols = row.columns();
    
        for col in cols {

            let type_info = col.type_info();
        
            // if text, just push
            if is_text(type_info) {
                row_strs.push(row.get(col.name()));
            }
            
            // if int4, convert to string, then push
            if is_int4(type_info) {
                let data: i32 = row.get(col.name());
                let data_str_res = data.to_string();
                row_strs.push(data_str_res);
            }  
        }
        row_strs
}


impl<'a> App<'a> {
    async fn new(terminal: &'a mut DefaultTerminal) -> App<'a> {

        let textarea = TextArea::default();
        let db_url = get_db_url();
        let pool = init_db(db_url).await;
        App{
            _conn: pool,
            textarea: textarea,
            term: terminal,
            results: Vec::new(),
            mode: Mode::Visual,
            should_quit: false,
            focus: Section::Editor,
        }
    }

    async fn query(&mut self, query: String) {
        let result = sqlx::query(query.as_str())
                .fetch_all(&self._conn)
                .await
                .expect("failed to execute query");
            
        
        let mut result_strs: Vec<Row> = Vec::new();

        for row in result {
            result_strs.push(Row::new(stringify(&row)));
        }
        self.results = result_strs;
    }

    fn draw(&mut self) {
        let _ = self.term.draw(| frame: &mut Frame |  {
    
        // create layout
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(4)
            .constraints(vec![ 
                Constraint::Percentage(50),
                Constraint::Percentage(50)
            ])
            .split(frame.area());

        // create results table
        let results_table = Table::default()
            .rows(self.results.clone())
            .style(Style::default().fg(ratatui::style::Color::Red));

        // create results block
        let b = Block::default()
            .title("results")
            .borders(Borders::ALL);
        
        // create editor block
        let editor = Block::default()
            .title("sql editor")
            .borders(Borders::ALL);
        
        frame.render_widget(&editor, layout[0]);
        frame.render_widget(&self.textarea, editor.inner(layout[0]));
        frame.render_widget(&b, layout[1]);
        frame.render_widget(&results_table, b.inner(layout[1]));
        });
    }

    async fn editor_handle_key(&mut self, key: KeyEvent) {

            match (&self.mode, key) {
                (Mode::Insert, KeyEvent{ code: KeyCode::Esc,  .. }) => {
                    // set mode to visual
                    self.mode = Mode::Visual;
                    // set cursor style to reversed
                    self.textarea.set_cursor_style(Style::default().reversed());
                },
                (_, KeyEvent{ code: KeyCode::Char('s'), modifiers: KeyModifiers::CONTROL, .. }) => {
                    // set content and perform query
                    let content = self.textarea.lines().join("\n");
                    self.query(content).await;
                },
                (Mode::Insert, _) => {
                    // insert character
                    self.textarea.input(key);
                },
                (Mode::Visual, KeyEvent{ code: KeyCode::Char('i'), ..}) => {
                    // set mode to insert
                    self.mode = Mode::Insert;  
                    self.textarea.set_cursor_style(Style::new().not_reversed());
                },
                (Mode::Visual, KeyEvent{ code, modifiers,  .. }) => {
                    match (code, modifiers) {
                        (KeyCode::Char('k'), _) => {
                            self.textarea.move_cursor(CursorMove::Up);
                        },
                        (KeyCode::Char('j'), KeyModifiers::CONTROL) => {
                            self.focus = Section::Results;   
                        },
                        (KeyCode::Char('j'), _) => {
                            self.textarea.move_cursor(CursorMove::Down);
                        },
                        (KeyCode::Char('l'), _) => {
                            self.textarea.move_cursor(CursorMove::Forward);
                        },
                        (KeyCode::Char('h'), _) => {
                            self.textarea.move_cursor(CursorMove::Back);
                        },
                        _ => {}
                    }
                }
            }
    }

    async fn results_handle_key(&mut self, key: KeyEvent) {
            match key {
                KeyEvent{ code, modifiers, ..} => {
                    match (code, modifiers) {
                        (KeyCode::Char('k'), KeyModifiers::CONTROL) => {
                            self.focus = Section::Editor;   
                        },
                        (KeyCode::Char('k'), _) => {
                            let mut curr = self.results.clone();
                            let first = curr.remove(0);
                            self.results = curr;
                            self.results.push(first);
                        },
                        (KeyCode::Char('j'), _) => {
                            if let Some(last) = self.results.pop() {
                                let curr = self.results.clone();
                                self.results = Vec::new();
                                self.results.push(last);
                                self.results.extend(curr);
                            }
                        },
                        (KeyCode::Char('l'), _) => {
                            // self.textarea.move_cursor(CursorMove::Forward);
                        },
                        (KeyCode::Char('h'), _) => {
                            // self.textarea.move_cursor(CursorMove::Back);
                        },
                        _ => {}
                    }
                }
            }

    }

    async fn handle_key(&mut self, key: KeyEvent)  {

        match key {
            KeyEvent{ code: KeyCode::Char('c'), modifiers: KeyModifiers::CONTROL, .. } => {
                    // quit app
                    self.should_quit = true;
                    return;
            },
            _ => {},
        }
        match self.focus {
            Section::Editor => {
                self.editor_handle_key(key).await;
            },
            Section::Results => {
                self.results_handle_key(key).await;
            },
        }
    }
    

    async fn handle_event(&mut self)  {

        if let Ok(event) = crossterm::event::read() {
            let _ = match event {
                    Event::Key(key) => {
                        self.handle_key(key).await ;
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

#[tokio::main]
pub async fn app(terminal: &mut DefaultTerminal) -> Result<(), anyhow::Error> {
    
    let mut app = App::new(terminal).await;
    
    
    logln("app start").await;
    
    app.run().await?;
    Ok(())
}


async fn logln(msg: &str) {
    let mut file = tokio::fs::OpenOptions::new()
        .append(true)
        .open("log.txt")
        .await
        .expect("failed to open log file");

        
    let _ = file.write_all(msg.as_bytes()).await;
    let _ = file.write_all("\n".as_bytes()).await;

}




fn get_db_url() -> String {
        env::var("DB_URL").expect("DB_URL must be set")
}


pub async fn init_db(db_url: String) -> PgPool {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(db_url.as_str())
        .await
        .expect("failed to connect to db");
    pool

}



// struct Grid {
//     cols: usize,
//     rows: usize,
// }
//
// impl Widget for Grid {
//     fn render(self, area: Rect, buf: &mut Buffer) {
//         let col_constraints = (0..self.cols).map(|_| Constraint::Length(9));
//         let row_constraints = (0..self.rows).map(|_| Constraint::Length(3));
//         let horizontal = Layout::horizontal(col_constraints).spacing(1);
//         let vertical = Layout::vertical(row_constraints).spacing(1);
//
//         let rows = vertical.split(area);
//         let cells = rows.iter().flat_map(|&row| horizontal.split(row).to_vec());
//
//         for (i, cell) in cells.enumerate() {
//             Paragraph::new(format!("Area {:02}", i + 1))
//                 .block(Block::bordered())
//                 .render(cell, buf);
//         }
//     }
// }
//
// fn render(frame: &mut Frame) {
//
//     // let inner = Block::default().title("hi").borders(Borders::ALL);
//     //
//     // let b = Block::default()
//     //     .title("Sqltui")
//     //     .borders(Borders::ALL)
//     //     .blue();
//     //
//     // let inner_area = b.inner(frame.area());
//     //
//     // frame.render_widget(b, frame.area());
//     // frame.render_widget(inner, inner_area);
//     frame.render_widget(Grid{ rows: 10, cols: 10}, frame.area());
// }





