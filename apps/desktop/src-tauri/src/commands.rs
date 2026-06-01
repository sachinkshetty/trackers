use crate::backend::{
    DesktopBootstrap, ProfileDiscoveryRequest, discover_profiles as discover_profiles_snapshot,
};
use crate::cleanup::{
    CleanupPreviewRequest, CleanupPreviewResult, preview_cleanup as preview_cleanup_snapshot,
};
use crate::scan::{
    CancellationFlag, ScanProgress, ScanRequest, ScanRunResult, embedded_rule_bundle, run_scan,
    scan_profile,
};
use tauri::{Emitter, State, Window};

#[tauri::command]
pub fn discover_profiles(request: ProfileDiscoveryRequest) -> DesktopBootstrap {
    discover_profiles_snapshot(request)
}

#[tauri::command]
pub fn cancel_scan(scan_state: State<'_, CancellationFlag>) {
    scan_state.cancel();
}

#[tauri::command]
pub fn preview_cleanup(request: CleanupPreviewRequest) -> Result<CleanupPreviewResult, String> {
    preview_cleanup_snapshot(request)
}

#[tauri::command]
pub fn start_scan(
    window: Window,
    scan_state: State<'_, CancellationFlag>,
    request: ScanRequest,
) -> ScanRunResult {
    scan_state.reset();
    let result = run_scan(
        request,
        &embedded_rule_bundle(),
        &scan_state,
        |progress: ScanProgress| {
            let _ = window.emit("scan-progress", progress);
        },
        scan_profile,
    );
    let _ = window.emit("scan-complete", &result);
    result
}
