mod backend;
mod cleanup;
mod commands;
mod scan;
mod settings;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(scan::CancellationFlag::new())
        .invoke_handler(tauri::generate_handler![
            commands::discover_profiles,
            commands::start_scan,
            commands::cancel_scan,
            commands::preview_cleanup,
            commands::settings_snapshot
        ])
        .run(tauri::generate_context!())
        .expect("error while running trackers desktop application");
}
