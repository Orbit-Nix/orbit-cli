use std::io::{self, Write};
use std::path::Path;
use std::process::Command;
use anyhow::{bail, Context, Result};
use colored::Colorize;
use crate::util::{detect_flake_dir, print_banner, print_err, print_warn, run_interactive};

pub fn sync_config(explicit_flake: Option<&Path>) -> Result<()> {
    let config_dir = detect_flake_dir(explicit_flake)?;

    if !config_dir.join(".git").exists() {
        print_err(&format!(
            "Configuration directory {} is not a git repository.",
            config_dir.display()
        ));
        bail!("Git repository not found in config directory");
    }

    print_banner(&format!(
        "Syncing configuration repository at {}...",
        config_dir.display()
    ));

    // Check git status
    let status_output = Command::new("git")
        .arg("-C")
        .arg(&config_dir)
        .arg("status")
        .arg("--porcelain")
        .output()
        .context("Failed to check git status")?;

    let dirty = !status_output.stdout.is_empty();

    if dirty {
        print_warn("Local changes detected in the configuration directory!");
        println!(
            "{}",
            String::from_utf8_lossy(&status_output.stdout).trim()
        );
        println!();
        println!("How would you like to proceed?");
        println!("  1) {}", "Abort sync (preserve local changes unchanged)".bold());
        println!("  2) {}", "Overwrite local changes (reset --hard origin/HEAD)".red().bold());
        println!("  3) {}", "Overwrite remote changes (commit and force push)".yellow().bold());
        print!("Select an option [1/2/3] (default 1): ");
        let _ = io::stdout().flush();

        let mut input = String::new();
        let _ = io::stdin().read_line(&mut input);
        let choice = input.trim();

        match choice {
            "2" => {
                print_banner("Overwriting local changes with remote...");
                let mut fetch_cmd = Command::new("git");
                fetch_cmd.arg("-C").arg(&config_dir).arg("fetch");
                run_interactive(&mut fetch_cmd)?;

                let mut reset_cmd = Command::new("git");
                reset_cmd
                    .arg("-C")
                    .arg(&config_dir)
                    .arg("reset")
                    .arg("--hard")
                    .arg("@{u}");
                run_interactive(&mut reset_cmd)?;
                print_banner("Local changes overwritten successfully.");
                return Ok(());
            }
            "3" => {
                print_banner("Committing and pushing local changes to remote...");
                let mut add_cmd = Command::new("git");
                add_cmd.arg("-C").arg(&config_dir).arg("add").arg("-A");
                run_interactive(&mut add_cmd)?;

                let mut commit_cmd = Command::new("git");
                commit_cmd
                    .arg("-C")
                    .arg(&config_dir)
                    .arg("commit")
                    .arg("-m")
                    .arg("orbit sync: update configuration");
                let _ = commit_cmd.status(); // Ignore if nothing new to commit

                let mut push_cmd = Command::new("git");
                push_cmd
                    .arg("-C")
                    .arg(&config_dir)
                    .arg("push")
                    .arg("--force-with-lease");
                run_interactive(&mut push_cmd)?;
                print_banner("Remote repository updated successfully.");
                return Ok(());
            }
            _ => {
                print_banner("Sync aborted. Local changes were not modified.");
                return Ok(());
            }
        }
    }

    // If clean, do a safe git pull
    print_banner("Pulling incoming changes from remote...");
    let mut pull_cmd = Command::new("git");
    pull_cmd.arg("-C").arg(&config_dir).arg("pull");
    run_interactive(&mut pull_cmd).context("Failed to git pull incoming changes")?;
    print_banner("Configuration synced successfully.");
    Ok(())
}
