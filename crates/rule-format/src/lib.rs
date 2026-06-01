use std::fmt;

use serde::{Deserialize, Serialize};

pub const SUPPORTED_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuleBundle {
    pub schema_version: u32,
    pub bundle_version: String,
    pub generated_at: String,
    pub sources: Vec<RuleSource>,
    pub rules: Vec<TrackerRule>,
}

impl RuleBundle {
    pub fn from_json(json: &str) -> Result<Self, RuleBundleError> {
        let bundle: Self = serde_json::from_str(json)?;
        if bundle.schema_version != SUPPORTED_SCHEMA_VERSION {
            return Err(RuleBundleError::UnsupportedSchemaVersion(
                bundle.schema_version,
            ));
        }
        Ok(bundle)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuleSource {
    pub id: String,
    pub name: String,
    pub url: String,
    pub license: String,
    pub attribution: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrackerRule {
    pub id: String,
    pub domain: String,
    pub category: TrackerCategory,
    pub confidence: Confidence,
    pub source_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SupplementalRuleSet {
    pub source: RuleSource,
    pub rules: Vec<SupplementalRule>,
}

impl SupplementalRuleSet {
    pub fn from_json(json: &str) -> Result<Self, RuleBundleError> {
        let input: Self = serde_json::from_str(json)?;
        validate_source(&input.source)?;
        for rule in &input.rules {
            if !is_valid_domain(&rule.domain) {
                return Err(RuleBundleError::Validation(format!(
                    "invalid tracker domain '{}'",
                    rule.domain
                )));
            }
        }
        Ok(input)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SupplementalRule {
    pub domain: String,
    pub category: TrackerCategory,
    pub confidence: Confidence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrackerCategory {
    Advertising,
    Analytics,
    Social,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    Low,
    Medium,
    High,
}

#[derive(Debug)]
pub enum RuleBundleError {
    Json(serde_json::Error),
    UnsupportedSchemaVersion(u32),
    Validation(String),
}

impl fmt::Display for RuleBundleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(error) => write!(formatter, "invalid rule-bundle JSON: {error}"),
            Self::UnsupportedSchemaVersion(version) => write!(
                formatter,
                "unsupported rule-bundle schema version {version}; supported version is {SUPPORTED_SCHEMA_VERSION}"
            ),
            Self::Validation(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for RuleBundleError {}

impl From<serde_json::Error> for RuleBundleError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rule_bundle_round_trips_as_json() {
        let bundle = RuleBundle {
            schema_version: SUPPORTED_SCHEMA_VERSION,
            bundle_version: "2026.06.01.1".into(),
            generated_at: "2026-06-01T00:00:00Z".into(),
            sources: vec![RuleSource {
                id: "supplemental".into(),
                name: "Reviewed supplemental rules".into(),
                url: "https://github.com/sachinkshetty/trackers".into(),
                license: "MIT OR Apache-2.0".into(),
                attribution: "Browser Tracker Cleaner contributors".into(),
            }],
            rules: vec![TrackerRule {
                id: "supplemental:analytics.example".into(),
                domain: "analytics.example".into(),
                category: TrackerCategory::Analytics,
                confidence: Confidence::High,
                source_id: "supplemental".into(),
            }],
        };

        let encoded = serde_json::to_string(&bundle).unwrap();
        let decoded = RuleBundle::from_json(&encoded).unwrap();

        assert_eq!(decoded, bundle);
    }

    #[test]
    fn unsupported_schema_version_is_rejected() {
        let json = r#"{
            "schema_version": 999,
            "bundle_version": "future",
            "generated_at": "2026-06-01T00:00:00Z",
            "sources": [],
            "rules": []
        }"#;

        let error = RuleBundle::from_json(json).unwrap_err();

        assert_eq!(
            error.to_string(),
            "unsupported rule-bundle schema version 999; supported version is 1"
        );
    }

    #[test]
    fn supplemental_input_accepts_reviewed_source_metadata() {
        let json = r#"{
            "source": {
                "id": "supplemental",
                "name": "Reviewed supplemental rules",
                "url": "https://github.com/sachinkshetty/trackers",
                "license": "MIT OR Apache-2.0",
                "attribution": "Browser Tracker Cleaner contributors"
            },
            "rules": [{
                "domain": "analytics.example",
                "category": "analytics",
                "confidence": "high"
            }]
        }"#;

        let input = SupplementalRuleSet::from_json(json).unwrap();

        assert_eq!(input.rules[0].domain, "analytics.example");
    }

    #[test]
    fn supplemental_input_rejects_missing_license() {
        let json = r#"{
            "source": {
                "id": "supplemental",
                "name": "Reviewed supplemental rules",
                "url": "https://github.com/sachinkshetty/trackers",
                "license": "",
                "attribution": "Browser Tracker Cleaner contributors"
            },
            "rules": []
        }"#;

        let error = SupplementalRuleSet::from_json(json).unwrap_err();

        assert_eq!(error.to_string(), "source license must not be empty");
    }

    #[test]
    fn supplemental_input_rejects_invalid_domain() {
        let json = r#"{
            "source": {
                "id": "supplemental",
                "name": "Reviewed supplemental rules",
                "url": "https://github.com/sachinkshetty/trackers",
                "license": "MIT OR Apache-2.0",
                "attribution": "Browser Tracker Cleaner contributors"
            },
            "rules": [{
                "domain": "https://analytics.example/path",
                "category": "analytics",
                "confidence": "high"
            }]
        }"#;

        let error = SupplementalRuleSet::from_json(json).unwrap_err();

        assert_eq!(
            error.to_string(),
            "invalid tracker domain 'https://analytics.example/path'"
        );
    }
}

fn validate_source(source: &RuleSource) -> Result<(), RuleBundleError> {
    for (field, value) in [
        ("id", &source.id),
        ("name", &source.name),
        ("url", &source.url),
        ("license", &source.license),
        ("attribution", &source.attribution),
    ] {
        if value.trim().is_empty() {
            return Err(RuleBundleError::Validation(format!(
                "source {field} must not be empty"
            )));
        }
    }
    Ok(())
}

fn is_valid_domain(domain: &str) -> bool {
    !domain.is_empty()
        && domain == domain.to_ascii_lowercase()
        && domain.contains('.')
        && domain.split('.').all(|label| {
            !label.is_empty()
                && !label.starts_with('-')
                && !label.ends_with('-')
                && label
                    .bytes()
                    .all(|character| character.is_ascii_alphanumeric() || character == b'-')
        })
}
