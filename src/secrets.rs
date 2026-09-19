use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use anyhow::{bail, Context, Result};
use colored::Colorize;
use crate::cli::SecretsCommands;
use crate::ssh::{default_ssh_archive, has_ssh_keys};
use crate::util::{get_target_user, print_banner, print_err, set_permissions};

pub fn handle_secrets_command(cmd: SecretsCommands, explicit_flake: Option<&Path>) -> Result<()> {
    match cmd {
        SecretsCommands::Store { secret_type, encryption } => {
            let st = secret_type.as_deref().unwrap_or("ssh");
            let enc = encryption.as_deref().unwrap_or("pass");
            store_secret(st, enc, explicit_flake)?;
        }
        SecretsCommands::Restore { secret_type, encryption, archive } => {
            let st = secret_type.as_deref().unwrap_or("ssh");
            let enc = encryption.as_deref().unwrap_or("pass");
            restore_secret(st, enc, archive.as_deref(), explicit_flake)?;
        }
    }
    Ok(())
}

pub fn store_secret(secret_type: &str, encryption: &str, explicit_flake: Option<&Path>) -> Result<()> {
    let user_info = get_target_user();
    let home_dir = &user_info.home_dir;

    match secret_type {
        "ssh" => {
            let archive = default_ssh_archive(explicit_flake);
            let ssh_dir = home_dir.join(".ssh");

            if !ssh_dir.is_dir() {
                print_err(&format!("SSH directory {} does not exist.", ssh_dir.display()));
                bail!("SSH directory does not exist");
            }

            if let Some(parent) = archive.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
            }

            print_banner(&format!(
                "Storing SSH secret archive ({}) to {}...",
                encryption,
                archive.display()
            ));

            let mut tar_cmd = Command::new("tar")
                .arg("-cz")
                .arg("-C")
                .arg(home_dir)
                .arg(".ssh")
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::inherit())
                .spawn()
                .context("Failed to spawn tar")?;

            let tar_stdout = tar_cmd.stdout.take().context("Failed to capture tar stdout")?;

            let mut age_cmd = Command::new("age");
            if encryption == "ssh" {
                // Find SSH public keys or identity to encrypt with
                let pub_key_path = ssh_dir.join("id_ed25519.pub");
                if pub_key_path.exists() {
                    age_cmd.arg("-R").arg(&pub_key_path);
                } else {
                    print_banner("No default id_ed25519.pub found, falling back to passphrase...");
                    age_cmd.arg("-p");
                }
            } else {
                age_cmd.arg("-p");
            }

            age_cmd
                .arg("-o")
                .arg(&archive)
                .stdin(tar_stdout)
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit());

            let mut age_child = age_cmd.spawn().context("Failed to spawn age")?;
            let age_status = age_child.wait().context("Failed waiting for age")?;
            let tar_status = tar_cmd.wait().context("Failed waiting for tar")?;

            if !age_status.success() || !tar_status.success() {
                print_err("Failed to encrypt or store SSH secrets.");
                bail!("Secrets store failed");
            }

            let _ = set_permissions(&archive, 0o644);
            print_banner(&format!("SSH secrets successfully stored at {}", archive.display()));
        }
        "wifi" => {
            print_banner("WiFi secrets support will be available soon.");
        }
        other => {
            print_err(&format!("Unknown secret type '{}'. Available: ssh, wifi", other));
            bail!("Unknown secret type");
        }
    }
    Ok(())
}

pub fn restore_secret(
    secret_type: &str,
    _encryption: &str,
    archive_path: Option<&Path>,
    explicit_flake: Option<&Path>,
) -> Result<()> {
    let user_info = get_target_user();
    let home_dir = &user_info.home_dir;
    let target_user = &user_info.username;

    match secret_type {
        "ssh" => {
            let archive = archive_path
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| default_ssh_archive(explicit_flake));

            if !archive.is_file() {
                print_err(&format!("Encrypted SSH archive not found at: {}", archive.display()));
                bail!("SSH archive does not exist");
            }

            let ssh_dir = home_dir.join(".ssh");
            print_banner(&format!(
                "Restoring SSH secrets for {} into {} from {}...",
                target_user.bold(),
                ssh_dir.display(),
                archive.display()
            ));

            fs::create_dir_all(&ssh_dir)?;

            let mut age_cmd = Command::new("age")
                .arg("-d")
                .arg(&archive)
                .stdin(Stdio::inherit())
                .stdout(Stdio::piped())
                .stderr(Stdio::inherit())
                .spawn()
                .context("Failed to spawn age")?;

            let age_stdout = age_cmd.stdout.take().context("Failed to capture age stdout")?;

            let mut tar_cmd = Command::new("tar")
                .arg("-xz")
                .arg("-C")
                .arg(home_dir)
                .stdin(age_stdout)
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .spawn()
                .context("Failed to spawn tar")?;

            let tar_status = tar_cmd.wait().context("Failed waiting on tar")?;
            let age_status = age_cmd.wait().context("Failed waiting on age")?;

            if !age_status.success() || !tar_status.success() {
                print_err("Decryption failed or incorrect passphrase/key.");
                bail!("Secrets restore failed");
            }

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

            print_banner(&format!("SSH secrets successfully restored to {}/", ssh_dir.display()));
        }
        "wifi" => {
            print_banner("WiFi secrets support will be available soon.");
        }
        other => {
            print_err(&format!("Unknown secret type '{}'. Available: ssh, wifi", other));
            bail!("Unknown secret type");
        }
    }
    Ok(())
}
