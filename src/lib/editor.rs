use crate::lib::highlight::{self, HighlightParser};
use ratatui::{ style::Style, text::Line, widgets::{Block, Borders}};
use ratatui_textarea::TextArea;


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

    pub fn take_focus(&mut self) {
        self.block = Block::default()
                            .title("editor")
                            .borders(Borders::ALL)
                            .border_style(Style::default().cyan())
    }

    pub fn lose_focus(&mut self) {
        self.block = Block::default()
                            .title("editor")
                            .borders(Borders::ALL);
    }


    pub fn line(&self) -> Line<'a> { 
        let line = Line::from(self.highlighter.spans.clone());
        line
    }

    pub fn content(&self) -> String {
        self.textarea.lines().join("\n")
    }
}
