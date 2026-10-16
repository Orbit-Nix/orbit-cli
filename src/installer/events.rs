// OrbitOS — Installer Event Handling and Action Dispatcher

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::installer::state::{
    HostSubStep, InstallerState, Step, AVAILABLE_APPS, AVAILABLE_FEATURES,
};
use crate::installer::types::{Account, AccountType, CpuType, DesktopShell, GpuType, HostConfig, HostType};

pub fn handle_key_event(state: &mut InstallerState, key: KeyEvent) {
    // 1. Text input modes take priority for typing
    match &state.current_step {
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
                        state.accounts.push(Account {
                            name,
                            account_type: acc_type,
                            description: desc,
                        });
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
                        // Default assign all user accounts created so far
                        config.assigned_users = state.accounts.iter().map(|a| a.name.clone()).collect();
                        state.hosts.push(config);
                        state.set_step(Step::HostsMenu);
                    }
                }
                KeyCode::Esc => {
                    state.set_step(Step::HostsMenu);
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
        state.should_exit = true;
        return;
    }

    // 3. Navigation and Step Actions
    match &state.current_step {
        Step::Welcome => {
            if key.code == KeyCode::Enter || key.code == KeyCode::Char(' ') {
                state.set_step(Step::AccountsMenu);
            }
        }

        Step::AccountsMenu => {
            let total_items = 1 + state.accounts.len() + if state.accounts.is_empty() { 0 } else { 1 };
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    if state.selected_index > 0 {
                        state.selected_index -= 1;
                    } else {
                        state.selected_index = total_items.saturating_sub(1);
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if state.selected_index + 1 < total_items {
                        state.selected_index += 1;
                    } else {
                        state.selected_index = 0;
                    }
                }
                KeyCode::Enter => {
                    if state.selected_index == 0 {
                        // + Add New Account
                        state.set_step(Step::AddAccountType);
                    } else if !state.accounts.is_empty() && state.selected_index == total_items - 1 {
                        // Continue to Hosts
                        if state.hosts.is_empty() {
                            // Seed default host based on typical machine if empty
                            state.hosts.push(HostConfig::new("laptop".to_string(), HostType::Laptop));
                        }
                        state.set_step(Step::HostsMenu);
                    }
                }
                KeyCode::Char('d') | KeyCode::Delete => {
                    // Delete selected account if on an account item
                    if state.selected_index >= 1 && state.selected_index <= state.accounts.len() {
                        let acc_idx = state.selected_index - 1;
                        state.accounts.remove(acc_idx);
                        state.selected_index = state.selected_index.saturating_sub(1);
                    }
                }
                KeyCode::Esc => {
                    state.set_step(Step::Welcome);
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
                    let chosen_type = types[state.selected_index];
                    let default_name = match chosen_type {
                        AccountType::User => "user".to_string(),
                        AccountType::Restricted => "dev".to_string(),
                        AccountType::Anonymous => "incognito".to_string(),
                    };
                    state.text_input = default_name;
                    state.set_step(Step::AddAccountName { account_type: chosen_type });
                }
                KeyCode::Esc => {
                    state.set_step(Step::AccountsMenu);
                }
                _ => {}
            }
        }

        Step::HostsMenu => {
            let total_items = 1 + state.hosts.len() + if state.hosts.is_empty() { 0 } else { 1 };
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    if state.selected_index > 0 {
                        state.selected_index -= 1;
                    } else {
                        state.selected_index = total_items.saturating_sub(1);
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if state.selected_index + 1 < total_items {
                        state.selected_index += 1;
                    } else {
                        state.selected_index = 0;
                    }
                }
                KeyCode::Enter => {
                    if state.selected_index == 0 {
                        // + Add New Host
                        state.set_step(Step::AddHostType);
                    } else if !state.hosts.is_empty() && state.selected_index == total_items - 1 {
                        // Start per-host configuration wizard
                        state.set_step(Step::HostConfig {
                            host_idx: 0,
                            sub_step: HostSubStep::AssignUsers,
                        });
                    }
                }
                KeyCode::Char('d') | KeyCode::Delete => {
                    // Delete selected host
                    if state.selected_index >= 1 && state.selected_index <= state.hosts.len() {
                        let h_idx = state.selected_index - 1;
                        state.hosts.remove(h_idx);
                        state.selected_index = state.selected_index.saturating_sub(1);
                    }
                }
                KeyCode::Esc => {
                    state.set_step(Step::AccountsMenu);
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
                    let chosen_type = types[state.selected_index];
                    let default_name = match chosen_type {
                        HostType::Laptop => "laptop".to_string(),
                        HostType::PC => "desktop".to_string(),
                        HostType::Server => "server".to_string(),
                        HostType::Portable => "portable".to_string(),
                    };
                    state.text_input = default_name;
                    state.set_step(Step::AddHostName { host_type: chosen_type });
                }
                KeyCode::Esc => {
                    state.set_step(Step::HostsMenu);
                }
                _ => {}
            }
        }

        Step::HostConfig { host_idx, sub_step } => {
            let h_idx = *host_idx;
            let current_sub = sub_step.clone();

            match current_sub {
                HostSubStep::AssignUsers => {
                    let total_users = state.accounts.len();
                    match key.code {
                        KeyCode::Up | KeyCode::Char('k') => {
                            if state.selected_index > 0 { state.selected_index -= 1; }
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            if state.selected_index + 1 < total_users { state.selected_index += 1; }
                        }
                        KeyCode::Char(' ') => {
                            if let Some(acc) = state.accounts.get(state.selected_index) {
                                let acc_name = acc.name.clone();
                                if let Some(host) = state.hosts.get_mut(h_idx) {
                                    if let Some(pos) = host.assigned_users.iter().position(|u| u == &acc_name) {
                                        host.assigned_users.remove(pos);
                                    } else {
                                        host.assigned_users.push(acc_name);
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

                HostSubStep::SelectCpu => {
                    let cpus = [CpuType::Amd, CpuType::Intel, CpuType::Universal];
                    match key.code {
                        KeyCode::Up | KeyCode::Char('k') => {
                            state.selected_index = if state.selected_index > 0 { state.selected_index - 1 } else { cpus.len() - 1 };
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            state.selected_index = (state.selected_index + 1) % cpus.len();
                        }
                        KeyCode::Enter => {
                            if let Some(host) = state.hosts.get_mut(h_idx) {
                                host.cpu = cpus[state.selected_index];
                            }
                            state.next_host_substep(h_idx, &current_sub);
                        }
                        KeyCode::Esc => {
                            state.prev_host_substep(h_idx, &current_sub);
                        }
                        _ => {}
                    }
                }

                HostSubStep::SelectGpu => {
                    let gpus = [
                        GpuType::Amd,
                        GpuType::NvidiaDesktop,
                        GpuType::NvidiaHybrid,
                        GpuType::NvidiaLegacy,
                        GpuType::IntelIntegrated,
                        GpuType::IntelArc,
                        GpuType::Universal,
                        GpuType::None,
                    ];
                    match key.code {
                        KeyCode::Up | KeyCode::Char('k') => {
                            state.selected_index = if state.selected_index > 0 { state.selected_index - 1 } else { gpus.len() - 1 };
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            state.selected_index = (state.selected_index + 1) % gpus.len();
                        }
                        KeyCode::Enter => {
                            if let Some(host) = state.hosts.get_mut(h_idx) {
                                host.gpu = gpus[state.selected_index];
                            }
                            state.next_host_substep(h_idx, &current_sub);
                        }
                        KeyCode::Esc => {
                            state.prev_host_substep(h_idx, &current_sub);
                        }
                        _ => {}
                    }
                }

                HostSubStep::SelectShell => {
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
                                host.desktop_shell = shells[state.selected_index];
                            }
                            state.next_host_substep(h_idx, &current_sub);
                        }
                        KeyCode::Esc => {
                            state.prev_host_substep(h_idx, &current_sub);
                        }
                        _ => {}
                    }
                }

                HostSubStep::SelectApps => {
                    let total_apps = AVAILABLE_APPS.len();
                    match key.code {
                        KeyCode::Up | KeyCode::Char('k') => {
                            if state.selected_index > 0 { state.selected_index -= 1; }
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            if state.selected_index + 1 < total_apps { state.selected_index += 1; }
                        }
                        KeyCode::Char(' ') => {
                            if let Some(app) = AVAILABLE_APPS.get(state.selected_index) {
                                let id = app.id.to_string();
                                if let Some(host) = state.hosts.get_mut(h_idx) {
                                    match app.category {
                                        "Browsers" => toggle_vec(&mut host.browsers, id),
                                        "File Managers" => toggle_vec(&mut host.file_managers, id),
                                        "IDEs" => toggle_vec(&mut host.ides, id),
                                        "AI Tools" => toggle_vec(&mut host.ai_tools, id),
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

                HostSubStep::SelectFeatures => {
                    let total_feats = AVAILABLE_FEATURES.len();
                    match key.code {
                        KeyCode::Up | KeyCode::Char('k') => {
                            if state.selected_index > 0 { state.selected_index -= 1; }
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            if state.selected_index + 1 < total_feats { state.selected_index += 1; }
                        }
                        KeyCode::Char(' ') => {
                            if let Some(feat) = AVAILABLE_FEATURES.get(state.selected_index) {
                                let id = feat.id.to_string();
                                if let Some(host) = state.hosts.get_mut(h_idx) {
                                    match feat.category {
                                        "Gaming" => toggle_vec(&mut host.gaming, id),
                                        _ => toggle_vec(&mut host.extras, id),
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
            }
        }

        Step::Review => {
            match key.code {
                KeyCode::Enter => {
                    state.set_step(Step::Done);
                }
                KeyCode::Esc => {
                    state.set_step(Step::HostsMenu);
                }
                _ => {}
            }
        }

        Step::Done => {
            state.should_exit = true;
        }

        Step::AddAccountName { .. } | Step::AddHostName { .. } => {}
    }
}

fn toggle_vec(vec: &mut Vec<String>, item: String) {
    if let Some(pos) = vec.iter().position(|x| x == &item) {
        vec.remove(pos);
    } else {
        vec.push(item);
    }
}
