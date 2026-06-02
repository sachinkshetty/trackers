use rule_format::{Confidence, RuleBundle};
use scanner_core::{
    ArtifactType, BrowserProfile, CleanupImpact, DiscoveryResult, Finding, PrivacySettingsResult,
    ScanResult, SettingStatus, finding_id, inspect_privacy_settings, inventory_extensions,
    inventory_site_storage, scan_cookies,
};
use serde::{Deserialize, Serialize};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

#[derive(Debug, Clone, Default)]
pub struct CancellationFlag(Arc<AtomicBool>);

impl CancellationFlag {
    pub fn new() -> Self {
        Self(Arc::new(AtomicBool::new(false)))
    }

    pub fn reset(&self) {
        self.0.store(false, Ordering::SeqCst);
    }

    pub fn cancel(&self) {
        self.0.store(true, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanProgress {
    pub browser: String,
    pub profile_name: String,
    pub completed_profiles: usize,
    pub total_profiles: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanRunResult {
    pub completed_profiles: usize,
    pub total_profiles: usize,
    pub cancelled: bool,
    pub profiles: Vec<ProfileScanSummary>,
    pub findings: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileScanSummary {
    pub browser: String,
    pub profile_name: String,
    pub profile_path: std::path::PathBuf,
    pub findings: Vec<Finding>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanRequest {
    pub discovery: DiscoveryResult,
}

pub fn run_scan<F, G>(
    request: ScanRequest,
    bundle: &RuleBundle,
    cancel: &CancellationFlag,
    mut progress: G,
    mut scan_profile: F,
) -> ScanRunResult
where
    F: FnMut(&BrowserProfile, &RuleBundle) -> ScanResult,
    G: FnMut(ScanProgress),
{
    let total_profiles = request.discovery.profiles.len();
    let mut result = ScanRunResult {
        total_profiles,
        ..ScanRunResult::default()
    };

    for profile in request.discovery.profiles {
        if cancel.is_cancelled() {
            result.cancelled = true;
            break;
        }

        progress(ScanProgress {
            browser: format!("{:?}", profile.browser),
            profile_name: profile.profile_name.clone(),
            completed_profiles: result.completed_profiles,
            total_profiles,
        });

        let scan = scan_profile(&profile, bundle);
        result.completed_profiles += 1;
        result.profiles.push(ProfileScanSummary {
            browser: format!("{:?}", profile.browser),
            profile_name: profile.profile_name.clone(),
            profile_path: profile.profile_path.clone(),
            findings: scan.findings.clone(),
            warnings: scan
                .warnings
                .iter()
                .map(|warning| warning.message.clone())
                .collect(),
        });
        result.findings.push(format!(
            "{}: {} finding(s)",
            profile.profile_name,
            scan.findings.len()
        ));
        result.warnings.extend(
            scan.warnings
                .into_iter()
                .map(|warning| format!("{}: {}", profile.profile_name, warning.message)),
        );

        if cancel.is_cancelled() {
            result.cancelled = true;
            break;
        }
    }

    result
}

pub fn scan_profile(profile: &BrowserProfile, bundle: &RuleBundle) -> ScanResult {
    let mut findings = Vec::new();
    let mut warnings = Vec::new();

    let cookie_scan = scan_cookies(profile, bundle);
    findings.extend(cookie_scan.findings);
    warnings.extend(cookie_scan.warnings);

    let storage_scan = inventory_site_storage(profile, bundle);
    findings.extend(storage_scan.findings);
    warnings.extend(storage_scan.warnings);

    let extensions = inventory_extensions(profile);
    findings.extend(extension_findings(profile, extensions.extensions));
    warnings.extend(extensions.warnings);

    let privacy = inspect_privacy_settings(profile);
    findings.extend(privacy_findings(profile, &privacy));
    warnings.extend(privacy.warnings);

    ScanResult { findings, warnings }
}

fn extension_findings(
    profile: &BrowserProfile,
    extensions: Vec<scanner_core::ExtensionInventoryItem>,
) -> Vec<Finding> {
    extensions
        .into_iter()
        .map(|extension| Finding {
            id: finding_id(profile, ArtifactType::Extension, &extension.id),
            profile: profile.clone(),
            artifact_type: ArtifactType::Extension,
            site: None,
            classification: None,
            evidence_summary: match extension.display_name {
                Some(display_name) => format!("extension '{display_name}' is installed"),
                None => format!("extension '{}' is installed", extension.id),
            },
            confidence: Some(Confidence::Low),
            cleanup_impact: CleanupImpact::ReviewRequired,
        })
        .collect()
}

fn privacy_findings(profile: &BrowserProfile, result: &PrivacySettingsResult) -> Vec<Finding> {
    result
        .settings
        .iter()
        .map(|setting| Finding {
            id: finding_id(profile, ArtifactType::Setting, &setting.key),
            profile: profile.clone(),
            artifact_type: ArtifactType::Setting,
            site: None,
            classification: None,
            evidence_summary: match &setting.status {
                SettingStatus::Supported { value } => {
                    format!("privacy setting '{}' is exposed as {value}", setting.key)
                }
                SettingStatus::Unsupported { reason } => {
                    format!("privacy setting '{}' is unsupported: {reason}", setting.key)
                }
            },
            confidence: None,
            cleanup_impact: CleanupImpact::ReviewRequired,
        })
        .collect()
}

pub fn embedded_rule_bundle() -> RuleBundle {
    RuleBundle::from_json(include_str!("../rules/easyprivacy.bundle.json"))
        .expect("embedded EasyPrivacy rule bundle must be valid")
}

#[cfg(test)]
mod tests {
    use super::*;
    use rule_format::RuleSource;
    use scanner_core::{BrowserFamily, BrowserProfile, DiscoveryResult};
    use std::path::PathBuf;

    fn profile(name: &str) -> BrowserProfile {
        BrowserProfile {
            browser: BrowserFamily::Chrome,
            installation_root: PathBuf::from(r"C:\Chrome\User Data"),
            profile_name: name.into(),
            profile_path: PathBuf::from(format!(r"C:\Chrome\User Data\{name}")),
        }
    }

    fn bundle() -> RuleBundle {
        RuleBundle {
            schema_version: 1,
            bundle_version: "embedded".into(),
            generated_at: "2026-06-01T00:00:00Z".into(),
            sources: vec![RuleSource {
                id: "starter".into(),
                name: "Starter".into(),
                url: "https://github.com/sachinkshetty/trackers".into(),
                license: "MIT OR Apache-2.0".into(),
                attribution: "Browser Tracker Cleaner contributors".into(),
            }],
            rules: vec![],
        }
    }

    #[test]
    fn scan_runner_stops_when_cancelled() {
        let request = ScanRequest {
            discovery: DiscoveryResult {
                profiles: vec![profile("Default"), profile("Profile 1")],
                warnings: vec![],
            },
        };
        let cancel = CancellationFlag::new();
        let mut progress_events = Vec::new();
        let mut scanned_profiles = Vec::new();

        let result = run_scan(
            request,
            &bundle(),
            &cancel,
            |progress| {
                progress_events.push(progress);
            },
            |profile, _bundle| {
                scanned_profiles.push(profile.profile_name.clone());
                cancel.cancel();
                ScanResult::default()
            },
        );

        assert_eq!(scanned_profiles, vec!["Default"]);
        assert_eq!(progress_events.len(), 1);
        assert!(result.cancelled);
        assert_eq!(result.completed_profiles, 1);
        assert_eq!(result.total_profiles, 2);
    }

    #[test]
    fn embedded_rule_bundle_loads_generated_easyprivacy_rules() {
        let bundle = embedded_rule_bundle();

        assert_eq!(bundle.sources[0].id, "easyprivacy");
        assert!(!bundle.rules.is_empty());
    }

    #[test]
    fn finding_ids_include_browser_profile_artifact_and_key() {
        let profile = BrowserProfile {
            browser: BrowserFamily::Edge,
            installation_root: PathBuf::from(r"C:\Edge\User Data"),
            profile_name: "Profile 2".into(),
            profile_path: PathBuf::from(r"C:\Edge\User Data\Profile 2"),
        };
        let extensions = vec![scanner_core::ExtensionInventoryItem {
            id: "abcdefghijklmnopabcdefghijklmnop".into(),
            display_name: Some("Fixture Extension".into()),
            enabled: true,
            evidence_source: PathBuf::from(r"C:\Edge\User Data\Profile 2\manifest.json"),
        }];
        let privacy = PrivacySettingsResult {
            settings: vec![scanner_core::PrivacySetting {
                key: "homepage".into(),
                status: SettingStatus::Supported {
                    value: "edge://newtab".into(),
                },
                evidence_source: PathBuf::from(r"C:\Edge\User Data\Profile 2\Preferences"),
            }],
            warnings: vec![],
        };

        let extension_ids = extension_findings(&profile, extensions)
            .into_iter()
            .map(|finding| finding.id)
            .collect::<Vec<_>>();
        let privacy_ids = privacy_findings(&profile, &privacy)
            .into_iter()
            .map(|finding| finding.id)
            .collect::<Vec<_>>();

        assert_eq!(
            extension_ids,
            vec!["edge|Profile 2|extension|abcdefghijklmnopabcdefghijklmnop"]
        );
        assert_eq!(privacy_ids, vec!["edge|Profile 2|setting|homepage"]);
    }
}
