use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct SSHKey {
    pub name: String,
    pub private_key_path: String,
    pub public_key_path: String,
    #[serde(default = "default_key_type")]
    pub key_type: String,
}

fn default_key_type() -> String {
    "rsa".to_string()
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct JumpHost {
    pub name: String,
    pub hostname: String,
    pub username: String,
    #[serde(default = "default_port")]
    pub port: u16,
    pub key_name: Option<String>,
    #[serde(default)]
    pub extra_options: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Server {
    pub alias: String,
    pub hostname: String,
    pub username: String,
    #[serde(default = "default_port")]
    pub port: u16,
    pub key_name: Option<String>,
    pub jump_host: Option<String>,
    #[serde(default)]
    pub extra_options: String,
}

fn default_port() -> u16 {
    22
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Settings {
    #[serde(default = "default_theme")]
    pub theme: String,
    pub git_repo_url: Option<String>,
    #[serde(default)]
    pub auto_sync: bool,
    pub last_sync_timestamp: Option<String>,
}

fn default_theme() -> String {
    "dark".to_string()
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            git_repo_url: None,
            auto_sync: false,
            last_sync_timestamp: None,
        }
    }
}
