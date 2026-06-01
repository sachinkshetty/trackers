mod backend;
mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![commands::discover_profiles])
        .run(tauri::generate_context!())
        .expect("error while running trackers desktop application");
}
