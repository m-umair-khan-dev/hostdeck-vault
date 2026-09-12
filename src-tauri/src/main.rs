// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let _guard = sentry::init(("https://93c09cda03ee28115f0fcd5462523775@o4511788661669888.ingest.de.sentry.io/4512074160799824", sentry::ClientOptions {
        release: sentry::release_name!(),
        traces_sample_rate: 1.0,
        ..Default::default()
    }));

    ssh_manager_tauri_lib::run()
}
