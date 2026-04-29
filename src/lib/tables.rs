// use ratatui::{Frame, layout::Rect};

use crate::lib::{ list::List};




pub struct Tables<'a> {
    pub list: List<'a>
}

impl<'a> Tables<'a> {
    pub fn new(table_names: Vec<String>) -> Self {
        Tables{
            list: List::new("tables", table_names)
        }
    }

    pub fn expand_focused(&mut self) {
        self.list.insert_items(vec![ " ├SELECT *".to_string()]);
    }
}

