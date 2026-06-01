use crate::scan::embedded_rule_bundle;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UpdateState {
    EmbeddedStarterBundle,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopSettingsSnapshot {
    pub rule_bundle_version: String,
    pub update_state: UpdateState,
}

pub fn settings_snapshot() -> DesktopSettingsSnapshot {
    DesktopSettingsSnapshot {
        rule_bundle_version: embedded_rule_bundle().bundle_version,
        update_state: UpdateState::EmbeddedStarterBundle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_snapshot_exposes_rule_version_and_update_state() {
        let snapshot = settings_snapshot();

        assert_eq!(snapshot.rule_bundle_version, "embedded-starter");
        assert!(matches!(
            snapshot.update_state,
            UpdateState::EmbeddedStarterBundle
        ));
    }
}
