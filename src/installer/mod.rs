// OrbitOS — Interactive TUI Installer Entry

pub mod events;
pub mod generator;
pub mod git_sync;
pub mod orbit_sim;
pub mod probe;
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

    if state.current_step == state::Step::Done && state.chosen_deployment.is_some() {
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

        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                events::handle_key_event(state, key);
            }
        }
        state.tick_animation();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::orbit_sim::OrbitSimulation;
    use super::state::{HostSubStep, InstallerState, Step};
    use super::types::{Account, AccountType, CpuType, DesktopShell, GpuType, HostConfig, HostType};
    use super::generator::{write_account_files, write_host_files};
    use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn test_orbit_simulation_dimensions_and_frames() {
        let sim = OrbitSimulation::new();

        // 1. Extreme small / zero bounds should not panic
        let _ = sim.render(0, 0, 0.0);
        let _ = sim.render(4, 2, 1.0);

        // 2. Standard TUI sidebar dimensions across 120 animation ticks
        for tick in 0..120 {
            let t = tick as f64 * 0.04;
            let grid = sim.render(30, 16, t);
            assert_eq!(grid.len(), 16);
            assert_eq!(grid[0].len(), 30);
        }
    }

    #[test]
    fn test_state_flow_and_navigation() {
        let mut state = InstallerState::new();

        // 1. Starts on Welcome
        assert_eq!(state.current_step, Step::Welcome);
        assert_eq!(state.current_scope(), "ONBOARDING");

        // 2. Advances to EnvironmentPrompt
        state.set_step(Step::EnvironmentPrompt);
        assert_eq!(state.current_scope(), "PROBE");

        // 3. Move to GithubForkPrompt
        state.set_step(Step::GithubForkPrompt);
        assert_eq!(state.current_scope(), "WORKSPACE");

        // 4. Accounts setup
        state.set_step(Step::AccountsMenu);
        state.accounts.clear();
        state.accounts.push(Account {
            name: "testuser".to_string(),
            account_type: AccountType::User,
            description: "Primary Administrator".to_string(),
        });
        state.accounts.push(Account {
            name: "devuser".to_string(),
            account_type: AccountType::Restricted,
            description: "Restricted Dev".to_string(),
        });
        state.accounts.push(Account {
            name: "incognito".to_string(),
            account_type: AccountType::Anonymous,
            description: "Anonymous Session".to_string(),
        });
        assert_eq!(state.accounts.len(), 3);

        // 5. Host configuration
        let mut host = HostConfig::new("laptop".to_string(), HostType::Laptop);
        host.cpu = CpuType::Amd;
        host.gpu = GpuType::NvidiaHybrid;
        host.des_and_wms = vec!["hyprland".to_string()];
        host.desktop_shell = Some(DesktopShell::End4Pc);
        host.ai_enabled = true;
        host.ai_tools = vec!["opencode".to_string()];
        state.hosts.push(host);

        state.set_step(Step::HostConfig {
            host_idx: 0,
            sub_step: HostSubStep::Hardware,
        });
        assert_eq!(state.current_scope(), "HOST: LAPTOP");

        // Navigate sub-steps forward
        state.next_host_substep(0, &HostSubStep::Hardware);
        assert_eq!(state.current_step, Step::HostConfig { host_idx: 0, sub_step: HostSubStep::DesktopsAndWms });

        state.next_host_substep(0, &HostSubStep::DesktopsAndWms);
        assert_eq!(state.current_step, Step::HostConfig { host_idx: 0, sub_step: HostSubStep::Shell });

        state.next_host_substep(0, &HostSubStep::Shell);
        assert_eq!(state.current_step, Step::HostConfig { host_idx: 0, sub_step: HostSubStep::AppsAndTerminal });

        // Navigate sub-steps backward
        state.prev_host_substep(0, &HostSubStep::AppsAndTerminal);
        assert_eq!(state.current_step, Step::HostConfig { host_idx: 0, sub_step: HostSubStep::Shell });
    }

    #[test]
    fn test_event_loop_keystroke_simulation() {
        let mut state = InstallerState::new();

        // Simulate Enter on Welcome
        events::handle_key_event(&mut state, KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert!(matches!(state.current_step, Step::EnvironmentPrompt | Step::GithubForkPrompt));

        // Set to AccountsMenu and test key navigation
        state.set_step(Step::AccountsMenu);
        assert_eq!(state.selected_index, 0);

        // Down arrow
        events::handle_key_event(&mut state, KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        assert_eq!(state.selected_index, 1);

        // Up arrow
        events::handle_key_event(&mut state, KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
        assert_eq!(state.selected_index, 0);

        // Press 'd' to test delete bounds safety
        events::handle_key_event(&mut state, KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE));

        // Test Bail Out confirmation toggle
        state.set_step(Step::BailOutConfirm);
        assert_eq!(state.selected_index, 0);
        events::handle_key_event(&mut state, KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        assert_eq!(state.selected_index, 1);
        events::handle_key_event(&mut state, KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(state.current_step, Step::DeploymentMenu);
    }

    #[test]
    fn test_nix_file_generation() {
        let tmp_dir = std::env::temp_dir().join(format!("orbit_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp_dir);
        let _ = std::fs::create_dir_all(&tmp_dir);

        // 1. Write accounts
        let user_acc = Account {
            name: "musa".to_string(),
            account_type: AccountType::User,
            description: "Admin".to_string(),
        };
        let anon_acc = Account {
            name: "incognito".to_string(),
            account_type: AccountType::Anonymous,
            description: "RAM tmpfs".to_string(),
        };

        write_account_files(&tmp_dir, &user_acc).expect("write_account_files user failed");
        write_account_files(&tmp_dir, &anon_acc).expect("write_account_files anon failed");

        let user_file = tmp_dir.join("users/musa/default.nix");
        let anon_file = tmp_dir.join("users/incognito/default.nix");
        assert!(user_file.exists());
        assert!(anon_file.exists());

        let user_nix = std::fs::read_to_string(&user_file).unwrap();
        assert!(user_nix.contains("users.users.musa"));
        assert!(user_nix.contains("wheel"));

        let anon_nix = std::fs::read_to_string(&anon_file).unwrap();
        assert!(anon_nix.contains("fileSystems.\"/home/incognito\""));
        assert!(anon_nix.contains("tmpfs"));

        // 2. Write host
        let mut host = HostConfig::new("workstation".to_string(), HostType::PC);
        host.cpu = CpuType::Amd;
        host.gpu = GpuType::NvidiaDesktop;
        host.des_and_wms = vec!["hyprland".to_string()];
        host.desktop_shell = Some(DesktopShell::End4Pc);
        host.assigned_users = vec!["musa".to_string()];
        host.gaming = vec!["steam".to_string(), "gamemode".to_string()];

        write_host_files(&tmp_dir, &host).expect("write_host_files failed");

        let host_file = tmp_dir.join("hosts/workstation/default.nix");
        assert!(host_file.exists());

        let host_nix = std::fs::read_to_string(&host_file).unwrap();
        assert!(host_nix.contains("networking.hostName = \"workstation\";"));
        assert!(host_nix.contains("../../modules/hardware/amd-cpu.nix"));
        assert!(host_nix.contains("../../modules/hardware/nvidia-desktop.nix"));
        assert!(host_nix.contains("mySystem.desktop.shell = \"end4-pC\";"));
        assert!(host_nix.contains("steam = true;"));

        let _ = std::fs::remove_dir_all(&tmp_dir);
    }
}
