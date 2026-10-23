// OrbitOS — Git Sync, Fork Management, and Step Persistence Engine

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use anyhow::{bail, Context, Result};
use crate::installer::types::{Account, AccountType, HostConfig, HostType};
use crate::util::get_target_user;

pub fn get_orbitos_dir() -> PathBuf {
    let user_info = get_target_user();
    user_info.home_dir.join("OrbitOS")
}

// Clone user's fork or template into ~/OrbitOS cleanly without leaking to stdout
pub fn clone_fork(fork_url: &str) -> Result<PathBuf> {
    let dest = get_orbitos_dir();

    if dest.join(".git").exists() {
        return Ok(dest);
    }

    if let Some(parent) = dest.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let normalized_url = if fork_url.starts_with("http://")
        || fork_url.starts_with("https://")
        || fork_url.starts_with("git@")
    {
        fork_url.to_string()
    } else {
        let trimmed = fork_url.trim_matches('/');
        if trimmed.contains('/') {
            format!("https://github.com/{}.git", trimmed)
        } else {
            format!("https://github.com/{}/OrbitOS.git", trimmed)
        }
    };

    let output = Command::new("git")
        .arg("clone")
        .arg(&normalized_url)
        .arg(&dest)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .with_context(|| format!("Failed to run git clone for {}", normalized_url))?;

    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to clone repository: {}", err_msg.trim());
    }

    Ok(dest)
}

// Commit and push current progress cleanly
pub fn commit_and_push_step(message: &str) -> Result<()> {
    let repo_dir = get_orbitos_dir();
    if !repo_dir.join(".git").exists() {
        return Ok(());
    }

    // git add .
    let _ = Command::new("git")
        .arg("add")
        .arg(".")
        .current_dir(&repo_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    // git commit -m "orbit: <message>"
    let commit_msg = format!("orbit: {}", message);
    let _ = Command::new("git")
        .arg("commit")
        .arg("-m")
        .arg(&commit_msg)
        .current_dir(&repo_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    // git push origin HEAD
    let _ = Command::new("git")
        .arg("push")
        .arg("origin")
        .arg("HEAD")
        .current_dir(&repo_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    Ok(())
}

// Discard local changes and bail out cleanly
pub fn discard_and_bail() -> Result<()> {
    let repo_dir = get_orbitos_dir();
    if repo_dir.join(".git").exists() {
        let _ = Command::new("git")
            .arg("reset")
            .arg("--hard")
            .arg("HEAD")
            .current_dir(&repo_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output();

        let _ = Command::new("git")
            .arg("clean")
            .arg("-fd")
            .current_dir(&repo_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output();
    }
    Ok(())
}

// Scan ~/OrbitOS to resume existing configured accounts and hosts
pub fn scan_existing_repo() -> Option<(Vec<Account>, Vec<HostConfig>)> {
    let repo_dir = get_orbitos_dir();
    if !repo_dir.join("flake.nix").exists() {
        return None;
    }

    let mut accounts = Vec::new();
    let mut hosts = Vec::new();

    let users_dir = repo_dir.join("users");
    if let Ok(entries) = fs::read_dir(users_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name == "incognito" {
                    accounts.push(Account {
                        name,
                        account_type: AccountType::Anonymous,
                        description: "Incognito RAM-only Session".to_string(),
                    });
                } else if name == "oliver" || name == "dev" {
                    accounts.push(Account {
                        name,
                        account_type: AccountType::Restricted,
                        description: "Restricted User".to_string(),
                    });
                } else {
                    accounts.push(Account {
                        name,
                        account_type: AccountType::User,
                        description: "Primary Administrator".to_string(),
                    });
                }
            }
        }
    }

    let hosts_dir = repo_dir.join("hosts");
    if let Ok(entries) = fs::read_dir(hosts_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                let hostname = entry.file_name().to_string_lossy().to_string();
                let lower = hostname.to_lowercase();
                let host_type = if lower.contains("laptop") || lower.contains("lt") {
                    HostType::Laptop
                } else if lower.contains("server") || lower.contains("srv") {
                    HostType::Server
                } else if lower.contains("portable") {
                    HostType::Portable
                } else {
                    HostType::PC
                };

                let mut config = HostConfig::new(hostname, host_type);
                config.assigned_users = accounts.iter().map(|a| a.name.clone()).collect();
                hosts.push(config);
            }
        }
    }

    if !accounts.is_empty() || !hosts.is_empty() {
        Some((accounts, hosts))
    } else {
        None
    }
}
