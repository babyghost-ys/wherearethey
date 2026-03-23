use std::collections::BTreeMap;
use std::fs;
use std::io::Write;

use crate::output::*;
use crate::util::home_dir;

fn aliases_file() -> std::path::PathBuf {
    home_dir().join(".wherearethey").join("aliases.json")
}

fn load_aliases() -> BTreeMap<String, String> {
    let path = aliases_file();
    if !path.exists() {
        return BTreeMap::new();
    }
    let data = fs::read_to_string(&path).unwrap_or_default();
    serde_json::from_str(&data).unwrap_or_default()
}

fn save_aliases(aliases: &BTreeMap<String, String>) {
    let dir = home_dir().join(".wherearethey");
    if !dir.exists() {
        let _ = fs::create_dir_all(&dir);
    }
    let json = serde_json::to_string_pretty(aliases).unwrap_or_default();
    if let Ok(mut f) = fs::File::create(aliases_file()) {
        let _ = f.write_all(json.as_bytes());
    }
}

/// Set an alias: maps a friendly name to a binary name.
/// e.g. set_alias("gemini-cli", "Gemini") means `wherearethey Gemini` → looks up `gemini-cli`
pub fn set_alias(binary: &str, friendly_name: &str) {
    let mut aliases = load_aliases();
    aliases.insert(friendly_name.to_lowercase(), binary.to_string());
    save_aliases(&aliases);
    eprintln!(
        "  {GREEN}Alias set:{RESET} \"{friendly_name}\" {DIM}→{RESET} {BOLD}{binary}{RESET}"
    );
}

/// Remove an alias by its friendly name.
pub fn remove_alias(friendly_name: &str) {
    let mut aliases = load_aliases();
    let key = friendly_name.to_lowercase();
    if aliases.remove(&key).is_some() {
        save_aliases(&aliases);
        eprintln!("  {GREEN}Alias removed:{RESET} \"{friendly_name}\"");
    } else {
        eprintln!("  {RED}No alias found for \"{friendly_name}\"{RESET}");
    }
}

/// List all aliases.
pub fn list_aliases() {
    let aliases = load_aliases();
    if aliases.is_empty() {
        eprintln!("  {DIM}No aliases set yet.{RESET}");
        eprintln!("  {DIM}Usage: wherearethey name <binary> <friendly-name>{RESET}");
        return;
    }
    println!("\n  {BOLD}Aliases{RESET} {DIM}({} entries){RESET}\n", aliases.len());
    for (friendly, binary) in &aliases {
        println!("  {BOLD}{friendly}{RESET} {DIM}→{RESET} {binary}");
    }
    println!();
}

/// Resolve a friendly name to a binary name, case-insensitively.
/// Returns None if no alias exists.
pub fn resolve_alias(name: &str) -> Option<String> {
    let aliases = load_aliases();
    aliases.get(&name.to_lowercase()).cloned()
}

/// Return all aliases (friendly_name → binary_name).
pub fn all_aliases() -> BTreeMap<String, String> {
    load_aliases()
}
