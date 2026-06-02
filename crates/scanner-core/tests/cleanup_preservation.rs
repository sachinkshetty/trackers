use rule_format::Confidence;
use scanner_core::{
    AllowedCookieHost, ArtifactType, BrowserFamily, BrowserProfile, CleanupImpact, CleanupMode,
    CleanupTarget, Finding, plan_aggressive_cleanup, plan_balanced_cleanup,
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
