mod backend;
mod linux;
mod macos;
mod windows;

pub use backend::{DesktopPlatform, DesktopUpdateBackend};

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use blinc_update::{
        InstallHandoff, InstallIntent, ReleaseArtifact, ReleaseChannel, ReleaseManifest,
        UpdateBackend, UpdateCheckRequest, UpdateError,
    };

    use crate::{DesktopPlatform, DesktopUpdateBackend};

    #[test]
    fn macos_backend_emits_bundle_replace_install_intent() {
        let backend = DesktopUpdateBackend::new(
            DesktopPlatform::MacOs {
                bundle_id: "io.test.demo".to_string(),
            },
            "/tmp/downloads",
        );
        let artifact = ReleaseArtifact {
            platform: "macos".to_string(),
            arch: "universal".to_string(),
            target_id: "io.test.demo".to_string(),
            url: "https://example.com/releases/Demo.zip".to_string(),
            size: 42,
            sha256: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
                .to_string(),
            signature:
                "CQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQ=="
                    .to_string(),
        };
        let bundle_path = PathBuf::from("/tmp/downloads/Demo.app");

        let intent = backend
            .build_install_intent(&artifact, bundle_path.clone())
            .expect("macOS backend should build a bundle replacement handoff intent");

        assert_eq!(
            intent.downloaded_file, bundle_path,
            "install intent should point at the downloaded macOS artifact"
        );
        assert_eq!(
            intent.handoff,
            InstallHandoff::MacOsBundleReplace {
                bundle_id: "io.test.demo".to_string(),
            },
            "macOS backend should request bundle replacement handoff"
        );
    }

    #[test]
    fn windows_backend_reports_unsupported_for_now() {
        let backend = DesktopUpdateBackend::new(DesktopPlatform::Windows, "/tmp/downloads");
        let artifact = ReleaseArtifact {
            platform: "windows".to_string(),
            arch: "x86_64".to_string(),
            target_id: "Demo".to_string(),
            url: "https://example.com/releases/Demo.msi".to_string(),
            size: 42,
            sha256: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
                .to_string(),
            signature:
                "CQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQ=="
                    .to_string(),
        };

        let err = backend
            .build_install_intent(&artifact, PathBuf::from("/tmp/downloads/Demo.msi"))
            .expect_err("windows backend should stay unsupported in v1");
        assert!(
            matches!(err, UpdateError::Backend(message) if message.contains("unsupported")),
            "windows backend should report an explicit unsupported error"
        );
    }

    #[test]
    fn linux_backend_reports_unsupported_for_now() {
        let backend = DesktopUpdateBackend::new(DesktopPlatform::Linux, "/tmp/downloads");
        let artifact = ReleaseArtifact {
            platform: "linux".to_string(),
            arch: "x86_64".to_string(),
            target_id: "demo.desktop".to_string(),
            url: "https://example.com/releases/Demo.AppImage".to_string(),
            size: 42,
            sha256: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
                .to_string(),
            signature:
                "CQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQ=="
                    .to_string(),
        };

        let err = backend
            .build_install_intent(&artifact, PathBuf::from("/tmp/downloads/Demo.AppImage"))
            .expect_err("linux backend should stay unsupported in v1");
        assert!(
            matches!(err, UpdateError::Backend(message) if message.contains("unsupported")),
            "linux backend should report an explicit unsupported error"
        );
    }

    #[test]
    fn macos_backend_selects_newer_matching_release() {
        let backend = DesktopUpdateBackend::new(
            DesktopPlatform::MacOs {
                bundle_id: "io.test.demo".to_string(),
            },
            "/tmp/downloads",
        );
        let artifact = ReleaseArtifact {
            platform: "macos".to_string(),
            arch: "universal".to_string(),
            target_id: "io.test.demo".to_string(),
            url: "https://example.com/releases/Demo.zip".to_string(),
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
            platform: "macos".to_string(),
            arch: "universal".to_string(),
            target_id: "io.test.demo".to_string(),
            current_version: "1.2.3".to_string(),
        };

        let selected = backend
            .check_for_update(&manifest, &request)
            .expect("macOS backend should compare versions and select matching artifacts")
            .expect("macOS backend should return the newer matching artifact");

        assert_eq!(
            selected.url, artifact.url,
            "macOS backend should return the matching desktop artifact"
        );
    }

    #[test]
    fn macos_backend_accepts_matching_install_handoff() {
        let backend = DesktopUpdateBackend::new(
            DesktopPlatform::MacOs {
                bundle_id: "io.test.demo".to_string(),
            },
            "/tmp/downloads",
        );
        let intent = InstallIntent {
            artifact: ReleaseArtifact {
                platform: "macos".to_string(),
                arch: "universal".to_string(),
                target_id: "io.test.demo".to_string(),
                url: "https://example.com/releases/Demo.zip".to_string(),
                size: 42,
                sha256:
                    "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
                        .to_string(),
                signature:
                    "CQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQ=="
                        .to_string(),
            },
            downloaded_file: PathBuf::from("/tmp/downloads/Demo.app"),
            handoff: InstallHandoff::MacOsBundleReplace {
                bundle_id: "io.test.demo".to_string(),
            },
        };

        backend
            .install_update(&intent)
            .expect("macOS backend should accept matching bundle-replace handoffs");
    }
}
