use ratatui::{Frame, layout::Rect, style::Style, widgets::{Block, Borders, Row, Table}};

use crate::lib::{Focusable, command::MoveDirection};



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

    pub fn set_results(&mut self, columns: Vec<String>, values: Vec<Vec<String>>) {
        self.columns = columns;
        self.values = values;
        self.update_table();
    }

    fn update_table(&mut self) {
        let cols = Row::new(self.columns.clone());
            
        let rows: Vec<Row> = self
            .values
            .iter()
            .map(| row | Row::new(row.clone()))
            .collect();

        self.table = Table::default()
                        .header(cols)
                        .rows(rows);
    }


     fn scroll_right(&mut self) {
        self.values
            .iter_mut()
            .for_each(| val | {
                val.rotate_left(1);
            });
    
        self.columns.rotate_left(1);
    }


    fn scroll_left(&mut self) {
        self.values
            .iter_mut()
            .for_each(| val | {
                val.rotate_right(1);
            });
        self.columns.rotate_right(1);
    }

    pub fn render(&mut self, frame: &mut Frame, rect: Rect) {
        frame.render_widget(&self.block, rect);
        frame.render_widget(&self.table, self.block.inner(rect));
    }
}

impl <'a>Focusable for Results<'a> {

    fn take_focus(&mut self) {
        self.block = Block::default()
                            .title("results")
                            .borders(Borders::ALL)
                            .border_style(Style::default().cyan())
    }

     fn lose_focus(&mut self) {
        self.block = Block::default()
                            .title("results")
                            .borders(Borders::ALL);
    }

    fn move_cursor(&mut self, direction: MoveDirection) {
        if self.values.len() < 1 {
            return;
        }
        match direction {
            MoveDirection::Up => self.values.rotate_right(1),
            MoveDirection::Down => self.values.rotate_left(1),
            MoveDirection::Left => self.scroll_left(),
            MoveDirection::Right => self.scroll_right(),
        }

        self.update_table();
    }

}
