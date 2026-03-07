use std::path::{Path, PathBuf};

use blinc_update::{
    is_newer_release, InstallHandoff, InstallIntent, ReleaseArtifact, ReleaseManifest,
    UpdateBackend, UpdateCheckRequest, UpdateError,
};

#[derive(Debug, Clone)]
pub struct AndroidUpdateBackend {
    package_name: String,
    allow_unknown_sources_prompt: bool,
    download_dir: PathBuf,
}

impl AndroidUpdateBackend {
    pub fn new(
        package_name: impl Into<String>,
        allow_unknown_sources_prompt: bool,
        download_dir: impl AsRef<Path>,
    ) -> Self {
        Self {
            package_name: package_name.into(),
            allow_unknown_sources_prompt,
            download_dir: download_dir.as_ref().to_path_buf(),
        }
    }

    pub fn build_install_intent(
        &self,
        artifact: &ReleaseArtifact,
        downloaded_file: PathBuf,
    ) -> Result<InstallIntent, UpdateError> {
        if artifact.platform != "android" {
            return Err(UpdateError::Backend(
                "android backend only accepts android artifacts".to_string(),
            ));
        }

        if artifact.target_id != self.package_name {
            return Err(UpdateError::Backend(format!(
                "android artifact target_id '{}' does not match expected target_id '{}'",
                artifact.target_id, self.package_name
            )));
        }

        let is_apk = downloaded_file
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("apk"))
            .unwrap_or(false);
        if !is_apk {
            return Err(UpdateError::Backend(
                "android backend requires an .apk downloaded file".to_string(),
            ));
        }

        Ok(InstallIntent {
            artifact: artifact.clone(),
            downloaded_file,
            handoff: InstallHandoff::AndroidPackageInstaller {
                package_name: self.package_name.clone(),
                allow_unknown_sources_prompt: self.allow_unknown_sources_prompt,
            },
        })
    }
}

impl UpdateBackend for AndroidUpdateBackend {
    fn check_for_update(
        &self,
        manifest: &ReleaseManifest,
        request: &UpdateCheckRequest,
    ) -> Result<Option<ReleaseArtifact>, UpdateError> {
        if request.platform != "android" {
            return Err(UpdateError::Backend(
                "android backend only supports android update checks".to_string(),
            ));
        }
        if request.target_id != self.package_name {
            return Err(UpdateError::Backend(format!(
                "android request target_id '{}' does not match backend target_id '{}'",
                request.target_id, self.package_name
            )));
        }

        let Some(artifact) = manifest
            .select_artifact("android", &request.arch, &self.package_name)
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
            .unwrap_or("update.apk");
        let downloaded_file = self.download_dir.join(file_name);

        self.build_install_intent(artifact, downloaded_file)
    }

    fn install_update(&self, intent: &InstallIntent) -> Result<(), UpdateError> {
        match &intent.handoff {
            InstallHandoff::AndroidPackageInstaller { package_name, .. }
                if package_name == &self.package_name
                    && intent.artifact.target_id == self.package_name =>
            {
                Ok(())
            }
            _ => Err(UpdateError::Backend(
                "android install intent does not match the backend package".to_string(),
            )),
        }
    }
}
