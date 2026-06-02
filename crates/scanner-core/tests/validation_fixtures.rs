use rule_format::{RuleBundle, RuleSource, SUPPORTED_SCHEMA_VERSION};
use scanner_core::{
    BrowserFamily, BrowserProfile, discover_chrome_profiles, discover_edge_profiles,
    inventory_extensions, inventory_site_storage, scan_cookies,
};
use std::path::{Path, PathBuf};

struct DisposableFixtureRoots {
    root: PathBuf,
    chrome: PathBuf,
    edge: PathBuf,
}

impl DisposableFixtureRoots {
    fn new() -> Self {
        let root = temp_directory("validation-fixtures");
        let chrome = root.join("Chrome User Data");
        let edge = root.join("Edge User Data");
        std::fs::create_dir_all(&chrome).unwrap();
        std::fs::create_dir_all(&edge).unwrap();

        Self { root, chrome, edge }
    }
}

impl Drop for DisposableFixtureRoots {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

struct ProfileFixture {
    root: PathBuf,
    profile_path: PathBuf,
}

impl ProfileFixture {
    fn new(root: &Path, name: &str) -> Self {
        let profile_path = root.join(name);
        std::fs::create_dir_all(&profile_path).unwrap();
        std::fs::write(profile_path.join("Preferences"), "{}").unwrap();
        Self {
            root: root.to_path_buf(),
            profile_path,
        }
    }

    fn browser_profile(&self, browser: BrowserFamily) -> BrowserProfile {
        BrowserProfile {
            browser,
            installation_root: self.root.clone(),
            profile_name: self
                .profile_path
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into_owned(),
            profile_path: self.profile_path.clone(),
        }
    }

    fn seed_cookie(&self, host: &str, name: &str, value: &str) {
        let network = self.profile_path.join("Network");
        std::fs::create_dir_all(&network).unwrap();
        let database = network.join("Cookies");
        let connection = rusqlite::Connection::open(&database).unwrap();
        connection
            .execute(
                "CREATE TABLE cookies (host_key TEXT NOT NULL, name TEXT NOT NULL, value TEXT)",
                [],
            )
            .unwrap();
        connection
            .execute(
                "INSERT INTO cookies VALUES (?1, ?2, ?3)",
                rusqlite::params![host, name, value],
            )
            .unwrap();
    }

    fn seed_storage(&self) {
        for directory in ["Local Storage", "IndexedDB", "Cache", "Service Worker"] {
            std::fs::create_dir_all(self.profile_path.join(directory)).unwrap();
        }
        std::fs::write(self.profile_path.join("History"), "fixture").unwrap();
    }

    fn seed_extension(&self, id: &str, display_name: &str) {
        let version_root = self
            .profile_path
            .join("Extensions")
            .join(id)
            .join("1.0.0_0");
        std::fs::create_dir_all(&version_root).unwrap();
        std::fs::write(
            version_root.join("manifest.json"),
            format!(r#"{{"name":"{display_name}","version":"1.0.0"}}"#),
        )
        .unwrap();
        std::fs::write(
            self.profile_path.join("Preferences"),
            format!(r#"{{"extensions":{{"settings":{{"{id}":{{"state":1}}}}}}}}"#),
        )
        .unwrap();
    }

    fn seed_malformed_preferences(&self) {
        std::fs::write(self.profile_path.join("Preferences"), "not json").unwrap();
    }

    fn seed_malformed_cookie_database(&self) {
        let network = self.profile_path.join("Network");
        std::fs::create_dir_all(&network).unwrap();
        std::fs::write(network.join("Cookies"), "not sqlite").unwrap();
    }
}

fn temp_directory(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "tracker-cleaner-{name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&path).unwrap();
    path
}

fn tracker_bundle() -> RuleBundle {
    RuleBundle {
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
        rules: vec![],
    }
}

#[test]
fn disposable_fixtures_stay_under_temp_and_cover_supported_artifacts() {
    let roots = DisposableFixtureRoots::new();

    let chrome_default = ProfileFixture::new(&roots.chrome, "Default");
    chrome_default.seed_cookie("analytics.example.test", "session", "secret-token");
    chrome_default.seed_storage();

    let chrome_named = ProfileFixture::new(&roots.chrome, "Profile 1");
    chrome_named.seed_malformed_cookie_database();
    chrome_named.seed_malformed_preferences();

    let edge_default = ProfileFixture::new(&roots.edge, "Default");
    edge_default.seed_extension("abcdefghijklmnopabcdefghijklmnop", "Fixture Extension");

    let chrome_profiles = discover_chrome_profiles(&roots.chrome);
    let edge_profiles = discover_edge_profiles(&roots.edge);

    assert!(roots.root.starts_with(std::env::temp_dir()));
    assert_eq!(chrome_profiles.profiles.len(), 2);
    assert_eq!(edge_profiles.profiles.len(), 1);

    let cookie_scan = scan_cookies(
        &chrome_default.browser_profile(BrowserFamily::Chrome),
        &tracker_bundle(),
    );
    let storage_scan = inventory_site_storage(
        &chrome_default.browser_profile(BrowserFamily::Chrome),
        &tracker_bundle(),
    );
    let extension_scan = inventory_extensions(&edge_default.browser_profile(BrowserFamily::Edge));

    assert_eq!(cookie_scan.findings.len(), 1);
    assert_eq!(
        cookie_scan.findings[0].site.as_deref(),
        Some("analytics.example.test")
    );
    assert_eq!(storage_scan.findings.len(), 5);
    assert_eq!(extension_scan.extensions.len(), 1);
    assert_eq!(
        extension_scan.extensions[0].display_name.as_deref(),
        Some("Fixture Extension")
    );
    assert!(
        extension_scan
            .extensions
            .iter()
            .all(|extension| extension.enabled)
    );
}

#[test]
fn malformed_fixture_inputs_produce_scoped_warnings() {
    let roots = DisposableFixtureRoots::new();
    let chrome_named = ProfileFixture::new(&roots.chrome, "Profile 1");
    chrome_named.seed_malformed_cookie_database();
    chrome_named.seed_malformed_preferences();

    let cookie_scan = scan_cookies(
        &chrome_named.browser_profile(BrowserFamily::Chrome),
        &tracker_bundle(),
    );
    let extension_scan = inventory_extensions(&chrome_named.browser_profile(BrowserFamily::Chrome));

    assert_eq!(cookie_scan.warnings.len(), 1);
    assert_eq!(extension_scan.warnings.len(), 1);
    assert!(
        cookie_scan.warnings[0]
            .message
            .contains("could not read copied cookie database")
    );
    assert!(
        extension_scan.warnings[0]
            .message
            .contains("could not parse Preferences")
    );
}
