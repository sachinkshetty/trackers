use rule_format::{Confidence, RuleBundle, TrackerCategory};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BrowserFamily {
    Chrome,
    Edge,
}

pub fn finding_id(profile: &BrowserProfile, artifact_type: ArtifactType, key: &str) -> String {
    format!(
        "{}|{}|{}|{}",
        browser_identity(profile.browser),
        profile.profile_name,
        artifact_identity(artifact_type),
        key
    )
}

fn browser_identity(browser: BrowserFamily) -> &'static str {
    match browser {
        BrowserFamily::Chrome => "chrome",
        BrowserFamily::Edge => "edge",
    }
}

fn artifact_identity(artifact_type: ArtifactType) -> &'static str {
    match artifact_type {
        ArtifactType::Cookie => "cookie",
        ArtifactType::LocalStorage => "local_storage",
        ArtifactType::IndexedDb => "indexed_db",
        ArtifactType::Cache => "cache",
        ArtifactType::History => "history",
        ArtifactType::ServiceWorker => "service_worker",
        ArtifactType::Extension => "extension",
        ArtifactType::Setting => "setting",
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrowserProfile {
    pub browser: BrowserFamily,
    pub installation_root: PathBuf,
    pub profile_name: String,
    pub profile_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscoveryWarning {
    pub browser: BrowserFamily,
    pub root: PathBuf,
    pub message: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscoveryResult {
    pub profiles: Vec<BrowserProfile>,
    pub warnings: Vec<DiscoveryWarning>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactType {
    Cookie,
    LocalStorage,
    IndexedDb,
    Cache,
    History,
    ServiceWorker,
    Extension,
    Setting,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CleanupImpact {
    Low,
    MayRemovePreferences,
    MaySignOut,
    ReviewRequired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finding {
    pub id: String,
    pub profile: BrowserProfile,
    pub artifact_type: ArtifactType,
    pub site: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub classification: Option<StorageClassification>,
    pub evidence_summary: String,
    pub confidence: Option<Confidence>,
    pub cleanup_impact: CleanupImpact,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StorageOwnership {
    TrackerOwned,
    SiteOwned,
    Ambiguous,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StorageClassificationProvenance {
    TrackerRules,
    ChromiumLayout,
    AmbiguousFallback,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageClassification {
    pub ownership: StorageOwnership,
    pub provenance: StorageClassificationProvenance,
    pub confidence: Option<Confidence>,
    pub matched_rule_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScanWarning {
    pub profile_path: PathBuf,
    pub artifact_type: ArtifactType,
    pub message: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScanResult {
    pub findings: Vec<Finding>,
    pub warnings: Vec<ScanWarning>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CleanupMode {
    Review,
    Balanced,
    Aggressive,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CleanupTarget {
    CookieHost {
        profile_path: PathBuf,
        host: String,
    },
    IndexedDbOrigin {
        profile_path: PathBuf,
        origin: String,
    },
    ProfileArtifact {
        path: PathBuf,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CleanupAction {
    pub id: String,
    pub artifact_type: ArtifactType,
    pub target: CleanupTarget,
    pub requires_browser_closed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CleanupPlan {
    pub mode: CleanupMode,
    pub actions: Vec<CleanupAction>,
    pub warnings: Vec<String>,
    pub estimated_action_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CleanupPlanError {
    FindingNotAvailable(String),
    FindingCannotBeCleaned(String),
    AggressiveConfirmationRequired,
}

impl std::fmt::Display for CleanupPlanError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FindingNotAvailable(id) => {
                write!(formatter, "selected finding '{id}' is not available")
            }
            Self::FindingCannotBeCleaned(id) => {
                write!(
                    formatter,
                    "selected finding '{id}' cannot be cleaned safely"
                )
            }
            Self::AggressiveConfirmationRequired => {
                formatter.write_str("aggressive cleanup requires explicit confirmation")
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AggressiveConfirmation {
    NotConfirmed,
    Confirmed,
}

impl std::error::Error for CleanupPlanError {}

pub fn plan_review_cleanup(
    findings: &[Finding],
    selected_ids: &[String],
) -> Result<CleanupPlan, CleanupPlanError> {
    let mut actions = Vec::new();
    for id in selected_ids {
        let finding = findings
            .iter()
            .find(|finding| finding.id == *id)
            .ok_or_else(|| CleanupPlanError::FindingNotAvailable(id.clone()))?;
        actions.push(cleanup_action_for_review(finding)?);
    }
    Ok(CleanupPlan {
        mode: CleanupMode::Review,
        estimated_action_count: actions.len(),
        actions,
        warnings: vec![],
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AllowedCookieHost {
    pub profile_path: PathBuf,
    pub host: String,
}

pub fn plan_balanced_cleanup(
    findings: &[Finding],
    allowlist: &[AllowedCookieHost],
) -> Result<CleanupPlan, CleanupPlanError> {
    let actions = findings
        .iter()
        .filter(|finding| finding.artifact_type == ArtifactType::Cookie)
        .filter(|finding| finding.confidence == Some(Confidence::High))
        .filter(|finding| finding.cleanup_impact != CleanupImpact::MaySignOut)
        .filter(|finding| !cookie_is_allowlisted(finding, allowlist))
        .map(cleanup_action_for_review)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(CleanupPlan {
        mode: CleanupMode::Balanced,
        estimated_action_count: actions.len(),
        actions,
        warnings: vec![],
    })
}

fn cookie_is_allowlisted(finding: &Finding, allowlist: &[AllowedCookieHost]) -> bool {
    allowlist.iter().any(|allowed| {
        finding.profile.profile_path == allowed.profile_path
            && finding.site.as_deref() == Some(allowed.host.as_str())
    })
}

pub fn plan_aggressive_cleanup(
    findings: &[Finding],
    confirmation: AggressiveConfirmation,
) -> Result<CleanupPlan, CleanupPlanError> {
    if confirmation != AggressiveConfirmation::Confirmed {
        return Err(CleanupPlanError::AggressiveConfirmationRequired);
    }
    let actions = findings
        .iter()
        .map(cleanup_action_for_aggressive)
        .collect::<Result<Vec<_>, _>>()?;
    let mut warnings = vec![
        "Aggressive cleanup may sign the user out of websites.".into(),
        "Aggressive cleanup may affect site functionality and saved preferences.".into(),
    ];
    if actions
        .iter()
        .any(|action| matches!(action.target, CleanupTarget::ProfileArtifact { .. }))
    {
        warnings.push(
            "Aggressive cleanup may delete broad browser storage directories for local storage, cache, history, and service workers.".into(),
        );
    }
    Ok(CleanupPlan {
        mode: CleanupMode::Aggressive,
        estimated_action_count: actions.len(),
        actions,
        warnings,
    })
}

fn cleanup_action_for_review(finding: &Finding) -> Result<CleanupAction, CleanupPlanError> {
    let target = match finding.artifact_type {
        ArtifactType::Cookie => CleanupTarget::CookieHost {
            profile_path: finding.profile.profile_path.clone(),
            host: finding
                .site
                .clone()
                .ok_or_else(|| CleanupPlanError::FindingCannotBeCleaned(finding.id.clone()))?,
        },
        ArtifactType::IndexedDb if is_tracker_owned_storage(finding) => {
            CleanupTarget::IndexedDbOrigin {
                profile_path: finding.profile.profile_path.clone(),
                origin: finding
                    .site
                    .clone()
                    .ok_or_else(|| CleanupPlanError::FindingCannotBeCleaned(finding.id.clone()))?,
            }
        }
        _ => return Err(CleanupPlanError::FindingCannotBeCleaned(finding.id.clone())),
    };
    Ok(CleanupAction {
        id: finding.id.clone(),
        artifact_type: finding.artifact_type,
        target,
        requires_browser_closed: true,
    })
}

fn cleanup_action_for_aggressive(finding: &Finding) -> Result<CleanupAction, CleanupPlanError> {
    let target = if finding.artifact_type == ArtifactType::Cookie {
        CleanupTarget::CookieHost {
            profile_path: finding.profile.profile_path.clone(),
            host: finding
                .site
                .clone()
                .ok_or_else(|| CleanupPlanError::FindingCannotBeCleaned(finding.id.clone()))?,
        }
    } else if let Some(relative_path) = profile_artifact_path(finding.artifact_type) {
        CleanupTarget::ProfileArtifact {
            path: finding.profile.profile_path.join(relative_path),
        }
    } else {
        return Err(CleanupPlanError::FindingCannotBeCleaned(finding.id.clone()));
    };
    Ok(CleanupAction {
        id: finding.id.clone(),
        artifact_type: finding.artifact_type,
        target,
        requires_browser_closed: true,
    })
}

fn is_tracker_owned_storage(finding: &Finding) -> bool {
    matches!(
        finding
            .classification
            .as_ref()
            .map(|classification| classification.ownership),
        Some(StorageOwnership::TrackerOwned)
    )
}

fn profile_artifact_path(artifact_type: ArtifactType) -> Option<&'static str> {
    match artifact_type {
        ArtifactType::LocalStorage => Some("Local Storage"),
        ArtifactType::IndexedDb => Some("IndexedDB"),
        ArtifactType::Cache => Some("Cache"),
        ArtifactType::History => Some("History"),
        ArtifactType::ServiceWorker => Some("Service Worker"),
        ArtifactType::Cookie | ArtifactType::Extension | ArtifactType::Setting => None,
    }
}

pub trait ResourceLockProbe {
    fn is_locked(&self, target: &CleanupTarget) -> bool;
}

pub trait BrowserCloser {
    fn close_browsers(&self) -> Result<(), String>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockResolution {
    RetryAfterManualClose,
    SkipLocked,
    RequestAutomaticClose { confirmed: bool },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreflightResult {
    Ready { skipped_ids: Vec<String> },
    RetryAfterClose,
    ConfirmationRequired { locked_ids: Vec<String> },
    BrowserCloseFailed { message: String },
}

pub fn preflight_locked_resources(
    plan: &CleanupPlan,
    probe: &impl ResourceLockProbe,
    resolution: LockResolution,
    closer: &impl BrowserCloser,
) -> PreflightResult {
    let locked_ids = plan
        .actions
        .iter()
        .filter(|action| action.requires_browser_closed && probe.is_locked(&action.target))
        .map(|action| action.id.clone())
        .collect::<Vec<_>>();
    if locked_ids.is_empty() {
        return PreflightResult::Ready {
            skipped_ids: vec![],
        };
    }
    match resolution {
        LockResolution::RetryAfterManualClose => PreflightResult::RetryAfterClose,
        LockResolution::SkipLocked => PreflightResult::Ready {
            skipped_ids: locked_ids,
        },
        LockResolution::RequestAutomaticClose { confirmed: false } => {
            PreflightResult::ConfirmationRequired { locked_ids }
        }
        LockResolution::RequestAutomaticClose { confirmed: true } => {
            match closer.close_browsers() {
                Ok(()) => PreflightResult::RetryAfterClose,
                Err(message) => PreflightResult::BrowserCloseFailed { message },
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CleanupFailure {
    pub id: String,
    pub message: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CleanupExecutionResult {
    pub completed_ids: Vec<String>,
    pub skipped_ids: Vec<String>,
    pub failed: Vec<CleanupFailure>,
}

impl CleanupExecutionResult {
    pub fn is_full_success(&self) -> bool {
        self.skipped_ids.is_empty() && self.failed.is_empty()
    }
}

pub fn execute_cleanup(plan: &CleanupPlan, skipped_ids: &[String]) -> CleanupExecutionResult {
    let mut result = CleanupExecutionResult::default();
    for action in &plan.actions {
        if skipped_ids.contains(&action.id) {
            result.skipped_ids.push(action.id.clone());
            continue;
        }
        match execute_action(action) {
            Ok(()) => result.completed_ids.push(action.id.clone()),
            Err(message) => result.failed.push(CleanupFailure {
                id: action.id.clone(),
                message,
            }),
        }
    }
    result
}

fn execute_action(action: &CleanupAction) -> Result<(), String> {
    match &action.target {
        CleanupTarget::CookieHost { profile_path, host } => delete_cookie_host(profile_path, host),
        CleanupTarget::IndexedDbOrigin {
            profile_path,
            origin,
        } => delete_indexeddb_origin(profile_path, origin),
        CleanupTarget::ProfileArtifact { path } if path.is_dir() => {
            std::fs::remove_dir_all(path).map_err(|error| error.to_string())
        }
        CleanupTarget::ProfileArtifact { path } => {
            std::fs::remove_file(path).map_err(|error| error.to_string())
        }
    }
}

fn delete_cookie_host(profile_path: &std::path::Path, host: &str) -> Result<(), String> {
    let path = profile_path.join("Network").join("Cookies");
    let connection = rusqlite::Connection::open(&path).map_err(|error| error.to_string())?;
    connection
        .execute(
            "DELETE FROM cookies WHERE host_key = ?1 OR host_key = ?2",
            rusqlite::params![host, format!(".{host}")],
        )
        .map_err(|error| error.to_string())?;
    Ok(())
}

fn delete_indexeddb_origin(profile_path: &std::path::Path, origin: &str) -> Result<(), String> {
    let identifier = origin_to_identifier(origin).ok_or_else(|| {
        format!("could not derive an IndexedDB storage identifier for '{origin}'")
    })?;
    let indexeddb_root = profile_path.join("IndexedDB");
    for path in [
        indexeddb_root.join(format!("{identifier}.indexeddb.leveldb")),
        indexeddb_root.join(format!("{identifier}.indexeddb.blob")),
    ] {
        if path.is_dir() {
            std::fs::remove_dir_all(&path).map_err(|error| error.to_string())?;
        } else if path.is_file() {
            std::fs::remove_file(&path).map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}

fn origin_to_identifier(origin: &str) -> Option<String> {
    let url = url::Url::parse(origin).ok()?;
    let host = url.host_str()?;
    let port = url.port().unwrap_or(0);
    Some(format!("{}_{}_{}", url.scheme(), host, port))
}

pub fn discover_chrome_profiles(root: &std::path::Path) -> DiscoveryResult {
    discover_profiles(BrowserFamily::Chrome, root)
}

pub fn discover_edge_profiles(root: &std::path::Path) -> DiscoveryResult {
    discover_profiles(BrowserFamily::Edge, root)
}

fn discover_profiles(browser: BrowserFamily, root: &std::path::Path) -> DiscoveryResult {
    let entries = match std::fs::read_dir(root) {
        Ok(entries) => entries,
        Err(error) => {
            let message = if error.kind() == std::io::ErrorKind::NotFound {
                "profile root does not exist".into()
            } else {
                format!("could not read profile root: {error}")
            };
            return DiscoveryResult {
                profiles: vec![],
                warnings: vec![DiscoveryWarning {
                    browser,
                    root: root.to_path_buf(),
                    message,
                }],
            };
        }
    };

    let mut profiles = entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let profile_path = entry.path();
            if !profile_path.join("Preferences").is_file() {
                return None;
            }
            Some(BrowserProfile {
                browser,
                installation_root: root.to_path_buf(),
                profile_name: entry.file_name().to_string_lossy().into_owned(),
                profile_path,
            })
        })
        .collect::<Vec<_>>();
    profiles.sort_by(|left, right| left.profile_name.cmp(&right.profile_name));

    DiscoveryResult {
        profiles,
        warnings: vec![],
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Classification {
    pub category: TrackerCategory,
    pub confidence: Confidence,
    pub matched_rule_ids: Vec<String>,
}

pub fn classify_domain(bundle: &RuleBundle, domain: &str) -> Option<Classification> {
    let domain = domain.trim_end_matches('.').to_ascii_lowercase();
    bundle
        .rules
        .iter()
        .filter(|rule| domain_matches(&domain, &rule.domain))
        .max_by(|left, right| {
            left.domain
                .len()
                .cmp(&right.domain.len())
                .then_with(|| right.id.cmp(&left.id))
        })
        .map(|rule| Classification {
            category: rule.category,
            confidence: rule.confidence,
            matched_rule_ids: vec![rule.id.clone()],
        })
}

fn domain_matches(candidate: &str, rule_domain: &str) -> bool {
    candidate == rule_domain
        || candidate
            .strip_suffix(rule_domain)
            .is_some_and(|prefix| prefix.ends_with('.'))
}

pub fn scan_cookies(profile: &BrowserProfile, bundle: &RuleBundle) -> ScanResult {
    let source = profile.profile_path.join("Network").join("Cookies");
    let copied = std::env::temp_dir().join(format!(
        "tracker-cleaner-cookies-{}-{}",
        std::process::id(),
        unique_suffix()
    ));
    if let Err(error) = std::fs::copy(&source, &copied) {
        return cookie_warning(
            profile,
            format!("cookie database could not be copied: {error}"),
        );
    }

    let result = read_copied_cookies(profile, bundle, &copied).unwrap_or_else(|error| {
        cookie_warning(
            profile,
            format!("could not read copied cookie database: {error}"),
        )
    });
    let _ = std::fs::remove_file(copied);
    result
}

fn read_copied_cookies(
    profile: &BrowserProfile,
    bundle: &RuleBundle,
    copied: &std::path::Path,
) -> rusqlite::Result<ScanResult> {
    let connection =
        rusqlite::Connection::open_with_flags(copied, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let mut statement = connection.prepare("SELECT DISTINCT host_key FROM cookies")?;
    let hosts = statement.query_map([], |row| row.get::<_, String>(0))?;
    let mut findings = Vec::new();
    for host in hosts {
        let site = host?.trim_start_matches('.').to_ascii_lowercase();
        let classification = classify_domain(bundle, &site);
        let is_tracker_owned = classification.is_some();
        let classification_confidence = classification
            .as_ref()
            .map(|classification| classification.confidence);
        let storage_classification = classification.map(|classification| StorageClassification {
            ownership: StorageOwnership::TrackerOwned,
            provenance: StorageClassificationProvenance::TrackerRules,
            confidence: Some(classification.confidence),
            matched_rule_ids: classification.matched_rule_ids,
        });
        findings.push(Finding {
            id: finding_id(profile, ArtifactType::Cookie, &site),
            profile: profile.clone(),
            artifact_type: ArtifactType::Cookie,
            site: Some(site),
            classification: storage_classification,
            evidence_summary: if is_tracker_owned {
                "cookie host matched tracker rule".into()
            } else {
                "cookie host found in browser profile".into()
            },
            confidence: classification_confidence,
            cleanup_impact: CleanupImpact::MaySignOut,
        });
    }
    findings.sort_by(|left, right| left.site.cmp(&right.site));
    Ok(ScanResult {
        findings,
        warnings: vec![],
    })
}

fn cookie_warning(profile: &BrowserProfile, message: String) -> ScanResult {
    ScanResult {
        findings: vec![],
        warnings: vec![ScanWarning {
            profile_path: profile.profile_path.clone(),
            artifact_type: ArtifactType::Cookie,
            message,
        }],
    }
}

fn unique_suffix() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock is before Unix epoch")
        .as_nanos()
}

pub fn inventory_site_storage(profile: &BrowserProfile, bundle: &RuleBundle) -> ScanResult {
    let mut findings = Vec::new();
    findings.extend(profile_level_storage_finding(
        profile,
        ArtifactType::LocalStorage,
        "Local Storage",
        "local_storage",
    ));
    findings.extend(indexeddb_storage_findings(profile, bundle));
    findings.extend(profile_level_storage_finding(
        profile,
        ArtifactType::Cache,
        "Cache",
        "cache",
    ));
    findings.extend(profile_level_storage_finding(
        profile,
        ArtifactType::History,
        "History",
        "history",
    ));
    findings.extend(profile_level_storage_finding(
        profile,
        ArtifactType::ServiceWorker,
        "Service Worker",
        "service_worker",
    ));
    ScanResult {
        findings,
        warnings: vec![],
    }
}

fn profile_level_storage_finding(
    profile: &BrowserProfile,
    artifact_type: ArtifactType,
    relative_path: &str,
    key: &str,
) -> Vec<Finding> {
    let path = profile.profile_path.join(relative_path);
    if !path.exists() {
        return vec![];
    }

    vec![Finding {
        id: finding_id(profile, artifact_type, key),
        profile: profile.clone(),
        artifact_type,
        site: None,
        classification: None,
        evidence_summary: format!("{relative_path} data is present in the browser profile"),
        confidence: None,
        cleanup_impact: CleanupImpact::ReviewRequired,
    }]
}

fn indexeddb_storage_findings(profile: &BrowserProfile, bundle: &RuleBundle) -> Vec<Finding> {
    let root = profile.profile_path.join("IndexedDB");
    if !root.exists() {
        return vec![];
    }

    let mut origins = BTreeSet::new();
    if let Ok(entries) = std::fs::read_dir(&root) {
        for entry in entries.flatten() {
            if let Some(origin) =
                indexeddb_origin_from_entry_name(&entry.file_name().to_string_lossy())
            {
                origins.insert(origin);
            }
        }
    }

    if origins.is_empty() {
        return vec![Finding {
            id: finding_id(profile, ArtifactType::IndexedDb, "indexed_db"),
            profile: profile.clone(),
            artifact_type: ArtifactType::IndexedDb,
            site: None,
            classification: Some(StorageClassification {
                ownership: StorageOwnership::Ambiguous,
                provenance: StorageClassificationProvenance::AmbiguousFallback,
                confidence: None,
                matched_rule_ids: vec![],
            }),
            evidence_summary: "IndexedDB data is present in the browser profile".into(),
            confidence: None,
            cleanup_impact: CleanupImpact::ReviewRequired,
        }];
    }

    origins
        .into_iter()
        .map(|origin| Finding {
            id: finding_id(profile, ArtifactType::IndexedDb, &origin),
            profile: profile.clone(),
            artifact_type: ArtifactType::IndexedDb,
            site: Some(origin.clone()),
            classification: Some(classify_storage_origin(bundle, &origin)),
            evidence_summary: format!("IndexedDB data is present for origin {origin}"),
            confidence: None,
            cleanup_impact: CleanupImpact::ReviewRequired,
        })
        .collect()
}

fn classify_storage_origin(bundle: &RuleBundle, origin: &str) -> StorageClassification {
    let host = origin
        .split_once("://")
        .and_then(|(_, remainder)| remainder.split('/').next())
        .map(|authority| {
            authority
                .rsplit_once(':')
                .and_then(|(host, port)| port.parse::<u16>().ok().map(|_| host).or(Some(authority)))
                .unwrap_or(authority)
        })
        .unwrap_or(origin);

    if let Some(classification) = classify_domain(bundle, host) {
        return StorageClassification {
            ownership: StorageOwnership::TrackerOwned,
            provenance: StorageClassificationProvenance::TrackerRules,
            confidence: Some(classification.confidence),
            matched_rule_ids: classification.matched_rule_ids,
        };
    }

    StorageClassification {
        ownership: StorageOwnership::SiteOwned,
        provenance: StorageClassificationProvenance::ChromiumLayout,
        confidence: None,
        matched_rule_ids: vec![],
    }
}

fn indexeddb_origin_from_entry_name(entry_name: &str) -> Option<String> {
    let serialized_origin = entry_name
        .strip_suffix(".indexeddb.leveldb")
        .or_else(|| entry_name.strip_suffix(".indexeddb.blob"))
        .or_else(|| entry_name.strip_suffix(".leveldb"))
        .or_else(|| entry_name.strip_suffix(".blob"))?;
    origin_identifier_to_string(serialized_origin)
}

fn origin_identifier_to_string(identifier: &str) -> Option<String> {
    if identifier == "null" || identifier == "__0" {
        return None;
    }

    let (scheme, rest) = identifier.split_once('_')?;
    let (host, port) = rest.rsplit_once('_')?;
    let port = port.parse::<u16>().ok()?;
    if port == 0 {
        Some(format!("{scheme}://{host}"))
    } else {
        Some(format!("{scheme}://{host}:{port}"))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtensionInventoryItem {
    pub id: String,
    pub display_name: Option<String>,
    pub enabled: bool,
    pub evidence_source: PathBuf,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtensionInventoryResult {
    pub extensions: Vec<ExtensionInventoryItem>,
    pub warnings: Vec<ScanWarning>,
}

pub fn inventory_extensions(profile: &BrowserProfile) -> ExtensionInventoryResult {
    let preferences_path = profile.profile_path.join("Preferences");
    let preferences = match std::fs::read_to_string(&preferences_path)
        .map_err(|error| error.to_string())
        .and_then(|json| {
            serde_json::from_str::<serde_json::Value>(&json).map_err(|error| error.to_string())
        }) {
        Ok(preferences) => preferences,
        Err(error) => {
            return ExtensionInventoryResult {
                extensions: vec![],
                warnings: vec![ScanWarning {
                    profile_path: profile.profile_path.clone(),
                    artifact_type: ArtifactType::Extension,
                    message: format!("could not parse Preferences: {error}"),
                }],
            };
        }
    };
    let settings = preferences
        .pointer("/extensions/settings")
        .and_then(serde_json::Value::as_object);
    let mut extensions = settings
        .into_iter()
        .flatten()
        .filter_map(|(id, settings)| extension_item(profile, id, settings))
        .collect::<Vec<_>>();
    extensions.sort_by(|left, right| left.id.cmp(&right.id));
    ExtensionInventoryResult {
        extensions,
        warnings: vec![],
    }
}

fn extension_item(
    profile: &BrowserProfile,
    id: &str,
    settings: &serde_json::Value,
) -> Option<ExtensionInventoryItem> {
    let versions_root = profile.profile_path.join("Extensions").join(id);
    let manifest_path = std::fs::read_dir(versions_root)
        .ok()?
        .filter_map(Result::ok)
        .map(|entry| entry.path().join("manifest.json"))
        .filter(|path| path.is_file())
        .max()?;
    let display_name = std::fs::read_to_string(&manifest_path)
        .ok()
        .and_then(|json| serde_json::from_str::<serde_json::Value>(&json).ok())
        .and_then(|manifest| {
            manifest
                .get("name")
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned)
        });
    Some(ExtensionInventoryItem {
        id: id.into(),
        display_name,
        enabled: settings.get("state").and_then(serde_json::Value::as_i64) == Some(1),
        evidence_source: manifest_path,
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum SettingStatus {
    Supported { value: String },
    Unsupported { reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrivacySetting {
    pub key: String,
    pub status: SettingStatus,
    pub evidence_source: PathBuf,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrivacySettingsResult {
    pub settings: Vec<PrivacySetting>,
    pub warnings: Vec<ScanWarning>,
}

impl PrivacySettingsResult {
    pub fn setting(&self, key: &str) -> Option<&PrivacySetting> {
        self.settings.iter().find(|setting| setting.key == key)
    }
}

pub fn inspect_privacy_settings(profile: &BrowserProfile) -> PrivacySettingsResult {
    let preferences_path = profile.profile_path.join("Preferences");
    let preferences = match read_json(&preferences_path) {
        Ok(preferences) => preferences,
        Err(error) => {
            return PrivacySettingsResult {
                settings: vec![],
                warnings: vec![ScanWarning {
                    profile_path: profile.profile_path.clone(),
                    artifact_type: ArtifactType::Setting,
                    message: format!("could not parse Preferences: {error}"),
                }],
            };
        }
    };
    let settings = vec![
        text_setting("homepage", &preferences, "/homepage", &preferences_path),
        text_setting(
            "default_search_engine",
            &preferences,
            "/default_search_provider/name",
            &preferences_path,
        ),
        count_setting(
            "notification_permissions",
            &preferences,
            "/profile/content_settings/exceptions/notifications",
            &preferences_path,
        ),
        sensitive_permissions_setting(&preferences, &preferences_path),
        text_setting("proxy", &preferences, "/proxy/mode", &preferences_path),
        text_setting(
            "secure_dns",
            &preferences,
            "/dns_over_https/mode",
            &preferences_path,
        ),
    ];
    PrivacySettingsResult {
        settings,
        warnings: vec![],
    }
}

fn read_json(path: &std::path::Path) -> Result<serde_json::Value, String> {
    std::fs::read_to_string(path)
        .map_err(|error| error.to_string())
        .and_then(|json| serde_json::from_str(&json).map_err(|error| error.to_string()))
}

fn text_setting(
    key: &str,
    preferences: &serde_json::Value,
    pointer: &str,
    evidence_source: &std::path::Path,
) -> PrivacySetting {
    let status = preferences
        .pointer(pointer)
        .and_then(serde_json::Value::as_str)
        .map(|value| SettingStatus::Supported {
            value: value.to_owned(),
        })
        .unwrap_or_else(|| SettingStatus::Unsupported {
            reason: "not exposed in readable profile preferences".into(),
        });
    PrivacySetting {
        key: key.into(),
        status,
        evidence_source: evidence_source.to_path_buf(),
    }
}

fn count_setting(
    key: &str,
    preferences: &serde_json::Value,
    pointer: &str,
    evidence_source: &std::path::Path,
) -> PrivacySetting {
    let count = preferences
        .pointer(pointer)
        .and_then(serde_json::Value::as_object)
        .map(|entries| entries.len())
        .unwrap_or(0);
    PrivacySetting {
        key: key.into(),
        status: SettingStatus::Supported {
            value: format!("{count} configured entries"),
        },
        evidence_source: evidence_source.to_path_buf(),
    }
}

fn sensitive_permissions_setting(
    preferences: &serde_json::Value,
    evidence_source: &std::path::Path,
) -> PrivacySetting {
    let count = ["geolocation", "media_stream_camera", "media_stream_mic"]
        .into_iter()
        .map(|permission| {
            preferences
                .pointer(&format!(
                    "/profile/content_settings/exceptions/{permission}"
                ))
                .and_then(serde_json::Value::as_object)
                .map(|entries| entries.len())
                .unwrap_or(0)
        })
        .sum::<usize>();
    PrivacySetting {
        key: "sensitive_site_permissions".into(),
        status: SettingStatus::Supported {
            value: format!("{count} configured entries"),
        },
        evidence_source: evidence_source.to_path_buf(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExpertEvidence {
    pub source_path: PathBuf,
    pub fields: BTreeMap<String, String>,
}

impl ExpertEvidence {
    pub fn new(source_path: impl Into<PathBuf>) -> Self {
        Self {
            source_path: source_path.into(),
            fields: BTreeMap::new(),
        }
    }

    pub fn with_field(mut self, key: impl Into<String>, value: impl AsRef<str>) -> Self {
        let key = key.into();
        let value = redact_field(&key, value.as_ref());
        self.fields.insert(key, value);
        self
    }
}

fn redact_field(key: &str, value: &str) -> String {
    let key = key.to_ascii_lowercase();
    if ["cookie", "token", "authorization", "password", "secret"]
        .iter()
        .any(|sensitive| key.contains(sensitive))
    {
        return "[redacted]".into();
    }
    if key.contains("url") {
        return redact_url(value);
    }
    value.into()
}

fn redact_url(value: &str) -> String {
    let Ok(mut url) = url::Url::parse(value) else {
        return "[redacted invalid URL]".into();
    };
    let _ = url.set_username("");
    let _ = url.set_password(None);
    let query_keys = url
        .query_pairs()
        .map(|(key, _)| key.into_owned())
        .collect::<Vec<_>>();
    if !query_keys.is_empty() {
        url.query_pairs_mut()
            .clear()
            .extend_pairs(query_keys.iter().map(|key| (key.as_str(), "[redacted]")));
    }
    url.set_fragment(None);
    url.into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rule_format::{
        Confidence, RuleBundle, SUPPORTED_SCHEMA_VERSION, TrackerCategory, TrackerRule,
    };

    fn bundle() -> RuleBundle {
        RuleBundle {
            schema_version: SUPPORTED_SCHEMA_VERSION,
            bundle_version: "test".into(),
            generated_at: "2026-06-01T00:00:00Z".into(),
            sources: vec![],
            rules: vec![
                TrackerRule {
                    id: "base".into(),
                    domain: "example.test".into(),
                    category: TrackerCategory::Analytics,
                    confidence: Confidence::Medium,
                    source_id: "test".into(),
                },
                TrackerRule {
                    id: "specific".into(),
                    domain: "analytics.example.test".into(),
                    category: TrackerCategory::Advertising,
                    confidence: Confidence::High,
                    source_id: "test".into(),
                },
            ],
        }
    }

    #[test]
    fn exact_domain_match_returns_rule_evidence() {
        let classification = classify_domain(&bundle(), "analytics.example.test").unwrap();

        assert_eq!(classification.category, TrackerCategory::Advertising);
        assert_eq!(classification.confidence, Confidence::High);
        assert_eq!(classification.matched_rule_ids, vec!["specific"]);
    }

    #[test]
    fn subdomain_match_uses_the_most_specific_rule() {
        let classification = classify_domain(&bundle(), "cdn.analytics.example.test").unwrap();

        assert_eq!(classification.matched_rule_ids, vec!["specific"]);
    }

    #[test]
    fn unrelated_domain_is_not_classified() {
        assert_eq!(classify_domain(&bundle(), "unrelated.test"), None);
    }

    #[test]
    fn suffix_without_label_boundary_is_not_classified() {
        assert_eq!(classify_domain(&bundle(), "notexample.test"), None);
    }

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

    fn create_profile(root: &std::path::Path, name: &str) {
        let profile = root.join(name);
        std::fs::create_dir_all(&profile).unwrap();
        std::fs::write(profile.join("Preferences"), "{}").unwrap();
    }

    #[test]
    fn browser_profile_model_keeps_paths_profile_scoped() {
        let profile = BrowserProfile {
            browser: BrowserFamily::Chrome,
            installation_root: r"C:\Users\test\AppData\Local\Google\Chrome\User Data".into(),
            profile_name: "Profile 1".into(),
            profile_path: r"C:\Users\test\AppData\Local\Google\Chrome\User Data\Profile 1".into(),
        };

        assert_eq!(profile.browser, BrowserFamily::Chrome);
        assert_eq!(profile.profile_name, "Profile 1");
        assert!(profile.profile_path.ends_with("Profile 1"));
        assert_ne!(profile.profile_path, profile.installation_root);
    }

    #[test]
    fn discovery_result_serializes_profiles_and_warnings() {
        let result = DiscoveryResult {
            profiles: vec![BrowserProfile {
                browser: BrowserFamily::Edge,
                installation_root: r"C:\Users\test\AppData\Local\Microsoft\Edge\User Data".into(),
                profile_name: "Default".into(),
                profile_path: r"C:\Users\test\AppData\Local\Microsoft\Edge\User Data\Default"
                    .into(),
            }],
            warnings: vec![DiscoveryWarning {
                browser: BrowserFamily::Chrome,
                root: r"C:\missing".into(),
                message: "profile root does not exist".into(),
            }],
        };

        let json = serde_json::to_string(&result).unwrap();

        assert!(json.contains(r#""browser":"edge""#));
        assert!(json.contains(r#""profile_name":"Default""#));
        assert!(json.contains(r#""message":"profile root does not exist""#));
    }

    #[test]
    fn chrome_discovery_finds_default_and_named_profiles() {
        let root = temp_directory("chrome-profiles");
        create_profile(&root, "Default");
        create_profile(&root, "Profile 1");
        std::fs::create_dir_all(root.join("Crashpad")).unwrap();

        let result = discover_chrome_profiles(&root);

        let names = result
            .profiles
            .iter()
            .map(|profile| profile.profile_name.as_str())
            .collect::<Vec<_>>();
        assert_eq!(names, vec!["Default", "Profile 1"]);
        assert!(
            result
                .profiles
                .iter()
                .all(|profile| profile.browser == BrowserFamily::Chrome)
        );
        assert!(result.warnings.is_empty());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn chrome_discovery_warns_when_root_is_missing() {
        let root = temp_directory("missing-chrome-root");
        std::fs::remove_dir_all(&root).unwrap();

        let result = discover_chrome_profiles(&root);

        assert!(result.profiles.is_empty());
        assert_eq!(result.warnings.len(), 1);
        assert_eq!(result.warnings[0].browser, BrowserFamily::Chrome);
        assert_eq!(result.warnings[0].message, "profile root does not exist");
    }

    #[test]
    fn edge_discovery_finds_default_and_named_profiles() {
        let root = temp_directory("edge-profiles");
        create_profile(&root, "Profile 2");
        create_profile(&root, "Default");

        let result = discover_edge_profiles(&root);

        let names = result
            .profiles
            .iter()
            .map(|profile| profile.profile_name.as_str())
            .collect::<Vec<_>>();
        assert_eq!(names, vec!["Default", "Profile 2"]);
        assert!(
            result
                .profiles
                .iter()
                .all(|profile| profile.browser == BrowserFamily::Edge)
        );
        assert!(result.warnings.is_empty());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn edge_discovery_warns_when_root_is_missing() {
        let root = temp_directory("missing-edge-root");
        std::fs::remove_dir_all(&root).unwrap();

        let result = discover_edge_profiles(&root);

        assert!(result.profiles.is_empty());
        assert_eq!(result.warnings[0].browser, BrowserFamily::Edge);
    }

    #[test]
    fn scan_result_serializes_findings_and_partial_failures() {
        let result = ScanResult {
            findings: vec![Finding {
                id: "finding-1".into(),
                profile: BrowserProfile {
                    browser: BrowserFamily::Chrome,
                    installation_root: r"C:\Chrome\User Data".into(),
                    profile_name: "Default".into(),
                    profile_path: r"C:\Chrome\User Data\Default".into(),
                },
                artifact_type: ArtifactType::Cookie,
                site: Some("analytics.example".into()),
                classification: Some(StorageClassification {
                    ownership: StorageOwnership::TrackerOwned,
                    provenance: StorageClassificationProvenance::TrackerRules,
                    confidence: Some(Confidence::High),
                    matched_rule_ids: vec!["finding-1-rule".into()],
                }),
                evidence_summary: "cookie host matched tracker rule".into(),
                confidence: Some(Confidence::High),
                cleanup_impact: CleanupImpact::MaySignOut,
            }],
            warnings: vec![ScanWarning {
                profile_path: r"C:\Chrome\User Data\Default".into(),
                artifact_type: ArtifactType::Cookie,
                message: "cookie database could not be copied".into(),
            }],
        };

        let json = serde_json::to_string(&result).unwrap();

        assert!(json.contains(r#""artifact_type":"cookie""#));
        assert!(json.contains(r#""cleanup_impact":"may_sign_out""#));
        assert!(json.contains(r#""message":"cookie database could not be copied""#));
    }

    fn profile_at(path: &std::path::Path) -> BrowserProfile {
        BrowserProfile {
            browser: BrowserFamily::Chrome,
            installation_root: path.parent().unwrap().to_path_buf(),
            profile_name: path.file_name().unwrap().to_string_lossy().into_owned(),
            profile_path: path.to_path_buf(),
        }
    }

    #[test]
    fn cookie_scan_reports_hosts_without_exposing_values() {
        let root = temp_directory("cookie-scan");
        let profile_path = root.join("Default");
        std::fs::create_dir_all(profile_path.join("Network")).unwrap();
        let database = profile_path.join("Network").join("Cookies");
        let connection = rusqlite::Connection::open(&database).unwrap();
        connection
            .execute_batch(
                "CREATE TABLE cookies (host_key TEXT NOT NULL, name TEXT NOT NULL, value TEXT);
                 INSERT INTO cookies VALUES ('analytics.example.test', 'session', 'secret-token');",
            )
            .unwrap();
        drop(connection);

        let result = scan_cookies(&profile_at(&profile_path), &bundle());
        let json = serde_json::to_string(&result).unwrap();

        assert_eq!(result.findings.len(), 1);
        assert_eq!(
            result.findings[0].site.as_deref(),
            Some("analytics.example.test")
        );
        assert_eq!(
            result.findings[0].id,
            "chrome|Default|cookie|analytics.example.test"
        );
        assert_eq!(result.findings[0].confidence, Some(Confidence::High));
        assert!(!json.contains("secret-token"));
        assert!(!json.contains("session"));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn malformed_cookie_database_returns_scoped_warning() {
        let root = temp_directory("malformed-cookie-scan");
        let profile_path = root.join("Default");
        std::fs::create_dir_all(profile_path.join("Network")).unwrap();
        std::fs::write(profile_path.join("Network").join("Cookies"), "not sqlite").unwrap();

        let result = scan_cookies(&profile_at(&profile_path), &bundle());

        assert!(result.findings.is_empty());
        assert_eq!(result.warnings.len(), 1);
        assert_eq!(result.warnings[0].artifact_type, ArtifactType::Cookie);
        assert!(
            result.warnings[0]
                .message
                .contains("could not read copied cookie database")
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn storage_inventory_reports_supported_artifacts_as_ambiguous() {
        let root = temp_directory("storage-inventory");
        let profile_path = root.join("Default");
        for directory in ["Local Storage", "IndexedDB", "Cache", "Service Worker"] {
            std::fs::create_dir_all(profile_path.join(directory)).unwrap();
        }
        std::fs::write(profile_path.join("History"), "fixture").unwrap();

        let result = inventory_site_storage(&profile_at(&profile_path), &bundle());

        let artifact_types = result
            .findings
            .iter()
            .map(|finding| finding.artifact_type)
            .collect::<Vec<_>>();
        assert_eq!(
            artifact_types,
            vec![
                ArtifactType::LocalStorage,
                ArtifactType::IndexedDb,
                ArtifactType::Cache,
                ArtifactType::History,
                ArtifactType::ServiceWorker,
            ]
        );
        assert_eq!(
            result.findings[0].id,
            "chrome|Default|local_storage|local_storage"
        );
        assert!(
            result
                .findings
                .iter()
                .all(|finding| finding.confidence.is_none())
        );
        assert!(
            result
                .findings
                .iter()
                .all(|finding| finding.cleanup_impact == CleanupImpact::ReviewRequired)
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn storage_inventory_reports_indexeddb_origin_when_available() {
        let root = temp_directory("storage-indexeddb-origin");
        let profile_path = root.join("Default");
        std::fs::create_dir_all(
            profile_path
                .join("IndexedDB")
                .join("https_example.test_0.indexeddb.leveldb"),
        )
        .unwrap();

        let bundle = RuleBundle {
            schema_version: 1,
            bundle_version: "test".into(),
            generated_at: "2026-06-01T00:00:00Z".into(),
            sources: vec![],
            rules: vec![rule_format::TrackerRule {
                id: "analytics-rule".into(),
                domain: "analytics.example".into(),
                category: rule_format::TrackerCategory::Analytics,
                confidence: Confidence::High,
                source_id: "fixture".into(),
            }],
        };
        let result = inventory_site_storage(&profile_at(&profile_path), &bundle);

        let finding = result
            .findings
            .iter()
            .find(|finding| finding.artifact_type == ArtifactType::IndexedDb)
            .expect("indexeddb finding should exist");
        assert_eq!(finding.site.as_deref(), Some("https://example.test"));
        assert_eq!(finding.id, "chrome|Default|indexed_db|https://example.test");
        assert!(finding.evidence_summary.contains("origin"));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn storage_inventory_classifies_indexeddb_origins_with_tracker_rules() {
        let root = temp_directory("storage-indexeddb-tracker");
        let profile_path = root.join("Default");
        std::fs::create_dir_all(
            profile_path
                .join("IndexedDB")
                .join("https_analytics.example_0.indexeddb.leveldb"),
        )
        .unwrap();

        let bundle = RuleBundle {
            schema_version: 1,
            bundle_version: "test".into(),
            generated_at: "2026-06-01T00:00:00Z".into(),
            sources: vec![],
            rules: vec![rule_format::TrackerRule {
                id: "analytics-rule".into(),
                domain: "analytics.example".into(),
                category: rule_format::TrackerCategory::Analytics,
                confidence: Confidence::High,
                source_id: "fixture".into(),
            }],
        };
        let result = inventory_site_storage(&profile_at(&profile_path), &bundle);

        let finding = result
            .findings
            .iter()
            .find(|finding| finding.artifact_type == ArtifactType::IndexedDb)
            .expect("indexeddb finding should exist");
        let classification = finding
            .classification
            .as_ref()
            .expect("indexeddb finding should be classified");
        assert_eq!(classification.ownership, StorageOwnership::TrackerOwned);
        assert_eq!(
            classification.provenance,
            StorageClassificationProvenance::TrackerRules
        );
        assert_eq!(classification.confidence, Some(Confidence::High));
        assert_eq!(classification.matched_rule_ids, vec!["analytics-rule"]);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn storage_inventory_marks_unmatched_origins_as_site_owned() {
        let root = temp_directory("storage-indexeddb-site-owned");
        let profile_path = root.join("Default");
        std::fs::create_dir_all(
            profile_path
                .join("IndexedDB")
                .join("https_site-owned.example_0.indexeddb.leveldb"),
        )
        .unwrap();

        let result = inventory_site_storage(&profile_at(&profile_path), &bundle());

        let finding = result
            .findings
            .iter()
            .find(|finding| finding.artifact_type == ArtifactType::IndexedDb)
            .expect("indexeddb finding should exist");
        let classification = finding
            .classification
            .as_ref()
            .expect("indexeddb finding should be classified");
        assert_eq!(classification.ownership, StorageOwnership::SiteOwned);
        assert_eq!(
            classification.provenance,
            StorageClassificationProvenance::ChromiumLayout
        );
        assert_eq!(classification.confidence, None);
        assert!(classification.matched_rule_ids.is_empty());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn storage_inventory_falls_back_when_indexeddb_origin_is_unreadable() {
        let root = temp_directory("storage-indexeddb-fallback");
        let profile_path = root.join("Default");
        std::fs::create_dir_all(profile_path.join("IndexedDB").join("unexpected-name")).unwrap();

        let result = inventory_site_storage(&profile_at(&profile_path), &bundle());

        assert_eq!(result.findings.len(), 1);
        assert_eq!(result.findings[0].artifact_type, ArtifactType::IndexedDb);
        assert_eq!(result.findings[0].site, None);
        assert_eq!(
            result.findings[0].id,
            "chrome|Default|indexed_db|indexed_db"
        );
        let classification = result.findings[0]
            .classification
            .as_ref()
            .expect("fallback finding should be classified");
        assert_eq!(classification.ownership, StorageOwnership::Ambiguous);
        assert_eq!(
            classification.provenance,
            StorageClassificationProvenance::AmbiguousFallback
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn extension_inventory_reports_identifier_name_state_and_evidence() {
        let root = temp_directory("extension-inventory");
        let profile_path = root.join("Default");
        let extension_id = "abcdefghijklmnopabcdefghijklmnop";
        let extension_root = profile_path
            .join("Extensions")
            .join(extension_id)
            .join("1.2.3_0");
        std::fs::create_dir_all(&extension_root).unwrap();
        std::fs::write(
            extension_root.join("manifest.json"),
            r#"{"name":"Fixture Extension","version":"1.2.3"}"#,
        )
        .unwrap();
        std::fs::write(
            profile_path.join("Preferences"),
            format!(r#"{{"extensions":{{"settings":{{"{extension_id}":{{"state":1}}}}}}}}"#),
        )
        .unwrap();

        let result = inventory_extensions(&profile_at(&profile_path));

        assert_eq!(result.extensions.len(), 1);
        assert_eq!(result.extensions[0].id, extension_id);
        assert_eq!(
            result.extensions[0].display_name.as_deref(),
            Some("Fixture Extension")
        );
        assert!(result.extensions[0].enabled);
        assert!(
            result.extensions[0]
                .evidence_source
                .ends_with("manifest.json")
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn extension_inventory_warns_when_preferences_are_malformed() {
        let root = temp_directory("malformed-extension-preferences");
        let profile_path = root.join("Default");
        std::fs::create_dir_all(&profile_path).unwrap();
        std::fs::write(profile_path.join("Preferences"), "not json").unwrap();

        let result = inventory_extensions(&profile_at(&profile_path));

        assert!(result.extensions.is_empty());
        assert_eq!(result.warnings.len(), 1);
        assert!(
            result.warnings[0]
                .message
                .contains("could not parse Preferences")
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn privacy_settings_report_supported_and_unsupported_values() {
        let root = temp_directory("privacy-settings");
        let profile_path = root.join("Default");
        std::fs::create_dir_all(&profile_path).unwrap();
        std::fs::write(
            profile_path.join("Preferences"),
            r#"{
                "homepage":"https://example.test/",
                "default_search_provider":{"name":"Fixture Search"},
                "profile":{"content_settings":{"exceptions":{
                    "notifications":{"https://news.example,*":{"setting":1}},
                    "geolocation":{"https://maps.example,*":{"setting":1}}
                }}}
            }"#,
        )
        .unwrap();

        let result = inspect_privacy_settings(&profile_at(&profile_path));

        assert_eq!(
            result.setting("homepage").unwrap().status,
            SettingStatus::Supported {
                value: "https://example.test/".into()
            }
        );
        assert_eq!(
            result.setting("notification_permissions").unwrap().status,
            SettingStatus::Supported {
                value: "1 configured entries".into()
            }
        );
        assert!(matches!(
            result.setting("proxy").unwrap().status,
            SettingStatus::Unsupported { .. }
        ));
        assert!(matches!(
            result.setting("secure_dns").unwrap().status,
            SettingStatus::Unsupported { .. }
        ));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn expert_evidence_redacts_sensitive_fields_and_url_values() {
        let evidence = ExpertEvidence::new(r"C:\Chrome\User Data\Default\History")
            .with_field("cookie_value", "secret-cookie")
            .with_field("authorization", "Bearer secret-token")
            .with_field(
                "url",
                "https://user:password@example.test/path?search=private&token=abc#fragment",
            )
            .with_field("domain", "example.test");
        let json = serde_json::to_string(&evidence).unwrap();

        assert!(!json.contains("secret-cookie"));
        assert!(!json.contains("secret-token"));
        assert!(!json.contains("password"));
        assert!(!json.contains("private"));
        assert!(!json.contains("abc"));
        assert!(!json.contains("fragment"));
        assert!(json.contains(r#""cookie_value":"[redacted]""#));
        assert!(json.contains("search=%5Bredacted%5D"));
        assert!(json.contains(r#""domain":"example.test""#));
    }

    #[test]
    fn cleanup_plan_serializes_mode_targets_warnings_and_counts() {
        let plan = CleanupPlan {
            mode: CleanupMode::Review,
            actions: vec![CleanupAction {
                id: "action-1".into(),
                artifact_type: ArtifactType::Cookie,
                target: CleanupTarget::CookieHost {
                    profile_path: r"C:\Chrome\User Data\Default".into(),
                    host: "analytics.example".into(),
                },
                requires_browser_closed: true,
            }],
            warnings: vec!["cleanup may sign the user out".into()],
            estimated_action_count: 1,
        };

        let json = serde_json::to_string(&plan).unwrap();

        assert!(json.contains(r#""mode":"review""#));
        assert!(json.contains(r#""kind":"cookie_host""#));
        assert!(json.contains(r#""requires_browser_closed":true"#));
        assert!(json.contains(r#""estimated_action_count":1"#));
    }

    #[test]
    fn review_plan_maps_selected_findings_to_explicit_actions() {
        let profile = profile_at(std::path::Path::new(r"C:\Chrome\User Data\Default"));
        let findings = vec![Finding {
            id: "cookie:analytics.example".into(),
            profile,
            artifact_type: ArtifactType::Cookie,
            site: Some("analytics.example".into()),
            classification: Some(StorageClassification {
                ownership: StorageOwnership::TrackerOwned,
                provenance: StorageClassificationProvenance::TrackerRules,
                confidence: Some(Confidence::High),
                matched_rule_ids: vec!["cookie:analytics.example-rule".into()],
            }),
            evidence_summary: "cookie host matched tracker rule".into(),
            confidence: Some(Confidence::High),
            cleanup_impact: CleanupImpact::MaySignOut,
        }];

        let plan = plan_review_cleanup(&findings, &["cookie:analytics.example".into()]).unwrap();

        assert_eq!(plan.mode, CleanupMode::Review);
        assert_eq!(plan.actions.len(), 1);
        assert!(matches!(
            plan.actions[0].target,
            CleanupTarget::CookieHost { .. }
        ));
    }

    #[test]
    fn review_plan_uses_precise_indexeddb_origin_target_when_tracker_owned() {
        let profile = profile_at(std::path::Path::new(r"C:\Chrome\User Data\Default"));
        let findings = vec![Finding {
            id: "indexeddb:https://analytics.example".into(),
            profile,
            artifact_type: ArtifactType::IndexedDb,
            site: Some("https://analytics.example".into()),
            classification: Some(StorageClassification {
                ownership: StorageOwnership::TrackerOwned,
                provenance: StorageClassificationProvenance::TrackerRules,
                confidence: Some(Confidence::High),
                matched_rule_ids: vec!["indexeddb-rule".into()],
            }),
            evidence_summary: "IndexedDB data is present for origin https://analytics.example"
                .into(),
            confidence: Some(Confidence::High),
            cleanup_impact: CleanupImpact::ReviewRequired,
        }];

        let plan = plan_review_cleanup(&findings, &["indexeddb:https://analytics.example".into()])
            .unwrap();

        assert_eq!(plan.mode, CleanupMode::Review);
        assert_eq!(plan.actions.len(), 1);
        assert!(matches!(
            plan.actions[0].target,
            CleanupTarget::IndexedDbOrigin { .. }
        ));
    }

    #[test]
    fn review_plan_rejects_unsupported_storage_artifacts() {
        let profile = profile_at(std::path::Path::new(r"C:\Chrome\User Data\Default"));
        let findings = vec![Finding {
            id: "artifact:Cache".into(),
            profile,
            artifact_type: ArtifactType::Cache,
            site: None,
            classification: Some(StorageClassification {
                ownership: StorageOwnership::TrackerOwned,
                provenance: StorageClassificationProvenance::TrackerRules,
                confidence: None,
                matched_rule_ids: vec!["cache-rule".into()],
            }),
            evidence_summary: "Cache data is present in the browser profile".into(),
            confidence: None,
            cleanup_impact: CleanupImpact::ReviewRequired,
        }];

        let error = plan_review_cleanup(&findings, &["artifact:Cache".into()]).unwrap_err();

        assert_eq!(
            error.to_string(),
            "selected finding 'artifact:Cache' cannot be cleaned safely"
        );
    }

    #[test]
    fn review_plan_rejects_stale_selection() {
        let error = plan_review_cleanup(&[], &["missing".into()]).unwrap_err();

        assert_eq!(
            error.to_string(),
            "selected finding 'missing' is not available"
        );
    }

    fn cookie_finding(
        id: &str,
        host: &str,
        confidence: Option<Confidence>,
        impact: CleanupImpact,
    ) -> Finding {
        Finding {
            id: id.into(),
            profile: profile_at(std::path::Path::new(r"C:\Chrome\User Data\Default")),
            artifact_type: ArtifactType::Cookie,
            site: Some(host.into()),
            classification: Some(StorageClassification {
                ownership: StorageOwnership::TrackerOwned,
                provenance: StorageClassificationProvenance::TrackerRules,
                confidence,
                matched_rule_ids: vec![id.into()],
            }),
            evidence_summary: "cookie host found in browser profile".into(),
            confidence,
            cleanup_impact: impact,
        }
    }

    #[test]
    fn balanced_plan_selects_only_high_confidence_non_allowlisted_trackers() {
        let findings = vec![
            cookie_finding(
                "safe",
                "analytics.example",
                Some(Confidence::High),
                CleanupImpact::MayRemovePreferences,
            ),
            cookie_finding(
                "allowlisted",
                "allowed.example",
                Some(Confidence::High),
                CleanupImpact::MayRemovePreferences,
            ),
            cookie_finding(
                "ambiguous",
                "unknown.example",
                None,
                CleanupImpact::ReviewRequired,
            ),
            cookie_finding(
                "login",
                "login.example",
                Some(Confidence::High),
                CleanupImpact::MaySignOut,
            ),
        ];
        let allowlist = vec![AllowedCookieHost {
            profile_path: r"C:\Chrome\User Data\Default".into(),
            host: "allowed.example".into(),
        }];

        let plan = plan_balanced_cleanup(&findings, &allowlist).unwrap();

        assert_eq!(plan.mode, CleanupMode::Balanced);
        assert_eq!(plan.actions.len(), 1);
        assert_eq!(plan.actions[0].id, "safe");
    }

    #[test]
    fn aggressive_plan_requires_explicit_confirmation() {
        let error = plan_aggressive_cleanup(&[], AggressiveConfirmation::NotConfirmed).unwrap_err();

        assert_eq!(
            error.to_string(),
            "aggressive cleanup requires explicit confirmation"
        );
    }

    #[test]
    fn aggressive_plan_includes_broader_artifacts_and_mandatory_warnings() {
        let profile = profile_at(std::path::Path::new(r"C:\Chrome\User Data\Default"));
        let findings = vec![
            cookie_finding(
                "login-cookie",
                "login.example",
                None,
                CleanupImpact::MaySignOut,
            ),
            Finding {
                id: "artifact:Cache".into(),
                profile,
                artifact_type: ArtifactType::Cache,
                site: None,
                classification: None,
                evidence_summary: "Cache data is present in the browser profile".into(),
                confidence: None,
                cleanup_impact: CleanupImpact::ReviewRequired,
            },
        ];

        let plan = plan_aggressive_cleanup(&findings, AggressiveConfirmation::Confirmed).unwrap();

        assert_eq!(plan.mode, CleanupMode::Aggressive);
        assert_eq!(plan.actions.len(), 2);
        assert!(plan.warnings.iter().any(|warning| warning.contains("sign")));
        assert!(
            plan.warnings
                .iter()
                .any(|warning| warning.contains("functionality"))
        );
        assert!(
            plan.warnings
                .iter()
                .any(|warning| warning.contains("broad browser storage directories"))
        );
    }

    struct AlwaysLocked;

    impl ResourceLockProbe for AlwaysLocked {
        fn is_locked(&self, _target: &CleanupTarget) -> bool {
            true
        }
    }

    #[derive(Default)]
    struct RecordingCloser {
        calls: std::cell::Cell<usize>,
    }

    impl BrowserCloser for RecordingCloser {
        fn close_browsers(&self) -> Result<(), String> {
            self.calls.set(self.calls.get() + 1);
            Ok(())
        }
    }

    fn one_action_plan() -> CleanupPlan {
        CleanupPlan {
            mode: CleanupMode::Review,
            actions: vec![CleanupAction {
                id: "cookie:analytics.example".into(),
                artifact_type: ArtifactType::Cookie,
                target: CleanupTarget::CookieHost {
                    profile_path: r"C:\Chrome\User Data\Default".into(),
                    host: "analytics.example".into(),
                },
                requires_browser_closed: true,
            }],
            warnings: vec![],
            estimated_action_count: 1,
        }
    }

    #[test]
    fn locked_preflight_never_closes_browsers_without_confirmation() {
        let closer = RecordingCloser::default();

        let result = preflight_locked_resources(
            &one_action_plan(),
            &AlwaysLocked,
            LockResolution::RequestAutomaticClose { confirmed: false },
            &closer,
        );

        assert_eq!(closer.calls.get(), 0);
        assert!(matches!(
            result,
            PreflightResult::ConfirmationRequired { .. }
        ));
    }

    #[test]
    fn locked_preflight_can_skip_or_confirm_automatic_close() {
        let closer = RecordingCloser::default();

        let skipped = preflight_locked_resources(
            &one_action_plan(),
            &AlwaysLocked,
            LockResolution::SkipLocked,
            &closer,
        );
        assert!(
            matches!(skipped, PreflightResult::Ready { skipped_ids } if skipped_ids == vec!["cookie:analytics.example"])
        );

        let closed = preflight_locked_resources(
            &one_action_plan(),
            &AlwaysLocked,
            LockResolution::RequestAutomaticClose { confirmed: true },
            &closer,
        );
        assert!(matches!(closed, PreflightResult::RetryAfterClose));
        assert_eq!(closer.calls.get(), 1);
    }

    #[test]
    fn cleanup_execution_reports_completed_skipped_and_failed_actions() {
        let root = temp_directory("cleanup-execution");
        let profile_path = root.join("Default");
        std::fs::create_dir_all(profile_path.join("Cache")).unwrap();
        std::fs::write(profile_path.join("Cache").join("entry"), "cached").unwrap();
        std::fs::create_dir_all(profile_path.join("Network")).unwrap();
        std::fs::write(profile_path.join("Network").join("Cookies"), "not sqlite").unwrap();

        let plan = CleanupPlan {
            mode: CleanupMode::Aggressive,
            actions: vec![
                CleanupAction {
                    id: "cache".into(),
                    artifact_type: ArtifactType::Cache,
                    target: CleanupTarget::ProfileArtifact {
                        path: profile_path.join("Cache"),
                    },
                    requires_browser_closed: true,
                },
                CleanupAction {
                    id: "skipped".into(),
                    artifact_type: ArtifactType::History,
                    target: CleanupTarget::ProfileArtifact {
                        path: profile_path.join("History"),
                    },
                    requires_browser_closed: true,
                },
                CleanupAction {
                    id: "cookie".into(),
                    artifact_type: ArtifactType::Cookie,
                    target: CleanupTarget::CookieHost {
                        profile_path: profile_path.clone(),
                        host: "analytics.example".into(),
                    },
                    requires_browser_closed: true,
                },
            ],
            warnings: vec![],
            estimated_action_count: 3,
        };

        let result = execute_cleanup(&plan, &["skipped".into()]);

        assert_eq!(result.completed_ids, vec!["cache"]);
        assert_eq!(result.skipped_ids, vec!["skipped"]);
        assert_eq!(result.failed.len(), 1);
        assert_eq!(result.failed[0].id, "cookie");
        assert!(!result.is_full_success());
        assert!(!profile_path.join("Cache").exists());
        std::fs::remove_dir_all(root).unwrap();
    }
}
