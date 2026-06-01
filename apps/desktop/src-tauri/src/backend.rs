use scanner_core::{discover_chrome_profiles, discover_edge_profiles, DiscoveryResult};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileDiscoveryRequest {
    pub chrome_root: Option<PathBuf>,
    pub edge_root: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopBootstrap {
    pub chrome: DiscoveryResult,
    pub edge: DiscoveryResult,
}

pub fn discover_profiles(request: ProfileDiscoveryRequest) -> DesktopBootstrap {
    let chrome_root = request.chrome_root.unwrap_or_else(default_chrome_root);
    let edge_root = request.edge_root.unwrap_or_else(default_edge_root);

    DesktopBootstrap {
        chrome: discover_chrome_profiles(&chrome_root),
        edge: discover_edge_profiles(&edge_root),
    }
}

fn default_chrome_root() -> PathBuf {
    windows_profile_root().join("Google").join("Chrome").join("User Data")
}

fn default_edge_root() -> PathBuf {
    windows_profile_root().join("Microsoft").join("Edge").join("User Data")
}

fn windows_profile_root() -> PathBuf {
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\Users\Public\AppData\Local"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use scanner_core::BrowserFamily;

    fn create_profile(root: &std::path::Path, name: &str) {
        let profile = root.join(name);
        std::fs::create_dir_all(&profile).unwrap();
        std::fs::write(profile.join("Preferences"), "{}").unwrap();
    }

    #[test]
    fn discover_profiles_invokes_scanner_core_for_both_browsers() {
        let temp = tempfile::tempdir().unwrap();
        let chrome_root = temp.path().join("chrome");
        let edge_root = temp.path().join("edge");
        create_profile(&chrome_root, "Default");
        create_profile(&edge_root, "Default");

        let snapshot = discover_profiles(ProfileDiscoveryRequest {
            chrome_root: Some(chrome_root),
            edge_root: Some(edge_root),
        });

        assert_eq!(snapshot.chrome.profiles.len(), 1);
        assert_eq!(snapshot.edge.profiles.len(), 1);
        assert_eq!(snapshot.chrome.profiles[0].browser, BrowserFamily::Chrome);
        assert_eq!(snapshot.edge.profiles[0].browser, BrowserFamily::Edge);
    }
}
