// OrbitOS — Installer Wizard State Machine

use crate::installer::orbit_sim::OrbitSimulation;
use crate::installer::probe::DetectedMachine;
use crate::installer::types::{
    Account, AccountType, DeploymentChoice, HostConfig, HostType,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostSubStep {
    Hardware,
    DesktopsAndWms,
    Shell,
    AppsAndTerminal,
    AiTools,
    GamingAndStreaming,
    NetworkingAndServer,
    Maintenance,
}

impl HostSubStep {
    pub fn title(&self) -> &'static str {
        match self {
            HostSubStep::Hardware => "Hardware & Performance",
            HostSubStep::DesktopsAndWms => "Window Managers & Desktops",
            HostSubStep::Shell => "Hyprland Desktop Shell",
            HostSubStep::AppsAndTerminal => "Applications & Shell Defaults",
            HostSubStep::AiTools => "AI Tools (Opt-In)",
            HostSubStep::GamingAndStreaming => "Gaming & Remote Desktop",
            HostSubStep::NetworkingAndServer => "Networking & Server Stacks",
            HostSubStep::Maintenance => "Auto-Upgrades & Backups",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Step {
    Welcome,
    EnvironmentPrompt,
    GithubForkPrompt,
    GithubForkInput,
    AccountsMenu,
    AddAccountType,
    AddAccountName { account_type: AccountType },
    PrimaryHostSetup,
    HostConfig { host_idx: usize, sub_step: HostSubStep },
    FleetPrompt,
    AddHostType,
    AddHostName { host_type: HostType },
    Review,
    DeploymentMenu,
    BailOutConfirm,
    Done,
}

#[derive(Debug, Clone)]
pub struct AppOption {
    pub id: &'static str,
    pub name: &'static str,
    pub category: &'static str,
}

pub const AVAILABLE_DES_WMS: &[AppOption] = &[
    AppOption { id: "hyprland", name: "Hyprland (Dynamic Wayland Compositor)", category: "WM" },
    AppOption { id: "gnome", name: "GNOME Desktop", category: "DE" },
    AppOption { id: "plasma", name: "KDE Plasma 6", category: "DE" },
];

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
    // Terminals
    AppOption { id: "kitty", name: "Kitty", category: "Terminals" },
    AppOption { id: "foot", name: "Foot", category: "Terminals" },
    AppOption { id: "alacritty", name: "Alacritty", category: "Terminals" },
    AppOption { id: "ghostty", name: "Ghostty", category: "Terminals" },
];

pub const AVAILABLE_AI_TOOLS: &[AppOption] = &[
    AppOption { id: "opencode", name: "OpenCode CLI & Desktop", category: "AI CLI/IDE" },
    AppOption { id: "claude-code", name: "Claude Code", category: "AI CLI" },
    AppOption { id: "antigravity", name: "Antigravity", category: "AI Agent" },
    AppOption { id: "cursor", name: "Cursor Editor", category: "AI Editor" },
    AppOption { id: "ollama", name: "Ollama (Local LLM Server)", category: "AI Backend" },
];

pub const AVAILABLE_GAMING: &[AppOption] = &[
    AppOption { id: "steam", name: "Steam + Proton", category: "Gaming" },
    AppOption { id: "heroic", name: "Heroic Games Launcher (GOG/Epic)", category: "Gaming" },
    AppOption { id: "lutris", name: "Lutris", category: "Gaming" },
    AppOption { id: "prismlauncher", name: "Prism Launcher (Minecraft)", category: "Gaming" },
    AppOption { id: "gamemode", name: "Feral GameMode & MangoHud", category: "Gaming" },
];

pub const AVAILABLE_SERVER_STACKS: &[AppOption] = &[
    AppOption { id: "infra", name: "Infrastructure (Caddy/Traefik + Authelia)", category: "Routing" },
    AppOption { id: "cloud", name: "Cloud (PocketBase / PostgreSQL)", category: "Database" },
    AppOption { id: "media", name: "Media Suite (Jellyfin + *arr)", category: "Media" },
    AppOption { id: "dev", name: "Dev Containers & Sandbox", category: "Docker" },
];

pub struct InstallerState {
    pub current_step: Step,
    pub detected: DetectedMachine,
    pub github_fork: String,
    pub accounts: Vec<Account>,
    pub hosts: Vec<HostConfig>,
    pub selected_index: usize,
    pub text_input: String,
    pub should_exit: bool,
    pub chosen_deployment: Option<DeploymentChoice>,
    pub animation_frame: usize,
    pub orbit_sim: OrbitSimulation,
    pub modal_message: Option<(String, String)>,
    pub bundle_ssh_secrets: bool,
}

impl InstallerState {
    pub fn new() -> Self {
        let detected = DetectedMachine::probe();

        // Check if existing repo can be resumed
        let (accounts, hosts) = crate::installer::git_sync::scan_existing_repo()
            .unwrap_or_else(|| {
                (
                    vec![Account {
                        name: "user".to_string(),
                        account_type: AccountType::User,
                        description: "Primary Administrator".to_string(),
                    }],
                    Vec::new(),
                )
            });

        Self {
            current_step: Step::Welcome,
            detected,
            github_fork: "Orbit-Nix/OrbitOS".to_string(),
            accounts,
            hosts,
            selected_index: 0,
            text_input: String::new(),
            should_exit: false,
            chosen_deployment: None,
            animation_frame: 0,
            orbit_sim: OrbitSimulation::new(),
            modal_message: None,
            bundle_ssh_secrets: false,
        }
    }

    pub fn tick_animation(&mut self) {
        self.animation_frame = self.animation_frame.wrapping_add(1);
    }

    pub fn set_step(&mut self, step: Step) {
        self.current_step = step;
        self.selected_index = 0;
        self.text_input.clear();
    }

    pub fn current_scope(&self) -> String {
        match &self.current_step {
            Step::Welcome => "ONBOARDING".to_string(),
            Step::EnvironmentPrompt => "PROBE".to_string(),
            Step::GithubForkPrompt | Step::GithubForkInput => "WORKSPACE".to_string(),
            Step::AccountsMenu | Step::AddAccountType | Step::AddAccountName { .. } => "ACCOUNTS".to_string(),
            Step::PrimaryHostSetup => "PRIMARY HOST".to_string(),
            Step::HostConfig { host_idx, .. } => {
                if let Some(host) = self.hosts.get(*host_idx) {
                    format!("HOST: {}", host.hostname.to_uppercase())
                } else {
                    "HOST CONFIG".to_string()
                }
            }
            Step::FleetPrompt | Step::AddHostType | Step::AddHostName { .. } => "FLEET SETUP".to_string(),
            Step::Review => "REVIEW ARCHITECTURE".to_string(),
            Step::DeploymentMenu => "DEPLOYMENT TARGET".to_string(),
            Step::BailOutConfirm => "CONFIRM BAIL OUT".to_string(),
            Step::Done => "COMPLETE".to_string(),
        }
    }

    pub fn next_host_substep(&mut self, host_idx: usize, current: &HostSubStep) {
        let has_hyprland = self.hosts.get(host_idx)
            .map(|h| h.des_and_wms.contains(&"hyprland".to_string()))
            .unwrap_or(false);

        let next = match current {
            HostSubStep::Hardware => HostSubStep::DesktopsAndWms,
            HostSubStep::DesktopsAndWms => {
                if has_hyprland {
                    HostSubStep::Shell
                } else {
                    HostSubStep::AppsAndTerminal
                }
            }
            HostSubStep::Shell => HostSubStep::AppsAndTerminal,
            HostSubStep::AppsAndTerminal => HostSubStep::AiTools,
            HostSubStep::AiTools => HostSubStep::GamingAndStreaming,
            HostSubStep::GamingAndStreaming => HostSubStep::NetworkingAndServer,
            HostSubStep::NetworkingAndServer => HostSubStep::Maintenance,
            HostSubStep::Maintenance => {
                // Done with this host!
                if host_idx == 0 && self.hosts.len() == 1 {
                    // Finished primary host: prompt if user wants fleet expansion
                    self.set_step(Step::FleetPrompt);
                } else if host_idx + 1 < self.hosts.len() {
                    self.set_step(Step::HostConfig {
                        host_idx: host_idx + 1,
                        sub_step: HostSubStep::Hardware,
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
        let has_hyprland = self.hosts.get(host_idx)
            .map(|h| h.des_and_wms.contains(&"hyprland".to_string()))
            .unwrap_or(false);

        let prev = match current {
            HostSubStep::Hardware => {
                if host_idx > 0 {
                    self.set_step(Step::HostConfig {
                        host_idx: host_idx - 1,
                        sub_step: HostSubStep::Maintenance,
                    });
                } else {
                    self.set_step(Step::AccountsMenu);
                }
                return;
            }
            HostSubStep::DesktopsAndWms => HostSubStep::Hardware,
            HostSubStep::Shell => HostSubStep::DesktopsAndWms,
            HostSubStep::AppsAndTerminal => {
                if has_hyprland {
                    HostSubStep::Shell
                } else {
                    HostSubStep::DesktopsAndWms
                }
            }
            HostSubStep::AiTools => HostSubStep::AppsAndTerminal,
            HostSubStep::GamingAndStreaming => HostSubStep::AiTools,
            HostSubStep::NetworkingAndServer => HostSubStep::GamingAndStreaming,
            HostSubStep::Maintenance => HostSubStep::NetworkingAndServer,
        };

        self.set_step(Step::HostConfig {
            host_idx,
            sub_step: prev,
        });
    }
}
