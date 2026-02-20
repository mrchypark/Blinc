#!/bin/bash
# Drive Android emulator virtual sensors for Blinc mobile example.
#
# Features:
# - updates GPS + accelerometer + gyroscope + magnetometer values repeatedly
# - default interval is 0.5s
# - clean Ctrl+C shutdown
#
# Usage:
#   ./simulate-android-sensors.sh
#   ./simulate-android-sensors.sh --device emulator-5554 --interval 0.5
#   ./simulate-android-sensors.sh --list-only

set -euo pipefail

DEVICE=""
INTERVAL="0.5"
LIST_ONLY=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --device)
      DEVICE="${2:-}"
      shift 2
      ;;
    --interval)
      INTERVAL="${2:-0.5}"
      shift 2
      ;;
    --list-only)
      LIST_ONLY=1
      shift
      ;;
    -h|--help)
      sed -n '1,30p' "$0"
      exit 0
      ;;
    *)
      echo "Unknown option: $1"
      exit 1
      ;;
  esac
done

command -v adb >/dev/null 2>&1 || {
  echo "adb not found. Install Android platform-tools first."
  exit 1
}

if [[ -z "$DEVICE" ]]; then
  DEVICE="$(adb devices | awk 'NR>1 && $2=="device" {print $1; exit}')"
fi

if [[ -z "$DEVICE" ]]; then
  echo "No Android device/emulator in 'device' state."
  echo "Start an emulator first, then rerun."
  exit 1
fi

echo "Using device: $DEVICE"
echo "Sensor capability status:"
adb -s "$DEVICE" emu sensor status || true

if [[ "$LIST_ONLY" -eq 1 ]]; then
  exit 0
fi

LATITUDES=(37.7749 37.7755 37.7760 37.7754 37.7748 37.7743)
LONGITUDES=(-122.4117 -122.4108 -122.4101 -122.4096 -122.4104 -122.4112)
ACCELS=("0.00:0.00:9.81" "0.18:0.04:9.73" "0.35:0.08:9.65" "0.12:-0.10:9.72" "-0.22:0.03:9.78" "-0.10:0.06:9.80")
GYROS=("0.00:0.00:0.00" "0.02:0.01:0.00" "0.04:0.02:-0.01" "0.01:-0.01:0.02" "-0.03:0.01:0.01" "-0.01:0.00:-0.02")
MAGS=("33.1:4.8:-40.2" "33.8:5.1:-39.6" "34.4:5.5:-39.1" "34.0:5.2:-39.8" "33.5:4.9:-40.4" "33.0:4.6:-40.9")

running=1
trap 'running=0' INT TERM

index=0
echo
echo "Streaming virtual sensor updates every ${INTERVAL}s (Ctrl+C to stop)"

while [[ "$running" -eq 1 ]]; do
  i=$((index % ${#LATITUDES[@]}))

  lat="${LATITUDES[$i]}"
  lon="${LONGITUDES[$i]}"
  accel="${ACCELS[$i]}"
  gyro="${GYROS[$i]}"
  mag="${MAGS[$i]}"

  # geo fix expects "longitude latitude"
  adb -s "$DEVICE" emu geo fix "$lon" "$lat" >/dev/null
  adb -s "$DEVICE" emu sensor set acceleration "$accel" >/dev/null || true
  adb -s "$DEVICE" emu sensor set gyroscope "$gyro" >/dev/null || true
  adb -s "$DEVICE" emu sensor set magnetic-field "$mag" >/dev/null || true

  echo "[$(date '+%H:%M:%S')] geo=${lat},${lon} accel=${accel} gyro=${gyro} magnet=${mag}"

  index=$((index + 1))
  sleep "$INTERVAL" || true
done

echo
echo "Stopped virtual sensor stream."
