use crate::backend::{discover_profiles as discover_profiles_snapshot, DesktopBootstrap, ProfileDiscoveryRequest};

#[tauri::command]
pub fn discover_profiles(request: ProfileDiscoveryRequest) -> DesktopBootstrap {
    discover_profiles_snapshot(request)
}
