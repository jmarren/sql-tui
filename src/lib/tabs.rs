
use crate::lib::{list::List};



pub struct SideTabs<'a> {
    pub list: List<'a>
}


// pub struct SideTabs<'a> {
//     list: List<'a>
// }
//
impl<'a> SideTabs<'a> {
    pub fn new() -> Self {
        Self{
            list: List::new("", vec![" editor ".to_string(), " tables ".to_string()])
        }
    }
    
    pub fn active_tab(&self) -> &str {
        match self.list.active_item_name() {
            Some(name) if name == " tables ".to_string() => " tables ", 
            Some(name) if name == " editor ".to_string() => " editor ", 
            Some(_) | None  =>  panic!("tab name is not tables or editor!"),
        }
    }

}

