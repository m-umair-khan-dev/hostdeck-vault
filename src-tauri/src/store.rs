use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

use crate::models::{Server, JumpHost, SSHKey, Settings};

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("I/O Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization Error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Validation Error: {0}")]
    Validation(String),
}

#[derive(Serialize, Deserialize, Default)]
struct StoreData {
    #[serde(default)]
    servers: Vec<Server>,
    #[serde(default)]
    jump_hosts: Vec<JumpHost>,
    #[serde(default)]
    keys: Vec<SSHKey>,
    #[serde(default)]
    settings: Settings,
}

pub struct AppStore {
    pub path: PathBuf,
    pub servers: HashMap<String, Server>,
    pub jump_hosts: HashMap<String, JumpHost>,
    pub keys: HashMap<String, SSHKey>,
    pub settings: Settings,
}

impl AppStore {
    pub fn new() -> Self {
        let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push(".ssh-bootstrap-manager");
        path.push("data.json");
        
        let mut store = AppStore {
            path,
            servers: HashMap::new(),
            jump_hosts: HashMap::new(),
            keys: HashMap::new(),
            settings: Settings::default(),
        };
        let _ = store.load();
        store
    }

    pub fn load(&mut self) -> Result<(), StoreError> {
        if !self.path.exists() {
            return Ok(());
        }
        let data_str = fs::read_to_string(&self.path)?;
        let data: StoreData = serde_json::from_str(&data_str)?;
        
        self.servers = data.servers.into_iter().map(|s| (s.alias.clone(), s)).collect();
        self.jump_hosts = data.jump_hosts.into_iter().map(|j| (j.name.clone(), j)).collect();
        self.keys = data.keys.into_iter().map(|k| (k.name.clone(), k)).collect();
        self.settings = data.settings;
        
        Ok(())
    }

    pub fn save(&self) -> Result<(), StoreError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let mut servers: Vec<Server> = self.servers.values().cloned().collect();
        servers.sort_by(|a, b| a.alias.to_lowercase().cmp(&b.alias.to_lowercase()));
        
        let mut jump_hosts: Vec<JumpHost> = self.jump_hosts.values().cloned().collect();
        jump_hosts.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        
        let mut keys: Vec<SSHKey> = self.keys.values().cloned().collect();
        keys.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        
        let data = StoreData {
            servers,
            jump_hosts,
            keys,
            settings: self.settings.clone(),
        };
        
        let data_str = serde_json::to_string_pretty(&data)?;
        
        // For atomic write, we write to a temporary file in the same directory and rename
        let tmp_path = self.path.with_extension("json.tmp");
        fs::write(&tmp_path, data_str)?;
        fs::rename(&tmp_path, &self.path)?;
        
        Ok(())
    }

    // --- Servers ---

    fn validate_server(&self, server: &Server, original_alias: Option<&str>) -> Result<(), StoreError> {
        if server.alias.trim().is_empty() {
            return Err(StoreError::Validation("Host Alias is required.".into()));
        }
        if server.hostname.trim().is_empty() {
            return Err(StoreError::Validation("Hostname/IP Address is required.".into()));
        }
        if server.username.trim().is_empty() {
            return Err(StoreError::Validation("Username is required.".into()));
        }
        if server.port == 0 {
            return Err(StoreError::Validation("Port must be valid.".into()));
        }
        
        let is_new = original_alias != Some(&server.alias);
        if is_new && self.servers.contains_key(&server.alias) {
            return Err(StoreError::Validation(format!("A server with alias '{}' already exists.", server.alias)));
        }
        if let Some(ref jh) = server.jump_host {
            if !self.jump_hosts.contains_key(jh) {
                return Err(StoreError::Validation(format!("Jump Host '{}' does not exist.", jh)));
            }
        }
        if let Some(ref k) = server.key_name {
            if !self.keys.contains_key(k) {
                return Err(StoreError::Validation(format!("SSH Key '{}' does not exist.", k)));
            }
        }
        Ok(())
    }

    pub fn list_servers(&self) -> Vec<Server> {
        let mut list: Vec<Server> = self.servers.values().cloned().collect();
        list.sort_by(|a, b| a.alias.to_lowercase().cmp(&b.alias.to_lowercase()));
        list
    }

    pub fn list_keys(&self) -> Vec<SSHKey> {
        let mut list: Vec<SSHKey> = self.keys.values().cloned().collect();
        list.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        list
    }

    fn validate_key(&self, key: &SSHKey, original_name: Option<&str>) -> Result<(), StoreError> {
        if key.name.trim().is_empty() {
            return Err(StoreError::Validation("Key Name is required.".into()));
        }
        if key.private_key_path.trim().is_empty() {
            return Err(StoreError::Validation("Private Key Path is required.".into()));
        }
        
        let is_new = original_name != Some(&key.name);
        if is_new && self.keys.contains_key(&key.name) {
            return Err(StoreError::Validation(format!("A key with name '{}' already exists.", key.name)));
        }
        Ok(())
    }

    pub fn add_key(&mut self, key: SSHKey) -> Result<(), StoreError> {
        self.validate_key(&key, None)?;
        self.keys.insert(key.name.clone(), key);
        self.save()
    }

    pub fn delete_key(&mut self, name: &str) -> Result<(), StoreError> {
        if self.keys.remove(name).is_none() {
            return Err(StoreError::Validation(format!("Key '{}' does not exist.", name)));
        }
        self.save()
    }

    pub fn add_server(&mut self, server: Server) -> Result<(), StoreError> {
        self.validate_server(&server, None)?;
        self.servers.insert(server.alias.clone(), server);
        self.save()
    }

    pub fn update_server(&mut self, original_alias: &str, server: Server) -> Result<(), StoreError> {
        if !self.servers.contains_key(original_alias) {
            return Err(StoreError::Validation(format!("Server '{}' does not exist.", original_alias)));
        }
        self.validate_server(&server, Some(original_alias))?;
        
        if original_alias != server.alias {
            self.servers.remove(original_alias);
        }
        self.servers.insert(server.alias.clone(), server);
        self.save()
    }

    pub fn delete_server(&mut self, alias: &str) -> Result<(), StoreError> {
        if self.servers.remove(alias).is_none() {
            return Err(StoreError::Validation(format!("Server '{}' does not exist.", alias)));
        }
        self.save()
    }
}

pub struct AppState(pub Mutex<AppStore>);

// ---------------- Tauri Commands ----------------

#[tauri::command]
pub fn list_servers(state: tauri::State<AppState>) -> Result<Vec<Server>, String> {
    let store = state.0.lock().unwrap();
    Ok(store.list_servers())
}

#[tauri::command]
pub fn add_server(state: tauri::State<AppState>, server: Server) -> Result<(), String> {
    let mut store = state.0.lock().unwrap();
    store.add_server(server).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_server(state: tauri::State<AppState>, alias: String) -> Result<(), String> {
    let mut store = state.0.lock().unwrap();
    store.delete_server(&alias).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_server(state: tauri::State<AppState>, original_alias: String, server: Server) -> Result<(), String> {
    let mut store = state.0.lock().unwrap();
    store.update_server(&original_alias, server).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_keys(state: tauri::State<AppState>) -> Result<Vec<SSHKey>, String> {
    let store = state.0.lock().unwrap();
    Ok(store.list_keys())
}

#[tauri::command]
pub fn add_key(state: tauri::State<AppState>, key: SSHKey) -> Result<(), String> {
    let mut store = state.0.lock().unwrap();
    store.add_key(key).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_key(state: tauri::State<AppState>, name: String) -> Result<(), String> {
    let mut store = state.0.lock().unwrap();
    store.delete_key(&name).map_err(|e| e.to_string())
}
