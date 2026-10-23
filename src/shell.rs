// OrbitOS - Modular Desktop Shell Switcher

use std::fs::{self, File, OpenOptions};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;
use std::fmt;

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
    // Default fallback (matches mySystem.desktop.shell default)
    "illogical-impulse".to_string()
}

pub fn normalize_shell_name(name: &str) -> Option<&'static str> {
    match name.to_lowercase().as_str() {
        // Illogical Impulse family (parent or specific child)
        "illogical-impulse" | "illogical" | "impulse" | "1" => Some("illogical-impulse"),
        "end4-pc" | "end4" | "end-4" => Some("end4-pC"),
        "ii-vynx" | "vynx" | "3" => Some("ii-vynx"),
        // Caelestia family (parent or specific child)
        "caelestia" | "4" => Some("caelestia"),
        "midnight" | "midnight-shell" => Some("midnight"),
        // DMS (no children)
        "dms" | "dankmaterialshell" | "dank-material-shell" | "dank" | "5" => Some("dms"),
        // Noctalia (no children)
        "noctalia" | "6" => Some("noctalia"),
        // Disabled
        "none" | "off" | "disabled" | "7" => Some("none"),
        _ => None,
    }
}

// Kill any related instances
pub fn kill_running_shells() {
    print_info("Stopping existing desktop shell processes...");
    let targets = [
        "qs", "quickshell", ".quickshell-wra", "caelestia-shell", "dms", "dms-greeter", "dgop",
        "cava", "noctalia-shell",
    ];
    for target in targets {
        let _ = Command::new("pkill")
            .arg("-x")
            .arg(target)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
    thread::sleep(Duration::from_millis(300));
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

// Quickshell config directory backing a shell (parents map to their default child)
fn qs_config_for(shell: &str) -> &'static str {
    match shell {
        "illogical-impulse" | "end4-pC" => "end4-pC",
        "ii-vynx" => "ii-vynx",
        "caelestia" | "midnight" => "midnight",
        "noctalia" => "noctalia",
        _ => "",
    }
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

    // Log stuff
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

    // Verify config directory exists if quickshell-backed
    let qs_config = qs_config_for(shell);
    if !qs_config.is_empty() {
        let qs_config_dir = PathBuf::from(&home).join(".config/quickshell").join(qs_config);
        if !qs_config_dir.exists() {
            print_warn(&format!(
                "Configuration directory ~/.config/quickshell/{} not found. You may need to rebuild your NixOS configuration once.",
                qs_config
            ));
        }
    }

    print_info(&format!("Spawning shell: {} (logs: {})", shell, log_path.display()));

    let mut cmd = match shell {
        // Illogical Impulse family: parent and default child share the launcher
        "illogical-impulse" | "end4-pC" => {
            let mut c = Command::new("qs");
            c.arg("-c").arg("end4-pC");
            c
        }
        "ii-vynx" => {
            let mut c = Command::new("qs");
            c.arg("-c").arg("ii-vynx");
            c
        }
        // Caelestia family: native binary first, quickshell fallback
        "caelestia" | "midnight" => {
            if command_exists("caelestia-shell") {
                Command::new("caelestia-shell")
            } else {
                let mut c = Command::new("qs");
                c.arg("-c").arg("midnight");
                c
            }
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
        "noctalia" => {
            if command_exists("noctalia-shell") {
                Command::new("noctalia-shell")
            } else {
                let mut c = Command::new("qs");
                c.arg("-c").arg("noctalia");
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
    println!("  \x1b[1;34m├── Illogical Impulse\x1b[0m (selectable parent or specific child)");
    println!("  \x1b[1;32m│   ├── 1) illogical-impulse\x1b[0m : Default Illogical Impulse suite");
    println!("  \x1b[1;32m│   ├── 2) end4-pC\x1b[0m           : Material 3 Quickshell desktop");
    println!("  \x1b[1;32m│   └── 3) ii-vynx\x1b[0m           : vaguesyntax ii-vynx scrolling-dots fork");
    println!("  \x1b[1;34m├── Caelestia\x1b[0m (selectable parent or specific child)");
    println!("  \x1b[1;32m│   ├── 4) caelestia\x1b[0m         : Default Caelestia shell");
    println!("  \x1b[1;32m│   └── 5) midnight\x1b[0m          : Midnight Shell (dim-ghub/midnight-shell)");
    println!("  \x1b[1;34m├── DMS\x1b[0m (selectable, no childs)");
    println!("  \x1b[1;32m│   └── 6) dms\x1b[0m              : DankMaterialShell (AvengeMedia/DankMaterialShell)");
    println!("  \x1b[1;34m├── Noctalia\x1b[0m (selectable, no childs)");
    println!("  \x1b[1;32m│   └── 7) noctalia\x1b[0m         : Noctalia Shell");
    println!("  \x1b[1;34m└── none\x1b[0m");
    println!("      Bare window manager without desktop shell widgets.");
    println!("\x1b[1;35m===================================================================\x1b[0m\n");
}

pub fn print_shell_status() {
    let current = get_current_shell();
    let path = get_state_file_path();
    println!("\x1b[1;34m[orbit-shell]\x1b[0m Active Runtime Shell: \x1b[1;32m{}\x1b[0m", current);
    println!("\x1b[1;34m[orbit-shell]\x1b[0m State File: {}", path.display());
}

#[derive(Clone)]
struct ShellOption {
    id: &'static str,
    family: &'static str,
    name: &'static str,
    info: &'static str,
}

impl fmt::Display for ShellOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.id == "quit" {
            write!(f, "{:<18} │ \x1b[3m{}\x1b[0m", "quit", self.info)
        } else if self.family.is_empty() {
            write!(f, "{:<18} │ \x1b[2m{}\x1b[0m", self.id, self.info)
        } else {
            write!(
                f,
                "{:<18} │ \x1b[34m{:<16}\x1b[0m \x1b[1m{}\x1b[0m - \x1b[2m{}\x1b[0m",
                self.id, self.family, self.name, self.info
            )
        }
    }
}

pub fn interactive_menu() -> anyhow::Result<()> {
    let current = get_current_shell();
    println!("\n\x1b[1;35mOrbitOS Shell Switcher\x1b[0m (Current: \x1b[1;32m{}\x1b[0m)\n", current);

    // Tree order mirrors modules/desktop/shells.nix: families with selectable
    // parents, followed by their specific children.
    let options = vec![
        ShellOption {
            id: "illogical-impulse",
            family: "Illogical Impulse",
            name: "Default II",
            info: "Default Illogical Impulse suite.",
        },
        ShellOption {
            id: "end4-pC",
            family: "Illogical Impulse",
            name: "Material 3",
            info: "Full widget ecosystem, dynamic sidebar, search & blur.",
        },
        ShellOption {
            id: "ii-vynx",
            family: "Illogical Impulse",
            name: "ii-vynx",
            info: "vaguesyntax ii-vynx scrolling-dots fork.",
        },
        ShellOption {
            id: "caelestia",
            family: "Caelestia",
            name: "Default Caelestia",
            info: "Default Caelestia shell.",
        },
        ShellOption {
            id: "midnight",
            family: "Caelestia",
            name: "Midnight Shell",
            info: "Ultra-clean, minimal QML UI (Caelestia fork).",
        },
        ShellOption {
            id: "dms",
            family: "DMS",
            name: "DankMaterialShell",
            info: "Modern shell with plugin registry, dgop stats & cava.",
        },
        ShellOption {
            id: "noctalia",
            family: "Noctalia",
            name: "Noctalia",
            info: "Noctalia Shell.",
        },
        ShellOption {
            id: "none",
            family: "",
            name: "",
            info: "Bare window manager without desktop shell widgets.",
        },
        ShellOption {
            id: "quit",
            family: "",
            name: "",
            info: "Exit without making changes",
        },
    ];

    let selection = inquire::Select::new("Select a desktop shell to activate:", options)
        .with_help_message("Use Up/Down arrows to select, Enter to confirm, Esc/Ctrl-C to cancel")
        .prompt();

    match selection {
        Ok(choice) => {
            if choice.id == "quit" {
                println!("Exiting without changes.");
                return Ok(());
            }

            if let Some(normalized) = normalize_shell_name(choice.id) {
                kill_running_shells();
                spawn_shell(normalized)?;
            } else {
                print_err(&format!("Invalid selection '{}'", choice.id));
            }
        },
        Err(_) => {
            println!("Exiting without changes.");
        }
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
                        print_err(&format!(
                            "Unknown shell '{}'. Available: illogical-impulse, end4-pC, ii-vynx, caelestia, midnight, dms, noctalia, none",
                            name
                        ));
                        print_shell_list();
                        std::process::exit(1);
                    }
                }
            }
        }
    }
}
