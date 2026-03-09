# counter

A Blinc UI scaffold example. This project is not the canonical mobile-native
reference app.

## Development

```bash
blinc dev
```

## Build

```bash
# Desktop (current platform)
blinc build --release

# Mobile scaffold output
blinc build --target android --release
blinc build --target ios --release
```

## Support Tiers

- Tier 1: scaffold generation and desktop/local structure validation
- Tier 2: Android/iOS output depends on generated platform projects
- Tier 3: release packaging and native feature parity are out of scope here

Use [`mobile/example`](../../mobile/example/README.md)
as the canonical native reference app for IME, permissions, sensors, and bridge behavior.

Repo-wide native support tiers are defined in
[`docs/native-readiness.md`](../../docs/native-readiness.md).

## Project Structure

```
counter/
├── .blincproj           # Project configuration
├── src/
│   └── main.blinc       # Application entry point
├── assets/              # Static assets (images, fonts, etc.)
├── plugins/             # Local plugins
└── platforms/           # Platform-specific code
    ├── android/         # Android project files
    ├── ios/             # iOS scaffold files
    ├── macos/           # macOS app bundle config
    ├── windows/         # Windows executable config
    └── linux/           # Linux desktop config
```

## Configuration

Edit `.blincproj` to configure:
- Project metadata (name, version, description)
- Platform-specific settings (package IDs, SDK versions)
- Dependencies (plugins, external packages)
