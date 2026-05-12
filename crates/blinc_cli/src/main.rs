//! Blinc CLI
//!
//! Build, run, and hot-reload Blinc applications.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{info, warn};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

mod automation;
mod config;
mod doctor;
mod project;
mod release;

use automation::{cmd_automation, AutomationCommands};
use config::BlincConfig;

#[derive(Parser)]
#[command(name = "blinc")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Blinc UI Framework CLI", long_about = None)]
struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build a Blinc application
    Build {
        /// Source file or directory
        #[arg(default_value = ".")]
        source: String,

        /// Target platform (desktop, android, ios, macos, windows, linux)
        #[arg(short, long, default_value = "desktop")]
        target: String,

        /// Build in release mode
        #[arg(short, long)]
        release: bool,

        /// Output path
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Run a Blinc application with hot-reload (development mode)
    Dev {
        /// Source file or directory
        #[arg(default_value = ".")]
        source: String,

        /// Target platform
        #[arg(short, long, default_value = "desktop")]
        target: String,

        /// Port for hot-reload server
        #[arg(short, long, default_value = "3000")]
        port: u16,

        /// How to compile changes — `rust` (default, drives subsecond
        /// hot-patches) or `dsl` (Zyntax DSL, in plan).
        #[arg(short = 'm', long, value_enum, default_value_t = DevMode::Rust)]
        mode: DevMode,

        /// Device to run on (for mobile targets)
        #[arg(long)]
        device: Option<String>,
    },

    /// Run a compiled Blinc application
    Run {
        /// Compiled binary or source file
        #[arg(default_value = ".")]
        source: String,
    },

    /// Build a ZRTL plugin
    Plugin {
        #[command(subcommand)]
        command: PluginCommands,
    },

    /// Generate release metadata
    Release {
        #[command(subcommand)]
        command: ReleaseCommands,
    },

    /// Create a new Blinc project
    New {
        /// Project name
        name: String,

        /// Template to use (default, minimal, counter)
        #[arg(short, long, default_value = "default")]
        template: String,

        /// Organization/package prefix (e.g., "com.mycompany" results in "com.mycompany.appname")
        #[arg(short, long, default_value = "com.example")]
        org: String,

        /// Create a Rust-first project (native code instead of .blinc DSL)
        #[arg(long)]
        rust: bool,
    },

    /// Initialize a Blinc project in the current directory
    Init {
        /// Template to use
        #[arg(short, long, default_value = "default")]
        template: String,

        /// Organization/package prefix (e.g., "com.mycompany" results in "com.mycompany.appname")
        #[arg(short, long, default_value = "com.example")]
        org: String,
    },

    /// Check a Blinc project for errors
    Check {
        /// Source file or directory
        #[arg(default_value = ".")]
        source: String,
    },

    /// Show toolchain and target information
    Info,

    /// Check platform setup and dependencies
    Doctor,

    /// Run automation scenarios; validate or export playbooks
    Automation {
        #[command(subcommand)]
        command: AutomationCommands,
    },
}

/// How `blinc dev` compiles your project on file change.
///
/// Defaults to `rust` because the Zyntax DSL toolchain is still in
/// planning. Once it lands, `dsl` will route through Zyntax's
/// Runtime2 JIT instead of cargo + subsecond.
#[derive(Clone, Copy, Debug, Default, clap::ValueEnum)]
pub enum DevMode {
    /// Hot-patch a Rust binary crate via the `subsecond` runtime
    /// (driven by `cargo build` + the dx-CLI websocket protocol
    /// today; native driver is on the roadmap).
    #[default]
    Rust,
    /// Re-compile a `.blinc` DSL project through Zyntax Grammar2 +
    /// Runtime2 JIT and push the new bytecode into the running app.
    /// Currently unimplemented — emits a friendly "not yet ready"
    /// message and exits.
    Dsl,
}

#[derive(Subcommand)]
enum PluginCommands {
    /// Build a plugin
    Build {
        /// Plugin directory
        #[arg(default_value = ".")]
        path: String,

        /// Plugin mode (dynamic or static)
        #[arg(short, long, default_value = "dynamic")]
        mode: String,
    },

    /// Create a new plugin project
    New {
        /// Plugin name
        name: String,
    },
}

#[derive(Subcommand)]
enum ReleaseCommands {
    /// Generate a release manifest JSON file
    Manifest {
        /// Project directory or source path
        #[arg(default_value = ".")]
        source: String,

        /// Artifact platform
        #[arg(long)]
        platform: String,

        /// Artifact architecture
        #[arg(long)]
        arch: String,

        /// Published artifact URL
        #[arg(long)]
        url: String,

        /// Built artifact to inspect and sign
        #[arg(long)]
        artifact_path: Option<String>,

        /// Artifact size in bytes
        #[arg(long)]
        size: Option<u64>,

        /// Artifact SHA-256 hex digest
        #[arg(long)]
        sha256: Option<String>,

        /// Artifact signature
        #[arg(long)]
        signature: Option<String>,

        /// Base64-encoded Ed25519 private key seed
        #[arg(long)]
        private_key: Option<String>,

        /// Optional path to write the matching base64-encoded public key
        #[arg(long)]
        public_key_output: Option<String>,

        /// Manifest output path
        #[arg(long)]
        output: String,

        /// RFC 3339 publish timestamp
        #[arg(long)]
        published_at: String,

        /// Optional release notes URL
        #[arg(long)]
        notes_url: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let filter = if cli.verbose {
        EnvFilter::new("debug")
    } else {
        EnvFilter::new("info")
    };

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(filter)
        .init();

    match cli.command {
        Commands::Build {
            source,
            target,
            release,
            output,
        } => cmd_build(&source, &target, release, output.as_deref()),

        Commands::Dev {
            source,
            target,
            port,
            device,
            mode,
        } => cmd_dev(&source, &target, port, device.as_deref(), mode),

        Commands::Run { source } => cmd_run(&source),

        Commands::Plugin { command } => match command {
            PluginCommands::Build { path, mode } => cmd_plugin_build(&path, &mode),
            PluginCommands::New { name } => cmd_plugin_new(&name),
        },

        Commands::Release { command } => match command {
            ReleaseCommands::Manifest {
                source,
                platform,
                arch,
                url,
                artifact_path,
                size,
                sha256,
                signature,
                private_key,
                public_key_output,
                output,
                published_at,
                notes_url,
            } => cmd_release_manifest(
                &source,
                &platform,
                &arch,
                &url,
                artifact_path.as_deref(),
                size,
                sha256.as_deref(),
                signature.as_deref(),
                private_key.as_deref(),
                public_key_output.as_deref(),
                &output,
                &published_at,
                notes_url.as_deref(),
            ),
        },

        Commands::New {
            name,
            template,
            org,
            rust,
        } => cmd_new(&name, &template, &org, rust),

        Commands::Init { template, org } => cmd_init(&template, &org),

        Commands::Check { source } => cmd_check(&source),

        Commands::Info => cmd_info(),

        Commands::Doctor => cmd_doctor(),

        Commands::Automation { command } => cmd_automation(command),
    }
}

fn cmd_build(source: &str, target: &str, release: bool, output: Option<&str>) -> Result<()> {
    let path = PathBuf::from(source);
    let project_name = load_build_project_name(&path, release)?;

    info!(
        "Building {} for {} ({})",
        project_name,
        target,
        if release { "release" } else { "debug" }
    );

    // Validate target
    let valid_targets = [
        "desktop", "android", "ios", "macos", "windows", "linux", "wasm",
    ];
    if !valid_targets.contains(&target) {
        anyhow::bail!(
            "Invalid target '{}'. Valid targets: {:?}",
            target,
            valid_targets
        );
    }

    // TODO: When Zyntax Grammar2 is ready:
    // 1. Parse .blinc files
    // 2. Generate Rust code
    // 3. Compile with cargo

    warn!("Build not yet implemented - waiting for Zyntax Grammar2");

    if let Some(out) = output {
        info!("Output will be written to: {}", out);
    }

    Ok(())
}

fn load_build_project_name(path: &std::path::Path, release: bool) -> Result<String> {
    if release {
        return Ok(release::load_release_project(path)?.project.name);
    }

    Ok(BlincConfig::load_from_dir(project_config_root(path)?)?
        .project
        .name)
}

fn project_config_root(path: &Path) -> Result<&Path> {
    let start = if path.is_file() {
        path.parent().unwrap_or(path)
    } else {
        path
    };

    start
        .ancestors()
        .find(|candidate| {
            candidate.join(".blincproj").exists() || candidate.join("blinc.toml").exists()
        })
        .context("No .blincproj or blinc.toml found for the provided project path")
}

fn cmd_dev(
    source: &str,
    target: &str,
    port: u16,
    device: Option<&str>,
    mode: DevMode,
) -> Result<()> {
    let path = PathBuf::from(source);
    let config = BlincConfig::load_from_dir(project_config_root(&path)?)?;

    info!(
        "Starting dev server for {} on port {} targeting {}",
        config.project.name, port, target
    );

    if let Some(dev) = device {
        info!("Running on device: {}", dev);
    }

    match mode {
        DevMode::Rust => cmd_dev_rust(source, target, port, device),
        DevMode::Dsl => cmd_dev_dsl(source, target, port, device),
    }
}

fn cmd_dev_rust(_source: &str, _target: &str, _port: u16, _device: Option<&str>) -> Result<()> {
    info!(
        "Rust hot-reload mode. Native driver is on the roadmap; \
         install `dioxus-cli` and run `dx serve --hotpatch` to drive \
         patches over the subsecond protocol meanwhile."
    );
    // Existing stub body — file watcher + cargo + subsecond websocket
    // is the level-2 milestone (issue #30).
    warn!(
        "blinc dev --mode rust is not yet implemented — see docs/book/src/advanced/hot-reload.md"
    );
    Ok(())
}

fn cmd_dev_dsl(_source: &str, _target: &str, _port: u16, _device: Option<&str>) -> Result<()> {
    info!("Blinc DSL hot-reload mode (Zyntax Runtime2 JIT).");
    warn!("Blinc DSL toolchain is still in plan — Zyntax Grammar2 + Runtime2 JIT not yet ready");
    Ok(())
}

fn cmd_run(source: &str) -> Result<()> {
    info!("Running {}", source);

    // TODO: Execute compiled binary or interpret source
    warn!("Run not yet implemented - waiting for Zyntax Runtime2");

    Ok(())
}

fn cmd_plugin_build(path: &str, mode: &str) -> Result<()> {
    info!("Building plugin at {} (mode: {})", path, mode);

    let valid_modes = ["dynamic", "static"];
    if !valid_modes.contains(&mode) {
        anyhow::bail!("Invalid mode '{}'. Valid modes: {:?}", mode, valid_modes);
    }

    // TODO: Build the plugin crate with appropriate flags
    warn!("Plugin build not yet implemented");

    Ok(())
}

fn cmd_plugin_new(name: &str) -> Result<()> {
    info!("Creating new plugin: {}", name);

    let path = PathBuf::from(name);
    if path.exists() {
        anyhow::bail!("Directory '{}' already exists", name);
    }

    fs::create_dir_all(&path)?;
    project::create_plugin_project(&path, name)?;

    info!("Plugin created at {}/", name);
    Ok(())
}

fn cmd_release_manifest(
    source: &str,
    platform: &str,
    arch: &str,
    url: &str,
    artifact_path: Option<&str>,
    size: Option<u64>,
    sha256: Option<&str>,
    signature: Option<&str>,
    private_key: Option<&str>,
    public_key_output: Option<&str>,
    output: &str,
    published_at: &str,
    notes_url: Option<&str>,
) -> Result<()> {
    release::write_release_manifest(&release::ReleaseManifestArgs {
        source: PathBuf::from(source),
        platform: platform.to_string(),
        arch: arch.to_string(),
        url: url.to_string(),
        artifact_path: artifact_path.map(PathBuf::from),
        size: size.unwrap_or_default(),
        sha256: sha256.unwrap_or_default().to_string(),
        signature: signature.unwrap_or_default().to_string(),
        private_key: private_key.map(str::to_owned),
        public_key_output: public_key_output.map(PathBuf::from),
        output: PathBuf::from(output),
        published_at: published_at.to_string(),
        notes_url: notes_url.map(str::to_owned),
    })?;

    info!("Release manifest written to {}", output);
    Ok(())
}

fn cmd_new(name: &str, template: &str, org: &str, rust: bool) -> Result<()> {
    let path = PathBuf::from(name);

    // Extract the actual project name from the path (last component)
    let project_name = path.file_name().and_then(|n| n.to_str()).unwrap_or(name);

    project::validate_org_name(org)?;

    if rust {
        info!("Creating new Rust project: {}", project_name);
    } else {
        info!(
            "Creating new project: {} (template: {})",
            project_name, template
        );
    }
    info!("Organization prefix: {}", org);

    if path.exists() {
        anyhow::bail!("Directory '{}' already exists", name);
    }

    fs::create_dir_all(&path)?;

    if rust {
        project::create_rust_project(&path, project_name, org)?;
        info!("Rust project created at {}/", name);
        info!("To get started:");
        info!("  cd {}", name);
        info!("  cargo run --features desktop");
    } else {
        project::create_project(&path, project_name, template, org)?;
        info!("Project created at {}/", name);
        info!("To get started:");
        info!("  cd {}", name);
        info!("  blinc dev");
    }

    Ok(())
}

fn cmd_init(template: &str, org: &str) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let name = cwd
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("blinc_app");

    info!(
        "Initializing Blinc project in current directory (template: {})",
        template
    );
    info!("Organization prefix: {}", org);

    // Check if already initialized
    if cwd.join(".blincproj").exists() {
        anyhow::bail!("This directory already contains a .blincproj");
    }
    if cwd.join("blinc.toml").exists() {
        anyhow::bail!("This directory already contains a blinc.toml (legacy format)");
    }

    project::create_project(&cwd, name, template, org)?;

    info!("Project initialized!");
    info!("Run `blinc dev` to start development");

    Ok(())
}

fn cmd_check(source: &str) -> Result<()> {
    let path = PathBuf::from(source);
    let config = BlincConfig::load_from_dir(project_config_root(&path)?)?;

    info!("Checking project: {}", config.project.name);

    // TODO: Parse and validate all .blinc files
    warn!("Check not yet implemented - waiting for Zyntax Grammar2");

    Ok(())
}

fn cmd_info() -> Result<()> {
    println!("Blinc UI Framework");
    println!("==================");
    println!();
    let git_hash = option_env!("BLINC_GIT_HASH").unwrap_or("unknown");
    println!("Version: {} ({})", env!("CARGO_PKG_VERSION"), git_hash);
    println!();
    println!("Supported targets:");
    println!("  - desktop (native window)");
    println!("  - macos");
    println!("  - windows");
    println!("  - linux");
    println!("  - android");
    println!("  - ios");
    println!("  - wasm (WebGPU/WebGL2)");
    println!();
    println!("Build modes:");
    println!("  - JIT (development, hot-reload) - requires Zyntax Runtime2");
    println!("  - AOT (production) - requires Zyntax Grammar2");
    println!();
    println!("Status:");
    println!("  - Core reactive system: Ready");
    println!("  - FSM runtime: Ready");
    println!("  - Animation system: Ready");
    println!("  - Zyntax integration: Pending Grammar2/Runtime2");

    Ok(())
}

fn cmd_doctor() -> Result<()> {
    let categories = doctor::run_doctor();
    doctor::print_doctor_results(&categories);

    // Return error if there are critical issues
    let has_errors = categories
        .iter()
        .any(|c| c.status() == doctor::CheckStatus::Error);

    if has_errors {
        std::process::exit(1);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn release_build_path_requires_blincproj() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time must be after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("blinc_cli_release_build_loader_{nonce}"));

        fs::create_dir_all(&root).expect("temp project root should be created");
        fs::write(
            root.join("blinc.toml"),
            r#"
                [project]
                name = "LegacyDemo"
                version = "0.1.0"
            "#,
        )
        .expect("legacy blinc.toml should be written");

        let err = load_build_project_name(&root, true)
            .expect_err("release builds should require .blincproj metadata");
        assert!(
            err.to_string().contains("No .blincproj found"),
            "release build path should use the release loader"
        );

        assert_eq!(
            load_build_project_name(&root, false)
                .expect("debug builds should keep legacy config support"),
            "LegacyDemo",
            "non-release builds should keep using the legacy config loader"
        );

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn release_build_path_discovers_project_root_from_nested_directory() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time must be after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("blinc_cli_release_build_nested_{nonce}"));
        let nested = root.join("src/features");

        fs::create_dir_all(&nested).expect("nested directory should be created");
        fs::write(
            root.join(".blincproj"),
            r#"
                [project]
                name = "Demo"
                version = "0.1.0"

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

        assert_eq!(
            load_build_project_name(&nested, true)
                .expect("release builds should discover the project root from nested dirs"),
            "Demo",
            "release build path should load metadata from the nearest ancestor .blincproj"
        );

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn debug_build_path_discovers_project_root_from_nested_directory() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time must be after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("blinc_cli_debug_build_nested_{nonce}"));
        let nested = root.join("src/features");

        fs::create_dir_all(&nested).expect("nested directory should be created");
        fs::write(
            root.join(".blincproj"),
            r#"
                [project]
                name = "Demo"
                version = "0.1.0"

                [platforms.android]
                package = "io.test.demo"

                [updates]
                enabled = true
                channel = "stable"
                manifest_url = "https://example.com/releases/manifest.json"
                public_key = "abc"
            "#,
        )
        .expect(".blincproj should be written");

        assert_eq!(
            load_build_project_name(&nested, false)
                .expect("debug builds should discover the project root from nested dirs"),
            "Demo",
            "debug build path should load metadata from the nearest ancestor project config"
        );

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn cmd_new_does_not_leave_directory_on_invalid_org() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time must be after epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("blinc_cli_invalid_org_{nonce}"));
        let path_string = path.to_string_lossy().to_string();

        let err = cmd_new(&path_string, "default", "123.example", false)
            .expect_err("invalid org should fail before scaffolding begins");
        assert!(
            err.to_string().contains("Invalid organization name"),
            "invalid org error should be surfaced to the caller"
        );
        assert!(
            !path.exists(),
            "failed project creation should not leave an empty directory behind"
        );
    }

    #[test]
    fn cmd_new_non_rust_uses_leaf_name_for_scaffold_metadata() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time must be after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("blinc_cli_nested_new_{nonce}"));
        let path = root.join("apps/Demo App");
        let path_string = path.to_string_lossy().to_string();

        cmd_new(&path_string, "default", "io.blinc.dev", false)
            .expect("non-rust project creation should succeed from a nested path");

        let project = crate::config::BlincProject::load_from_dir(&path)
            .expect("scaffolded non-rust project should include .blincproj");
        assert_eq!(
            project.project.name, "Demo App",
            "scaffold metadata should use the leaf project name instead of the full path"
        );
        assert_eq!(
            project
                .platforms
                .android
                .as_ref()
                .map(|android| android.package.as_str()),
            Some("io.blinc.dev.demo_app"),
            "android package ids should use the normalized leaf project name"
        );

        let readme =
            fs::read_to_string(path.join("README.md")).expect("generated README should exist");
        assert!(
            readme.contains("dist/demo_app.zip"),
            "release guidance should use the normalized leaf project name in artifact paths"
        );
        assert!(
            !readme.contains("apps/Demo App"),
            "generated files should not embed the full requested path as project metadata"
        );

        let _ = fs::remove_dir_all(&root);
    }
}
