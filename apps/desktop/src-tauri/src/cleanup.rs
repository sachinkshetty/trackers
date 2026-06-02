use scanner_core::{
    AggressiveConfirmation, BrowserCloser, BrowserFamily, CleanupExecutionResult, CleanupMode,
    CleanupPlan, CleanupTarget, Finding, LockResolution, PreflightResult, ResourceLockProbe,
    execute_cleanup as execute_cleanup_plan, plan_aggressive_cleanup, plan_balanced_cleanup,
    plan_review_cleanup, preflight_locked_resources,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
    process::Command,
};

use crate::backend::DesktopBootstrap;
use crate::scan::ScanRunResult;
use crate::state::AppState;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupPreviewRequest {
    pub mode: CleanupMode,
    pub selected_finding_ids: Vec<String>,
    pub aggressive_confirmed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupPreviewResult {
    pub plan: CleanupPlan,
    pub locked_action_ids: Vec<String>,
    pub locked_profiles: Vec<CleanupLockedProfile>,
    pub requires_confirmation: bool,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupLockedProfile {
    pub browser: BrowserFamily,
    pub profile_name: String,
    pub profile_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CleanupLockResolution {
    RetryAfterManualClose,
    SkipLocked,
    RequestAutomaticClose { confirmed: bool },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupExecuteRequest {
    pub preview: CleanupPreviewResult,
    pub lock_resolution: CleanupLockResolution,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupExecuteResult {
    pub execution: CleanupExecutionResult,
    pub locked_action_ids: Vec<String>,
    pub locked_profiles: Vec<CleanupLockedProfile>,
    pub status: CleanupExecutionStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CleanupExecutionStatus {
    Completed,
    RetryAfterClose,
    ConfirmationRequired,
    BrowserCloseFailed { message: String },
}

pub fn preview_cleanup(
    state: &AppState,
    request: CleanupPreviewRequest,
) -> Result<CleanupPreviewResult, String> {
    let scan = state
        .latest_scan()
        .ok_or_else(|| "latest scan is not available".to_string())?;
    let discovery = state
        .latest_discovery()
        .ok_or_else(|| "discovered profiles are not available".to_string())?;
    let findings =
        selected_findings_from_scan(&flatten_scan_findings(&scan), &request.selected_finding_ids)?;
    let findings = validate_cleanup_findings(findings, &discovery)?;

    let plan = match request.mode {
        CleanupMode::Review => plan_review_cleanup(&findings, &request.selected_finding_ids),
        CleanupMode::Balanced => plan_balanced_cleanup(&findings, &[]),
        CleanupMode::Aggressive => plan_aggressive_cleanup(
            &findings,
            if request.aggressive_confirmed {
                AggressiveConfirmation::Confirmed
            } else {
                AggressiveConfirmation::NotConfirmed
            },
        ),
    }
    .map_err(|error| error.to_string())?;

    let lock_probe = FilesystemResourceLockProbe;
    let (locked_action_ids, locked_profiles) = detect_locked_actions(&plan, &findings, &lock_probe);

    let preview = CleanupPreviewResult {
        warnings: plan.warnings.clone(),
        locked_action_ids,
        locked_profiles,
        requires_confirmation: matches!(request.mode, CleanupMode::Aggressive)
            && !request.aggressive_confirmed,
        plan,
    };
    state.replace_cleanup_preview(preview.clone());
    Ok(preview)
}

pub fn execute_cleanup(
    state: &AppState,
    request: CleanupExecuteRequest,
) -> Result<CleanupExecuteResult, String> {
    let stored_preview = state
        .latest_cleanup_preview()
        .ok_or_else(|| "cleanup preview is not available".to_string())?;
    if stored_preview != request.preview {
        return Err("cleanup preview is stale or was not issued by the backend".into());
    }
    let lock_probe = FilesystemResourceLockProbe;
    let locked_browser_families = request
        .preview
        .locked_profiles
        .iter()
        .map(|profile| profile.browser)
        .collect::<BTreeSet<_>>();
    let closer = BrowserFamilyCloser {
        families: locked_browser_families.iter().copied().collect(),
    };

    let resolution = match request.lock_resolution {
        CleanupLockResolution::RetryAfterManualClose => LockResolution::RetryAfterManualClose,
        CleanupLockResolution::SkipLocked => LockResolution::SkipLocked,
        CleanupLockResolution::RequestAutomaticClose { confirmed } => {
            LockResolution::RequestAutomaticClose { confirmed }
        }
    };

    let preflight =
        preflight_locked_resources(&request.preview.plan, &lock_probe, resolution, &closer);
    let (status, execution) = match preflight {
        PreflightResult::Ready { skipped_ids } => (
            CleanupExecutionStatus::Completed,
            execute_cleanup_plan(&request.preview.plan, &skipped_ids),
        ),
        PreflightResult::RetryAfterClose => (
            CleanupExecutionStatus::RetryAfterClose,
            CleanupExecutionResult::default(),
        ),
        PreflightResult::ConfirmationRequired { .. } => (
            CleanupExecutionStatus::ConfirmationRequired,
            CleanupExecutionResult::default(),
        ),
        PreflightResult::BrowserCloseFailed { message } => (
            CleanupExecutionStatus::BrowserCloseFailed { message },
            CleanupExecutionResult::default(),
        ),
    };

    if matches!(status, CleanupExecutionStatus::Completed) {
        state.clear_cleanup_preview();
    }

    Ok(CleanupExecuteResult {
        execution,
        locked_action_ids: request.preview.locked_action_ids,
        locked_profiles: request.preview.locked_profiles,
        status,
    })
}

fn selected_findings_from_scan(
    findings: &[Finding],
    selected_ids: &[String],
) -> Result<Vec<Finding>, String> {
    let mut selected = Vec::with_capacity(selected_ids.len());
    for id in selected_ids {
        let finding = findings
            .iter()
            .find(|finding| finding.id == *id)
            .ok_or_else(|| format!("selected finding '{id}' is not available"))?;
        selected.push(finding.clone());
    }
    Ok(selected)
}

fn flatten_scan_findings(scan: &ScanRunResult) -> Vec<Finding> {
    scan.profiles
        .iter()
        .flat_map(|profile| profile.findings.iter().cloned())
        .collect()
}

fn validate_cleanup_findings(
    findings: Vec<Finding>,
    discovery: &DesktopBootstrap,
) -> Result<Vec<Finding>, String> {
    let allowed_profiles = discovered_profile_paths(discovery)?;
    findings
        .into_iter()
        .map(|finding| {
            let canonical_profile_path = std::fs::canonicalize(&finding.profile.profile_path)
                .map_err(|error| {
                    format!(
                        "selected finding '{}' profile path could not be validated: {error}",
                        finding.id
                    )
                })?;
            if !allowed_profiles.contains(&canonical_profile_path) {
                return Err(format!(
                    "selected finding '{}' is not within a discovered browser profile",
                    finding.id
                ));
            }
            let mut normalized = finding;
            normalized.profile.profile_path = canonical_profile_path.clone();
            normalized.profile.installation_root = canonical_profile_path
                .parent()
                .map(PathBuf::from)
                .unwrap_or_else(|| normalized.profile.profile_path.clone());
            Ok(normalized)
        })
        .collect()
}

fn discovered_profile_paths(discovery: &DesktopBootstrap) -> Result<BTreeSet<PathBuf>, String> {
    let mut profiles = BTreeSet::new();
    for profile in discovery
        .chrome
        .profiles
        .iter()
        .chain(discovery.edge.profiles.iter())
    {
        let canonical = std::fs::canonicalize(&profile.profile_path).map_err(|error| {
            format!(
                "discovered profile '{}' could not be validated: {error}",
                profile.profile_name
            )
        })?;
        profiles.insert(canonical);
    }
    Ok(profiles)
}

fn detect_locked_actions(
    plan: &CleanupPlan,
    findings: &[Finding],
    probe: &impl ResourceLockProbe,
) -> (Vec<String>, Vec<CleanupLockedProfile>) {
    let finding_map = findings
        .iter()
        .map(|finding| (finding.id.clone(), finding))
        .collect::<std::collections::BTreeMap<_, _>>();
    let mut locked_action_ids = Vec::new();
    let mut locked_profiles = Vec::new();
    for action in &plan.actions {
        if !action.requires_browser_closed || !probe.is_locked(&action.target) {
            continue;
        }
        locked_action_ids.push(action.id.clone());
        if let Some(finding) = finding_map.get(&action.id) {
            locked_profiles.push(CleanupLockedProfile {
                browser: finding.profile.browser,
                profile_name: finding.profile.profile_name.clone(),
                profile_path: finding.profile.profile_path.clone(),
            });
        }
    }
    (locked_action_ids, locked_profiles)
}

#[derive(Default)]
struct FilesystemResourceLockProbe;

impl ResourceLockProbe for FilesystemResourceLockProbe {
    fn is_locked(&self, target: &CleanupTarget) -> bool {
        match target {
            CleanupTarget::CookieHost { profile_path, .. } => {
                let cookies = profile_path.join("Network").join("Cookies");
                path_is_locked(&cookies)
            }
            CleanupTarget::IndexedDbOrigin {
                profile_path,
                origin,
            } => {
                let identifier = origin_to_identifier(origin);
                if let Some(identifier) = identifier {
                    let leveldb = profile_path
                        .join("IndexedDB")
                        .join(format!("{identifier}.indexeddb.leveldb"));
                    let blob = profile_path
                        .join("IndexedDB")
                        .join(format!("{identifier}.indexeddb.blob"));
                    path_is_locked(&leveldb) || path_is_locked(&blob)
                } else {
                    false
                }
            }
            CleanupTarget::ProfileArtifact { path } => path_contains_locked_entry(path),
        }
    }
}

struct BrowserFamilyCloser {
    families: Vec<BrowserFamily>,
}

impl BrowserCloser for BrowserFamilyCloser {
    fn close_browsers(&self) -> Result<(), String> {
        if self.families.is_empty() {
            return Ok(());
        }

        for family in &self.families {
            let image = match family {
                BrowserFamily::Chrome => "chrome.exe",
                BrowserFamily::Edge => "msedge.exe",
            };
            let status = Command::new("taskkill")
                .args(["/IM", image, "/T", "/F"])
                .status()
                .map_err(|error| format!("could not close {image}: {error}"))?;
            if !status.success() {
                return Err(format!("could not close {image}"));
            }
        }

        Ok(())
    }
}

fn path_contains_locked_entry(path: &Path) -> bool {
    if path.is_file() {
        return path_is_locked(path);
    }

    if path.is_dir() {
        if path_is_locked(path) {
            return true;
        }
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                if path_contains_locked_entry(&entry.path()) {
                    return true;
                }
            }
        }
    }

    false
}

#[cfg(windows)]
fn path_is_locked(path: &Path) -> bool {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::Storage::FileSystem::{
        CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_FLAG_BACKUP_SEMANTICS, LOCKFILE_EXCLUSIVE_LOCK,
        LOCKFILE_FAIL_IMMEDIATELY, LockFileEx, OPEN_EXISTING, UnlockFileEx,
    };
    use windows_sys::Win32::System::IO::OVERLAPPED;

    let mut wide = path.as_os_str().encode_wide().collect::<Vec<u16>>();
    wide.push(0);
    let attributes = if path.is_dir() {
        FILE_FLAG_BACKUP_SEMANTICS
    } else {
        FILE_ATTRIBUTE_NORMAL
    };
    let handle: HANDLE = unsafe {
        CreateFileW(
            wide.as_ptr(),
            0x40000000 | 0x80000000,
            0x00000001 | 0x00000002 | 0x00000004,
            std::ptr::null_mut(),
            OPEN_EXISTING,
            attributes,
            std::ptr::null_mut(),
        )
    };
    if handle == INVALID_HANDLE_VALUE {
        return true;
    }

    let mut overlapped = OVERLAPPED::default();
    let locked = unsafe {
        LockFileEx(
            handle,
            LOCKFILE_EXCLUSIVE_LOCK | LOCKFILE_FAIL_IMMEDIATELY,
            0,
            1,
            0,
            &mut overlapped,
        )
    };
    if locked == 0 {
        unsafe {
            CloseHandle(handle);
        }
        return true;
    }

    unsafe {
        let _ = UnlockFileEx(handle, 0, 1, 0, &mut overlapped);
        CloseHandle(handle);
    }
    false
}

#[cfg(not(windows))]
fn path_is_locked(_path: &Path) -> bool {
    false
}

fn origin_to_identifier(origin: &str) -> Option<String> {
    let (scheme, rest) = origin.split_once("://")?;
    let authority = rest.split('/').next()?;
    let (host, port) = match authority.rsplit_once(':') {
        Some((host, port)) if port.parse::<u16>().is_ok() => (host, port.parse::<u16>().ok()?),
        _ => (authority, 0),
    };
    Some(format!("{scheme}_{host}_{port}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::DesktopBootstrap;
    use crate::scan::{ProfileScanSummary, ScanRunResult};
    use crate::state::AppState;
    use rule_format::Confidence;
    use scanner_core::{ArtifactType, BrowserFamily, BrowserProfile, CleanupMode, Finding};
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::Storage::FileSystem::{
        CreateFileW, FILE_ATTRIBUTE_NORMAL, LOCKFILE_EXCLUSIVE_LOCK, LockFileEx, OPEN_EXISTING,
    };
    use windows_sys::Win32::System::IO::OVERLAPPED;

    fn profile(root: &tempfile::TempDir) -> BrowserProfile {
        let profile_path = root.path().join("Default");
        std::fs::create_dir_all(&profile_path).unwrap();
        BrowserProfile {
            browser: BrowserFamily::Chrome,
            installation_root: root.path().to_path_buf(),
            profile_name: "Default".into(),
            profile_path,
        }
    }

    fn discovery_snapshot(profile: &BrowserProfile) -> DesktopBootstrap {
        DesktopBootstrap {
            chrome: scanner_core::DiscoveryResult {
                profiles: vec![profile.clone()],
                warnings: vec![],
            },
            edge: scanner_core::DiscoveryResult::default(),
        }
    }

    fn scan_snapshot(profile: &BrowserProfile, findings: Vec<Finding>) -> ScanRunResult {
        ScanRunResult {
            completed_profiles: 1,
            total_profiles: 1,
            cancelled: false,
            profiles: vec![ProfileScanSummary {
                browser: "Chrome".into(),
                profile_name: "Default".into(),
                profile_path: profile.profile_path.clone(),
                findings,
                warnings: vec![],
            }],
            findings: vec![],
            warnings: vec![],
        }
    }

    fn lock_file(path: &std::path::Path) -> HANDLE {
        let mut wide = path.as_os_str().encode_wide().collect::<Vec<u16>>();
        wide.push(0);
        let handle = unsafe {
            CreateFileW(
                wide.as_ptr(),
                0x40000000 | 0x80000000,
                0x00000001 | 0x00000002 | 0x00000004,
                std::ptr::null_mut(),
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL,
                std::ptr::null_mut(),
            )
        };
        if handle == INVALID_HANDLE_VALUE {
            return handle;
        }

        let mut overlapped = OVERLAPPED::default();
        let locked =
            unsafe { LockFileEx(handle, LOCKFILE_EXCLUSIVE_LOCK, 0, 1, 0, &mut overlapped) };
        if locked == 0 {
            unsafe {
                CloseHandle(handle);
            }
            return INVALID_HANDLE_VALUE;
        }

        handle
    }

    fn close_handle(handle: HANDLE) {
        if handle != INVALID_HANDLE_VALUE {
            let mut overlapped = OVERLAPPED::default();
            unsafe {
                let _ = windows_sys::Win32::Storage::FileSystem::UnlockFileEx(
                    handle,
                    0,
                    1,
                    0,
                    &mut overlapped,
                );
                CloseHandle(handle);
            }
        }
    }

    #[test]
    fn preview_reports_locked_profiles_and_actions_without_mutating() {
        let temp = tempfile::tempdir().unwrap();
        let profile = profile(&temp);
        let cookies_path = profile.profile_path.join("Network");
        std::fs::create_dir_all(&cookies_path).unwrap();
        std::fs::write(cookies_path.join("Cookies"), "fixture").unwrap();

        let state = AppState::default();
        state.replace_discovery(discovery_snapshot(&profile));
        state.replace_scan(scan_snapshot(
            &profile,
            vec![Finding {
                id: "chrome|Default|cookie|analytics.example".into(),
                profile: profile.clone(),
                artifact_type: ArtifactType::Cookie,
                site: Some("analytics.example".into()),
                classification: Some(scanner_core::StorageClassification {
                    ownership: scanner_core::StorageOwnership::TrackerOwned,
                    provenance: scanner_core::StorageClassificationProvenance::TrackerRules,
                    confidence: Some(Confidence::High),
                    matched_rule_ids: vec!["chrome|Default|cookie|analytics.example".into()],
                }),
                evidence_summary: "cookie host matched tracker rule".into(),
                confidence: Some(Confidence::High),
                cleanup_impact: scanner_core::CleanupImpact::MaySignOut,
            }],
        ));

        let lock_handle = lock_file(&cookies_path.join("Cookies"));
        assert_ne!(lock_handle, INVALID_HANDLE_VALUE);

        let result = preview_cleanup(
            &state,
            CleanupPreviewRequest {
                mode: CleanupMode::Review,
                selected_finding_ids: vec!["chrome|Default|cookie|analytics.example".into()],
                aggressive_confirmed: false,
            },
        )
        .unwrap();

        assert_eq!(
            result.locked_action_ids,
            vec!["chrome|Default|cookie|analytics.example"]
        );
        assert_eq!(result.locked_profiles.len(), 1);
        assert_eq!(result.locked_profiles[0].browser, BrowserFamily::Chrome);
        assert_eq!(result.locked_profiles[0].profile_name, "Default");
        assert!(temp.path().join("Default").exists());
        close_handle(lock_handle);
    }

    #[test]
    fn execute_cleanup_can_skip_locked_actions_and_report_retry_after_close() {
        let temp = tempfile::tempdir().unwrap();
        let profile = profile(&temp);
        let cookies_path = profile.profile_path.join("Network");
        std::fs::create_dir_all(&cookies_path).unwrap();
        std::fs::write(cookies_path.join("Cookies"), "fixture").unwrap();

        let state = AppState::default();
        state.replace_discovery(discovery_snapshot(&profile));
        state.replace_scan(scan_snapshot(
            &profile,
            vec![Finding {
                id: "chrome|Default|cookie|analytics.example".into(),
                profile: profile.clone(),
                artifact_type: ArtifactType::Cookie,
                site: Some("analytics.example".into()),
                classification: Some(scanner_core::StorageClassification {
                    ownership: scanner_core::StorageOwnership::TrackerOwned,
                    provenance: scanner_core::StorageClassificationProvenance::TrackerRules,
                    confidence: Some(Confidence::High),
                    matched_rule_ids: vec!["chrome|Default|cookie|analytics.example".into()],
                }),
                evidence_summary: "cookie host matched tracker rule".into(),
                confidence: Some(Confidence::High),
                cleanup_impact: scanner_core::CleanupImpact::MaySignOut,
            }],
        ));

        let lock_handle = lock_file(&cookies_path.join("Cookies"));
        assert_ne!(lock_handle, INVALID_HANDLE_VALUE);

        let preview = preview_cleanup(
            &state,
            CleanupPreviewRequest {
                mode: CleanupMode::Review,
                selected_finding_ids: vec!["chrome|Default|cookie|analytics.example".into()],
                aggressive_confirmed: false,
            },
        )
        .unwrap();

        let skipped = execute_cleanup(
            &state,
            CleanupExecuteRequest {
                preview: preview.clone(),
                lock_resolution: CleanupLockResolution::SkipLocked,
            },
        )
        .unwrap();
        assert_eq!(
            skipped.execution.skipped_ids,
            vec!["chrome|Default|cookie|analytics.example"]
        );
        assert!(matches!(skipped.status, CleanupExecutionStatus::Completed));

        let retry_preview = preview_cleanup(
            &state,
            CleanupPreviewRequest {
                mode: CleanupMode::Review,
                selected_finding_ids: vec!["chrome|Default|cookie|analytics.example".into()],
                aggressive_confirmed: false,
            },
        )
        .unwrap();

        let retry = execute_cleanup(
            &state,
            CleanupExecuteRequest {
                preview: retry_preview,
                lock_resolution: CleanupLockResolution::RetryAfterManualClose,
            },
        )
        .unwrap();
        assert!(matches!(
            retry.status,
            CleanupExecutionStatus::RetryAfterClose
        ));
        close_handle(lock_handle);
    }

    #[test]
    fn execute_cleanup_can_request_confirmed_browser_closure() {
        let temp = tempfile::tempdir().unwrap();
        let profile = profile(&temp);
        let cookies_path = profile.profile_path.join("Network");
        std::fs::create_dir_all(&cookies_path).unwrap();
        std::fs::write(cookies_path.join("Cookies"), "fixture").unwrap();

        let state = AppState::default();
        state.replace_discovery(discovery_snapshot(&profile));
        state.replace_scan(scan_snapshot(
            &profile,
            vec![Finding {
                id: "chrome|Default|cookie|analytics.example".into(),
                profile: profile.clone(),
                artifact_type: ArtifactType::Cookie,
                site: Some("analytics.example".into()),
                classification: Some(scanner_core::StorageClassification {
                    ownership: scanner_core::StorageOwnership::TrackerOwned,
                    provenance: scanner_core::StorageClassificationProvenance::TrackerRules,
                    confidence: Some(Confidence::High),
                    matched_rule_ids: vec!["chrome|Default|cookie|analytics.example".into()],
                }),
                evidence_summary: "cookie host matched tracker rule".into(),
                confidence: Some(Confidence::High),
                cleanup_impact: scanner_core::CleanupImpact::MaySignOut,
            }],
        ));

        let lock_handle = lock_file(&cookies_path.join("Cookies"));
        assert_ne!(lock_handle, INVALID_HANDLE_VALUE);

        let preview = preview_cleanup(
            &state,
            CleanupPreviewRequest {
                mode: CleanupMode::Review,
                selected_finding_ids: vec!["chrome|Default|cookie|analytics.example".into()],
                aggressive_confirmed: false,
            },
        )
        .unwrap();

        let confirmation = execute_cleanup(
            &state,
            CleanupExecuteRequest {
                preview,
                lock_resolution: CleanupLockResolution::RequestAutomaticClose { confirmed: false },
            },
        )
        .unwrap();
        assert!(matches!(
            confirmation.status,
            CleanupExecutionStatus::ConfirmationRequired
        ));
        close_handle(lock_handle);
    }

    #[test]
    fn backend_preview_uses_latest_scan_findings() {
        let temp = tempfile::tempdir().unwrap();
        let profile = profile(&temp);
        let state = AppState::default();
        state.replace_discovery(discovery_snapshot(&profile));
        state.replace_scan(scan_snapshot(
            &profile,
            vec![Finding {
                id: "chrome|Default|cookie|analytics.example".into(),
                profile: profile.clone(),
                artifact_type: ArtifactType::Cookie,
                site: Some("analytics.example".into()),
                classification: Some(scanner_core::StorageClassification {
                    ownership: scanner_core::StorageOwnership::TrackerOwned,
                    provenance: scanner_core::StorageClassificationProvenance::TrackerRules,
                    confidence: Some(Confidence::High),
                    matched_rule_ids: vec!["chrome|Default|cookie|analytics.example".into()],
                }),
                evidence_summary: "cookie host matched tracker rule".into(),
                confidence: Some(Confidence::High),
                cleanup_impact: scanner_core::CleanupImpact::MaySignOut,
            }],
        ));

        let result = preview_cleanup(
            &state,
            CleanupPreviewRequest {
                mode: CleanupMode::Review,
                selected_finding_ids: vec!["chrome|Default|cookie|analytics.example".into()],
                aggressive_confirmed: false,
            },
        )
        .unwrap();

        assert_eq!(result.plan.mode, CleanupMode::Review);
        assert_eq!(result.plan.actions.len(), 1);
        assert_eq!(
            result.locked_action_ids,
            vec!["chrome|Default|cookie|analytics.example"]
        );
    }

    #[test]
    fn backend_preview_rejects_unknown_finding_ids() {
        let temp = tempfile::tempdir().unwrap();
        let profile = profile(&temp);
        let state = AppState::default();
        state.replace_discovery(discovery_snapshot(&profile));
        state.replace_scan(scan_snapshot(&profile, vec![]));

        let error = preview_cleanup(
            &state,
            CleanupPreviewRequest {
                mode: CleanupMode::Review,
                selected_finding_ids: vec!["missing".into()],
                aggressive_confirmed: false,
            },
        );

        assert!(
            error
                .unwrap_err()
                .contains("selected finding 'missing' is not available")
        );
    }

    #[test]
    fn backend_execute_rejects_tampered_preview() {
        let state = AppState::default();
        let preview = CleanupPreviewResult {
            plan: CleanupPlan {
                mode: CleanupMode::Review,
                warnings: vec![],
                estimated_action_count: 0,
                actions: vec![],
            },
            locked_action_ids: vec![],
            locked_profiles: vec![],
            requires_confirmation: false,
            warnings: vec![],
        };
        state.replace_cleanup_preview(preview.clone());

        let error = execute_cleanup(
            &state,
            CleanupExecuteRequest {
                preview: CleanupPreviewResult {
                    warnings: vec!["tampered".into()],
                    ..preview
                },
                lock_resolution: CleanupLockResolution::SkipLocked,
            },
        );

        assert!(error.is_err());
    }

    #[test]
    fn review_preview_includes_selected_actions_and_locked_choices() {
        let temp = tempfile::tempdir().unwrap();
        let profile = profile(&temp);
        let state = AppState::default();
        state.replace_discovery(discovery_snapshot(&profile));
        state.replace_scan(scan_snapshot(
            &profile,
            vec![Finding {
                id: "chrome|Default|cookie|analytics.example".into(),
                profile: profile.clone(),
                artifact_type: ArtifactType::Cookie,
                site: Some("analytics.example".into()),
                classification: Some(scanner_core::StorageClassification {
                    ownership: scanner_core::StorageOwnership::TrackerOwned,
                    provenance: scanner_core::StorageClassificationProvenance::TrackerRules,
                    confidence: Some(Confidence::High),
                    matched_rule_ids: vec!["chrome|Default|cookie|analytics.example".into()],
                }),
                evidence_summary: "cookie host matched tracker rule".into(),
                confidence: Some(Confidence::High),
                cleanup_impact: scanner_core::CleanupImpact::MaySignOut,
            }],
        ));

        let result = preview_cleanup(
            &state,
            CleanupPreviewRequest {
                mode: CleanupMode::Review,
                selected_finding_ids: vec!["chrome|Default|cookie|analytics.example".into()],
                aggressive_confirmed: false,
            },
        )
        .unwrap();

        assert_eq!(result.plan.mode, CleanupMode::Review);
        assert_eq!(result.plan.actions.len(), 1);
        assert_eq!(
            result.locked_action_ids,
            vec!["chrome|Default|cookie|analytics.example"]
        );
        assert_eq!(result.locked_profiles.len(), 1);
        assert!(!result.requires_confirmation);
    }

    #[test]
    fn aggressive_preview_requires_confirmation() {
        let state = AppState::default();
        let temp = tempfile::tempdir().unwrap();
        let profile = profile(&temp);
        state.replace_discovery(discovery_snapshot(&profile));
        state.replace_scan(scan_snapshot(&profile, vec![]));
        let error = preview_cleanup(
            &state,
            CleanupPreviewRequest {
                mode: CleanupMode::Aggressive,
                selected_finding_ids: vec![],
                aggressive_confirmed: false,
            },
        )
        .unwrap_err();

        assert!(error.contains("aggressive cleanup requires explicit confirmation"));
    }

    #[test]
    fn execute_cleanup_skips_locked_actions_when_requested() {
        let temp = tempfile::tempdir().unwrap();
        let profile = profile(&temp);
        let cache_path = profile.profile_path.join("Cache");
        std::fs::create_dir_all(&cache_path).unwrap();
        let state = AppState::default();
        let preview = CleanupPreviewResult {
            plan: CleanupPlan {
                mode: CleanupMode::Review,
                warnings: vec![],
                estimated_action_count: 1,
                actions: vec![scanner_core::CleanupAction {
                    id: "chrome|Default|cookie|analytics.example".into(),
                    artifact_type: scanner_core::ArtifactType::Cookie,
                    target: scanner_core::CleanupTarget::ProfileArtifact { path: cache_path },
                    requires_browser_closed: true,
                }],
            },
            locked_action_ids: vec!["chrome|Default|cookie|analytics.example".into()],
            locked_profiles: vec![CleanupLockedProfile {
                browser: BrowserFamily::Chrome,
                profile_name: "Default".into(),
                profile_path: profile.profile_path.clone(),
            }],
            requires_confirmation: false,
            warnings: vec![],
        };
        state.replace_cleanup_preview(preview.clone());

        let result = execute_cleanup(
            &state,
            CleanupExecuteRequest {
                preview,
                lock_resolution: CleanupLockResolution::SkipLocked,
            },
        )
        .unwrap();

        assert_eq!(
            result.execution.skipped_ids,
            vec!["chrome|Default|cookie|analytics.example"]
        );
        assert_eq!(result.execution.completed_ids.len(), 0);
        assert!(matches!(result.status, CleanupExecutionStatus::Completed));
    }
}
