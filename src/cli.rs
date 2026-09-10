use std::path::PathBuf;
use clap::{Args, Parser, Subcommand};
use crate::rebuild::{RebuildAction, RebuildOptions};

#[derive(Parser, Debug)]
#[command(
    name = "orbit",
    version,
    about = "Unified system rebuilder, sync, and secret management tool for OrbitOS / NixOS",
    long_about = "Orbit CLI: A fast, standalone Rust rebuilder and system management tool for NixOS.\n\
                  Drop-in replacement for rebuild.nix with support for nh visual diffs, flake updates,\n\
                  Antigravity IDE chat history synchronization, and encrypted SSH key management."
)]
pub struct OrbitCli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Path to flake directory (auto-detected if omitted)
    #[arg(short = 'f', long = "flake", global = true)]
    pub flake: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Rebuild and apply NixOS system configuration
    #[command(name = "rebuild")]
    Rebuild(RebuildArgs),

    /// Build and activate configuration (default)
    #[command(name = "switch")]
    Switch(CommonRebuildArgs),

    /// Build and activate without adding to bootloader menu
    #[command(name = "test")]
    Test(CommonRebuildArgs),

    /// Build and add to bootloader menu without activating now
    #[command(name = "boot")]
    Boot(CommonRebuildArgs),

    /// Build system generation only
    #[command(name = "build")]
    Build(CommonRebuildArgs),

    /// Preview package additions, upgrades, and diffs without switching
    #[command(name = "dry")]
    Dry(CommonRebuildArgs),

    /// Garbage-collect older generations (nh clean all)
    #[command(name = "clean")]
    Clean {
        #[arg(short = 'd', long = "dry")]
        dry: bool,
        #[arg(short = 'a', long = "ask")]
        ask: bool,
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        extra_args: Vec<String>,
    },

    /// Synchronize Antigravity IDE conversation history to state.vscdb
    #[command(name = "sync-chats", alias = "sync")]
    SyncChats,

    /// SSH secret management (backup and restore)
    #[command(name = "ssh")]
    Ssh {
        #[command(subcommand)]
        cmd: SshCommands,
    },

    /// Decrypt secrets/ssh.tar.age into ~/.ssh (interactive passphrase)
    #[command(name = "restore-ssh")]
    RestoreSsh {
        /// Path to encrypted archive (defaults to secrets/ssh.tar.age)
        archive: Option<PathBuf>,
    },

    /// Encrypt ~/.ssh into secrets/ssh.tar.age with an age passphrase
    #[command(name = "backup-ssh")]
    BackupSsh {
        /// Path to destination archive (defaults to secrets/ssh.tar.age)
        archive: Option<PathBuf>,
    },
}

#[derive(Args, Debug, Clone)]
pub struct CommonRebuildArgs {
    /// Target host (defaults to current machine hostname)
    pub host: Option<String>,

    /// Update flake inputs before building
    #[arg(short = 'u', long = "update")]
    pub update: bool,

    /// Preview package diffs without switching
    #[arg(short = 'd', long = "dry")]
    pub dry: bool,

    /// Prompt for confirmation before switching
    #[arg(short = 'a', long = "ask")]
    pub ask: bool,

    /// Show detailed Nix stack traces on evaluation error
    #[arg(long = "show-trace")]
    pub show_trace: bool,

    /// Skip Antigravity IDE chat history synchronization
    #[arg(long = "no-chat-sync")]
    pub no_chat_sync: bool,

    /// Skip interactive SSH key restoration prompt
    #[arg(long = "no-ssh-prompt")]
    pub no_ssh_prompt: bool,

    /// Skip reloading Hyprland
    #[arg(long = "no-hypr-reload")]
    pub no_hypr_reload: bool,

    /// Extra arguments passed to nh
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub extra_args: Vec<String>,
}

#[derive(Args, Debug, Clone)]
pub struct RebuildArgs {
    /// Action: switch, test, boot, build, dry, clean
    pub action: Option<String>,

    /// Target host or options
    pub host: Option<String>,

    /// Update flake inputs before building
    #[arg(short = 'u', long = "update")]
    pub update: bool,

    /// Preview package diffs without switching
    #[arg(short = 'd', long = "dry")]
    pub dry: bool,

    /// Prompt for confirmation before switching
    #[arg(short = 'a', long = "ask")]
    pub ask: bool,

    /// Show detailed Nix stack traces on evaluation error
    #[arg(long = "show-trace")]
    pub show_trace: bool,

    /// Skip Antigravity IDE chat history synchronization
    #[arg(long = "no-chat-sync")]
    pub no_chat_sync: bool,

    /// Skip interactive SSH key restoration prompt
    #[arg(long = "no-ssh-prompt")]
    pub no_ssh_prompt: bool,

    /// Skip reloading Hyprland
    #[arg(long = "no-hypr-reload")]
    pub no_hypr_reload: bool,

    /// Extra arguments passed to nh
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub extra_args: Vec<String>,
}

#[derive(Subcommand, Debug)]
pub enum SshCommands {
    /// Encrypt ~/.ssh into secrets/ssh.tar.age with an age passphrase
    #[command(name = "backup")]
    Backup {
        /// Destination archive path (defaults to secrets/ssh.tar.age)
        archive: Option<PathBuf>,
    },

    /// Decrypt secrets/ssh.tar.age into ~/.ssh (interactive passphrase)
    #[command(name = "restore")]
    Restore {
        /// Source archive path (defaults to secrets/ssh.tar.age)
        archive: Option<PathBuf>,
    },
}

/// Flexible argument parser supporting the legacy rebuild script syntax:
/// Usage: rebuild [ACTION] [HOST] [update|--update|-u] [OPTIONS...]
pub fn parse_flexible_rebuild_args(args: &[String], flake_dir: Option<PathBuf>) -> RebuildOptions {
    let mut action = RebuildAction::Switch;
    let mut host: Option<String> = None;
    let mut update = false;
    let mut dry = false;
    let mut ask = false;
    let mut show_trace = false;
    let mut no_chat_sync = false;
    let mut no_ssh_prompt = false;
    let mut no_hypr_reload = false;
    let mut explicit_flake = flake_dir;
    let mut extra_args: Vec<String> = Vec::new();

    let mut iter = args.iter().peekable();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "switch" => action = RebuildAction::Switch,
            "test" => action = RebuildAction::Test,
            "boot" => action = RebuildAction::Boot,
            "build" => action = RebuildAction::Build,
            "dry" | "--dry" | "-d" => dry = true,
            "clean" => action = RebuildAction::Clean,
            "update" | "--update" | "-u" => update = true,
            "--ask" | "-a" => ask = true,
            "--show-trace" => show_trace = true,
            "--no-chat-sync" => no_chat_sync = true,
            "--no-ssh-prompt" => no_ssh_prompt = true,
            "--no-hypr-reload" => no_hypr_reload = true,
            "-f" | "--flake" => {
                if let Some(val) = iter.next() {
                    explicit_flake = Some(PathBuf::from(val));
                }
            }
            s if s.starts_with("--flake=") => {
                let val = s.trim_start_matches("--flake=");
                explicit_flake = Some(PathBuf::from(val));
            }
            s if s.starts_with('-') => {
                extra_args.push(s.to_string());
            }
            s => {
                if host.is_none() {
                    host = Some(s.to_string());
                } else {
                    extra_args.push(s.to_string());
                }
            }
        }
    }

    if dry && action != RebuildAction::Clean {
        action = RebuildAction::Dry;
    }

    RebuildOptions {
        action,
        host,
        flake_dir: explicit_flake,
        update,
        dry,
        ask,
        show_trace,
        extra_args,
        no_chat_sync,
        no_ssh_prompt,
        no_hypr_reload,
    }
}
