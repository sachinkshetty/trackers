use std::fmt;

use base64::Engine;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use rule_format::{RuleBundle, RuleBundleError};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrustedSigningKey {
    pub key_id: String,
    pub public_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedUpdateBundle {
    pub manifest: UpdateManifest,
    pub bundle: RuleBundle,
}

#[derive(Debug)]
pub enum UpdateVerificationError {
    Manifest(UpdateManifestError),
    Bundle(RuleBundleError),
    UnknownSigningKey(String),
    InvalidHashFormat(String),
    HashMismatch { expected: String, actual: String },
    InvalidPublicKey(String),
    InvalidSignatureFormat(String),
    InvalidSignature(String),
    SignatureMismatch,
}

impl fmt::Display for UpdateVerificationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Manifest(error) => write!(formatter, "{error}"),
            Self::Bundle(error) => write!(formatter, "{error}"),
            Self::UnknownSigningKey(key_id) => {
                write!(formatter, "unknown signing key '{key_id}'")
            }
            Self::InvalidHashFormat(value) => {
                write!(formatter, "invalid update hash '{value}'")
            }
            Self::HashMismatch { expected, actual } => {
                write!(
                    formatter,
                    "update hash mismatch: expected {expected}, got {actual}"
                )
            }
            Self::InvalidPublicKey(value) => {
                write!(formatter, "invalid signing public key '{value}'")
            }
            Self::InvalidSignatureFormat(value) => {
                write!(formatter, "invalid signature '{value}'")
            }
            Self::InvalidSignature(value) => {
                write!(formatter, "invalid signature bytes '{value}'")
            }
            Self::SignatureMismatch => formatter.write_str("update signature did not verify"),
        }
    }
}

impl std::error::Error for UpdateVerificationError {}

impl From<UpdateManifestError> for UpdateVerificationError {
    fn from(error: UpdateManifestError) -> Self {
        Self::Manifest(error)
    }
}

impl From<RuleBundleError> for UpdateVerificationError {
    fn from(error: RuleBundleError) -> Self {
        Self::Bundle(error)
    }
}

pub fn verify_update_bundle(
    manifest_json: &str,
    bundle_json: &str,
    trusted_keys: &[TrustedSigningKey],
) -> Result<VerifiedUpdateBundle, UpdateVerificationError> {
    let manifest = UpdateManifest::from_json(manifest_json)?;
    let bundle = RuleBundle::from_json(bundle_json)?;
    let actual_hash = hash_bundle(bundle_json.as_bytes());
    let expected_hash = parse_hash(&manifest.bundle_hash)?;

    if actual_hash != expected_hash {
        return Err(UpdateVerificationError::HashMismatch {
            expected: format!("sha256:{expected_hash}"),
            actual: format!("sha256:{actual_hash}"),
        });
    }

    let trusted_key = trusted_keys
        .iter()
        .find(|key| key.key_id == manifest.signing_key_id)
        .ok_or_else(|| {
            UpdateVerificationError::UnknownSigningKey(manifest.signing_key_id.clone())
        })?;

    let public_key_bytes = parse_public_key(&trusted_key.public_key)?;
    let verifying_key = VerifyingKey::from_bytes(&public_key_bytes)
        .map_err(|_| UpdateVerificationError::InvalidPublicKey(trusted_key.public_key.clone()))?;

    let signature_bytes = parse_signature(&manifest.signature)?;
    let signature = Signature::from_slice(&signature_bytes)
        .map_err(|_| UpdateVerificationError::InvalidSignature(manifest.signature.clone()))?;

    verifying_key
        .verify(manifest.bundle_hash.as_bytes(), &signature)
        .map_err(|_| UpdateVerificationError::SignatureMismatch)?;

    Ok(VerifiedUpdateBundle { manifest, bundle })
}

fn hash_bundle(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn parse_hash(value: &str) -> Result<String, UpdateVerificationError> {
    let Some(hash) = value.strip_prefix("sha256:") else {
        return Err(UpdateVerificationError::InvalidHashFormat(value.into()));
    };
    if hash.len() != 64 || !hash.chars().all(|character| character.is_ascii_hexdigit()) {
        return Err(UpdateVerificationError::InvalidHashFormat(value.into()));
    }
    Ok(hash.to_ascii_lowercase())
}

fn parse_public_key(value: &str) -> Result<[u8; 32], UpdateVerificationError> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(value)
        .map_err(|_| UpdateVerificationError::InvalidPublicKey(value.into()))?;
    bytes
        .try_into()
        .map_err(|_| UpdateVerificationError::InvalidPublicKey(value.into()))
}

fn parse_signature(value: &str) -> Result<Vec<u8>, UpdateVerificationError> {
    let Some(encoded) = value.strip_prefix("ed25519:") else {
        return Err(UpdateVerificationError::InvalidSignatureFormat(
            value.into(),
        ));
    };
    base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .map_err(|_| UpdateVerificationError::InvalidSignatureFormat(value.into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};

    fn source_metadata() -> UpdateSourceMetadata {
        UpdateSourceMetadata {
            id: "supplemental".into(),
            name: "Reviewed supplemental rules".into(),
            url: "https://github.com/sachinkshetty/trackers".into(),
            license: "MIT OR Apache-2.0".into(),
            attribution: "Browser Tracker Cleaner contributors".into(),
        }
    }

    fn make_manifest(bundle_hash: String, signature: String, key_id: String) -> UpdateManifest {
        UpdateManifest {
            schema_version: SUPPORTED_MANIFEST_SCHEMA_VERSION,
            bundle_version: "2026.06.01.1".into(),
            bundle_hash,
            signature,
            signing_key_id: key_id,
            minimum_client_version: "0.1.0".into(),
            sources: vec![source_metadata()],
        }
    }

    fn signing_key() -> SigningKey {
        SigningKey::from_bytes(&[7u8; 32])
    }

    fn trusted_key() -> TrustedSigningKey {
        let verifying_key = signing_key().verifying_key();
        TrustedSigningKey {
            key_id: "test-key-1".into(),
            public_key: base64::engine::general_purpose::STANDARD.encode(verifying_key.to_bytes()),
        }
    }

    fn bundle_json() -> String {
        serde_json::json!({
            "schema_version": 1,
            "bundle_version": "2026.06.01.1",
            "generated_at": "2026-06-01T00:00:00Z",
            "sources": [{
                "id": "supplemental",
                "name": "Reviewed supplemental rules",
                "url": "https://github.com/sachinkshetty/trackers",
                "license": "MIT OR Apache-2.0",
                "attribution": "Browser Tracker Cleaner contributors"
            }],
            "rules": []
        })
        .to_string()
    }

    #[test]
    fn verify_update_bundle_accepts_matching_hash_and_signature() {
        let bundle_json = bundle_json();
        let hash = format!("sha256:{}", hash_bundle(bundle_json.as_bytes()));
        let key = signing_key();
        let signature = key.sign(hash.as_bytes());
        let manifest = make_manifest(
            hash,
            format!(
                "ed25519:{}",
                base64::engine::general_purpose::STANDARD.encode(signature.to_bytes())
            ),
            trusted_key().key_id,
        );

        let verified = verify_update_bundle(
            &serde_json::to_string(&manifest).unwrap(),
            &bundle_json,
            &[trusted_key()],
        )
        .unwrap();

        assert_eq!(verified.manifest.bundle_version, "2026.06.01.1");
        assert_eq!(verified.bundle.bundle_version, "2026.06.01.1");
    }

    #[test]
    fn verify_update_bundle_rejects_hash_mismatch() {
        let bundle_json = bundle_json();
        let hash: String =
            "sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".into();
        let key = signing_key();
        let signature = key.sign(hash.as_bytes());
        let manifest = make_manifest(
            hash,
            format!(
                "ed25519:{}",
                base64::engine::general_purpose::STANDARD.encode(signature.to_bytes())
            ),
            trusted_key().key_id,
        );

        let error = verify_update_bundle(
            &serde_json::to_string(&manifest).unwrap(),
            &bundle_json,
            &[trusted_key()],
        )
        .unwrap_err();

        assert!(matches!(
            error,
            UpdateVerificationError::HashMismatch { .. }
        ));
    }

    #[test]
    fn verify_update_bundle_rejects_unknown_key_and_malformed_fields() {
        let bundle_json = bundle_json();
        let hash = format!("sha256:{}", hash_bundle(bundle_json.as_bytes()));
        let key = signing_key();
        let signature = key.sign(hash.as_bytes());
        let manifest = make_manifest(
            hash,
            format!(
                "ed25519:{}",
                base64::engine::general_purpose::STANDARD.encode(signature.to_bytes())
            ),
            "missing-key".into(),
        );

        let error = verify_update_bundle(
            &serde_json::to_string(&manifest).unwrap(),
            &bundle_json,
            &[trusted_key()],
        )
        .unwrap_err();

        assert!(matches!(
            error,
            UpdateVerificationError::UnknownSigningKey(_)
        ));

        let malformed_manifest = make_manifest(
            "not-a-hash".into(),
            "broken-signature".into(),
            trusted_key().key_id,
        );
        let error = verify_update_bundle(
            &serde_json::to_string(&malformed_manifest).unwrap(),
            &bundle_json,
            &[trusted_key()],
        )
        .unwrap_err();

        assert!(matches!(
            error,
            UpdateVerificationError::InvalidHashFormat(_)
                | UpdateVerificationError::InvalidSignatureFormat(_)
        ));
    }

    #[test]
    fn verify_update_bundle_rejects_unsupported_manifest_schema() {
        let json = r#"{
            "schemaVersion": 9,
            "bundleVersion": "2026.06.01.1",
            "bundleHash": "sha256:abc123abc123abc123abc123abc123abc123abc123abc123abc123abc123abc1",
            "signature": "ed25519:AA==",
            "signingKeyId": "test-key-1",
            "minimumClientVersion": "0.1.0",
            "sources": []
        }"#;

        let error = UpdateManifest::from_json(json).unwrap_err();

        assert_eq!(
            error.to_string(),
            "unsupported update-manifest schema version 9; supported version is 1"
        );
    }
}
