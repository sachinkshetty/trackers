use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const DEFAULT_INTERVAL_DAYS: u32 = 7;
const MAX_INTERVAL_DAYS: u32 = 365;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SchedulerTaskResult {
    NeverRun,
    Succeeded { message: String },
    Failed { message: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SchedulerTaskState {
    enabled: bool,
    interval_days: u32,
    last_run_at_ms: Option<u64>,
    last_result: SchedulerTaskResult,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SchedulerStateFile {
    rule_refresh: SchedulerTaskState,
    rescan: SchedulerTaskState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchedulerTaskSnapshot {
    pub enabled: bool,
    pub interval_days: u32,
    pub last_run_at_ms: Option<u64>,
    pub next_run_at_ms: Option<u64>,
    pub last_result: SchedulerTaskResult,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchedulerSnapshot {
    pub rule_refresh: SchedulerTaskSnapshot,
    pub rescan: SchedulerTaskSnapshot,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchedulerUpdateRequest {
    pub rule_refresh_enabled: Option<bool>,
    pub rule_refresh_interval_days: Option<u32>,
    pub rescan_enabled: Option<bool>,
    pub rescan_interval_days: Option<u32>,
}

pub fn scheduler_snapshot() -> Result<SchedulerSnapshot, String> {
    scheduler_snapshot_for_path(&scheduler_state_path())
}

pub fn scheduler_snapshot_for_path(path: &Path) -> Result<SchedulerSnapshot, String> {
    let state = load_scheduler_state_for_path(path)?;
    Ok(snapshot_from_state(&state, current_timestamp_ms()))
}

pub fn update_scheduler_settings(
    request: SchedulerUpdateRequest,
) -> Result<SchedulerSnapshot, String> {
    update_scheduler_settings_for_path(&scheduler_state_path(), request)
}

pub fn update_scheduler_settings_for_path(
    path: &Path,
    request: SchedulerUpdateRequest,
) -> Result<SchedulerSnapshot, String> {
    let mut state = load_scheduler_state_for_path(path)?;

    if let Some(enabled) = request.rule_refresh_enabled {
        state.rule_refresh.enabled = enabled;
    }
    if let Some(interval_days) = request.rule_refresh_interval_days {
        state.rule_refresh.interval_days = validate_interval_days(interval_days)?;
    }
    if let Some(enabled) = request.rescan_enabled {
        state.rescan.enabled = enabled;
    }
    if let Some(interval_days) = request.rescan_interval_days {
        state.rescan.interval_days = validate_interval_days(interval_days)?;
    }

    save_scheduler_state(path, &state)?;
    Ok(snapshot_from_state(&state, current_timestamp_ms()))
}

fn load_scheduler_state_for_path(path: &Path) -> Result<SchedulerStateFile, String> {
    if !path.exists() {
        return Ok(default_scheduler_state());
    }

    let contents = std::fs::read_to_string(path).map_err(|error| error.to_string())?;
    serde_json::from_str(&contents).map_err(|error| error.to_string())
}

fn save_scheduler_state(path: &Path, state: &SchedulerStateFile) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let json = serde_json::to_string_pretty(state).map_err(|error| error.to_string())?;
    std::fs::write(path, json).map_err(|error| error.to_string())
}

fn snapshot_from_state(state: &SchedulerStateFile, now_ms: u64) -> SchedulerSnapshot {
    SchedulerSnapshot {
        rule_refresh: snapshot_task(&state.rule_refresh, now_ms),
        rescan: snapshot_task(&state.rescan, now_ms),
    }
}

fn snapshot_task(state: &SchedulerTaskState, now_ms: u64) -> SchedulerTaskSnapshot {
    SchedulerTaskSnapshot {
        enabled: state.enabled,
        interval_days: state.interval_days,
        last_run_at_ms: state.last_run_at_ms,
        next_run_at_ms: if state.enabled {
            Some(
                state
                    .last_run_at_ms
                    .unwrap_or(now_ms)
                    .saturating_add(interval_days_to_ms(state.interval_days)),
            )
        } else {
            None
        },
        last_result: state.last_result.clone(),
    }
}

fn default_scheduler_state() -> SchedulerStateFile {
    SchedulerStateFile {
        rule_refresh: default_task_state(),
        rescan: default_task_state(),
    }
}

fn default_task_state() -> SchedulerTaskState {
    SchedulerTaskState {
        enabled: false,
        interval_days: DEFAULT_INTERVAL_DAYS,
        last_run_at_ms: None,
        last_result: SchedulerTaskResult::NeverRun,
    }
}

fn validate_interval_days(value: u32) -> Result<u32, String> {
    if value == 0 {
        return Err("scheduled interval must be at least 1 day".into());
    }
    if value > MAX_INTERVAL_DAYS {
        return Err(format!(
            "scheduled interval must not exceed {MAX_INTERVAL_DAYS} days"
        ));
    }
    Ok(value)
}

fn interval_days_to_ms(days: u32) -> u64 {
    u64::from(days)
        .saturating_mul(24)
        .saturating_mul(60)
        .saturating_mul(60)
        .saturating_mul(1000)
}

pub fn scheduler_state_path() -> PathBuf {
    let base = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    base.join("Trackers").join("scheduler-state.json")
}

fn current_timestamp_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scheduler_snapshot_defaults_are_conservative() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("scheduler.json");

        let snapshot = scheduler_snapshot_for_path(&path).unwrap();

        assert!(!snapshot.rule_refresh.enabled);
        assert!(!snapshot.rescan.enabled);
        assert_eq!(snapshot.rule_refresh.interval_days, DEFAULT_INTERVAL_DAYS);
        assert_eq!(snapshot.rescan.interval_days, DEFAULT_INTERVAL_DAYS);
        assert!(snapshot.rule_refresh.next_run_at_ms.is_none());
        assert!(snapshot.rescan.next_run_at_ms.is_none());
        assert!(matches!(
            snapshot.rule_refresh.last_result,
            SchedulerTaskResult::NeverRun
        ));
    }

    #[test]
    fn scheduler_settings_persist_and_compute_next_run() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("scheduler.json");

        let snapshot = update_scheduler_settings_for_path(
            &path,
            SchedulerUpdateRequest {
                rule_refresh_enabled: Some(true),
                rule_refresh_interval_days: Some(14),
                rescan_enabled: Some(true),
                rescan_interval_days: Some(3),
            },
        )
        .unwrap();

        assert!(snapshot.rule_refresh.enabled);
        assert!(snapshot.rescan.enabled);
        assert_eq!(snapshot.rule_refresh.interval_days, 14);
        assert_eq!(snapshot.rescan.interval_days, 3);
        assert!(snapshot.rule_refresh.next_run_at_ms.is_some());
        assert!(snapshot.rescan.next_run_at_ms.is_some());

        let reloaded = scheduler_snapshot_for_path(&path).unwrap();
        assert_eq!(reloaded, snapshot);
    }

    #[test]
    fn scheduler_settings_reject_zero_intervals() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("scheduler.json");

        let error = update_scheduler_settings_for_path(
            &path,
            SchedulerUpdateRequest {
                rule_refresh_interval_days: Some(0),
                ..SchedulerUpdateRequest::default()
            },
        )
        .unwrap_err();

        assert!(error.contains("at least 1 day"));
    }
}
