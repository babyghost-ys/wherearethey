mod aliases;
mod detect;
mod history;
mod hooks;
mod managers;
mod output;
mod types;
mod util;

use std::collections::BTreeSet;

use clap::{Parser, Subcommand};

use crate::aliases::{all_aliases, list_aliases, remove_alias, resolve_alias, set_alias};
use crate::detect::{lookup_binary, lookup_binary_fast};
use crate::history::{clear_history, log_install, print_history};
use crate::hooks::generate_zsh_hook;
use crate::managers::{get_all_path_binaries, scan_all, scan_all_quiet};
use crate::util::glob_matches;
use crate::output::*;
use crate::types::LookupResult;

// ── CLI ──────────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(
    name = "wherearethey",
    about = "Find where your CLI tools were installed from",
    version,
    disable_help_flag = true,
    disable_help_subcommand = true
)]
struct Cli {
    /// Binary or friendly name to look up (e.g. "ffmpeg", "rg", "Gemini")
    binary: Option<String>,

    /// List all detected CLI tools grouped by source
    #[arg(long)]
    all: bool,

    /// Show binaries in PATH that no package manager claims
    #[arg(long)]
    unmanaged: bool,

    /// Output as JSON
    #[arg(long)]
    json: bool,

    /// Print help
    #[arg(short, long)]
    help: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Output shell hook code for tracking future installs
    Hook {
        /// Shell type (currently only "zsh" is supported)
        shell: String,
    },
    /// Show install history recorded by shell hooks
    History {
        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Clear all history
        #[arg(long)]
        clear: bool,
    },
    /// (internal) Log an install event — called by the shell hooks
    #[command(hide = true)]
    Log {
        /// Package manager that performed the install
        source: String,
        /// Action performed (install, uninstall)
        action: String,
        /// Package names
        packages: Vec<String>,
    },
    /// Give a binary a friendly name for easy lookup
    Name {
        /// Binary name (e.g. "gemini-cli")
        binary: Option<String>,
        /// Friendly name (e.g. "Gemini")
        friendly_name: Option<String>,
        /// List all aliases
        #[arg(long)]
        list: bool,
        /// Remove an alias by friendly name
        #[arg(long)]
        remove: Option<String>,
    },
}

// ── Main ─────────────────────────────────────────────────────────────

fn print_help() {
    eprintln!("{BOLD}wherearethey{RESET} — find where your CLI tools were installed from\n");
    eprintln!("Usage:");
    eprintln!("  wherearethey <name>            Look up by binary, alias, or pattern");
    eprintln!("  wherearethey --all             List all tools by source");
    eprintln!("  wherearethey --unmanaged       Find unmanaged binaries");
    eprintln!("  wherearethey --json            Output as JSON");
    eprintln!("  wherearethey hook zsh          Output shell hooks for tracking");
    eprintln!("  wherearethey history           Show tracked install history");
    eprintln!("  wherearethey history --clear   Clear history");
    eprintln!("  wherearethey name <b> <name>   Give a binary a friendly name");
    eprintln!("  wherearethey name --list       List all friendly names");
    eprintln!("  wherearethey name --remove <n> Remove a friendly name\n");
    eprintln!("Setup tracking:");
    eprintln!("  eval \"$(wherearethey hook zsh)\"    # add to ~/.zshrc\n");
    eprintln!("Examples:");
    eprintln!("  wherearethey ffmpeg");
    eprintln!("  wherearethey 'cl*'            Glob: all binaries starting with cl");
    eprintln!("  wherearethey '*cli*'          Glob: all binaries containing cli");
    eprintln!("  wherearethey name gemini-cli Gemini");
    eprintln!("  wherearethey Gemini");
    eprintln!("  wherearethey --all --json\n");
}

fn main() {
    let cli = Cli::parse();

    if cli.help {
        print_help();
        return;
    }

    // Handle subcommands first
    if let Some(ref cmd) = cli.command {
        match cmd {
            Commands::Hook { shell } => {
                if shell != "zsh" {
                    eprintln!("  {RED}Only 'zsh' is supported for now.{RESET}");
                    eprintln!("  Usage: eval \"$(wherearethey hook zsh)\"");
                    std::process::exit(1);
                }
                print!("{}", generate_zsh_hook());
                return;
            }
            Commands::History { json, clear } => {
                if *clear {
                    clear_history();
                } else {
                    print_history(*json);
                }
                return;
            }
            Commands::Log {
                source,
                action,
                packages,
            } => {
                log_install(source, action, packages);
                return;
            }
            Commands::Name {
                binary,
                friendly_name,
                list,
                remove,
            } => {
                if *list {
                    list_aliases();
                } else if let Some(name) = remove {
                    remove_alias(name);
                } else if let (Some(bin), Some(name)) = (binary, friendly_name) {
                    set_alias(bin, name);
                } else {
                    eprintln!("  {RED}Usage: wherearethey name <binary> <friendly-name>{RESET}");
                    eprintln!("  {DIM}Example: wherearethey name gemini-cli Gemini{RESET}");
                    std::process::exit(1);
                }
                return;
            }
        }
    }

    // Single binary lookup (resolve alias first, or glob search)
    if let Some(ref name) = cli.binary {
        if !cli.all && !cli.unmanaged {
            // Glob pattern search
            if name.contains('*') || name.contains('?') {
                eprint!("  {DIM}Searching...{RESET}");

                let mut seen: BTreeSet<String> = BTreeSet::new();
                let mut results = Vec::new();

                // Match binary names from PATH
                let all_binaries = get_all_path_binaries();
                for b in &all_binaries {
                    if glob_matches(name, b) {
                        if seen.insert(b.clone()) {
                            if let Some(r) = lookup_binary(b) {
                                results.push(r);
                            }
                        }
                    }
                }

                // Match tool names from package managers (e.g. npm
                // package "gemini-cli" installs binary "gemini")
                let tools = scan_all_quiet();
                for tool in &tools {
                    if glob_matches(name, &tool.name) && !seen.contains(&tool.name) {
                        seen.insert(tool.name.clone());
                        results.push(LookupResult {
                            binary: tool.name.clone(),
                            resolved_path: tool.path.clone(),
                            source: tool.source.clone(),
                            symlink_target: None,
                            version: tool.version.clone(),
                        });
                    }
                }

                // Match alias friendly names
                let aliases = all_aliases();
                for (friendly, binary) in &aliases {
                    if glob_matches(name, friendly) && !seen.contains(friendly) {
                        seen.insert(friendly.clone());
                        if let Some(mut r) = lookup_binary(binary) {
                            r.binary = format!("{friendly} → {binary}");
                            results.push(r);
                        }
                    }
                }

                // Clear the "Searching..." line
                eprint!("\r                    \r");

                results.sort_by(|a, b| a.binary.cmp(&b.binary));

                if results.is_empty() {
                    eprintln!("  {RED}No tools matching '{name}' found{RESET}");
                    std::process::exit(1);
                }

                if cli.json {
                    println!("{}", serde_json::to_string_pretty(&results).unwrap());
                } else {
                    eprintln!(
                        "\n  {BOLD}Pattern \"{name}\"{RESET} {DIM}— {} matches{RESET}",
                        results.len()
                    );
                    for result in &results {
                        print_lookup(result);
                    }
                }
                return;
            }

            let resolved = resolve_alias(name).unwrap_or_else(|| name.clone());
            let display_alias = if resolved != *name {
                Some(name.as_str())
            } else {
                None
            };
            match lookup_binary(&resolved) {
                Some(result) => {
                    if cli.json {
                        println!("{}", serde_json::to_string_pretty(&result).unwrap());
                    } else {
                        if let Some(alias) = display_alias {
                            eprintln!(
                                "  {DIM}Alias \"{alias}\" → {}{RESET}\n",
                                result.binary
                            );
                        }
                        print_lookup(&result);
                    }
                }
                None => {
                    if display_alias.is_some() {
                        eprintln!(
                            "  {RED}'{resolved}' (alias for \"{name}\") not found in PATH{RESET}"
                        );
                    } else {
                        eprintln!("  {RED}'{name}' not found in PATH{RESET}");
                    }
                    std::process::exit(1);
                }
            }
            return;
        }
    }

    // --all: scan everything
    if cli.all {
        eprintln!("\n{BOLD}Scanning package managers...{RESET}\n");
        let tools = scan_all();
        if cli.json {
            println!("{}", serde_json::to_string_pretty(&tools).unwrap());
        } else {
            print_all(&tools);
        }
        return;
    }

    // --unmanaged: find binaries not managed by any package manager
    if cli.unmanaged {
        eprintln!("\n{BOLD}Scanning package managers...{RESET}\n");
        let tools = scan_all();
        let claimed: BTreeSet<String> = tools.iter().map(|t| t.name.clone()).collect();
        let all_binaries = get_all_path_binaries();

        let unclaimed: Vec<&String> = all_binaries
            .iter()
            .filter(|b| !claimed.contains(*b))
            .collect();
        let total = unclaimed.len();
        eprintln!("\n  {DIM}Checking {total} unclaimed binaries...{RESET}");

        let mut unmanaged = Vec::new();
        for (i, b) in unclaimed.iter().enumerate() {
            if (i + 1) % 50 == 0 || i + 1 == total {
                eprint!("\r  {DIM}Checked {}/{total}...{RESET}  ", i + 1);
            }
            if let Some(r) = lookup_binary_fast(b) {
                if r.source == "unknown" {
                    unmanaged.push(r);
                }
            }
        }
        eprintln!();

        if cli.json {
            println!("{}", serde_json::to_string_pretty(&unmanaged).unwrap());
        } else {
            print_unmanaged(&unmanaged);
        }
        return;
    }

    // No args — print help
    print_help();
}
