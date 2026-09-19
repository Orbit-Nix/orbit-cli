// OrbitOS — Unified system rebuilder, sync, package runner, and secret management CLI

mod chats;
mod cli;
mod distro;
mod install;
mod proto;
mod rebuild;
mod run;
mod secrets;
mod shell;
mod ssh;
mod sync;
mod update;
mod util;

use clap::Parser;
use cli::{Commands, OrbitCli};
use distro::ensure_nixos;
use rebuild::{execute_rebuild, RebuildOptions};
use util::print_err;

fn main() -> anyhow::Result<()> {
    // --- DISTRO COMPATIBILITY CHECK ---
    if let Err(e) = ensure_nixos() {
        print_err(&format!("{}", e));
        std::process::exit(1);
    }

    let args: Vec<String> = std::env::args().collect();

    // Support legacy rebuild argument style (e.g., "orbit switch", "orbit update", etc.)
    if args.len() > 1 {
        let first_arg = args[1].as_str();
        if first_arg == "shell" {
            let shell_arg = args.get(2).cloned();
            return shell::handle_shell_command(shell_arg);
        }
        let known_actions = ["switch", "test", "boot", "build", "build-only", "dry", "clean"];
        if known_actions.contains(&first_arg) {
            let opts = cli::parse_flexible_rebuild_args(&args[1..], None);
            return execute_rebuild(opts);
        }
    }

    // --- CLI COMMAND DISPATCH ---
    let cli = OrbitCli::parse();

    match cli.command {
        Some(Commands::Rebuild(r_args)) => {
            let mut opts = RebuildOptions {
                action: rebuild::RebuildAction::Switch,
                host: r_args.host,
                flake_dir: cli.flake,
                update: r_args.update,
                dry: r_args.dry,
                ask: r_args.ask,
                show_trace: r_args.show_trace,
                extra_args: r_args.extra_args,
                no_chat_sync: r_args.no_chat_sync,
                no_ssh_prompt: r_args.no_ssh_prompt,
                no_hypr_reload: r_args.no_hypr_reload,
            };

            if r_args.test {
                opts.action = rebuild::RebuildAction::Test;
            } else if r_args.vm {
                opts.action = rebuild::RebuildAction::Vm;
            } else if r_args.dry {
                opts.action = rebuild::RebuildAction::Dry;
            }

            execute_rebuild(opts)?;
        }
        Some(Commands::Shell { shell }) => {
            shell::handle_shell_command(shell)?;
        }
        Some(Commands::Run { query }) => {
            run::handle_run(&query, false)?;
        }
        Some(Commands::RunGui { query }) => {
            run::handle_run(&query, true)?;
        }
        Some(Commands::Update) => {
            update::update_flake(cli.flake.as_deref())?;
        }
        Some(Commands::Build(common)) => {
            let opts = RebuildOptions {
                action: rebuild::RebuildAction::Build,
                host: common.host,
                flake_dir: cli.flake,
                update: common.update,
                dry: common.dry,
                ask: common.ask,
                show_trace: common.show_trace,
                extra_args: common.extra_args,
                no_chat_sync: common.no_chat_sync,
                no_ssh_prompt: common.no_ssh_prompt,
                no_hypr_reload: common.no_hypr_reload,
            };
            execute_rebuild(opts)?;
        }
        Some(Commands::Test(common)) => {
            let opts = RebuildOptions {
                action: rebuild::RebuildAction::Test,
                host: common.host,
                flake_dir: cli.flake,
                update: common.update,
                dry: common.dry,
                ask: common.ask,
                show_trace: common.show_trace,
                extra_args: common.extra_args,
                no_chat_sync: common.no_chat_sync,
                no_ssh_prompt: common.no_ssh_prompt,
                no_hypr_reload: common.no_hypr_reload,
            };
            execute_rebuild(opts)?;
        }
        Some(Commands::BuildOnly(common)) => {
            let opts = RebuildOptions {
                action: rebuild::RebuildAction::Build,
                host: common.host,
                flake_dir: cli.flake,
                update: common.update,
                dry: common.dry,
                ask: common.ask,
                show_trace: common.show_trace,
                extra_args: common.extra_args,
                no_chat_sync: common.no_chat_sync,
                no_ssh_prompt: common.no_ssh_prompt,
                no_hypr_reload: common.no_hypr_reload,
            };
            execute_rebuild(opts)?;
        }
        Some(Commands::Dry(common)) => {
            let opts = RebuildOptions {
                action: rebuild::RebuildAction::Dry,
                host: common.host,
                flake_dir: cli.flake,
                update: common.update,
                dry: true,
                ask: common.ask,
                show_trace: common.show_trace,
                extra_args: common.extra_args,
                no_chat_sync: common.no_chat_sync,
                no_ssh_prompt: common.no_ssh_prompt,
                no_hypr_reload: common.no_hypr_reload,
            };
            execute_rebuild(opts)?;
        }
        Some(Commands::Clean { dry, ask, extra_args }) => {
            let opts = RebuildOptions {
                action: rebuild::RebuildAction::Clean,
                host: None,
                flake_dir: cli.flake,
                update: false,
                dry,
                ask,
                show_trace: false,
                extra_args,
                no_chat_sync: true,
                no_ssh_prompt: true,
                no_hypr_reload: true,
            };
            execute_rebuild(opts)?;
        }
        Some(Commands::Sync) => {
            sync::sync_config(cli.flake.as_deref())?;
        }
        Some(Commands::Secrets { cmd }) => {
            secrets::handle_secrets_command(cmd, cli.flake.as_deref())?;
        }
        Some(Commands::Install { config }) => {
            install::install_orbit(config.as_deref())?;
        }
        None => {
            // Default action: rebuild switch on current host
            let opts = RebuildOptions {
                action: rebuild::RebuildAction::Switch,
                host: None,
                flake_dir: cli.flake,
                update: false,
                dry: false,
                ask: false,
                show_trace: false,
                extra_args: Vec::new(),
                no_chat_sync: false,
                no_ssh_prompt: false,
                no_hypr_reload: false,
            };
            execute_rebuild(opts)?;
        }
    }

    Ok(())
}
