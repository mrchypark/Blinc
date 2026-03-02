#!/bin/bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

pairs=(
  "extensions/blinc_platform_android/templates/BlincNativeBridge.kt:mobile/example/platforms/android/app/src/main/kotlin/com/blinc/BlincNativeBridge.kt"
  "extensions/blinc_platform_ios/templates/BlincNativeBridge.swift:mobile/example/platforms/ios/BlincApp/BlincNativeBridge.swift"
)

failed=0
for pair in "${pairs[@]}"; do
  src="${pair%%:*}"
  dst="${pair#*:}"
  src_path="$ROOT_DIR/$src"
  dst_path="$ROOT_DIR/$dst"

  if ! diff -u "$src_path" "$dst_path" >/tmp/blinc-native-bridge.diff 2>&1; then
    echo "[FAIL] Native bridge files are out of sync:"
    echo "  source: $src"
    echo "  target: $dst"
    echo "  diff (first 80 lines):"
    sed -n '1,80p' /tmp/blinc-native-bridge.diff
    failed=1
  else
    echo "[OK] $dst is synced from $src"
  fi
done

if [[ "$failed" -ne 0 ]]; then
  echo
  echo "Run: scripts/sync-mobile-native-bridge.sh"
  exit 1
fi

echo "All native bridge files are in sync."
