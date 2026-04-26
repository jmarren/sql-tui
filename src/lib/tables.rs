use ratatui::{style::{Color, Modifier, Style}, text::Line, widgets::{Block, Borders, Paragraph}};


pub struct Table<'a> {
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



impl <'a>Table<'a> {
    pub fn new(name: String) -> Table<'a> {
        Table {
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


pub struct Tables<'a>{
    pub block: Block<'a>,
    tables_vec: Vec<Table<'a>>,
    focused_idx: i32, 
    pub paragraph: Paragraph<'a>,
}

impl <'a>Tables<'a> {
    pub fn new(table_names: Vec<String>) -> Tables<'a> {
        let mut tables_vec = Vec::<Table>::new();

        for name in table_names {
            tables_vec.push(Table::new(name));
        }
        let mut tables = Tables {
            block: Block::default().borders(Borders::ALL).title("tables"),
            paragraph: Paragraph::new(vec![]),
            tables_vec,
            focused_idx: 0,
        };

        tables.update_paragraph();

        tables
    }


    // collect lines into a vector
    fn lines(&self) -> Vec<Line<'a>> {
        self.tables_vec.iter().map(| table | { table.line.clone() }).collect()
    }

    // set the paragraph field using the lines
    fn update_paragraph(&mut self) {
        self.paragraph = Paragraph::new(self.lines()).block(Block::default())
    }

    // increment active tab and reset paragraph
    pub fn handle_tab_pressed(&mut self) {
        // set current active tab to inactive
        self.tables_vec[self.focused_idx as usize].set_inactive();
        // increment active index and mod it by length of tabs
        self.focused_idx = (self.focused_idx + 1) % self.tables_vec.len() as i32;
        // set new current active tab to active
        self.tables_vec[self.focused_idx as usize].set_active();
        // get vec of lines 
        self.update_paragraph();
    }

    pub fn focused_table(&self) -> &Table<'a> {
        &self.tables_vec[self.focused_idx as usize]
    }
}
