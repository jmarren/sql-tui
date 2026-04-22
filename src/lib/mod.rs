mod styles;
use styles::{Styles};
mod pgtype;
mod highlight;

use ratatui_textarea::{CursorMove, TextArea};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{DefaultTerminal, Frame, layout::{Constraint, Direction, Layout}, style::Style, text::{Line, Span}, widgets::{Block, Borders, Paragraph, Row, Table} };
use sqlx::{Column, Row as SqlxRow};
use sqlx_postgres::{ PgPool, PgPoolOptions};
use tokio::io::AsyncWriteExt;
use std::{env};
// Import the colors for the global
// use colorize::{BrightRed, Blue};
enum Mode {
    Insert,
    Visual,
}

enum Section {
    Editor,
    Results,
}

enum SideTab {
    Editor,
    Tables,
}



struct App<'a> {
    _conn: PgPool,
    textarea: TextArea<'a>,
    term: &'a mut DefaultTerminal,
    result_columns: Vec<String>,
    results: Vec<Vec<String>>,
    mode: Mode,
    should_quit: bool,
    focus: Section,
    highlighter: highlight::HighlightParser<'a>,
    tables: Vec<String>,
    side_tab: SideTab,
    styles: Styles,
}

 


impl<'a> App<'a> {
    async fn new(terminal: &'a mut DefaultTerminal) -> App<'a> {
        let mut app = App{
            _conn: init_db(get_db_url()).await,
            textarea: TextArea::default(),
            term: terminal,
            results: Vec::new(),
            result_columns: Vec::new(),
            mode: Mode::Visual,
            should_quit: false,
            focus: Section::Editor,
            highlighter: highlight::HighlightParser::new(),
            tables: Vec::<String>::new(),
            side_tab: SideTab::Editor,
            styles: Styles::new(),
        };
    
        // get user defined table names and set them in app.tables
        let (_, table_names_row) = app.query(TABLES_QUERY).await;

        let mut table_names = Vec::<String>::new();
        table_names_row.iter().for_each(| item | {
            table_names.push(item[0].clone());
        });
        app.tables = table_names;
    
        // return the app
        app
    }

    // perform the provided query and set result_columns and results
    async fn user_query(&mut self, query: String) {
        (self.result_columns, self.results) = self.query(query.as_str()).await;
    }

    // perform query and return (result_columns, result_rows) (converted to strings)
    async fn query(&mut self, query: &str) -> (Vec<String>, Vec<Vec<String>>) {
        // perform the query
        let result = sqlx::query(query)
                .fetch_all(&self._conn)
                .await
                .expect("failed to execute query");
         
        
        let mut result_strs: Vec<Vec<String>> = Vec::new();
        let mut result_cols = Vec::<String>::new();

        // use the column names from the first row as result_columns
        if result.len() > 0 {
            result_cols = result[0].columns().iter().map(| col | {
                    col.name().to_string()
            }).collect();
        }
    
        // push stringified rows into result_strs
        for row in result {
            result_strs.push(pgtype::stringify(&row));
        }
        
        (result_cols, result_strs)

    }

    fn draw(&mut self) {

        let tab_names = ["editor", "tables"];
        let active_tab_idx = match self.side_tab {
            SideTab::Editor => 0,
            SideTab::Tables => 1,
        };
        let spans_vec: Vec<Line> = tab_names.iter().enumerate().map(|(i, name)| {
            let style = if i == active_tab_idx {
                self.styles.active_tab
            } else {
                self.styles.inactive_tab
            };
            Line::from(Span::styled(format!(" {} ", name), style))
        }).collect();

        let _ = self.term.draw(| frame: &mut Frame |  {

        // outer horizontal split: narrow tab bar on left, main content on right
        let h_layout = Layout::default()
            .direction(Direction::Horizontal)
            .margin(2)
            .constraints(vec![
                Constraint::Length(10),
                Constraint::Min(0),
            ])
            .split(frame.area());

        // render side tab bar
        let tab_block = Block::default().borders(Borders::ALL);
        let tab_inner = tab_block.inner(h_layout[0]);
        frame.render_widget(&tab_block, h_layout[0]);
        let tab_list = Paragraph::new(spans_vec).block(Block::default());
        frame.render_widget(tab_list, tab_inner);

        // create layout
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Percentage(50),
                Constraint::Percentage(50)
            ])
            .split(h_layout[1]);

        let mut rows = Vec::new();

        for result_row in self.results.clone() {
            rows.push(Row::new(result_row));
        }


        // create results table
        let results_table = Table::default()
            .header(Row::new(self.result_columns.clone()))
            .rows(rows);

        // create results block
        let b = Block::default()
            .title("results")
            .borders(Borders::ALL);

        // create editor block
        let editor = Block::default()
            .title("sql editor")
            .borders(Borders::ALL);

        frame.render_widget(&editor, layout[0]);
        frame.render_widget(&b, layout[1]);

        let line = Line::from(self.highlighter.spans.clone());
        frame.render_widget(line, editor.inner(layout[0]));
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
                     self.user_query(content).await;
                },
                (Mode::Insert, _) => {
                    // insert character
                    // input the key
                    self.textarea.input(key);
                    self.highlighter.highlight(self.textarea.lines().join("\n"));
                },
                (Mode::Visual, KeyEvent{ code: KeyCode::Char('i'), ..}) => {
                    // set mode to insert
                    self.mode = Mode::Insert;  
                    self.textarea.set_cursor_style(Style::new().not_reversed());
                },
                (Mode::Visual, KeyEvent{ code, modifiers,  .. }) => {
                    match (code, modifiers) {
                        (KeyCode::Char('k'), _) => {
                            // cursor up
                            self.textarea.move_cursor(CursorMove::Up);
                        },
                        (KeyCode::Char('j'), KeyModifiers::CONTROL) => {
                            // move focus to results section
                            self.focus = Section::Results;   
                        },
                        (KeyCode::Char('j'), _) => {
                            // cursor down
                            self.textarea.move_cursor(CursorMove::Down);
                        },
                        (KeyCode::Char('l'), _) => {
                            // cursor forward
                            self.textarea.move_cursor(CursorMove::Forward);
                        },
                        (KeyCode::Char('h'), _) => {
                            // cursor back
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
                            // move focus up to editor
                            self.focus = Section::Editor;   
                        },
                        (KeyCode::Char('k'), _) => {
                            // scroll results up
                            let first = self.results.remove(0);
                            self.results.push(first);
                        },
                        (KeyCode::Char('j'), _) => {
                            // scroll results down
                            if let Some(last) = self.results.pop() {
                                let curr = self.results.clone();
                                self.results = Vec::new();
                                self.results.push(last);
                                self.results.extend(curr);
                            }
                        },
                        (KeyCode::Char('w'), _) => {
                            // scroll results forward (horizontally)
                            let mut new_rows = Vec::new();
                            for row in &mut self.results  {
                                let mut new_row = Vec::new();
                                if let Some(last) = row.pop() {
                                    new_row.push(last);   
                                }
                                new_row.extend(row.clone());
                                new_rows.push(new_row);
                            }
                            self.results = new_rows;
                        },
                        (KeyCode::Char('b'), _) => {
                            // scroll results backward (horizontally)
                            for row in &mut self.results  {
                                let first = row.remove(0);
                                row.push(first);
                            }
                        },
                        _ => {}
                    }
                }
            }

    }

    async fn handle_key(&mut self, key: KeyEvent)  {

        match key {
            // always quit app on Ctrl-C
            KeyEvent{ code: KeyCode::Char('c'), modifiers: KeyModifiers::CONTROL, .. } => {
                    // quit app
                    self.should_quit = true;
                    return;
            },
            // Tab cycles side tabs
            KeyEvent{ code: KeyCode::Tab, .. } => {
                self.side_tab = match self.side_tab {
                    SideTab::Editor => SideTab::Tables,
                    SideTab::Tables => SideTab::Editor,
                };
                return;
            },
            _ => {},
        }
    
        // handle key depending on focused section
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
    
    app.run().await?;
    Ok(())
}


async fn logln(msg: &str) {
    let mut file = tokio::fs::OpenOptions::new()
        .append(true)
        .open("log.txt")
        .await
        .expect("failed to open log file");
        
    // push a newline
    let mut msg_string = msg.to_string();
    msg_string.push('\n');
    let _ = file.write_all(msg_string.as_bytes()).await;

}




fn get_db_url() -> String {
        env::var("DB_URL").expect("DB_URL must be set")
}


pub async fn init_db(db_url: String) -> PgPool {
    let logval = format!("db_url = {:?}", db_url);
    tokio::io::stdout().write_all(logval.as_bytes()).await.expect("failed to write to stdout");
    tokio::io::stdout().flush().await.expect("failed to flush stdout");
    // std::io::stdout().flush().unwrap();
    // std::io::Stdout::flush().await;
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(db_url.as_str())
        .await
        .expect("failed to connect to db");
    pool

}




static TABLES_QUERY: &str  = "SELECT table_name FROM information_schema.tables WHERE table_schema NOT IN ('pg_catalog', 'information_schema');";


