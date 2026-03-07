use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::BlincProject;

#[derive(Debug)]
pub(crate) struct ReleaseManifestArgs {
    pub source: PathBuf,
    pub platform: String,
    pub arch: String,
    pub url: String,
    pub size: u64,
    pub sha256: String,
    pub signature: String,
    pub output: PathBuf,
    pub published_at: String,
    pub notes_url: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ReleaseManifestDocument {
    schema_version: u32,
    product: String,
    channel: crate::config::ReleaseChannel,
    version: String,
    published_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    notes_url: Option<String>,
    artifacts: Vec<ReleaseManifestArtifact>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ReleaseManifestArtifact {
    platform: String,
    arch: String,
    target_id: String,
    url: String,
    size: u64,
    sha256: String,
    signature: String,
}

pub(crate) fn load_release_project(path: &Path) -> Result<BlincProject> {
    BlincProject::load_from_dir(path)
}

pub(crate) fn write_release_manifest(args: &ReleaseManifestArgs) -> Result<()> {
    let project = load_release_project(release_project_root(&args.source)?)?;
    let target_id = resolve_target_id(&project, &args.platform)?;
    let artifact = ReleaseManifestArtifact {
        platform: args.platform.clone(),
        arch: args.arch.clone(),
        target_id,
        url: args.url.clone(),
        size: args.size,
        sha256: args.sha256.clone(),
        signature: args.signature.clone(),
    };

    let manifest = if args.output.exists() {
        let existing = fs::read_to_string(&args.output)
            .with_context(|| format!("Failed to read {}", args.output.display()))?;
        let mut manifest: ReleaseManifestDocument =
            serde_json::from_str(&existing).context("Failed to parse existing release manifest")?;

        if manifest.schema_version != 1
            || manifest.product != project.project.name
            || manifest.channel != project.updates.channel
            || manifest.version != project.project.version
            || manifest.published_at != args.published_at
        {
            bail!("Existing release manifest metadata does not match this artifact");
        }

        match (&mut manifest.notes_url, &args.notes_url) {
            (slot @ None, Some(notes_url)) => *slot = Some(notes_url.clone()),
            (Some(existing), Some(notes_url)) if existing != notes_url => {
                bail!("Existing release manifest notes_url does not match this artifact")
            }
            _ => {}
        }

        manifest.artifacts.push(artifact);
        manifest
    } else {
        ReleaseManifestDocument {
            schema_version: 1,
            product: project.project.name.clone(),
            channel: project.updates.channel,
            version: project.project.version.clone(),
            published_at: args.published_at.clone(),
            notes_url: args.notes_url.clone(),
            artifacts: vec![artifact],
        }
    };

    if let Some(parent) = args.output.parent() {
        fs::create_dir_all(parent)?;
    }

    let json =
        serde_json::to_string_pretty(&manifest).context("Failed to serialize release manifest")?;
    fs::write(&args.output, json)
        .with_context(|| format!("Failed to write {}", args.output.display()))?;

    Ok(())
}

fn release_project_root(path: &Path) -> Result<&Path> {
    let start = if path.is_file() {
        path.parent().unwrap_or(path)
    } else {
        path
    };

    start
        .ancestors()
        .find(|candidate| candidate.join(".blincproj").exists())
        .context("No .blincproj found for the provided release source path")
}

fn resolve_target_id(project: &BlincProject, platform: &str) -> Result<String> {
    match platform {
        "android" => project
            .platforms
            .android
            .as_ref()
            .map(|android| android.package.clone())
            .context("Android release manifests require platforms.android.package"),
        "macos" => project
            .platforms
            .macos
            .as_ref()
            .map(|macos| macos.bundle_id.clone())
            .context("macOS release manifests require platforms.macos.bundle_id"),
        unsupported => bail!(
            "Unsupported release manifest platform '{}'. Supported platforms: android, macos",
            unsupported
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
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

    #[test]
    fn release_manifest_command_writes_manifest_json() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time must be after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("blinc_cli_release_manifest_{nonce}"));
        let output = root.join("dist/release-manifest.json");

        fs::create_dir_all(&root).expect("temp project root should be created");
        fs::write(
            root.join(".blincproj"),
            r#"
                [project]
                name = "Demo"
                version = "1.2.3"

                [platforms.macos]
                bundle_id = "io.test.demo"

                [updates]
                enabled = true
                channel = "stable"
                manifest_url = "https://example.com/releases/manifest.json"
                public_key = "abc"
            "#,
        )
        .expect(".blincproj should be written");

        write_release_manifest(&ReleaseManifestArgs {
            source: root.clone(),
            platform: "macos".to_string(),
            arch: "universal".to_string(),
            url: "https://example.com/releases/demo-1.2.3-macos.zip".to_string(),
            size: 12_345,
            sha256: "deadbeef".to_string(),
            signature: "c2ln".to_string(),
            output: output.clone(),
            published_at: "2026-03-07T00:00:00Z".to_string(),
            notes_url: Some("https://example.com/releases/1.2.3".to_string()),
        })
        .expect("release manifest command should write manifest JSON");

        let manifest = fs::read_to_string(&output).expect("manifest file should exist");
        let json: Value = serde_json::from_str(&manifest).expect("manifest should be valid JSON");

        assert_eq!(
            json["schema_version"], 1,
            "manifest should use schema version 1"
        );
        assert_eq!(json["product"], "Demo", "manifest should use project name");
        assert_eq!(
            json["channel"], "stable",
            "manifest should serialize release channel"
        );
        assert_eq!(
            json["version"], "1.2.3",
            "manifest should use project version"
        );
        assert_eq!(
            json["published_at"], "2026-03-07T00:00:00Z",
            "manifest should preserve publish timestamp"
        );
        assert_eq!(
            json["notes_url"], "https://example.com/releases/1.2.3",
            "manifest should preserve release notes URL"
        );
        assert_eq!(
            json["artifacts"][0]["target_id"], "io.test.demo",
            "manifest artifact should use canonical platform target identity"
        );
        assert_eq!(
            json["artifacts"][0]["platform"], "macos",
            "manifest artifact should preserve platform"
        );
        assert_eq!(
            json["artifacts"][0]["arch"], "universal",
            "manifest artifact should preserve architecture"
        );
        assert_eq!(
            json["artifacts"][0]["url"], "https://example.com/releases/demo-1.2.3-macos.zip",
            "manifest artifact should preserve URL"
        );
        assert_eq!(
            json["artifacts"][0]["size"], 12_345,
            "manifest artifact should preserve size"
        );
        assert_eq!(
            json["artifacts"][0]["sha256"], "deadbeef",
            "manifest artifact should preserve sha256"
        );
        assert_eq!(
            json["artifacts"][0]["signature"], "c2ln",
            "manifest artifact should preserve signature"
        );

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn release_manifest_command_appends_artifacts_for_multiple_platforms() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time must be after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("blinc_cli_release_manifest_append_{nonce}"));
        let output = root.join("dist/release-manifest.json");

        fs::create_dir_all(&root).expect("temp project root should be created");
        fs::write(
            root.join(".blincproj"),
            r#"
                [project]
                name = "Demo"
                version = "1.2.3"

                [platforms.android]
                package = "io.test.demo"

                [platforms.macos]
                bundle_id = "io.test.demo"

                [updates]
                enabled = true
                channel = "stable"
                manifest_url = "https://example.com/releases/manifest.json"
                public_key = "abc"
            "#,
        )
        .expect(".blincproj should be written");

        write_release_manifest(&ReleaseManifestArgs {
            source: root.clone(),
            platform: "macos".to_string(),
            arch: "universal".to_string(),
            url: "https://example.com/releases/demo-1.2.3-macos.zip".to_string(),
            size: 12_345,
            sha256: "deadbeef".to_string(),
            signature: "c2ln".to_string(),
            output: output.clone(),
            published_at: "2026-03-07T00:00:00Z".to_string(),
            notes_url: None,
        })
        .expect("first manifest write should succeed");

        write_release_manifest(&ReleaseManifestArgs {
            source: root.clone(),
            platform: "android".to_string(),
            arch: "arm64-v8a".to_string(),
            url: "https://example.com/releases/demo-1.2.3.apk".to_string(),
            size: 67_890,
            sha256: "beadfeed".to_string(),
            signature: "YWJj".to_string(),
            output: output.clone(),
            published_at: "2026-03-07T00:00:00Z".to_string(),
            notes_url: None,
        })
        .expect("second manifest write should append another artifact");

        let manifest = fs::read_to_string(&output).expect("manifest file should exist");
        let json: Value = serde_json::from_str(&manifest).expect("manifest should be valid JSON");

        assert_eq!(
            json["artifacts"].as_array().map(Vec::len),
            Some(2),
            "manifest command should accumulate multiple artifacts in one file"
        );
        assert_eq!(
            json["artifacts"][1]["platform"], "android",
            "second artifact should preserve its platform"
        );

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn release_manifest_command_accepts_source_file_input() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time must be after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("blinc_cli_release_manifest_source_{nonce}"));
        let source_file = root.join("src/main.blinc");
        let output = root.join("dist/release-manifest.json");

        fs::create_dir_all(root.join("src")).expect("temp source directory should be created");
        fs::write(&source_file, "App {}").expect("source file should be written");
        fs::write(
            root.join(".blincproj"),
            r#"
                [project]
                name = "Demo"
                version = "1.2.3"

                [platforms.macos]
                bundle_id = "io.test.demo"

                [updates]
                enabled = true
                channel = "stable"
                manifest_url = "https://example.com/releases/manifest.json"
                public_key = "abc"
            "#,
        )
        .expect(".blincproj should be written");

        write_release_manifest(&ReleaseManifestArgs {
            source: source_file,
            platform: "macos".to_string(),
            arch: "universal".to_string(),
            url: "https://example.com/releases/demo-1.2.3-macos.zip".to_string(),
            size: 12_345,
            sha256: "deadbeef".to_string(),
            signature: "c2ln".to_string(),
            output: output.clone(),
            published_at: "2026-03-07T00:00:00Z".to_string(),
            notes_url: None,
        })
        .expect("source-file inputs should resolve to the project root");

        let manifest = fs::read_to_string(&output).expect("manifest file should exist");
        let json: Value = serde_json::from_str(&manifest).expect("manifest should be valid JSON");
        assert_eq!(
            json["artifacts"][0]["target_id"], "io.test.demo",
            "source-file inputs should still resolve canonical target identity"
        );

        let _ = fs::remove_dir_all(&root);
    }
}
