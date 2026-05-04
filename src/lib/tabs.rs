
use std::{cell::RefCell, rc::Rc};

use ratatui::style::{Color, Style};

use crate::lib::list::{List, ListItem};



pub struct SideTabs<'a> {
    pub list: Rc<RefCell<List<'a>>>,
}


// pub struct SideTabs<'a> {
//     list: List<'a>
// }
//
impl<'a> SideTabs<'a> {
    pub fn new() -> Self {
        let list = List::new("");
        let list_ref = Rc::new(RefCell::new(list));
    
        let editor = ListItem::new(" editor ".to_string(), Style::default().fg(Color::DarkGray), super::list::ItemType::Tab, list_ref.clone());
        let tables = ListItem::new(" tables ".to_string(), Style::default().fg(Color::DarkGray), super::list::ItemType::Tab, list_ref.clone());
        list_ref.borrow_mut().items.extend(vec![editor,tables]);
        list_ref.borrow_mut().update_paragraph();
        list_ref.borrow_mut().make_first_active();
        Self{
            list: list_ref,
        }
    }
    
    pub fn active_tab(&self) -> &str {
        match self.list.borrow().active_item_name() {
            Some(name) if name == " tables ".to_string() => " tables ", 
            Some(name) if name == " editor ".to_string() => " editor ", 
            Some(_) | None  =>  panic!("tab name is not tables or editor!"),
        }
    }

}

