use std::fs;
use std::io::Write;

use crate::output::*;
use crate::types::InstallEvent;
use crate::util::{home_dir, run_cmd};

fn history_dir() -> std::path::PathBuf {
    home_dir().join(".wherearethey")
}

fn history_file() -> std::path::PathBuf {
    history_dir().join("history.json")
}

fn load_history() -> Vec<InstallEvent> {
    let path = history_file();
    if !path.exists() {
        return vec![];
    }
    let data = fs::read_to_string(&path).unwrap_or_default();
    serde_json::from_str(&data).unwrap_or_default()
}

fn save_history(events: &[InstallEvent]) {
    let dir = history_dir();
    if !dir.exists() {
        let _ = fs::create_dir_all(&dir);
    }
    let json = serde_json::to_string_pretty(events).unwrap_or_default();
    if let Ok(mut f) = fs::File::create(history_file()) {
        let _ = f.write_all(json.as_bytes());
    }
}

pub fn log_install(source: &str, action: &str, packages: &[String]) {
    let mut history = load_history();
    let now = run_cmd("date", &["+%Y-%m-%d %H:%M:%S"])
        .unwrap_or_else(|| "unknown".into())
        .trim()
        .to_string();
    history.push(InstallEvent {
        timestamp: now,
        source: source.to_string(),
        action: action.to_string(),
        packages: packages.to_vec(),
    });
    save_history(&history);
}

pub fn print_history(json_output: bool) {
    let history = load_history();
    if history.is_empty() {
        if json_output {
            println!("[]");
        } else {
            eprintln!("  {DIM}No install history recorded yet.{RESET}");
            eprintln!("  {DIM}Add this to your ~/.zshrc to start tracking:{RESET}");
            eprintln!("  eval \"$(wherearethey hook zsh)\"\n");
        }
        return;
    }
    if json_output {
        println!("{}", serde_json::to_string_pretty(&history).unwrap());
        return;
    }
    println!(
        "\n  {BOLD}Install history{RESET} {DIM}({} events){RESET}\n",
        history.len()
    );
    for event in &history {
        let colour = source_colour(&event.source);
        let action_colour = if event.action == "uninstall" {
            RED
        } else {
            GREEN
        };
        println!(
            "  {DIM}{}{RESET}  {action_colour}{:<10}{RESET} {colour}{:<10}{RESET} {}",
            event.timestamp,
            event.action,
            event.source,
            event.packages.join(", ")
        );
    }
    println!();
}

pub fn clear_history() {
    save_history(&[]);
    eprintln!("  {GREEN}History cleared.{RESET}");
}
