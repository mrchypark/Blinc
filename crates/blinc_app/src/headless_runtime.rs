//! Headless runtime primitives for diagnostics execution.

use anyhow::{bail, Result};

/// Configuration for deterministic headless frame execution.
#[derive(Debug, Clone, Copy)]
pub struct HeadlessRunConfig {
    /// Logical viewport width used by the headless run.
    pub width: u32,
    /// Logical viewport height used by the headless run.
    pub height: u32,
    /// Number of frames to execute.
    pub max_frames: u32,
    /// Logical milliseconds between frames.
    pub tick_ms: u64,
    /// Probe sampling interval in frames (1 = every frame, 4 = every 4 frames).
    pub probe_every_frames: u32,
}

impl Default for HeadlessRunConfig {
    fn default() -> Self {
        Self {
            width: 1280,
            height: 720,
            max_frames: 1,
            tick_ms: 16,
            probe_every_frames: 4,
        }
    }
}

/// Frame context passed to headless frame callbacks.
#[derive(Debug, Clone, Copy)]
pub struct HeadlessContext {
    pub frame_index: u32,
    pub width: u32,
    pub height: u32,
    pub elapsed_ms: u64,
}

/// Deterministic headless runtime loop.
pub struct HeadlessRuntime;

impl HeadlessRuntime {
    /// Run a fixed frame budget in headless mode.
    pub fn run<F>(cfg: HeadlessRunConfig, mut on_frame: F) -> Result<()>
    where
        F: FnMut(&HeadlessContext),
    {
        if cfg.width == 0 || cfg.height == 0 {
            bail!("headless dimensions must be non-zero");
        }
        if cfg.max_frames == 0 {
            bail!("headless max_frames must be > 0");
        }
        if cfg.tick_ms == 0 {
            bail!("headless tick_ms must be > 0");
        }

        for frame in 0..cfg.max_frames {
            let elapsed_ms = cfg.tick_ms.saturating_mul(frame as u64);
            on_frame(&HeadlessContext {
                frame_index: frame,
                width: cfg.width,
                height: cfg.height,
                elapsed_ms,
            });
        }

        Ok(())
    }
}
