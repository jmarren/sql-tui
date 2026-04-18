mod lib;

// extern mod colorize;
// Import the trait implemented for &'static str and ~str
// use colorize::AnsiColor;
// Import the colors for the global
// use colorize::{BrightRed, Blue};
fn main() -> anyhow::Result<()> {
    println!("hi");
    ratatui::run(lib::app)
}


