use crate::cleanup::CleanupPreviewResult;
use crate::scan::ScanRunResult;
use scanner_core::{ArtifactType, BrowserFamily, CleanupMode, CleanupTarget, Finding};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const BACKUP_HISTORY_LIMIT: usize = 20;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupBackupRecord {
    pub timestamp_ms: u64,
    pub browser: BrowserFamily,
    pub profile_name: String,
    pub profile_path: PathBuf,
    pub mode: CleanupMode,
    pub rule_bundle_version: String,
    pub action_id: String,
    pub artifact_type: ArtifactType,
    pub backup_path: PathBuf,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupBackupHistory {
    pub records: Vec<CleanupBackupRecord>,
}

pub fn create_cleanup_backups(
    scan: &ScanRunResult,
    preview: &CleanupPreviewResult,
    skipped_ids: &[String],
) -> Result<Vec<CleanupBackupRecord>, String> {
    create_cleanup_backups_for_path(&cleanup_backup_root(), scan, preview, skipped_ids)
}

pub fn create_cleanup_backups_for_path(
    root: &Path,
    scan: &ScanRunResult,
    preview: &CleanupPreviewResult,
    skipped_ids: &[String],
) -> Result<Vec<CleanupBackupRecord>, String> {
    let scan_map = scan
        .profiles
        .iter()
        .flat_map(|profile| profile.findings.iter())
        .map(|finding| (finding.id.clone(), finding))
        .collect::<std::collections::BTreeMap<_, _>>();

    let mut created = Vec::new();
    for action in &preview.plan.actions {
        if skipped_ids.iter().any(|id| id == &action.id) {
            continue;
        }
        let finding = scan_map
            .get(&action.id)
            .ok_or_else(|| format!("cleanup backup could not find finding '{}'", action.id))?;
        let backup_root = root.join(timestamped_backup_name(action.id.as_str()));
        snapshot_action_target(&backup_root, finding, action.target.clone())?;
        created.push(CleanupBackupRecord {
            timestamp_ms: current_timestamp_ms(),
            browser: finding.profile.browser,
            profile_name: finding.profile.profile_name.clone(),
            profile_path: finding.profile.profile_path.clone(),
            mode: preview.plan.mode,
            rule_bundle_version: scan.rule_bundle_version.clone(),
            action_id: action.id.clone(),
            artifact_type: action.artifact_type,
            backup_path: backup_root,
        });
    }

    append_cleanup_backup_records(root, &created)?;
    Ok(created)
}

pub fn cleanup_backup_history() -> Result<CleanupBackupHistory, String> {
    load_cleanup_backup_history_for_path(&cleanup_backup_history_path())
}

pub fn clear_cleanup_backup_history() -> Result<(), String> {
    clear_cleanup_backup_history_for_path(&cleanup_backup_history_path())
}

pub fn load_cleanup_backup_history_for_path(path: &Path) -> Result<CleanupBackupHistory, String> {
    if !path.exists() {
        return Ok(CleanupBackupHistory::default());
    }
    let contents = std::fs::read_to_string(path).map_err(|error| error.to_string())?;
    serde_json::from_str(&contents).map_err(|error| error.to_string())
}

pub fn clear_cleanup_backup_history_for_path(path: &Path) -> Result<(), String> {
    if path.exists() {
        std::fs::remove_file(path).map_err(|error| error.to_string())?;
    }
    Ok(())
}

pub fn append_cleanup_backup_records(
    root: &Path,
    records: &[CleanupBackupRecord],
) -> Result<(), String> {
    if records.is_empty() {
        return Ok(());
    }

    let history_path = cleanup_backup_history_path_for_root(root);
    let mut history = load_cleanup_backup_history_for_path(&history_path)?;
    history.records.extend(records.iter().cloned());
    if history.records.len() > BACKUP_HISTORY_LIMIT {
        let drain_count = history.records.len() - BACKUP_HISTORY_LIMIT;
        for record in history.records.drain(0..drain_count) {
            let _ = std::fs::remove_dir_all(&record.backup_path);
        }
    }
    write_cleanup_backup_history(&history_path, &history)
}

pub fn cleanup_backup_history_path() -> PathBuf {
    cleanup_backup_history_path_for_root(&cleanup_backup_root())
}

pub fn cleanup_backup_history_path_for_root(root: &Path) -> PathBuf {
    root.join("cleanup-backups.json")
}

pub fn cleanup_backup_root() -> PathBuf {
    let base = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    base.join("Trackers").join("cleanup-backups")
}

fn write_cleanup_backup_history(path: &Path, history: &CleanupBackupHistory) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let json = serde_json::to_string_pretty(history).map_err(|error| error.to_string())?;
    std::fs::write(path, json).map_err(|error| error.to_string())
}

fn snapshot_action_target(
    backup_root: &Path,
    finding: &Finding,
    target: CleanupTarget,
) -> Result<(), String> {
    let source_paths = source_paths_for_target(&finding.profile.profile_path, &target)?;
    for source_path in source_paths {
        let relative = source_path
            .strip_prefix(&finding.profile.profile_path)
            .map_err(|error| error.to_string())?;
        let destination = backup_root.join(relative);
        if source_path.is_dir() {
            copy_directory(&source_path, &destination)?;
        } else {
            if let Some(parent) = destination.parent() {
                std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
            }
            std::fs::copy(&source_path, &destination).map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}

fn source_paths_for_target(
    profile_path: &Path,
    target: &CleanupTarget,
) -> Result<Vec<PathBuf>, String> {
    Ok(match target {
        CleanupTarget::CookieHost { .. } => vec![profile_path.join("Network").join("Cookies")],
        CleanupTarget::IndexedDbOrigin { origin, .. } => {
            let identifier = origin_to_identifier(origin)
                .ok_or_else(|| format!("could not derive backup identifier for '{origin}'"))?;
            vec![
                profile_path
                    .join("IndexedDB")
                    .join(format!("{identifier}.indexeddb.leveldb")),
                profile_path
                    .join("IndexedDB")
                    .join(format!("{identifier}.indexeddb.blob")),
            ]
        }
        CleanupTarget::ProfileArtifact { path } => vec![path.clone()],
    })
}

fn copy_directory(source: &Path, destination: &Path) -> Result<(), String> {
    std::fs::create_dir_all(destination).map_err(|error| error.to_string())?;
    for entry in std::fs::read_dir(source).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        if source_path.is_dir() {
            copy_directory(&source_path, &destination_path)?;
        } else {
            if let Some(parent) = destination_path.parent() {
                std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
            }
            std::fs::copy(&source_path, &destination_path).map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}

fn timestamped_backup_name(action_id: &str) -> String {
    format!("{}-{}", current_timestamp_ms(), sanitize_component(action_id))
}

fn sanitize_component(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '-' || character == '_' {
                character
            } else {
                '_'
            }
        })
        .collect()
}

fn current_timestamp_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
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
    use crate::cleanup::CleanupPreviewResult;
    use crate::scan::{ProfileScanSummary, ScanRunResult};
    use scanner_core::{
        ArtifactType, BrowserFamily, BrowserProfile, CleanupAction, CleanupMode, CleanupPlan,
        CleanupTarget, Finding,
    };

    fn sample_scan(profile_path: PathBuf) -> ScanRunResult {
        let profile = BrowserProfile {
            browser: BrowserFamily::Chrome,
            installation_root: profile_path.parent().unwrap().to_path_buf(),
            profile_name: "Default".into(),
            profile_path: profile_path.clone(),
        };
        ScanRunResult {
            completed_profiles: 1,
            total_profiles: 1,
            cancelled: false,
            rule_bundle_version: "embedded".into(),
            profiles: vec![ProfileScanSummary {
                browser: "Chrome".into(),
                profile_name: "Default".into(),
                profile_path: profile_path.clone(),
                findings: vec![Finding {
                    id: "chrome|Default|cookie|analytics.example".into(),
                    profile,
                    artifact_type: ArtifactType::Cookie,
                    site: Some("analytics.example".into()),
                    classification: None,
                    evidence_summary: "cookie host matched tracker rule".into(),
                    confidence: None,
                    cleanup_impact: scanner_core::CleanupImpact::MaySignOut,
                }],
                warnings: vec![],
            }],
            findings: vec![],
            warnings: vec![],
        }
    }

    fn sample_preview(profile_path: PathBuf) -> CleanupPreviewResult {
        CleanupPreviewResult {
            plan: CleanupPlan {
                mode: CleanupMode::Review,
                actions: vec![CleanupAction {
                    id: "chrome|Default|cookie|analytics.example".into(),
                    artifact_type: ArtifactType::Cookie,
                    target: CleanupTarget::CookieHost {
                        profile_path,
                        host: "analytics.example".into(),
                    },
                    requires_browser_closed: true,
                }],
                warnings: vec![],
                estimated_action_count: 1,
            },
            locked_action_ids: vec![],
            locked_profiles: vec![],
            requires_confirmation: false,
            warnings: vec![],
        }
    }

    #[test]
    fn cleanup_backup_captures_target_files_and_tracks_history() {
        let temp = tempfile::tempdir().unwrap();
        let profile_path = temp.path().join("Default");
        std::fs::create_dir_all(profile_path.join("Network")).unwrap();
        std::fs::write(profile_path.join("Network").join("Cookies"), "cookie-db").unwrap();

        let scan = sample_scan(profile_path.clone());
        let preview = sample_preview(profile_path);
        let root = temp.path().join("backups");

        let backups = create_cleanup_backups_for_path(&root, &scan, &preview, &[]).unwrap();

        assert_eq!(backups.len(), 1);
        assert!(backups[0].backup_path.join("Network").join("Cookies").exists());
        assert_eq!(
            std::fs::read_to_string(
                backups[0].backup_path.join("Network").join("Cookies")
            )
            .unwrap(),
            "cookie-db"
        );

        let history = load_cleanup_backup_history_for_path(&cleanup_backup_history_path_for_root(&root)).unwrap();
        assert_eq!(history.records.len(), 1);
    }

    #[test]
    fn cleanup_backup_failure_propagates_for_invalid_backup_root() {
        let temp = tempfile::tempdir().unwrap();
        let profile_path = temp.path().join("Default");
        std::fs::create_dir_all(profile_path.join("Network")).unwrap();
        std::fs::write(profile_path.join("Network").join("Cookies"), "cookie-db").unwrap();

        let scan = sample_scan(profile_path.clone());
        let preview = sample_preview(profile_path);
        let invalid_root = temp.path().join("backup-root-file");
        std::fs::write(&invalid_root, "not a directory").unwrap();

        let error = create_cleanup_backups_for_path(&invalid_root, &scan, &preview, &[])
            .unwrap_err();

        assert!(!error.is_empty());
    }
}
