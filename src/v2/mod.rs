use std::{ sync::Arc, time::Duration};

use crossterm::event::{self, Event, KeyCode, KeyEvent};
use ratatui::{DefaultTerminal, layout::{Position, Rect}, style::{Color, Style}, text::Span, widgets::{List, ListItem, ListState, StatefulWidget, Widget}};
use tokio::sync::{Mutex, mpsc::{Receiver}};

#[derive(Default)]
struct App {
    should_quit: bool,
    list_state: ListState,
    items: Vec<String>,
    ball_state: Position,
}

#[derive(Default)]
struct Ball {
    style: Style,
}

impl Ball {
    fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}





impl StatefulWidget for Ball {
    type State = Position;
    fn render(self, mut area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer, state: &mut Self::State) {
        area.x = state.x;
        area.y = state.y;
        Widget::render(Span::default().content("●").style(self.style), area, buf);
    }
}


const POLL_RATE: Duration = std::time::Duration::from_millis(50);

fn list_from_strs<'a>(value: &'a[String]) -> List<'a> {
        let items: Vec<ListItem> = value
                    .iter()
                    .enumerate()
                    .map(|(_i, item)| {
                        ListItem::from(format!("{item}"))
                    })
                    .collect();

        List::default().items(items)

}

// impl <'a>From<Vec<String>> for List<'a> {
//     fn from(value: Vec<String>) -> Self {
//
//     }
// }


impl App {
    fn select(&mut self, i: usize) {
        self.list_state.select(Some(i));
    }

    fn items(mut self, items: Vec<String>) -> Self {
        self.items = items;
        self
    }

    fn ball_state(mut self, ball_state: Position) -> Self {
        self.ball_state = ball_state;
        self
    }

    fn list_state(mut self, list_state: ListState) -> Self {
        self.list_state = list_state;  
        self
    }

    fn push_item(&mut self, item: String) { 
        self.items.push(item);
    }

    fn render_list(&mut self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let highlight_style = Style::default().bg(Color::DarkGray);
        let list = list_from_strs(&self.items).highlight_style(highlight_style);
        StatefulWidget::render(list, area, buf, &mut self.list_state);   
    }

    fn render_ball(&mut self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let ball = Ball::default().style(Style::default().fg(Color::Red));
        StatefulWidget::render(ball, area, buf, &mut self.ball_state);   
    }
}

impl Widget for &mut App {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
        where
            Self: Sized {
        self.render_list(area, buf);
        self.render_ball(area, buf);
    }
}




enum InputMessage { 
    KeyPress(KeyEvent)
}

struct InputHandler {
    rx: Receiver<InputMessage>,
    app: Arc<Mutex<App>>,
}

impl InputHandler { 

    async fn handle_key_press(&mut self, key: KeyEvent) {
        match key {
            KeyEvent { code: KeyCode::Esc,  .. } => self.app.lock().await.should_quit = true,
            KeyEvent { code: KeyCode::Char('j'),  .. } => self.app.lock().await.list_state.select_next(),
            KeyEvent { code: KeyCode::Char('k'),  .. } => self.app.lock().await.list_state.select_previous(),
            KeyEvent { code: KeyCode::Char('G'),  .. } => self.app.lock().await.list_state.select_last(),
            KeyEvent { code: KeyCode::Down,  .. } => self.app.lock().await.ball_state.y += 1,
            _=> (),
        }
    }
    
    // async fn handle_message(&mut self, message: InputMessage) {
    //     match message {
    //         InputMessage::KeyPress(key) => self.handle_key_press(key).await,
    //     }
    // }

    async fn start(rx: Receiver<InputMessage>, app_arc: Arc<Mutex<App>>) {
        let mut handler = Self  {
            rx: rx,
            app: app_arc,
        };

        loop {
            if crossterm::event::poll(POLL_RATE).unwrap_or(false)
                && let Ok(event) = event::read() {
                  match event {
                    Event::Key(key) if key.kind == event::KeyEventKind::Press => {
                        // if key.code == event::KeyCode::Esc
                        handler.handle_key_press(key).await;
                    },
                    _=>(),
                }
            };

            let app = handler.app.lock().await;

            if app.should_quit {
                break;
            }
            
        }
    }
}

#[tokio::main]
pub async fn run(terminal: &mut DefaultTerminal) -> Result<(), anyhow::Error> {
    
    let items = vec![
        String::from("1"),
        String::from("2"),
        String::from("3")
    ];
    let list_state = ListState::default().with_selected(Some(0));
    let ball_state = Position::from((10, 10));
    let app = App::default()
                     .items(items)
                     .list_state(list_state)
                     .ball_state(ball_state);
            
    let app_arc = Arc::new(Mutex::new(app));
    let (input_tx, input_rx) = tokio::sync::mpsc::channel(32);

    tokio::spawn(InputHandler::start(input_rx, Arc::clone(&app_arc)));

    
    loop {
        let mut app = app_arc.lock().await;

        if app.should_quit {
            break;
        }
        let _ = terminal.draw(| frame | {
            app.render(frame.area(), frame.buffer_mut());
        });
    }

    Ok(())
}


