use std::path::PathBuf;

use trackers_desktop::audit::{
    append_cleanup_audit_records, clear_cleanup_audit_history_for_path,
    load_cleanup_audit_records_for_path, CleanupAuditOutcome, CleanupAuditRecord,
};
use scanner_core::{ArtifactType, BrowserFamily, CleanupMode};

fn record(outcome: CleanupAuditOutcome) -> CleanupAuditRecord {
    CleanupAuditRecord {
        timestamp_ms: 123456789,
        browser: BrowserFamily::Chrome,
        profile_name: "Default".into(),
        profile_path: PathBuf::from(r"C:\Chrome\User Data\Default"),
        mode: CleanupMode::Review,
        rule_bundle_version: "embedded".into(),
        action_id: "chrome|Default|cookie|analytics.example".into(),
        artifact_type: ArtifactType::Cookie,
        outcome,
    }
}

#[test]
fn cleanup_audit_records_can_be_appended_loaded_and_cleared() {
    let temp = tempfile::tempdir().unwrap();
    let audit_path = temp.path().join("cleanup-audit.json");

    append_cleanup_audit_records(
        &audit_path,
        &[record(CleanupAuditOutcome::Completed)],
    )
    .unwrap();

    let records = load_cleanup_audit_records_for_path(&audit_path).unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].action_id, "chrome|Default|cookie|analytics.example");
    assert_eq!(records[0].outcome, CleanupAuditOutcome::Completed);

    clear_cleanup_audit_history_for_path(&audit_path).unwrap();
    let records = load_cleanup_audit_records_for_path(&audit_path).unwrap();
    assert!(records.is_empty());
}
