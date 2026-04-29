use ratatui::{Frame, layout::Rect, style::{Color, Modifier, Style}, text::Line, widgets::{Block, Borders, Paragraph}};

use crate::lib::{Focusable, command::MoveDirection};

#[derive(Clone)]
pub struct ListItem<'a> {
    line: Line<'a>,
}

// fn make_active_style() -> Style {
//     Style::default().fg(Color::Black).bg(Color::White).add_modifier(Modifier::BOLD)
// }
//
// fn make_inactive_style() -> Style {
//     Style::default().fg(Color::DarkGray)
// }



impl <'a>ListItem<'a> {
    pub fn new(name: String, style: Style) -> ListItem<'a> {
        ListItem {
            line: Line::from(name).style(style),
        }
    }

    fn set_style(&mut self, style: Style) {
        self.line = self.line.clone().style(style);
    }

    fn name(&self) -> &str {
        &(*self.line.spans[0].content)
    }
}


pub struct List<'a>{
    pub block: Block<'a>,
    line_vec: Vec<ListItem<'a>>,
    focused_idx: usize, 
    pub paragraph: Paragraph<'a>,
    title: &'a str,
    active_style: Style,
    inactive_style: Style,
}

impl <'a>List<'a> {
    pub fn new(title: &'a str, items: Vec<String>) -> List<'a> {
        
        let active_style = Style::default().fg(Color::Black).bg(Color::White).add_modifier(Modifier::BOLD);
        let inactive_style = Style::default().fg(Color::DarkGray);

        let mut list_items: Vec<ListItem> = items
            .into_iter()
            .map(| item | ListItem::new(item, inactive_style))
            .collect();
    
        if list_items.len() > 0 {
            list_items[0].set_style(active_style);
        }

        let mut list = List {
            block: Block::default().borders(Borders::ALL).title(title),
            title: title,
            paragraph: Paragraph::new(vec![]),
            line_vec: list_items,
            focused_idx: 0,
            active_style: active_style,
            inactive_style: inactive_style,
        };

        list.update_paragraph();
        list
    }

    pub fn insert_items(&mut self, items: Vec<String>) {
        let new_lines: Vec<ListItem<'a>> = items
            .into_iter()
            .map(| item | ListItem::new(item, self.inactive_style))
            .collect();

        let lines_before = &mut self.line_vec[..self.focused_idx + 1].to_vec();
        let lines_after = &mut self.line_vec[self.focused_idx + 1..].to_vec();
        
        let mut lines = Vec::new();

        lines.append(lines_before);
        lines.extend(new_lines);
        lines.append(lines_after);
    
        self.line_vec = lines;
        self.update_paragraph();
    }


    // collect lines into a vector
    fn lines(&self) -> Vec<Line<'a>> {
        self.line_vec.iter().map(| table | { table.line.clone() }).collect()
    }

    // set the paragraph field using the lines
    fn update_paragraph(&mut self) {
        self.paragraph = Paragraph::new(self.lines()).block(Block::default())
    }

    pub fn active_item_name(&self) -> Option<&str> {
        if self.line_vec.len() < 1 {
            None
        } else {
            Some(self.line_vec[self.focused_idx as usize].name())
        }
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


    // increment active tab and reset paragraph
    fn move_cursor(&mut self, direction: MoveDirection) {
        // set current active tab to inactive
        self.line_vec[self.focused_idx as usize].set_style(self.inactive_style.clone());
        // increment or decrement focused index
        match direction {
            MoveDirection::Up if self.focused_idx > 0 => self.focused_idx -= 1,
            MoveDirection::Down if self.focused_idx < self.line_vec.len() - 1 => self.focused_idx += 1,
            _ => ()
        };

        // set new current active tab to active
        self.line_vec[self.focused_idx as usize].set_style(self.active_style.clone());
        // get vec of lines 
        self.update_paragraph();
    }
    
}
