use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ManifestError {
    #[error("manifest artifact at index {index} is missing target_id")]
    MissingTargetId { index: usize },
    #[error("manifest artifact at index {index} is missing {field}")]
    MissingArtifactMetadata { index: usize, field: &'static str },
}

#[derive(Debug, Error)]
pub enum VersionError {
    #[error("invalid version '{0}'")]
    InvalidVersion(String),
}

#[derive(Debug, Error)]
pub enum UpdateError {
    #[error(transparent)]
    Manifest(#[from] ManifestError),
    #[error(transparent)]
    Version(#[from] VersionError),
    #[error("failed to read artifact '{path}': {source}")]
    ArtifactRead {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("artifact checksum mismatch")]
    ChecksumMismatch,
    #[error("artifact sha256 must be a 64-character hexadecimal digest")]
    InvalidSha256,
    #[error("artifact signature must be valid base64")]
    InvalidSignatureEncoding,
    #[error("artifact signature must decode to 64 bytes")]
    InvalidSignatureLength,
    #[error("public key must be valid base64")]
    InvalidPublicKeyEncoding,
    #[error("public key must decode to 32 bytes")]
    InvalidPublicKeyLength,
    #[error("artifact signature verification failed")]
    SignatureMismatch,
    #[error("{0}")]
    Backend(String),
}
