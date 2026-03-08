use blinc_update::{InstallHandoff, InstallIntent, ReleaseArtifact, UpdateError};
use std::path::PathBuf;

pub(crate) fn build_install_intent(
    bundle_id: &str,
    artifact: &ReleaseArtifact,
    downloaded_file: PathBuf,
) -> Result<InstallIntent, UpdateError> {
    if artifact.platform != "macos" {
        return Err(UpdateError::Backend(
            "macOS backend only accepts macOS artifacts".to_string(),
        ));
    }
    if artifact.target_id != bundle_id {
        return Err(UpdateError::Backend(format!(
            "macOS artifact target_id '{}' does not match expected target_id '{}'",
            artifact.target_id, bundle_id
        )));
    }

    Ok(InstallIntent {
        artifact: artifact.clone(),
        downloaded_file,
        handoff: InstallHandoff::MacOsBundleReplace {
            bundle_id: bundle_id.to_string(),
        },
    })
}
