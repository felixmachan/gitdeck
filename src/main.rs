mod app;
mod commands;
mod config;
mod git;
mod models;
mod ui;

use anyhow::Result;
use app::App;

fn main() -> Result<()> {
    let mut app = App::new()?;
    app.run()
}
