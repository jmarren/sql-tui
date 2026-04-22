mod log;
mod styles;
use styles::{Styles};
mod pgtype;
mod highlight;
mod db;

use ratatui_textarea::{CursorMove, TextArea};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{DefaultTerminal, Frame, layout::{Constraint, Direction, Layout}, style::Style, text::{Line, Span}, widgets::{Block, Borders, Paragraph, Row, Table} };
use sqlx::{Column, Row as SqlxRow};
use sqlx_postgres::{ PgPool};

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
    // _conn: PgPool,
    db: db::Db,
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
    
        let mut db = db::Db::new().await;
    
        let table_names = db.query_table_names().await;

        App{
            db: db,
            textarea: TextArea::default(),
            term: terminal,
            results: Vec::new(),
            result_columns: Vec::new(),
            mode: Mode::Visual,
            should_quit: false,
            focus: Section::Editor,
            highlighter: highlight::HighlightParser::new(),
            tables: table_names,
            side_tab: SideTab::Editor,
            styles: Styles::new(),
        }
    }

    // perform the provided query and set result_columns and results
    async fn user_query(&mut self, query: String) {
        (self.result_columns, self.results) = self.db.query(query.as_str()).await;
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




