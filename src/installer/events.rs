// OrbitOS — Installer Event Handling and Action Dispatcher

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::installer::generator::{write_account_files, write_host_files};
use crate::installer::git_sync::{clone_fork, commit_and_push_step, discard_and_bail, get_orbitos_dir};
use crate::installer::state::{
    HostSubStep, InstallerState, Step, AVAILABLE_AI_TOOLS, AVAILABLE_APPS, AVAILABLE_DES_WMS,
    AVAILABLE_GAMING,
};
use crate::installer::types::{
    Account, AccountType, CpuType, DeploymentChoice, DesktopShell, GpuType, HostConfig, HostType,
    RemoteDesktopRole, TerminalShell,
};

pub fn handle_key_event(state: &mut InstallerState, key: KeyEvent) {
    // 0. If a modal is currently displayed, dismiss it
    if state.modal_message.is_some() {
        if key.code == KeyCode::Enter || key.code == KeyCode::Esc || key.code == KeyCode::Char(' ') {
            state.modal_message = None;
        }
        return;
    }

    // 1. Text input modes take priority
    match &state.current_step {
        Step::GithubForkInput => {
            match key.code {
                KeyCode::Enter => {
                    let fork = state.text_input.trim().to_string();
                    if !fork.is_empty() {
                        state.github_fork = fork.clone();
                        match clone_fork(&fork) {
                            Ok(_) => {
                                state.set_step(Step::AccountsMenu);
                            }
                            Err(e) => {
                                state.modal_message = Some((
                                    "Clone Failed".to_string(),
                                    format!("{}\n\nPlease verify your repository URL or fork permissions.", e),
                                ));
                            }
                        }
                    }
                }
                KeyCode::Esc => {
                    state.set_step(Step::GithubForkPrompt);
                }
                KeyCode::Backspace => {
                    state.text_input.pop();
                }
                KeyCode::Char(c) => {
                    state.text_input.push(c);
                }
                _ => {}
            }
            return;
        }

        Step::AddAccountName { account_type } => {
            let acc_type = *account_type;
            match key.code {
                KeyCode::Enter => {
                    let name = state.text_input.trim().to_lowercase();
                    if !name.is_empty() {
                        let desc = match acc_type {
                            AccountType::User => "Primary Administrator".to_string(),
                            AccountType::Restricted => "Restricted User".to_string(),
                            AccountType::Anonymous => "Incognito RAM-only Session".to_string(),
                        };
                        let new_acc = Account {
                            name,
                            account_type: acc_type,
                            description: desc,
                        };
                        let repo_dir = get_orbitos_dir();
                        let _ = write_account_files(&repo_dir, &new_acc);
                        state.accounts.push(new_acc);
                        state.set_step(Step::AccountsMenu);
                    }
                }
                KeyCode::Esc => {
                    state.set_step(Step::AccountsMenu);
                }
                KeyCode::Backspace => {
                    state.text_input.pop();
                }
                KeyCode::Char(c) => {
                    if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                        state.text_input.push(c);
                    }
                }
                _ => {}
            }
            return;
        }

        Step::AddHostName { host_type } => {
            let h_type = *host_type;
            match key.code {
                KeyCode::Enter => {
                    let name = state.text_input.trim().to_lowercase();
                    if !name.is_empty() {
                        let mut config = HostConfig::new(name, h_type);
                        config.assigned_users = state.accounts.iter().map(|a| a.name.clone()).collect();
                        let new_idx = state.hosts.len();
                        state.hosts.push(config);
                        state.set_step(Step::HostConfig {
                            host_idx: new_idx,
                            sub_step: HostSubStep::Hardware,
                        });
                    }
                }
                KeyCode::Esc => {
                    state.set_step(Step::FleetPrompt);
                }
                KeyCode::Backspace => {
                    state.text_input.pop();
                }
                KeyCode::Char(c) => {
                    if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                        state.text_input.push(c);
                    }
                }
                _ => {}
            }
            return;
        }
        _ => {}
    }

    // 2. Global exit keys (when not in text input)
    if key.code == KeyCode::Char('q') || (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL)) {
        if state.current_step == Step::BailOutConfirm {
            state.should_exit = true;
        } else {
            state.set_step(Step::BailOutConfirm);
        }
        return;
    }

    // 3. Navigation and Step Actions
    match &state.current_step {
        Step::Welcome => {
            if key.code == KeyCode::Enter || key.code == KeyCode::Char(' ') {
                if !state.detected.is_live_image {
                    state.set_step(Step::EnvironmentPrompt);
                } else {
                    state.set_step(Step::GithubForkPrompt);
                }
            }
        }

        Step::EnvironmentPrompt => {
            match key.code {
                KeyCode::Up | KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('k') => {
                    state.selected_index = (state.selected_index + 1) % 2;
                }
                KeyCode::Enter => {
                    if state.selected_index == 0 {
                        // Adopt current machine identity
                        let d = &state.detected;
                        let mut primary_host = HostConfig::new(d.hostname.clone(), d.host_type);
                        primary_host.cpu = d.cpu;
                        primary_host.gpu = d.gpu;
                        primary_host.assigned_users = state.accounts.iter().map(|a| a.name.clone()).collect();
                        state.hosts.clear();
                        state.hosts.push(primary_host);
                        state.bundle_ssh_secrets = true;
                    }
                    state.set_step(Step::GithubForkPrompt);
                }
                KeyCode::Esc => {
                    state.set_step(Step::Welcome);
                }
                _ => {}
            }
        }

        Step::GithubForkPrompt => {
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    state.selected_index = if state.selected_index > 0 { state.selected_index - 1 } else { 1 };
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    state.selected_index = (state.selected_index + 1) % 2;
                }
                KeyCode::Enter => {
                    match state.selected_index {
                        0 => {
                            let user_info = crate::util::get_target_user();
                            let current_user = user_info.username;
                            let default_input = if state.detected.is_live_image || current_user == "nixos" || current_user == "root" {
                                "".to_string()
                            } else {
                                current_user
                            };
                            state.text_input = default_input;
                            state.set_step(Step::GithubForkInput);
                        }
                        1 => {
                            state.github_fork = "Orbit-Nix/OrbitOS".to_string();
                            match clone_fork("https://github.com/Orbit-Nix/OrbitOS.git") {
                                Ok(_) => state.set_step(Step::AccountsMenu),
                                Err(e) => {
                                    state.modal_message = Some((
                                        "Clone Failed".to_string(),
                                        format!("{}\n\nPlease check your network connection.", e),
                                    ));
                                }
                            }
                        }
                        _ => {}
                    }
                }
                KeyCode::Esc => {
                    if !state.detected.is_live_image {
                        state.set_step(Step::EnvironmentPrompt);
                    } else {
                        state.set_step(Step::Welcome);
                    }
                }
                _ => {}
            }
        }

        Step::AccountsMenu => {
            let total_items = 1 + state.accounts.len() + if state.accounts.is_empty() { 0 } else { 1 };
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    if state.selected_index > 0 { state.selected_index -= 1; }
                    else { state.selected_index = total_items.saturating_sub(1); }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if state.selected_index + 1 < total_items { state.selected_index += 1; }
                    else { state.selected_index = 0; }
                }
                KeyCode::Enter => {
                    if state.selected_index == 0 {
                        state.set_step(Step::AddAccountType);
                    } else if !state.accounts.is_empty() && state.selected_index == total_items - 1 {
                        // Commit and push accounts step
                        let repo_dir = get_orbitos_dir();
                        for acc in &state.accounts {
                            let _ = write_account_files(&repo_dir, acc);
                        }
                        let _ = commit_and_push_step("configure user accounts");

                        if state.hosts.is_empty() {
                            state.set_step(Step::PrimaryHostSetup);
                        } else {
                            state.set_step(Step::HostConfig {
                                host_idx: 0,
                                sub_step: HostSubStep::Hardware,
                            });
                        }
                    }
                }
                KeyCode::Char('d') | KeyCode::Delete => {
                    if state.selected_index >= 1 && state.selected_index <= state.accounts.len() {
                        let idx = state.selected_index - 1;
                        state.accounts.remove(idx);
                        state.selected_index = state.selected_index.saturating_sub(1);
                    }
                }
                _ => {}
            }
        }

        Step::AddAccountType => {
            let types = [AccountType::User, AccountType::Restricted, AccountType::Anonymous];
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    state.selected_index = if state.selected_index > 0 { state.selected_index - 1 } else { types.len() - 1 };
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    state.selected_index = (state.selected_index + 1) % types.len();
                }
                KeyCode::Enter => {
                    let chosen = types[state.selected_index];
                    let def_name = match chosen {
                        AccountType::User => "user".to_string(),
                        AccountType::Restricted => "dev".to_string(),
                        AccountType::Anonymous => "incognito".to_string(),
                    };
                    state.text_input = def_name;
                    state.set_step(Step::AddAccountName { account_type: chosen });
                }
                KeyCode::Esc => {
                    state.set_step(Step::AccountsMenu);
                }
                _ => {}
            }
        }

        Step::PrimaryHostSetup => {
            let types = [HostType::Laptop, HostType::PC, HostType::Server, HostType::Portable];
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    state.selected_index = if state.selected_index > 0 { state.selected_index - 1 } else { types.len() - 1 };
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    state.selected_index = (state.selected_index + 1) % types.len();
                }
                KeyCode::Enter => {
                    let chosen = types[state.selected_index];
                    let name = match chosen {
                        HostType::Laptop => "laptop".to_string(),
                        HostType::PC => "desktop".to_string(),
                        HostType::Server => "server".to_string(),
                        HostType::Portable => "portable".to_string(),
                    };
                    let mut config = HostConfig::new(name, chosen);
                    config.assigned_users = state.accounts.iter().map(|a| a.name.clone()).collect();
                    state.hosts.push(config);
                    state.set_step(Step::HostConfig {
                        host_idx: 0,
                        sub_step: HostSubStep::Hardware,
                    });
                }
                _ => {}
            }
        }

        Step::HostConfig { host_idx, sub_step } => {
            let h_idx = *host_idx;
            let current_sub = sub_step.clone();

            match current_sub {
                HostSubStep::Hardware => {
                    match key.code {
                        KeyCode::Up | KeyCode::Char('k') => {
                            if state.selected_index > 0 { state.selected_index -= 1; }
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            if state.selected_index < 2 { state.selected_index += 1; }
                        }
                        KeyCode::Char(' ') => {
                            if let Some(host) = state.hosts.get_mut(h_idx) {
                                match state.selected_index {
                                    0 => host.cpu = if host.cpu == CpuType::Amd { CpuType::Intel } else { CpuType::Amd },
                                    1 => host.gpu = if host.gpu == GpuType::Amd { GpuType::NvidiaDesktop } else { GpuType::Amd },
                                    2 => host.zram = !host.zram,
                                    _ => {}
                                }
                            }
                        }
                        KeyCode::Enter => {
                            state.next_host_substep(h_idx, &current_sub);
                        }
                        KeyCode::Esc => {
                            state.prev_host_substep(h_idx, &current_sub);
                        }
                        _ => {}
                    }
                }

                HostSubStep::DesktopsAndWms => {
                    let total = AVAILABLE_DES_WMS.len();
                    match key.code {
                        KeyCode::Up | KeyCode::Char('k') => {
                            if state.selected_index > 0 { state.selected_index -= 1; }
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            if state.selected_index + 1 < total { state.selected_index += 1; }
                        }
                        KeyCode::Char(' ') => {
                            if let Some(wm) = AVAILABLE_DES_WMS.get(state.selected_index) {
                                let id = wm.id.to_string();
                                if let Some(host) = state.hosts.get_mut(h_idx) {
                                    toggle_vec(&mut host.des_and_wms, id);
                                }
                            }
                        }
                        KeyCode::Enter => {
                            state.next_host_substep(h_idx, &current_sub);
                        }
                        KeyCode::Esc => {
                            state.prev_host_substep(h_idx, &current_sub);
                        }
                        _ => {}
                    }
                }

                HostSubStep::Shell => {
                    let shells = [DesktopShell::End4Pc, DesktopShell::Midnight, DesktopShell::Dms, DesktopShell::None];
                    match key.code {
                        KeyCode::Up | KeyCode::Char('k') => {
                            state.selected_index = if state.selected_index > 0 { state.selected_index - 1 } else { shells.len() - 1 };
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            state.selected_index = (state.selected_index + 1) % shells.len();
                        }
                        KeyCode::Enter => {
                            if let Some(host) = state.hosts.get_mut(h_idx) {
                                host.desktop_shell = Some(shells[state.selected_index]);
                            }
                            state.next_host_substep(h_idx, &current_sub);
                        }
                        KeyCode::Esc => {
                            state.prev_host_substep(h_idx, &current_sub);
                        }
                        _ => {}
                    }
                }

                HostSubStep::AppsAndTerminal => {
                    let total = 2 + AVAILABLE_APPS.len();
                    match key.code {
                        KeyCode::Up | KeyCode::Char('k') => {
                            if state.selected_index > 0 { state.selected_index -= 1; }
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            if state.selected_index + 1 < total { state.selected_index += 1; }
                        }
                        KeyCode::Char(' ') => {
                            if let Some(host) = state.hosts.get_mut(h_idx) {
                                if state.selected_index == 0 {
                                    host.terminal_shell = if host.terminal_shell == TerminalShell::Fish { TerminalShell::Bash } else { TerminalShell::Fish };
                                } else if state.selected_index == 1 {
                                    host.flatpak = !host.flatpak;
                                } else if let Some(app) = AVAILABLE_APPS.get(state.selected_index - 2) {
                                    let id = app.id.to_string();
                                    match app.category {
                                        "Browsers" => toggle_vec(&mut host.browsers, id),
                                        "File Managers" => toggle_vec(&mut host.file_managers, id),
                                        "IDEs" => toggle_vec(&mut host.ides, id),
                                        "Terminals" => toggle_vec(&mut host.terminals, id),
                                        _ => {}
                                    }
                                }
                            }
                        }
                        KeyCode::Enter => {
                            state.next_host_substep(h_idx, &current_sub);
                        }
                        KeyCode::Esc => {
                            state.prev_host_substep(h_idx, &current_sub);
                        }
                        _ => {}
                    }
                }

                HostSubStep::AiTools => {
                    let host_ai_enabled = state.hosts.get(h_idx).map(|h| h.ai_enabled).unwrap_or(false);
                    let total = if host_ai_enabled { 1 + AVAILABLE_AI_TOOLS.len() } else { 1 };

                    match key.code {
                        KeyCode::Up | KeyCode::Char('k') => {
                            if state.selected_index > 0 { state.selected_index -= 1; }
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            if state.selected_index + 1 < total { state.selected_index += 1; }
                        }
                        KeyCode::Char(' ') => {
                            if let Some(host) = state.hosts.get_mut(h_idx) {
                                if state.selected_index == 0 {
                                    host.ai_enabled = !host.ai_enabled;
                                } else if let Some(tool) = AVAILABLE_AI_TOOLS.get(state.selected_index - 1) {
                                    toggle_vec(&mut host.ai_tools, tool.id.to_string());
                                }
                            }
                        }
                        KeyCode::Enter => {
                            state.next_host_substep(h_idx, &current_sub);
                        }
                        KeyCode::Esc => {
                            state.prev_host_substep(h_idx, &current_sub);
                        }
                        _ => {}
                    }
                }

                HostSubStep::GamingAndStreaming => {
                    let total = AVAILABLE_GAMING.len() + 1;
                    match key.code {
                        KeyCode::Up | KeyCode::Char('k') => {
                            if state.selected_index > 0 { state.selected_index -= 1; }
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            if state.selected_index + 1 < total { state.selected_index += 1; }
                        }
                        KeyCode::Char(' ') => {
                            if let Some(host) = state.hosts.get_mut(h_idx) {
                                if state.selected_index < AVAILABLE_GAMING.len() {
                                    let g = &AVAILABLE_GAMING[state.selected_index];
                                    toggle_vec(&mut host.gaming, g.id.to_string());
                                } else {
                                    host.remote_desktop = match host.remote_desktop {
                                        RemoteDesktopRole::None => RemoteDesktopRole::SunshineHost,
                                        RemoteDesktopRole::SunshineHost => RemoteDesktopRole::MoonlightClient,
                                        RemoteDesktopRole::MoonlightClient => RemoteDesktopRole::None,
                                    };
                                }
                            }
                        }
                        KeyCode::Enter => {
                            state.next_host_substep(h_idx, &current_sub);
                        }
                        KeyCode::Esc => {
                            state.prev_host_substep(h_idx, &current_sub);
                        }
                        _ => {}
                    }
                }

                HostSubStep::NetworkingAndServer => {
                    match key.code {
                        KeyCode::Up | KeyCode::Char('k') => {
                            if state.selected_index > 0 { state.selected_index -= 1; }
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            if state.selected_index < 1 { state.selected_index += 1; }
                        }
                        KeyCode::Char(' ') => {
                            if let Some(host) = state.hosts.get_mut(h_idx) {
                                match state.selected_index {
                                    0 => host.tailscale = !host.tailscale,
                                    1 => host.openssh = !host.openssh,
                                    _ => {}
                                }
                            }
                        }
                        KeyCode::Enter => {
                            state.next_host_substep(h_idx, &current_sub);
                        }
                        KeyCode::Esc => {
                            state.prev_host_substep(h_idx, &current_sub);
                        }
                        _ => {}
                    }
                }

                HostSubStep::Maintenance => {
                    match key.code {
                        KeyCode::Up | KeyCode::Char('k') => {
                            if state.selected_index > 0 { state.selected_index -= 1; }
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            if state.selected_index < 1 { state.selected_index += 1; }
                        }
                        KeyCode::Char(' ') => {
                            if let Some(host) = state.hosts.get_mut(h_idx) {
                                match state.selected_index {
                                    0 => host.auto_upgrade = !host.auto_upgrade,
                                    1 => host.restic_backup = !host.restic_backup,
                                    _ => {}
                                }
                            }
                        }
                        KeyCode::Enter => {
                            // Finish this host: generate files and commit
                            let repo_dir = get_orbitos_dir();
                            if let Some(host) = state.hosts.get(h_idx) {
                                let _ = write_host_files(&repo_dir, host);
                                let _ = commit_and_push_step(&format!("configure host {}", host.hostname));
                            }
                            state.next_host_substep(h_idx, &current_sub);
                        }
                        KeyCode::Esc => {
                            state.prev_host_substep(h_idx, &current_sub);
                        }
                        _ => {}
                    }
                }
            }
        }

        Step::FleetPrompt => {
            match key.code {
                KeyCode::Up | KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('k') => {
                    state.selected_index = (state.selected_index + 1) % 2;
                }
                KeyCode::Enter => {
                    if state.selected_index == 0 {
                        state.set_step(Step::AddHostType);
                    } else {
                        state.set_step(Step::Review);
                    }
                }
                _ => {}
            }
        }

        Step::AddHostType => {
            let types = [HostType::Laptop, HostType::PC, HostType::Server, HostType::Portable];
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    state.selected_index = if state.selected_index > 0 { state.selected_index - 1 } else { types.len() - 1 };
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    state.selected_index = (state.selected_index + 1) % types.len();
                }
                KeyCode::Enter => {
                    let chosen = types[state.selected_index];
                    let def_name = match chosen {
                        HostType::Laptop => "laptop-2".to_string(),
                        HostType::PC => "desktop-2".to_string(),
                        HostType::Server => "server".to_string(),
                        HostType::Portable => "portable".to_string(),
                    };
                    state.text_input = def_name;
                    state.set_step(Step::AddHostName { host_type: chosen });
                }
                KeyCode::Esc => {
                    state.set_step(Step::FleetPrompt);
                }
                _ => {}
            }
        }

        Step::Review => {
            match key.code {
                KeyCode::Enter => {
                    state.set_step(Step::DeploymentMenu);
                }
                KeyCode::Esc => {
                    state.set_step(Step::FleetPrompt);
                }
                _ => {}
            }
        }

        Step::DeploymentMenu => {
            let choices = [
                DeploymentChoice::InstallLocal,
                DeploymentChoice::RemoteNixosAnywhere,
                DeploymentChoice::BuildIsoFlashUsb,
                DeploymentChoice::PushGitOnly,
                DeploymentChoice::BailOut,
            ];
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    state.selected_index = if state.selected_index > 0 { state.selected_index - 1 } else { choices.len() - 1 };
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    state.selected_index = (state.selected_index + 1) % choices.len();
                }
                KeyCode::Enter => {
                    let chosen = choices[state.selected_index];
                    if chosen == DeploymentChoice::BailOut {
                        state.selected_index = 1; // Default to Cancel on destructive action
                        state.set_step(Step::BailOutConfirm);
                    } else {
                        state.chosen_deployment = Some(chosen);
                        let _ = commit_and_push_step("final deployment configuration");
                        state.set_step(Step::Done);
                    }
                }
                _ => {}
            }
        }

        Step::BailOutConfirm => {
            match key.code {
                KeyCode::Up | KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('k') => {
                    state.selected_index = (state.selected_index + 1) % 2;
                }
                KeyCode::Enter => {
                    if state.selected_index == 0 {
                        // Confirm bail out
                        let _ = discard_and_bail();
                        state.should_exit = true;
                    } else {
                        // Cancel bail out, return to menu
                        state.set_step(Step::DeploymentMenu);
                    }
                }
                KeyCode::Esc => {
                    state.set_step(Step::DeploymentMenu);
                }
                _ => {}
            }
        }

        Step::Done => {
            state.should_exit = true;
        }

        Step::GithubForkInput | Step::AddAccountName { .. } | Step::AddHostName { .. } => {}
    }
}

fn toggle_vec(vec: &mut Vec<String>, item: String) {
    if let Some(pos) = vec.iter().position(|x| x == &item) {
        vec.remove(pos);
    } else {
        vec.push(item);
    }
}
