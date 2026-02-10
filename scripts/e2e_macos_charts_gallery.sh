#!/usr/bin/env bash
set -euo pipefail

# High-signal macOS e2e for the charts gallery demo.
#
# Requires: Hammerspoon CLI (`hs`) with IPC enabled in ~/.hammerspoon/init.lua:
#   require("hs.ipc")
#
# What this checks:
# - Line chart actually contains a blue-ish line (not just the grid)
# - Heatmap renders without panicking and shows non-grid colors
# - App behaves under a smaller window (responsive layout path)
#
# Output artifacts:
# - /tmp/blinc-e2e-<case>-<WxH>.png
# - /tmp/blinc-e2e-<case>-<WxH>.log

if ! command -v hs >/dev/null 2>&1; then
  echo "error: missing 'hs' (Hammerspoon CLI). Install Hammerspoon and enable hs.ipc." >&2
  exit 2
fi

example="charts_gallery_demo"
title="blinc_charts: Gallery"

export BLINC_CHARTS_N="${BLINC_CHARTS_N:-2000}"

cd "$(dirname "$0")/.."

# Build once (keeps per-case runtime stable).
cargo build -p blinc_app --example "${example}" --features windowed >/dev/null

run_case() {
  local case_name="$1" # e.g. line|heatmap
  local selected="$2"  # index into ITEMS
  local size="$3"      # WxH, e.g. 1200x840

  local w="${size%x*}"
  local h="${size#*x}"

  local out_png="${TMPDIR:-/tmp}/blinc-e2e-${case_name}-${w}x${h}.png"
  local out_log="${TMPDIR:-/tmp}/blinc-e2e-${case_name}-${w}x${h}.log"

  (
    RUST_BACKTRACE=1 \
    BLINC_GALLERY_SELECTED="${selected}" \
    cargo run -p blinc_app --example "${example}" --features windowed >"${out_log}" 2>&1
  ) &
  local pid=$!

  cleanup() {
    kill "${pid}" >/dev/null 2>&1 || true
    wait "${pid}" >/dev/null 2>&1 || true
  }
  trap cleanup EXIT

  hs -A -t 180 -c "
    local expected = [[${title}]]
    local case_name = [[${case_name}]]
    local out_png = [[${out_png}]]
    local w = ${w}
    local h = ${h}

    local function find_window()
      for _, win in ipairs(hs.window.allWindows()) do
        local t = win:title() or ''
        if string.find(t, expected, 1, true) then
          return win
        end
      end
      return nil
    end

    -- Close any leftover windows from a previous failed run (best-effort).
    for _, win in ipairs(hs.window.allWindows()) do
      local t = win:title() or ''
      if string.find(t, expected, 1, true) then
        win:close()
      end
    end

    -- Wait for the window to appear.
    local win = nil
    for _ = 1, 240 do -- ~120s
      win = find_window()
      if win ~= nil then break end
      hs.timer.usleep(500000)
    end
    if win == nil then
      io.stderr:write('error: window not detected: ' .. expected .. '\n')
      os.exit(1)
    end

    win:focus()

    -- Resize and center on screen.
    local screen = win:screen() or hs.screen.mainScreen()
    local sf = screen:frame()
    local fx = math.floor(sf.x + (sf.w - w) / 2)
    local fy = math.floor(sf.y + (sf.h - h) / 2)
    win:setFrame(hs.geometry.rect(fx, fy, w, h))

    hs.timer.usleep(800000) -- let one frame render after resize

    -- Snapshot just the window region.
    local frame = win:frame()
    local img = screen:snapshot(frame)
    if img == nil then
      io.stderr:write('error: snapshot failed\n')
      os.exit(1)
    end

    if not img:saveToFile(out_png) then
      io.stderr:write('error: failed to save png: ' .. out_png .. '\n')
      os.exit(1)
    end

    local sz = img:size()
    local iw = sz.w or 0
    local ih = sz.h or 0

    -- Choose a sampling rect that avoids the sidebar/tabs.
    local narrow = (iw < 900)
    local x0 = narrow and math.floor(iw * 0.08) or math.floor(iw * 0.35)
    local x1 = math.floor(iw * 0.96)
    local y0 = math.floor(ih * 0.22)
    local y1 = math.floor(ih * 0.92)

    local function is_blueish(c)
      if c == nil then return false end
      local r = c.red or 0
      local g = c.green or 0
      local b = c.blue or 0
      local a = c.alpha or 1
      if a < 0.8 then return false end
      local mx = math.max(r, g)
      return (b > 0.35) and ((b - mx) > 0.12) and (g > 0.20)
    end

    local function is_warm(c)
      if c == nil then return false end
      local r = c.red or 0
      local g = c.green or 0
      local b = c.blue or 0
      local a = c.alpha or 1
      if a < 0.8 then return false end
      return (r > 0.55) and (g > 0.25) and (b < 0.55)
    end

    local step_x = math.max(4, math.floor((x1 - x0) / 120))
    local step_y = math.max(4, math.floor((y1 - y0) / 80))

    local blue = 0
    local warm = 0
    local total = 0
    for y = y0, y1, step_y do
      for x = x0, x1, step_x do
        total = total + 1
        local c = img:colorAt(hs.geometry.point(x, y))
        if is_blueish(c) then blue = blue + 1 end
        if is_warm(c) then warm = warm + 1 end
      end
    end

    if case_name == 'line' or case_name == 'multi' then
      if blue < 6 then
        io.stderr:write(string.format('error: expected blue-ish line pixels, got=%d (total=%d) png=%s\n', blue, total, out_png))
        os.exit(1)
      end
    elseif case_name == 'heatmap' then
      if warm < 10 then
        io.stderr:write(string.format('error: expected warm heatmap pixels, got=%d (total=%d) png=%s\n', warm, total, out_png))
        os.exit(1)
      end
    end

    print(string.format('ok: case=%s size=%dx%d blue=%d warm=%d png=%s', case_name, w, h, blue, warm, out_png))
  "

  cleanup
  trap - EXIT
}

run_case "line" 0 "1200x840"
run_case "line" 0 "640x520"
run_case "multi" 1 "1200x840"
run_case "heatmap" 7 "1200x840"
run_case "heatmap" 7 "640x520"

echo "ok: charts gallery e2e finished"
