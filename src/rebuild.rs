use std::path::{Path, PathBuf};
use std::process::Command;
use anyhow::{Context, Result};
use colored::Colorize;

use crate::chats::sync_chats;
use crate::ssh::{default_ssh_archive, has_ssh_keys, restore_ssh};
use crate::util::{
    clean_stale_home_manager_backups, detect_flake_dir, get_hostname, get_target_user,
    print_banner, print_warn, prompt_confirm, run_interactive,
};

#[derive(Debug, Clone, PartialEq)]
pub enum RebuildAction {
    Switch,
    Test,
    Boot,
    Build,
    Dry,
    Clean,
}

impl RebuildAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            RebuildAction::Switch => "switch",
            RebuildAction::Test => "test",
            RebuildAction::Boot => "boot",
            RebuildAction::Build => "build",
            RebuildAction::Dry => "switch", // nh os switch --dry
            RebuildAction::Clean => "clean",
        }
    }

    pub fn is_dry(&self) -> bool {
        matches!(self, RebuildAction::Dry)
    }

    pub fn is_clean(&self) -> bool {
        matches!(self, RebuildAction::Clean)
    }
}

#[derive(Debug, Clone)]
pub struct RebuildOptions {
    pub action: RebuildAction,
    pub host: Option<String>,
    pub flake_dir: Option<PathBuf>,
    pub update: bool,
    pub dry: bool,
    pub ask: bool,
    pub show_trace: bool,
    pub extra_args: Vec<String>,
    pub no_chat_sync: bool,
    pub no_ssh_prompt: bool,
    pub no_hypr_reload: bool,
}

impl Default for RebuildOptions {
    fn default() -> Self {
        Self {
            action: RebuildAction::Switch,
            host: None,
            flake_dir: None,
            update: false,
            dry: false,
            ask: false,
            show_trace: false,
            extra_args: Vec::new(),
            no_chat_sync: false,
            no_ssh_prompt: false,
            no_hypr_reload: false,
        }
    }
}

pub fn execute_rebuild(opts: RebuildOptions) -> Result<()> {
    let user_info = get_target_user();

    // Clean action
    if opts.action.is_clean() {
        print_banner("Running nh clean all...");
        let mut cmd = Command::new("nh");
        cmd.arg("clean").arg("all");
        if opts.dry {
            cmd.arg("--dry");
        }
        if opts.ask {
            cmd.arg("--ask");
        }
        for arg in &opts.extra_args {
            cmd.arg(arg);
        }
        return run_interactive(&mut cmd);
    }

    let flake_dir = detect_flake_dir(opts.flake_dir.as_deref())?;
    let host = opts.host.unwrap_or_else(get_hostname);

    // Update flake inputs if requested
    if opts.update {
        print_banner(&format!(
            "Updating flake inputs in {}...",
            flake_dir.display()
        ));
        let mut cmd = if user_info.username != "root" && std::env::var("SUDO_USER").is_err() {
            let mut c = Command::new("sudo");
            c.arg("nix")
                .arg("flake")
                .arg("update")
                .arg("--flake")
                .arg(&flake_dir);
            c
        } else {
            let mut c = Command::new("nix");
            c.arg("flake")
                .arg("update")
                .arg("--flake")
                .arg(&flake_dir);
            c
        };
        run_interactive(&mut cmd)?;
    }

    // Clean up stale Home Manager backup files
    clean_stale_home_manager_backups(&user_info.home_dir);

    // Build nh command
    let action_str = opts.action.as_str();
    let display_action = if opts.action.is_dry() || opts.dry {
        "switch (dry-run)"
    } else {
        action_str
    };

    print_banner(&format!(
        "Building and applying configuration ({}) for: {}...",
        display_action,
        host.bold()
    ));

    let mut cmd = Command::new("nh");
    cmd.arg("os")
        .arg(action_str)
        .arg(&flake_dir)
        .arg("-H")
        .arg(&host);

    if opts.action.is_dry() || opts.dry {
        cmd.arg("--dry");
    }
    if opts.ask {
        cmd.arg("--ask");
    }
    if opts.show_trace {
        cmd.arg("--show-trace");
    }
    for arg in &opts.extra_args {
        cmd.arg(arg);
    }

    run_interactive(&mut cmd)?;

    // Post-build hooks on switch or test
    let is_switch_or_test = matches!(opts.action, RebuildAction::Switch | RebuildAction::Test);
    if is_switch_or_test && !opts.dry && !opts.action.is_dry() {
        // 1. Sync Antigravity IDE chats
        if !opts.no_chat_sync {
            if let Err(e) = sync_chats(Some(&user_info.home_dir)) {
                print_warn(&format!("could not sync Antigravity IDE chat history: {}", e));
            }
        }

        // 2. Interactive SSH key restoration check
        if !opts.no_ssh_prompt {
            let secrets_archive = default_ssh_archive(Some(&flake_dir));
            let ssh_dir = user_info.home_dir.join(".ssh");
            if secrets_archive.is_file() && !has_ssh_keys(&ssh_dir) {
                println!();
                print_banner(&format!("No SSH keys detected in {}", ssh_dir.display()));
                print_banner(&format!("Found encrypted backup: {}", secrets_archive.display()));
                if prompt_confirm("==> Would you like to restore ~/.ssh now?", true) {
                    if let Err(e) = restore_ssh(Some(&secrets_archive), Some(&user_info.home_dir)) {
                        print_warn(&format!("SSH restore failed: {}", e));
                    }
                }
            }
        }

        // 3. Reload Hyprland if running
        if !opts.no_hypr_reload {
            reload_hyprland_if_running(&user_info.username, user_info.uid);
        }
    }

    Ok(())
}

fn reload_hyprland_if_running(target_user: &str, target_uid: u32) {
    let hyprland_running = std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok()
        || Command::new("pgrep")
            .arg("-x")
            .arg("Hyprland")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

    if hyprland_running {
        print_banner("Reloading Hyprland configuration...");
        let is_sudo = std::env::var("SUDO_USER").map(|u| u != "root").unwrap_or(false);

        let success = if is_sudo {
            let status = Command::new("sudo")
                .arg("-u")
                .arg(target_user)
                .arg("env")
                .arg(format!("XDG_RUNTIME_DIR=/run/user/{}", target_uid))
                .arg("hyprctl")
                .arg("reload")
                .status();
            status.map(|s| s.success()).unwrap_or(false)
        } else {
            Command::new("hyprctl")
                .arg("reload")
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
        };

        if success {
            print_banner("Hyprland reloaded.");
        } else {
            print_warn("Could not reload Hyprland automatically.");
        }
    }
}
