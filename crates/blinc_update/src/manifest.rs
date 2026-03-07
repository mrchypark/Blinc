use serde::{Deserialize, Serialize};

use crate::error::ManifestError;

#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseChannel {
    Stable,
    Beta,
    Canary,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ReleaseManifest {
    pub schema_version: u32,
    pub product: String,
    pub channel: ReleaseChannel,
    pub version: String,
    pub published_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes_url: Option<String>,
    pub artifacts: Vec<ReleaseArtifact>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ReleaseArtifact {
    pub platform: String,
    pub arch: String,
    pub target_id: String,
    pub url: String,
    pub size: u64,
    pub sha256: String,
    pub signature: String,
}

impl ReleaseManifest {
    pub fn validate(&self) -> Result<(), ManifestError> {
        for (index, artifact) in self.artifacts.iter().enumerate() {
            if artifact.target_id.trim().is_empty() {
                return Err(ManifestError::MissingTargetId { index });
            }
            if artifact.sha256.trim().is_empty() {
                return Err(ManifestError::MissingArtifactMetadata {
                    index,
                    field: "sha256",
                });
            }
            if artifact.signature.trim().is_empty() {
                return Err(ManifestError::MissingArtifactMetadata {
                    index,
                    field: "signature",
                });
            }
        }

        Ok(())
    }

    pub fn select_artifact(
        &self,
        platform: &str,
        arch: &str,
        target_id: &str,
    ) -> Option<&ReleaseArtifact> {
        self.artifacts.iter().find(|artifact| {
            artifact.platform == platform
                && artifact.arch == arch
                && artifact.target_id == target_id
        })
    }
}
