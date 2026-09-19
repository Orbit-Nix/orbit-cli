use std::collections::HashSet;
use std::io::{self, Write};
use std::process::Command;
use anyhow::{bail, Context, Result};
use colored::Colorize;
use serde::Deserialize;

use crate::util::{print_banner, print_err, print_warn, run_interactive};

#[derive(Debug, Deserialize)]
struct SearchResponse {
    results: Option<Vec<SearchResult>>,
}

#[derive(Debug, Deserialize, Clone)]
struct SearchResult {
    package_attr_name: Option<String>,
    package_pname: Option<String>,
    package_pversion: Option<String>,
    package_description: Option<String>,
    package_mainProgram: Option<String>,
    package_programs: Option<Vec<String>>,
}

/// Helper to check if a package / program is already in PATH
fn is_installed(pkg_name: &str, main_prog: Option<&str>) -> bool {
    if let Some(prog) = main_prog {
        if Command::new("which").arg(prog).output().map(|o| o.status.success()).unwrap_or(false) {
            return true;
        }
    }
    Command::new("which").arg(pkg_name).output().map(|o| o.status.success()).unwrap_or(false)
}

/// Queries packages using nh search (or nix search fallback)
fn query_packages(query: &str) -> Result<Vec<SearchResult>> {
    // 1. Try nh search packages <query> --json --limit 20
    let nh_out = Command::new("nh")
        .arg("search")
        .arg("packages")
        .arg(query)
        .arg("--json")
        .arg("--limit")
        .arg("20")
        .output();

    if let Ok(out) = nh_out {
        if out.status.success() {
            if let Ok(parsed) = serde_json::from_slice::<SearchResponse>(&out.stdout) {
                if let Some(mut results) = parsed.results {
                    if !results.is_empty() {
                        // Prioritize exact matches to the top
                        results.sort_by(|a, b| {
                            let a_name = a.package_attr_name.as_deref().unwrap_or("");
                            let b_name = b.package_attr_name.as_deref().unwrap_or("");
                            let a_exact = a_name == query;
                            let b_exact = b_name == query;
                            b_exact.cmp(&a_exact)
                        });
                        return Ok(results);
                    }
                }
            }
        }
    }

    // 2. Fallback to nix search nixpkgs <query> --json
    let nix_out = Command::new("nix")
        .arg("search")
        .arg("nixpkgs")
        .arg(query)
        .arg("--json")
        .output()
        .context("Failed to search packages using nix")?;

    if !nix_out.status.success() {
        bail!("Failed to query package repositories.");
    }

    let val: serde_json::Value = serde_json::from_slice(&nix_out.stdout)?;
    let mut results = Vec::new();

    if let Some(obj) = val.as_object() {
        for (key, v) in obj {
            // key is like "legacyPackages.x86_64-linux.sl" or "packages.x86_64-linux.sl"
            let attr = key.split('.').last().unwrap_or(key).to_string();
            let pname = v.get("pname").and_then(|s| s.as_str()).unwrap_or(&attr).to_string();
            let version = v.get("version").and_then(|s| s.as_str()).map(|s| s.to_string());
            let description = v.get("description").and_then(|s| s.as_str()).map(|s| s.to_string());

            results.push(SearchResult {
                package_attr_name: Some(attr),
                package_pname: Some(pname),
                package_pversion: version,
                package_description: description,
                package_mainProgram: None,
                package_programs: None,
            });
        }
    }

    Ok(results)
}

/// Parses selection strings like "1", "1 2 3", "1-3, 5"
fn parse_selection(input: &str, max_len: usize) -> Vec<usize> {
    let mut selected_indices = HashSet::new();
    let cleaned = input.replace(',', " ");
    for token in cleaned.split_whitespace() {
        if token.contains('-') {
            let parts: Vec<&str> = token.split('-').collect();
            if parts.len() == 2 {
                if let (Ok(start), Ok(end)) = (parts[0].parse::<usize>(), parts[1].parse::<usize>()) {
                    let (min_idx, max_idx) = if start <= end { (start, end) } else { (end, start) };
                    for idx in min_idx..=max_idx {
                        if idx >= 1 && idx <= max_len {
                            selected_indices.insert(idx - 1);
                        }
                    }
                }
            }
        } else if let Ok(idx) = token.parse::<usize>() {
            if idx >= 1 && idx <= max_len {
                selected_indices.insert(idx - 1);
            }
        }
    }
    let mut list: Vec<usize> = selected_indices.into_iter().collect();
    list.sort();
    list
}

pub fn handle_run(query: &str, is_gui: bool) -> Result<()> {
    print_banner(&format!("Searching for packages matching '{}'...", query.cyan()));

    let results = query_packages(query)?;
    if results.is_empty() {
        print_warn(&format!("No packages found matching '{}'.", query));
        return Ok(());
    }

    println!();
    // Print styled results yay-like in 1..N order
    for (i, pkg) in results.iter().enumerate() {
        let num = (i + 1).to_string().bold().green();
        let attr_name = pkg.package_attr_name.as_deref().unwrap_or("unknown");
        let version = pkg.package_pversion.as_deref().unwrap_or("");
        let main_prog = pkg.package_mainProgram.as_deref();
        let installed = is_installed(attr_name, main_prog);

        let status_badge = if installed {
            format!(" [{}]", "installed".bold().green())
        } else {
            "".to_string()
        };

        let ver_str = if !version.is_empty() {
            format!(" ({})", version.yellow())
        } else {
            "".to_string()
        };

        println!(
            "{:>3} {} <{}>{}{}",
            num,
            attr_name.bold().white(),
            "nixpkgs".cyan(),
            ver_str,
            status_badge
        );

        if let Some(desc) = &pkg.package_description {
            let trimmed_desc = desc.trim();
            if !trimmed_desc.is_empty() {
                println!("    {} {}", "↳".cyan(), trimmed_desc.dimmed());
            }
        }
    }

    println!();
    print!("{} ", "==> Enter n° of package(s) to run (e.g. 1 2 3, 1-3, or ^C to abort):".bold().cyan());
    let _ = io::stdout().flush();

    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() || input.trim().is_empty() {
        print_banner("No selection made. Aborted.");
        return Ok(());
    }

    let selected_indices = parse_selection(input.trim(), results.len());
    if selected_indices.is_empty() {
        print_err("Invalid selection. Aborted.");
        return Ok(());
    }

    let mut chosen_attrs: Vec<String> = Vec::new();
    let mut first_main_prog: Option<String> = None;

    for &idx in &selected_indices {
        let pkg = &results[idx];
        let attr = pkg.package_attr_name.clone().unwrap_or_else(|| "unknown".to_string());
        if first_main_prog.is_none() {
            first_main_prog = pkg.package_mainProgram.clone().or_else(|| Some(attr.clone()));
        }
        chosen_attrs.push(attr);
    }

    let chosen_str = chosen_attrs.join(", ");
    print_banner(&format!(
        "Preparing environment with package(s): {}...",
        chosen_str.bold().green()
    ));

    if is_gui {
        // Automatically launch the main application program
        let prog_to_run = first_main_prog.unwrap_or_else(|| chosen_attrs[0].clone());
        print_banner(&format!("Launching GUI application '{}'...", prog_to_run.bold().green()));

        // nix-shell -p <pkg1> <pkg2> --run "<prog>"
        let mut shell_cmd = Command::new("nix-shell");
        shell_cmd.arg("-p");
        for attr in &chosen_attrs {
            shell_cmd.arg(format!("nixpkgs#{}", attr));
        }
        shell_cmd.arg("--run").arg(&prog_to_run);

        run_interactive(&mut shell_cmd)?;
    } else {
        // Drop user into interactive nix-shell with all selected packages installed
        print_banner("Entering nix-shell environment (type 'exit' or press Ctrl+D to return)...");

        let mut shell_cmd = Command::new("nix-shell");
        shell_cmd.arg("-p");
        for attr in &chosen_attrs {
            shell_cmd.arg(format!("nixpkgs#{}", attr));
        }

        run_interactive(&mut shell_cmd)?;
    }

    Ok(())
}
