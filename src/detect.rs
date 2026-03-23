use std::fs;
use std::path::{Path, PathBuf};

use crate::types::LookupResult;
use crate::util::run_cmd;

pub fn resolve_binary(name: &str) -> Option<PathBuf> {
    run_cmd("which", &[name]).map(|p| PathBuf::from(p.trim()))
}

pub fn resolve_symlink(path: &Path) -> Option<PathBuf> {
    fs::read_link(path).ok()
}

pub fn guess_source_from_path(path: &Path) -> &'static str {
    let s = path.to_string_lossy();
    if s.contains("/rustup") {
        return "rustup (cargo)";
    }
    if s.contains("/opt/homebrew/") || s.contains("/usr/local/Cellar/") || s.contains("/Homebrew/")
    {
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

fn detect_source(bin_path: &Path, symlink_target: &Option<PathBuf>) -> &'static str {
    let source_from_original = guess_source_from_path(bin_path);
    if source_from_original != "unknown" {
        return source_from_original;
    }
    let detect_path = symlink_target.as_deref().unwrap_or(bin_path);
    let final_path = fs::canonicalize(detect_path).unwrap_or(detect_path.to_path_buf());
    guess_source_from_path(&final_path)
}

fn lookup_binary_inner(name: &str, fetch_version: bool) -> Option<LookupResult> {
    let bin_path = resolve_binary(name)?;
    let symlink_target = resolve_symlink(&bin_path);
    let source = detect_source(&bin_path, &symlink_target);

    let version = if fetch_version {
        run_cmd(name, &["--version"])
            .or_else(|| run_cmd(name, &["-V"]))
            .or_else(|| run_cmd(name, &["version"]))
            .map(|v| v.lines().next().unwrap_or("").trim().to_string())
            .filter(|v| !v.is_empty())
    } else {
        None
    };

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

pub fn lookup_binary(name: &str) -> Option<LookupResult> {
    lookup_binary_inner(name, true)
}

pub fn lookup_binary_fast(name: &str) -> Option<LookupResult> {
    lookup_binary_inner(name, false)
}
