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
            HostType::Laptop => "Laptop (Portable Workstation / Hybrid GPU)",
            HostType::PC => "PC / Desktop (Dedicated Workstation or Gaming Rig)",
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostConfig {
    pub hostname: String,
    pub host_type: HostType,
    pub assigned_users: Vec<String>,
    pub cpu: CpuType,
    pub gpu: GpuType,
    pub swap: bool,
    pub desktop_shell: DesktopShell,
    pub browsers: Vec<String>,
    pub file_managers: Vec<String>,
    pub ides: Vec<String>,
    pub ai_tools: Vec<String>,
    pub terminals: Vec<String>,
    pub gaming: Vec<String>,
    pub extras: Vec<String>,
}

impl HostConfig {
    pub fn new(hostname: String, host_type: HostType) -> Self {
        let (cpu, gpu, shell) = match host_type {
            HostType::Laptop => (CpuType::Amd, GpuType::NvidiaHybrid, DesktopShell::End4Pc),
            HostType::PC => (CpuType::Amd, GpuType::NvidiaDesktop, DesktopShell::End4Pc),
            HostType::Server => (CpuType::Universal, GpuType::None, DesktopShell::None),
            HostType::Portable => (CpuType::Universal, GpuType::Universal, DesktopShell::End4Pc),
        };

        Self {
            hostname,
            host_type,
            assigned_users: Vec::new(),
            cpu,
            gpu,
            swap: true,
            desktop_shell: shell,
            browsers: vec!["zen".to_string()],
            file_managers: vec!["nautilus".to_string()],
            ides: vec!["vscode".to_string()],
            ai_tools: vec!["opencode".to_string()],
            terminals: vec!["kitty".to_string()],
            gaming: vec!["steam".to_string(), "gamemode".to_string()],
            extras: Vec::new(),
        }
    }
}
