use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use anyhow::{bail, Context, Result};
use crate::util::{get_target_user, print_banner, print_err, run_interactive};

pub fn install_orbit(custom_config: Option<&str>) -> Result<()> {
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
            print_banner(&format!("Cloning config from {} into {}...", cfg, clone_dest.display()));
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
        // Check current directory
        let cwd = std::env::current_dir().context("Failed to get current dir")?;
        let dir_name = cwd.file_name().and_then(|s| s.to_str()).unwrap_or("");

        if cwd.join("flake.nix").exists()
            || dir_name == "orbitos"
            || dir_name == "orbit-config"
            || dir_name == "NixOS"
        {
            print_banner(&format!(
                "Using existing NixOS/Orbit configuration in {}",
                cwd.display()
            ));
            final_config_dir = cwd;
        } else {
            // Clone default repo
            let default_repo = "https://github.com/m-uvex/NixOS.git";
            let clone_dest = if target_orbitos.exists() {
                fallback_orbitos.clone()
            } else {
                target_orbitos.to_path_buf()
            };

            print_banner(&format!(
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

    print_banner(&format!(
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

    // Fallback if orbit executable is not yet in path
    if run_interactive(&mut rebuild_cmd).is_err() {
        print_banner("Running nixos-rebuild switch --flake directly...");
        let mut fallback = Command::new("sudo");
        fallback
            .arg("nixos-rebuild")
            .arg("switch")
            .arg("--flake")
            .arg(&final_config_dir);
        run_interactive(&mut fallback)?;
    }

    print_banner("OrbitOS installation and rebuild complete!");
    Ok(())
}
