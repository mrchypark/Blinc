use std::path::PathBuf;

use crate::error::UpdateError;
use crate::manifest::{ReleaseArtifact, ReleaseManifest};

#[derive(Debug, Clone)]
pub struct UpdateCheckRequest {
    pub platform: String,
    pub arch: String,
    pub target_id: String,
    pub current_version: String,
}

#[derive(Debug, Clone)]
pub struct InstallIntent {
    pub artifact: ReleaseArtifact,
    pub downloaded_file: PathBuf,
}

#[derive(Debug, Clone)]
pub enum UpdateState {
    Idle,
    Checking,
    Available(ReleaseArtifact),
    Downloading { downloaded: u64, total: Option<u64> },
    ReadyToInstall(InstallIntent),
    Installing,
    Complete,
    Failed(String),
}

pub trait UpdateBackend {
    fn check_for_update(
        &self,
        manifest: &ReleaseManifest,
        request: &UpdateCheckRequest,
    ) -> Result<Option<ReleaseArtifact>, UpdateError>;

    fn download_artifact(&self, artifact: &ReleaseArtifact) -> Result<InstallIntent, UpdateError>;

    fn install_update(&self, intent: &InstallIntent) -> Result<(), UpdateError>;
}
