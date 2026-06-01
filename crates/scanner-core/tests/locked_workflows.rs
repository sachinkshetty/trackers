use rule_format::Confidence;
use scanner_core::{
    ArtifactType, BrowserCloser, BrowserFamily, BrowserProfile, CleanupImpact, CleanupPlan,
    CleanupTarget, Finding, LockResolution, PreflightResult, ResourceLockProbe,
    plan_aggressive_cleanup, plan_review_cleanup, preflight_locked_resources,
};
use std::cell::Cell;
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

fn profile(root: &Path) -> BrowserProfile {
    BrowserProfile {
        browser: BrowserFamily::Chrome,
        installation_root: root.to_path_buf(),
        profile_name: "Default".into(),
        profile_path: root.join("Default"),
    }
}

fn plan(root: &Path) -> CleanupPlan {
    let finding = Finding {
        id: "cookie:analytics.example".into(),
        profile: profile(root),
        artifact_type: ArtifactType::Cookie,
        site: Some("analytics.example".into()),
        evidence_summary: "cookie host matched tracker rule".into(),
        confidence: Some(Confidence::High),
        cleanup_impact: CleanupImpact::MaySignOut,
    };

    plan_review_cleanup(&[finding], &["cookie:analytics.example".into()]).unwrap()
}

struct LockedProbe;

impl ResourceLockProbe for LockedProbe {
    fn is_locked(&self, _target: &CleanupTarget) -> bool {
        true
    }
}

#[derive(Default)]
struct RecordingCloser {
    calls: Cell<usize>,
}

impl BrowserCloser for RecordingCloser {
    fn close_browsers(&self) -> Result<(), String> {
        self.calls.set(self.calls.get() + 1);
        Ok(())
    }
}

#[test]
fn locked_workflows_support_retry_skip_and_confirmed_close() {
    let root = temp_directory("locked-workflows");
    let plan = plan(&root);
    let closer = RecordingCloser::default();

    let retry = preflight_locked_resources(
        &plan,
        &LockedProbe,
        LockResolution::RetryAfterManualClose,
        &closer,
    );
    assert!(matches!(retry, PreflightResult::RetryAfterClose));

    let skipped =
        preflight_locked_resources(&plan, &LockedProbe, LockResolution::SkipLocked, &closer);
    assert!(matches!(
        skipped,
        PreflightResult::Ready { skipped_ids } if skipped_ids == vec!["cookie:analytics.example"]
    ));

    let confirmation = preflight_locked_resources(
        &plan,
        &LockedProbe,
        LockResolution::RequestAutomaticClose { confirmed: false },
        &closer,
    );
    assert!(matches!(
        confirmation,
        PreflightResult::ConfirmationRequired { locked_ids } if locked_ids == vec!["cookie:analytics.example"]
    ));

    let closed = preflight_locked_resources(
        &plan,
        &LockedProbe,
        LockResolution::RequestAutomaticClose { confirmed: true },
        &closer,
    );
    assert!(matches!(closed, PreflightResult::RetryAfterClose));
    assert_eq!(closer.calls.get(), 1);

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn aggressive_cleanup_requires_confirmation_before_preflight() {
    let root = temp_directory("locked-aggressive");
    let finding = Finding {
        id: "artifact:Cache".into(),
        profile: profile(&root),
        artifact_type: ArtifactType::Cache,
        site: None,
        evidence_summary: "Cache data is present in the browser profile".into(),
        confidence: None,
        cleanup_impact: CleanupImpact::ReviewRequired,
    };
    let plan = plan_aggressive_cleanup(&[finding], scanner_core::AggressiveConfirmation::Confirmed)
        .unwrap();
    let closer = RecordingCloser::default();

    let confirmation = preflight_locked_resources(
        &plan,
        &LockedProbe,
        LockResolution::RequestAutomaticClose { confirmed: false },
        &closer,
    );

    assert!(matches!(
        confirmation,
        PreflightResult::ConfirmationRequired { .. }
    ));
    assert_eq!(closer.calls.get(), 0);

    std::fs::remove_dir_all(root).unwrap();
}
