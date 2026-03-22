use clap::Parser;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

// ── CLI ──────────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(
    name = "wherearethey",
    about = "Find where your CLI tools were installed from",
    version
)]
struct Cli {
    /// Binary name to look up (e.g. "ffmpeg", "rg", "node")
    binary: Option<String>,

    /// List all detected CLI tools grouped by source
    #[arg(long)]
    all: bool,

    /// Show binaries in PATH that no package manager claims
    #[arg(long)]
    orphans: bool,

    /// Output as JSON
    #[arg(long)]
    json: bool,
}

// ── Data types ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
struct ToolInfo {
    name: String,
    path: String,
    source: String,
    version: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct LookupResult {
    binary: String,
    resolved_path: String,
    symlink_target: Option<String>,
    source: String,
    version: Option<String>,
}

// ── Helpers ──────────────────────────────────────────────────────────

fn home_dir() -> PathBuf {
    env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/tmp"))
}

fn run_cmd(program: &str, args: &[&str]) -> Option<String> {
    Command::new(program)
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
}

// ── Homebrew ─────────────────────────────────────────────────────────

fn list_brew() -> Vec<ToolInfo> {
    let output = match run_cmd("brew", &["list", "--formula", "-1"]) {
        Some(o) => o,
        None => return vec![],
    };
    let prefix = run_cmd("brew", &["--prefix"])
        .unwrap_or_else(|| "/opt/homebrew".into())
        .trim()
        .to_string();
    let bin_dir = format!("{prefix}/bin");

    output
        .lines()
        .filter(|l| !l.is_empty())
        .filter_map(|formula| {
            let bin_path = format!("{bin_dir}/{formula}");
            if Path::new(&bin_path).exists() {
                Some(ToolInfo {
                    name: formula.to_string(),
                    path: bin_path,
                    source: "brew".into(),
                    version: run_cmd("brew", &["list", "--versions", formula])
                        .map(|v| v.trim().to_string()),
                })
            } else {
                // Some formulae install binaries with different names
                // List files in the cellar linked to bin
                if let Some(files) = run_cmd("brew", &["list", "--formula", formula]) {
                    for line in files.lines() {
                        if line.contains("/bin/") {
                            let p = Path::new(line.trim());
                            if let Some(fname) = p.file_name() {
                                return Some(ToolInfo {
                                    name: fname.to_string_lossy().to_string(),
                                    path: format!("{bin_dir}/{}", fname.to_string_lossy()),
                                    source: "brew".into(),
                                    version: run_cmd("brew", &["list", "--versions", formula])
                                        .map(|v| v.trim().to_string()),
                                });
                            }
                        }
                    }
                }
                None
            }
        })
        .collect()
}

// ── npm (global) ─────────────────────────────────────────────────────

fn list_npm() -> Vec<ToolInfo> {
    let output = match run_cmd("npm", &["list", "-g", "--depth=0", "--parseable"]) {
        Some(o) => o,
        None => return vec![],
    };
    let bin_dir = run_cmd("npm", &["bin", "-g"])
        .unwrap_or_default()
        .trim()
        .to_string();

    output
        .lines()
        .skip(1) // first line is the prefix
        .filter(|l| !l.is_empty())
        .filter_map(|line| {
            let pkg_name = Path::new(line.trim()).file_name()?.to_string_lossy().to_string();
            if pkg_name == "npm" || pkg_name == "corepack" {
                return None; // skip npm itself
            }
            Some(ToolInfo {
                name: pkg_name.clone(),
                path: format!("{bin_dir}/{pkg_name}"),
                source: "npm".into(),
                version: None,
            })
        })
        .collect()
}

// ── pnpm (global) ────────────────────────────────────────────────────

fn list_pnpm() -> Vec<ToolInfo> {
    let output = match run_cmd("pnpm", &["list", "-g", "--depth=0", "--parseable"]) {
        Some(o) => o,
        None => return vec![],
    };

    output
        .lines()
        .skip(1)
        .filter(|l| !l.is_empty())
        .filter_map(|line| {
            let pkg_name = Path::new(line.trim()).file_name()?.to_string_lossy().to_string();
            Some(ToolInfo {
                name: pkg_name.clone(),
                path: line.trim().to_string(),
                source: "pnpm".into(),
                version: None,
            })
        })
        .collect()
}

// ── Bun (global) ─────────────────────────────────────────────────────

fn list_bun() -> Vec<ToolInfo> {
    let bin_dir = home_dir().join(".bun/bin");
    if !bin_dir.exists() {
        return vec![];
    }
    read_bin_dir(&bin_dir, "bun", &["bun", "bunx"])
}

// ── Deno ─────────────────────────────────────────────────────────────

fn list_deno() -> Vec<ToolInfo> {
    let bin_dir = home_dir().join(".deno/bin");
    if !bin_dir.exists() {
        return vec![];
    }
    read_bin_dir(&bin_dir, "deno", &["deno"])
}

// ── Cargo (Rust) ─────────────────────────────────────────────────────

fn list_cargo() -> Vec<ToolInfo> {
    let output = match run_cmd("cargo", &["install", "--list"]) {
        Some(o) => o,
        None => return vec![],
    };
    let bin_dir = home_dir().join(".cargo/bin");
    let mut tools = Vec::new();
    let mut current_pkg: Option<String> = None;

    for line in output.lines() {
        if !line.starts_with(' ') && line.contains(' ') {
            // Package line: "ripgrep v14.1.0:"
            current_pkg = Some(line.trim_end_matches(':').to_string());
        } else if line.starts_with("    ") {
            // Binary line: "    rg"
            let bin_name = line.trim().to_string();
            tools.push(ToolInfo {
                name: bin_name.clone(),
                path: bin_dir.join(&bin_name).to_string_lossy().to_string(),
                source: "cargo".into(),
                version: current_pkg.clone(),
            });
        }
    }
    tools
}

// ── Go ───────────────────────────────────────────────────────────────

fn list_go() -> Vec<ToolInfo> {
    let gobin = env::var("GOBIN")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            env::var("GOPATH")
                .map(|p| PathBuf::from(p).join("bin"))
                .unwrap_or_else(|_| home_dir().join("go/bin"))
        });
    if !gobin.exists() {
        return vec![];
    }
    read_bin_dir(&gobin, "go", &[])
}

// ── pipx ─────────────────────────────────────────────────────────────

fn list_pipx() -> Vec<ToolInfo> {
    let output = match run_cmd("pipx", &["list", "--short"]) {
        Some(o) => o,
        None => return vec![],
    };
    let bin_dir = home_dir().join(".local/bin");

    output
        .lines()
        .filter(|l| !l.is_empty())
        .map(|line| {
            // Format: "package 1.2.3"
            let parts: Vec<&str> = line.splitn(2, ' ').collect();
            let name = parts[0].to_string();
            let version = parts.get(1).map(|v| v.to_string());
            ToolInfo {
                name: name.clone(),
                path: bin_dir.join(&name).to_string_lossy().to_string(),
                source: "pipx".into(),
                version,
            }
        })
        .collect()
}

// ── uv tool ──────────────────────────────────────────────────────────

fn list_uv() -> Vec<ToolInfo> {
    let output = match run_cmd("uv", &["tool", "list"]) {
        Some(o) => o,
        None => return vec![],
    };
    let bin_dir = home_dir().join(".local/bin");

    output
        .lines()
        .filter(|l| !l.is_empty() && !l.starts_with('-'))
        .filter_map(|line| {
            // Format: "ruff v0.8.0" or "ruff v0.8.0 (python 3.12)"
            let parts: Vec<&str> = line.splitn(2, ' ').collect();
            if parts.is_empty() {
                return None;
            }
            let name = parts[0].to_string();
            let version = parts.get(1).map(|v| v.to_string());
            Some(ToolInfo {
                name: name.clone(),
                path: bin_dir.join(&name).to_string_lossy().to_string(),
                source: "uv".into(),
                version,
            })
        })
        .collect()
}

// ── pip (user) ───────────────────────────────────────────────────────

fn list_pip() -> Vec<ToolInfo> {
    let output = match run_cmd("pip3", &["list", "--user", "--format=json"]) {
        Some(o) => o,
        None => return vec![],
    };
    let packages: Vec<serde_json::Value> = serde_json::from_str(&output).unwrap_or_default();

    // Find the user bin dir
    let user_bin = run_cmd("python3", &["-m", "site", "--user-base"])
        .map(|p| PathBuf::from(p.trim()).join("bin"))
        .unwrap_or_else(|| home_dir().join("Library/Python/3.12/bin"));

    packages
        .iter()
        .filter_map(|pkg| {
            let name = pkg["name"].as_str()?.to_string();
            let version = pkg["version"].as_str().map(|v| v.to_string());
            let bin_path = user_bin.join(&name);
            if bin_path.exists() {
                Some(ToolInfo {
                    name,
                    path: bin_path.to_string_lossy().to_string(),
                    source: "pip".into(),
                    version,
                })
            } else {
                None
            }
        })
        .collect()
}

// ── Ruby gems ────────────────────────────────────────────────────────

fn list_gem() -> Vec<ToolInfo> {
    let output = match run_cmd("gem", &["list", "--local", "--no-details"]) {
        Some(o) => o,
        None => return vec![],
    };
    let gem_bin = run_cmd("gem", &["environment", "gemdir"])
        .map(|p| PathBuf::from(p.trim()).join("bin"))
        .unwrap_or_default();

    output
        .lines()
        .filter(|l| !l.is_empty())
        .filter_map(|line| {
            let name = line.split(' ').next()?.to_string();
            let bin_path = gem_bin.join(&name);
            if bin_path.exists() {
                Some(ToolInfo {
                    name,
                    path: bin_path.to_string_lossy().to_string(),
                    source: "gem".into(),
                    version: line
                        .split('(')
                        .nth(1)
                        .map(|v| v.trim_end_matches(')').to_string()),
                })
            } else {
                None
            }
        })
        .collect()
}

// ── Composer (PHP) ───────────────────────────────────────────────────

fn list_composer() -> Vec<ToolInfo> {
    let bin_dir = home_dir().join(".composer/vendor/bin");
    if !bin_dir.exists() {
        let alt = home_dir().join(".config/composer/vendor/bin");
        if !alt.exists() {
            return vec![];
        }
        return read_bin_dir(&alt, "composer", &[]);
    }
    read_bin_dir(&bin_dir, "composer", &[])
}

// ── .NET tools ───────────────────────────────────────────────────────

fn list_dotnet() -> Vec<ToolInfo> {
    let bin_dir = home_dir().join(".dotnet/tools");
    if !bin_dir.exists() {
        return vec![];
    }
    read_bin_dir(&bin_dir, "dotnet", &["dotnet"])
}

// ── Nix ──────────────────────────────────────────────────────────────

fn list_nix() -> Vec<ToolInfo> {
    let bin_dir = home_dir().join(".nix-profile/bin");
    if !bin_dir.exists() {
        return vec![];
    }
    read_bin_dir(&bin_dir, "nix", &[])
}

// ── MacPorts ─────────────────────────────────────────────────────────

fn list_macports() -> Vec<ToolInfo> {
    let output = match run_cmd("port", &["installed"]) {
        Some(o) => o,
        None => return vec![],
    };

    output
        .lines()
        .skip(1) // "The following ports are currently installed:"
        .filter(|l| !l.is_empty())
        .filter_map(|line| {
            let trimmed = line.trim();
            let parts: Vec<&str> = trimmed.splitn(2, ' ').collect();
            let name = parts[0].to_string();
            let bin_path = format!("/opt/local/bin/{name}");
            if Path::new(&bin_path).exists() {
                Some(ToolInfo {
                    name,
                    path: bin_path,
                    source: "macports".into(),
                    version: parts.get(1).map(|v| v.trim().to_string()),
                })
            } else {
                None
            }
        })
        .collect()
}

// ── Conda ────────────────────────────────────────────────────────────

fn list_conda() -> Vec<ToolInfo> {
    let output = match run_cmd("conda", &["list", "--json"]) {
        Some(o) => o,
        None => return vec![],
    };
    let packages: Vec<serde_json::Value> = serde_json::from_str(&output).unwrap_or_default();
    let conda_bin = run_cmd("conda", &["info", "--base"])
        .map(|p| PathBuf::from(p.trim()).join("bin"))
        .unwrap_or_default();

    packages
        .iter()
        .filter_map(|pkg| {
            let name = pkg["name"].as_str()?.to_string();
            let bin_path = conda_bin.join(&name);
            if bin_path.exists() {
                Some(ToolInfo {
                    name,
                    path: bin_path.to_string_lossy().to_string(),
                    source: "conda".into(),
                    version: pkg["version"].as_str().map(|v| v.to_string()),
                })
            } else {
                None
            }
        })
        .collect()
}

// ── mise ─────────────────────────────────────────────────────────────

fn list_mise() -> Vec<ToolInfo> {
    let output = match run_cmd("mise", &["list", "--current", "--json"]) {
        Some(o) => o,
        None => return vec![],
    };
    let data: serde_json::Value = serde_json::from_str(&output).unwrap_or_default();
    let mut tools = Vec::new();

    if let Some(obj) = data.as_object() {
        for (tool_name, versions) in obj {
            if let Some(arr) = versions.as_array() {
                for entry in arr {
                    let version = entry["version"].as_str().map(|v| v.to_string());
                    let install_path = entry["install_path"]
                        .as_str()
                        .unwrap_or_default()
                        .to_string();
                    tools.push(ToolInfo {
                        name: tool_name.clone(),
                        path: install_path,
                        source: "mise".into(),
                        version,
                    });
                }
            }
        }
    }
    tools
}

// ── gh extensions ────────────────────────────────────────────────────

fn list_gh_extensions() -> Vec<ToolInfo> {
    let output = match run_cmd("gh", &["extension", "list"]) {
        Some(o) => o,
        None => return vec![],
    };

    output
        .lines()
        .filter(|l| !l.is_empty())
        .filter_map(|line| {
            let parts: Vec<&str> = line.split('\t').collect();
            let name = parts.first()?.trim().to_string();
            Some(ToolInfo {
                name: name.clone(),
                path: home_dir()
                    .join(".local/share/gh/extensions")
                    .join(&name)
                    .to_string_lossy()
                    .to_string(),
                source: "gh-extension".into(),
                version: parts.get(1).map(|v| v.trim().to_string()),
            })
        })
        .collect()
}


// ── Helpers ──────────────────────────────────────────────────────────

fn read_bin_dir(dir: &Path, source: &str, skip: &[&str]) -> Vec<ToolInfo> {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return vec![],
    };

    entries
        .filter_map(|e| e.ok())
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().to_string();
            if skip.contains(&name.as_str()) || name.starts_with('.') {
                return None;
            }
            // Only include executable files
            let path = entry.path();
            if path.is_file() || path.is_symlink() {
                Some(ToolInfo {
                    name,
                    path: path.to_string_lossy().to_string(),
                    source: source.to_string(),
                    version: None,
                })
            } else {
                None
            }
        })
        .collect()
}

fn resolve_binary(name: &str) -> Option<PathBuf> {
    run_cmd("which", &[name]).map(|p| PathBuf::from(p.trim()))
}

fn resolve_symlink(path: &Path) -> Option<PathBuf> {
    fs::read_link(path).ok()
}

fn guess_source_from_path(path: &Path) -> &'static str {
    let s = path.to_string_lossy();
    // Rustup-managed tools live in .cargo/bin but symlink to rustup
    if s.contains("/rustup") {
        return "rustup (cargo)";
    }
    if s.contains("/opt/homebrew/") || s.contains("/usr/local/Cellar/") || s.contains("/Homebrew/") {
        "brew"
    } else if s.contains("/.cargo/bin") {
        "cargo"
    } else if s.contains("/go/bin") {
        "go"
    } else if s.contains("/.bun/bin") || s.contains("/.bun/install") {
        "bun"
    } else if s.contains("/.deno/bin") {
        "deno"
    } else if s.contains("/.nvm/") || s.contains("/nodejs/") {
        "npm (via nvm)"
    } else if s.contains("/.local/share/mise/") {
        "mise"
    } else if s.contains("/.asdf/") {
        "asdf"
    } else if s.contains("/.nix-profile/") || s.contains("/nix/store/") {
        "nix"
    } else if s.contains("/opt/local/") {
        "macports"
    } else if s.contains("/.pipx/") || s.contains("/pipx/venvs/") {
        "pipx"
    } else if s.contains("/.local/bin") {
        "pipx/uv/manual (~/.local/bin)"
    } else if s.contains("/Library/Python/") {
        "pip"
    } else if s.contains("/.gem/") || s.contains("/ruby/gems/") {
        "gem"
    } else if s.contains("/.composer/") {
        "composer"
    } else if s.contains("/.dotnet/tools") {
        "dotnet"
    } else if s.contains("/.mint/bin") {
        "mint"
    } else if s.contains("/.proto/") {
        "proto"
    } else if s.contains("/.sdkman/") {
        "sdkman"
    } else if s.contains("/.ghcup/") {
        "ghcup"
    } else if s.contains("/.pkgx/") {
        "pkgx"
    } else if s.contains("/miniforge") || s.contains("/miniconda") || s.contains("/anaconda") {
        "conda"
    } else if s.contains("/Library/Developer/CommandLineTools/") {
        "xcode-clt"
    } else if s.starts_with("/usr/bin") {
        "macos-system"
    } else {
        "unknown"
    }
}

// ── Scan all package managers ────────────────────────────────────────

fn scan_all() -> Vec<ToolInfo> {
    let mut all_tools = Vec::new();

    let managers: Vec<(&str, fn() -> Vec<ToolInfo>)> = vec![
        ("brew", list_brew as fn() -> Vec<ToolInfo>),
        ("npm", list_npm),
        ("pnpm", list_pnpm),
        ("bun", list_bun),
        ("deno", list_deno),
        ("cargo", list_cargo),
        ("go", list_go),
        ("pipx", list_pipx),
        ("uv", list_uv),
        ("pip", list_pip),
        ("gem", list_gem),
        ("composer", list_composer),
        ("dotnet", list_dotnet),
        ("nix", list_nix),
        ("macports", list_macports),
        ("conda", list_conda),
        ("mise", list_mise),
        ("gh-extension", list_gh_extensions),
    ];

    for (name, list_fn) in &managers {
        eprint!("  scanning {name}...");
        let tools = list_fn();
        eprintln!(" {} found", tools.len());
        all_tools.extend(tools);
    }

    all_tools
}

fn get_all_path_binaries() -> BTreeSet<String> {
    let path_var = env::var("PATH").unwrap_or_default();
    let mut binaries = BTreeSet::new();

    for dir in path_var.split(':') {
        let dir_path = Path::new(dir);
        if !dir_path.exists() {
            continue;
        }
        // Skip system dirs for orphan detection
        if dir == "/usr/bin" || dir == "/bin" || dir == "/sbin" || dir == "/usr/sbin" {
            continue;
        }
        // Skip Xcode CLT
        if dir.contains("/Library/Developer/CommandLineTools") {
            continue;
        }
        if let Ok(entries) = fs::read_dir(dir_path) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if !name.starts_with('.') {
                        binaries.insert(name.to_string());
                    }
                }
            }
        }
    }
    binaries
}

// ── Lookup a single binary ───────────────────────────────────────────

fn lookup_binary(name: &str) -> Option<LookupResult> {
    let bin_path = resolve_binary(name)?;
    let symlink_target = resolve_symlink(&bin_path);

    // Try source detection on the original path first, then the resolved path
    let source_from_original = guess_source_from_path(&bin_path);
    let source = if source_from_original != "unknown" {
        source_from_original
    } else {
        let detect_path = symlink_target.as_ref().unwrap_or(&bin_path);
        let final_path = fs::canonicalize(detect_path).unwrap_or(detect_path.clone());
        guess_source_from_path(&final_path)
    };

    let version = run_cmd(name, &["--version"])
        .or_else(|| run_cmd(name, &["-V"]))
        .or_else(|| run_cmd(name, &["version"]))
        .map(|v| v.lines().next().unwrap_or("").trim().to_string())
        .filter(|v| !v.is_empty());

    Some(LookupResult {
        binary: name.to_string(),
        resolved_path: bin_path.to_string_lossy().to_string(),
        symlink_target: symlink_target.map(|p| {
            fs::canonicalize(&p)
                .unwrap_or(p)
                .to_string_lossy()
                .to_string()
        }),
        source: source.to_string(),
        version,
    })
}

// ── Output formatting ────────────────────────────────────────────────

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const CYAN: &str = "\x1b[36m";
const RED: &str = "\x1b[31m";
const MAGENTA: &str = "\x1b[35m";

fn source_colour(source: &str) -> &'static str {
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

fn print_lookup(result: &LookupResult) {
    let colour = source_colour(&result.source);
    println!(
        "\n  {BOLD}{}{RESET}",
        result.binary
    );
    println!(
        "  {DIM}path:{RESET}    {}",
        result.resolved_path
    );
    if let Some(ref target) = result.symlink_target {
        println!("  {DIM}target:{RESET}  {target}");
    }
    println!(
        "  {DIM}source:{RESET}  {colour}{}{RESET}",
        result.source
    );
    if let Some(ref ver) = result.version {
        println!("  {DIM}version:{RESET} {ver}");
    }
    println!();
}

fn print_all(tools: &[ToolInfo]) {
    let mut by_source: BTreeMap<String, Vec<&ToolInfo>> = BTreeMap::new();
    for tool in tools {
        by_source
            .entry(tool.source.clone())
            .or_default()
            .push(tool);
    }

    for (source, tools) in &by_source {
        let colour = source_colour(source);
        println!(
            "\n  {colour}{BOLD}{source}{RESET} {DIM}({} tools){RESET}",
            tools.len()
        );
        for tool in tools {
            let ver = tool
                .version
                .as_deref()
                .unwrap_or("");
            if ver.is_empty() {
                println!("    {}", tool.name);
            } else {
                println!("    {} {DIM}{ver}{RESET}", tool.name);
            }
        }
    }
    println!();
}

fn print_orphans(orphans: &[LookupResult]) {
    if orphans.is_empty() {
        println!("\n  {GREEN}No orphan binaries found. Everything is claimed.{RESET}\n");
        return;
    }
    println!(
        "\n  {YELLOW}{BOLD}Orphans{RESET} {DIM}({} binaries no package manager claims){RESET}\n",
        orphans.len()
    );
    for orphan in orphans {
        let target_info = orphan
            .symlink_target
            .as_deref()
            .map(|t| format!(" {DIM}-> {t}{RESET}"))
            .unwrap_or_default();
        println!(
            "    {YELLOW}{}{RESET}  {DIM}{}{RESET}{target_info}",
            orphan.binary, orphan.resolved_path
        );
    }
    println!();
}

// ── Main ─────────────────────────────────────────────────────────────

fn main() {
    let cli = Cli::parse();

    // Single binary lookup
    if let Some(ref name) = cli.binary {
        if !cli.all && !cli.orphans {
            match lookup_binary(name) {
                Some(result) => {
                    if cli.json {
                        println!("{}", serde_json::to_string_pretty(&result).unwrap());
                    } else {
                        print_lookup(&result);
                    }
                }
                None => {
                    eprintln!(
                        "  {RED}'{name}' not found in PATH{RESET}"
                    );
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

    // --orphans: find unclaimed binaries
    if cli.orphans {
        eprintln!("\n{BOLD}Scanning package managers...{RESET}\n");
        let tools = scan_all();
        let claimed: BTreeSet<String> = tools.iter().map(|t| t.name.clone()).collect();
        let all_binaries = get_all_path_binaries();

        let orphans: Vec<LookupResult> = all_binaries
            .iter()
            .filter(|b| !claimed.contains(*b))
            .filter_map(|b| lookup_binary(b))
            .filter(|r| r.source == "unknown")
            .collect();

        if cli.json {
            println!("{}", serde_json::to_string_pretty(&orphans).unwrap());
        } else {
            print_orphans(&orphans);
        }
        return;
    }

    // No args — print help
    eprintln!("{BOLD}wherearethey{RESET} — find where your CLI tools were installed from\n");
    eprintln!("Usage:");
    eprintln!("  wherearethey <binary>    Look up a specific tool");
    eprintln!("  wherearethey --all       List all tools by source");
    eprintln!("  wherearethey --orphans   Find unclaimed binaries");
    eprintln!("  wherearethey --json      Output as JSON\n");
    eprintln!("Examples:");
    eprintln!("  wherearethey ffmpeg");
    eprintln!("  wherearethey rg");
    eprintln!("  wherearethey --all --json\n");
}
