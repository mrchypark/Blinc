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

## Formatting Gate (IMPORTANT)

- Before commit/push/PR, ALWAYS run formatting:
  - `cargo fmt --all`
  - `cargo fmt --all -- --check`
- Do not skip formatting even for small or single-file changes.

## Remote Safety (Optional)

To reduce accidental pushes to upstream, you may disable the upstream push URL locally:

```sh
git remote set-url --push upstream DISABLED
```

## Rust Skills Routing (Codex Local)

- Rust 관련 질의/구현/리뷰에서는 먼저 `rust-router`를 진입점으로 사용한다.
  - 위치: `~/.codex/skills/rust-router/SKILL.md`
- 아래 패턴은 직접 라우팅 가능하다:
  - ownership/borrow/lifetime/E0382/E0597 -> `m01-ownership`
  - mutability/E0499/E0502/E0596 -> `m03-mutability`
  - error handling/anyhow/thiserror -> `m06-error-handling`
  - async/concurrency/Send/Sync/tokio -> `m07-concurrency`
  - style/naming/clippy/rustfmt -> `coding-guidelines`
  - unsafe/FFI/raw pointer/transmute -> `unsafe-checker` (최우선)
- 도메인 키워드가 있으면 L1+L3를 같이 로드한다:
  - web/http/axum -> `m07-concurrency` + `domain-web`
  - trading/payment/fintech -> `m01-ownership` + `domain-fintech`
  - cli/clap -> `m07-concurrency` + `domain-cli`
  - kubernetes/grpc/microservice -> `m07-concurrency` + `domain-cloud-native`
  - embedded/no_std/mcu -> `m02-resource` + `domain-embedded`
- 최신 버전/릴리즈/크레이트 정보 질의는 `rust-learner`를 우선 사용한다.

### Project Compatibility Rule

- 이 저장소에서는 현재 `Cargo.toml` 기준 설정(`edition = 2021`, `rust-version = 1.75`)을 기본으로 유지한다.
- 사용자가 명시적으로 요청하지 않으면 `edition = 2024` 또는 Rust MSRV 상향을 자동 적용하지 않는다.
