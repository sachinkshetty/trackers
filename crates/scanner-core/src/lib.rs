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
}
