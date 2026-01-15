# blinc_platform_harmony

HarmonyOS platform support for the Blinc UI framework.

## Overview

This crate provides HarmonyOS-specific implementations of the `blinc_platform` traits:

- **HarmonyPlatform** - Main platform implementation using XComponent
- **HarmonyWindow** - Window wrapper using XComponent's NativeWindow
- **HarmonyEventLoop** - Callback-based event handling
- **HarmonyAssetLoader** - Asset loading from rawfile directory
- **N-API Bridge** - ArkTS/JavaScript interop

## Requirements

- DevEco Studio 4.0+
- OHOS SDK
- HarmonyOS 3.0+ device or emulator

## Building

```bash
# Build native module
hvigorw assembleHap

# Or using DevEco Studio
# Build > Build Hap(s)/APP(s) > Build Hap(s)
```

## Module Configuration

Example `entry/src/main/module.json5`:

```json5
{
  "module": {
    "name": "blinc_native",
    "type": "shared",
    "deviceTypes": ["phone", "tablet"],
    "deliveryWithInstall": true,
    "pages": "$profile:main_pages"
  }
}
```

## ArkTS Integration

```typescript
import blinc from 'libblinc_platform_harmony.so'

@Entry
@Component
struct Index {
  private context: number = 0

  build() {
    Column() {
      XComponent({
        id: 'blinc_canvas',
        type: 'surface',
        libraryname: 'blinc_platform_harmony'
      })
      .width('100%')
      .height('100%')
      .onLoad((xcomponentContext) => {
        console.log('Blinc XComponent loaded')
        this.context = blinc.init(xcomponentContext)
      })
      .onDestroy(() => {
        blinc.destroy(this.context)
      })
    }
  }
}
```

## Status

**Work in Progress** - This platform extension is scaffolded but requires full implementation:

- [ ] XComponent integration
- [ ] N-API module registration
- [ ] Vulkan surface creation
- [ ] Touch event handling
- [ ] Resource manager asset loading

## License

Apache-2.0 OR MIT
