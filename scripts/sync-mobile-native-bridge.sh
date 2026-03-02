#!/bin/bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

pairs=(
  "extensions/blinc_platform_android/templates/BlincNativeBridge.kt:mobile/example/platforms/android/app/src/main/kotlin/com/blinc/BlincNativeBridge.kt"
  "extensions/blinc_platform_ios/templates/BlincNativeBridge.swift:mobile/example/platforms/ios/BlincApp/BlincNativeBridge.swift"
)

for pair in "${pairs[@]}"; do
  src="${pair%%:*}"
  dst="${pair#*:}"
  src_path="$ROOT_DIR/$src"
  dst_path="$ROOT_DIR/$dst"

  mkdir -p "$(dirname "$dst_path")"
  cp "$src_path" "$dst_path"
  echo "Synced: $src -> $dst"
done

echo "Native bridge template sync completed."
