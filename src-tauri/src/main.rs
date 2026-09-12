// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let mut options = sentry::ClientOptions::default();
    options.release = sentry::release_name!();
    let _guard = sentry::init((
        "https://93c09cda03ee28115f0fcd5462523775@o4511788661669888.ingest.de.sentry.io/4512074160799824",
        options,
    ));

    ssh_manager_tauri_lib::run()
}
