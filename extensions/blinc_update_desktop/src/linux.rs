use blinc_update::{InstallIntent, ReleaseArtifact, UpdateError};
use std::path::PathBuf;

pub(crate) fn build_install_intent(
    _artifact: &ReleaseArtifact,
    _downloaded_file: PathBuf,
) -> Result<InstallIntent, UpdateError> {
    Err(UpdateError::Backend(
        "linux desktop updater backend is unsupported in v1".to_string(),
    ))
}
