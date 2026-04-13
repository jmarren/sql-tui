mod lib;
fn main() -> anyhow::Result<()> {
    ratatui::run(lib::app)
}


