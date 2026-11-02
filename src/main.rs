mod app;
mod layout;
mod session;
mod state;
mod text;
mod ui;

use std::io;
use std::time::Duration;

use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{self, Event, KeyEventKind};

fn main() -> io::Result<()> {
    let mut app = app::App::new();
    let mut terminal = ratatui::init();
    let result = run(&mut terminal, &mut app);
    ratatui::restore();
    result
}

fn run(terminal: &mut DefaultTerminal, app: &mut app::App) -> io::Result<()> {
    while !app.quit {
        app.tick();
        terminal.draw(|f| ui::draw(f, app))?;
        if event::poll(Duration::from_millis(50))?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            app.on_key(key);
        }
    }
    Ok(())
}
