use std::process::Command;

fn temp_directory(name: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "tracker-cleaner-cli-{name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&path).unwrap();
    path
}

fn create_profile(root: &std::path::Path, name: &str, preferences: &str) {
    let profile = root.join(name);
    std::fs::create_dir_all(&profile).unwrap();
    std::fs::write(profile.join("Preferences"), preferences).unwrap();
}

#[test]
fn discover_command_prints_json_without_modifying_profiles() {
    let chrome_root = temp_directory("chrome");
    let edge_root = temp_directory("edge");
    create_profile(&chrome_root, "Default", r#"{"browser":"chrome"}"#);
    create_profile(&edge_root, "Profile 1", r#"{"browser":"edge"}"#);

    let chrome_preferences = chrome_root.join("Default").join("Preferences");
    let edge_preferences = edge_root.join("Profile 1").join("Preferences");
    let chrome_before = std::fs::read(&chrome_preferences).unwrap();
    let edge_before = std::fs::read(&edge_preferences).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_scanner-cli"))
        .args([
            "discover",
            "--chrome-root",
            chrome_root.to_str().unwrap(),
            "--edge-root",
            edge_root.to_str().unwrap(),
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["profiles"].as_array().unwrap().len(), 2);
    assert_eq!(json["warnings"].as_array().unwrap().len(), 0);
    assert_eq!(std::fs::read(chrome_preferences).unwrap(), chrome_before);
    assert_eq!(std::fs::read(edge_preferences).unwrap(), edge_before);

    std::fs::remove_dir_all(chrome_root).unwrap();
    std::fs::remove_dir_all(edge_root).unwrap();
}
