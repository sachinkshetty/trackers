use crate::scan::ScanRunResult;
use scanner_core::{
    AggressiveConfirmation, CleanupMode, CleanupPlan, CleanupPlanError, plan_aggressive_cleanup,
    plan_balanced_cleanup, plan_review_cleanup,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupPreviewRequest {
    pub scan_result: ScanRunResult,
    pub mode: CleanupMode,
    pub selected_finding_ids: Vec<String>,
    pub aggressive_confirmed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupPreviewResult {
    pub plan: CleanupPlan,
    pub locked_action_ids: Vec<String>,
    pub requires_confirmation: bool,
    pub warnings: Vec<String>,
}

pub fn preview_cleanup(request: CleanupPreviewRequest) -> Result<CleanupPreviewResult, String> {
    let findings = request
        .scan_result
        .profiles
        .iter()
        .flat_map(|profile| profile.findings.iter().cloned())
        .collect::<Vec<_>>();

    let plan = match request.mode {
        CleanupMode::Review => plan_review_cleanup(&findings, &request.selected_finding_ids),
        CleanupMode::Balanced => plan_balanced_cleanup(&findings, &[]),
        CleanupMode::Aggressive => plan_aggressive_cleanup(
            &findings,
            if request.aggressive_confirmed {
                AggressiveConfirmation::Confirmed
            } else {
                AggressiveConfirmation::NotConfirmed
            },
        ),
    }
    .map_err(|error: CleanupPlanError| error.to_string())?;

    let locked_action_ids = plan
        .actions
        .iter()
        .filter(|action| action.requires_browser_closed)
        .map(|action| action.id.clone())
        .collect::<Vec<_>>();

    Ok(CleanupPreviewResult {
        warnings: plan.warnings.clone(),
        locked_action_ids,
        requires_confirmation: matches!(request.mode, CleanupMode::Aggressive)
            && !request.aggressive_confirmed,
        plan,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scan::{ProfileScanSummary, ScanRunResult};
    use rule_format::Confidence;
    use scanner_core::{ArtifactType, BrowserFamily, BrowserProfile, CleanupImpact, Finding};
    use std::path::PathBuf;

    fn profile() -> BrowserProfile {
        BrowserProfile {
            browser: BrowserFamily::Chrome,
            installation_root: PathBuf::from(r"C:\Chrome\User Data"),
            profile_name: "Default".into(),
            profile_path: PathBuf::from(r"C:\Chrome\User Data\Default"),
        }
    }

    fn scan_result() -> ScanRunResult {
        ScanRunResult {
            completed_profiles: 1,
            total_profiles: 1,
            cancelled: false,
            profiles: vec![ProfileScanSummary {
                browser: "chrome".into(),
                profile_name: "Default".into(),
                profile_path: PathBuf::from(r"C:\Chrome\User Data\Default"),
                warnings: vec![],
                findings: vec![Finding {
                    id: "cookie:analytics.example".into(),
                    profile: profile(),
                    artifact_type: ArtifactType::Cookie,
                    site: Some("analytics.example".into()),
                    evidence_summary: "cookie host matched tracker rule".into(),
                    confidence: Some(Confidence::High),
                    cleanup_impact: CleanupImpact::MaySignOut,
                }],
            }],
            findings: vec![],
            warnings: vec![],
        }
    }

    #[test]
    fn review_preview_includes_selected_actions_and_locked_choices() {
        let result = preview_cleanup(CleanupPreviewRequest {
            scan_result: scan_result(),
            mode: CleanupMode::Review,
            selected_finding_ids: vec!["cookie:analytics.example".into()],
            aggressive_confirmed: false,
        })
        .unwrap();

        assert_eq!(result.plan.mode, CleanupMode::Review);
        assert_eq!(result.plan.actions.len(), 1);
        assert_eq!(result.locked_action_ids, vec!["cookie:analytics.example"]);
        assert!(!result.requires_confirmation);
    }

    #[test]
    fn aggressive_preview_requires_confirmation() {
        let result = preview_cleanup(CleanupPreviewRequest {
            scan_result: scan_result(),
            mode: CleanupMode::Aggressive,
            selected_finding_ids: vec!["cookie:analytics.example".into()],
            aggressive_confirmed: true,
        })
        .unwrap();

        assert_eq!(result.plan.mode, CleanupMode::Aggressive);
        assert!(
            result
                .warnings
                .iter()
                .any(|warning| warning.contains("sign the user out"))
        );
    }
}
