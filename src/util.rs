use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use anyhow::{Context, Result};
use colored::Colorize;

pub struct UserInfo {
    pub username: String,
    pub uid: u32,
    pub gid: u32,
    pub home_dir: PathBuf,
}

pub fn get_target_user() -> UserInfo {
    let sudo_user = std::env::var("SUDO_USER").ok().filter(|s| !s.is_empty());
    let user_env = std::env::var("USER").ok().filter(|s| !s.is_empty());
    let target_username = sudo_user.or(user_env).unwrap_or_else(|| "user".to_string());

    // Look up user in /etc/passwd
    if let Ok(passwd_content) = fs::read_to_string("/etc/passwd") {
        for line in passwd_content.lines() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 7 && parts[0] == target_username {
                let uid = parts[2].parse::<u32>().unwrap_or(1000);
                let gid = parts[3].parse::<u32>().unwrap_or(100);
                let home = PathBuf::from(parts[5]);
                return UserInfo {
                    username: target_username,
                    uid,
                    gid,
                    home_dir: home,
                };
            }
        }
    }

    // Fallback: estimate home directory
    let home = if target_username == "root" {
        PathBuf::from("/root")
    } else {
        PathBuf::from(format!("/home/{}", target_username))
    };

    UserInfo {
        username: target_username,
        uid: 1000,
        gid: 100,
        home_dir: home,
    }
}

pub fn get_hostname() -> String {
    if let Ok(hostname) = fs::read_to_string("/etc/hostname") {
        let trimmed = hostname.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }

    if let Ok(output) = Command::new("hostname").output() {
        if output.status.success() {
            let h = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !h.is_empty() {
                return h;
            }
        }
    }

    std::env::var("HOSTNAME").unwrap_or_else(|_| "nixos".to_string())
}

pub fn detect_flake_dir(explicit: Option<&Path>) -> Result<PathBuf> {
    if let Some(path) = explicit {
        if path.exists() {
            return Ok(path.to_path_buf());
        } else {
            anyhow::bail!("Specified flake path does not exist: {}", path.display());
        }
    }

    if let Ok(flake_env) = std::env::var("ORBIT_FLAKE").or_else(|_| std::env::var("FLAKE_DIR")) {
        let p = PathBuf::from(flake_env);
        if p.exists() {
            return Ok(p);
        }
    }

    // Check current directory
    if let Ok(cwd) = std::env::current_dir() {
        if cwd.join("flake.nix").is_file() {
            return Ok(cwd);
        }
        // Walk up parents
        let mut cur = cwd.as_path();
        while let Some(parent) = cur.parent() {
            if parent.join("flake.nix").is_file() {
                return Ok(parent.to_path_buf());
            }
            cur = parent;
        }
    }

    // Standard system paths
    let orbitos_dir = Path::new("/orbitos");
    if orbitos_dir.is_dir() {
        return Ok(orbitos_dir.to_path_buf());
    }

    let nixos_dir = Path::new("/etc/nixos");
    if nixos_dir.is_dir() {
        return Ok(nixos_dir.to_path_buf());
    }

    anyhow::bail!("Could not automatically determine flake directory (checked /orbitos, /etc/nixos, and current directory). Specify via --flake <path>.")
}

pub fn clean_stale_home_manager_backups(home: &Path) {
    let patterns = [
        home.join(".config/gtk-3.0/*.backup"),
        home.join(".config/gtk-4.0/*.backup"),
        home.join(".config/matugen/templates/*/*.backup"),
        home.join(".config/fish/*.backup"),
    ];

    for pat in &patterns {
        if let Some(pat_str) = pat.to_str() {
            if let Ok(paths) = glob::glob(pat_str) {
                for entry in paths.flatten() {
                    let _ = fs::remove_file(entry);
                }
            }
        }
    }
}

pub fn run_interactive(cmd: &mut Command) -> Result<()> {
    cmd.stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    let status = cmd.status().with_context(|| {
        format!("Failed to execute command: {:?}", cmd)
    })?;

    if !status.success() {
        if let Some(code) = status.code() {
            anyhow::bail!("Command failed with exit code: {}", code);
        } else {
            anyhow::bail!("Command terminated by signal");
        }
    }
    Ok(())
}

pub fn prompt_confirm(prompt: &str, default_yes: bool) -> bool {
    use std::io::{self, Write};
    let suffix = if default_yes { "[Y/n]" } else { "[y/N]" };
    print!("{} {} ", prompt.bold(), suffix);
    let _ = io::stdout().flush();

    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() {
        return default_yes;
    }

    let trimmed = input.trim().to_lowercase();
    if trimmed.is_empty() {
        default_yes
    } else {
        trimmed == "y" || trimmed == "yes"
    }
}

pub fn set_permissions(path: &Path, mode: u32) -> Result<()> {
    let metadata = fs::metadata(path)?;
    let mut permissions = metadata.permissions();
    permissions.set_mode(mode);
    fs::set_permissions(path, permissions)?;
    Ok(())
}

pub fn print_banner(msg: &str) {
    println!("{} {}", "==>".bold().green(), msg.bold());
}

pub fn print_warn(msg: &str) {
    eprintln!("{} {}", "warning:".bold().yellow(), msg);
}

pub fn print_err(msg: &str) {
    eprintln!("{} {}", "error:".bold().red(), msg);
}
