# blinc_recorder

> **Part of the [Blinc UI Framework](https://project-blinc.github.io/Blinc)**
>
> This crate is a component of Blinc, a GPU-accelerated UI framework for Rust.
> For full documentation and guides, visit the [Blinc documentation](https://project-blinc.github.io/Blinc).

Recording, replay, and debug-server infrastructure for Blinc applications.

## Overview

`blinc_recorder` provides:

- user input/event recording
- element tree snapshot capture
- replay with virtual time controls
- local debug server for debugger tooling
- basic test helpers (headless/framebuffer/visual comparison)

## Quick Start

```rust
use std::sync::Arc;
use blinc_recorder::{install_recorder, RecordingConfig, SharedRecordingSession};

let session = Arc::new(SharedRecordingSession::new(RecordingConfig::debug()));
install_recorder(session.clone());

session.start();
// ... run your app ...
session.stop();

let export = session.export();
let json = serde_json::to_string_pretty(&export)?;
std::fs::write("recording.json", json)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Loading + Replay

```rust
use blinc_recorder::{RecordingExport, ReplayConfig, ReplayPlayer};

let json = std::fs::read_to_string("recording.json")?;
let export: RecordingExport = serde_json::from_str(&json)?;

let mut player = ReplayPlayer::new(export, ReplayConfig::interactive());
player.clock_mut().set_speed(1.5);
player.play();

let frame = player.update();
if frame.has_snapshot() {
    // inspect snapshot
}
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Debug Server Integration

```rust
// Debug build convenience macro:
// - creates a debug recording session
// - installs recorder + hooks
// - starts local debug server
// - starts recording
let (_session, _server) = blinc_recorder::enable_debug_server!("my_app");
```

`blinc_debugger` can connect via:

```bash
blinc-debugger --connect my_app
# or explicit socket path on Unix
blinc-debugger --connect unix:/tmp/blinc/my_app.sock
# or explicit TCP endpoint
blinc-debugger --connect tcp:127.0.0.1:7331
```

## Blinc Layout Integration

If your app uses `blinc_layout`, enable the `recorder` feature so layout-side recorder bridge code is active:

```toml
blinc_layout = { version = "0.1.12", features = ["recorder"] }
```

## Optional Features

- `png`: enable PNG exporting in framebuffer helpers

## License

MIT OR Apache-2.0
