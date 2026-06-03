use rule_compiler::compile_rules;
use rule_format::{RuleBundle, RuleSource, SupplementalRule, SupplementalRuleSet, TrackerCategory};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const EASYPRIVACY_URL: &str = "https://easylist.to/easylist/easyprivacy.txt";
const EASYPRIVACY_SOURCE_ID: &str = "easyprivacy";
const EASYPRIVACY_SOURCE_NAME: &str = "EasyPrivacy";
const EASYPRIVACY_SOURCE_LICENSE: &str = "CC-BY-SA-3.0-or-later";
const EASYPRIVACY_SOURCE_ATTRIBUTION: &str = "The EasyList authors (https://easylist.to/)";
const MAX_EXTENSION_STATIC_RULES: usize = 30_000;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RefreshTaskResult {
    NeverRun,
    Succeeded { message: String },
    Failed { message: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EasyPrivacyRefreshSource {
    pub id: String,
    pub name: String,
    pub url: String,
    pub license: String,
    pub attribution: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EasyPrivacyRefreshSnapshot {
    pub last_run_at_ms: Option<u64>,
    pub last_result: RefreshTaskResult,
    pub staged_bundle: Option<EasyPrivacyBundleSummary>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EasyPrivacyBundleSummary {
    pub bundle_version: String,
    pub generated_at: String,
    pub source: EasyPrivacyRefreshSource,
    pub rule_count: usize,
    pub extension_shard_count: usize,
    pub shard_size_limit: usize,
    pub bundle_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EasyPrivacyRefreshState {
    last_run_at_ms: Option<u64>,
    last_result: RefreshTaskResult,
}

impl Default for EasyPrivacyRefreshState {
    fn default() -> Self {
        Self {
            last_run_at_ms: None,
            last_result: RefreshTaskResult::NeverRun,
        }
    }
}

pub fn easyprivacy_refresh_snapshot() -> Result<EasyPrivacyRefreshSnapshot, String> {
    easyprivacy_refresh_snapshot_for_path(&refresh_root_dir())
}

pub fn easyprivacy_refresh_snapshot_for_path(
    root: &Path,
) -> Result<EasyPrivacyRefreshSnapshot, String> {
    let state = load_refresh_state(&refresh_state_path(root))?;
    let bundle_path = staged_bundle_path(root);
    let mut warnings = Vec::new();
    let staged_bundle = match load_staged_bundle(&bundle_path) {
        Ok(Some(bundle)) => Some(bundle),
        Ok(None) => None,
        Err(error) => {
            warnings.push(error);
            None
        }
    };

    Ok(EasyPrivacyRefreshSnapshot {
        last_run_at_ms: state.last_run_at_ms,
        last_result: state.last_result,
        staged_bundle,
        warnings,
    })
}

pub fn refresh_easyprivacy_rules() -> Result<EasyPrivacyRefreshSnapshot, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|error| format!("failed to build EasyPrivacy refresh client: {error}"))?;

    refresh_easyprivacy_rules_with_client(&client, &refresh_root_dir())
}

pub fn refresh_easyprivacy_rules_for_subscription(
    root: &Path,
    subscription: &str,
) -> Result<EasyPrivacyRefreshSnapshot, String> {
    refresh_easyprivacy_rules_from_subscription(root, subscription)
}

fn refresh_easyprivacy_rules_with_client(
    client: &reqwest::blocking::Client,
    root: &Path,
) -> Result<EasyPrivacyRefreshSnapshot, String> {
    let subscription = client
        .get(EASYPRIVACY_URL)
        .header(reqwest::header::USER_AGENT, "trackers-desktop/0.1")
        .send()
        .map_err(|error| format!("EasyPrivacy download failed: {error}"))?
        .error_for_status()
        .map_err(|error| format!("EasyPrivacy download failed: {error}"))?
        .text()
        .map_err(|error| format!("EasyPrivacy download failed: {error}"))?;

    refresh_easyprivacy_rules_from_subscription(root, &subscription)
}

fn refresh_easyprivacy_rules_from_subscription(
    root: &Path,
    subscription: &str,
) -> Result<EasyPrivacyRefreshSnapshot, String> {
    let now_ms = current_time_millis();
    let generated_at = now_ms.to_string();
    let bundle_version = format!("easyprivacy-refresh-{now_ms}");
    let refresh_state_path = refresh_state_path(root);

    match compile_easyprivacy_bundle(root, subscription, &bundle_version, &generated_at) {
        Ok(compiled) => {
            if let Err(error) = write_stage(root, &compiled.bundle) {
                let _ = write_refresh_state(
                    &refresh_state_path,
                    &EasyPrivacyRefreshState {
                        last_run_at_ms: Some(now_ms),
                        last_result: RefreshTaskResult::Failed {
                            message: error.clone(),
                        },
                    },
                );
                return Err(error);
            }

            write_refresh_state(
                &refresh_state_path,
                &EasyPrivacyRefreshState {
                    last_run_at_ms: Some(now_ms),
                    last_result: RefreshTaskResult::Succeeded {
                        message: format!(
                            "staged {} EasyPrivacy rule(s) in {} shard(s)",
                            compiled.summary.rule_count, compiled.summary.extension_shard_count
                        ),
                    },
                },
            )?;
            easyprivacy_refresh_snapshot_for_path(root)
        }
        Err(error) => {
            let message = error.clone();
            let _ = write_refresh_state(
                &refresh_state_path,
                &EasyPrivacyRefreshState {
                    last_run_at_ms: Some(now_ms),
                    last_result: RefreshTaskResult::Failed { message },
                },
            );
            Err(error)
        }
    }
}

fn compile_easyprivacy_bundle(
    root: &Path,
    subscription: &str,
    bundle_version: &str,
    generated_at: &str,
) -> Result<CompiledEasyPrivacyBundle, String> {
    let domains = parse_easyprivacy_domains(subscription);
    if domains.is_empty() {
        return Err("EasyPrivacy subscription did not contain any supported domain rules".into());
    }

    let source = easyprivacy_source();
    let rules = domains
        .iter()
        .map(|domain| SupplementalRule {
            domain: domain.clone(),
            category: TrackerCategory::Analytics,
            confidence: rule_format::Confidence::High,
        })
        .collect::<Vec<_>>();
    let inputs = vec![SupplementalRuleSet {
        source: RuleSource {
            id: source.id.clone(),
            name: source.name.clone(),
            url: source.url.clone(),
            license: source.license.clone(),
            attribution: source.attribution.clone(),
        },
        rules,
    }];
    let compiled = compile_rules(bundle_version, generated_at, &inputs);
    let bundle = RuleBundle::from_json(&compiled.desktop_json)
        .map_err(|error| format!("EasyPrivacy desktop bundle is invalid: {error}"))?;
    validate_bundle(&bundle, &source)?;

    let rule_count = bundle.rules.len();
    let extension_shard_count = shard_count(rule_count, MAX_EXTENSION_STATIC_RULES);
    let summary = EasyPrivacyBundleSummary {
        bundle_version: bundle.bundle_version.clone(),
        generated_at: bundle.generated_at.clone(),
        source,
        rule_count,
        extension_shard_count,
        shard_size_limit: MAX_EXTENSION_STATIC_RULES,
        bundle_path: staged_bundle_path(root),
    };

    Ok(CompiledEasyPrivacyBundle { bundle, summary })
}

fn validate_bundle(
    bundle: &RuleBundle,
    expected_source: &EasyPrivacyRefreshSource,
) -> Result<(), String> {
    if bundle.schema_version != rule_format::SUPPORTED_SCHEMA_VERSION {
        return Err(format!(
            "unsupported EasyPrivacy bundle schema version {}",
            bundle.schema_version
        ));
    }

    if bundle.sources.len() != 1 {
        return Err(format!(
            "EasyPrivacy bundle must contain exactly one source, found {}",
            bundle.sources.len()
        ));
    }

    let source = &bundle.sources[0];
    let expected = RuleSource {
        id: expected_source.id.clone(),
        name: expected_source.name.clone(),
        url: expected_source.url.clone(),
        license: expected_source.license.clone(),
        attribution: expected_source.attribution.clone(),
    };
    if source != &expected {
        return Err(
            "EasyPrivacy source metadata did not match the selected upstream source".into(),
        );
    }

    Ok(())
}

fn parse_easyprivacy_domains(subscription: &str) -> Vec<String> {
    let mut domains = std::collections::BTreeSet::new();

    for raw_line in subscription.lines() {
        let line = raw_line.trim();
        let Some(domain) = line
            .strip_prefix("||")
            .and_then(|candidate| candidate.strip_suffix('^'))
        else {
            continue;
        };

        if is_supported_domain(domain) {
            domains.insert(domain.to_lowercase());
        }
    }

    domains.into_iter().collect()
}

fn is_supported_domain(domain: &str) -> bool {
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

fn shard_count(rule_count: usize, shard_size: usize) -> usize {
    if rule_count == 0 {
        return 0;
    }
    rule_count.div_ceil(shard_size)
}

fn easyprivacy_source() -> EasyPrivacyRefreshSource {
    EasyPrivacyRefreshSource {
        id: EASYPRIVACY_SOURCE_ID.into(),
        name: EASYPRIVACY_SOURCE_NAME.into(),
        url: EASYPRIVACY_URL.into(),
        license: EASYPRIVACY_SOURCE_LICENSE.into(),
        attribution: EASYPRIVACY_SOURCE_ATTRIBUTION.into(),
    }
}

fn refresh_root_dir() -> PathBuf {
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\Users\Public\AppData\Local"))
        .join("Trackers")
        .join("rule-refresh")
}

fn refresh_state_path(root: &Path) -> PathBuf {
    root.join("refresh-state.json")
}

fn staged_bundle_path(root: &Path) -> PathBuf {
    root.join("easyprivacy.bundle.json")
}

fn load_refresh_state(path: &Path) -> Result<EasyPrivacyRefreshState, String> {
    match fs::read_to_string(path) {
        Ok(json) => serde_json::from_str(&json)
            .map_err(|error| format!("failed to load EasyPrivacy refresh state: {error}")),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Default::default()),
        Err(error) => Err(format!("failed to load EasyPrivacy refresh state: {error}")),
    }
}

fn write_refresh_state(path: &Path, state: &EasyPrivacyRefreshState) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!("failed to prepare EasyPrivacy refresh state directory: {error}")
        })?;
    }
    let json = serde_json::to_string_pretty(state)
        .map_err(|error| format!("failed to serialize EasyPrivacy refresh state: {error}"))?;
    fs::write(path, format!("{json}\n"))
        .map_err(|error| format!("failed to save EasyPrivacy refresh state: {error}"))
}

fn load_staged_bundle(path: &Path) -> Result<Option<EasyPrivacyBundleSummary>, String> {
    let json = match fs::read_to_string(path) {
        Ok(json) => json,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(format!("failed to read staged EasyPrivacy bundle: {error}")),
    };

    let bundle = RuleBundle::from_json(&json)
        .map_err(|error| format!("staged EasyPrivacy bundle is invalid: {error}"))?;
    let source = bundle
        .sources
        .first()
        .ok_or_else(|| "staged EasyPrivacy bundle does not contain any sources".to_string())?;
    let summary = EasyPrivacyBundleSummary {
        bundle_version: bundle.bundle_version.clone(),
        generated_at: bundle.generated_at.clone(),
        source: EasyPrivacyRefreshSource {
            id: source.id.clone(),
            name: source.name.clone(),
            url: source.url.clone(),
            license: source.license.clone(),
            attribution: source.attribution.clone(),
        },
        rule_count: bundle.rules.len(),
        extension_shard_count: shard_count(bundle.rules.len(), MAX_EXTENSION_STATIC_RULES),
        shard_size_limit: MAX_EXTENSION_STATIC_RULES,
        bundle_path: path.to_path_buf(),
    };

    Ok(Some(summary))
}

fn write_stage(root: &Path, bundle: &RuleBundle) -> Result<(), String> {
    let bundle_path = staged_bundle_path(root);
    if let Some(parent) = bundle_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("failed to prepare EasyPrivacy refresh directory: {error}"))?;
    }

    let json = serde_json::to_string_pretty(bundle)
        .map_err(|error| format!("failed to serialize EasyPrivacy bundle: {error}"))?;
    fs::write(&bundle_path, format!("{json}\n"))
        .map_err(|error| format!("failed to stage EasyPrivacy bundle: {error}"))
}

fn current_time_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[derive(Debug)]
struct CompiledEasyPrivacyBundle {
    bundle: RuleBundle,
    summary: EasyPrivacyBundleSummary,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn subscription() -> &'static str {
        r#"
            ! comment
            ||analytics.example^
            ||tracking.example^
            ||analytics.example^
            ||bad-rule/path
        "#
    }

    #[test]
    fn parse_easyprivacy_domains_deduplicates_supported_rules() {
        let domains = parse_easyprivacy_domains(subscription());

        assert_eq!(domains, vec!["analytics.example", "tracking.example"]);
    }

    #[test]
    fn refresh_snapshot_defaults_are_conservative() {
        let root = tempdir().unwrap();
        let snapshot = easyprivacy_refresh_snapshot_for_path(root.path()).unwrap();

        assert_eq!(snapshot.last_run_at_ms, None);
        assert!(matches!(snapshot.last_result, RefreshTaskResult::NeverRun));
        assert!(snapshot.staged_bundle.is_none());
        assert!(snapshot.warnings.is_empty());
    }

    #[test]
    fn refresh_stages_and_preserves_the_last_successful_bundle() {
        let root = tempdir().unwrap();
        let snapshot = refresh_easyprivacy_rules_for_subscription(
            root.path(),
            "||analytics.example^\n||tracking.example^\n",
        )
        .unwrap();

        assert!(snapshot.staged_bundle.is_some());
        assert_eq!(snapshot.staged_bundle.as_ref().unwrap().rule_count, 2);
        assert!(matches!(
            snapshot.last_result,
            RefreshTaskResult::Succeeded { .. }
        ));

        let failed = refresh_easyprivacy_rules_for_subscription(root.path(), "no supported rules")
            .unwrap_err();
        assert!(failed.contains("did not contain any supported domain rules"));

        let reloaded = easyprivacy_refresh_snapshot_for_path(root.path()).unwrap();
        assert_eq!(reloaded.staged_bundle.as_ref().unwrap().rule_count, 2);
        assert!(matches!(
            reloaded.last_result,
            RefreshTaskResult::Failed { .. }
        ));
    }

    #[test]
    fn refresh_validation_reports_source_metadata_and_shard_limits() {
        let root = tempdir().unwrap();
        let snapshot = refresh_easyprivacy_rules_for_subscription(
            root.path(),
            "||analytics.example^\n||tracking.example^\n||static.example^\n",
        )
        .unwrap();

        let staged = snapshot.staged_bundle.unwrap();
        assert_eq!(staged.source.id, EASYPRIVACY_SOURCE_ID);
        assert_eq!(staged.source.license, EASYPRIVACY_SOURCE_LICENSE);
        assert_eq!(staged.source.attribution, EASYPRIVACY_SOURCE_ATTRIBUTION);
        assert_eq!(staged.extension_shard_count, 1);
        assert_eq!(staged.shard_size_limit, MAX_EXTENSION_STATIC_RULES);
    }
}
