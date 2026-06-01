mod backend;
mod commands;
mod scan;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(scan::CancellationFlag::new())
        .invoke_handler(tauri::generate_handler![
            commands::discover_profiles,
            commands::start_scan,
            commands::cancel_scan
        ])
        .run(tauri::generate_context!())
        .expect("error while running trackers desktop application");
}
