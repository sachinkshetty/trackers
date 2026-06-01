use std::fmt;

use serde::{Deserialize, Serialize};

pub const SUPPORTED_MANIFEST_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateManifest {
    pub schema_version: u32,
    pub bundle_version: String,
    pub bundle_hash: String,
    pub signature: String,
    pub signing_key_id: String,
    pub minimum_client_version: String,
    pub sources: Vec<UpdateSourceMetadata>,
}

impl UpdateManifest {
    pub fn from_json(json: &str) -> Result<Self, UpdateManifestError> {
        let manifest: Self = serde_json::from_str(json)?;
        if manifest.schema_version != SUPPORTED_MANIFEST_SCHEMA_VERSION {
            return Err(UpdateManifestError::UnsupportedSchemaVersion(
                manifest.schema_version,
            ));
        }
        validate_manifest(&manifest)?;
        Ok(manifest)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSourceMetadata {
    pub id: String,
    pub name: String,
    pub url: String,
    pub license: String,
    pub attribution: String,
}

#[derive(Debug)]
pub enum UpdateManifestError {
    Json(serde_json::Error),
    UnsupportedSchemaVersion(u32),
    Validation(String),
}

impl fmt::Display for UpdateManifestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(error) => write!(formatter, "invalid update-manifest JSON: {error}"),
            Self::UnsupportedSchemaVersion(version) => write!(
                formatter,
                "unsupported update-manifest schema version {version}; supported version is {SUPPORTED_MANIFEST_SCHEMA_VERSION}"
            ),
            Self::Validation(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for UpdateManifestError {}

impl From<serde_json::Error> for UpdateManifestError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

fn validate_manifest(manifest: &UpdateManifest) -> Result<(), UpdateManifestError> {
    for (field, value) in [
        ("bundle_version", &manifest.bundle_version),
        ("bundle_hash", &manifest.bundle_hash),
        ("signature", &manifest.signature),
        ("signing_key_id", &manifest.signing_key_id),
        ("minimum_client_version", &manifest.minimum_client_version),
    ] {
        if value.trim().is_empty() {
            return Err(UpdateManifestError::Validation(format!(
                "{field} must not be empty"
            )));
        }
    }

    for source in &manifest.sources {
        validate_source(source)?;
    }

    Ok(())
}

fn validate_source(source: &UpdateSourceMetadata) -> Result<(), UpdateManifestError> {
    for (field, value) in [
        ("id", &source.id),
        ("name", &source.name),
        ("url", &source.url),
        ("license", &source.license),
        ("attribution", &source.attribution),
    ] {
        if value.trim().is_empty() {
            return Err(UpdateManifestError::Validation(format!(
                "source {field} must not be empty"
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_manifest_round_trips_as_json() {
        let manifest = UpdateManifest {
            schema_version: SUPPORTED_MANIFEST_SCHEMA_VERSION,
            bundle_version: "2026.06.01.1".into(),
            bundle_hash: "sha256:abc123".into(),
            signature: "ed25519:signature".into(),
            signing_key_id: "test-key-1".into(),
            minimum_client_version: "0.1.0".into(),
            sources: vec![UpdateSourceMetadata {
                id: "supplemental".into(),
                name: "Reviewed supplemental rules".into(),
                url: "https://github.com/sachinkshetty/trackers".into(),
                license: "MIT OR Apache-2.0".into(),
                attribution: "Browser Tracker Cleaner contributors".into(),
            }],
        };

        let encoded = serde_json::to_string(&manifest).unwrap();
        let decoded = UpdateManifest::from_json(&encoded).unwrap();

        assert_eq!(decoded, manifest);
    }

    #[test]
    fn update_manifest_rejects_unsupported_schema_versions() {
        let json = r#"{
            "schemaVersion": 99,
            "bundleVersion": "future",
            "bundleHash": "sha256:abc123",
            "signature": "ed25519:signature",
            "signingKeyId": "test-key-1",
            "minimumClientVersion": "0.1.0",
            "sources": []
        }"#;

        let error = UpdateManifest::from_json(json).unwrap_err();

        assert_eq!(
            error.to_string(),
            "unsupported update-manifest schema version 99; supported version is 1"
        );
    }

    #[test]
    fn update_manifest_rejects_missing_metadata() {
        let json = r#"{
            "schemaVersion": 1,
            "bundleVersion": "",
            "bundleHash": "sha256:abc123",
            "signature": "ed25519:signature",
            "signingKeyId": "test-key-1",
            "minimumClientVersion": "0.1.0",
            "sources": [{
                "id": "supplemental",
                "name": "Reviewed supplemental rules",
                "url": "https://github.com/sachinkshetty/trackers",
                "license": "MIT OR Apache-2.0",
                "attribution": "Browser Tracker Cleaner contributors"
            }]
        }"#;

        let error = UpdateManifest::from_json(json).unwrap_err();

        assert_eq!(error.to_string(), "bundle_version must not be empty");
    }
}
