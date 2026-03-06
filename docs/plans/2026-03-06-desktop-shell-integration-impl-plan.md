# Desktop Shell Integration Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add desktop-native app shell integrations including local/global shortcuts, menu models, tray support, notifications, and deep-link entrypoints.

**Architecture:** Start with shared shell contracts in `blinc_platform`, add an application-scope event sink in `blinc_platform_desktop` that queues `blinc_platform::Event` values so tray/global shortcut/deep-link events survive without a live window, implement desktop-native adapters on top of that sink, and then surface only the minimum runtime hooks needed by `blinc_app`.

**Tech Stack:** Rust 2021, `winit`, menu/tray/shortcut helper crates, integration tests.

---

### Task 1: Add Red Tests For Shortcut And Menu Contracts

**Files:**
- Create: `crates/blinc_platform/tests/shell_api.rs`
- Modify: `crates/blinc_platform/src/lib.rs`

**Step 1: Write the failing tests**

- Add a test for `Shortcut` parsing plus normalized modifier matching.
- Add a test for a tree-shaped `Menu` / `MenuItem` model with stable command IDs.
- Add a test for notification and deep-link request value objects.

**Step 2: Run targeted tests to verify failure**

Run:
- `cargo test -p blinc_platform --test shell_api -- --nocapture`

Expected: compile failures because shell modules are not yet defined.

**Step 3: Commit the red tests**

```bash
git add crates/blinc_platform/tests/shell_api.rs crates/blinc_platform/src/lib.rs
git commit -m "test: add desktop shell integration contracts"
```

### Task 2: Add Shared Shell Modules

**Files:**
- Create: `crates/blinc_platform/src/shortcuts.rs`
- Create: `crates/blinc_platform/src/menu.rs`
- Create: `crates/blinc_platform/src/tray.rs`
- Create: `crates/blinc_platform/src/notifications.rs`
- Create: `crates/blinc_platform/src/app_links.rs`
- Modify: `crates/blinc_platform/src/event.rs`
- Modify: `crates/blinc_platform/src/lib.rs`

**Step 1: Implement shared value types**

- Add `Shortcut`, `ShortcutScope`, `ShortcutAction`, `Menu`, `MenuItem`, `TrayMenu`, `NotificationRequest`, and `AppLinkEvent`.

**Step 2: Extend platform events**

- Add event variants for menu command dispatch, tray activation, shortcut activation, and app-link open requests.

**Step 3: Re-run shared API tests**

Run:
- `cargo test -p blinc_platform --test shell_api -- --nocapture`

Expected: PASS.

**Step 4: Commit the shared shell abstractions**

```bash
git add crates/blinc_platform/src/shortcuts.rs crates/blinc_platform/src/menu.rs crates/blinc_platform/src/tray.rs crates/blinc_platform/src/notifications.rs crates/blinc_platform/src/app_links.rs crates/blinc_platform/src/event.rs crates/blinc_platform/src/lib.rs crates/blinc_platform/tests/shell_api.rs
git commit -m "feat: add shared desktop shell abstractions"
```

### Task 3: Implement Desktop Shortcuts, Menus, Tray, And Notifications

**Files:**
- Create: `extensions/blinc_platform_desktop/src/app_event_sink.rs`
- Create: `extensions/blinc_platform_desktop/src/shortcuts.rs`
- Create: `extensions/blinc_platform_desktop/src/menu.rs`
- Create: `extensions/blinc_platform_desktop/src/tray.rs`
- Create: `extensions/blinc_platform_desktop/src/notifications.rs`
- Create: `extensions/blinc_platform_desktop/src/app_links.rs`
- Modify: `extensions/blinc_platform_desktop/src/event_loop.rs`
- Modify: `extensions/blinc_platform_desktop/src/lib.rs`
- Modify: `extensions/blinc_platform_desktop/Cargo.toml`
- Create: `extensions/blinc_platform_desktop/tests/shell_runtime.rs`
- Modify: `extensions/blinc_platform_desktop/tests/support.rs`

**Step 1: Write the failing backend tests**

- Add a test for accelerator registration and dispatch into shared shortcut events.
- Add a test for menu command IDs routing back into the event loop.
- Add a test for tray click activation and notification request submission through a backend stub.

**Step 2: Run targeted backend tests to verify failure**

Run:
- Linux/CI: `xvfb-run --auto-servernum cargo test -p blinc_platform_desktop --test shell_runtime -- --nocapture`
- macOS/Windows/local GUI session: `cargo test -p blinc_platform_desktop --test shell_runtime -- --nocapture`

Expected: failures because none of the desktop shell modules exist.

**Step 3: Introduce an app-scope event sink**

- Create `app_event_sink.rs` to queue `blinc_platform::Event` values before a window exists and after it closes.
- Keep the sink internal to `blinc_platform_desktop` and store shared `blinc_platform::Event` values so no reverse dependency from `blinc_platform_desktop` back into `blinc_app` is introduced.
- Make `event_loop.rs` drain that queue into the existing handler path when a window becomes available.

**Step 4: Implement backend adapters**

- Register local shortcuts per window and global shortcuts behind an opt-in scope.
- Build native menus and tray items from the shared menu model.
- Map notification requests into native desktop notification APIs.
- Convert custom URL / deep-link callbacks into `AppLinkEvent`.

**Step 5: Re-run backend and shared tests**

Run:
- Linux/CI: `xvfb-run --auto-servernum cargo test -p blinc_platform_desktop --test shell_runtime -- --nocapture`
- macOS/Windows/local GUI session: `cargo test -p blinc_platform_desktop --test shell_runtime -- --nocapture`
- `cargo test -p blinc_platform --test shell_api -- --nocapture`

Expected: PASS.

**Step 6: Commit the desktop shell backend**

```bash
git add extensions/blinc_platform_desktop/src/app_event_sink.rs extensions/blinc_platform_desktop/src/shortcuts.rs extensions/blinc_platform_desktop/src/menu.rs extensions/blinc_platform_desktop/src/tray.rs extensions/blinc_platform_desktop/src/notifications.rs extensions/blinc_platform_desktop/src/app_links.rs extensions/blinc_platform_desktop/src/event_loop.rs extensions/blinc_platform_desktop/src/lib.rs extensions/blinc_platform_desktop/tests/support.rs extensions/blinc_platform_desktop/tests/shell_runtime.rs extensions/blinc_platform_desktop/Cargo.toml
git commit -m "feat: add desktop shell integrations"
```

### Task 4: Add A Minimal App Runtime Hook

**Files:**
- Modify: `crates/blinc_app/src/windowed.rs`
- Modify: `crates/blinc_app/src/app.rs`
- Create: `crates/blinc_app/src/tests_shell.rs`
- Modify: `crates/blinc_app/src/lib.rs`

**Step 1: Write the failing runtime test**

- Add `#[cfg(test)] mod tests_shell;` to `crates/blinc_app/src/lib.rs`, guarded the same way as the `windowed` module so the test harness compiles only when `--features windowed` is enabled.
- Add a test for wiring shell-originated events into the existing app event pump without panicking.

**Step 2: Run the targeted runtime test to verify failure**

Run:
- `cargo test -p blinc_app --features windowed tests_shell -- --nocapture`

Expected: the app runtime has no knowledge of menu/shortcut/app-link event variants.

**Step 3: Implement the minimal runtime hook**

- Update event dispatch so the new shell events can be observed by applications.
- Do not add app-level sugar APIs yet; only preserve the events.

**Step 4: Re-run the runtime test**

Run:
- `cargo test -p blinc_app --features windowed tests_shell -- --nocapture`

Expected: PASS.

**Step 5: Commit the runtime hook**

```bash
git add crates/blinc_app/src/windowed.rs crates/blinc_app/src/app.rs crates/blinc_app/src/tests_shell.rs crates/blinc_app/src/lib.rs
git commit -m "feat: route desktop shell events into app runtime"
```
