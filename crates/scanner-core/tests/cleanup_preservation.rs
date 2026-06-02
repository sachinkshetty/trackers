use rule_format::Confidence;
use scanner_core::{
    AllowedCookieHost, ArtifactType, BrowserFamily, BrowserProfile, CleanupImpact, CleanupMode,
    CleanupTarget, Finding, StorageOwnership, execute_cleanup, inventory_site_storage,
    plan_aggressive_cleanup, plan_balanced_cleanup, plan_review_cleanup,
};
use std::path::{Path, PathBuf};

fn temp_directory(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "tracker-cleaner-{name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&path).unwrap();
    path
}

fn browser_profile(root: &Path, name: &str) -> BrowserProfile {
    BrowserProfile {
        browser: BrowserFamily::Chrome,
        installation_root: root.to_path_buf(),
        profile_name: name.into(),
        profile_path: root.join(name),
    }
}

fn cookie_finding(
    root: &Path,
    profile_name: &str,
    site: &str,
    confidence: Option<Confidence>,
    impact: CleanupImpact,
) -> Finding {
    Finding {
        id: format!("{profile_name}:{site}"),
        profile: browser_profile(root, profile_name),
        artifact_type: ArtifactType::Cookie,
        site: Some(site.into()),
        classification: None,
        evidence_summary: "cookie host found in browser profile".into(),
        confidence,
        cleanup_impact: impact,
    }
}

#[test]
fn balanced_cleanup_preserves_ambiguous_and_allowlisted_artifacts() {
    let root = temp_directory("cleanup-preservation");
    let findings = vec![
        cookie_finding(
            &root,
            "Default",
            "analytics.example",
            Some(Confidence::High),
            CleanupImpact::MayRemovePreferences,
        ),
        cookie_finding(
            &root,
            "Default",
            "allowed.example",
            Some(Confidence::High),
            CleanupImpact::MayRemovePreferences,
        ),
        cookie_finding(
            &root,
            "Default",
            "unknown.example",
            None,
            CleanupImpact::ReviewRequired,
        ),
        cookie_finding(
            &root,
            "Default",
            "login.example",
            Some(Confidence::High),
            CleanupImpact::MaySignOut,
        ),
    ];

    let allowlist = vec![AllowedCookieHost {
        profile_path: root.join("Default"),
        host: "allowed.example".into(),
    }];

    let plan = plan_balanced_cleanup(&findings, &allowlist).unwrap();

    assert_eq!(plan.mode, CleanupMode::Balanced);
    assert_eq!(plan.actions.len(), 1);
    assert_eq!(plan.actions[0].id, "Default:analytics.example");
    assert!(matches!(
        plan.actions[0].target,
        CleanupTarget::CookieHost { .. }
    ));

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn balanced_cleanup_selects_tracker_cookie_on_shared_domain() {
    let root = temp_directory("cleanup-shared-domain");
    let findings = vec![cookie_finding(
        &root,
        "Default",
        "cdn.analytics.example",
        Some(Confidence::High),
        CleanupImpact::MayRemovePreferences,
    )];

    let plan = plan_balanced_cleanup(&findings, &[]).unwrap();

    assert_eq!(plan.mode, CleanupMode::Balanced);
    assert_eq!(plan.actions.len(), 1);
    assert_eq!(plan.actions[0].id, "Default:cdn.analytics.example");

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn aggressive_cleanup_requires_explicit_confirmation() {
    let root = temp_directory("cleanup-aggressive");
    let findings = vec![cookie_finding(
        &root,
        "Default",
        "login.example",
        None,
        CleanupImpact::MaySignOut,
    )];

    let error = plan_aggressive_cleanup(
        &findings,
        scanner_core::AggressiveConfirmation::NotConfirmed,
    )
    .unwrap_err();

    assert_eq!(
        error.to_string(),
        "aggressive cleanup requires explicit confirmation"
    );

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn precise_indexeddb_cleanup_preserves_unrelated_origins() {
    let root = temp_directory("cleanup-indexeddb-preservation");
    let profile_path = root.join("Default");
    let tracker_origin_dir = profile_path
        .join("IndexedDB")
        .join("https_tracker.example_0.indexeddb.leveldb");
    let unrelated_origin_dir = profile_path
        .join("IndexedDB")
        .join("https_other.example_0.indexeddb.leveldb");
    std::fs::create_dir_all(&tracker_origin_dir).unwrap();
    std::fs::create_dir_all(&unrelated_origin_dir).unwrap();

    let bundle = rule_format::RuleBundle {
        schema_version: rule_format::SUPPORTED_SCHEMA_VERSION,
        bundle_version: "test".into(),
        generated_at: "2026-06-01T00:00:00Z".into(),
        sources: vec![],
        rules: vec![rule_format::TrackerRule {
            id: "tracker-rule".into(),
            domain: "tracker.example".into(),
            category: rule_format::TrackerCategory::Analytics,
            confidence: Confidence::High,
            source_id: "fixture".into(),
        }],
    };

    let profile = browser_profile(&root, "Default");
    let inventory = inventory_site_storage(&profile, &bundle);
    let tracker_finding = inventory
        .findings
        .iter()
        .find(|finding| {
            matches!(
                finding
                    .classification
                    .as_ref()
                    .map(|classification| classification.ownership),
                Some(StorageOwnership::TrackerOwned)
            )
        })
        .expect("tracker-owned indexeddb finding should exist");

    let plan = plan_review_cleanup(&inventory.findings, &[tracker_finding.id.clone()]).unwrap();
    let result = execute_cleanup(&plan, &[]);

    assert_eq!(result.completed_ids, vec![tracker_finding.id.clone()]);
    assert!(!tracker_origin_dir.exists());
    assert!(unrelated_origin_dir.exists());

    std::fs::remove_dir_all(root).unwrap();
}
