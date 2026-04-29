use ratatui::{Frame, layout::Rect, style::{Color, Modifier, Style}, text::Line, widgets::{Block, Borders, Paragraph}};

use crate::lib::{Focusable, command::MoveDirection};


pub struct ListItem<'a> {
    name: String,
    line: Line<'a>,
    active_style: Style,
    inactive_style: Style,
}

fn make_active_style() -> Style {
    Style::default().fg(Color::Black).bg(Color::White).add_modifier(Modifier::BOLD)
}

fn make_inactive_style() -> Style {
    Style::default().fg(Color::DarkGray)
}



impl <'a>ListItem<'a> {
    pub fn new(name: String) -> ListItem<'a> {
        ListItem {
            name: name.clone(),
            line: Line::from(name.clone()),
            active_style: make_active_style(),
            inactive_style: make_inactive_style(),
        }
    }

    fn set_active(&mut self) {
        self.line = self.line.clone().style(self.active_style);
    }

    fn set_inactive(&mut self)  {
        self.line = self.line.clone().style(self.inactive_style);
    }
}


pub struct List<'a>{
    pub block: Block<'a>,
    line_vec: Vec<ListItem<'a>>,
    focused_idx: i32, 
    pub paragraph: Paragraph<'a>,
    title: &'a str,
}

impl <'a>List<'a> {
    pub fn new(title: &'a str, items: Vec<String>) -> List<'a> {
        let mut items_vec = Vec::<ListItem>::new();
    
        // create a vec of ListItems from the provided items 
        // set first to active and the rest to inactive
        let mut first = true;
        for name in items {
            let mut item = ListItem::new(name);
            // make first item active and the rest inactive
            match first {
                true => {
                    item.set_active();
                    first = false;
                },
                false => item.set_inactive()
            }
            items_vec.push(item);
        }
        let mut list = List {
            block: Block::default().borders(Borders::ALL).title(title),
            title: title,
            paragraph: Paragraph::new(vec![]),
            line_vec: items_vec,
            focused_idx: 0,
        };

        list.update_paragraph();
        list
    }


    // collect lines into a vector
    fn lines(&self) -> Vec<Line<'a>> {
        self.line_vec.iter().map(| table | { table.line.clone() }).collect()
    }

    // set the paragraph field using the lines
    fn update_paragraph(&mut self) {
        self.paragraph = Paragraph::new(self.lines()).block(Block::default())
    }

    pub fn active_item(&self) -> String {
        self.line_vec[self.focused_idx as usize].name.clone()
    }


    // increment active tab and reset paragraph
    pub fn scroll(&mut self, direction: MoveDirection) {
        // set current active tab to inactive
        self.line_vec[self.focused_idx as usize].set_inactive();
        // increment or decrement focused index
        match direction {
            MoveDirection::Up => {
                self.focused_idx = self.focused_idx - 1;
            },
            MoveDirection::Down => {
                self.focused_idx = self.focused_idx + 1;
            },
            _ => {}
        };
        if self.focused_idx >= self.line_vec.len() as i32 {
            self.focused_idx = 0;
        }

        if self.focused_idx < 0 {
            self.focused_idx = self.line_vec.len() as i32 - 1 ;
        }
        // set new current active tab to active
        self.line_vec[self.focused_idx as usize].set_active();
        // get vec of lines 
        self.update_paragraph();
    }

    pub fn render(&mut self, frame: &mut Frame, rect: Rect) {
                frame.render_widget(&self.block, rect);
                frame.render_widget(&self.paragraph, self.block.inner(rect));
    }
    
}

impl <'a>Focusable for List<'a> {

    fn take_focus(&mut self) {
        self.block = Block::default()
                            .title(self.title)
                            .borders(Borders::ALL)
                            .border_style(Style::default().cyan())
    }

     fn lose_focus(&mut self) {
        self.block = Block::default()
                            .title(self.title)
                            .borders(Borders::ALL);
    }
    
}
