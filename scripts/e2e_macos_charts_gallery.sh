#!/usr/bin/env bash
set -euo pipefail

# High-signal macOS e2e for the charts gallery demo.
#
# This script intentionally avoids OS-level screenshots (Screen Recording permission).
# Instead, it relies on blinc_app's windowed swapchain readback (BLINC_E2E_* env vars)
# to validate pixels deterministically.
#
# For the sidebar scrollability check we still use Hammerspoon to inject wheel scroll
# events, but the assertion/capture is still done via swapchain readback.
#
# Output artifacts:
# - /tmp/blinc-e2e-<case>-<WxH>.png (and -2.png for multi-capture cases)
# - /tmp/blinc-e2e-<case>-<WxH>.log

example="charts_gallery_demo"
title="blinc_charts: Gallery"

cd "$(dirname "$0")/.."

export BLINC_CHARTS_N="${BLINC_CHARTS_N:-2000}"

# Build once (keeps per-case runtime stable).
cargo build -p blinc_app --example "${example}" --features windowed >/dev/null

ensure_hammerspoon_ipc() {
  if ! command -v hs >/dev/null 2>&1; then
    echo "error: sidebar scroll case requires Hammerspoon CLI ('hs')." >&2
    echo "Install Hammerspoon and enable IPC in ~/.hammerspoon/init.lua: require(\"hs.ipc\")" >&2
    exit 2
  fi

  # Fast-path: IPC works.
  if hs -A -t 2 -c 'print("hs_ok")' >/dev/null 2>&1; then
    return
  fi

  # Best-effort: start the Hammerspoon app (CLI needs a running instance).
  if command -v open >/dev/null 2>&1; then
    open -g -a Hammerspoon >/dev/null 2>&1 || true
  fi

  for _ in $(seq 1 40); do
    if hs -A -t 2 -c 'print("hs_ok")' >/dev/null 2>&1; then
      return
    fi
    sleep 0.25
  done

  echo "error: Hammerspoon IPC is not available." >&2
  echo "Fix: add 'require(\"hs.ipc\")' to ~/.hammerspoon/init.lua and reload Hammerspoon once." >&2
  exit 2
}

run_readback_case() {
  local case_name="$1" # e.g. line|multi|heatmap
  local selected="$2"  # index into ITEMS
  local size="$3"      # WxH, e.g. 1200x840
  local expect="$4"    # blueish|warm

  local out_png="${TMPDIR:-/tmp}/blinc-e2e-${case_name}-${size}.png"
  local out_log="${TMPDIR:-/tmp}/blinc-e2e-${case_name}-${size}.log"

  (
    RUST_BACKTRACE=1 \
    BLINC_WINDOW_SIZE="${size}" \
    BLINC_GALLERY_SELECTED="${selected}" \
    BLINC_E2E_CAPTURE_PATH="${out_png}" \
    BLINC_E2E_EXPECT="${expect}" \
    BLINC_E2E_EXIT=1 \
    cargo run -p blinc_app --example "${example}" --features windowed >"${out_log}" 2>&1
  )

  echo "ok: case=${case_name} size=${size} png=${out_png}"
}

run_sidebar_scroll_case() {
  local size="1200x420"
  local case_name="sidebar_scroll"
  local out_png="${TMPDIR:-/tmp}/blinc-e2e-${case_name}-${size}.png"
  local out_png2="${TMPDIR:-/tmp}/blinc-e2e-${case_name}-${size}-2.png"
  local out_log="${TMPDIR:-/tmp}/blinc-e2e-${case_name}-${size}.log"
  local trigger="${TMPDIR:-/tmp}/blinc-e2e-trigger-${$}"

  ensure_hammerspoon_ipc

  rm -f "${out_png}" "${out_png2}" "${trigger}" || true

  (
    RUST_BACKTRACE=1 \
    BLINC_WINDOW_SIZE="${size}" \
    BLINC_GALLERY_SELECTED="0" \
    BLINC_E2E_CAPTURE_PATH="${out_png}" \
    BLINC_E2E_TRIGGER_PATH="${trigger}" \
    BLINC_E2E_MAX_CAPTURES=2 \
    BLINC_E2E_EXIT=1 \
    cargo run -p blinc_app --example "${example}" --features windowed >"${out_log}" 2>&1
  ) &
  local pid=$!

  cleanup() {
    kill "${pid}" >/dev/null 2>&1 || true
    wait "${pid}" >/dev/null 2>&1 || true
    rm -f "${trigger}" >/dev/null 2>&1 || true
  }
  trap cleanup EXIT

  # Wait for the window, then focus it and move mouse over sidebar.
  hs -A -t 120 -c "
    local expected = [[${title}]]
    local function find_window()
      for _, win in ipairs(hs.window.allWindows()) do
        local t = win:title() or ''
        if string.find(t, expected, 1, true) then
          return win
        end
      end
      return nil
    end
    local win = nil
    for _ = 1, 240 do
      win = find_window()
      if win ~= nil then break end
      hs.timer.usleep(500000)
    end
    if win == nil then
      io.stderr:write('error: window not detected: ' .. expected .. '\\n')
      os.exit(1)
    end
    win:focus()
    local f = win:frame()
    hs.mouse.absolutePosition(hs.geometry.point(f.x + f.w * 0.15, f.y + f.h * 0.65))
  "

  # Capture baseline (trigger #1)
  : > "${trigger}"
  for _ in $(seq 1 240); do
    [[ -f "${out_png}" ]] && break
    sleep 0.25
  done
  if [[ ! -f "${out_png}" ]]; then
    echo "error: did not receive first capture: ${out_png}" >&2
    exit 1
  fi

  # Wheel scroll down over sidebar list.
  hs -A -t 30 -c "
    for _ = 1, 14 do
      hs.eventtap.event.newScrollEvent({0, -140}, {}, 'pixel'):post()
      hs.timer.usleep(50000)
    end
  "

  # Capture after scroll (trigger #2)
  : > "${trigger}"
  for _ in $(seq 1 240); do
    [[ -f "${out_png2}" ]] && break
    sleep 0.25
  done
  if [[ ! -f "${out_png2}" ]]; then
    echo "error: did not receive second capture: ${out_png2}" >&2
    exit 1
  fi

  wait "${pid}"

  local h1 h2
  h1="$(shasum -a 256 "${out_png}" | awk '{print $1}')"
  h2="$(shasum -a 256 "${out_png2}" | awk '{print $1}')"
  if [[ "${h1}" == "${h2}" ]]; then
    echo "error: sidebar scroll did not change captured pixels (hash identical)" >&2
    echo "png1: ${out_png}" >&2
    echo "png2: ${out_png2}" >&2
    exit 1
  fi

  echo "ok: case=${case_name} size=${size} png=${out_png} png2=${out_png2}"
  trap - EXIT
}

run_readback_case "line" 0 "1200x840" "blueish"
run_readback_case "line" 0 "640x520" "blueish"
run_readback_case "line" 0 "480x360" "blueish"
run_readback_case "multi" 1 "1200x840" "blueish"
run_readback_case "heatmap" 7 "1200x840" "warm"
run_readback_case "heatmap" 7 "640x520" "warm"
run_readback_case "heatmap" 7 "480x360" "warm"
run_sidebar_scroll_case

echo "ok: charts gallery e2e finished"
