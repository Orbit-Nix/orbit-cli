use std::path::Path;
use std::process::Command;
use anyhow::{Context, Result};
use crate::util::{detect_flake_dir, get_target_user, print_banner, run_interactive};

pub fn update_flake(explicit_flake: Option<&Path>) -> Result<()> {
    let flake_dir = detect_flake_dir(explicit_flake)?;
    let user_info = get_target_user();

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

    run_interactive(&mut cmd).context("Failed to update flake inputs")?;
    print_banner("Flake inputs updated successfully.");
    Ok(())
}
