// OrbitOS — Declarative NixOS File and Module Generator

use std::fs;
use std::path::Path;
use anyhow::{Context, Result};
use crate::installer::types::{
    Account, AccountType, CpuType, DesktopShell, GpuType, HostConfig, HostType, RemoteDesktopRole,
};

pub fn write_account_files(repo_dir: &Path, account: &Account) -> Result<()> {
    let user_dir = repo_dir.join("users").join(&account.name);
    fs::create_dir_all(&user_dir)
        .with_context(|| format!("Failed to create user directory: {}", user_dir.display()))?;

    let file_path = user_dir.join("default.nix");

    let content = match account.account_type {
        AccountType::User => format!(
            r#"{{ pkgs, lib, ... }}:

{{
  users.users.{name} = {{
    isNormalUser = true;
    description = "{desc}";
    extraGroups = [
      "wheel"
      "networkmanager"
      "video"
      "audio"
      "input"
      "docker"
    ];
    initialPassword = "nixos";
  }};
}}
"#,
            name = account.name,
            desc = account.description,
        ),
        AccountType::Restricted => format!(
            r#"{{ pkgs, lib, ... }}:

{{
  users.users.{name} = {{
    isNormalUser = true;
    description = "{desc}";
    extraGroups = [
      "networkmanager"
      "video"
      "audio"
    ];
    initialPassword = "nixos";
  }};
}}
"#,
            name = account.name,
            desc = account.description,
        ),
        AccountType::Anonymous => format!(
            r#"{{ config, pkgs, ... }}:

{{
  users.users.{name} = {{
    isNormalUser = true;
    description = "Incognito Session";
    extraGroups = [ "networkmanager" "video" "audio" ];
    hashedPassword = ""; # Passwordless login
  }};

  # Mount ~ strictly in RAM (gets wiped completely on logout/reboot)
  fileSystems."/home/{name}" = {{
    device = "tmpfs";
    fsType = "tmpfs";
    options = [
      "mode=0700"
      "size=4G"
    ];
  }};

  security.pam.services.{name} = {{
    text = ''
      session optional pam_exec.so type=close_session log=/tmp/incognito_cleanup.log ${{pkgs.util-linux}}/bin/umount -l /home/{name}
    '';
  }};
}}
"#,
            name = account.name,
        ),
    };

    fs::write(&file_path, content)
        .with_context(|| format!("Failed to write {}", file_path.display()))?;

    Ok(())
}

pub fn write_host_files(repo_dir: &Path, host: &HostConfig) -> Result<()> {
    let host_dir = repo_dir.join("hosts").join(&host.hostname);
    fs::create_dir_all(&host_dir)
        .with_context(|| format!("Failed to create host directory: {}", host_dir.display()))?;

    // 1. Create placeholder hardware-configuration.nix if not present
    let hw_file = host_dir.join("hardware-configuration.nix");
    if !hw_file.exists() {
        let hw_content = r#"{ config, lib, pkgs, modulesPath, ... }:

{
  imports = [ (modulesPath + "/installer/scan/not-detected.nix") ];
  boot.initrd.availableKernelModules = [ "nvme" "xhci_pci" "ahci" "usbhid" "usb_storage" "sd_mod" ];
  fileSystems."/" = {
    device = "/dev/disk/by-label/nixos";
    fsType = "ext4";
  };
  fileSystems."/boot" = {
    device = "/dev/disk/by-label/boot";
    fsType = "vfat";
  };
}
"#;
        let _ = fs::write(&hw_file, hw_content);
    }

    // 2. Build default.nix
    let default_file = host_dir.join("default.nix");

    let cpu_import = match host.cpu {
        CpuType::Amd => "../../modules/hardware/amd-cpu.nix",
        CpuType::Intel => "../../modules/hardware/intel-cpu.nix",
        CpuType::Universal => "../../modules/hardware/universal-cpu.nix",
    };

    let gpu_import = match host.gpu {
        GpuType::Amd => "../../modules/hardware/amd-gpu.nix",
        GpuType::NvidiaDesktop => "../../modules/hardware/nvidia-desktop.nix",
        GpuType::NvidiaHybrid => "../../modules/hardware/nvidia-hybrid.nix",
        GpuType::NvidiaLegacy => "../../modules/hardware/nvidia-legacy.nix",
        GpuType::IntelIntegrated => "../../modules/hardware/intel-integrated.nix",
        GpuType::IntelArc => "../../modules/hardware/intel-arc.nix",
        GpuType::Universal => "../../modules/hardware/universal-gpu.nix",
        GpuType::None => "",
    };

    let shell_name = if host.des_and_wms.contains(&"hyprland".to_string()) {
        match host.desktop_shell {
            Some(DesktopShell::End4Pc) => "\"end4-pC\"",
            Some(DesktopShell::Midnight) => "\"midnight\"",
            Some(DesktopShell::Dms) => "\"dms\"",
            _ => "\"none\"",
        }
    } else {
        "\"none\""
    };

    let mut imports = Vec::new();
    imports.push("./hardware-configuration.nix".to_string());
    imports.push("../../modules/hardware/default.nix".to_string());
    if host.zram {
        imports.push("../../modules/hardware/swap.nix".to_string());
    }
    imports.push(cpu_import.to_string());
    if !gpu_import.is_empty() {
        imports.push(gpu_import.to_string());
    }

    // Users
    for user in &host.assigned_users {
        imports.push(format!("../../users/{}", user));
    }

    // Core & modules
    imports.push("../../modules/core".to_string());
    if host.host_type != HostType::Server {
        imports.push("../../modules/desktop".to_string());
        imports.push("../../modules/desktop/apps.nix".to_string());
        imports.push("../../modules/desktop/gaming.nix".to_string());
    } else {
        imports.push("../../modules/roles/server.nix".to_string());
    }

    match host.remote_desktop {
        RemoteDesktopRole::SunshineHost => {
            imports.push("../../modules/desktop/remote-desktop/host.nix".to_string());
        }
        RemoteDesktopRole::MoonlightClient => {
            imports.push("../../modules/desktop/remote-desktop/client.nix".to_string());
        }
        RemoteDesktopRole::None => {}
    }

    // Format imports
    let imports_str = imports
        .iter()
        .map(|imp| format!("    {}", imp))
        .collect::<Vec<_>>()
        .join("\n");

    let browsers_nix = format_nix_list(&host.browsers);
    let fm_nix = format_nix_list(&host.file_managers);
    let ides_nix = format_nix_list(&host.ides);
    let terminals_nix = format_nix_list(&host.terminals);
    let gaming_nix = format_nix_list(&host.gaming);

    let ai_nix = if host.ai_enabled && !host.ai_tools.is_empty() {
        format!("    ai = {};\n", format_nix_list(&host.ai_tools))
    } else {
        String::new()
    };

    let content = format!(
        r#"# OrbitOS Configuration for {hostname}
# Archetype: {archetype}

{{ pkgs, lib, ... }}:

{{
  imports = [
{imports}
  ];

  networking.hostName = "{hostname}";
  system.stateVersion = "24.05";

  mySystem.desktop.shell = {shell};

  mySystem.apps = {{
    enable = true;
    browsers = {browsers};
    fileManagers = {fm};
    ides = {ides};
    terminals = {terminals};
{ai}  }};

  mySystem.gaming = {{
    enable = {gaming_enable};
    steam = {has_steam};
    games = {gaming_list};
  }};
}}
"#,
        hostname = host.hostname,
        archetype = host.host_type.short_name(),
        imports = imports_str,
        shell = shell_name,
        browsers = browsers_nix,
        fm = fm_nix,
        ides = ides_nix,
        terminals = terminals_nix,
        ai = ai_nix,
        gaming_enable = if host.gaming.is_empty() { "false" } else { "true" },
        has_steam = if host.gaming.contains(&"steam".to_string()) { "true" } else { "false" },
        gaming_list = gaming_nix,
    );

    fs::write(&default_file, content)
        .with_context(|| format!("Failed to write {}", default_file.display()))?;

    Ok(())
}

fn format_nix_list(items: &[String]) -> String {
    if items.is_empty() {
        "[]".to_string()
    } else {
        let inside = items
            .iter()
            .map(|i| format!("\"{}\"", i))
            .collect::<Vec<_>>()
            .join(" ");
        format!("[ {} ]", inside)
    }
}
