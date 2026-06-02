use crate::cleanup::{CleanupExecuteResult, CleanupExecutionStatus, CleanupPreviewResult};
use crate::scan::ScanRunResult;
use scanner_core::{ArtifactType, BrowserFamily, CleanupMode};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const AUDIT_HISTORY_LIMIT: usize = 100;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CleanupAuditOutcome {
    Completed,
    Skipped,
    Failed { message: String },
    Blocked { reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupAuditRecord {
    pub timestamp_ms: u64,
    pub browser: BrowserFamily,
    pub profile_name: String,
    pub profile_path: PathBuf,
    pub mode: CleanupMode,
    pub rule_bundle_version: String,
    pub action_id: String,
    pub artifact_type: ArtifactType,
    pub outcome: CleanupAuditOutcome,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupAuditHistory {
    pub records: Vec<CleanupAuditRecord>,
}

pub fn cleanup_audit_history() -> Result<CleanupAuditHistory, String> {
    load_cleanup_audit_history_for_path(&cleanup_audit_path())
}

pub fn clear_cleanup_audit_history() -> Result<(), String> {
    clear_cleanup_audit_history_for_path(&cleanup_audit_path())
}

pub fn load_cleanup_audit_records_for_path(path: &Path) -> Result<Vec<CleanupAuditRecord>, String> {
    Ok(load_cleanup_audit_history_for_path(path)?.records)
}

pub fn append_cleanup_audit_records(
    path: &Path,
    records: &[CleanupAuditRecord],
) -> Result<(), String> {
    if records.is_empty() {
        return Ok(());
    }

    let mut history = load_cleanup_audit_history_for_path(path)?;
    history.records.extend(records.iter().cloned());
    if history.records.len() > AUDIT_HISTORY_LIMIT {
        let drain_count = history.records.len() - AUDIT_HISTORY_LIMIT;
        history.records.drain(0..drain_count);
    }
    write_cleanup_audit_history(path, &history)
}

pub fn record_cleanup_audit(
    scan: &ScanRunResult,
    preview: &CleanupPreviewResult,
    execution: &CleanupExecuteResult,
    status: &CleanupExecutionStatus,
) -> Result<(), String> {
    let records = cleanup_audit_records(scan, preview, execution, status)?;
    append_cleanup_audit_records(&cleanup_audit_path(), &records)
}

pub fn cleanup_audit_records(
    scan: &ScanRunResult,
    preview: &CleanupPreviewResult,
    execution: &CleanupExecuteResult,
    status: &CleanupExecutionStatus,
) -> Result<Vec<CleanupAuditRecord>, String> {
    let scan_map = scan
        .profiles
        .iter()
        .flat_map(|profile| profile.findings.iter())
        .map(|finding| (finding.id.clone(), finding))
        .collect::<std::collections::BTreeMap<_, _>>();

    let mut records = Vec::new();
    for action in &preview.plan.actions {
        let finding = scan_map
            .get(&action.id)
            .ok_or_else(|| format!("cleanup audit could not find finding '{}'", action.id))?;
        records.push(CleanupAuditRecord {
            timestamp_ms: current_timestamp_ms(),
            browser: finding.profile.browser,
            profile_name: finding.profile.profile_name.clone(),
            profile_path: finding.profile.profile_path.clone(),
            mode: preview.plan.mode,
            rule_bundle_version: scan.rule_bundle_version.clone(),
            action_id: action.id.clone(),
            artifact_type: action.artifact_type,
            outcome: cleanup_outcome_for(action.id.as_str(), execution, status),
        });
    }

    Ok(records)
}

pub fn cleanup_audit_path() -> PathBuf {
    let root = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    root.join("Trackers").join("cleanup-audit.json")
}

pub fn load_cleanup_audit_history_for_path(path: &Path) -> Result<CleanupAuditHistory, String> {
    if !path.exists() {
        return Ok(CleanupAuditHistory::default());
    }
    let contents = std::fs::read_to_string(path).map_err(|error| error.to_string())?;
    serde_json::from_str(&contents).map_err(|error| error.to_string())
}

pub fn clear_cleanup_audit_history_for_path(path: &Path) -> Result<(), String> {
    if path.exists() {
        std::fs::remove_file(path).map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn write_cleanup_audit_history(path: &Path, history: &CleanupAuditHistory) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let json = serde_json::to_string_pretty(history).map_err(|error| error.to_string())?;
    std::fs::write(path, json).map_err(|error| error.to_string())
}

fn cleanup_outcome_for(
    action_id: &str,
    execution: &CleanupExecuteResult,
    status: &CleanupExecutionStatus,
) -> CleanupAuditOutcome {
    if execution
        .execution
        .completed_ids
        .iter()
        .any(|id| id == action_id)
    {
        return CleanupAuditOutcome::Completed;
    }
    if execution.execution.skipped_ids.iter().any(|id| id == action_id) {
        return CleanupAuditOutcome::Skipped;
    }
    if let Some(failure) = execution
        .execution
        .failed
        .iter()
        .find(|failure| failure.id == action_id)
    {
        return CleanupAuditOutcome::Failed {
            message: failure.message.clone(),
        };
    }

    CleanupAuditOutcome::Blocked {
        reason: match status {
            CleanupExecutionStatus::Completed => "action was not executed".into(),
            CleanupExecutionStatus::RetryAfterClose => "browser must be closed before retry".into(),
            CleanupExecutionStatus::ConfirmationRequired => {
                "automatic browser close was not confirmed".into()
            }
            CleanupExecutionStatus::BrowserCloseFailed { message } => {
                format!("browser close failed: {message}")
            }
        },
    }
}

fn current_timestamp_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cleanup::{CleanupExecuteResult, CleanupPreviewResult};
    use crate::scan::ProfileScanSummary;
    use scanner_core::{
        ArtifactType, BrowserProfile, CleanupAction, CleanupExecutionResult, CleanupMode,
        CleanupPlan, CleanupTarget, CleanupFailure, Finding,
    };

    fn sample_scan(rule_bundle_version: &str) -> ScanRunResult {
        let profile = BrowserProfile {
            browser: BrowserFamily::Chrome,
            installation_root: r"C:\Chrome\User Data".into(),
            profile_name: "Default".into(),
            profile_path: r"C:\Chrome\User Data\Default".into(),
        };
        ScanRunResult {
            completed_profiles: 1,
            total_profiles: 1,
            cancelled: false,
            rule_bundle_version: rule_bundle_version.into(),
            profiles: vec![ProfileScanSummary {
                browser: "Chrome".into(),
                profile_name: "Default".into(),
                profile_path: profile.profile_path.clone(),
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

    fn sample_preview() -> CleanupPreviewResult {
        CleanupPreviewResult {
            plan: CleanupPlan {
                mode: CleanupMode::Review,
                actions: vec![CleanupAction {
                    id: "chrome|Default|cookie|analytics.example".into(),
                    artifact_type: ArtifactType::Cookie,
                    target: CleanupTarget::CookieHost {
                        profile_path: r"C:\Chrome\User Data\Default".into(),
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
    fn audit_history_can_be_appended_loaded_and_cleared() {
        let temp = tempfile::tempdir().unwrap();
        let audit_path = temp.path().join("cleanup-audit.json");
        let history = CleanupAuditHistory {
            records: vec![CleanupAuditRecord {
                timestamp_ms: 123,
                browser: BrowserFamily::Chrome,
                profile_name: "Default".into(),
                profile_path: r"C:\Chrome\User Data\Default".into(),
                mode: CleanupMode::Review,
                rule_bundle_version: "embedded".into(),
                action_id: "chrome|Default|cookie|analytics.example".into(),
                artifact_type: ArtifactType::Cookie,
                outcome: CleanupAuditOutcome::Completed,
            }],
        };

        write_cleanup_audit_history(&audit_path, &history).unwrap();
        let loaded = load_cleanup_audit_records_for_path(&audit_path).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].action_id, "chrome|Default|cookie|analytics.example");

        clear_cleanup_audit_history_for_path(&audit_path).unwrap();
        let loaded = load_cleanup_audit_records_for_path(&audit_path).unwrap();
        assert!(loaded.is_empty());
    }

    #[test]
    fn audit_records_derive_outcome_from_execution_status() {
        let scan = sample_scan("embedded");
        let preview = sample_preview();
        let execution = CleanupExecuteResult {
            execution: CleanupExecutionResult {
                completed_ids: vec![],
                skipped_ids: vec![],
                failed: vec![CleanupFailure {
                    id: "chrome|Default|cookie|analytics.example".into(),
                    message: "locked".into(),
                }],
            },
            locked_action_ids: vec![],
            locked_profiles: vec![],
            status: crate::cleanup::CleanupExecutionStatus::Completed,
        };

        let records = cleanup_audit_records(&scan, &preview, &execution, &execution.status).unwrap();
        assert!(matches!(
            records[0].outcome,
            CleanupAuditOutcome::Failed { .. }
        ));
        assert_eq!(records[0].rule_bundle_version, "embedded");
    }

    #[test]
    fn audit_records_mark_blocked_runs() {
        let scan = sample_scan("embedded");
        let preview = sample_preview();
        let execution = CleanupExecuteResult {
            execution: CleanupExecutionResult::default(),
            locked_action_ids: vec![],
            locked_profiles: vec![],
            status: crate::cleanup::CleanupExecutionStatus::RetryAfterClose,
        };

        let records = cleanup_audit_records(&scan, &preview, &execution, &execution.status).unwrap();
        assert!(matches!(
            records[0].outcome,
            CleanupAuditOutcome::Blocked { .. }
        ));
    }
}
