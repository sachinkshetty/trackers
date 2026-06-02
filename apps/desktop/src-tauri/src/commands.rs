use crate::backend::{
    DesktopBootstrap, ProfileDiscoveryRequest, discover_profiles as discover_profiles_snapshot,
};
use crate::cleanup::{
    CleanupExecuteRequest, CleanupExecuteResult, CleanupPreviewRequest, CleanupPreviewResult,
    execute_cleanup as execute_cleanup_snapshot, preview_cleanup as preview_cleanup_snapshot,
};
use crate::audit::{
    CleanupAuditHistory, clear_cleanup_audit_history as clear_cleanup_audit_history_snapshot,
    cleanup_audit_history as cleanup_audit_history_snapshot,
};
use crate::scan::{
    CancellationFlag, ScanProgress, ScanRequest, ScanRunResult, embedded_rule_bundle, run_scan,
    scan_profile,
};
use crate::settings::{DesktopSettingsSnapshot, settings_snapshot as settings_snapshot_snapshot};
use crate::state::AppState;
use tauri::{Emitter, State, Window};

#[tauri::command]
pub fn discover_profiles(
    state: State<'_, AppState>,
    request: ProfileDiscoveryRequest,
) -> DesktopBootstrap {
    let snapshot = discover_profiles_snapshot(request);
    state.replace_discovery(snapshot.clone());
    snapshot
}

#[tauri::command]
pub fn cancel_scan(scan_state: State<'_, CancellationFlag>) {
    scan_state.cancel();
}

#[tauri::command]
pub fn preview_cleanup(
    state: State<'_, AppState>,
    request: CleanupPreviewRequest,
) -> Result<CleanupPreviewResult, String> {
    preview_cleanup_snapshot(&state, request)
}

#[tauri::command]
pub fn execute_cleanup(
    state: State<'_, AppState>,
    request: CleanupExecuteRequest,
) -> Result<CleanupExecuteResult, String> {
    execute_cleanup_snapshot(&state, request)
}

#[tauri::command]
pub fn settings_snapshot() -> DesktopSettingsSnapshot {
    settings_snapshot_snapshot()
}

#[tauri::command]
pub fn cleanup_audit_history() -> Result<CleanupAuditHistory, String> {
    cleanup_audit_history_snapshot()
}

#[tauri::command]
pub fn clear_cleanup_audit_history() -> Result<(), String> {
    clear_cleanup_audit_history_snapshot()
}

#[tauri::command]
pub fn start_scan(
    window: Window,
    scan_state: State<'_, CancellationFlag>,
    app_state: State<'_, AppState>,
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
    app_state.replace_scan(result.clone());
    let _ = window.emit("scan-complete", &result);
    result
}
