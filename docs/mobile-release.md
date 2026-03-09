# Mobile Release Packaging

This document defines the current packaging contract for the canonical native
reference app at [`mobile/example`](../mobile/example/README.md).

## Build Lanes

- `debug smoke`
  - Android: `./build-android.sh debug`
  - iOS: `./build-ios.sh debug-libs`
- `release artifact`
  - Android APK: `./build-android.sh release`
  - Android AAB: `./build-android.sh bundle-release`
  - iOS static libs: `./build-ios.sh release-libs`
  - iOS archive: `./build-ios.sh archive`
  - iOS export: `./build-ios.sh export-archive`

## Artifact Paths

- Android debug APK:
  - `mobile/example/artifacts/android/debug-apk/app-debug.apk`
- Android release APK:
  - `mobile/example/artifacts/android/release-apk/app-release.apk`
- Android release AAB:
  - `mobile/example/artifacts/android/release-bundle/app-release.aab`
- iOS release libraries:
  - `mobile/example/artifacts/ios/libs/release/device/libexample.a`
  - `mobile/example/artifacts/ios/libs/release/simulator/libexample.a`
- iOS archive default:
  - `mobile/example/artifacts/ios/archive/BlincApp.xcarchive`
- iOS export default:
  - `mobile/example/artifacts/ios/export/`

## Signing Inputs

### Android

- `BLINC_ANDROID_KEYSTORE_PATH`
- `BLINC_ANDROID_KEYSTORE_PASSWORD`
- `BLINC_ANDROID_KEY_ALIAS`
- `BLINC_ANDROID_KEY_PASSWORD`

If these are omitted, Gradle must already be configured for unsigned or local
debug-style builds. The script still exports the generated APK/AAB when Gradle
produces one.

### iOS

- `BLINC_IOS_TEAM_ID`
- `BLINC_IOS_CODE_SIGNING_ALLOWED`
- `BLINC_IOS_ARCHIVE_PATH`
- `BLINC_IOS_EXPORT_PATH`
- `BLINC_IOS_EXPORT_OPTIONS_PLIST`

`archive` defaults to `CODE_SIGNING_ALLOWED=NO` so CI and local packaging can
produce an archive contract without assuming developer signing is available.
`export-archive` requires an export options plist because `.ipa` export policy
is distribution-specific.

## CI Split

- `runtime correctness`
  - Rust tests
  - bridge sync check
  - debug-oriented mobile crate checks
- `packaging`
  - Android release APK/AAB generation
  - iOS archive command contract validation

Current repository automation validates the command contract and artifact paths.
Store submission, notarization, and App Store / Play Console upload remain out
of scope.
