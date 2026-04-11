use ratatui_textarea::TextArea;
use crossterm::event::{Event, KeyCode};
use ratatui::{DefaultTerminal, Frame };
use sqlx_postgres::{PgPool, PgPoolOptions};
use tokio::io::AsyncWriteExt;
use std::{env};

struct App<'a> {
    _conn: PgPool,
    textarea: TextArea<'a>,
    term: &'a mut DefaultTerminal,
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
        }
    }

    async fn _query(self, query: String) {
        let result = sqlx::query(query.as_str())
                .fetch_all(&self._conn)
                .await
                .expect("failed to execute query");
        logln(format!("{:?}", result).as_str()).await;
    }

    fn draw(&mut self) {
        let _ = self.term.draw(| frame: &mut Frame |  {
            frame.render_widget(&self.textarea, frame.area());
        });
    }

    async fn run(&mut self) -> Result<(), anyhow::Error> {
        loop {
            self.draw();
            
            if let Ok(event) = crossterm::event::read() {
            match event {
                Event::Key(key) => {
                    match key.code {
                        KeyCode::Esc => {
                            break Ok(());
                        }
                        _ => {
                        self.textarea.input(key);
                        }
                    }
                }
                _ => {}
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








