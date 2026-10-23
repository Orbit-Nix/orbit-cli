// OrbitOS — Fresh system installer and repo bootstrapper

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use anyhow::{bail, Context, Result};
use crate::util::{get_target_user, print_step, print_success, run_interactive};

// Bootstrap OrbitOS configuration from a repository or existing local directory
pub fn install_orbit(custom_config: Option<&str>) -> Result<()> {
    if custom_config.is_none() {
        // Launch the interactive architecture and host wizard
        if let Some(state) = crate::installer::run_installer()? {
            print_success(&format!(
                "Installer wizard complete: {} account(s), {} host(s) configured.",
                state.accounts.len(),
                state.hosts.len()
            ));

            let repo_dir = crate::installer::git_sync::get_orbitos_dir();
            let primary_host = state
                .hosts
                .first()
                .map(|h| h.hostname.clone())
                .unwrap_or_else(|| "laptop".to_string());

            if state.bundle_ssh_secrets {
                print_step("Bundling and encrypting SSH fleet identity with orbit secrets...");
                let _ = crate::secrets::store_secret("ssh", "pass", None);
            }

            match state.chosen_deployment {
                Some(crate::installer::types::DeploymentChoice::InstallLocal) => {
                    print_step(&format!(
                        "Applying OrbitOS configuration for host '{}' on local machine...",
                        primary_host
                    ));
                    let mut cmd = Command::new("sudo");
                    cmd.arg("nixos-rebuild")
                        .arg("switch")
                        .arg("--flake")
                        .arg(format!("{}#{}", repo_dir.display(), primary_host));

                    let _ = run_interactive(&mut cmd);
                    print_success("Local OrbitOS system installation complete!");
                }
                Some(crate::installer::types::DeploymentChoice::RemoteNixosAnywhere) => {
                    crate::util::print_info(&format!(
                        "Configuration pushed to GitHub. To deploy to your remote machine via SSH, run:\n  nixos-anywhere --flake {}#{} root@<target-ip>",
                        repo_dir.display(), primary_host
                    ));
                }
                Some(crate::installer::types::DeploymentChoice::BuildIsoFlashUsb) => {
                    crate::util::print_info(&format!(
                        "Configuration pushed to GitHub. To build the bootable installer ISO, run:\n  nix build {}#nixosConfigurations.portable.config.system.build.isoImage",
                        repo_dir.display()
                    ));
                }
                Some(crate::installer::types::DeploymentChoice::PushGitOnly) => {
                    print_success(&format!(
                        "All configurations successfully saved and pushed to ~/OrbitOS ({}).",
                        state.github_fork
                    ));
                }
                Some(crate::installer::types::DeploymentChoice::BailOut) | None => {
                    print_step("Installation cancelled and local state discarded.");
                }
            }

            return Ok(());
        } else {
            print_step("Installation wizard cancelled.");
            return Ok(());
        }
    }

    let user_info = get_target_user();
    let home = &user_info.home_dir;
    let target_orbitos = Path::new("/orbitos");
    let fallback_orbitos = home.join(".cache/orbitos");

    let final_config_dir: PathBuf;

    if let Some(cfg) = custom_config {
        if cfg.starts_with("http://") || cfg.starts_with("https://") || cfg.starts_with("git@") {
            let clone_dest = if target_orbitos.exists() {
                fallback_orbitos.clone()
            } else {
                target_orbitos.to_path_buf()
            };
            print_step(&format!("Cloning config from {} into {}...", cfg, clone_dest.display()));
            let mut clone_cmd = Command::new("git");
            clone_cmd.arg("clone").arg(cfg).arg(&clone_dest);
            run_interactive(&mut clone_cmd)?;
            final_config_dir = clone_dest;
        } else {
            let src_path = Path::new(cfg);
            if !src_path.exists() {
                bail!("Specified config path does not exist: {}", cfg);
            }
            final_config_dir = src_path.to_path_buf();
        }
    } else {
        // Check current working directory
        let cwd = std::env::current_dir().context("Failed to get current dir")?;
        let dir_name = cwd.file_name().and_then(|s| s.to_str()).unwrap_or("");

        if cwd.join("flake.nix").exists()
            || dir_name == "orbitos"
            || dir_name == "orbit-config"
            || dir_name == "NixOS"
        {
            print_step(&format!(
                "Using existing NixOS/Orbit configuration in {}",
                cwd.display()
            ));
            final_config_dir = cwd;
        } else {
            // Clone default repository
            let default_repo = "https://github.com/Orbit-Nix/OrbitOS.git";
            let clone_dest = if target_orbitos.exists() {
                fallback_orbitos.clone()
            } else {
                target_orbitos.to_path_buf()
            };

            print_step(&format!(
                "Cloning default OrbitOS configuration into {}...",
                clone_dest.display()
            ));
            if let Some(parent) = clone_dest.parent() {
                let _ = fs::create_dir_all(parent);
            }

            let mut clone_cmd = Command::new("git");
            clone_cmd.arg("clone").arg(default_repo).arg(&clone_dest);
            run_interactive(&mut clone_cmd)?;
            final_config_dir = clone_dest;
        }
    }

    print_step(&format!(
        "Configuration prepared at {}. Triggering orbit rebuild -u...",
        final_config_dir.display()
    ));

    // Run rebuild with -u
    let mut rebuild_cmd = Command::new("orbit");
    rebuild_cmd
        .arg("rebuild")
        .arg("-u")
        .arg("-f")
        .arg(&final_config_dir);

    // Fallback if orbit executable is not yet in PATH
    if run_interactive(&mut rebuild_cmd).is_err() {
        print_step("Running nixos-rebuild switch --flake directly...");
        let mut fallback = Command::new("sudo");
        fallback
            .arg("nixos-rebuild")
            .arg("switch")
            .arg("--flake")
            .arg(&final_config_dir);
        run_interactive(&mut fallback)?;
    }

    print_success("OrbitOS installation and rebuild complete!");
    Ok(())
}
