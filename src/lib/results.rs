use ratatui::{style::Style, widgets::{Block, Borders, Row, Table}};



pub struct Results<'a>{
    columns: Vec<String>,
    values: Vec<Vec<String>>,
    pub block: Block<'a>,
    pub table: Table<'a>,
} 



impl <'a>Results<'a> {
    pub fn new() -> Results<'a> {
        Results{ 
            columns: Vec::new(),
            values: Vec::new(),
            table: Table::default(),
            block: Block::default()
            .title("results")
            .borders(Borders::ALL)
        }
    }

    pub fn take_focus(&mut self) {
        self.block = Block::default()
                            .title("results")
                            .borders(Borders::ALL)
                            .border_style(Style::default().cyan())
    }

    pub fn lose_focus(&mut self) {
        self.block = Block::default()
                            .title("results")
                            .borders(Borders::ALL);
    }

    pub fn set_results(&mut self, columns: Vec<String>, values: Vec<Vec<String>>) {
        self.columns = columns;
        self.values = values;
        self.update_table();
    }

    fn update_table(&mut self) {
        
        let mut rows = Vec::new();
        let cols = Row::new(self.columns.clone());

        for result_row in self.values.clone() {
            rows.push(Row::new(result_row));
        }
        self.table = Table::default()
                        .header(cols)
                        .rows(rows);
    }
    
    pub fn scroll_down(&mut self) {
      if let Some(last) = self.values.pop() {
          let curr = self.values.clone();
          self.values = Vec::new();
          self.values.push(last);
          self.values.extend(curr);
          self.update_table();
      }
    }

    pub fn scroll_up(&mut self) {
       let first = self.values.remove(0);
       self.values.push(first);
       self.update_table();
    }

    pub fn scroll_right(&mut self) {
         let mut new_rows = Vec::new();
         for row in &mut self.values  {
             let mut new_row = Vec::new();
             if let Some(last) = row.pop() {
                 new_row.push(last);   
             }
             new_row.extend(row.clone());
             new_rows.push(new_row);
         }
         self.values = new_rows;
         self.update_table();
    }


    pub fn scroll_left(&mut self) {
        for row in &mut self.values {
            let first = row.remove(0);
            row.push(first);
        }
        self.update_table();
    }
}
