# blinc_debugger

> **Part of the [Blinc UI Framework](https://project-blinc.github.io/Blinc)**
>
> This crate is a component of Blinc, a GPU-accelerated UI framework for Rust.
> For full documentation and guides, visit the [Blinc documentation](https://project-blinc.github.io/Blinc).

Visual debugger application for Blinc UI recordings.

## Overview

`blinc_debugger` is a standalone GUI application for inspecting recorded Blinc UI sessions. It provides visual tools for debugging layout issues, tracking events, and analyzing UI state.

## Features

- **Element Tree**: Hierarchical view of recorded element trees with selection
- **UI Preview**: Snapshot preview with cursor/bounds/zoom controls
- **Inspector Panel**: Selected element properties and bounds
- **Event Timeline**: Play/pause/step/seek/speed controls for recorded sessions
- **Server Import**: Load recording export from a running debug server (`--connect`)

## Installation

```bash
cargo install blinc_debugger
```

Or build from source:

```bash
cargo build -p blinc_debugger --release
```

## Usage

```bash
# Open a recording file (positional)
blinc-debugger recording.json

# Open a recording file (flag)
blinc-debugger --file recording.json

# Connect to a running debug server (default app socket mapping)
blinc-debugger --connect blinc_app

# Connect to an explicit Unix socket path
blinc-debugger --connect unix:/tmp/blinc/blinc_app.sock

# Connect to a TCP debug server
blinc-debugger --connect tcp:127.0.0.1:7331

# Or launch and open via UI
blinc-debugger
```

## Interface

```
┌─────────────────────────────────────────────────────────────┐
│  File  View  Help                                           │
├────────────────┬────────────────────────┬───────────────────┤
│                │                        │                   │
│  Element Tree  │    UI Preview          │   Inspector       │
│                │                        │                   │
│  ▼ Root        │  ┌─────────────────┐  │   Type: div       │
│    ▼ Header    │  │                 │  │   Width: 800      │
│      Logo      │  │   [Preview]     │  │   Height: 600     │
│      Nav       │  │                 │  │   Background: #fff│
│    ▼ Content   │  └─────────────────┘  │   ...             │
│      ...       │                        │                   │
│                │                        │                   │
├────────────────┴────────────────────────┴───────────────────┤
│  Event Timeline                                   [▶][◀][▶] │
│  ═══════════════════●═══════════════════════════════════════│
│  00:00.000  Click (200, 150)                                │
│  00:00.500  KeyDown 'a'                                     │
│  00:01.000  Scroll (0, -50)                                 │
└─────────────────────────────────────────────────────────────┘
```

## Features in Detail

### Element Tree

- Expand/collapse element hierarchy
- Highlight elements on hover
- Filter by element type
- Show/hide hidden elements

### UI Preview

- Zoom control
- Cursor overlay
- Bounds overlay
- Element highlighting on selection
- Snapshot metadata (size/element count)

### Inspector Panel

- Element type and ID
- Bounds (x, y, width, height)
- Style properties (background, border, etc.)
- Layout properties (flex, padding, margin)
- Event handlers attached

### Event Timeline

- Play/pause/step through events
- Jump to specific timestamp
- Change playback speed

## Recording Format

The debugger reads JSON files created by `blinc_recorder`:

```json
{
  "config": {
    "app_name": "my_app"
  },
  "events": [...],
  "snapshots": [...],
  "stats": {
    "total_events": 123,
    "total_snapshots": 45
  }
}
```

## License

MIT OR Apache-2.0
