// OrbitOS — Interactive TUI Installer Entry

pub mod events;
pub mod state;
pub mod types;
pub mod ui;

use std::io::stdout;
use anyhow::Result;
use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        cursor,
        event::{self, Event},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    Terminal,
};

pub use state::InstallerState;

pub fn run_installer() -> Result<Option<InstallerState>> {
    // 1. Prepare terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, cursor::Hide)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 2. State & Event Loop
    let mut state = InstallerState::new();

    let loop_res = run_app_loop(&mut terminal, &mut state);

    // 3. Always restore terminal state cleanly
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, cursor::Show)?;
    terminal.show_cursor()?;

    loop_res?;

    if state.current_step == state::Step::Done {
        Ok(Some(state))
    } else {
        Ok(None)
    }
}

fn run_app_loop(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    state: &mut InstallerState,
) -> Result<()> {
    while !state.should_exit {
        terminal.draw(|f| ui::render(f, state)).map_err(|e| anyhow::anyhow!("{}", e))?;

        if let Event::Key(key) = event::read()? {
            events::handle_key_event(state, key);
        }
    }
    Ok(())
}
