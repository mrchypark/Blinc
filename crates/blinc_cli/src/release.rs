use anyhow::Result;
use std::path::Path;

use crate::config::BlincProject;

pub(crate) fn load_release_project(path: &Path) -> Result<BlincProject> {
    BlincProject::load_from_dir(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn release_loader_preserves_updates_and_non_legacy_platforms() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time must be after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("blinc_cli_release_loader_{nonce}"));

        fs::create_dir_all(&root).expect("temp project root should be created");
        fs::write(
            root.join(".blincproj"),
            r#"
                [project]
                name = "Demo"
                version = "0.1.0"

                [platforms.android]
                package = "io.test.demo"

                [platforms.macos]
                bundle_id = "io.test.demo"

                [platforms.windows]
                product_name = "Demo"
                company = "Blinc Labs"

                [platforms.linux]
                desktop_name = "Demo"
                categories = ["Utility", "Development"]

                [platforms.wasm]
                base_url = "/demo/"
                canvas_id = "demo-canvas"
                pwa = true
                gpu_backend = "webgpu"
                dev_port = 9000

                [updates]
                enabled = true
                channel = "stable"
                manifest_url = "https://example.com/manifest.json"
                public_key = "abc"

                [updates.desktop]
                enabled = true
                restart_strategy = "prompt"
            "#,
        )
        .expect(".blincproj should be written");

        let project =
            load_release_project(&root).expect("release loader should keep full project metadata");

        assert!(
            project.updates.enabled,
            "release loader should preserve updates"
        );
        assert_eq!(
            project.updates.manifest_url.as_deref(),
            Some("https://example.com/manifest.json"),
            "release loader should preserve updater manifest URL"
        );
        assert_eq!(
            project
                .platforms
                .macos
                .as_ref()
                .map(|macos| macos.bundle_id.as_str()),
            Some("io.test.demo"),
            "release loader should preserve macOS metadata"
        );
        assert_eq!(
            project
                .platforms
                .windows
                .as_ref()
                .and_then(|windows| windows.company.as_deref()),
            Some("Blinc Labs"),
            "release loader should preserve Windows metadata"
        );
        assert_eq!(
            project
                .platforms
                .linux
                .as_ref()
                .map(|linux| linux.categories.clone()),
            Some(vec!["Utility".to_string(), "Development".to_string()]),
            "release loader should preserve Linux metadata"
        );
        assert_eq!(
            project
                .platforms
                .wasm
                .as_ref()
                .and_then(|wasm| wasm.base_url.as_deref()),
            Some("/demo/"),
            "release loader should preserve WASM metadata"
        );

        let _ = fs::remove_dir_all(&root);
    }
}
