// OrbitOS — Modular Desktop Shell Switcher and Lifecycle Manager

use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

use crate::util::{print_err, print_info, print_success, print_warn};

const STATE_DIR_NAME: &str = ".local/state/orbitos";
const STATE_FILE_NAME: &str = "active_shell";

pub fn get_state_file_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
    PathBuf::from(home).join(STATE_DIR_NAME).join(STATE_FILE_NAME)
}

pub fn get_current_shell() -> String {
    let path = get_state_file_path();
    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            let trimmed = content.trim().to_string();
            if !trimmed.is_empty() {
                return trimmed;
            }
        }
    }
    // Default fallback
    "end4-pC".to_string()
}

pub fn normalize_shell_name(name: &str) -> Option<&'static str> {
    match name.to_lowercase().as_str() {
        "end4-pc" | "end4" | "ii" | "illogical-impulse" | "1" => Some("end4-pC"),
        "midnight" | "midnight-shell" | "caelestia" | "2" => Some("midnight"),
        "dms" | "dankmaterialshell" | "dank-material-shell" | "3" => Some("dms"),
        "none" | "off" | "disabled" | "4" => Some("none"),
        _ => None,
    }
}

pub fn kill_running_shells() {
    print_info("Stopping existing desktop shell processes...");
    let targets = ["qs", "quickshell", ".quickshell-wra", "caelestia-shell", "dms", "dms-greeter", "dgop", "cava"];
    for target in targets {
        let _ = Command::new("pkill")
            .arg("-x")
            .arg(target)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
    thread::sleep(Duration::from_millis(300));

    // Force kill any remaining stubborn instances
    for target in targets {
        let _ = Command::new("pkill")
            .arg("-9")
            .arg("-x")
            .arg(target)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
}

fn command_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub fn spawn_shell(shell: &str) -> anyhow::Result<()> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
    let state_file = get_state_file_path();
    if let Some(parent) = state_file.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(&state_file, shell);

    if shell == "none" {
        print_warn("Shell set to 'none'. Desktop shell widgets disabled.");
        return Ok(());
    }

    // Ensure log directory exists
    let log_dir = PathBuf::from(&home).join(".cache/orbitos");
    let _ = fs::create_dir_all(&log_dir);
    let log_path = log_dir.join(format!("{}.log", shell));

    let log_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&log_path)
        .unwrap_or_else(|_| {
            File::create("/dev/null").expect("Failed to open fallback log destination")
        });

    let err_file = log_file.try_clone().unwrap_or_else(|_| {
        File::create("/dev/null").expect("Failed to open fallback error destination")
    });

    // Verify config directory exists if quickshell config
    let qs_config_dir = PathBuf::from(&home).join(".config/quickshell").join(shell);
    if (shell == "end4-pC" || shell == "midnight") && !qs_config_dir.exists() {
        print_warn(&format!(
            "Configuration directory ~/.config/quickshell/{} not found. You may need to rebuild your NixOS configuration once.",
            shell
        ));
    }

    print_info(&format!("Spawning shell: {} (logs: {})", shell, log_path.display()));

    let mut cmd = match shell {
        "end4-pC" => {
            let mut c = Command::new("qs");
            c.arg("-c").arg("end4-pC");
            c
        }
        "midnight" => {
            let mut c = Command::new("qs");
            c.arg("-c").arg("midnight");
            c
        }
        "dms" => {
            if command_exists("dms") {
                let mut c = Command::new("dms");
                c.arg("run");
                c
            } else {
                let mut c = Command::new("qs");
                c.arg("-c").arg("dms");
                c
            }
        }
        _ => {
            print_err(&format!("Unknown shell '{}'", shell));
            return Ok(());
        }
    };

    cmd.env("qsConfig", shell);
    cmd.stdout(Stdio::from(log_file));
    cmd.stderr(Stdio::from(err_file));
    cmd.stdin(Stdio::null());

    match cmd.spawn() {
        Ok(_) => {
            print_success(&format!("Desktop shell '{}' started successfully!", shell));
        }
        Err(e) => {
            print_err(&format!("Failed to spawn shell '{}': {}", shell, e));
        }
    }

    // Trigger cursor recolor sync in background if available
    let cursor_script = PathBuf::from(home).join(".config/cursor/cursor-material-set-color.sh");
    if cursor_script.exists() {
        let _ = Command::new("bash")
            .arg(cursor_script)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();
    }

    // Send desktop notification if notify-send is present
    let _ = Command::new("notify-send")
        .arg("-a")
        .arg("OrbitOS")
        .arg("-i")
        .arg("preferences-desktop-theme")
        .arg("Desktop Shell Activated")
        .arg(format!("Switched to <b>{}</b> shell.", shell))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    Ok(())
}

pub fn print_shell_list() {
    println!("\n\x1b[1;35m===================================================================\x1b[0m");
    println!("\x1b[1;36m                   OrbitOS Available Desktop Shells\x1b[0m");
    println!("\x1b[1;35m===================================================================\x1b[0m");
    println!("  \x1b[1;32m1) end4-pC\x1b[0m   - Illogical Impulse Material 3 Quickshell suite");
    println!("                 Full widget ecosystem, dynamic sidebar, search & blur.");
    println!("");
    println!("  \x1b[1;32m2) midnight\x1b[0m  - Midnight Shell (dim-ghub/midnight-shell)");
    println!("                 Refined Caelestia fork with ultra-clean, minimal QML UI.");
    println!("");
    println!("  \x1b[1;32m3) dms\x1b[0m       - DankMaterialShell (AvengeMedia/DankMaterialShell)");
    println!("                 Modern shell with plugin registry, dgop stats & cava.");
    println!("");
    println!("  \x1b[1;32m4) none\x1b[0m      - Bare window manager without desktop shell widgets.");
    println!("\x1b[1;35m===================================================================\x1b[0m\n");
}

pub fn print_shell_status() {
    let current = get_current_shell();
    let path = get_state_file_path();
    println!("\x1b[1;34m[orbit-shell]\x1b[0m Active Runtime Shell: \x1b[1;32m{}\x1b[0m", current);
    println!("\x1b[1;34m[orbit-shell]\x1b[0m State File: {}", path.display());
}

pub fn interactive_menu() -> anyhow::Result<()> {
    let current = get_current_shell();
    println!("\n\x1b[1;35mOrbitOS Shell Switcher\x1b[0m (Current: \x1b[1;32m{}\x1b[0m)\n", current);
    println!("  1) end4-pC   (Illogical Impulse Material 3)");
    println!("  2) midnight  (Midnight Shell - Caelestia+)");
    println!("  3) dms       (DankMaterialShell + Plugins)");
    println!("  4) none      (Disable Desktop Shell)");
    println!("  q) Quit");
    print!("\nSelect shell [1-4, q]: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let choice = input.trim();

    if choice.eq_ignore_ascii_case("q") || choice.is_empty() {
        println!("Exiting without changes.");
        return Ok(());
    }

    if let Some(normalized) = normalize_shell_name(choice) {
        kill_running_shells();
        spawn_shell(normalized)?;
    } else {
        print_err(&format!("Invalid selection '{}'", choice));
    }

    Ok(())
}

pub fn handle_shell_command(shell_arg: Option<String>) -> anyhow::Result<()> {
    match shell_arg {
        None => interactive_menu(),
        Some(arg) => {
            let trimmed = arg.trim();
            match trimmed {
                "list" | "ls" | "--list" => {
                    print_shell_list();
                    Ok(())
                }
                "status" | "current" | "--status" => {
                    print_shell_status();
                    Ok(())
                }
                "restart" | "reload" | "--restart" => {
                    let current = get_current_shell();
                    kill_running_shells();
                    spawn_shell(&current)
                }
                "autostart" => {
                    let current = get_current_shell();
                    spawn_shell(&current)
                }
                "menu" => interactive_menu(),
                name => {
                    if let Some(normalized) = normalize_shell_name(name) {
                        kill_running_shells();
                        spawn_shell(normalized)?;
                        Ok(())
                    } else {
                        print_err(&format!("Unknown shell '{}'. Available: end4-pC, midnight, dms, none", name));
                        print_shell_list();
                        std::process::exit(1);
                    }
                }
            }
        }
    }
}
