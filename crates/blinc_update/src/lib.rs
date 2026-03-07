pub mod error;
pub mod manifest;
pub mod service;
pub mod version;

pub use error::{ManifestError, UpdateError, VersionError};
pub use manifest::{ReleaseArtifact, ReleaseChannel, ReleaseManifest};
pub use service::{InstallIntent, UpdateBackend, UpdateCheckRequest, UpdateState};
pub use version::is_newer_release;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selects_matching_artifact_for_platform_and_arch() {
        let manifest = ReleaseManifest {
            schema_version: 1,
            product: "Demo".to_string(),
            channel: ReleaseChannel::Stable,
            version: "1.2.3".to_string(),
            published_at: "2026-03-07T00:00:00Z".to_string(),
            notes_url: None,
            artifacts: vec![
                ReleaseArtifact {
                    platform: "macos".to_string(),
                    arch: "universal".to_string(),
                    target_id: "io.test.demo".to_string(),
                    url: "https://example.com/demo-macos.zip".to_string(),
                    size: 10,
                    sha256: "deadbeef".to_string(),
                    signature: "c2ln".to_string(),
                },
                ReleaseArtifact {
                    platform: "android".to_string(),
                    arch: "arm64-v8a".to_string(),
                    target_id: "io.test.demo".to_string(),
                    url: "https://example.com/demo.apk".to_string(),
                    size: 20,
                    sha256: "beadfeed".to_string(),
                    signature: "YWJj".to_string(),
                },
            ],
        };

        let artifact = manifest
            .select_artifact("android", "arm64-v8a", "io.test.demo")
            .expect("matching artifact should be selected");
        assert_eq!(artifact.url, "https://example.com/demo.apk");
    }

    #[test]
    fn rejects_manifest_with_missing_target_id() {
        let manifest = ReleaseManifest {
            schema_version: 1,
            product: "Demo".to_string(),
            channel: ReleaseChannel::Stable,
            version: "1.2.3".to_string(),
            published_at: "2026-03-07T00:00:00Z".to_string(),
            notes_url: None,
            artifacts: vec![ReleaseArtifact {
                platform: "android".to_string(),
                arch: "arm64-v8a".to_string(),
                target_id: String::new(),
                url: "https://example.com/demo.apk".to_string(),
                size: 20,
                sha256: "beadfeed".to_string(),
                signature: "YWJj".to_string(),
            }],
        };

        let err = manifest
            .validate()
            .expect_err("manifest validation should reject missing target_id");
        assert!(matches!(err, ManifestError::MissingTargetId { .. }));
    }

    #[test]
    fn version_check_detects_newer_release() {
        assert!(
            is_newer_release("1.2.3", "1.3.0").expect("version comparison should succeed"),
            "newer release should be detected"
        );
        assert!(
            !is_newer_release("1.2.3", "1.2.3").expect("version comparison should succeed"),
            "equal versions should not be treated as newer"
        );
        assert!(
            is_newer_release("1.2.3-beta.1", "1.2.3")
                .expect("stable release should outrank its prerelease"),
            "stable release should be considered newer than prerelease"
        );
        assert!(
            !is_newer_release("1.2.3", "1.2.3-beta.1")
                .expect("prerelease should not outrank stable release"),
            "prerelease should not be considered newer than stable release"
        );
    }
}
