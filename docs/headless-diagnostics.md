# Headless Diagnostics Workflow

This workflow is for UI development tooling where you want to:

- define explicit goals/assertions,
- run deterministic scenarios without opening a native window,
- capture machine-readable failure reports.

## 1) Define scenario

Example `scenario.json`:

```json
{
  "steps": [
    { "type": "tick", "frames": 1 },
    { "type": "assert_exists", "id": "app.title" },
    { "type": "assert_text_contains", "id": "app.title", "value": "Welcome" }
  ]
}
```

## 2) Provide snapshot probe

Your app provides a probe closure `FnMut(&ProbeContext) -> DiagnosticsSnapshot` from app-observable state.
`ProbeContext` contains `elapsed_ms`, `elapsed_frames`, and `step_index`.

## 3) Execute runner

Call:

- `run_loaded_scenario_with_probe(...)`

It returns `RunOutcome::Passed` or `RunOutcome::Failed`.

## 4) Emit report

Use `outcome.report()` and write JSON to stdout or file via `HeadlessReport` helpers.

## Exit behavior recommendation

- exit `0` on pass,
- exit non-zero on failure,
- persist JSON report as CI artifact.
