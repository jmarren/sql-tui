use ratatui_textarea::TextArea;
use crossterm::event::{Event, KeyCode, KeyModifiers, ModifierKeyCode};
use ratatui::{DefaultTerminal, Frame, layout::{Constraint, Direction, Layout, Rows}, style::Stylize, widgets::{Block, Borders, Row, Table} };
use sqlx::{Column, Row as SqlxRow, Value, any::AnyRow};
// use sqlx::Row;
use sqlx_postgres::{PgColumn, PgPool, PgPoolOptions, PgRow, PgTypeInfo, PgTypeKind, Postgres};
use tokio::io::AsyncWriteExt;
use std::{any::Any, env};

struct App<'a> {
    _conn: PgPool,
    textarea: TextArea<'a>,
    term: &'a mut DefaultTerminal,
    results: Vec<Row<'a>>,
}

// fn as_string(mut row: PgRow, index: usize) -> String {
//     if let value = row.get(index) {
//         match value {
//             Value::Null => String::from(""),
//             Value::Bytes(v) => String::from_utf8_lossy(v.as_slice()).into_owned(),
//             Value::Int(v) => format!("{v}"),
//             Value::UInt(v) => format!("{v}"),
//             Value::Float(v) => format!("{v}"),
//             Value::Double(v) => format!("{v}"),
//             Value::Date(year, month, day, hour, minutes, seconds, micro) => todo!(),
//             Value::Time(negative, days, hours, minutes, seconds, micro) => todo!(),
//         }
//     } else {
//         String::from("")
//     }
// }

// enum RowE {
//     Any(sqlx::any::AnyRow),
//     PgRow(sqlx_postgres::PgRow),
// }



fn stringify(row: &PgRow, col: &PgColumn) -> String {

        let str_type = PgTypeInfo::with_name("Text");
        let int4_type = PgTypeInfo::with_name("Int4");

        let type_info = col.type_info();

        let is_str = type_info.type_eq(&str_type);
        
        if is_str {
            row.get(col.name())
        }

        let is_int4 = type_info.type_eq(&int4_type);
        
        if is_int4 {
            let data: i32 = row.get(col.name());
            
            let data_str_res = data.to_string();
            data_str_res

        } else {
            panic!("type not implemented");
        }
}


impl<'a> App<'a> {
    async fn new(terminal: &'a mut DefaultTerminal) -> App<'a> {

        let textarea = TextArea::default();
        let db_url = get_db_url();
        println!("db_url = {:?}",db_url);
        let pool = init_db(db_url).await;
        App{
            _conn: pool,
            textarea: textarea,
            term: terminal,
            results: Vec::new(),
        }
    }

    async fn query(&mut self, query: String) {
        let result = sqlx::query(query.as_str())
                .fetch_all(&self._conn)
                .await
                .expect("failed to execute query");
            
    
        let mut result_strs: Vec<Row> = Vec::new();

        for row in result {
            let mut row_strs: Vec<String> = Vec::new();
            let cols = row.columns();
            for col in cols {
                let str_type = PgTypeInfo::with_name("Text");
                let int4_type = PgTypeInfo::with_name("Int4");

                let type_info = col.type_info();

                let is_str = type_info.type_eq(&str_type);
                
                if is_str {
                    row_strs.push(row.get(col.name()));
                }

                let is_int4 = type_info.type_eq(&int4_type);
                
                if is_int4 {
                    let data: i32 = row.get(col.name());
                    
                    let data_str_res = data.to_string();
                    row_strs.push(data_str_res);
                    // data_str_res

                } 
                // else {
                //     panic!("type not implemented");
                // }
            }
            result_strs.push(Row::new(row_strs));
        }
        

        logln(format!("result_strs = {:?}", result_strs).as_str()).await;
        self.results = result_strs;

        // self.results = result;

        // for ele in result {
        //     ele
        // }

        // logln(format!("{:?}", result).as_str()).await;
    }

    fn draw(&mut self) {
        let _ = self.term.draw(| frame: &mut Frame |  {
    
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(4)
            .constraints(vec![ 
                Constraint::Percentage(50),
                Constraint::Percentage(50)
            ])
            .split(frame.area());

           // self.results.into_iter()
    
        // let mut row_one: Vec<String> = Vec::new();

        
    
    

        // for res in &self.results {
        //     row_one.push(res.get("name"));
        //     let id = res.get("id");
        //     row_one.push(id);
        //     // logln(format!("res = {:?}", res).as_str()).await;
        // }
        // self.results

        let results_table = Table::default()
            .rows(self.results.clone());
            // .rows(Row::new(self.results));

        let b = Block::default()
            .title("results")
            .borders(Borders::ALL);
        
        let editor = Block::default()
            .title("sql editor")
            .borders(Borders::ALL);
        
        frame.render_widget(&editor, layout[0]);
        frame.render_widget(&self.textarea, editor.inner(layout[0]));
        frame.render_widget(&b, layout[1]);
        frame.render_widget(&results_table, b.inner(layout[1]));
        });
    }

    async fn handle_event(&mut self) -> bool {

        if let Ok(event) = crossterm::event::read() {
            let _ = match event {
                    Event::Key(key) => {
                        match key.code {
                            KeyCode::Esc => {
                                return true;
                            },
                            KeyCode::Char('s') => {
                                match key.modifiers {
                                    KeyModifiers::CONTROL => {
                                        println!("control S");
                                        let content = self.textarea.lines().join("\n");
                                        logln(content.as_str()).await;
                                        self.query(content).await;

                                    },
                                    _ => {
                                        self.textarea.input(key);
                                    }
                                }
                            },
                            _ => {
                                self.textarea.input(key);
                            }
                        }
                    }
                    _ => {}
                };
        }
        false
    }

    async fn run(&mut self) -> Result<(), anyhow::Error> {
        loop {
            self.draw();
            match self.handle_event().await {
                true => {
                    break Ok(());
                }
                false => {
                }
            }
            
        }
    }

}


fn main() -> anyhow::Result<()> {
    ratatui::run(app)
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

#[tokio::main]
async fn app(terminal: &mut DefaultTerminal) -> Result<(), anyhow::Error> {
    
    let mut app = App::new(terminal).await;
    
    
    logln("app start").await;
    
    app.run().await?;
    Ok(())
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



fn get_db_url() -> String {
        if let Ok(db_url) = env::var("DB_URL") {
            // logln(db_url.as_str()).await;
            db_url
        }else {
            panic!("DB_URL must be set");

        }
}


pub async fn init_db(db_url: String) -> PgPool {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(db_url.as_str())
        .await
        .expect("failed to connect to db");
    pool

}








