# Context Snapshot: Agent Debugging Platform

- Timestamp: `20260307T201138`
- Context type: `brownfield`
- Interview profile: `standard`
- Repository: `Blinc`

## Current Codebase Facts

- `blinc_recorder` already captures runtime events and tree snapshots, supports replay, and exposes a local debug server.
- `blinc_debugger` already loads recording exports and shows a basic tree/preview/inspector/timeline UI.
- Existing debugger behavior is viewer-oriented, not command/trace-forensics-oriented.
- Existing recorder behavior captures events, but automatic frame snapshot capture is not fully wired across the runtime.
- There is an earlier app-level headless diagnostics plan in `docs/plans/2026-02-14-app-headless-diagnostics.md`.

## Product Need

Blinc needs a complete developer tool that lets an agent:

- launch and drive applications directly
- inject commands and input in a stable way
- capture command, event, state tree, render evidence, and assertion output
- inspect failures later without rerunning the app

## External Reference

- `paiml/probar` was reviewed as a product reference for:
  - library + CLI split
  - headless execution
  - deterministic replay
  - playbook validation
  - Playwright-like ergonomics

## Confirmed Scope

- Desktop automation is required.
- Headless execution is required in MVP.
- Mobile is not required in MVP.
- Rust-native API is the primary automation surface.
- Scenario DSL and state-machine playbooks are both required.
- Element addressing must support both stable IDs and semantic locators.

