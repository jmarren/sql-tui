use ratatui::{macros::ratatui_core, style::{Color, Modifier, Style}, text::Span};

pub struct Styles {
    pub focused_border: Style,
    pub unfocused_border: Style,
    pub active_tab: Style,
    pub inactive_tab: Style,
}

impl Styles {
    pub fn new() -> Self {
        Self {
            focused_border: Style::default().fg(Color::Gray),
            unfocused_border: Style::default(),
            active_tab: Style::default().fg(Color::Black).bg(Color::White).add_modifier(Modifier::BOLD),
            inactive_tab: Style::default().fg(Color::DarkGray),
        }
    }
}

pub enum TextColor {
    BurntOrange,
    Cyan,
    Magenta,
    Gray,
    Blue1,
    Todo,
    Todo2,
}

impl TextColor {
    pub fn highlight<'a>(&self, text: String) -> Span<'a> {
        match self {
            TextColor::BurntOrange => Span::raw(text).style(Style::default().fg(ratatui_core::style::Color::Rgb(240, 120, 100))),
            TextColor::Cyan        => Span::raw(text).style(Style::default().fg(ratatui_core::style::Color::Cyan)),
            TextColor::Magenta     => Span::raw(text).style(Style::default().fg(ratatui_core::style::Color::Magenta)),
            TextColor::Gray        => Span::raw(text).style(Style::default().fg(ratatui_core::style::Color::Gray)),
            TextColor::Blue1       => Span::raw(text).style(Style::default().fg(ratatui_core::style::Color::Rgb(103, 85, 230))),
            TextColor::Todo        => Span::raw(text).style(Style::default().fg(ratatui_core::style::Color::Rgb(90, 25, 210))),
            TextColor::Todo2       => Span::raw(text).style(Style::default().fg(ratatui_core::style::Color::Rgb(20, 25, 180))),
        }
    }
}
