// OrbitOS — Installer Wizard State Machine

use crate::installer::types::{Account, AccountType, HostConfig, HostType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostSubStep {
    AssignUsers,
    SelectCpu,
    SelectGpu,
    SelectShell,
    SelectApps,
    SelectFeatures,
}

impl HostSubStep {
    pub fn title(&self) -> &'static str {
        match self {
            HostSubStep::AssignUsers => "Assign User Accounts",
            HostSubStep::SelectCpu => "CPU Architecture",
            HostSubStep::SelectGpu => "Graphics / GPU Driver",
            HostSubStep::SelectShell => "Desktop Shell",
            HostSubStep::SelectApps => "Applications",
            HostSubStep::SelectFeatures => "Gaming & Features",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Step {
    Welcome,
    AccountsMenu,
    AddAccountType,
    AddAccountName { account_type: AccountType },
    HostsMenu,
    AddHostType,
    AddHostName { host_type: HostType },
    HostConfig { host_idx: usize, sub_step: HostSubStep },
    Review,
    Done,
}

#[derive(Debug, Clone)]
pub struct AppOption {
    pub id: &'static str,
    pub name: &'static str,
    pub category: &'static str,
}

pub const AVAILABLE_APPS: &[AppOption] = &[
    // Browsers
    AppOption { id: "zen", name: "Zen Browser", category: "Browsers" },
    AppOption { id: "firefox", name: "Firefox", category: "Browsers" },
    AppOption { id: "brave", name: "Brave Browser", category: "Browsers" },
    AppOption { id: "chromium", name: "Chromium", category: "Browsers" },
    AppOption { id: "librewolf", name: "LibreWolf", category: "Browsers" },
    AppOption { id: "tor", name: "Tor Browser", category: "Browsers" },
    // File Managers
    AppOption { id: "nautilus", name: "GNOME Files (Nautilus)", category: "File Managers" },
    AppOption { id: "nemo", name: "Nemo", category: "File Managers" },
    AppOption { id: "thunar", name: "Thunar", category: "File Managers" },
    AppOption { id: "yazi", name: "Yazi (Terminal File Manager)", category: "File Managers" },
    // IDEs
    AppOption { id: "vscode", name: "Visual Studio Code", category: "IDEs" },
    AppOption { id: "vscodium", name: "VSCodium", category: "IDEs" },
    AppOption { id: "zed", name: "Zed Editor", category: "IDEs" },
    AppOption { id: "pycharm", name: "PyCharm Community", category: "IDEs" },
    AppOption { id: "rust-rover", name: "RustRover", category: "IDEs" },
    // AI Tools
    AppOption { id: "opencode", name: "OpenCode CLI & Desktop", category: "AI Tools" },
    AppOption { id: "claude-code", name: "Claude Code", category: "AI Tools" },
    AppOption { id: "antigravity", name: "Antigravity", category: "AI Tools" },
    AppOption { id: "cursor", name: "Cursor Editor", category: "AI Tools" },
    AppOption { id: "ollama", name: "Ollama (Local LLM Server)", category: "AI Tools" },
    // Terminals
    AppOption { id: "kitty", name: "Kitty", category: "Terminals" },
    AppOption { id: "foot", name: "Foot", category: "Terminals" },
    AppOption { id: "alacritty", name: "Alacritty", category: "Terminals" },
    AppOption { id: "ghostty", name: "Ghostty", category: "Terminals" },
];

pub const AVAILABLE_FEATURES: &[AppOption] = &[
    AppOption { id: "steam", name: "Steam + Proton", category: "Gaming" },
    AppOption { id: "heroic", name: "Heroic Games Launcher (GOG/Epic)", category: "Gaming" },
    AppOption { id: "lutris", name: "Lutris", category: "Gaming" },
    AppOption { id: "prismlauncher", name: "Prism Launcher (Minecraft)", category: "Gaming" },
    AppOption { id: "gamemode", name: "Feral GameMode & MangoHud", category: "Gaming" },
    AppOption { id: "moonlight", name: "Moonlight (Remote Desktop Client)", category: "Streaming" },
    AppOption { id: "sunshine", name: "Sunshine (GameStream Server Host)", category: "Streaming" },
    AppOption { id: "docker", name: "Docker Container Runtime", category: "Services" },
    AppOption { id: "zram", name: "ZRAM Swap Compression", category: "Performance" },
];

pub struct InstallerState {
    pub current_step: Step,
    pub accounts: Vec<Account>,
    pub hosts: Vec<HostConfig>,
    pub selected_index: usize,
    pub text_input: String,
    pub should_exit: bool,
    pub status_message: Option<String>,
}

impl Default for InstallerState {
    fn default() -> Self {
        Self::new()
    }
}

impl InstallerState {
    pub fn new() -> Self {
        // Seed default initial user matching typical OrbitOS setups
        let initial_accounts = vec![
            Account {
                name: "user".to_string(),
                account_type: AccountType::User,
                description: "Primary Administrator".to_string(),
            },
        ];

        Self {
            current_step: Step::Welcome,
            accounts: initial_accounts,
            hosts: Vec::new(),
            selected_index: 0,
            text_input: String::new(),
            should_exit: false,
            status_message: None,
        }
    }

    pub fn set_step(&mut self, step: Step) {
        self.current_step = step;
        self.selected_index = 0;
        self.text_input.clear();
        self.status_message = None;
    }

    pub fn current_scope(&self) -> String {
        match &self.current_step {
            Step::Welcome => "WELCOME".to_string(),
            Step::AccountsMenu | Step::AddAccountType | Step::AddAccountName { .. } => "ACCOUNTS".to_string(),
            Step::HostsMenu | Step::AddHostType | Step::AddHostName { .. } => "HOSTS".to_string(),
            Step::HostConfig { host_idx, .. } => {
                if let Some(host) = self.hosts.get(*host_idx) {
                    format!("HOST: {}", host.hostname.to_uppercase())
                } else {
                    "HOST CONFIG".to_string()
                }
            }
            Step::Review => "REVIEW & INSTALL".to_string(),
            Step::Done => "COMPLETE".to_string(),
        }
    }

    pub fn next_host_substep(&mut self, host_idx: usize, current: &HostSubStep) {
        let is_server = self.hosts.get(host_idx).map(|h| h.host_type == HostType::Server).unwrap_or(false);

        let next = match current {
            HostSubStep::AssignUsers => HostSubStep::SelectCpu,
            HostSubStep::SelectCpu => HostSubStep::SelectGpu,
            HostSubStep::SelectGpu => {
                if is_server {
                    HostSubStep::SelectApps
                } else {
                    HostSubStep::SelectShell
                }
            }
            HostSubStep::SelectShell => HostSubStep::SelectApps,
            HostSubStep::SelectApps => HostSubStep::SelectFeatures,
            HostSubStep::SelectFeatures => {
                // Done with this host! Move to next host or Review
                if host_idx + 1 < self.hosts.len() {
                    self.set_step(Step::HostConfig {
                        host_idx: host_idx + 1,
                        sub_step: HostSubStep::AssignUsers,
                    });
                } else {
                    self.set_step(Step::Review);
                }
                return;
            }
        };

        self.set_step(Step::HostConfig {
            host_idx,
            sub_step: next,
        });
    }

    pub fn prev_host_substep(&mut self, host_idx: usize, current: &HostSubStep) {
        let is_server = self.hosts.get(host_idx).map(|h| h.host_type == HostType::Server).unwrap_or(false);

        let prev = match current {
            HostSubStep::AssignUsers => {
                if host_idx > 0 {
                    self.set_step(Step::HostConfig {
                        host_idx: host_idx - 1,
                        sub_step: HostSubStep::SelectFeatures,
                    });
                } else {
                    self.set_step(Step::HostsMenu);
                }
                return;
            }
            HostSubStep::SelectCpu => HostSubStep::AssignUsers,
            HostSubStep::SelectGpu => HostSubStep::SelectCpu,
            HostSubStep::SelectShell => HostSubStep::SelectGpu,
            HostSubStep::SelectApps => {
                if is_server {
                    HostSubStep::SelectGpu
                } else {
                    HostSubStep::SelectShell
                }
            }
            HostSubStep::SelectFeatures => HostSubStep::SelectApps,
        };

        self.set_step(Step::HostConfig {
            host_idx,
            sub_step: prev,
        });
    }
}
