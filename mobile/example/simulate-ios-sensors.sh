#!/usr/bin/env bash
# iOS Simulator sensor driving helper for Blinc example.
#
# What this script can drive on Simulator:
# - GPS/location (and heading derived from movement direction)
#
# What Simulator cannot faithfully drive via simctl:
# - accelerometer, gyroscope, magnetometer, barometer, pedometer, activity
#
# Use this script to continuously move location every 0.5s and verify sensor UI/logs.

set -euo pipefail

DEVICE_ID="booted"
BUNDLE_ID="com.blinc.example"
INTERVAL_SECS="0.5"
MODE="set-loop" # set-loop | start-route | scenario
SCENARIO_NAME="City Run"
WITH_LOGS=1
WITH_SCREENSHOTS=0
SCREENSHOT_INTERVAL_SECS="2.0"
OUTPUT_DIR="${TMPDIR:-/tmp}/blinc-sensor-check"
LAUNCH_APP=1
CURRENT_DEVICE=""
LOG_PID=""
SHOT_PID=""
CLEANED_UP=0

# Small SF loop route (walk-scale)
COORDS=(
  "37.7749,-122.4117"
  "37.7752,-122.4110"
  "37.7755,-122.4101"
  "37.7750,-122.4094"
  "37.7743,-122.4098"
  "37.7740,-122.4107"
)

usage() {
  cat <<USAGE
Usage: $(basename "$0") [options]

Options:
  --device <udid|booted>          Simulator device (default: booted)
  --bundle <bundle_id>            App bundle id (default: com.blinc.example)
  --interval <seconds>            Location update interval (default: 0.5)
  --mode <set-loop|start-route|scenario>
                                  set-loop: repeatedly runs 'simctl location set ...'
                                  start-route: uses 'simctl location start --interval=...'
                                  scenario: uses built-in scenario via 'simctl location run'
  --scenario <name>               Scenario name for --mode scenario (default: City Run)
  --coords "lat,lon;lat,lon;..."  Override coordinate list for set-loop/start-route
  --no-logs                       Disable log stream monitor
  --screenshots                   Save screenshots periodically
  --screenshot-interval <seconds> Screenshot interval (default: 2.0)
  --output-dir <path>             Output directory for logs/screenshots
  --no-launch                     Do not auto-launch app
  -h, --help                      Show this help

Examples:
  $(basename "$0") --device 6D96849D-E6A4-4628-940A-9EFED1BD0829
  $(basename "$0") --mode start-route --interval 0.5
  $(basename "$0") --mode scenario --scenario "Freeway Drive"
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --device)
      DEVICE_ID="$2"; shift 2 ;;
    --bundle)
      BUNDLE_ID="$2"; shift 2 ;;
    --interval)
      INTERVAL_SECS="$2"; shift 2 ;;
    --mode)
      MODE="$2"; shift 2 ;;
    --scenario)
      SCENARIO_NAME="$2"; shift 2 ;;
    --coords)
      IFS=';' read -r -a COORDS <<< "$2"; shift 2 ;;
    --no-logs)
      WITH_LOGS=0; shift ;;
    --screenshots)
      WITH_SCREENSHOTS=1; shift ;;
    --screenshot-interval)
      SCREENSHOT_INTERVAL_SECS="$2"; shift 2 ;;
    --output-dir)
      OUTPUT_DIR="$2"; shift 2 ;;
    --no-launch)
      LAUNCH_APP=0; shift ;;
    -h|--help)
      usage; exit 0 ;;
    *)
      echo "Unknown arg: $1" >&2
      usage
      exit 1 ;;
  esac
done

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "Missing required command: $1" >&2
    exit 1
  }
}

resolve_device() {
  if [[ "$DEVICE_ID" != "booted" ]]; then
    echo "$DEVICE_ID"
    return
  fi

  local booted
  booted="$(xcrun simctl list devices booted | awk -F '[()]' '/Booted/ {print $2; exit}')"
  if [[ -z "$booted" ]]; then
    echo "No booted simulator found. Boot one first." >&2
    exit 1
  fi
  echo "$booted"
}

grant_permissions() {
  local device="$1"
  set +e
  xcrun simctl privacy "$device" grant location "$BUNDLE_ID" >/dev/null 2>&1
  xcrun simctl privacy "$device" grant location-always "$BUNDLE_ID" >/dev/null 2>&1
  xcrun simctl privacy "$device" grant motion "$BUNDLE_ID" >/dev/null 2>&1
  set -e
}

enable_location_services() {
  local device="$1"
  set +e
  xcrun simctl spawn "$device" defaults write com.apple.locationd LocationServicesEnabled -bool YES >/dev/null 2>&1
  xcrun simctl spawn "$device" defaults write com.apple.locationd LocationServicesEnabledInClonedDevice -bool YES >/dev/null 2>&1
  set -e
}

launch_app() {
  local device="$1"
  set +e
  xcrun simctl launch "$device" "$BUNDLE_ID" >/dev/null 2>&1
  set -e
}

print_check_matrix() {
  cat <<'MATRIX'

[Sensor Check Matrix]
- GPS:              Simulator drive 가능 (이 스크립트로 확인)
- Heading:          이동 경로 기반으로 변화 가능 (GPS 경로 변화로 확인)
- Accelerometer:    simctl로 값 주입 불가 (실기기 필요)
- Gyroscope:        simctl로 값 주입 불가 (실기기 필요)
- Magnetometer:     simctl로 값 주입 불가 (실기기 필요)
- Barometer:        simctl로 값 주입 불가 (실기기 필요)
- Step/Cadence:     simctl로 값 주입 불가 (실기기 필요)
- Activity:         simctl로 값 주입 불가 (실기기 필요)

검증 팁:
1) 앱 Sensor Inspector에서 `Sensors ON`으로 전환
2) status running=true, sample/kinds가 주기적으로 바뀌는지 확인
3) OFF 전환 시 값 갱신이 멈추는지 확인
MATRIX
}

cleanup() {
  local device="${1:-$CURRENT_DEVICE}"
  if [[ "$CLEANED_UP" -eq 1 ]]; then
    return
  fi
  CLEANED_UP=1

  set +e
  if [[ -n "$device" ]]; then
    xcrun simctl location "$device" clear >/dev/null 2>&1
  fi
  if [[ -n "$LOG_PID" ]]; then
    kill "$LOG_PID" >/dev/null 2>&1 || true
  fi
  if [[ -n "$SHOT_PID" ]]; then
    kill "$SHOT_PID" >/dev/null 2>&1 || true
  fi
  echo
  echo "Cleaned up: location cleared, background jobs stopped."
}

handle_interrupt() {
  cleanup "$CURRENT_DEVICE"
  exit 130
}

start_logs() {
  local device="$1"
  mkdir -p "$OUTPUT_DIR"
  local log_file="$OUTPUT_DIR/sim-log-$(date +%Y%m%d-%H%M%S).log"
  echo "Log stream -> $log_file"
  xcrun simctl spawn "$device" log stream \
    --style compact \
    --predicate "process CONTAINS[c] 'Blinc' OR eventMessage CONTAINS[c] 'sensor' OR subsystem CONTAINS[c] 'CoreLocation'" \
    >"$log_file" 2>&1 &
  LOG_PID=$!
}

start_screenshots() {
  local device="$1"
  mkdir -p "$OUTPUT_DIR/screens"
  (
    while true; do
      local ts
      ts="$(date +%Y%m%d-%H%M%S)"
      xcrun simctl io "$device" screenshot "$OUTPUT_DIR/screens/$ts.png" >/dev/null 2>&1 || true
      sleep "$SCREENSHOT_INTERVAL_SECS"
    done
  ) &
  SHOT_PID=$!
  echo "Screenshots -> $OUTPUT_DIR/screens (every ${SCREENSHOT_INTERVAL_SECS}s)"
}

run_set_loop() {
  local device="$1"
  echo "Mode: set-loop (interval=${INTERVAL_SECS}s)"
  echo "Press Ctrl+C to stop."
  while true; do
    for c in "${COORDS[@]}"; do
      xcrun simctl location "$device" set "$c"
      printf '[%s] location set %s\n' "$(date +%H:%M:%S)" "$c"
      sleep "$INTERVAL_SECS"
    done
  done
}

run_start_route() {
  local device="$1"
  echo "Mode: start-route (simctl interpolation, interval=${INTERVAL_SECS}s)"
  echo "Press Ctrl+C to stop."

  # shellcheck disable=SC2086
  xcrun simctl location "$device" start \
    --interval="$INTERVAL_SECS" \
    "${COORDS[@]}"

  # Keep foreground alive so Ctrl+C cleanup runs.
  while true; do
    sleep 1
  done
}

run_scenario() {
  local device="$1"
  echo "Mode: scenario (${SCENARIO_NAME})"
  echo "Press Ctrl+C to stop."
  xcrun simctl location "$device" run "$SCENARIO_NAME"
  while true; do
    sleep 1
  done
}

main() {
  require_cmd xcrun

  local device
  device="$(resolve_device)"
  CURRENT_DEVICE="$device"

  mkdir -p "$OUTPUT_DIR"

  echo "Device:      $device"
  echo "Bundle:      $BUNDLE_ID"
  echo "Output dir:  $OUTPUT_DIR"

  print_check_matrix

  enable_location_services "$device"
  grant_permissions "$device"

  if [[ "$LAUNCH_APP" -eq 1 ]]; then
    launch_app "$device"
  fi

  if [[ "$WITH_LOGS" -eq 1 ]]; then
    start_logs "$device"
  fi

  if [[ "$WITH_SCREENSHOTS" -eq 1 ]]; then
    start_screenshots "$device"
  fi

  trap 'handle_interrupt' INT TERM
  trap 'cleanup "$CURRENT_DEVICE"' EXIT

  case "$MODE" in
    set-loop)
      run_set_loop "$device" ;;
    start-route)
      run_start_route "$device" ;;
    scenario)
      run_scenario "$device" ;;
    *)
      echo "Unsupported mode: $MODE" >&2
      exit 1 ;;
  esac
}

main "$@"
