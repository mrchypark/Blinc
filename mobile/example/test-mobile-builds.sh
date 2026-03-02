#!/bin/bash
# End-to-end mobile build verification for the example app.
# - Rust cross-target checks (Android + iOS)
# - Android APK assembly
# - iOS static library build
# - Optional iOS app build (when iOS platform SDK is installed in Xcode)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "== [0/5] Native bridge template sync check =="
"$SCRIPT_DIR/../../scripts/check-mobile-native-bridge-sync.sh"

echo
echo "== [1/5] Rust cross-target checks =="
cargo check --target aarch64-linux-android --features android
cargo check --target aarch64-apple-ios --features ios

echo
echo "== [2/5] Android Rust library build =="
cargo ndk -t arm64-v8a -o platforms/android/app/src/main/jniLibs build --release

echo
echo "== [3/5] Android APK assemble (Debug) =="
(
  cd platforms/android
  ./gradlew :app:assembleDebug
)

echo
echo "== [4/5] iOS Rust static libraries build =="
./build-ios.sh

if xcodebuild -showsdks | grep -q "iphoneos"; then
  echo
  echo "== Optional: iOS app build with xcodebuild =="
  (
    cd platforms/ios
    xcodebuild -project BlincApp.xcodeproj \
      -scheme BlincApp \
      -configuration Debug \
      -sdk iphonesimulator \
      -destination "generic/platform=iOS Simulator" \
      CODE_SIGNING_ALLOWED=NO build
  )
else
  echo
  echo "Skipping xcodebuild: iOS SDK platform is not available on this machine."
fi

echo
echo "Mobile example build verification completed."
