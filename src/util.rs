use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::types::ToolInfo;

pub fn home_dir() -> PathBuf {
    env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/tmp"))
}

pub fn run_cmd(program: &str, args: &[&str]) -> Option<String> {
    Command::new(program)
        .args(args)
        .stdin(std::process::Stdio::null())
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
}

pub fn read_bin_dir(dir: &Path, source: &str, skip: &[&str]) -> Vec<ToolInfo> {
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
