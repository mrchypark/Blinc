# Blinc Agent Instructions

## Upstream / PR Rules (IMPORTANT)

- Upstream canonical repository: [project-blinc/Blinc](https://github.com/project-blinc/Blinc)
- Fork (your repo / default target): [mrchypark/Blinc](https://github.com/mrchypark/Blinc)

### Hard rule

- NEVER open a Pull Request targeting `project-blinc/Blinc` unless the user explicitly asks to contribute upstream (for example: "upstream에 PR 올려줘", "project-blinc에 PR 보내줘").
- When the user asks to "reflect upstream changes", that means:
  - fetch from upstream
  - integrate locally (merge/rebase)
  - push to the fork (`mrchypark/Blinc`)
  - open a PR with base repo = `mrchypark/Blinc`

### PR UI guardrail (GitHub)

- Before creating a PR, explicitly verify:
  - Base repository is `mrchypark/Blinc`
  - Base branch is `main` (unless the user says otherwise)
  - Head repository/branch is the feature branch in `mrchypark/Blinc`

## Remote Safety (Optional)

To reduce accidental pushes to upstream, you may disable the upstream push URL locally:

```sh
git remote set-url --push upstream DISABLED
```

