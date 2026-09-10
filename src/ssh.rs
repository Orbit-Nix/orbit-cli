use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use anyhow::{Context, Result};
use colored::Colorize;

use crate::util::{get_target_user, print_banner, print_err, set_permissions};

pub fn default_ssh_archive(flake_dir: Option<&Path>) -> PathBuf {
    if let Some(f) = flake_dir {
        let flake_secret = f.join("secrets/ssh.tar.age");
        if flake_secret.exists() {
            return flake_secret;
        }
    }

    let orbitos_archive = Path::new("/orbitos/secrets/ssh.tar.age");
    if orbitos_archive.exists() {
        return orbitos_archive.to_path_buf();
    }

    let nixos_archive = Path::new("/etc/nixos/secrets/ssh.tar.age");
    if nixos_archive.exists() {
        return nixos_archive.to_path_buf();
    }

    if Path::new("/orbitos/secrets").is_dir() {
        return orbitos_archive.to_path_buf();
    }

    nixos_archive.to_path_buf()
}

pub fn has_ssh_keys(ssh_dir: &Path) -> bool {
    if !ssh_dir.is_dir() {
        return false;
    }

    let candidates = ["id_ed25519", "id_rsa", "id_ecdsa", "id_dsa", "m_uvex"];
    for name in &candidates {
        if ssh_dir.join(name).is_file() {
            return true;
        }
    }

    // Check if any non-.pub file exists in .ssh
    if let Ok(entries) = fs::read_dir(ssh_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                    if ext == "pub" {
                        continue;
                    }
                }
                let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
                if !file_name.starts_with("known_hosts") && !file_name.starts_with("config") && !file_name.starts_with("authorized_keys") {
                    return true;
                }
            }
        }
    }

    false
}

pub fn restore_ssh(archive_path: Option<&Path>, custom_home: Option<&Path>) -> Result<()> {
    let user_info = get_target_user();
    let home_dir = custom_home
        .map(|p| p.to_path_buf())
        .unwrap_or(user_info.home_dir);
    let target_user = user_info.username;

    let archive = match archive_path {
        Some(p) => p.to_path_buf(),
        None => default_ssh_archive(None),
    };

    if !archive.is_file() {
        print_err(&format!("encrypted SSH archive not found at: {}", archive.display()));
        anyhow::bail!("Encrypted SSH archive does not exist");
    }

    let ssh_dir = home_dir.join(".ssh");
    print_banner(&format!(
        "Restoring SSH keys for {} into {} from {}...",
        target_user.bold(),
        ssh_dir.display(),
        archive.display()
    ));

    fs::create_dir_all(&ssh_dir)
        .with_context(|| format!("Failed to create .ssh directory at {}", ssh_dir.display()))?;

    // Pipe: age -d "$ARCHIVE" | tar -xz -C "$TARGET_HOME/"
    let mut age_cmd = Command::new("age")
        .arg("-d")
        .arg(&archive)
        .stdin(Stdio::inherit())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .context("Failed to spawn 'age'. Is 'age' installed and in PATH?")?;

    let age_stdout = age_cmd.stdout.take().context("Failed to capture age stdout")?;

    let mut tar_cmd = Command::new("tar")
        .arg("-xz")
        .arg("-C")
        .arg(&home_dir)
        .stdin(age_stdout)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .context("Failed to spawn 'tar'. Is 'tar' installed and in PATH?")?;

    let tar_status = tar_cmd.wait().context("Failed waiting on tar command")?;
    let age_status = age_cmd.wait().context("Failed waiting on age command")?;

    if !age_status.success() || !tar_status.success() {
        print_err("decryption failed or incorrect passphrase.");
        anyhow::bail!("SSH restoration failed during decryption or decompression");
    }

    // Fix permissions: chown -R $TARGET_USER:users "$TARGET_HOME/.ssh"
    let _ = Command::new("chown")
        .arg("-R")
        .arg(format!("{}:users", target_user))
        .arg(&ssh_dir)
        .output();

    let _ = set_permissions(&ssh_dir, 0o700);

    if let Ok(entries) = fs::read_dir(&ssh_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let is_pub = path.extension().and_then(|s| s.to_str()) == Some("pub");
                let mode = if is_pub { 0o644 } else { 0o600 };
                let _ = set_permissions(&path, mode);
            }
        }
    }

    print_banner(&format!(
        "SSH keys successfully restored to {}/",
        ssh_dir.display()
    ));
    Ok(())
}

pub fn backup_ssh(archive_path: Option<&Path>, custom_home: Option<&Path>) -> Result<()> {
    let user_info = get_target_user();
    let home_dir = custom_home
        .map(|p| p.to_path_buf())
        .unwrap_or(user_info.home_dir);
    let target_user = user_info.username;

    let archive = match archive_path {
        Some(p) => p.to_path_buf(),
        None => default_ssh_archive(None),
    };

    let ssh_dir = home_dir.join(".ssh");
    if !ssh_dir.is_dir() {
        print_err(&format!("directory {} does not exist.", ssh_dir.display()));
        anyhow::bail!("SSH directory does not exist");
    }

    print_banner(&format!("Backing up {} to {}...", ssh_dir.display(), archive.display()));
    print_banner("You will be prompted to set an age passphrase:");

    if let Some(parent) = archive.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    // Pipe: tar -cz -C "$TARGET_HOME" .ssh | age -p -o "$ARCHIVE"
    let mut tar_cmd = Command::new("tar")
        .arg("-cz")
        .arg("-C")
        .arg(&home_dir)
        .arg(".ssh")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .context("Failed to spawn 'tar'. Is 'tar' installed and in PATH?")?;

    let tar_stdout = tar_cmd.stdout.take().context("Failed to capture tar stdout")?;

    let mut age_cmd = Command::new("age")
        .arg("-p")
        .arg("-o")
        .arg(&archive)
        .stdin(tar_stdout)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .context("Failed to spawn 'age'. Is 'age' installed and in PATH?")?;

    let age_status = age_cmd.wait().context("Failed waiting on age command")?;
    let tar_status = tar_cmd.wait().context("Failed waiting on tar command")?;

    if !age_status.success() || !tar_status.success() {
        print_err("failed to encrypt or compress SSH directory.");
        anyhow::bail!("SSH backup failed");
    }

    let _ = set_permissions(&archive, 0o644);
    print_banner(&format!("Encrypted SSH archive saved to {}", archive.display()));
    Ok(())
}
