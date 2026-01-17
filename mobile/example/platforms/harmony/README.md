# Blinc HarmonyOS Example

HarmonyOS/OpenHarmony platform integration for the Blinc UI framework.

## Prerequisites

1. **DevEco Studio** - Download from [Huawei Developer](https://developer.huawei.com/consumer/en/download/)
2. **OpenHarmony SDK** - Install via DevEco Studio SDK Manager
3. **Rust toolchain** - With OHOS targets installed:
   ```bash
   rustup target add aarch64-unknown-linux-ohos
   rustup target add x86_64-unknown-linux-ohos
   ```

## Building the Native Library

1. Set up the environment:
   ```bash
   # Copy and edit .env file
   cp ../../.env.example ../../.env
   # Edit .env to set OHOS_NDK_HOME
   ```

2. Build the native library:
   ```bash
   cd ../..
   ./build-ohos.sh aarch64        # For ARM64 devices
   ./build-ohos.sh x86_64         # For x86_64 emulator
   ```

3. Copy the built library to the project:
   ```bash
   mkdir -p entry/libs/arm64-v8a
   cp ../../target/aarch64-unknown-linux-ohos/release/libexample.so entry/libs/arm64-v8a/
   ```

## Building the App

1. Open this directory in DevEco Studio
2. Sync the project (Build > Sync Project)
3. Build HAP: Build > Build Hap(s)/APP(s) > Build Hap(s)

## Running

- **Emulator**: Use DevEco Studio's built-in emulator
- **Device**: Connect via USB and use `hdc` tool

## Project Structure

```
harmony/
├── AppScope/           # Application-level configuration
│   └── app.json5       # App manifest
├── entry/              # Main entry module
│   └── src/main/
│       ├── ets/        # ArkTS source code
│       │   ├── entryability/
│       │   └── pages/
│       │       └── Index.ets   # Main UI with XComponent
│       ├── resources/  # Resources (strings, colors, etc.)
│       └── module.json5
├── build-profile.json5 # Build configuration
└── oh-package.json5    # Package configuration
```

## Architecture

```
┌─────────────────────────────────────┐
│         ArkTS UI (Index.ets)        │
│              XComponent             │
└─────────────┬───────────────────────┘
              │ N-API
┌─────────────▼───────────────────────┐
│      libexample.so (Rust)           │
│  ┌─────────────────────────────┐    │
│  │    blinc_platform_harmony   │    │
│  │    - XComponent callbacks   │    │
│  │    - Touch input            │    │
│  │    - Surface management     │    │
│  └─────────────────────────────┘    │
│  ┌─────────────────────────────┐    │
│  │       blinc_app             │    │
│  │    - UI tree building       │    │
│  │    - Event routing          │    │
│  │    - Animation              │    │
│  └─────────────────────────────┘    │
│  ┌─────────────────────────────┐    │
│  │       blinc_gpu             │    │
│  │    - Vulkan rendering       │    │
│  └─────────────────────────────┘    │
└─────────────────────────────────────┘
```

## Status

**Work in Progress** - Native integration scaffolded, XComponent callbacks need implementation.
