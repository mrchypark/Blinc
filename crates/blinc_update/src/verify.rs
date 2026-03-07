use std::path::Path;

use base64::{engine::general_purpose::STANDARD, Engine as _};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use sha2::{Digest, Sha256};

use crate::error::UpdateError;
use crate::manifest::ReleaseArtifact;

pub fn verify_artifact_file(
    path: &Path,
    artifact: &ReleaseArtifact,
    public_key: &str,
) -> Result<(), UpdateError> {
    let bytes = std::fs::read(path).map_err(|source| UpdateError::ArtifactRead {
        path: path.to_path_buf(),
        source,
    })?;

    verify_artifact_bytes(&bytes, artifact, public_key)
}

pub fn verify_artifact_bytes(
    bytes: &[u8],
    artifact: &ReleaseArtifact,
    public_key: &str,
) -> Result<(), UpdateError> {
    verify_sha256(bytes, &artifact.sha256)?;
    verify_signature(bytes, &artifact.signature, public_key)
}

fn verify_sha256(bytes: &[u8], expected: &str) -> Result<(), UpdateError> {
    if expected.len() != 64 || !expected.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return Err(UpdateError::InvalidSha256);
    }

    let actual = Sha256::digest(bytes);
    let mut actual_hex = String::with_capacity(actual.len() * 2);
    for byte in actual {
        actual_hex.push_str(&format!("{byte:02x}"));
    }

    if actual_hex != expected.to_ascii_lowercase() {
        return Err(UpdateError::ChecksumMismatch);
    }

    Ok(())
}

fn verify_signature(bytes: &[u8], signature: &str, public_key: &str) -> Result<(), UpdateError> {
    let signature_bytes = STANDARD
        .decode(signature)
        .map_err(|_| UpdateError::InvalidSignatureEncoding)?;
    let signature = Signature::from_bytes(
        &signature_bytes
            .try_into()
            .map_err(|_| UpdateError::InvalidSignatureLength)?,
    );
    let public_key_bytes = STANDARD
        .decode(public_key)
        .map_err(|_| UpdateError::InvalidPublicKeyEncoding)?;
    let verifying_key = VerifyingKey::from_bytes(
        &public_key_bytes
            .try_into()
            .map_err(|_| UpdateError::InvalidPublicKeyLength)?,
    )
    .map_err(|_| UpdateError::InvalidPublicKeyLength)?;

    verifying_key
        .verify(bytes, &signature)
        .map_err(|_| UpdateError::SignatureMismatch)
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::engine::general_purpose::STANDARD;
    use ed25519_dalek::{Signer, SigningKey};
    use sha2::{Digest, Sha256};
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    const PRIVATE_KEY: &str = "BwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwc=";

    #[test]
    fn accepts_matching_sha256_and_signature() {
        let (bytes, artifact, public_key) = signed_fixture();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time must be after epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("blinc_update_verify_ok_{nonce}.bin"));
        fs::write(&path, &bytes).expect("artifact fixture should be written");

        verify_artifact_file(&path, &artifact, &public_key)
            .expect("matching artifacts should verify successfully");

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn rejects_artifact_when_checksum_differs() {
        let (bytes, mut artifact, public_key) = signed_fixture();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time must be after epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("blinc_update_verify_bad_sha_{nonce}.bin"));
        fs::write(&path, &bytes).expect("artifact fixture should be written");
        artifact.sha256 =
            "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".to_string();

        let err = verify_artifact_file(&path, &artifact, &public_key)
            .expect_err("checksum mismatches must be rejected");
        assert!(
            matches!(err, UpdateError::ChecksumMismatch),
            "checksum mismatches should return a typed checksum error"
        );

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn accepts_uppercase_sha256_digest() {
        let (bytes, mut artifact, public_key) = signed_fixture();
        artifact.sha256 = artifact.sha256.to_uppercase();

        verify_artifact_bytes(&bytes, &artifact, &public_key)
            .expect("uppercase hex digests should verify successfully");
    }

    #[test]
    fn rejects_artifact_when_signature_differs() {
        let (bytes, mut artifact, public_key) = signed_fixture();
        artifact.signature = STANDARD.encode([0_u8; 64]);

        let err = verify_artifact_bytes(&bytes, &artifact, &public_key)
            .expect_err("signature mismatches must be rejected");
        assert!(
            matches!(err, UpdateError::SignatureMismatch),
            "signature mismatches should return a typed signature error"
        );
    }

    #[test]
    fn rejects_artifact_when_sha256_format_is_invalid() {
        let (bytes, mut artifact, public_key) = signed_fixture();
        artifact.sha256 = "not-hex".to_string();

        let err = verify_artifact_bytes(&bytes, &artifact, &public_key)
            .expect_err("invalid sha256 metadata must be rejected");
        assert!(
            matches!(err, UpdateError::InvalidSha256),
            "invalid digests should return a typed sha256 error"
        );
    }

    fn signed_fixture() -> (Vec<u8>, ReleaseArtifact, String) {
        let bytes = b"hello signed release".to_vec();
        let secret_bytes = STANDARD
            .decode(PRIVATE_KEY)
            .expect("fixture private key should decode");
        let signing_key = SigningKey::from_bytes(
            &secret_bytes
                .try_into()
                .expect("fixture private key should be 32 bytes"),
        );
        let signature = signing_key.sign(&bytes);
        let public_key = STANDARD.encode(signing_key.verifying_key().to_bytes());
        let digest = Sha256::digest(&bytes);
        let sha256 = digest
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();

        (
            bytes.clone(),
            ReleaseArtifact {
                platform: "macos".to_string(),
                arch: "universal".to_string(),
                target_id: "io.test.demo".to_string(),
                url: "https://example.com/releases/demo.zip".to_string(),
                size: bytes.len() as u64,
                sha256,
                signature: STANDARD.encode(signature.to_bytes()),
            },
            public_key,
        )
    }
}
