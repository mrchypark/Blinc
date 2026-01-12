//! Blinc Debugger - Visual debugger for UI recordings
//!
//! A standalone application for viewing and analyzing recorded UI sessions.
//! Provides:
//! - Element tree visualization with diff highlighting
//! - UI preview with debug overlay
//! - Element inspector panel
//! - Event timeline with playback controls
//!
//! Layout based on Phase 12 of the blinc_recorder implementation plan.

mod app;
mod panels;
mod theme;

use anyhow::Result;
use blinc_theme::{ColorScheme, ThemeState};
use clap::Parser;
use std::path::PathBuf;

/// Visual debugger for Blinc UI recordings
#[derive(Parser, Debug)]
#[command(name = "blinc-debugger")]
#[command(about = "Visual debugger for Blinc UI recordings")]
#[command(version)]
struct Args {
    /// Recording file to open (optional)
    #[arg(short, long)]
    file: Option<PathBuf>,

    /// Connect to debug server at address
    #[arg(short, long, default_value = "127.0.0.1:9999")]
    connect: Option<String>,

    /// Window width
    #[arg(long, default_value = "1280")]
    width: u32,

    /// Window height
    #[arg(long, default_value = "800")]
    height: u32,
}

fn main() -> Result<()> {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Initialize theme with dark mode for debugger
    ThemeState::init_default();
    ThemeState::get().set_scheme(ColorScheme::Dark);

    let args = Args::parse();

    log::info!("Starting Blinc Debugger");

    if let Some(ref file) = args.file {
        log::info!("Opening recording: {}", file.display());
    }

    if let Some(ref addr) = args.connect {
        log::info!("Will connect to debug server at: {}", addr);
    }

    // Run the app
    app::run(args.width, args.height, args.file, args.connect)
}
