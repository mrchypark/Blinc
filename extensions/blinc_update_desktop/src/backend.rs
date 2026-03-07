use std::path::{Path, PathBuf};

use blinc_update::{
    is_newer_release, InstallHandoff, InstallIntent, ReleaseArtifact, ReleaseManifest,
    UpdateBackend, UpdateCheckRequest, UpdateError,
};

use crate::{linux, macos, windows};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum DesktopPlatform {
    MacOs { bundle_id: String },
    Windows,
    Linux,
}

#[derive(Debug, Clone)]
pub struct DesktopUpdateBackend {
    platform: DesktopPlatform,
    download_dir: PathBuf,
}

impl DesktopUpdateBackend {
    pub fn new(platform: DesktopPlatform, download_dir: impl AsRef<Path>) -> Self {
        Self {
            platform,
            download_dir: download_dir.as_ref().to_path_buf(),
        }
    }

    pub fn build_install_intent(
        &self,
        artifact: &ReleaseArtifact,
        downloaded_file: PathBuf,
    ) -> Result<InstallIntent, UpdateError> {
        match &self.platform {
            DesktopPlatform::MacOs { bundle_id } => {
                macos::build_install_intent(bundle_id, artifact, downloaded_file)
            }
            DesktopPlatform::Windows => windows::build_install_intent(artifact, downloaded_file),
            DesktopPlatform::Linux => linux::build_install_intent(artifact, downloaded_file),
        }
    }
}

impl UpdateBackend for DesktopUpdateBackend {
    fn check_for_update(
        &self,
        manifest: &ReleaseManifest,
        request: &UpdateCheckRequest,
    ) -> Result<Option<ReleaseArtifact>, UpdateError> {
        let (platform_name, expected_target_id) = match &self.platform {
            DesktopPlatform::MacOs { bundle_id } => ("macos", bundle_id.as_str()),
            DesktopPlatform::Windows => {
                return Err(UpdateError::Backend(
                    "windows desktop updater backend is unsupported in v1".to_string(),
                ))
            }
            DesktopPlatform::Linux => {
                return Err(UpdateError::Backend(
                    "linux desktop updater backend is unsupported in v1".to_string(),
                ))
            }
        };

        if request.platform != platform_name {
            return Err(UpdateError::Backend(format!(
                "desktop backend for '{}' cannot handle platform '{}'",
                platform_name, request.platform
            )));
        }
        if request.target_id != expected_target_id {
            return Err(UpdateError::Backend(format!(
                "desktop request target_id '{}' does not match backend target_id '{}'",
                request.target_id, expected_target_id
            )));
        }

        let Some(artifact) = manifest
            .select_artifact(platform_name, &request.arch, expected_target_id)
            .cloned()
        else {
            return Ok(None);
        };

        let is_newer = is_newer_release(&request.current_version, &manifest.version)?;
        if !is_newer {
            return Ok(None);
        }

        Ok(Some(artifact))
    }

    fn download_artifact(&self, artifact: &ReleaseArtifact) -> Result<InstallIntent, UpdateError> {
        let file_name = artifact
            .url
            .rsplit('/')
            .next()
            .filter(|name| !name.is_empty())
            .unwrap_or("update.bin");
        let downloaded_file = self.download_dir.join(file_name);

        self.build_install_intent(artifact, downloaded_file)
    }

    fn install_update(&self, intent: &InstallIntent) -> Result<(), UpdateError> {
        match (&self.platform, &intent.handoff) {
            (
                DesktopPlatform::MacOs { bundle_id },
                InstallHandoff::MacOsBundleReplace {
                    bundle_id: intent_bundle_id,
                },
            ) if bundle_id == intent_bundle_id && intent.artifact.target_id == *bundle_id => Ok(()),
            (DesktopPlatform::Windows, _) => Err(UpdateError::Backend(
                "windows desktop updater backend is unsupported in v1".to_string(),
            )),
            (DesktopPlatform::Linux, _) => Err(UpdateError::Backend(
                "linux desktop updater backend is unsupported in v1".to_string(),
            )),
            _ => Err(UpdateError::Backend(
                "desktop install intent does not match the backend platform".to_string(),
            )),
        }
    }
}
