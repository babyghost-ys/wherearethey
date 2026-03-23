use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct ToolInfo {
    pub name: String,
    pub path: String,
    pub source: String,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LookupResult {
    pub binary: String,
    pub resolved_path: String,
    pub symlink_target: Option<String>,
    pub source: String,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallEvent {
    pub timestamp: String,
    pub source: String,
    pub action: String,
    pub packages: Vec<String>,
}
