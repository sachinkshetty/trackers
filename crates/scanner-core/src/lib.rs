use rule_format::{Confidence, RuleBundle, TrackerCategory};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BrowserFamily {
    Chrome,
    Edge,
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
    pub profile: BrowserProfile,
    pub artifact_type: ArtifactType,
    pub site: Option<String>,
    pub evidence_summary: String,
    pub confidence: Option<Confidence>,
    pub cleanup_impact: CleanupImpact,
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
        findings.push(Finding {
            profile: profile.clone(),
            artifact_type: ArtifactType::Cookie,
            site: Some(site),
            evidence_summary: if classification.is_some() {
                "cookie host matched tracker rule".into()
            } else {
                "cookie host found in browser profile".into()
            },
            confidence: classification.map(|classification| classification.confidence),
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

pub fn inventory_site_storage(profile: &BrowserProfile) -> ScanResult {
    let artifacts = [
        (ArtifactType::LocalStorage, "Local Storage"),
        (ArtifactType::IndexedDb, "IndexedDB"),
        (ArtifactType::Cache, "Cache"),
        (ArtifactType::History, "History"),
        (ArtifactType::ServiceWorker, "Service Worker"),
    ];
    let findings = artifacts
        .into_iter()
        .filter_map(|(artifact_type, relative_path)| {
            let path = profile.profile_path.join(relative_path);
            path.exists().then(|| Finding {
                profile: profile.clone(),
                artifact_type,
                site: None,
                evidence_summary: format!("{relative_path} data is present in the browser profile"),
                confidence: None,
                cleanup_impact: CleanupImpact::ReviewRequired,
            })
        })
        .collect();
    ScanResult {
        findings,
        warnings: vec![],
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
                profile: BrowserProfile {
                    browser: BrowserFamily::Chrome,
                    installation_root: r"C:\Chrome\User Data".into(),
                    profile_name: "Default".into(),
                    profile_path: r"C:\Chrome\User Data\Default".into(),
                },
                artifact_type: ArtifactType::Cookie,
                site: Some("analytics.example".into()),
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

        let result = inventory_site_storage(&profile_at(&profile_path));

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
}
