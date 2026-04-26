use ratatui::{style::{Color, Modifier, Style}, text::Line, widgets::{Block, Borders, Paragraph}};

use crate::lib::command::MoveDirection;


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
    pub fn new(title: &'a str, table_names: Vec<String>) -> List<'a> {
        let mut tables_vec = Vec::<ListItem>::new();

        for name in table_names {
            tables_vec.push(ListItem::new(name));
        }
        let mut list = List {
            block: Block::default().borders(Borders::ALL).title(title),
            title: title,
            paragraph: Paragraph::new(vec![]),
            line_vec: tables_vec,
            focused_idx: 0,
        };

        list.line_vec[list.focused_idx as usize].set_active();
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

    pub fn take_focus(&mut self) {
        self.block = Block::default()
                            .title(self.title)
                            .borders(Borders::ALL)
                            .border_style(Style::default().cyan())
    }

    pub fn lose_focus(&mut self) {
        self.block = Block::default()
                            .title(self.title)
                            .borders(Borders::ALL);
    }
    
    pub fn active_item(&self) -> String {
        self.line_vec[self.focused_idx as usize].name.clone()
    }


    // increment active tab and reset paragraph
    pub fn scroll(&mut self, direction: MoveDirection) {
        // set current active tab to inactive
        self.line_vec[self.focused_idx as usize].set_inactive();
        // increment active index and mod it by length of tabs
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
    
}
