
use std::{cell::{Cell, RefCell}, rc::Rc};

use ratatui::{Frame, layout::Rect, style::{Color, Modifier, Style}, text::Line, widgets::{Block, Borders, Paragraph}};

use crate::lib::{Focusable, command::MoveDirection};

// #[derive(Copy)]
pub struct ListItem<'a>
{
    line: Line<'a>,
    item_type: ItemType,
    list: Rc<RefCell<List<'a>>>,
    // on_select: Box<dyn FnMut() -> ()>,
    // list: Rc<&'a mut List<'a>>,
    // on_select: Rc<dyn Fn() + 'a>,
    // active_idx: usize,
    pub expanded: bool,
}

// fn make_active_style() -> Style {
//     Style::default().fg(Color::Black).bg(Color::White).add_modifier(Modifier::BOLD)
// }
//
// fn make_inactive_style() -> Style {
//     Style::default().fg(Color::DarkGray)
// }

#[derive(Copy, Clone)]
pub enum ItemType {
    Table,
    Statement,
    Tab,
}

impl <'a>ListItem<'a>{
    pub fn new(name: String, style: Style, item_type: ItemType, list: Rc<RefCell<List<'a>>>) -> ListItem<'a> {
        ListItem {
            line: Line::from(name).style(style),
            expanded: false,
            item_type: item_type,
            list: list,
        }
    }

    pub fn set_expanded(&mut self) {
        self.expanded = true;
    }
    
    pub fn expand(&self) {
        let active_idx: usize;
        {
            active_idx = self.list.borrow().active_idx.clone();
        }
        let select_item = ListItem::new(" ├SELECT *".to_string(),Style::default().fg(Color::DarkGray), ItemType::Statement, self.list.clone());
        {
        self.list.borrow_mut().items.insert(active_idx, select_item);
        }
    }
    //
    // fn compress(&mut self) {
    //     self.lines = vec![self.lines.remove(0)];
    //     self.expanded = false;
    //     self.active_idx = 0;
    // }

    fn set_style(&mut self, style: Style) {
        self.line = self.line.clone().style(style);
    }

    fn name(&self) -> &str {
        &(*self.line.spans[0].content)
    }
}

// #[derive(Copy)]
pub struct List<'a>{
    pub block: Block<'a>,
    pub paragraph: Paragraph<'a>,
    pub items: Vec<ListItem<'a>>,
    pub active_idx: usize, 
    title: &'a str,
    active_style: Style,
    inactive_style: Style,
}

impl <'a>List<'a> {
    pub fn new(title: &'a str) -> List<'a> {
        
        let active_style = Style::default().fg(Color::Black).bg(Color::White).add_modifier(Modifier::BOLD);
        let inactive_style = Style::default().fg(Color::DarkGray);

         List {
            block: Block::default().borders(Borders::ALL).title(title),
            title: title,
            paragraph: Paragraph::new(vec![]),
            items: Vec::new(),
            active_idx: 0,
            active_style: active_style,
            inactive_style: inactive_style,
        }
    }

    // pub fn select_active(&mut self) {
    //     match (self.items[self.active_idx].item_type, self.items[self.active_idx].expanded) {
    //         (ItemType::Table, false) => {
    //             self.insert_items(vec![ " ├SELECT *".to_string()], super::list::ItemType::Statement);
    //             self.items[self.active_idx].expanded = true;
    //         },
    //         (ItemType::Table, true) => {
    //             self.remove_items(1);
    //             self.items[self.active_idx].expanded = false;
    //         },
    //         _ => (),
    //     }
    //     self.update_paragraph();
    // }

    pub fn make_first_active(&mut self) {
        self.items[0].set_style(self.active_style);
        self.update_paragraph();
    }


    fn remove_items(&mut self, num: usize) {
        self.items.drain(self.active_idx+1..self.active_idx +1 + num);
    }
    
    pub fn handle_select(&self) {
        self.items[self.active_idx].expand();
    }


    // fn self_ref(&self) -> Rc<RefCell<&List<'a>>> {
    //      Rc::new(RefCell::new(self))
    // }
    //
    // pub fn extend_items(&self, items: Vec<String>, item_type: ItemType, list: Rc<RefCell<List<'a>>>) {
    //     let inactive_style = self.inactive_style.clone();
    //     // let mut list_ref = Rc::new(RefCell::new(self));                
    //
    //     {
    //
    //         items.iter().for_each(| item | {
    //             list.borrow_mut().items.push(ListItem::new((*item).clone(), inactive_style, item_type, list.clone()));
    //         });
    //         // let list_items: Vec<ListItem<'a>> = items
    //         //     .into_iter()
    //         //     .map(| item | {
    //         //         ListItem::new(item, inactive_style, item_type, list.clone())
    //         //     })
    //         //     .collect();
    //
    //         // list.borrow_mut().items.extend(list_items);
    //     }
    //     list.borrow_mut().update_paragraph();
    //
    //
    //     // list_ref.get_mut().items.get_mut().extend(list_items);
    //     // list_ref.get_mut().update_paragraph();
    // }

    // pub fn insert_items(&mut self, items: Vec<String>, item_type: ItemType) {
    //     let new_lines: Vec<ListItem> = items
    //         .into_iter()
    //         .map(| item | ListItem::new(item, self.inactive_style, item_type))
    //         .collect();
    //
    //     self.items.splice(self.active_idx + 1 ..self.active_idx + 1, new_lines);
    //
    //     self.update_paragraph();
    // }
    //

    // collect lines into a vector
    fn lines(&self) -> Vec<Line<'a>> {
        self.items
            .iter()
            .map(| item | item.line.clone() )
            .collect()
    }

    // set the paragraph field using the lines
    pub fn update_paragraph(&mut self) {
        self.paragraph = Paragraph::new(self.lines()).block(Block::default())
    }

    pub fn active_item_name(&self) -> Option<&str> {
        if self.items.len() < 1 {
            None
        } else {
            Some(self.items[self.active_idx as usize].name())
        }
    }

    pub fn active_item(&mut self) -> &mut ListItem<'a>  {
        match self.items.get_mut(self.active_idx) {
            Some(item) => item, 
            _ => panic!("no active item"),
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
        self.items[self.active_idx as usize].set_style(self.inactive_style.clone());
        // increment or decrement focused index
        match direction {
            MoveDirection::Up if self.active_idx > 0 => self.active_idx -= 1,
            MoveDirection::Down if self.active_idx < self.items.len() - 1 => self.active_idx += 1,
            _ => ()
        };

        // set new current active tab to active
        self.items[self.active_idx as usize].set_style(self.active_style.clone());
        // get vec of lines 
        self.update_paragraph();
    }
    
}
