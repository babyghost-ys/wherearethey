use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::types::ToolInfo;
use crate::util::{home_dir, read_bin_dir, run_cmd};

// ── Homebrew ─────────────────────────────────────────────────────────

fn list_brew() -> Vec<ToolInfo> {
    let prefix = run_cmd("brew", &["--prefix"])
        .unwrap_or_else(|| "/opt/homebrew".into())
        .trim()
        .to_string();
    let bin_dir = PathBuf::from(&prefix).join("bin");

    if !bin_dir.exists() {
        return vec![];
    }

    let cellar = format!("{prefix}/Cellar/");

    let entries = match fs::read_dir(&bin_dir) {
        Ok(e) => e,
        Err(_) => return vec![],
    };

    entries
        .filter_map(|e| e.ok())
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().to_string();
            if name == "brew" || name.starts_with('.') {
                return None;
            }
            let path = entry.path();
            let target = fs::read_link(&path).ok()?;
            let target_str = target.to_string_lossy();
            if target_str.contains("/Cellar/") || target_str.contains("../Cellar/") {
                let canon = fs::canonicalize(&path).unwrap_or(path.clone());
                let canon_str = canon.to_string_lossy().to_string();
                let version = canon_str.strip_prefix(&cellar).and_then(|rest| {
                    let parts: Vec<&str> = rest.splitn(3, '/').collect();
                    if parts.len() >= 2 {
                        Some(format!("{} {}", parts[0], parts[1]))
                    } else {
                        None
                    }
                });
                Some(ToolInfo {
                    name,
                    path: path.to_string_lossy().to_string(),
                    source: "brew".into(),
                    version,
                })
            } else {
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
        .skip(1)
        .filter(|l| !l.is_empty())
        .filter_map(|line| {
            let pkg_name = Path::new(line.trim())
                .file_name()?
                .to_string_lossy()
                .to_string();
            if pkg_name == "npm" || pkg_name == "corepack" {
                return None;
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
            let pkg_name = Path::new(line.trim())
                .file_name()?
                .to_string_lossy()
                .to_string();
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
            current_pkg = Some(line.trim_end_matches(':').to_string());
        } else if line.starts_with("    ") {
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
        .skip(1)
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

// ── Scan all package managers ────────────────────────────────────────

pub fn scan_all() -> Vec<ToolInfo> {
    scan_all_inner(false)
}

pub fn scan_all_quiet() -> Vec<ToolInfo> {
    scan_all_inner(true)
}

fn scan_all_inner(quiet: bool) -> Vec<ToolInfo> {
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
        if !quiet {
            eprint!("  scanning {name}...");
        }
        let tools = list_fn();
        if !quiet {
            eprintln!(" {} found", tools.len());
        }
        all_tools.extend(tools);
    }

    all_tools
}

pub fn get_all_path_binaries() -> BTreeSet<String> {
    let path_var = env::var("PATH").unwrap_or_default();
    let mut binaries = BTreeSet::new();

    for dir in path_var.split(':') {
        let dir_path = Path::new(dir);
        if !dir_path.exists() {
            continue;
        }
        if dir == "/usr/bin" || dir == "/bin" || dir == "/sbin" || dir == "/usr/sbin" {
            continue;
        }
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
