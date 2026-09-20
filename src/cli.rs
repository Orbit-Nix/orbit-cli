// OrbitOS — Command-line interface definitions and arguments parser

use std::path::PathBuf;
use clap::{Args, Parser, Subcommand};
use crate::rebuild::{RebuildAction, RebuildOptions};

//=========================================#
//               ORBIT CLI                 #
//=========================================#

#[derive(Parser, Debug)]
#[command(
    name = "orbit",
    version,
    about = "Unified system rebuilder, sync, package runner, and secret management tool for OrbitOS / NixOS",
    long_about = "Orbit CLI: Fast, modular system management and rebuilder tool for NixOS.\n\
                  Supports visual diffs, interactive package running (yay-style), flake updates, git synchronization, VM testing, modular desktop shells, and encrypted secrets."
)]
pub struct OrbitCli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Path to flake directory (auto-detected if omitted)
    #[arg(short = 'f', long = "flake", global = true)]
    pub flake: Option<PathBuf>,
}

//=========================================#
//              SUBCOMMANDS                #
//=========================================#

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Rebuild and apply NixOS system configuration with new options
    #[command(name = "rebuild")]
    Rebuild(RebuildArgs),

    /// Search and drop into nix-shell with package(s) installed (yay-style selection)
    #[command(name = "run")]
    Run {
        /// Package name or search query
        query: String,
    },

    /// Search, install, and immediately launch GUI application(s)
    #[command(name = "run-gui")]
    RunGui {
        /// Package name or search query
        query: String,
    },

    /// Switch, list, or inspect OrbitOS modular desktop shells (end4-pC, midnight, dms)
    #[command(name = "shell")]
    Shell {
        /// Target shell name (end4-pC, midnight, dms, none) or action (list, status, restart, autostart)
        shell: Option<String>,
    },

    /// Update flake inputs without rebuilding
    #[command(name = "update")]
    Update,

    /// Build system generation
    #[command(name = "build")]
    Build(CommonRebuildArgs),

    /// Test build without adding to bootloader menu
    #[command(name = "test")]
    Test(CommonRebuildArgs),

    /// Build system generation only
    #[command(name = "build-only")]
    BuildOnly(CommonRebuildArgs),

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

    /// Pull incoming changes from remote git config repository
    #[command(name = "sync")]
    Sync,

    /// Secrets management (store/restore)
    #[command(name = "secrets")]
    Secrets {
        #[command(subcommand)]
        cmd: SecretsCommands,
    },

    /// Install OrbitOS configuration from repository or path
    #[command(name = "install")]
    Install {
        #[arg(long = "config")]
        config: Option<String>,
    },

    /// Play the full Orbit animation easter egg
    #[command(name = "orbit")]
    Orbit,

    /// Play the heart animation easter egg (<3)
    #[command(name = "love", alias = "luv", alias = "heart")]
    Love,

    /// Play the mini animation easter egg (mini-orbit or mini-heart)
    #[command(name = "mini", alias = "mini-orbit", alias = "orbt")]
    Mini {
        /// Variant: "orbit" or "heart" / "love" / "<3" / "luv"
        #[arg(default_value = "orbit")]
        variant: Option<String>,
    },
}

//=========================================#
//            ARGUMENT STRUCTS             #
//=========================================#

#[derive(Args, Debug, Clone)]
pub struct CommonRebuildArgs {
    /// Target host configuration
    pub host: Option<String>,

    /// Update flake inputs before building
    #[arg(short = 'u', long = "update")]
    pub update: bool,

    /// Preview package diffs without switching (dry run)
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
    /// Target host configuration
    pub host: Option<String>,

    /// Update flake inputs before building (-u)
    #[arg(short = 'u', long = "update")]
    pub update: bool,

    /// Preview package diffs without switching (dry run) (-d)
    #[arg(short = 'd', long = "dry")]
    pub dry: bool,

    /// Test build without adding to bootloader (-t)
    #[arg(short = 't', long = "test")]
    pub test: bool,

    /// Build into VM and launch (-v)
    #[arg(short = 'v', long = "vm")]
    pub vm: bool,

    /// Prompt for confirmation before switching (-a)
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

    /// Extra arguments passed to nh / nixos-rebuild
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub extra_args: Vec<String>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum SecretsCommands {
    /// Store secrets into an encrypted archive
    Store {
        /// Secret type: ssh (default), wifi, or all
        #[arg(long = "ssh", default_missing_value = "ssh", num_args = 0..=1)]
        secret_type: Option<String>,

        /// Encryption method: pass (passphrase) or ssh (SSH public key)
        #[arg(long = "encryption", default_value = "pass")]
        encryption: Option<String>,
    },
    /// Restore secrets from an encrypted archive
    Restore {
        /// Secret type: ssh (default), wifi, or all
        #[arg(long = "ssh", default_missing_value = "ssh", num_args = 0..=1)]
        secret_type: Option<String>,

        /// Encryption method: pass or ssh
        #[arg(long = "encryption", default_value = "pass")]
        encryption: Option<String>,

        /// Optional path to the archive
        #[arg(short = 'a', long = "archive")]
        archive: Option<PathBuf>,
    },
}

//=========================================#
//        FLEXIBLE ARGUMENT PARSER         #
//=========================================#

// Flexible argument parser supporting legacy rebuild syntax (e.g. "orbit switch", "orbit update")
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
            "test" | "-t" => action = RebuildAction::Test,
            "build" | "build-only" => action = RebuildAction::Build,
            "vm" | "-v" => action = RebuildAction::Vm,
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
