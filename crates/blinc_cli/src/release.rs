use anyhow::{anyhow, bail, Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use ed25519_dalek::{Signer, SigningKey};
use serde::{Deserialize, Serialize};
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
    BlincProject::load_from_dir(release_project_root(path)?)
}

pub(crate) fn write_release_manifest(args: &ReleaseManifestArgs) -> Result<()> {
    validate_release_manifest_args(args)?;

    let project = load_release_project(&args.source)?;
    let target_id = resolve_target_id(&project, &args.platform)?;
    let metadata = resolve_artifact_metadata(args)?;
    let artifact = ReleaseManifestArtifact {
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

        manifest.artifacts.retain(|existing| {
            existing.platform != artifact.platform
                || existing.arch != artifact.arch
                || existing.target_id != artifact.target_id
        });
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

fn validate_release_manifest_args(args: &ReleaseManifestArgs) -> Result<()> {
    OffsetDateTime::parse(&args.published_at, &Rfc3339)
        .context("published_at must be a valid RFC 3339 timestamp")?;

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

        return Ok(());
    }

    if args.size == 0 && args.sha256.is_empty() && args.signature.is_empty() {
        bail!("manual manifest mode requires size, sha256, and signature, or use --artifact-path");
    }

    if args.size == 0 {
        bail!("size must be greater than zero");
    }

    validate_sha256(&args.sha256)?;
    validate_signature(&args.signature)
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
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(
        path,
        STANDARD.encode(signing_key.verifying_key().to_bytes()),
    )
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

#[cfg(test)]
mod tests {
    use super::*;
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
}
