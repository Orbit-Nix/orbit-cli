// OrbitOS — Installer TUI View and Layout Renderer

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::installer::state::{
    HostSubStep, InstallerState, Step, AVAILABLE_APPS, AVAILABLE_FEATURES,
};
use crate::installer::types::{AccountType, CpuType, DesktopShell, GpuType, HostType};

pub fn render(frame: &mut Frame, state: &InstallerState) {
    let size = frame.area();

    // Base background cleanup
    frame.render_widget(Clear, size);

    // Root layout: Title bar, Main Content Area, Footer Keybinds
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Top header
            Constraint::Min(10),   // Main area (left interactive + right sidebar)
            Constraint::Length(3), // Footer key hints
        ])
        .split(size);

    render_header(frame, chunks[0]);
    render_main(frame, chunks[1], state);
    render_footer(frame, chunks[2], state);
}

fn render_header(frame: &mut Frame, area: Rect) {
    let header_text = vec![
        Line::from(vec![
            Span::styled(" ORBITOS ", Style::default().fg(Color::Black).bg(Color::Cyan).bold()),
            Span::raw(" "),
            Span::styled("System Architecture & Host Installer", Style::default().fg(Color::White).bold()),
            Span::styled("  v0.1.0", Style::default().fg(Color::DarkGray)),
        ]),
    ];

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
            Span::styled("[Enter]", Style::default().fg(Color::Cyan).bold()),
            Span::raw(" Begin Setup   "),
            Span::styled("[q]", Style::default().fg(Color::DarkGray)),
            Span::raw(" Quit"),
        ],
        Step::AddAccountName { .. } | Step::AddHostName { .. } => vec![
            Span::styled("[Type]", Style::default().fg(Color::Cyan).bold()),
            Span::raw(" Enter Name   "),
            Span::styled("[Enter]", Style::default().fg(Color::Green).bold()),
            Span::raw(" Confirm   "),
            Span::styled("[Esc]", Style::default().fg(Color::DarkGray)),
            Span::raw(" Cancel"),
        ],
        Step::HostConfig { sub_step, .. } => match sub_step {
            HostSubStep::AssignUsers | HostSubStep::SelectApps | HostSubStep::SelectFeatures => vec![
                Span::styled("[↑/↓]", Style::default().fg(Color::Cyan)),
                Span::raw(" Move   "),
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
            Span::raw(" Generate Config & Install   "),
            Span::styled("[Esc]", Style::default().fg(Color::Yellow)),
            Span::raw(" Back to Hosts   "),
            Span::styled("[q]", Style::default().fg(Color::DarkGray)),
            Span::raw(" Cancel"),
        ],
        _ => vec![
            Span::styled("[↑/↓]", Style::default().fg(Color::Cyan)),
            Span::raw(" Navigate   "),
            Span::styled("[Enter]", Style::default().fg(Color::Green).bold()),
            Span::raw(" Select / Confirm   "),
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
    // 68% Left interactive area, 32% Right Roadmap Sidebar
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
        .constraints([Constraint::Length(4), Constraint::Min(8)])
        .split(area);

    // Current Scope Box
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

    // Vertical Roadmap Box
    let roadmap_block = Block::default()
        .title(" Roadmap ")
        .title_style(Style::default().fg(Color::DarkGray).bold())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));

    let mut roadmap_items = Vec::new();

    // 1. Accounts
    let is_accounts_active = matches!(
        state.current_step,
        Step::AccountsMenu | Step::AddAccountType | Step::AddAccountName { .. }
    );
    let accounts_done = !state.accounts.is_empty() && !matches!(state.current_step, Step::Welcome | Step::AccountsMenu | Step::AddAccountType | Step::AddAccountName { .. });
    roadmap_items.push(format_roadmap_entry(
        "Accounts",
        &format!("{} user(s)", state.accounts.len()),
        is_accounts_active,
        accounts_done,
    ));

    // 2. Hosts
    let is_hosts_menu_active = matches!(
        state.current_step,
        Step::HostsMenu | Step::AddHostType | Step::AddHostName { .. }
    );
    let hosts_done = !state.hosts.is_empty() && !matches!(
        state.current_step,
        Step::Welcome | Step::AccountsMenu | Step::AddAccountType | Step::AddAccountName { .. } | Step::HostsMenu | Step::AddHostType | Step::AddHostName { .. }
    );
    roadmap_items.push(format_roadmap_entry(
        "Hosts Architecture",
        &format!("{} host(s)", state.hosts.len()),
        is_hosts_menu_active,
        hosts_done,
    ));

    // 3. Per Host Steps
    for (idx, host) in state.hosts.iter().enumerate() {
        let (is_active_host, active_substep) = match &state.current_step {
            Step::HostConfig { host_idx, sub_step } if *host_idx == idx => (true, Some(sub_step)),
            _ => (false, None),
        };

        let host_completed = match &state.current_step {
            Step::HostConfig { host_idx, .. } => idx < *host_idx,
            Step::Review | Step::Done => true,
            _ => false,
        };

        let host_symbol = if is_active_host {
            "▶"
        } else if host_completed {
            "✔"
        } else {
            "○"
        };

        let host_color = if is_active_host {
            Color::Cyan
        } else if host_completed {
            Color::Green
        } else {
            Color::DarkGray
        };

        roadmap_items.push(Line::from(vec![
            Span::styled(format!("  {} ", host_symbol), Style::default().fg(host_color).bold()),
            Span::styled(&host.hostname, Style::default().fg(if is_active_host { Color::White } else { Color::Gray })),
            Span::styled(format!(" ({})", host.host_type.short_name()), Style::default().fg(Color::DarkGray)),
        ]));

        if is_active_host {
            let substeps = [
                (HostSubStep::AssignUsers, "Users"),
                (HostSubStep::SelectCpu, "CPU"),
                (HostSubStep::SelectGpu, "GPU"),
                (HostSubStep::SelectShell, "Shell"),
                (HostSubStep::SelectApps, "Apps"),
                (HostSubStep::SelectFeatures, "Features"),
            ];

            for (ss, label) in substeps {
                let is_current = active_substep == Some(&ss);
                let mark = if is_current { "  ▸" } else { "   ·" };
                let style = if is_current {
                    Style::default().fg(Color::Yellow).bold()
                } else {
                    Style::default().fg(Color::DarkGray)
                };
                roadmap_items.push(Line::from(vec![
                    Span::styled(format!("{} ", mark), style),
                    Span::styled(label, style),
                ]));
            }
        }
    }

    // 4. Review
    let is_review = matches!(state.current_step, Step::Review);
    let review_done = matches!(state.current_step, Step::Done);
    roadmap_items.push(format_roadmap_entry("Review & Install", "", is_review, review_done));

    let roadmap_p = Paragraph::new(roadmap_items)
        .block(roadmap_block)
        .wrap(Wrap { trim: true });

    frame.render_widget(roadmap_p, sidebar_chunks[1]);
}

fn format_roadmap_entry(label: &str, note: &str, is_active: bool, is_done: bool) -> Line<'static> {
    let (symbol, color) = if is_active {
        ("▶", Color::Cyan)
    } else if is_done {
        ("✔", Color::Green)
    } else {
        ("○", Color::DarkGray)
    };

    let mut spans = vec![
        Span::styled(format!("{} ", symbol), Style::default().fg(color).bold()),
        Span::styled(
            label.to_string(),
            Style::default()
                .fg(if is_active {
                    Color::White
                } else if is_done {
                    Color::Gray
                } else {
                    Color::DarkGray
                })
                .add_modifier(if is_active {
                    Modifier::BOLD
                } else {
                    Modifier::empty()
                }),
        ),
    ];

    if !note.is_empty() {
        spans.push(Span::styled(
            format!(" ({})", note),
            Style::default().fg(Color::DarkGray),
        ));
    }

    Line::from(spans)
}

fn render_left_pane(frame: &mut Frame, area: Rect, state: &InstallerState) {
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8), // Task / Question header
            Constraint::Min(8),    // Answer widget (List / Inputs)
        ])
        .split(area);

    render_task_question(frame, left_chunks[0], state);
    render_answer_widget(frame, left_chunks[1], state);
}

fn render_task_question(frame: &mut Frame, area: Rect, state: &InstallerState) {
    let (title, description) = match &state.current_step {
        Step::Welcome => (
            "Welcome to OrbitOS Installer",
            "This installer configures your declarative NixOS architecture.\n\
             We will set up user accounts, declare machines (laptop, PC, server, portable),\n\
             and customize the hardware profiles, desktop shells, and apps for each host.",
        ),
        Step::AccountsMenu => (
            "Account Setup",
            "Configure user accounts for your OrbitOS ecosystem.\n\
             You can create standard user accounts, restricted users, and RAM-wiped anonymous accounts.",
        ),
        Step::AddAccountType => (
            "Select Account Type",
            "Choose the privilege level and operational model for this new account.",
        ),
        Step::AddAccountName { account_type } => (
            "Account Username",
            match account_type {
                AccountType::User => "Enter username for the primary administrator account (e.g. musa, dev, admin).",
                AccountType::Restricted => "Enter username for the restricted account (e.g. oliver, guest, worker).",
                AccountType::Anonymous => "Enter username for the anonymous session (e.g. incognito, guest).",
            },
        ),
        Step::HostsMenu => (
            "Host Machines Architecture",
            "OrbitOS configs are modular and multi-host. Add all machines you intend to run.\n\
             Pick from Laptop, PC, Server, or Portable templates.",
        ),
        Step::AddHostType => (
            "Select Host Archetype",
            "Select machine archetype. Each has tailored hardware profiles, power settings, and services.",
        ),
        Step::AddHostName { host_type } => (
            "Assign Hostname",
            match host_type {
                HostType::Laptop => "Enter hostname for this laptop (e.g. lt-hp15-nix, framework, laptop).",
                HostType::PC => "Enter hostname for this desktop workstation (e.g. workstation, desktop, battlestation).",
                HostType::Server => "Enter hostname for this server (e.g. srv-c4030, homelab, lunar).",
                HostType::Portable => "Enter hostname for portable installation (e.g. portable, usb-key).",
            },
        ),
        Step::HostConfig { host_idx, sub_step } => {
            let _host = &state.hosts[*host_idx];
            match sub_step {
                HostSubStep::AssignUsers => (
                    "Assign Accounts to Host",
                    "Select which created user accounts should have a login and home directory on this machine.",
                ),
                HostSubStep::SelectCpu => (
                    "CPU Architecture",
                    "Select the processor vendor to include the appropriate microcode and CPU frequency drivers.",
                ),
                HostSubStep::SelectGpu => (
                    "Graphics / Display Hardware",
                    "Select the GPU configuration to enable hardware acceleration, Vulkan, and display drivers.",
                ),
                HostSubStep::SelectShell => (
                    "Default Desktop Shell",
                    "Select the default Hyprland desktop shell experience. You can switch anytime using `orbit shell`.",
                ),
                HostSubStep::SelectApps => (
                    "Default Applications",
                    "Select apps to pre-install into this host's declarative manifest. Press [Space] to toggle.",
                ),
                HostSubStep::SelectFeatures => (
                    "System Features & Gaming",
                    "Select gaming runtimes, streaming servers, and background container features for this host.",
                ),
            }
        }
        Step::Review => (
            "Review Configuration",
            "Review your OrbitOS specification before generating the NixOS flakes and modules.",
        ),
        Step::Done => (
            "Configuration Ready",
            "Your OrbitOS configuration is ready to install.",
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
            let items = vec![
                ListItem::new(Line::from(vec![
                    Span::styled("❯ ", Style::default().fg(Color::Green).bold()),
                    Span::styled("Start OrbitOS Setup", Style::default().fg(Color::White).bold()),
                ])),
            ];
            let list = List::new(items).block(block);
            frame.render_widget(list, area);
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
                    Span::styled("✔ Continue to Hosts Setup ❯", Style::default().fg(Color::Green).bold()),
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
                .title(" Input Name ");

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

        Step::HostsMenu => {
            let mut items = Vec::new();
            items.push(ListItem::new(Line::from(vec![
                Span::styled("+ Add New Host Machine", Style::default().fg(Color::Cyan).bold()),
            ])));

            for host in &state.hosts {
                items.push(ListItem::new(Line::from(vec![
                    Span::styled("• ", Style::default().fg(Color::Green)),
                    Span::styled(&host.hostname, Style::default().fg(Color::White).bold()),
                    Span::styled(format!(" [{}]", host.host_type.short_name()), Style::default().fg(Color::DarkGray)),
                ])));
            }

            if !state.hosts.is_empty() {
                items.push(ListItem::new(Line::from(vec![
                    Span::styled("✔ Configure Hosts & Features ❯", Style::default().fg(Color::Green).bold()),
                ])));
            }

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

        Step::HostConfig { host_idx, sub_step } => {
            let host = &state.hosts[*host_idx];

            match sub_step {
                HostSubStep::AssignUsers => {
                    let items: Vec<ListItem> = state
                        .accounts
                        .iter()
                        .map(|acc| {
                            let checked = host.assigned_users.contains(&acc.name);
                            let box_mark = if checked { "[x]" } else { "[ ]" };
                            let color = if checked { Color::Green } else { Color::DarkGray };
                            ListItem::new(Line::from(vec![
                                Span::styled(format!("{} ", box_mark), Style::default().fg(color).bold()),
                                Span::styled(&acc.name, Style::default().fg(Color::White)),
                                Span::styled(format!(" ({})", acc.account_type.short_name()), Style::default().fg(Color::DarkGray)),
                            ]))
                        })
                        .collect();

                    render_cursor_list(frame, area, block, items, state.selected_index);
                }

                HostSubStep::SelectCpu => {
                    let cpus = [CpuType::Amd, CpuType::Intel, CpuType::Universal];
                    let items: Vec<ListItem> = cpus
                        .iter()
                        .map(|c| {
                            let selected = host.cpu == *c;
                            let radio = if selected { "(●)" } else { "( )" };
                            ListItem::new(Line::from(vec![
                                Span::styled(format!("{} ", radio), Style::default().fg(if selected { Color::Cyan } else { Color::DarkGray }).bold()),
                                Span::styled(c.label(), Style::default().fg(Color::White)),
                            ]))
                        })
                        .collect();

                    render_cursor_list(frame, area, block, items, state.selected_index);
                }

                HostSubStep::SelectGpu => {
                    let gpus = [
                        GpuType::Amd,
                        GpuType::NvidiaDesktop,
                        GpuType::NvidiaHybrid,
                        GpuType::NvidiaLegacy,
                        GpuType::IntelIntegrated,
                        GpuType::IntelArc,
                        GpuType::Universal,
                        GpuType::None,
                    ];
                    let items: Vec<ListItem> = gpus
                        .iter()
                        .map(|g| {
                            let selected = host.gpu == *g;
                            let radio = if selected { "(●)" } else { "( )" };
                            ListItem::new(Line::from(vec![
                                Span::styled(format!("{} ", radio), Style::default().fg(if selected { Color::Cyan } else { Color::DarkGray }).bold()),
                                Span::styled(g.label(), Style::default().fg(Color::White)),
                            ]))
                        })
                        .collect();

                    render_cursor_list(frame, area, block, items, state.selected_index);
                }

                HostSubStep::SelectShell => {
                    let shells = [DesktopShell::End4Pc, DesktopShell::Midnight, DesktopShell::Dms, DesktopShell::None];
                    let items: Vec<ListItem> = shells
                        .iter()
                        .map(|s| {
                            let selected = host.desktop_shell == *s;
                            let radio = if selected { "(●)" } else { "( )" };
                            ListItem::new(Line::from(vec![
                                Span::styled(format!("{} ", radio), Style::default().fg(if selected { Color::Cyan } else { Color::DarkGray }).bold()),
                                Span::styled(s.label(), Style::default().fg(Color::White)),
                            ]))
                        })
                        .collect();

                    render_cursor_list(frame, area, block, items, state.selected_index);
                }

                HostSubStep::SelectApps => {
                    let items: Vec<ListItem> = AVAILABLE_APPS
                        .iter()
                        .map(|app| {
                            let checked = host.browsers.contains(&app.id.to_string())
                                || host.file_managers.contains(&app.id.to_string())
                                || host.ides.contains(&app.id.to_string())
                                || host.ai_tools.contains(&app.id.to_string())
                                || host.terminals.contains(&app.id.to_string());

                            let box_mark = if checked { "[x]" } else { "[ ]" };
                            let color = if checked { Color::Green } else { Color::DarkGray };
                            ListItem::new(Line::from(vec![
                                Span::styled(format!("{} ", box_mark), Style::default().fg(color).bold()),
                                Span::styled(app.name, Style::default().fg(Color::White)),
                                Span::styled(format!(" [{}]", app.category), Style::default().fg(Color::DarkGray)),
                            ]))
                        })
                        .collect();

                    render_cursor_list(frame, area, block, items, state.selected_index);
                }

                HostSubStep::SelectFeatures => {
                    let items: Vec<ListItem> = AVAILABLE_FEATURES
                        .iter()
                        .map(|feat| {
                            let checked = host.gaming.contains(&feat.id.to_string()) || host.extras.contains(&feat.id.to_string());
                            let box_mark = if checked { "[x]" } else { "[ ]" };
                            let color = if checked { Color::Green } else { Color::DarkGray };
                            ListItem::new(Line::from(vec![
                                Span::styled(format!("{} ", box_mark), Style::default().fg(color).bold()),
                                Span::styled(feat.name, Style::default().fg(Color::White)),
                                Span::styled(format!(" [{}]", feat.category), Style::default().fg(Color::DarkGray)),
                            ]))
                        })
                        .collect();

                    render_cursor_list(frame, area, block, items, state.selected_index);
                }
            }
        }

        Step::Review => {
            let mut lines = Vec::new();
            lines.push(Line::from(Span::styled("ORBITOS ARCHITECTURE SUMMARY", Style::default().fg(Color::Cyan).bold())));
            lines.push(Line::from(""));

            lines.push(Line::from(Span::styled("Accounts:", Style::default().fg(Color::Yellow).bold())));
            for acc in &state.accounts {
                lines.push(Line::from(vec![
                    Span::raw("  • "),
                    Span::styled(&acc.name, Style::default().fg(Color::White).bold()),
                    Span::styled(format!(" ({})", acc.account_type.label()), Style::default().fg(Color::DarkGray)),
                ]));
            }
            lines.push(Line::from(""));

            lines.push(Line::from(Span::styled("Hosts:", Style::default().fg(Color::Yellow).bold())));
            for host in &state.hosts {
                lines.push(Line::from(vec![
                    Span::raw("  • "),
                    Span::styled(&host.hostname, Style::default().fg(Color::White).bold()),
                    Span::styled(format!(" [{}]", host.host_type.short_name()), Style::default().fg(Color::Cyan)),
                    Span::styled(format!(" CPU: {:?}, GPU: {:?}", host.cpu, host.gpu), Style::default().fg(Color::DarkGray)),
                ]));
                lines.push(Line::from(format!("    Users: {}", host.assigned_users.join(", "))));
                lines.push(Line::from(format!("    Shell: {:?}", host.desktop_shell)));
                lines.push(Line::from(format!("    Apps: {} total configured", host.browsers.len() + host.ides.len() + host.ai_tools.len())));
            }

            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled("Press [Enter] to generate configuration files and continue.", Style::default().fg(Color::Green).bold())));

            let paragraph = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });
            frame.render_widget(paragraph, area);
        }

        Step::Done => {
            let p = Paragraph::new("Setup completed! Press any key to finish.")
                .block(block)
                .alignment(Alignment::Center);
            frame.render_widget(p, area);
        }
    }
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
                // Add cursor prefix
                item.style(Style::default().bg(Color::Rgb(30, 45, 60)).fg(Color::Yellow).bold())
            } else {
                item
            }
        })
        .collect();

    let list = List::new(styled_items).block(block);
    frame.render_widget(list, area);
}
