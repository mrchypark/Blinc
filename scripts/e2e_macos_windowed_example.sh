#!/usr/bin/env bash
set -euo pipefail

# Minimal macOS "e2e smoke" for a windowed blinc_app example.
# Requires: Hammerspoon CLI (`hs`) with IPC enabled in ~/.hammerspoon/init.lua:
#   require("hs.ipc")
#
# Usage:
#   scripts/e2e_macos_windowed_example.sh charts_gallery_demo "blinc_charts: Gallery"
#
# Notes:
# - We intentionally keep N small to avoid long startup times.
# - The example is terminated after the window is detected.

example="${1:-charts_gallery_demo}"
expected_title="${2:-}"

if [[ -z "${expected_title}" ]]; then
  echo "error: expected window title argument is required" >&2
  exit 2
fi

if ! command -v hs >/dev/null 2>&1; then
  echo "error: missing 'hs' (Hammerspoon CLI). Install Hammerspoon and enable hs.ipc." >&2
  exit 2
fi

if ! hs -c 'print("hs_ok")' >/dev/null 2>&1; then
  echo "error: Hammerspoon IPC is not available." >&2
  echo "Fix: add 'require(\"hs.ipc\")' to ~/.hammerspoon/init.lua and reload Hammerspoon once." >&2
  exit 2
fi

export BLINC_CHARTS_N="${BLINC_CHARTS_N:-5000}"

log_file="${TMPDIR:-/tmp}/blinc-e2e-${example}.log"

# Best-effort: close any existing window first to avoid ending up with two.
hs -A -t 20 -c "
  local expected = [[${expected_title}]]
  for _, w in ipairs(hs.window.allWindows()) do
    local t = w:title() or ''
    if string.find(t, expected, 1, true) then
      w:close()
    end
  end
" >/dev/null 2>&1 || true

(
  cd "$(dirname "$0")/.."
  cargo run -p blinc_app --example "${example}" --features windowed >"${log_file}" 2>&1
) &
pid=$!

cleanup() {
  kill "${pid}" >/dev/null 2>&1 || true
  wait "${pid}" >/dev/null 2>&1 || true
}
trap cleanup EXIT

found="0"
for _ in $(seq 1 60); do
  if hs -A -t 20 -c "
    local expected = [[${expected_title}]]
    local wins = hs.window.allWindows()
    for _, w in ipairs(wins) do
      local t = w:title() or ''
      if string.find(t, expected, 1, true) then
        print('FOUND')
        return
      end
    end
    print('NO')
  " | grep -q "^FOUND$"; then
    found="1"
    break
  fi
  sleep 0.5
done

if [[ "${found}" != "1" ]]; then
  echo "error: window not detected: ${expected_title}" >&2
  echo "log: ${log_file}" >&2
  exit 1
fi

# Best-effort: close the window if we can find it.
hs -A -t 20 -c "
  local expected = [[${expected_title}]]
  for _, w in ipairs(hs.window.allWindows()) do
    local t = w:title() or ''
    if string.find(t, expected, 1, true) then
      w:focus()
      w:close()
      return
    end
  end
" >/dev/null 2>&1 || true

echo "ok: detected window for ${example} (${expected_title})"
