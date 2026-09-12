pub mod models;
pub mod store;
pub mod ssh_config;
pub mod git_sync;
pub mod deploy;

use std::sync::Mutex;
use store::{AppState, AppStore, add_server, list_servers, update_server, delete_server, list_keys, add_key, delete_key};
use git_sync::{check_git_connected, git_connect, git_push, git_disconnect};
use deploy::{tauri_deploy_key, tauri_test_passwordless};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let store = AppStore::new();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState(Mutex::new(store)))
        .invoke_handler(tauri::generate_handler![
            list_servers,
            add_server,
            update_server,
            delete_server,
            list_keys,
            add_key,
            delete_key,
            check_git_connected,
            git_connect,
            git_push,
            git_disconnect,
            tauri_deploy_key,
            tauri_test_passwordless
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
