mod backend;

pub use backend::AndroidUpdateBackend;

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use blinc_update::{
        InstallHandoff, ReleaseArtifact, ReleaseChannel, ReleaseManifest, UpdateBackend,
        UpdateCheckRequest, UpdateError,
    };

    use crate::AndroidUpdateBackend;

    #[test]
    fn android_backend_rejects_artifact_with_wrong_target_id() {
        let backend = AndroidUpdateBackend::new("io.test.demo", true, "/tmp/downloads");
        let artifact = ReleaseArtifact {
            platform: "android".to_string(),
            arch: "arm64-v8a".to_string(),
            target_id: "io.test.other".to_string(),
            url: "https://example.com/releases/demo.apk".to_string(),
            size: 42,
            sha256: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
                .to_string(),
            signature:
                "CQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQ=="
                    .to_string(),
        };
        let apk_path = PathBuf::from("/tmp/downloads/demo.apk");

        let err = backend
            .build_install_intent(&artifact, apk_path)
            .expect_err("android backend should reject artifacts for a different package");
        assert!(
            matches!(err, UpdateError::Backend(message) if message.contains("target_id")),
            "target mismatches should return a backend error mentioning target_id"
        );
    }

    #[test]
    fn android_backend_builds_install_intent_for_matching_apk() {
        let backend = AndroidUpdateBackend::new("io.test.demo", true, "/tmp/downloads");
        let artifact = ReleaseArtifact {
            platform: "android".to_string(),
            arch: "arm64-v8a".to_string(),
            target_id: "io.test.demo".to_string(),
            url: "https://example.com/releases/demo.apk".to_string(),
            size: 42,
            sha256: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
                .to_string(),
            signature:
                "CQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQ=="
                    .to_string(),
        };
        let apk_path = PathBuf::from("/tmp/downloads/demo.apk");

        let intent = backend
            .build_install_intent(&artifact, apk_path.clone())
            .expect("android backend should build a package-installer handoff intent");

        assert_eq!(
            intent.downloaded_file, apk_path,
            "install intent should point at the downloaded apk path"
        );
        assert_eq!(
            intent.artifact.target_id, "io.test.demo",
            "install intent should preserve the release artifact metadata"
        );
        assert_eq!(
            intent.handoff,
            InstallHandoff::AndroidPackageInstaller {
                package_name: "io.test.demo".to_string(),
                allow_unknown_sources_prompt: true,
            },
            "android backend should request package installer handoff"
        );
    }

    #[test]
    fn android_backend_selects_newer_matching_release() {
        let backend = AndroidUpdateBackend::new("io.test.demo", true, "/tmp/downloads");
        let artifact = ReleaseArtifact {
            platform: "android".to_string(),
            arch: "arm64-v8a".to_string(),
            target_id: "io.test.demo".to_string(),
            url: "https://example.com/releases/demo.apk".to_string(),
            size: 42,
            sha256: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
                .to_string(),
            signature:
                "CQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQ=="
                    .to_string(),
        };
        let manifest = ReleaseManifest {
            schema_version: 1,
            product: "Demo".to_string(),
            channel: ReleaseChannel::Stable,
            version: "1.2.4".to_string(),
            published_at: "2026-03-07T00:00:00Z".to_string(),
            notes_url: None,
            artifacts: vec![artifact.clone()],
        };
        let request = UpdateCheckRequest {
            platform: "android".to_string(),
            arch: "arm64-v8a".to_string(),
            target_id: "io.test.demo".to_string(),
            current_version: "1.2.3".to_string(),
        };

        let selected = backend
            .check_for_update(&manifest, &request)
            .expect("android backend should compare versions and select matching artifacts")
            .expect("android backend should return the newer matching artifact");

        assert_eq!(
            selected.url, artifact.url,
            "android backend should return the matching android artifact"
        );
    }
}
