use rule_format::{Confidence, RuleBundle, TrackerCategory};

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
}
