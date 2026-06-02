use crate::{backend::DesktopBootstrap, cleanup::CleanupPreviewResult, scan::ScanRunResult};
use std::sync::Mutex;

#[derive(Debug, Default)]
pub struct AppState {
    inner: Mutex<AppStateSnapshot>,
}

#[derive(Debug, Default, Clone)]
struct AppStateSnapshot {
    discovery: Option<DesktopBootstrap>,
    scan: Option<ScanRunResult>,
    cleanup_preview: Option<CleanupPreviewResult>,
}

impl AppState {
    pub fn replace_discovery(&self, discovery: DesktopBootstrap) {
        self.with_snapshot(|snapshot| snapshot.discovery = Some(discovery));
    }

    pub fn latest_discovery(&self) -> Option<DesktopBootstrap> {
        self.with_snapshot(|snapshot| snapshot.discovery.clone())
    }

    pub fn replace_scan(&self, scan: ScanRunResult) {
        self.with_snapshot(|snapshot| snapshot.scan = Some(scan));
    }

    pub fn latest_scan(&self) -> Option<ScanRunResult> {
        self.with_snapshot(|snapshot| snapshot.scan.clone())
    }

    pub fn replace_cleanup_preview(&self, preview: CleanupPreviewResult) {
        self.with_snapshot(|snapshot| snapshot.cleanup_preview = Some(preview));
    }

    pub fn latest_cleanup_preview(&self) -> Option<CleanupPreviewResult> {
        self.with_snapshot(|snapshot| snapshot.cleanup_preview.clone())
    }

    pub fn clear_cleanup_preview(&self) {
        self.with_snapshot(|snapshot| snapshot.cleanup_preview = None);
    }

    fn with_snapshot<R>(&self, callback: impl FnOnce(&mut AppStateSnapshot) -> R) -> R {
        let mut snapshot = self.inner.lock().expect("app state poisoned");
        callback(&mut snapshot)
    }
}
