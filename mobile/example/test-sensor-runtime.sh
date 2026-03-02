#!/bin/bash
# Runtime sensor verification helper for Blinc mobile example.
# Usage:
#   ./test-sensor-runtime.sh android
#   ./test-sensor-runtime.sh ios-sim

set -euo pipefail

MODE="${1:-}"

if [[ -z "${MODE}" ]]; then
  echo "Usage: $0 <android|ios-sim>"
  exit 1
fi

case "${MODE}" in
  android)
    command -v adb >/dev/null 2>&1 || {
      echo "adb not found. Install Android platform-tools first."
      exit 1
    }

    if [[ -z "$(adb devices | awk 'NR>1 && $2==\"device\" {print $1}')" ]]; then
      echo "No Android device/emulator is connected."
      echo "Connect a device (or start an emulator), then rerun this script."
      exit 1
    fi

    adb logcat -c
    echo "Android runtime sensor log monitor started."
    echo "Now launch the app and grant Location + Activity Recognition permissions."
    echo
    echo "Watching for sensor lifecycle logs..."
    adb logcat | grep --line-buffered -E "BlincSensor|Sensor permissions requested|Supported mobile sensors|Sensor batch #|Mobile sensors started|Mobile sensors stopped|frame-stats|start: session|stop: session"
    ;;

  ios-sim)
    command -v xcrun >/dev/null 2>&1 || {
      echo "xcrun not found. Install Xcode command line tools first."
      exit 1
    }

    if ! xcrun simctl list devices booted | grep -q "Booted"; then
      echo "No booted iOS Simulator found."
      echo "Boot a simulator first (or run from Xcode), then rerun this script."
      exit 1
    fi

    echo "iOS simulator runtime sensor log monitor started."
    echo "Run the app in the booted simulator and grant location/motion permissions."
    echo
    echo "Watching for sensor lifecycle logs..."
    xcrun simctl spawn booted log stream \
      --style compact \
      --predicate 'eventMessage CONTAINS "Sensor permissions requested" OR eventMessage CONTAINS "Supported mobile sensors" OR eventMessage CONTAINS "Sensor batch #" OR eventMessage CONTAINS "Mobile sensors started" OR eventMessage CONTAINS "Mobile sensors stopped"'
    ;;

  *)
    echo "Unknown mode: ${MODE}"
    echo "Usage: $0 <android|ios-sim>"
    exit 1
    ;;
esac
