use std::collections::BTreeMap;

use rule_format::{
    Confidence, RuleBundle, RuleSource, SUPPORTED_SCHEMA_VERSION, SupplementalRuleSet, TrackerRule,
};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledArtifacts {
    pub desktop_json: String,
    pub extension_json: String,
}

pub fn compile_rules(
    bundle_version: &str,
    generated_at: &str,
    inputs: &[SupplementalRuleSet],
) -> CompiledArtifacts {
    let mut sources = BTreeMap::new();
    let mut rules = BTreeMap::new();

    for input in inputs {
        sources.insert(input.source.id.clone(), input.source.clone());
        for rule in &input.rules {
            let candidate = TrackerRule {
                id: format!("{}:{}", input.source.id, rule.domain),
                domain: rule.domain.clone(),
                category: rule.category,
                confidence: rule.confidence,
                source_id: input.source.id.clone(),
            };
            match rules.get(&rule.domain) {
                Some(existing) if !prefer_candidate(&candidate, existing) => {}
                _ => {
                    rules.insert(rule.domain.clone(), candidate);
                }
            }
        }
    }

    let rules: Vec<_> = rules.into_values().collect();
    let bundle = RuleBundle {
        schema_version: SUPPORTED_SCHEMA_VERSION,
        bundle_version: bundle_version.into(),
        generated_at: generated_at.into(),
        sources: sources.into_values().collect::<Vec<RuleSource>>(),
        rules: rules.clone(),
    };
    let extension_rules = rules
        .iter()
        .enumerate()
        .map(|(index, rule)| ExtensionRule::block((index + 1) as u32, &rule.domain))
        .collect::<Vec<_>>();

    CompiledArtifacts {
        desktop_json: serde_json::to_string(&bundle).expect("rule bundle is serializable"),
        extension_json: serde_json::to_string(&extension_rules)
            .expect("extension rules are serializable"),
    }
}

fn prefer_candidate(candidate: &TrackerRule, existing: &TrackerRule) -> bool {
    confidence_rank(candidate.confidence) > confidence_rank(existing.confidence)
        || (candidate.confidence == existing.confidence && candidate.source_id < existing.source_id)
}

fn confidence_rank(confidence: Confidence) -> u8 {
    match confidence {
        Confidence::Low => 1,
        Confidence::Medium => 2,
        Confidence::High => 3,
    }
}

#[derive(Serialize)]
struct ExtensionRule {
    id: u32,
    priority: u32,
    action: ExtensionAction,
    condition: ExtensionCondition,
}

impl ExtensionRule {
    fn block(id: u32, domain: &str) -> Self {
        Self {
            id,
            priority: 1,
            action: ExtensionAction { kind: "block" },
            condition: ExtensionCondition {
                url_filter: format!("||{domain}^"),
                resource_types: ["script", "image", "xmlhttprequest", "sub_frame"],
            },
        }
    }
}

#[derive(Serialize)]
struct ExtensionAction {
    #[serde(rename = "type")]
    kind: &'static str,
}

#[derive(Serialize)]
struct ExtensionCondition {
    url_filter: String,
    resource_types: [&'static str; 4],
}

#[cfg(test)]
mod tests {
    use super::*;
    use rule_format::{
        Confidence, RuleSource, SupplementalRule, SupplementalRuleSet, TrackerCategory,
    };

    fn source(id: &str) -> RuleSource {
        RuleSource {
            id: id.into(),
            name: format!("{id} rules"),
            url: format!("https://example.test/{id}"),
            license: "MIT".into(),
            attribution: format!("{id} contributors"),
        }
    }

    fn input(id: &str, rules: Vec<SupplementalRule>) -> SupplementalRuleSet {
        SupplementalRuleSet {
            source: source(id),
            rules,
        }
    }

    fn rule(domain: &str, confidence: Confidence) -> SupplementalRule {
        SupplementalRule {
            domain: domain.into(),
            category: TrackerCategory::Analytics,
            confidence,
        }
    }

    #[test]
    fn compiler_output_is_independent_of_input_order() {
        let first = input("first", vec![rule("z.example", Confidence::High)]);
        let second = input("second", vec![rule("a.example", Confidence::Medium)]);

        let forward = compile_rules(
            "2026.06.01.1",
            "2026-06-01T00:00:00Z",
            &[first.clone(), second.clone()],
        );
        let reversed = compile_rules("2026.06.01.1", "2026-06-01T00:00:00Z", &[second, first]);

        assert_eq!(forward, reversed);
    }

    #[test]
    fn duplicate_domain_keeps_the_higher_confidence_rule() {
        let low = input("low", vec![rule("analytics.example", Confidence::Low)]);
        let high = input("high", vec![rule("analytics.example", Confidence::High)]);

        let artifacts = compile_rules("2026.06.01.1", "2026-06-01T00:00:00Z", &[low, high]);

        assert!(artifacts.desktop_json.contains(r#""source_id":"high""#));
        assert!(!artifacts.desktop_json.contains(r#""source_id":"low""#));
    }

    #[test]
    fn extension_rule_ids_are_stable_and_domains_are_sorted() {
        let rules = input(
            "supplemental",
            vec![
                rule("z.example", Confidence::High),
                rule("a.example", Confidence::High),
            ],
        );

        let artifacts = compile_rules("2026.06.01.1", "2026-06-01T00:00:00Z", &[rules]);

        assert_eq!(
            artifacts.extension_json,
            r#"[{"id":1,"priority":1,"action":{"type":"block"},"condition":{"url_filter":"||a.example^","resource_types":["script","image","xmlhttprequest","sub_frame"]}},{"id":2,"priority":1,"action":{"type":"block"},"condition":{"url_filter":"||z.example^","resource_types":["script","image","xmlhttprequest","sub_frame"]}}]"#
        );
    }
}
