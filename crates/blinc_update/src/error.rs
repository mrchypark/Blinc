use thiserror::Error;

#[derive(Debug, Error)]
pub enum ManifestError {
    #[error("manifest artifact at index {index} is missing target_id")]
    MissingTargetId { index: usize },
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
    #[error("{0}")]
    Backend(String),
}
