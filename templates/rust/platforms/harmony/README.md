# Blinc HarmonyOS Example

HarmonyOS/OpenHarmony platform integration for the Blinc UI framework.

## Prerequisites

1. **DevEco Studio 5.0+** - Download from [Huawei Developer](https://developer.huawei.com/consumer/en/download/)
2. **OpenHarmony SDK** - Install via DevEco Studio SDK Manager (API 12+)
3. **Rust toolchain** - With OHOS targets installed:
   ```bash
   rustup target add aarch64-unknown-linux-ohos
   rustup target add x86_64-unknown-linux-ohos
   ```

## Initial Setup (First Time Only)

### 1. Configure SDK in DevEco Studio

If you see "SDK management mode has changed" error when building:

1. Open DevEco Studio
2. Go to **Tools > SDK Manager**
3. Under **SDK > OpenHarmony** tab:
   - Ensure API 12 (or latest) is installed
   - Click **Apply** to update/reinstall SDK components
4. Restart DevEco Studio

### 2. Configure local.properties

Create or update `local.properties` in this directory:

```properties
# Point to your SDK location (do NOT commit this file)
sdk.dir=/Users/YOUR_USERNAME/Library/OpenHarmony/Sdk/20
```

Replace `YOUR_USERNAME` with your actual username and `20` with your SDK API version.

## Building the Native Library

1. Set up the environment:
   ```bash
   # Copy and edit .env file
   cp ../../.env.example ../../.env
   # Edit .env to set OHOS_NDK_HOME to your SDK native path
   # Example: OHOS_NDK_HOME=/Users/YOUR_USERNAME/Library/OpenHarmony/Sdk/20/native
   ```

2. Build the native library:
   ```bash
   cd ../..
   ./build-ohos.sh aarch64        # For ARM64 devices
   ./build-ohos.sh x86_64         # For x86_64 emulator
   ```

3. Copy the built library to the project:
   ```bash
   mkdir -p platforms/harmony/entry/libs/arm64-v8a
   cp target/aarch64-unknown-linux-ohos/release/libexample.so platforms/harmony/entry/libs/arm64-v8a/
   ```

## Building the App

1. Open this directory (`platforms/harmony/`) in DevEco Studio
2. Wait for project sync to complete
3. If prompted to upgrade project structure, click **OK**
4. Build HAP: **Build > Build Hap(s)/APP(s) > Build Hap(s)**

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
