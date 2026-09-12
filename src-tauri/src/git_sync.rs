use git2::{Cred, RemoteCallbacks, Repository, Signature, PushOptions, FetchOptions};
use keyring::Entry;
use std::path::{Path, PathBuf};
use std::fs;
use thiserror::Error;
use chrono::Utc;

use crate::store::{AppStore, StoreError};

const SERVICE_NAME: &str = "ssh-bootstrap-manager";
const TOKEN_KEY: &str = "github-pat";
const REPO_CONFIG_FILENAME: &str = "data.json";

#[derive(Debug, Error)]
pub enum GitSyncError {
    #[error("Git Error: {0}")]
    Git(#[from] git2::Error),
    #[error("Keyring Error: {0}")]
    Keyring(#[from] keyring::Error),
    #[error("I/O Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Store Error: {0}")]
    Store(#[from] StoreError),
    #[error("Conflict Error: Merge conflict detected while pulling.")]
    Conflict,
    #[error("General Error: {0}")]
    General(String),
}

pub fn default_local_repo_path() -> PathBuf {
    let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push(".ssh-bootstrap-manager");
    path.push("git-repo");
    path
}

pub fn save_token(token: &str) -> Result<(), GitSyncError> {
    let entry = Entry::new(SERVICE_NAME, TOKEN_KEY)?;
    entry.set_password(token)?;
    Ok(())
}

pub fn get_token() -> Result<Option<String>, GitSyncError> {
    let entry = Entry::new(SERVICE_NAME, TOKEN_KEY)?;
    match entry.get_password() {
        Ok(pw) => Ok(Some(pw)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn clear_token() -> Result<(), GitSyncError> {
    let entry = Entry::new(SERVICE_NAME, TOKEN_KEY)?;
    match entry.delete_credential() {
        Ok(_) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(e.into()),
    }
}

fn authed_callbacks() -> Result<RemoteCallbacks<'static>, GitSyncError> {
    let mut callbacks = RemoteCallbacks::new();
    let token = get_token()?.unwrap_or_default();
    
    callbacks.credentials(move |_url, _username_from_url, _allowed_types| {
        Cred::userpass_plaintext(&token, "") // GitHub PAT acts as both or just username/password pair depending on git2, usually username is token and password empty for github PAT in git2
    });
    
    Ok(callbacks)
}

pub fn is_connected(store: &AppStore) -> bool {
    let path = default_local_repo_path();
    store.settings.git_repo_url.is_some() && path.join(".git").exists()
}

pub fn connect_existing(store: &mut AppStore, url: &str) -> Result<(), GitSyncError> {
    let path = default_local_repo_path();
    if path.exists() {
        fs::remove_dir_all(&path)?;
    }
    
    let mut fetch_options = FetchOptions::new();
    fetch_options.remote_callbacks(authed_callbacks()?);
    
    let mut builder = git2::build::RepoBuilder::new();
    builder.fetch_options(fetch_options);
    
    builder.clone(url, &path)?;
    
    store.settings.git_repo_url = Some(url.to_string());
    store.save()?;
    Ok(())
}

pub fn disconnect(store: &mut AppStore) -> Result<(), GitSyncError> {
    let path = default_local_repo_path();
    if path.exists() {
        fs::remove_dir_all(&path)?;
    }
    store.settings.git_repo_url = None;
    store.settings.last_sync_timestamp = None;
    store.save()?;
    Ok(())
}

pub fn push_changes(store: &mut AppStore, commit_message: &str) -> Result<bool, GitSyncError> {
    let path = default_local_repo_path();
    let repo = Repository::open(&path)?;
    
    // Copy current state to the repo
    let config_dest = path.join(REPO_CONFIG_FILENAME);
    let original_path = store.path.clone();
    
    // We hackily modify the path temporarily to save to the repo, then restore it
    let mut repo_store = AppStore::new();
    repo_store.path = config_dest.clone();
    repo_store.servers = store.servers.clone();
    repo_store.jump_hosts = store.jump_hosts.clone();
    repo_store.keys = store.keys.clone();
    repo_store.settings = store.settings.clone();
    repo_store.save()?;
    
    let mut index = repo.index()?;
    index.add_path(Path::new(REPO_CONFIG_FILENAME))?;
    
    if index.is_empty() {
        return Ok(false); // Nothing to push
    }
    
    let oid = index.write_tree()?;
    let tree = repo.find_tree(oid)?;
    
    let sig = Signature::now("SSH Bootstrap Manager", "app@localhost")?;
    
    // Get HEAD
    let parent_commit = if let Ok(head) = repo.head() {
        if let Ok(commit) = head.peel_to_commit() {
            Some(commit)
        } else {
            None
        }
    } else {
        None
    };
    
    let mut parents = Vec::new();
    if let Some(ref p) = parent_commit {
        parents.push(p);
    }
    
    repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        commit_message,
        &tree,
        &parents,
    )?;
    
    let mut remote = repo.find_remote("origin")?;
    let mut push_opts = PushOptions::new();
    push_opts.remote_callbacks(authed_callbacks()?);
    
    let head = repo.head()?;
    let head_name = head.name().unwrap();
    
    remote.push(&[format!("+{}:{}", head_name, head_name)], Some(&mut push_opts))?;
    
    store.settings.last_sync_timestamp = Some(Utc::now().to_rfc3339());
    store.save()?;
    
    Ok(true)
}

// ---------------- Tauri Commands ----------------

#[tauri::command]
pub fn check_git_connected(state: tauri::State<crate::store::AppState>) -> Result<bool, String> {
    let store = state.0.lock().unwrap();
    Ok(is_connected(&store))
}

#[tauri::command]
pub fn git_connect(state: tauri::State<crate::store::AppState>, url: String, token: String) -> Result<(), String> {
    save_token(&token).map_err(|e| e.to_string())?;
    let mut store = state.0.lock().unwrap();
    connect_existing(&mut store, &url).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn git_push(state: tauri::State<crate::store::AppState>, commit_message: String) -> Result<bool, String> {
    let mut store = state.0.lock().unwrap();
    push_changes(&mut store, &commit_message).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn git_disconnect(state: tauri::State<crate::store::AppState>) -> Result<(), String> {
    let mut store = state.0.lock().unwrap();
    disconnect(&mut store).map_err(|e| e.to_string())
}
