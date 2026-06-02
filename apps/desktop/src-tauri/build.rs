fn main() {
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "discover_profiles",
            "start_scan",
            "cancel_scan",
            "preview_cleanup",
            "execute_cleanup",
            "settings_snapshot",
        ]),
    ))
    .expect("failed to build tauri desktop manifest");
}
