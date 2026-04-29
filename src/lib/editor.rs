use crate::lib::{Focusable, command::MoveDirection, highlight::{self, HighlightParser}};
use crossterm::event::KeyEvent;
use ratatui::{ Frame, layout::Rect, style::Style, text::Line, widgets::{Block, Borders}};
use ratatui_textarea::{CursorMove, TextArea};


pub struct Editor<'a> {
    pub highlighter: HighlightParser<'a>,   
    pub textarea: TextArea<'a>,
    pub block: Block<'a>,
}

impl <'a>Editor<'a> {
    pub fn new() -> Editor<'a> { 
        Editor {
            highlighter: highlight::HighlightParser::new(),
            textarea: TextArea::default(),
            block: Block::default()
            .title("editor")
            .borders(Borders::ALL)
,
        }
    }



    pub fn line(&self) -> Line<'a> { 
        let line = Line::from(self.highlighter.spans.clone());
        line
    }

    pub fn content(&self) -> String {
        self.textarea.lines().join("\n")
    }


    pub fn render(&mut self, frame: &mut Frame, rect: Rect) {
            frame.render_widget(&self.block, rect);
            frame.render_widget(self.line(), self.block.inner(rect));
    }

    pub fn input_key(&mut self, key: KeyEvent) {
            self.textarea.input(key);
            self.highlighter.highlight(self.textarea.lines().join("\n"));
    }

}

impl <'a>Focusable for Editor<'a> {

    fn take_focus(&mut self) {
        self.block = Block::default()
                            .title("editor")
                            .borders(Borders::ALL)
                            .border_style(Style::default().cyan())
    }

    fn lose_focus(&mut self) {
        self.block = Block::default()
                            .title("editor")
                            .borders(Borders::ALL);
    }

    fn move_cursor(&mut self, direction: MoveDirection) {
        match direction {
                    MoveDirection::Up => {
                            self.textarea.move_cursor(CursorMove::Up);
                    },
                    MoveDirection::Down => {
                            self.textarea.move_cursor(CursorMove::Down);
                    },
                    MoveDirection::Left => {
                            self.textarea.move_cursor(CursorMove::Back);
                    },
                    MoveDirection::Right => {
                            self.textarea.move_cursor(CursorMove::Forward);
                    }
        }
    }
}
