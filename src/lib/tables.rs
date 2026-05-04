// use ratatui::{Frame, layout::Rect};

use std::{cell::RefCell, rc::Rc};

use ratatui::style::{Color, Style};

use crate::lib::list::{ItemType, List, ListItem};



// #[derive(DerefMut)]
pub struct Tables<'a> {
    pub list: Rc<RefCell<List<'a>>>,
}

impl<'a> Tables<'a> {
    pub fn new(table_names: Vec<String>) -> Self {
        let list = List::new("tables");
        let list_ref = Rc::new(RefCell::new(list));

        let list_items = table_names.iter().map(| item | {
                ListItem::new((*item).clone(), Style::default().fg(Color::DarkGray), super::list::ItemType::Table, list_ref.clone())
        });
        let mut mut_list = list_ref.borrow_mut();
        mut_list.items.extend(list_items);
        mut_list.make_first_active();
        mut_list.update_paragraph();
        Tables{
            list: list_ref.clone(),
        }
    }

    pub fn expand_focused(&mut self) {
        let mut list = self.list.borrow_mut();

        let select_item = ListItem::new(
                " ├SELECT *".to_string(),
                Style::default().fg(Color::DarkGray),
                ItemType::Statement,
                self.list.clone()
            );

        let active_idx = list.active_idx;

    
        if !list.items[active_idx].expanded {
            list.items.insert(active_idx + 1, select_item);
            list.update_paragraph();
            list.items[active_idx].set_expanded();
        } else {
            list.items.remove(active_idx + 1);
            list.update_paragraph();
            list.items[active_idx].expanded = false;
        }
    }
}

