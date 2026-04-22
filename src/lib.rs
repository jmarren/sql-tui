
use ratatui_textarea::{CursorMove, TextArea};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{DefaultTerminal, Frame, layout::{Constraint, Direction, Layout}, macros::ratatui_core, style::Style, text::{ Span}, widgets::{Block, Borders, Row, Table} };
use sqlx::{Column, Row as SqlxRow};
use sqlx_postgres::{ PgColumn, PgPool, PgPoolOptions, PgRow, PgTypeInfo};
use tokio::io::AsyncWriteExt;
use tree_sitter_highlight::{Highlight, HighlightConfiguration, HighlightEvent, Highlighter};
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


enum TextColor {
    BurntOrange,
    Cyan,
    Magenta,
    Gray,
    Blue1,
    Todo,
    Todo2,
}

impl TextColor {
    fn highlight<'a>(&self, text: String) -> Span<'a> {
        match &self {
            TextColor::BurntOrange => {
                let style =  Style::default().fg(ratatui_core::style::Color::Rgb(240, 120, 100));
                Span::raw(text).style(style).clone()
            },
            TextColor::Cyan => {
                let style =  Style::default().fg(ratatui_core::style::Color::Cyan);
                Span::raw(text).style(style).clone()
            }
            TextColor::Magenta => {
                let style =  Style::default().fg(ratatui_core::style::Color::Magenta);
                Span::raw(text).style(style).clone()
            },
            TextColor::Gray => {
                let style =  Style::default().fg(ratatui_core::style::Color::Gray);
                Span::raw(text).style(style).clone()
            },
            TextColor::Blue1 => {
                let style =  Style::default().fg(ratatui_core::style::Color::Rgb(103, 85, 230));
                Span::raw(text).style(style).clone()
            },
            TextColor::Todo => {
                let style =  Style::default().fg(ratatui_core::style::Color::Rgb(90, 25, 210));
                Span::raw(text).style(style).clone()
            },
            TextColor::Todo2 => {
                let style =  Style::default().fg(ratatui_core::style::Color::Rgb(20, 25, 180));
                Span::raw(text).style(style).clone()
            },

        }
    }
}

struct HighlightParser<'a>{
    pub highlighter: tree_sitter_highlight::Highlighter,
    pub sql_config: HighlightConfiguration,
    pub spans: Vec<Span<'a>>,
}

impl <'a> HighlightParser<'a> {
    fn new() -> HighlightParser<'a>  {
      let mut parser = tree_sitter::Parser::new();
      let language = tree_sitter_sequel::LANGUAGE;
      parser
          .set_language(&language.into())
          .expect("Error loading Sql parser");
     
     let hightlighter = Highlighter::new();
     let sql_language = tree_sitter_sequel::LANGUAGE.into();

     let mut sql_config = HighlightConfiguration::new(
                sql_language,
                "sql",
                tree_sitter_sequel::HIGHLIGHTS_QUERY,
                "",
                "",
         ).unwrap();


      sql_config.configure(&HIGHLIGHT_NAMES);
        
        HighlightParser { 
            highlighter: hightlighter,
            sql_config: sql_config,
            spans: Vec::<Span>::new(),
        }
    }

    pub fn highlight(&mut self, text: String) {
            // clear current spans
            self.spans.clear();

            // get highlights
            let highlights = self.highlighter.highlight(
                &self.sql_config,
                text.as_bytes(),
                None,
                |_| None
            ).unwrap();
    
           // declar spans and curr_color
           let mut spans = Vec::<Span>::new();
           let mut curr_color = TextColor::BurntOrange;

           // iterate through hightlights
           // on highlight start => set the curr_color
           // on source => apply the curr_color to get the styled span 
           //              and push it into spans
           for event in highlights {
               match event.unwrap() {
                   HighlightEvent::Source {start, end} => {
                       let target_str = text[start .. end].to_string().clone();
                       let span = curr_color.highlight(target_str.clone()).clone();
                       spans.push(span);
                   },
                   HighlightEvent::HighlightStart(s) => {
                       match s {
                           Highlight(33) => {
                               curr_color = TextColor::BurntOrange;
                           },
                           Highlight(14) => {
                               curr_color = TextColor::Cyan;
                           },
                           Highlight(38) => {
                               curr_color = TextColor::Magenta;
                           },
                           Highlight(46) => {
                               curr_color = TextColor::Blue1;
                           },
                           Highlight(48) => {
                               curr_color = TextColor::Todo;
                           },
                           Highlight(40) => {
                               curr_color = TextColor::Todo2;
                           },
                           _ => {
                                println!("missing color for s = {:?}", s);
                                curr_color = TextColor::Gray;
                           }
                       }
                   },
                   HighlightEvent::HighlightEnd => {},
               }
            }
           // set spans to the new styled spans
           self.spans = spans;
    }
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
    highlighter: HighlightParser<'a>,
    tables: Vec<String>,
}

 
static TEXT_TYPE: &PgTypeInfo = &PgTypeInfo::with_name("Text");
static INT4_TYPE: &PgTypeInfo = &PgTypeInfo::with_name("Int4");
static NAME_TYPE: &PgTypeInfo = &PgTypeInfo::with_name("Name");

fn is_text(type_info: &PgTypeInfo) -> bool {
    type_info.type_eq(TEXT_TYPE)
}

fn is_int4(type_info: &PgTypeInfo) -> bool {
    type_info.type_eq(INT4_TYPE)
}

fn is_name(type_info: &PgTypeInfo) -> bool {
    type_info.type_eq(NAME_TYPE)
}

// converts a cell to a String depending on its type_info
fn stringify_type_info(type_info: &PgTypeInfo, col: &PgColumn, row: &PgRow) -> String {
            let mut out = String::new();

            // simply return if text or name
            if is_text(type_info) || is_name(type_info) {
                out = row.get(col.name());
                return out
            }
            
            // if int4, convert to string first
            if is_int4(type_info) {
                let data: i32 = row.get(col.name());
                let data_str_res = data.to_string();
                return data_str_res;
            } 

            out
}   


// converts a row to a vec of Strings depending on the type of corresponding columns
fn stringify(row: &PgRow) -> Vec<String> {

        let mut row_strs: Vec<String> = Vec::new();
        let cols = row.columns();
    
        for col in cols {

            let type_info = col.type_info();

            row_strs.push(stringify_type_info(type_info, col, row));
        }
        row_strs
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
            highlighter: HighlightParser::new(),
            tables: Vec::<String>::new(),
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
            result_strs.push(stringify(&row));
        }
        
        (result_cols, result_strs)

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

        let line = ratatui::text::Line::from(self.highlighter.spans.clone());
        frame.render_widget(line, b.inner(layout[0]));
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



static HIGHLIGHT_NAMES: [&str; 52] = [
                        "attribute",
                        "boolean",
                        "carriage-return",
                        "comment",
                        "comment.documentation",
                        "constant",
                        "constant.builtin",
                        "constructor",
                        "constructor.builtin",
                        "embedded",
                        "error",
                        "escape",
                        "function",
                        "function.builtin",
                        "keyword",
                        "markup",
                        "markup.bold",
                        "markup.heading",
                        "markup.italic",
                        "markup.link",
                        "markup.link.url",
                        "markup.list",
                        "markup.list.checked",
                        "markup.list.numbered",
                        "markup.list.unchecked",
                        "markup.list.unnumbered",
                        "markup.quote",
                        "markup.raw",
                        "markup.raw.block",
                        "markup.raw.inline",
                        "markup.strikethrough",
                        "module",
                        "number",
                        "operator",
                        "property",
                        "property.builtin",
                        "punctuation",
                        "punctuation.bracket",
                        "punctuation.delimiter",
                        "punctuation.special",
                        "string",
                        "string.escape",
                        "string.regexp",
                        "string.special",
                        "string.special.symbol",
                        "tag",
                        "type",
                        "type.builtin",
                        "variable",
                        "variable.builtin",
                        "variable.member",
                        "variable.parameter",
    ];

