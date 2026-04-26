use ratatui::{style::{Color, Modifier, Style}, text::Line, widgets::{Block, Borders, Paragraph, Widget}};


static EDITOR: &str = " editor ";
static TABLES: &str = " tables ";


pub enum TabKind {
    Editor,
    Tables,
}


pub struct Tabs<'a> {
    pub paragraph: Paragraph<'a>,
    tabs: [Tab<'a>;2],
    active_idx: i32,
    pub block: Block<'a>,   
}

pub struct Tab<'a> {
    kind: TabKind,
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

impl <'a>Tab<'a> {
    pub fn new(kind: TabKind, active: bool) -> Tab<'a> {
        // create tab
        let text = match kind {
            TabKind::Tables => TABLES, 
            TabKind::Editor => EDITOR,
        };

        let mut tab = Tab {
            kind: kind,
            line: Line::from(text),
            active_style: make_active_style(),
            inactive_style: make_inactive_style(),
        };
        
        // set active or inactive
        if active {
          tab.set_active();
        } else {
          tab.set_inactive();
        }
        tab
    }

    fn set_active(&mut self) {
        self.line = self.line.clone().style(self.active_style);
    }

    fn set_inactive(&mut self)  {
        self.line = self.line.clone().style(self.inactive_style);
    }

}

impl <'a>Tabs<'a> {
    pub fn new() -> Tabs<'a> {
        // create array of tabs w/ first active and rest inactive
        let tabs = [Tab::new(TabKind::Editor, true) , Tab::new(TabKind::Tables, false)];
        let mut tabs = Tabs { 
            paragraph: Paragraph::new(vec![]),
            tabs: tabs,
            active_idx: 0,
            block: Block::default().borders(Borders::ALL),
        };
        // set the paragraph
        tabs.update_paragraph();
        tabs
    }
    
    // collect lines into a vector
    fn lines(&self) -> Vec<Line<'a>> {
        self.tabs.iter().map(| tab | { tab.line.clone() }).collect()
    }

    // set the paragraph field using the lines
    fn update_paragraph(&mut self) {
        self.paragraph = Paragraph::new(self.lines()).block(Block::default())
    }

    // increment active tab and reset paragraph
    pub fn scroll(&mut self) {
        // set current active tab to inactive
        self.tabs[self.active_idx as usize].set_inactive();
        // increment active index and mod it by length of tabs
        self.active_idx = (self.active_idx + 1) % self.tabs.len() as i32;
        // set new current active tab to active
        self.tabs[self.active_idx as usize].set_active();
        // get vec of lines 
        self.update_paragraph();
    }
    
    pub fn active_tab(&self) -> &TabKind {
        &self.tabs[self.active_idx as usize].kind
    }


    pub fn take_focus(&mut self) {
        self.block = Block::default()
                            .borders(Borders::ALL)
                            .border_style(Style::default().cyan())
    }

    pub fn lose_focus(&mut self) {
        self.block = Block::default()
                            .borders(Borders::ALL);
    }
    
}
