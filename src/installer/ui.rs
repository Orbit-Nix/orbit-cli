// OrbitOS — Installer TUI View and Layout Renderer

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::installer::state::{
    HostSubStep, InstallerState, Step, AVAILABLE_AI_TOOLS, AVAILABLE_APPS, AVAILABLE_DES_WMS,
    AVAILABLE_GAMING,
};
use crate::installer::types::{
    AccountType, DeploymentChoice, DesktopShell, HostType, TerminalShell,
};

pub fn render(frame: &mut Frame, state: &InstallerState) {
    let size = frame.area();

    frame.render_widget(Clear, size);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(size);

    render_header(frame, chunks[0]);
    render_main(frame, chunks[1], state);
    render_footer(frame, chunks[2], state);

    if state.current_step == Step::BailOutConfirm {
        render_bail_out_modal(frame, size, state);
    } else if let Some((title, msg)) = &state.modal_message {
        render_info_modal(frame, size, title, msg);
    }
}

fn render_header(frame: &mut Frame, area: Rect) {
    let header_text = vec![Line::from(vec![
        Span::styled(" ORBITOS ", Style::default().fg(Color::Black).bg(Color::Cyan).bold()),
        Span::raw(" "),
        Span::styled("System Architecture & Fleet Installer", Style::default().fg(Color::White).bold()),
        Span::styled("  ~/OrbitOS", Style::default().fg(Color::DarkGray)),
    ])];

    let header_block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(Color::DarkGray))
        .border_type(BorderType::Rounded);

    let paragraph = Paragraph::new(header_text)
        .block(header_block)
        .alignment(Alignment::Left);

    frame.render_widget(paragraph, area);
}

fn render_footer(frame: &mut Frame, area: Rect, state: &InstallerState) {
    let hints = match &state.current_step {
        Step::Welcome => vec![
            Span::styled("[Enter]", Style::default().fg(Color::Green).bold()),
            Span::raw(" Begin Onboarding Setup   "),
            Span::styled("[q]", Style::default().fg(Color::DarkGray)),
            Span::raw(" Quit"),
        ],
        Step::GithubForkInput | Step::AddAccountName { .. } | Step::AddHostName { .. } => vec![
            Span::styled("[Type]", Style::default().fg(Color::Cyan).bold()),
            Span::raw(" Enter Text   "),
            Span::styled("[Enter]", Style::default().fg(Color::Green).bold()),
            Span::raw(" Submit   "),
            Span::styled("[Esc]", Style::default().fg(Color::DarkGray)),
            Span::raw(" Cancel"),
        ],
        Step::HostConfig { sub_step, .. } => match sub_step {
            HostSubStep::DesktopsAndWms
            | HostSubStep::AppsAndTerminal
            | HostSubStep::AiTools
            | HostSubStep::GamingAndStreaming
            | HostSubStep::NetworkingAndServer
            | HostSubStep::Maintenance => vec![
                Span::styled("[↑/↓]", Style::default().fg(Color::Cyan)),
                Span::raw(" Navigate   "),
                Span::styled("[Space]", Style::default().fg(Color::Yellow).bold()),
                Span::raw(" Toggle   "),
                Span::styled("[Enter]", Style::default().fg(Color::Green).bold()),
                Span::raw(" Next Step   "),
                Span::styled("[Esc]", Style::default().fg(Color::DarkGray)),
                Span::raw(" Back"),
            ],
            _ => vec![
                Span::styled("[↑/↓]", Style::default().fg(Color::Cyan)),
                Span::raw(" Select   "),
                Span::styled("[Enter]", Style::default().fg(Color::Green).bold()),
                Span::raw(" Confirm   "),
                Span::styled("[Esc]", Style::default().fg(Color::DarkGray)),
                Span::raw(" Back"),
            ],
        },
        Step::Review => vec![
            Span::styled("[Enter]", Style::default().fg(Color::Green).bold()),
            Span::raw(" Choose Deployment Target   "),
            Span::styled("[Esc]", Style::default().fg(Color::Yellow)),
            Span::raw(" Back to Edit"),
        ],
        Step::DeploymentMenu => vec![
            Span::styled("[↑/↓]", Style::default().fg(Color::Cyan)),
            Span::raw(" Select Action   "),
            Span::styled("[Enter]", Style::default().fg(Color::Green).bold()),
            Span::raw(" Execute"),
        ],
        _ => vec![
            Span::styled("[↑/↓]", Style::default().fg(Color::Cyan)),
            Span::raw(" Navigate   "),
            Span::styled("[Enter]", Style::default().fg(Color::Green).bold()),
            Span::raw(" Select / Continue   "),
            Span::styled("[Esc]", Style::default().fg(Color::DarkGray)),
            Span::raw(" Back   "),
            Span::styled("[q]", Style::default().fg(Color::DarkGray)),
            Span::raw(" Quit"),
        ],
    };

    let footer_block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(Color::DarkGray))
        .border_type(BorderType::Rounded);

    let paragraph = Paragraph::new(Line::from(hints))
        .block(footer_block)
        .alignment(Alignment::Center);

    frame.render_widget(paragraph, area);
}

fn render_main(frame: &mut Frame, area: Rect, state: &InstallerState) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(68), Constraint::Percentage(32)])
        .split(area);

    render_left_pane(frame, cols[0], state);
    render_sidebar(frame, cols[1], state);
}

fn render_sidebar(frame: &mut Frame, area: Rect, state: &InstallerState) {
    let sidebar_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Min(12),
            Constraint::Length(16), // Taller Orbit animation window
        ])
        .split(area);

    let scope_block = Block::default()
        .title(" Current Scope ")
        .title_style(Style::default().fg(Color::Cyan).bold())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan));

    let scope_p = Paragraph::new(Line::from(vec![
        Span::styled("● ", Style::default().fg(Color::Green).bold()),
        Span::styled(state.current_scope(), Style::default().fg(Color::White).bold()),
    ]))
    .block(scope_block)
    .alignment(Alignment::Center);

    frame.render_widget(scope_p, sidebar_chunks[0]);

    // Descriptive Roadmap Box
    let roadmap_block = Block::default()
        .title(" Fleet Roadmap ")
        .title_style(Style::default().fg(Color::DarkGray).bold())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));

    let mut roadmap_items = Vec::new();

    // 1. Onboarding
    let is_welcome = matches!(state.current_step, Step::Welcome);
    let welcome_done = !is_welcome;
    push_timeline_node(
        &mut roadmap_items,
        "Onboarding",
        if is_welcome { "Architecture overview" } else { "" },
        is_welcome,
        welcome_done,
        false,
    );

    // 2. Hardware Probe
    let is_probe = matches!(state.current_step, Step::EnvironmentPrompt);
    let probe_done = !matches!(state.current_step, Step::Welcome | Step::EnvironmentPrompt);
    let probe_info = format!("{}: {:?}", state.detected.hostname, state.detected.host_type);
    push_timeline_node(
        &mut roadmap_items,
        "Hardware Probe",
        if is_probe { "Adopt system identity" } else { &probe_info },
        is_probe,
        probe_done,
        false,
    );

    // 3. Workspace & Git Sync
    let is_ws = matches!(state.current_step, Step::GithubForkPrompt | Step::GithubForkInput);
    let ws_done = !matches!(
        state.current_step,
        Step::Welcome | Step::EnvironmentPrompt | Step::GithubForkPrompt | Step::GithubForkInput
    );
    push_timeline_node(
        &mut roadmap_items,
        "Workspace",
        if is_ws { "Link repository" } else { &state.github_fork },
        is_ws,
        ws_done,
        false,
    );

    // 4. Accounts
    let is_accounts = matches!(
        state.current_step,
        Step::AccountsMenu | Step::AddAccountType | Step::AddAccountName { .. }
    );
    let accounts_done = !state.accounts.is_empty() && !matches!(
        state.current_step,
        Step::Welcome | Step::EnvironmentPrompt | Step::GithubForkPrompt | Step::GithubForkInput | Step::AccountsMenu | Step::AddAccountType | Step::AddAccountName { .. }
    );
    push_timeline_node(
        &mut roadmap_items,
        "Accounts",
        &format!("{} user(s)", state.accounts.len()),
        is_accounts,
        accounts_done,
        state.hosts.is_empty(),
    );

    // 5. Hosts & sub-steps
    for (idx, host) in state.hosts.iter().enumerate() {
        let (is_active_host, active_substep) = match &state.current_step {
            Step::HostConfig { host_idx, sub_step } if *host_idx == idx => (true, Some(sub_step)),
            _ => (false, None),
        };

        let host_completed = match &state.current_step {
            Step::HostConfig { host_idx, .. } => idx < *host_idx,
            Step::FleetPrompt | Step::Review | Step::DeploymentMenu | Step::BailOutConfirm | Step::Done => true,
            _ => false,
        };

        let host_symbol = if is_active_host { "▶" } else if host_completed { "✔" } else { "○" };
        let host_color = if is_active_host { Color::Cyan } else if host_completed { Color::Green } else { Color::DarkGray };

        roadmap_items.push(Line::from(vec![
            Span::styled(format!(" {} ", host_symbol), Style::default().fg(host_color).bold()),
            Span::styled(&host.hostname, Style::default().fg(if is_active_host { Color::White } else { Color::Gray })),
            Span::styled(format!(" [{}]", host.host_type.short_name()), Style::default().fg(Color::DarkGray)),
        ]));

        if is_active_host {
            let substeps = [
                (HostSubStep::Hardware, "Hardware & ZRAM"),
                (HostSubStep::DesktopsAndWms, "WMs & Desktops"),
                (HostSubStep::Shell, "Hyprland Shell"),
                (HostSubStep::AppsAndTerminal, "Apps & Terminal"),
                (HostSubStep::AiTools, "AI Toolchain"),
                (HostSubStep::GamingAndStreaming, "Gaming & Stream"),
                (HostSubStep::NetworkingAndServer, "Mesh Network"),
                (HostSubStep::Maintenance, "Maintenance"),
            ];

            for (ss, label) in substeps {
                let is_current = active_substep == Some(&ss);
                let mark = if is_current { "├──▸ " } else { "├──· " };
                let style = if is_current {
                    Style::default().fg(Color::Yellow).bold()
                } else {
                    Style::default().fg(Color::DarkGray)
                };
                roadmap_items.push(Line::from(vec![
                    Span::styled(" │  ", Style::default().fg(Color::DarkGray)),
                    Span::styled(mark, style),
                    Span::styled(label, style),
                ]));
            }
        }

        roadmap_items.push(Line::from(Span::styled(" │", Style::default().fg(Color::DarkGray))));
    }

    // 6. Review & Deploy
    let is_review = matches!(state.current_step, Step::Review | Step::DeploymentMenu);
    let review_done = matches!(state.current_step, Step::Done);
    push_timeline_node(
        &mut roadmap_items,
        "Deployment",
        if is_review { "Target selection" } else { "" },
        is_review,
        review_done,
        true,
    );

    let roadmap_p = Paragraph::new(roadmap_items)
        .block(roadmap_block)
        .wrap(Wrap { trim: true });

    frame.render_widget(roadmap_p, sidebar_chunks[1]);

    // Bottom-right Orbit animation widget
    render_orbit_animation(frame, sidebar_chunks[2], state);
}

fn push_timeline_node<'a>(
    items: &mut Vec<Line<'a>>,
    title: &str,
    subtitle: &str,
    is_active: bool,
    is_done: bool,
    is_last: bool,
) {
    let (dot, dot_color) = if is_active {
        ("▶", Color::Cyan)
    } else if is_done {
        ("✔", Color::Green)
    } else {
        ("○", Color::DarkGray)
    };

    let title_style = if is_active {
        Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
    } else if is_done {
        Style::default().fg(Color::Gray)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    items.push(Line::from(vec![
        Span::styled(format!(" {} ", dot), Style::default().fg(dot_color).bold()),
        Span::styled(title.to_string(), title_style),
    ]));

    if !subtitle.is_empty() {
        items.push(Line::from(vec![
            Span::styled(" │   ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                subtitle.to_string(),
                Style::default().fg(if is_active { Color::Yellow } else { Color::DarkGray }),
            ),
        ]));
    }

    if !is_last {
        items.push(Line::from(Span::styled(" │", Style::default().fg(Color::DarkGray))));
    }
}

fn render_left_pane(frame: &mut Frame, area: Rect, state: &InstallerState) {
    if state.current_step == Step::Welcome {
        // Expand onboarding description to occupy the whole left side
        render_task_question(frame, area, state);
    } else {
        let left_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8),
                Constraint::Min(8),
            ])
            .split(area);

        render_task_question(frame, left_chunks[0], state);
        render_answer_widget(frame, left_chunks[1], state);
    }
}

fn render_task_question(frame: &mut Frame, area: Rect, state: &InstallerState) {
    let (title, description) = match &state.current_step {
        Step::Welcome => (
            "OrbitOS Architecture & Setup Overview",
            "Welcome to the OrbitOS System Installer.\n\n\
             OrbitOS is a modular, declarative NixOS distribution built for high aesthetic refinement,\n\
             multi-machine cohesion, and instant developer productivity.\n\n\
             KEY ARCHITECTURE PRINCIPLES:\n\
             • Modular Fleet Archetypes: Declare Laptops, Desktops, Servers, and Portable USB boot targets in one tree.\n\
             • Aesthetic Desktop Shells: Dynamic Hyprland shells (end4-pC, Midnight, DankMaterialShell) with seamless switching.\n\
             • Unified Secrets Mesh: Orbit-Secrets automatically binds encrypted identities so all your machines trust each other.\n\
             • Full Reproducibility: Complete declarative manifests for apps, dev tools, and gaming.\n\n\
             WHAT THIS WIZARD WILL CONFIGURE:\n\
             1. Probe running hardware, chassis archetype, and SSH key identity.\n\
             2. Link your GitHub repository in ~/OrbitOS with per-step progress tracking.\n\
             3. Declare user accounts (admin, restricted dev, and RAM-wiped anonymous sessions).\n\
             4. Configure your primary machine (hardware, shells, opt-in AI tools, Sunshine/Moonlight, Tailscale).\n\
             5. Deploy locally, remote over SSH (nixos-anywhere), or generate a bootable USB ISO.\n\n\
             Press [Enter] to begin onboarding setup.",
        ),
        Step::EnvironmentPrompt => (
            "System Hardware & Identity Probe",
            "Orbit-CLI probed your running machine. We can adopt this machine's hostname,\n\
             hardware configuration, and archetype automatically into your fleet.",
        ),
        Step::GithubForkPrompt => (
            "Workspace & Repository Source",
            "OrbitOS is declaratively managed from a git repository cloned to ~/OrbitOS.\n\
             You can use your personal GitHub fork, clone the official template directly, or work locally.",
        ),
        Step::GithubForkInput => (
            "Fork & Repository Linking",
            "HOW TO FORK ORBITOS:\n\
             1. Open https://github.com/Orbit-Nix/OrbitOS/fork in your browser (or run 'gh repo fork Orbit-Nix/OrbitOS').\n\
             2. Keep repository name as 'OrbitOS' and click 'Create fork'.\n\
             3. Enter your GitHub username or repository below to link ~/OrbitOS.",
        ),
        Step::AccountsMenu => (
            "User Accounts Setup",
            "Configure user accounts. You can create standard administrator accounts, restricted\n\
             accounts for isolated workflows, or anonymous sessions mounted in RAM.",
        ),
        Step::AddAccountType => (
            "Account Privilege Type",
            "Choose account privileges: User (full admin), Restricted (confined groups), or Anonymous (RAM-wiped).",
        ),
        Step::AddAccountName { .. } => (
            "Account Username",
            "Type username and press [Enter].",
        ),
        Step::PrimaryHostSetup => (
            "Primary Host Machine",
            "Configure your primary host. Choose its machine archetype and network hostname.",
        ),
        Step::HostConfig { host_idx, sub_step } => {
            let _host = state.hosts.get(*host_idx);
            match sub_step {
                HostSubStep::Hardware => (
                    "Hardware Architecture & Memory",
                    "Configure processor vendor, graphics acceleration, and ZRAM in-memory compressed swap.",
                ),
                HostSubStep::DesktopsAndWms => (
                    "Window Managers & Desktop Environments",
                    "Select which WMs/DEs to install. Hyprland gives access to the Orbit desktop shell switcher.",
                ),
                HostSubStep::Shell => (
                    "Hyprland Desktop Shell",
                    "Select default Hyprland shell. `end4-pC` is the default (Material 3 Expressive dots).",
                ),
                HostSubStep::AppsAndTerminal => (
                    "Applications & Terminal Defaults",
                    "Select standard applications. Choose between Fish shell with Starship prompt or standard Bash.",
                ),
                HostSubStep::AiTools => (
                    "AI Toolchain (Opt-In)",
                    "AI tools are disabled by default. Enable master toggle to select specific CLI agents and backends.",
                ),
                HostSubStep::GamingAndStreaming => (
                    "Gaming & Remote Desktop Pairing",
                    "Configure Steam and gaming tools. Select Remote Desktop role (Sunshine streamer or Moonlight receiver).",
                ),
                HostSubStep::NetworkingAndServer => (
                    "Fleet Networking & Server Stacks",
                    "Tailscale creates an encrypted mesh between all your hosts. Enable server container stacks if applicable.",
                ),
                HostSubStep::Maintenance => (
                    "Automated Maintenance & Snapshots",
                    "Daily background upgrades pull from your GitHub fork. Restic manages encrypted automated snapshots.",
                ),
            }
        }
        Step::FleetPrompt => (
            "Multi-Host Fleet Expansion",
            "Primary host configuration is complete. Do you want to configure additional machines in your fleet?",
        ),
        Step::AddHostType => (
            "Select Machine Archetype",
            "Choose Laptop, PC, Server, or Portable.",
        ),
        Step::AddHostName { .. } => (
            "Hostname",
            "Type hostname for this machine.",
        ),
        Step::Review => (
            "Architecture Review",
            "Review your OrbitOS fleet configuration before committing and selecting deployment.",
        ),
        Step::DeploymentMenu => (
            "Deployment Target",
            "Select how you want to deploy this OrbitOS configuration.",
        ),
        Step::BailOutConfirm => (
            "Bail Out",
            "Confirm discarding local configuration without pushing to GitHub.",
        ),
        Step::Done => (
            "Installation Complete",
            "OrbitOS deployment initiated.",
        ),
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(format!(" Task: {} ", title))
        .title_style(Style::default().fg(Color::Cyan).bold());

    let paragraph = Paragraph::new(description)
        .block(block)
        .style(Style::default().fg(Color::White))
        .wrap(Wrap { trim: true });

    frame.render_widget(paragraph, area);
}

fn render_answer_widget(frame: &mut Frame, area: Rect, state: &InstallerState) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(" Answer / Selection ")
        .title_style(Style::default().fg(Color::Yellow).bold());

    match &state.current_step {
        Step::Welcome => {
            // Handled in full left pane
        }

        Step::EnvironmentPrompt => {
            let d = &state.detected;
            let items = vec![
                ListItem::new(vec![
                    Line::from(vec![
                        Span::styled("● Adopt Current Machine Identity", Style::default().fg(Color::Green).bold()),
                    ]),
                    Line::from(format!("  Hostname: {} | CPU: {:?} | GPU: {:?}", d.hostname, d.cpu, d.gpu)),
                    Line::from(format!("  Archetype: {:?} | Battery: {} | Bundle SSH Keys: Yes", d.host_type, d.has_battery)),
                ]),
                ListItem::new(vec![
                    Line::from(vec![
                        Span::styled("○ Configure Machine Manually", Style::default().fg(Color::White).bold()),
                    ]),
                    Line::from("  Set up hostname, hardware, and archetype from scratch"),
                ]),
            ];
            render_cursor_list(frame, area, block, items, state.selected_index);
        }

        Step::GithubForkPrompt => {
            let items = vec![
                ListItem::new(vec![
                    Line::from(Span::styled("● Clone My GitHub Fork (Recommended)", Style::default().fg(Color::Cyan).bold())),
                    Line::from("  Clones your personal fork into ~/OrbitOS and pushes step-by-step updates"),
                ]),
                ListItem::new(vec![
                    Line::from(Span::styled("○ Use Official OrbitOS Template Directly", Style::default().fg(Color::Green).bold())),
                    Line::from("  Clones Orbit-Nix/OrbitOS directly without requiring an existing fork"),
                ]),
            ];
            render_cursor_list(frame, area, block, items, state.selected_index);
        }

        Step::GithubForkInput => {
            let input_block = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Cyan))
                .title(" GitHub Fork Username or URL ");

            let user_info = crate::util::get_target_user();
            let current_user = user_info.username;
            let example_user = if state.detected.is_live_image || current_user == "nixos" || current_user == "root" {
                "yourname".to_string()
            } else {
                current_user
            };

            let text = vec![
                Line::from(vec![
                    Span::styled(" > https://github.com/", Style::default().fg(Color::DarkGray)),
                    Span::styled(&state.text_input, Style::default().fg(Color::White).bold()),
                    Span::styled("█", Style::default().fg(Color::Yellow)),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Example: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(format!("'{}'", example_user), Style::default().fg(Color::Yellow)),
                    Span::styled(format!(" or '{}/OrbitOS'", example_user), Style::default().fg(Color::Yellow)),
                ]),
                Line::from(Span::styled("Press [Enter] to clone into ~/OrbitOS, [Esc] to go back", Style::default().fg(Color::Cyan))),
            ];

            let paragraph = Paragraph::new(text).block(input_block);
            frame.render_widget(paragraph, area);
        }

        Step::AccountsMenu => {
            let mut items = Vec::new();
            items.push(ListItem::new(Line::from(vec![
                Span::styled("+ Add New Account", Style::default().fg(Color::Cyan).bold()),
            ])));

            for acc in &state.accounts {
                items.push(ListItem::new(Line::from(vec![
                    Span::styled("• ", Style::default().fg(Color::Green)),
                    Span::styled(&acc.name, Style::default().fg(Color::White).bold()),
                    Span::styled(format!(" ({})", acc.account_type.short_name()), Style::default().fg(Color::DarkGray)),
                ])));
            }

            if !state.accounts.is_empty() {
                items.push(ListItem::new(Line::from(vec![
                    Span::styled("✔ Continue to Host Configuration ❯", Style::default().fg(Color::Green).bold()),
                ])));
            }

            render_cursor_list(frame, area, block, items, state.selected_index);
        }

        Step::AddAccountType => {
            let types = [AccountType::User, AccountType::Restricted, AccountType::Anonymous];
            let items: Vec<ListItem> = types
                .iter()
                .map(|t| {
                    ListItem::new(vec![
                        Line::from(Span::styled(t.label(), Style::default().fg(Color::White).bold())),
                        Line::from(Span::styled(format!("  {}", t.description()), Style::default().fg(Color::DarkGray))),
                    ])
                })
                .collect();

            render_cursor_list(frame, area, block, items, state.selected_index);
        }

        Step::AddAccountName { .. } | Step::AddHostName { .. } => {
            let input_block = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Cyan))
                .title(" Name Input ");

            let text = vec![
                Line::from(vec![
                    Span::styled(" > ", Style::default().fg(Color::Cyan).bold()),
                    Span::styled(&state.text_input, Style::default().fg(Color::White).bold()),
                    Span::styled("█", Style::default().fg(Color::Yellow)),
                ]),
                Line::from(""),
                Line::from(Span::styled("Press [Enter] to submit, [Esc] to cancel", Style::default().fg(Color::DarkGray))),
            ];

            let paragraph = Paragraph::new(text).block(input_block);
            frame.render_widget(paragraph, area);
        }

        Step::PrimaryHostSetup => {
            let types = [HostType::Laptop, HostType::PC, HostType::Server, HostType::Portable];
            let items: Vec<ListItem> = types
                .iter()
                .map(|t| {
                    ListItem::new(vec![
                        Line::from(Span::styled(t.label(), Style::default().fg(Color::White).bold())),
                        Line::from(Span::styled(format!("  {}", t.description()), Style::default().fg(Color::DarkGray))),
                    ])
                })
                .collect();

            render_cursor_list(frame, area, block, items, state.selected_index);
        }

        Step::HostConfig { host_idx, sub_step } => {
            let host = match state.hosts.get(*host_idx) {
                Some(h) => h,
                None => return,
            };

            match sub_step {
                HostSubStep::Hardware => {
                    let items = vec![
                        ListItem::new(format!("CPU: {:?}", host.cpu)),
                        ListItem::new(format!("GPU: {:?}", host.gpu)),
                        ListItem::new(format!("ZRAM Compressed Swap: {}", if host.zram { "Enabled" } else { "Disabled" })),
                    ];
                    render_cursor_list(frame, area, block, items, state.selected_index);
                }

                HostSubStep::DesktopsAndWms => {
                    let items: Vec<ListItem> = AVAILABLE_DES_WMS
                        .iter()
                        .map(|wm| {
                            let checked = host.des_and_wms.contains(&wm.id.to_string());
                            let mark = if checked { "[x]" } else { "[ ]" };
                            let color = if checked { Color::Green } else { Color::DarkGray };
                            ListItem::new(Line::from(vec![
                                Span::styled(format!("{} ", mark), Style::default().fg(color).bold()),
                                Span::styled(wm.name, Style::default().fg(Color::White)),
                            ]))
                        })
                        .collect();
                    render_cursor_list(frame, area, block, items, state.selected_index);
                }

                HostSubStep::Shell => {
                    let shells = [DesktopShell::End4Pc, DesktopShell::Midnight, DesktopShell::Dms, DesktopShell::None];
                    let items: Vec<ListItem> = shells
                        .iter()
                        .map(|s| {
                            let selected = host.desktop_shell == Some(*s);
                            let radio = if selected { "(●)" } else { "( )" };
                            ListItem::new(Line::from(vec![
                                Span::styled(format!("{} ", radio), Style::default().fg(if selected { Color::Cyan } else { Color::DarkGray }).bold()),
                                Span::styled(s.label(), Style::default().fg(Color::White)),
                            ]))
                        })
                        .collect();
                    render_cursor_list(frame, area, block, items, state.selected_index);
                }

                HostSubStep::AppsAndTerminal => {
                    let mut items = Vec::new();
                    let fish_active = host.terminal_shell == TerminalShell::Fish;
                    items.push(ListItem::new(Line::from(vec![
                        Span::styled(format!("[{}] ", if fish_active { "x" } else { " " }), Style::default().fg(Color::Green).bold()),
                        Span::styled("Fish Shell + Starship Prompt (Modern & Fast)", Style::default().fg(Color::White)),
                    ])));

                    let flatpak_active = host.flatpak;
                    items.push(ListItem::new(Line::from(vec![
                        Span::styled(format!("[{}] ", if flatpak_active { "x" } else { " " }), Style::default().fg(Color::Green).bold()),
                        Span::styled("Flatpak Runtime (Declarative sandbox applications)", Style::default().fg(Color::White)),
                    ])));

                    for app in AVAILABLE_APPS {
                        let checked = host.browsers.contains(&app.id.to_string())
                            || host.file_managers.contains(&app.id.to_string())
                            || host.ides.contains(&app.id.to_string())
                            || host.terminals.contains(&app.id.to_string());
                        let mark = if checked { "[x]" } else { "[ ]" };
                        let color = if checked { Color::Green } else { Color::DarkGray };
                        items.push(ListItem::new(Line::from(vec![
                            Span::styled(format!("{} ", mark), Style::default().fg(color).bold()),
                            Span::styled(app.name, Style::default().fg(Color::White)),
                            Span::styled(format!(" [{}]", app.category), Style::default().fg(Color::DarkGray)),
                        ])));
                    }

                    render_cursor_list(frame, area, block, items, state.selected_index);
                }

                HostSubStep::AiTools => {
                    let mut items = Vec::new();
                    let ai_master = host.ai_enabled;
                    items.push(ListItem::new(Line::from(vec![
                        Span::styled(format!("[{}] ", if ai_master { "x" } else { " " }), Style::default().fg(if ai_master { Color::Green } else { Color::Yellow }).bold()),
                        Span::styled("Master Toggle: Enable AI Tools & Agent Runtimes", Style::default().fg(Color::White).bold()),
                        Span::styled(if ai_master { " (ACTIVE)" } else { " (OFF BY DEFAULT)" }, Style::default().fg(Color::DarkGray)),
                    ])));

                    if ai_master {
                        for tool in AVAILABLE_AI_TOOLS {
                            let checked = host.ai_tools.contains(&tool.id.to_string());
                            let mark = if checked { "[x]" } else { "[ ]" };
                            let color = if checked { Color::Green } else { Color::DarkGray };
                            items.push(ListItem::new(Line::from(vec![
                                Span::raw("   "),
                                Span::styled(format!("{} ", mark), Style::default().fg(color).bold()),
                                Span::styled(tool.name, Style::default().fg(Color::White)),
                                Span::styled(format!(" [{}]", tool.category), Style::default().fg(Color::DarkGray)),
                            ])));
                        }
                    }

                    render_cursor_list(frame, area, block, items, state.selected_index);
                }

                HostSubStep::GamingAndStreaming => {
                    let mut items = Vec::new();
                    for g in AVAILABLE_GAMING {
                        let checked = host.gaming.contains(&g.id.to_string());
                        let mark = if checked { "[x]" } else { "[ ]" };
                        let color = if checked { Color::Green } else { Color::DarkGray };
                        items.push(ListItem::new(Line::from(vec![
                            Span::styled(format!("{} ", mark), Style::default().fg(color).bold()),
                            Span::styled(g.name, Style::default().fg(Color::White)),
                        ])));
                    }

                    items.push(ListItem::new(Line::from(vec![
                        Span::styled("Remote Desktop Role: ", Style::default().fg(Color::Yellow).bold()),
                        Span::styled(host.remote_desktop.label(), Style::default().fg(Color::Cyan)),
                    ])));

                    render_cursor_list(frame, area, block, items, state.selected_index);
                }

                HostSubStep::NetworkingAndServer => {
                    let items = vec![
                        ListItem::new(Line::from(vec![
                            Span::styled(format!("[{}] ", if host.tailscale { "x" } else { " " }), Style::default().fg(Color::Green).bold()),
                            Span::styled("Tailscale Fleet Mesh VPN (Encrypted peer-to-peer networking)", Style::default().fg(Color::White)),
                        ])),
                        ListItem::new(Line::from(vec![
                            Span::styled(format!("[{}] ", if host.openssh { "x" } else { " " }), Style::default().fg(Color::Green).bold()),
                            Span::styled("OpenSSH Server (Allow remote shell logins)", Style::default().fg(Color::White)),
                        ])),
                    ];
                    render_cursor_list(frame, area, block, items, state.selected_index);
                }

                HostSubStep::Maintenance => {
                    let items = vec![
                        ListItem::new(Line::from(vec![
                            Span::styled(format!("[{}] ", if host.auto_upgrade { "x" } else { " " }), Style::default().fg(Color::Green).bold()),
                            Span::styled("Daily Unattended Upgrades (Auto-pull from your GitHub fork)", Style::default().fg(Color::White)),
                        ])),
                        ListItem::new(Line::from(vec![
                            Span::styled(format!("[{}] ", if host.restic_backup { "x" } else { " " }), Style::default().fg(Color::Green).bold()),
                            Span::styled("Automated Restic Snapshots & Cloud Backup", Style::default().fg(Color::White)),
                        ])),
                    ];
                    render_cursor_list(frame, area, block, items, state.selected_index);
                }
            }
        }

        Step::FleetPrompt => {
            let items = vec![
                ListItem::new(Line::from(vec![
                    Span::styled("+ Configure Another Machine in Fleet", Style::default().fg(Color::Cyan).bold()),
                ])),
                ListItem::new(Line::from(vec![
                    Span::styled("✔ Finished. Continue to Review & Deployment ❯", Style::default().fg(Color::Green).bold()),
                ])),
            ];
            render_cursor_list(frame, area, block, items, state.selected_index);
        }

        Step::AddHostType => {
            let types = [HostType::Laptop, HostType::PC, HostType::Server, HostType::Portable];
            let items: Vec<ListItem> = types
                .iter()
                .map(|t| {
                    ListItem::new(vec![
                        Line::from(Span::styled(t.label(), Style::default().fg(Color::White).bold())),
                        Line::from(Span::styled(format!("  {}", t.description()), Style::default().fg(Color::DarkGray))),
                    ])
                })
                .collect();

            render_cursor_list(frame, area, block, items, state.selected_index);
        }

        Step::Review => {
            let mut lines = Vec::new();
            lines.push(Line::from(Span::styled("ORBITOS ARCHITECTURE SUMMARY", Style::default().fg(Color::Cyan).bold())));
            lines.push(Line::from(format!("Fork Repository: ~/OrbitOS -> https://github.com/{}", state.github_fork)));
            lines.push(Line::from(""));

            lines.push(Line::from(Span::styled("Accounts:", Style::default().fg(Color::Yellow).bold())));
            for acc in &state.accounts {
                lines.push(Line::from(format!("  • {} ({})", acc.name, acc.account_type.label())));
            }
            lines.push(Line::from(""));

            lines.push(Line::from(Span::styled("Fleet Hosts:", Style::default().fg(Color::Yellow).bold())));
            for host in &state.hosts {
                lines.push(Line::from(format!(
                    "  • {} [{}] CPU: {:?} | GPU: {:?}",
                    host.hostname, host.host_type.short_name(), host.cpu, host.gpu
                )));
                lines.push(Line::from(format!("    WMs/DEs: {:?} | Shell: {:?}", host.des_and_wms, host.desktop_shell)));
                lines.push(Line::from(format!("    AI Enabled: {} | Gaming: {} apps | Remote: {:?}", host.ai_enabled, host.gaming.len(), host.remote_desktop)));
            }

            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled("Press [Enter] to choose Deployment Target.", Style::default().fg(Color::Green).bold())));

            let paragraph = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });
            frame.render_widget(paragraph, area);
        }

        Step::DeploymentMenu => {
            let choices = [
                DeploymentChoice::InstallLocal,
                DeploymentChoice::RemoteNixosAnywhere,
                DeploymentChoice::BuildIsoFlashUsb,
                DeploymentChoice::PushGitOnly,
                DeploymentChoice::BailOut,
            ];

            let items: Vec<ListItem> = choices
                .iter()
                .map(|c| {
                    let color = if *c == DeploymentChoice::BailOut {
                        Color::Red
                    } else {
                        Color::White
                    };
                    ListItem::new(Span::styled(c.label(), Style::default().fg(color).bold()))
                })
                .collect();

            render_cursor_list(frame, area, block, items, state.selected_index);
        }

        Step::BailOutConfirm => {
            // Rendered via modal overlay
        }

        Step::Done => {
            let p = Paragraph::new("Setup completed! Changes committed and ready.")
                .block(block)
                .alignment(Alignment::Center);
            frame.render_widget(p, area);
        }
    }
}

fn render_orbit_animation(frame: &mut Frame, area: Rect, state: &InstallerState) {
    let block = Block::default()
        .title(" Orbit ")
        .title_style(Style::default().fg(Color::Cyan).bold())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.width < 10 || inner.height < 4 {
        return;
    }

    let t = (state.animation_frame as f64) * 0.04;
    let grid = state.orbit_sim.render(inner.width as usize, inner.height as usize, t);

    let mut lines = Vec::new();
    for row in grid {
        let mut spans = Vec::new();
        for ch in row {
            let style = match ch {
                '[' | ']' | '0' | 'v' | '=' | '#' | '+' | '*' | '-' => {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                }
                '.' => Style::default().fg(Color::DarkGray),
                '|' | '/' | '\\' => Style::default().fg(Color::Cyan),
                ' ' => Style::default(),
                _ => Style::default().fg(Color::Cyan),
            };
            spans.push(Span::styled(ch.to_string(), style));
        }
        lines.push(Line::from(spans));
    }

    frame.render_widget(Paragraph::new(lines), inner);
}

fn render_cursor_list(
    frame: &mut Frame,
    area: Rect,
    block: Block,
    items: Vec<ListItem>,
    selected_index: usize,
) {
    let styled_items: Vec<ListItem> = items
        .into_iter()
        .enumerate()
        .map(|(i, item)| {
            if i == selected_index {
                item.style(Style::default().bg(Color::Rgb(30, 45, 60)).fg(Color::Yellow).bold())
            } else {
                item
            }
        })
        .collect();

    let list = List::new(styled_items).block(block);
    frame.render_widget(list, area);
}

fn render_bail_out_modal(frame: &mut Frame, size: Rect, state: &InstallerState) {
    let popup_area = centered_rect(60, 40, size);
    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(" CONFIRM DESTRUCTIVE ACTION ")
        .title_style(Style::default().fg(Color::Red).bold())
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(Color::Red));

    let text = vec![
        Line::from(Span::styled("WARNING: You are about to bail out!", Style::default().fg(Color::Red).bold())),
        Line::from(""),
        Line::from("This will discard all uncommitted changes in ~/OrbitOS without pushing to GitHub."),
        Line::from("Your remote repository will remain completely untouched."),
        Line::from(""),
        Line::from(vec![
            Span::styled(if state.selected_index == 0 { " > [ Discard & Exit ] < " } else { "   [ Discard & Exit ]   " }, Style::default().fg(Color::Red).bold()),
        ]),
        Line::from(vec![
            Span::styled(if state.selected_index == 1 { " > [ Cancel / Return to Setup ] < " } else { "   [ Cancel / Return to Setup ]   " }, Style::default().fg(Color::Green).bold()),
        ]),
    ];

    let p = Paragraph::new(text).block(block).alignment(Alignment::Center);
    frame.render_widget(p, popup_area);
}

fn render_info_modal(frame: &mut Frame, size: Rect, title: &str, msg: &str) {
    let popup_area = centered_rect(65, 45, size);
    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(format!(" {} ", title))
        .title_style(Style::default().fg(Color::Cyan).bold())
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(Color::Cyan));

    let mut lines = Vec::new();
    lines.push(Line::from(""));
    for line in msg.lines() {
        lines.push(Line::from(Span::styled(line.to_string(), Style::default().fg(Color::White))));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled("Press [Enter] or [Esc] to dismiss", Style::default().fg(Color::Green).bold())));

    let p = Paragraph::new(lines).block(block).alignment(Alignment::Center).wrap(Wrap { trim: true });
    frame.render_widget(p, popup_area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
