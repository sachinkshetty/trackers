use rule_format::{Confidence, RuleBundle, TrackerCategory};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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
}
