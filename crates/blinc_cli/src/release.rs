use anyhow::{anyhow, bail, Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use blinc_update::{
    ReleaseArtifact, ReleaseChannel as UpdateReleaseChannel, ReleaseManifest,
    RELEASE_MANIFEST_SCHEMA_VERSION,
};
use ed25519_dalek::{Signer, SigningKey};
use sha2::{Digest, Sha256};
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use crate::config::BlincProject;

#[derive(Debug)]
pub(crate) struct ReleaseManifestArgs {
    pub source: PathBuf,
    pub platform: String,
    pub arch: String,
    pub url: String,
    pub artifact_path: Option<PathBuf>,
    pub size: u64,
    pub sha256: String,
    pub signature: String,
    pub private_key: Option<String>,
    pub public_key_output: Option<PathBuf>,
    pub output: PathBuf,
    pub published_at: String,
    pub notes_url: Option<String>,
}

pub(crate) fn load_release_project(path: &Path) -> Result<BlincProject> {
    BlincProject::load_from_dir(release_project_root(path)?)
}

pub(crate) fn write_release_manifest(args: &ReleaseManifestArgs) -> Result<()> {
    validate_release_manifest_args(args)?;
    let published_at = normalize_published_at(&args.published_at)?;

    let project = load_release_project(&args.source)?;
    let target_id = resolve_target_id(&project, &args.platform)?;
    let metadata = resolve_artifact_metadata(args)?;
    let artifact = ReleaseArtifact {
        platform: args.platform.clone(),
        arch: args.arch.clone(),
        target_id,
        url: args.url.clone(),
        size: metadata.size,
        sha256: metadata.sha256,
        signature: metadata.signature,
    };

    let manifest = if args.output.exists() {
        let existing = fs::read_to_string(&args.output)
            .with_context(|| format!("Failed to read {}", args.output.display()))?;
        let mut manifest: ReleaseManifest =
            serde_json::from_str(&existing).context("Failed to parse existing release manifest")?;
        let existing_published_at = normalize_published_at(&manifest.published_at)
            .context("existing release manifest published_at must be a valid RFC 3339 timestamp")?;

        if manifest.schema_version != RELEASE_MANIFEST_SCHEMA_VERSION
            || manifest.product != project.project.name
            || manifest.channel != update_release_channel(project.updates.channel)
            || manifest.version != project.project.version
            || existing_published_at != published_at
        {
            bail!("Existing release manifest metadata does not match this artifact");
        }

        manifest.published_at = published_at.clone();
        match (&mut manifest.notes_url, &args.notes_url) {
            (slot @ None, Some(notes_url)) => *slot = Some(notes_url.clone()),
            (Some(existing), Some(notes_url)) if existing != notes_url => {
                bail!("Existing release manifest notes_url does not match this artifact")
            }
            _ => {}
        }

        manifest.artifacts.retain(|existing| {
            existing.platform != artifact.platform
                || existing.arch != artifact.arch
                || existing.target_id != artifact.target_id
        });
        manifest.artifacts.push(artifact);
        manifest
    } else {
        ReleaseManifest {
            schema_version: RELEASE_MANIFEST_SCHEMA_VERSION,
            product: project.project.name.clone(),
            channel: update_release_channel(project.updates.channel),
            version: project.project.version.clone(),
            published_at,
            notes_url: args.notes_url.clone(),
            artifacts: vec![artifact],
        }
    };

    if let Some(parent) = args.output.parent() {
        fs::create_dir_all(parent)?;
    }

    manifest
        .validate()
        .context("generated release manifest failed validation")?;

    let json =
        serde_json::to_string_pretty(&manifest).context("Failed to serialize release manifest")?;
    fs::write(&args.output, json)
        .with_context(|| format!("Failed to write {}", args.output.display()))?;

    Ok(())
}

fn validate_release_manifest_args(args: &ReleaseManifestArgs) -> Result<()> {
    normalize_published_at(&args.published_at)?;

    if let Some(artifact_path) = &args.artifact_path {
        if args.size != 0 || !args.sha256.is_empty() || !args.signature.is_empty() {
            bail!("artifact_path mode computes size, sha256, and signature automatically");
        }

        if !artifact_path.is_file() {
            bail!("artifact_path must point to a file");
        }

        if args
            .private_key
            .as_deref()
            .map(str::trim)
            .filter(|key| !key.is_empty())
            .is_none()
            && std::env::var_os("BLINC_RELEASE_PRIVATE_KEY").is_none()
        {
            bail!("artifact_path requires a private key via --private-key or BLINC_RELEASE_PRIVATE_KEY");
        }
    } else {
        if args.size == 0 && args.sha256.is_empty() && args.signature.is_empty() {
            bail!(
                "manual manifest mode requires size, sha256, and signature, or use --artifact-path"
            );
        }

        if args
            .private_key
            .as_deref()
            .map(str::trim)
            .filter(|key| !key.is_empty())
            .is_some()
            || args.public_key_output.is_some()
        {
            bail!("--private-key and --public-key-output require --artifact-path");
        }

        if args.size == 0 {
            bail!("--size is required and must be greater than zero in manual mode");
        }

        validate_sha256(&args.sha256)?;
        validate_signature(&args.signature)?;
    }

    Ok(())
}

fn normalize_published_at(published_at: &str) -> Result<String> {
    let parsed = OffsetDateTime::parse(published_at, &Rfc3339)
        .context("published_at must be a valid RFC 3339 timestamp")?;
    parsed
        .format(&Rfc3339)
        .context("published_at must be a valid RFC 3339 timestamp")
}

fn resolve_artifact_metadata(args: &ReleaseManifestArgs) -> Result<ResolvedArtifactMetadata> {
    if let Some(artifact_path) = &args.artifact_path {
        let signing_key = load_signing_key(args)?;
        let bytes = fs::read(artifact_path)
            .with_context(|| format!("Failed to read {}", artifact_path.display()))?;
        if bytes.is_empty() {
            bail!("artifact_path must point to a non-empty file");
        }

        if let Some(public_key_output) = &args.public_key_output {
            write_public_key(public_key_output, &signing_key)?;
        }

        return Ok(ResolvedArtifactMetadata {
            size: bytes.len() as u64,
            sha256: sha256_hex(&bytes),
            signature: STANDARD.encode(signing_key.sign(&bytes).to_bytes()),
        });
    }

    Ok(ResolvedArtifactMetadata {
        size: args.size,
        sha256: args.sha256.clone(),
        signature: args.signature.clone(),
    })
}

fn load_signing_key(args: &ReleaseManifestArgs) -> Result<SigningKey> {
    let private_key = args
        .private_key
        .clone()
        .or_else(|| std::env::var("BLINC_RELEASE_PRIVATE_KEY").ok())
        .map(|key| key.trim().to_string())
        .filter(|key| !key.is_empty())
        .context(
            "artifact_path requires a private key via --private-key or BLINC_RELEASE_PRIVATE_KEY",
        )?;

    let bytes = STANDARD
        .decode(private_key)
        .context("private_key must be a base64-encoded 32-byte ed25519 secret key seed")?;
    let key_bytes: [u8; 32] = bytes
        .try_into()
        .map_err(|_| anyhow!("private_key must decode to exactly 32 bytes"))?;

    Ok(SigningKey::from_bytes(&key_bytes))
}

fn write_public_key(path: &Path, signing_key: &SigningKey) -> Result<()> {
    let encoded_public_key = STANDARD.encode(signing_key.verifying_key().to_bytes());

    if path.exists() {
        let existing = fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        if existing.trim() != encoded_public_key {
            bail!(
                "public_key_output already contains a different public key; reuse the same signing key for every artifact in this release"
            );
        } else {
            return Ok(());
        }
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(path, encoded_public_key)
        .with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(())
}

fn validate_sha256(sha256: &str) -> Result<()> {
    if sha256.len() != 64 || !sha256.chars().all(|ch| ch.is_ascii_hexdigit()) {
        bail!("sha256 must be a 64-character hexadecimal digest");
    }

    Ok(())
}

fn validate_signature(signature: &str) -> Result<()> {
    if signature.is_empty() {
        bail!("signature must be a non-empty base64 string");
    }

    let signature_bytes = STANDARD
        .decode(signature)
        .context("signature must be a valid base64 string")?;
    if signature_bytes.len() != 64 {
        bail!("signature must decode to a 64-byte ed25519 signature");
    }
    Ok(())
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(digest.len() * 2);
    for byte in digest {
        let _ = write!(&mut output, "{byte:02x}");
    }
    output
}

struct ResolvedArtifactMetadata {
    size: u64,
    sha256: String,
    signature: String,
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

fn update_release_channel(channel: crate::config::ReleaseChannel) -> UpdateReleaseChannel {
    match channel {
        crate::config::ReleaseChannel::Stable => UpdateReleaseChannel::Stable,
        crate::config::ReleaseChannel::Beta => UpdateReleaseChannel::Beta,
        crate::config::ReleaseChannel::Canary => UpdateReleaseChannel::Canary,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use blinc_update::{
        verify_artifact_bytes, ReleaseManifest as UpdateReleaseManifest,
        RELEASE_MANIFEST_SCHEMA_VERSION,
    };
    use ed25519_dalek::{Signature, Verifier, VerifyingKey};
    use serde_json::Value;
    use std::time::{SystemTime, UNIX_EPOCH};

    const VALID_SIGNATURE: &str =
        "CQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQ==";

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
            artifact_path: None,
            size: 12_345,
            sha256: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string(),
            signature: VALID_SIGNATURE.to_string(),
            private_key: None,
            public_key_output: None,
            output: output.clone(),
            published_at: "2026-03-07T00:00:00Z".to_string(),
            notes_url: Some("https://example.com/releases/1.2.3".to_string()),
        })
        .expect("release manifest command should write manifest JSON");

        let manifest = fs::read_to_string(&output).expect("manifest file should exist");
        let json: Value = serde_json::from_str(&manifest).expect("manifest should be valid JSON");

        assert_eq!(
            json["schema_version"], RELEASE_MANIFEST_SCHEMA_VERSION,
            "manifest should use the shared updater schema version"
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
            json["artifacts"][0]["sha256"],
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            "manifest artifact should preserve sha256"
        );
        assert_eq!(
            json["artifacts"][0]["signature"], VALID_SIGNATURE,
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
            artifact_path: None,
            size: 12_345,
            sha256: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string(),
            signature: VALID_SIGNATURE.to_string(),
            private_key: None,
            public_key_output: None,
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
            artifact_path: None,
            size: 67_890,
            sha256: "fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210".to_string(),
            signature: VALID_SIGNATURE.to_string(),
            private_key: None,
            public_key_output: None,
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
    fn release_manifest_command_accepts_equivalent_existing_published_at_formats() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time must be after epoch")
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("blinc_cli_release_manifest_timestamp_{nonce}"));
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

        fs::create_dir_all(output.parent().expect("output should have a parent"))
            .expect("manifest parent should be created");
        fs::write(
            &output,
            format!(
                r#"{{
  "schema_version": {RELEASE_MANIFEST_SCHEMA_VERSION},
  "product": "Demo",
  "channel": "stable",
  "version": "1.2.3",
  "published_at": "2026-03-07T00:00:00+00:00",
  "artifacts": [
    {{
      "platform": "macos",
      "arch": "universal",
      "target_id": "io.test.demo",
      "url": "https://example.com/releases/demo-1.2.3-macos.zip",
      "size": 12345,
      "sha256": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
      "signature": "{VALID_SIGNATURE}"
    }}
  ]
}}"#
            ),
        )
        .expect("existing manifest should be written");

        write_release_manifest(&ReleaseManifestArgs {
            source: root.clone(),
            platform: "android".to_string(),
            arch: "arm64-v8a".to_string(),
            url: "https://example.com/releases/demo-1.2.3.apk".to_string(),
            artifact_path: None,
            size: 54_321,
            sha256: "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789".to_string(),
            signature: VALID_SIGNATURE.to_string(),
            private_key: None,
            public_key_output: None,
            output: output.clone(),
            published_at: "2026-03-07T00:00:00Z".to_string(),
            notes_url: None,
        })
        .expect("equivalent RFC 3339 timestamps should append successfully");

        let manifest = fs::read_to_string(&output).expect("manifest file should exist");
        let json: Value = serde_json::from_str(&manifest).expect("manifest should be valid JSON");
        assert_eq!(
            json["published_at"], "2026-03-07T00:00:00Z",
            "append flow should rewrite equivalent timestamps into the canonical manifest form"
        );
        assert_eq!(
            json["artifacts"].as_array().map(Vec::len),
            Some(2),
            "append flow should preserve the existing artifact and add the new one"
        );

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn release_manifest_command_replaces_existing_artifact_for_same_target() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time must be after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("blinc_cli_release_manifest_replace_{nonce}"));
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
            url: "https://example.com/releases/demo-1.2.3-macos-v1.zip".to_string(),
            artifact_path: None,
            size: 12_345,
            sha256: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string(),
            signature: VALID_SIGNATURE.to_string(),
            private_key: None,
            public_key_output: None,
            output: output.clone(),
            published_at: "2026-03-07T00:00:00Z".to_string(),
            notes_url: None,
        })
        .expect("first manifest write should succeed");

        write_release_manifest(&ReleaseManifestArgs {
            source: root.clone(),
            platform: "macos".to_string(),
            arch: "universal".to_string(),
            url: "https://example.com/releases/demo-1.2.3-macos-v2.zip".to_string(),
            artifact_path: None,
            size: 54_321,
            sha256: "fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210".to_string(),
            signature: VALID_SIGNATURE.to_string(),
            private_key: None,
            public_key_output: None,
            output: output.clone(),
            published_at: "2026-03-07T00:00:00Z".to_string(),
            notes_url: None,
        })
        .expect("second manifest write should replace the prior artifact");

        let manifest = fs::read_to_string(&output).expect("manifest file should exist");
        let json: Value = serde_json::from_str(&manifest).expect("manifest should be valid JSON");

        assert_eq!(
            json["artifacts"].as_array().map(Vec::len),
            Some(1),
            "rewriting the same artifact target should replace the prior entry"
        );
        assert_eq!(
            json["artifacts"][0]["url"], "https://example.com/releases/demo-1.2.3-macos-v2.zip",
            "replacement artifact should keep the latest URL"
        );
        assert_eq!(
            json["artifacts"][0]["size"], 54_321,
            "replacement artifact should keep the latest size"
        );

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn release_manifest_populates_artifact_signatures() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time must be after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("blinc_cli_release_manifest_signing_{nonce}"));
        let artifact_path = root.join("dist/demo.zip");
        let manifest_path = root.join("dist/release-manifest.json");
        let public_key_path = root.join("dist/public-key.txt");

        fs::create_dir_all(
            artifact_path
                .parent()
                .expect("artifact directory should exist"),
        )
        .expect("artifact directory should be created");
        fs::write(&artifact_path, b"hello signed release").expect("artifact should be written");
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
            artifact_path: Some(artifact_path),
            size: 0,
            sha256: String::new(),
            signature: String::new(),
            private_key: Some("BwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwc=".to_string()),
            public_key_output: Some(public_key_path.clone()),
            output: manifest_path.clone(),
            published_at: "2026-03-07T00:00:00Z".to_string(),
            notes_url: None,
        })
        .expect("artifact-path mode should compute and sign artifact metadata");

        let manifest = fs::read_to_string(&manifest_path).expect("manifest file should exist");
        let json: Value = serde_json::from_str(&manifest).expect("manifest should be valid JSON");
        let signature = json["artifacts"][0]["signature"]
            .as_str()
            .expect("signature should be serialized as a string");
        let public_key =
            fs::read_to_string(&public_key_path).expect("public key output should be written");

        assert_eq!(
            json["artifacts"][0]["size"], 20,
            "artifact-path mode should derive artifact size from the file"
        );
        assert_eq!(
            json["artifacts"][0]["sha256"],
            "1e7baffe75a68c049cc5f0bc7f0a894d4a3e8942adbc148e4a80c98a9d70866e",
            "artifact-path mode should derive sha256 from the file"
        );
        assert!(
            !signature.is_empty(),
            "artifact-path mode should generate a detached signature"
        );
        assert!(
            !public_key.trim().is_empty(),
            "artifact-path mode should expose the matching public key"
        );
        let signature_bytes = STANDARD
            .decode(signature)
            .expect("generated signatures should be valid base64");
        let signature = Signature::from_bytes(
            &signature_bytes
                .try_into()
                .expect("ed25519 signatures should decode to 64 bytes"),
        );
        let public_key_bytes = STANDARD
            .decode(public_key.trim())
            .expect("generated public keys should be valid base64");
        let verifying_key = VerifyingKey::from_bytes(
            &public_key_bytes
                .try_into()
                .expect("ed25519 public keys should decode to 32 bytes"),
        )
        .expect("public key bytes should decode into a verifying key");
        verifying_key
            .verify(b"hello signed release", &signature)
            .expect("generated signature should verify against the generated public key");

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn release_manifest_generation_requires_private_key_input() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time must be after epoch")
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("blinc_cli_release_manifest_missing_key_{nonce}"));
        let artifact_path = root.join("dist/demo.zip");

        fs::create_dir_all(
            artifact_path
                .parent()
                .expect("artifact directory should exist"),
        )
        .expect("artifact directory should be created");
        fs::write(&artifact_path, b"hello signed release").expect("artifact should be written");
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

        let err = write_release_manifest(&ReleaseManifestArgs {
            source: root.clone(),
            platform: "macos".to_string(),
            arch: "universal".to_string(),
            url: "https://example.com/releases/demo-1.2.3-macos.zip".to_string(),
            artifact_path: Some(artifact_path),
            size: 0,
            sha256: String::new(),
            signature: String::new(),
            private_key: None,
            public_key_output: None,
            output: root.join("dist/release-manifest.json"),
            published_at: "2026-03-07T00:00:00Z".to_string(),
            notes_url: None,
        })
        .expect_err("artifact-path mode should require a private key");

        assert!(
            err.to_string().contains("private key"),
            "missing signing key errors should mention the private key requirement"
        );

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn release_manifest_generation_rejects_manual_metadata_with_artifact_path() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time must be after epoch")
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("blinc_cli_release_manifest_mixed_metadata_{nonce}"));
        let artifact_path = root.join("dist/demo.zip");

        fs::create_dir_all(
            artifact_path
                .parent()
                .expect("artifact directory should exist"),
        )
        .expect("artifact directory should be created");
        fs::write(&artifact_path, b"hello signed release").expect("artifact should be written");
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

        let err = write_release_manifest(&ReleaseManifestArgs {
            source: root.clone(),
            platform: "macos".to_string(),
            arch: "universal".to_string(),
            url: "https://example.com/releases/demo-1.2.3-macos.zip".to_string(),
            artifact_path: Some(artifact_path),
            size: 12_345,
            sha256: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string(),
            signature: VALID_SIGNATURE.to_string(),
            private_key: Some("BwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwc=".to_string()),
            public_key_output: None,
            output: root.join("dist/release-manifest.json"),
            published_at: "2026-03-07T00:00:00Z".to_string(),
            notes_url: None,
        })
        .expect_err("artifact-path mode should reject conflicting manual metadata");

        assert!(
            err.to_string().contains("artifact_path"),
            "mixed-mode validation should mention artifact_path"
        );

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn release_manifest_generation_rejects_signing_flags_in_manual_mode() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time must be after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "blinc_cli_release_manifest_manual_signing_flags_{nonce}"
        ));

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

        let err = write_release_manifest(&ReleaseManifestArgs {
            source: root.clone(),
            platform: "macos".to_string(),
            arch: "universal".to_string(),
            url: "https://example.com/releases/demo-1.2.3-macos.zip".to_string(),
            artifact_path: None,
            size: 12_345,
            sha256: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string(),
            signature: VALID_SIGNATURE.to_string(),
            private_key: Some(STANDARD.encode([7u8; 32])),
            public_key_output: Some(root.join("dist/public-key.txt")),
            output: root.join("dist/release-manifest.json"),
            published_at: "2026-03-07T00:00:00Z".to_string(),
            notes_url: None,
        })
        .expect_err("manual mode should reject signing-only flags that it does not use");

        assert!(
            err.to_string().contains("--artifact-path"),
            "manual mode signing flag errors should explain that --artifact-path is required"
        );

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn release_manifest_generation_rejects_directory_artifact_path() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time must be after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "blinc_cli_release_manifest_directory_artifact_{nonce}"
        ));
        let artifact_path = root.join("dist");

        fs::create_dir_all(&artifact_path).expect("artifact directory should be created");
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

        let err = write_release_manifest(&ReleaseManifestArgs {
            source: root.clone(),
            platform: "macos".to_string(),
            arch: "universal".to_string(),
            url: "https://example.com/releases/demo-1.2.3-macos.zip".to_string(),
            artifact_path: Some(artifact_path),
            size: 0,
            sha256: String::new(),
            signature: String::new(),
            private_key: Some("BwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwc=".to_string()),
            public_key_output: None,
            output: root.join("dist/release-manifest.json"),
            published_at: "2026-03-07T00:00:00Z".to_string(),
            notes_url: None,
        })
        .expect_err("artifact-path mode should reject directory paths");

        assert!(
            err.to_string().contains("artifact_path"),
            "artifact path validation should mention artifact_path"
        );

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn release_manifest_command_rejects_invalid_published_at() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time must be after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "blinc_cli_release_manifest_invalid_timestamp_{nonce}"
        ));

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

        let err = write_release_manifest(&ReleaseManifestArgs {
            source: root.clone(),
            platform: "macos".to_string(),
            arch: "universal".to_string(),
            url: "https://example.com/releases/demo-1.2.3-macos.zip".to_string(),
            artifact_path: None,
            size: 12_345,
            sha256: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string(),
            signature: VALID_SIGNATURE.to_string(),
            private_key: None,
            public_key_output: None,
            output: root.join("dist/release-manifest.json"),
            published_at: "not-a-timestamp".to_string(),
            notes_url: None,
        })
        .expect_err("invalid RFC 3339 timestamps should be rejected");

        assert!(
            err.to_string().contains("published_at"),
            "timestamp validation error should mention published_at"
        );

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn release_manifest_command_rejects_invalid_sha256() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time must be after epoch")
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("blinc_cli_release_manifest_invalid_sha_{nonce}"));

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

        let err = write_release_manifest(&ReleaseManifestArgs {
            source: root.clone(),
            platform: "macos".to_string(),
            arch: "universal".to_string(),
            url: "https://example.com/releases/demo-1.2.3-macos.zip".to_string(),
            artifact_path: None,
            size: 12_345,
            sha256: "not-hex".to_string(),
            signature: VALID_SIGNATURE.to_string(),
            private_key: None,
            public_key_output: None,
            output: root.join("dist/release-manifest.json"),
            published_at: "2026-03-07T00:00:00Z".to_string(),
            notes_url: None,
        })
        .expect_err("invalid sha256 values should be rejected");

        assert!(
            err.to_string().contains("sha256"),
            "sha256 validation error should mention sha256"
        );

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn release_manifest_command_rejects_invalid_signature() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time must be after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "blinc_cli_release_manifest_invalid_signature_{nonce}"
        ));

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

        let err = write_release_manifest(&ReleaseManifestArgs {
            source: root.clone(),
            platform: "macos".to_string(),
            arch: "universal".to_string(),
            url: "https://example.com/releases/demo-1.2.3-macos.zip".to_string(),
            artifact_path: None,
            size: 12_345,
            sha256: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string(),
            signature: "not-base64!!!".to_string(),
            private_key: None,
            public_key_output: None,
            output: root.join("dist/release-manifest.json"),
            published_at: "2026-03-07T00:00:00Z".to_string(),
            notes_url: None,
        })
        .expect_err("invalid signatures should be rejected");

        assert!(
            err.to_string().contains("signature"),
            "signature validation error should mention signature"
        );

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn release_manifest_command_rejects_short_signature() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time must be after epoch")
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("blinc_cli_release_manifest_short_sig_{nonce}"));

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

        let err = write_release_manifest(&ReleaseManifestArgs {
            source: root.clone(),
            platform: "macos".to_string(),
            arch: "universal".to_string(),
            url: "https://example.com/releases/demo-1.2.3-macos.zip".to_string(),
            artifact_path: None,
            size: 12_345,
            sha256: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string(),
            signature: "c2ln".to_string(),
            private_key: None,
            public_key_output: None,
            output: root.join("dist/release-manifest.json"),
            published_at: "2026-03-07T00:00:00Z".to_string(),
            notes_url: None,
        })
        .expect_err("manual mode should reject base64 signatures with the wrong length");

        assert!(
            err.to_string().contains("signature"),
            "signature length errors should mention signature"
        );

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn release_manifest_command_rejects_empty_sha256() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time must be after epoch")
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("blinc_cli_release_manifest_empty_sha_{nonce}"));

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

        let err = write_release_manifest(&ReleaseManifestArgs {
            source: root.clone(),
            platform: "macos".to_string(),
            arch: "universal".to_string(),
            url: "https://example.com/releases/demo-1.2.3-macos.zip".to_string(),
            artifact_path: None,
            size: 12_345,
            sha256: String::new(),
            signature: "c2ln".to_string(),
            private_key: None,
            public_key_output: None,
            output: root.join("dist/release-manifest.json"),
            published_at: "2026-03-07T00:00:00Z".to_string(),
            notes_url: None,
        })
        .expect_err("empty sha256 values should be rejected");

        assert!(
            err.to_string().contains("sha256"),
            "sha256 validation error should mention sha256"
        );

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn release_manifest_command_rejects_empty_signature() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time must be after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "blinc_cli_release_manifest_empty_signature_{nonce}"
        ));

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

        let err = write_release_manifest(&ReleaseManifestArgs {
            source: root.clone(),
            platform: "macos".to_string(),
            arch: "universal".to_string(),
            url: "https://example.com/releases/demo-1.2.3-macos.zip".to_string(),
            artifact_path: None,
            size: 12_345,
            sha256: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string(),
            signature: String::new(),
            private_key: None,
            public_key_output: None,
            output: root.join("dist/release-manifest.json"),
            published_at: "2026-03-07T00:00:00Z".to_string(),
            notes_url: None,
        })
        .expect_err("empty signature values should be rejected");

        assert!(
            err.to_string().contains("signature"),
            "signature validation error should mention signature"
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
            artifact_path: None,
            size: 12_345,
            sha256: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string(),
            signature: VALID_SIGNATURE.to_string(),
            private_key: None,
            public_key_output: None,
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

    #[test]
    fn generated_manifest_round_trips_into_update_domain() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time must be after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("blinc_cli_release_roundtrip_{nonce}"));
        let dist = root.join("dist");
        let manifest_path = dist.join("release-manifest.json");
        let public_key_path = dist.join("public-key.txt");
        let macos_artifact_path = dist.join("Demo.zip");
        let android_artifact_path = dist.join("Demo.apk");
        let macos_bytes = b"macos signed release".to_vec();
        let android_bytes = b"android signed release".to_vec();

        fs::create_dir_all(&dist).expect("dist directory should be created");
        fs::write(&macos_artifact_path, &macos_bytes).expect("macOS artifact should be written");
        fs::write(&android_artifact_path, &android_bytes)
            .expect("Android artifact should be written");
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
                manifest_url = "https://example.com/releases/release-manifest.json"
                public_key = "abc"
            "#,
        )
        .expect(".blincproj should be written");

        write_release_manifest(&ReleaseManifestArgs {
            source: root.clone(),
            platform: "macos".to_string(),
            arch: "universal".to_string(),
            url: "https://example.com/releases/Demo.zip".to_string(),
            artifact_path: Some(macos_artifact_path.clone()),
            size: 0,
            sha256: String::new(),
            signature: String::new(),
            private_key: Some("BwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwc=".to_string()),
            public_key_output: Some(public_key_path.clone()),
            output: manifest_path.clone(),
            published_at: "2026-03-07T00:00:00Z".to_string(),
            notes_url: None,
        })
        .expect("macOS artifact should be added to the manifest");

        write_release_manifest(&ReleaseManifestArgs {
            source: root.clone(),
            platform: "android".to_string(),
            arch: "arm64-v8a".to_string(),
            url: "https://example.com/releases/Demo.apk".to_string(),
            artifact_path: Some(android_artifact_path.clone()),
            size: 0,
            sha256: String::new(),
            signature: String::new(),
            private_key: Some("BwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwc=".to_string()),
            public_key_output: Some(public_key_path.clone()),
            output: manifest_path.clone(),
            published_at: "2026-03-07T00:00:00Z".to_string(),
            notes_url: None,
        })
        .expect("Android artifact should be added to the manifest");

        let manifest_json =
            fs::read_to_string(&manifest_path).expect("release manifest should be written");
        let manifest: UpdateReleaseManifest =
            serde_json::from_str(&manifest_json).expect("manifest should parse into blinc_update");
        manifest
            .validate()
            .expect("shared update domain should validate the generated manifest");

        let public_key =
            fs::read_to_string(&public_key_path).expect("public key output should be written");
        let macos_artifact = manifest
            .select_artifact("macos", "universal", "io.test.demo")
            .expect("shared domain should select the macOS artifact");
        let android_artifact = manifest
            .select_artifact("android", "arm64-v8a", "io.test.demo")
            .expect("shared domain should select the Android artifact");

        verify_artifact_bytes(&macos_bytes, macos_artifact, public_key.trim())
            .expect("shared verifier should accept the generated macOS artifact");
        verify_artifact_bytes(&android_bytes, android_artifact, public_key.trim())
            .expect("shared verifier should accept the generated Android artifact");

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn release_manifest_generation_rejects_public_key_output_key_rotation() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time must be after epoch")
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("blinc_cli_release_manifest_key_rotation_{nonce}"));
        let dist = root.join("dist");
        let manifest_path = dist.join("release-manifest.json");
        let public_key_path = dist.join("public-key.txt");
        let macos_artifact_path = dist.join("Demo.zip");
        let android_artifact_path = dist.join("Demo.apk");

        fs::create_dir_all(&dist).expect("dist directory should be created");
        fs::write(&macos_artifact_path, b"macos signed release")
            .expect("macOS artifact should be written");
        fs::write(&android_artifact_path, b"android signed release")
            .expect("Android artifact should be written");
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
                manifest_url = "https://example.com/releases/release-manifest.json"
                public_key = "abc"
            "#,
        )
        .expect(".blincproj should be written");

        write_release_manifest(&ReleaseManifestArgs {
            source: root.clone(),
            platform: "macos".to_string(),
            arch: "universal".to_string(),
            url: "https://example.com/releases/Demo.zip".to_string(),
            artifact_path: Some(macos_artifact_path),
            size: 0,
            sha256: String::new(),
            signature: String::new(),
            private_key: Some(STANDARD.encode([7u8; 32])),
            public_key_output: Some(public_key_path.clone()),
            output: manifest_path.clone(),
            published_at: "2026-03-07T00:00:00Z".to_string(),
            notes_url: None,
        })
        .expect("first artifact should write the baseline public key");

        let err = write_release_manifest(&ReleaseManifestArgs {
            source: root.clone(),
            platform: "android".to_string(),
            arch: "arm64-v8a".to_string(),
            url: "https://example.com/releases/Demo.apk".to_string(),
            artifact_path: Some(android_artifact_path),
            size: 0,
            sha256: String::new(),
            signature: String::new(),
            private_key: Some(STANDARD.encode([8u8; 32])),
            public_key_output: Some(public_key_path.clone()),
            output: manifest_path,
            published_at: "2026-03-07T00:00:00Z".to_string(),
            notes_url: None,
        })
        .expect_err("appending artifacts should reject rotating the emitted public key");

        assert!(
            err.to_string().contains("public_key_output"),
            "key rotation errors should point at the emitted public key contract"
        );

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn release_manifest_generation_rejects_blank_target_id_before_writing() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time must be after epoch")
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("blinc_cli_release_manifest_blank_target_{nonce}"));
        let output = root.join("dist/release-manifest.json");

        fs::create_dir_all(&root).expect("temp project root should be created");
        fs::write(
            root.join(".blincproj"),
            r#"
                [project]
                name = "Demo"
                version = "1.2.3"

                [platforms.macos]
                bundle_id = ""

                [updates]
                enabled = true
                channel = "stable"
                manifest_url = "https://example.com/releases/manifest.json"
                public_key = "abc"
            "#,
        )
        .expect(".blincproj should be written");

        let err = write_release_manifest(&ReleaseManifestArgs {
            source: root.clone(),
            platform: "macos".to_string(),
            arch: "universal".to_string(),
            url: "https://example.com/releases/demo-1.2.3-macos.zip".to_string(),
            artifact_path: None,
            size: 12_345,
            sha256: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string(),
            signature: VALID_SIGNATURE.to_string(),
            private_key: None,
            public_key_output: None,
            output: output.clone(),
            published_at: "2026-03-07T00:00:00Z".to_string(),
            notes_url: None,
        })
        .expect_err("manifest generation should reject blank target identities before writing");

        assert!(
            err.to_string()
                .contains("generated release manifest failed validation"),
            "validation errors should surface manifest validation before writing"
        );
        assert!(
            !output.exists(),
            "invalid manifest metadata should not be written to disk"
        );

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn release_manifest_generation_requires_size_in_manual_mode() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time must be after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("blinc_cli_release_manifest_size_{nonce}"));

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

        let err = write_release_manifest(&ReleaseManifestArgs {
            source: root.clone(),
            platform: "macos".to_string(),
            arch: "universal".to_string(),
            url: "https://example.com/releases/demo-1.2.3-macos.zip".to_string(),
            artifact_path: None,
            size: 0,
            sha256: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string(),
            signature: VALID_SIGNATURE.to_string(),
            private_key: None,
            public_key_output: None,
            output: root.join("dist/release-manifest.json"),
            published_at: "2026-03-07T00:00:00Z".to_string(),
            notes_url: None,
        })
        .expect_err("manual mode should require an explicit non-zero size");

        assert!(
            err.to_string()
                .contains("--size is required and must be greater than zero in manual mode"),
            "manual mode should direct the caller to the missing --size argument"
        );

        let _ = fs::remove_dir_all(&root);
    }
}
