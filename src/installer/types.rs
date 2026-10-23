// OrbitOS — Installer Data Types and Models

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountType {
    User,
    Restricted,
    Anonymous,
}

impl AccountType {
    pub fn label(&self) -> &'static str {
        match self {
            AccountType::User => "User (Primary / Administrator)",
            AccountType::Restricted => "Restricted User (Limited Privileges)",
            AccountType::Anonymous => "Anonymous (Incognito / RAM tmpfs)",
        }
    }

    pub fn short_name(&self) -> &'static str {
        match self {
            AccountType::User => "User",
            AccountType::Restricted => "Restricted",
            AccountType::Anonymous => "Anonymous",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            AccountType::User => "Full administrator account (wheel/sudo, audio, video, networkmanager, docker).",
            AccountType::Restricted => "Isolated user account with restricted group access and limited scope.",
            AccountType::Anonymous => "Ephemeral session mounted entirely in RAM (tmpfs). Automatically wiped on logout.",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub name: String,
    pub account_type: AccountType,
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HostType {
    Laptop,
    PC,
    Server,
    Portable,
}

impl HostType {
    pub fn label(&self) -> &'static str {
        match self {
            HostType::Laptop => "Laptop (Portable Workstation / Battery / Hybrid GPU)",
            HostType::PC => "PC / Desktop (Dedicated Workstation / Gaming Rig)",
            HostType::Server => "Server (Headless / Infrastructure / Docker)",
            HostType::Portable => "Portable (Universal Boot / Live USB drive)",
        }
    }

    pub fn short_name(&self) -> &'static str {
        match self {
            HostType::Laptop => "Laptop",
            HostType::PC => "PC",
            HostType::Server => "Server",
            HostType::Portable => "Portable",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            HostType::Laptop => "Optimized for mobile power management, WiFi, and hybrid graphics switching.",
            HostType::PC => "Optimized for desktop performance, multi-monitor displays, and gaming.",
            HostType::Server => "Headless server running container stacks, reverse proxies, and automated backups.",
            HostType::Portable => "Universal hardware drivers for booting reliably across any unknown machine.",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CpuType {
    Amd,
    Intel,
    Universal,
}

impl CpuType {
    pub fn label(&self) -> &'static str {
        match self {
            CpuType::Amd => "AMD (Ryzen / EPYC / Threadripper)",
            CpuType::Intel => "Intel (Core / Xeon)",
            CpuType::Universal => "Universal CPU (Dynamic fallback detection)",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GpuType {
    Amd,
    NvidiaDesktop,
    NvidiaHybrid,
    NvidiaLegacy,
    IntelIntegrated,
    IntelArc,
    Universal,
    None,
}

impl GpuType {
    pub fn label(&self) -> &'static str {
        match self {
            GpuType::Amd => "AMD Radeon (Open-source RADV / Mesa drivers)",
            GpuType::NvidiaDesktop => "NVIDIA Desktop (Proprietary dGPU driver)",
            GpuType::NvidiaHybrid => "NVIDIA Hybrid Laptop (PRIME offload switching)",
            GpuType::NvidiaLegacy => "NVIDIA Legacy (Older GPU driver)",
            GpuType::IntelIntegrated => "Intel Integrated (Iris Xe / UHD graphics)",
            GpuType::IntelArc => "Intel Arc (Discrete Alchemist/Battlemage GPUs)",
            GpuType::Universal => "Universal GPU (Generic Mesa/Gallium fallback)",
            GpuType::None => "None (Headless / Server display output)",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DesktopShell {
    End4Pc,
    Midnight,
    Dms,
    None,
}

impl DesktopShell {
    pub fn label(&self) -> &'static str {
        match self {
            DesktopShell::End4Pc => "end4-pC (Material 3 Expressive, Agrs widgets, full dots)",
            DesktopShell::Midnight => "Midnight Shell (Caelestia fork, minimal aesthetic)",
            DesktopShell::Dms => "DMS (DankMaterialShell with plugin registry)",
            DesktopShell::None => "None (Bare Hyprland, no default shell)",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RemoteDesktopRole {
    None,
    SunshineHost,
    MoonlightClient,
}

impl RemoteDesktopRole {
    pub fn label(&self) -> &'static str {
        match self {
            RemoteDesktopRole::None => "None (Disabled)",
            RemoteDesktopRole::SunshineHost => "Sunshine Host (GameStream server to stream into)",
            RemoteDesktopRole::MoonlightClient => "Moonlight Client (Receiver to remote into desktops)",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TerminalShell {
    Fish,
    Bash,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeploymentChoice {
    InstallLocal,
    RemoteNixosAnywhere,
    BuildIsoFlashUsb,
    PushGitOnly,
    BailOut,
}

impl DeploymentChoice {
    pub fn label(&self) -> &'static str {
        match self {
            DeploymentChoice::InstallLocal => "Install to Current Machine (apply system rebuild switch)",
            DeploymentChoice::RemoteNixosAnywhere => "Deploy to Remote Fleet Host via SSH (nixos-anywhere)",
            DeploymentChoice::BuildIsoFlashUsb => "Build Bootable Installer ISO & Flash to USB Drive",
            DeploymentChoice::PushGitOnly => "Save & Push to GitHub (leave installation for later)",
            DeploymentChoice::BailOut => "Bail Out & Discard (destructive: exit without pushing)",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostConfig {
    pub hostname: String,
    pub host_type: HostType,
    pub assigned_users: Vec<String>,
    pub cpu: CpuType,
    pub gpu: GpuType,
    pub zram: bool,
    pub des_and_wms: Vec<String>, // "hyprland", "gnome", "plasma"
    pub desktop_shell: Option<DesktopShell>, // active only if "hyprland" is in des_and_wms
    pub terminal_shell: TerminalShell,
    pub starship: bool,
    pub flatpak: bool,
    pub browsers: Vec<String>,
    pub file_managers: Vec<String>,
    pub ides: Vec<String>,
    pub terminals: Vec<String>,
    pub ai_enabled: bool,
    pub ai_tools: Vec<String>,
    pub gaming: Vec<String>,
    pub remote_desktop: RemoteDesktopRole,
    pub tailscale: bool,
    pub openssh: bool,
    pub server_stacks: Vec<String>, // "infra", "cloud", "media", "dev"
    pub auto_upgrade: bool,
    pub restic_backup: bool,
}

impl HostConfig {
    pub fn new(hostname: String, host_type: HostType) -> Self {
        let (cpu, gpu, shell, remote) = match host_type {
            HostType::Laptop => (CpuType::Amd, GpuType::NvidiaHybrid, Some(DesktopShell::End4Pc), RemoteDesktopRole::MoonlightClient),
            HostType::PC => (CpuType::Amd, GpuType::NvidiaDesktop, Some(DesktopShell::End4Pc), RemoteDesktopRole::SunshineHost),
            HostType::Server => (CpuType::Universal, GpuType::None, None, RemoteDesktopRole::None),
            HostType::Portable => (CpuType::Universal, GpuType::Universal, Some(DesktopShell::End4Pc), RemoteDesktopRole::MoonlightClient),
        };

        let des_and_wms = if host_type == HostType::Server {
            Vec::new()
        } else {
            vec!["hyprland".to_string()]
        };

        Self {
            hostname,
            host_type,
            assigned_users: Vec::new(),
            cpu,
            gpu,
            zram: true,
            des_and_wms,
            desktop_shell: shell,
            terminal_shell: TerminalShell::Fish,
            starship: true,
            flatpak: true,
            browsers: vec!["zen".to_string()],
            file_managers: vec!["nautilus".to_string()],
            ides: vec!["vscode".to_string()],
            terminals: vec!["kitty".to_string()],
            ai_enabled: false, // OFF by default
            ai_tools: Vec::new(),
            gaming: if host_type == HostType::Server { Vec::new() } else { vec!["steam".to_string(), "gamemode".to_string()] },
            remote_desktop: remote,
            tailscale: true,
            openssh: true,
            server_stacks: if host_type == HostType::Server { vec!["infra".to_string(), "cloud".to_string()] } else { Vec::new() },
            auto_upgrade: true,
            restic_backup: false,
        }
    }
}
