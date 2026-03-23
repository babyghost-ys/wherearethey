use std::collections::BTreeMap;

use crate::types::{LookupResult, ToolInfo};

pub const RESET: &str = "\x1b[0m";
pub const BOLD: &str = "\x1b[1m";
pub const DIM: &str = "\x1b[2m";
pub const GREEN: &str = "\x1b[32m";
pub const YELLOW: &str = "\x1b[33m";
pub const CYAN: &str = "\x1b[36m";
pub const RED: &str = "\x1b[31m";
pub const MAGENTA: &str = "\x1b[35m";

pub fn source_colour(source: &str) -> &'static str {
    match source {
        "brew" => GREEN,
        "cargo" => YELLOW,
        "npm" | "pnpm" | "bun" => RED,
        "go" => CYAN,
        "pipx" | "uv" | "pip" => MAGENTA,
        "macos-system" | "xcode-clt" => DIM,
        _ => RESET,
    }
}

pub fn print_lookup(result: &LookupResult) {
    let colour = source_colour(&result.source);
    println!("\n  {BOLD}{}{RESET}", result.binary);
    println!("  {DIM}path:{RESET}    {}", result.resolved_path);
    if let Some(ref target) = result.symlink_target {
        println!("  {DIM}target:{RESET}  {target}");
    }
    println!("  {DIM}source:{RESET}  {colour}{}{RESET}", result.source);
    if let Some(ref ver) = result.version {
        println!("  {DIM}version:{RESET} {ver}");
    }
    println!();
}

pub fn print_all(tools: &[ToolInfo]) {
    let mut by_source: BTreeMap<String, Vec<&ToolInfo>> = BTreeMap::new();
    for tool in tools {
        by_source.entry(tool.source.clone()).or_default().push(tool);
    }

    for (source, tools) in &by_source {
        let colour = source_colour(source);
        println!(
            "\n  {colour}{BOLD}{source}{RESET} {DIM}({} tools){RESET}",
            tools.len()
        );
        for tool in tools {
            let ver = tool.version.as_deref().unwrap_or("");
            if ver.is_empty() {
                println!("    {}", tool.name);
            } else {
                println!("    {} {DIM}{ver}{RESET}", tool.name);
            }
        }
    }
    println!();
}

pub fn print_unmanaged(items: &[LookupResult]) {
    if items.is_empty() {
        println!("\n  {GREEN}No unmanaged binaries found. Everything is claimed.{RESET}\n");
        return;
    }
    println!(
        "\n  {YELLOW}{BOLD}Unmanaged{RESET} {DIM}({} binaries not managed by any package manager){RESET}\n",
        items.len()
    );
    for item in items {
        let target_info = item
            .symlink_target
            .as_deref()
            .map(|t| format!(" {DIM}-> {t}{RESET}"))
            .unwrap_or_default();
        println!(
            "    {YELLOW}{}{RESET}  {DIM}{}{RESET}{target_info}",
            item.binary, item.resolved_path
        );
    }
    println!();
}
