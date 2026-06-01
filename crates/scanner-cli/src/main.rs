use std::path::PathBuf;

use scanner_core::{DiscoveryResult, discover_chrome_profiles, discover_edge_profiles};

fn main() {
    match run(std::env::args().skip(1)) {
        Ok(output) => println!("{output}"),
        Err(message) => {
            eprintln!("{message}");
            std::process::exit(2);
        }
    }
}

fn run(arguments: impl Iterator<Item = String>) -> Result<String, String> {
    let mut arguments = arguments.peekable();
    match arguments.next().as_deref() {
        Some("discover") => discover(arguments),
        _ => Err("usage: scanner-cli discover [--chrome-root PATH] [--edge-root PATH]".into()),
    }
}

fn discover(mut arguments: impl Iterator<Item = String>) -> Result<String, String> {
    let mut chrome_root = None;
    let mut edge_root = None;
    while let Some(argument) = arguments.next() {
        let target = match argument.as_str() {
            "--chrome-root" => &mut chrome_root,
            "--edge-root" => &mut edge_root,
            _ => return Err(format!("unknown argument: {argument}")),
        };
        *target = Some(
            arguments
                .next()
                .map(PathBuf::from)
                .ok_or_else(|| format!("missing path after {argument}"))?,
        );
    }

    let local_app_data = std::env::var_os("LOCALAPPDATA").map(PathBuf::from);
    let chrome_root = chrome_root
        .or_else(|| {
            local_app_data
                .as_ref()
                .map(|root| root.join("Google").join("Chrome").join("User Data"))
        })
        .ok_or("LOCALAPPDATA is not set; pass --chrome-root")?;
    let edge_root = edge_root
        .or_else(|| {
            local_app_data
                .as_ref()
                .map(|root| root.join("Microsoft").join("Edge").join("User Data"))
        })
        .ok_or("LOCALAPPDATA is not set; pass --edge-root")?;

    let mut result = DiscoveryResult::default();
    merge(&mut result, discover_chrome_profiles(&chrome_root));
    merge(&mut result, discover_edge_profiles(&edge_root));
    serde_json::to_string_pretty(&result).map_err(|error| error.to_string())
}

fn merge(target: &mut DiscoveryResult, mut source: DiscoveryResult) {
    target.profiles.append(&mut source.profiles);
    target.warnings.append(&mut source.warnings);
}
