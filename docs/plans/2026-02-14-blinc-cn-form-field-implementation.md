# blinc_cn Form/Field Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add missing `Form` and `Field` components to `blinc_cn` so form composition follows the component plan and is available from `cn` and `prelude` exports.

**Architecture:** Implement two themed wrapper components on top of `blinc_layout::Div` and existing `cn::label` patterns. Keep the API composable (`.child(...)`, `.error(...)`, `.description(...)`) and avoid introducing a full validation framework in this pass; instead provide field-level message slots and form-level layout structure.

**Tech Stack:** Rust 1.75, edition 2021, `blinc_layout`, `blinc_theme`, `blinc_cn` component builder pattern (`OnceCell` lazy builders).

---

### Task 1: Define acceptance tests first (TDD RED)

**Files:**
- Create: `crates/blinc_cn/src/components/form.rs`
- Create: `crates/blinc_cn/src/components/field.rs`
- Modify: `crates/blinc_cn/src/components/mod.rs`
- Modify: `crates/blinc_cn/src/lib.rs`

**Step 1: Write failing tests for `Field` config behavior**

```rust
#[test]
fn test_field_builder_sets_required_and_error() {
    let field = field("Email").required().error("Required");
    assert!(field.config.required);
    assert_eq!(field.config.error.as_deref(), Some("Required"));
}
```

**Step 2: Write failing tests for `Form` layout config behavior**

```rust
#[test]
fn test_form_builder_sets_spacing_and_full_width() {
    let form = form().spacing(20.0).w_full();
    assert_eq!(form.config.spacing, Some(20.0));
    assert!(form.config.full_width);
}
```

**Step 3: Run targeted tests to verify RED**

Run: `cargo test -p blinc_cn field::tests:: test_form` (or equivalent targeted module tests)
Expected: FAIL because `form`/`field` modules and APIs do not exist yet.

**Step 4: Commit checkpoint (optional local checkpoint, no push required)**

```bash
git add crates/blinc_cn/src/components/{form.rs,field.rs} crates/blinc_cn/src/components/mod.rs crates/blinc_cn/src/lib.rs
git commit -m "test(blinc_cn): add failing tests for form and field components"
```

### Task 2: Implement minimal `Field` component (TDD GREEN)

**Files:**
- Modify: `crates/blinc_cn/src/components/field.rs`

**Step 1: Implement `FieldConfig`, `Field`, `FieldBuilder`, `field()`**

```rust
pub fn field(label: impl Into<String>) -> FieldBuilder { ... }
```

Required capabilities:
- Label text + `required` marker using `cn::label`
- Optional `description` (secondary text)
- Optional `error` (error text, higher priority than description)
- Optional `disabled` label style
- Child slot for any `ElementBuilder` input control

**Step 2: Keep implementation minimal and composable**
- Use `inner: Div` wrapper
- Use `OnceCell<Field>` in builder for lazy build
- No new validation engine in this task

**Step 3: Run module tests to verify GREEN**

Run: `cargo test -p blinc_cn field::tests::`
Expected: PASS

**Step 4: Refactor pass (if needed)**
- Remove duplicate label/message rendering code
- Keep public API small and aligned with existing components

### Task 3: Implement minimal `Form` component (TDD GREEN)

**Files:**
- Modify: `crates/blinc_cn/src/components/form.rs`

**Step 1: Implement `FormConfig`, `Form`, `FormBuilder`, `form()`**

Required capabilities:
- Vertical layout container for fields
- Configurable spacing between children
- Width helpers (`w`, `w_full`, `max_w`)
- Disabled visual state via opacity (container-level)

**Step 2: Add child composition APIs**

```rust
pub fn child(mut self, child: impl ElementBuilder + 'static) -> Self { ... }
pub fn children<I>(mut self, items: I) -> Self where I: IntoIterator<Item = Box<dyn ElementBuilder>> { ... }
```

(If generic `children` is awkward with trait objects, keep only `child` for YAGNI.)

**Step 3: Run module tests**

Run: `cargo test -p blinc_cn form::tests::`
Expected: PASS

### Task 4: Wire exports and integration coverage

**Files:**
- Modify: `crates/blinc_cn/src/components/mod.rs`
- Modify: `crates/blinc_cn/src/lib.rs`
- (Optional) Modify: `crates/blinc_app/examples/cn_demo.rs`

**Step 1: Add module declarations and re-exports**
- `pub mod form;`
- `pub mod field;`
- `pub use form::{form, Form, FormBuilder};`
- `pub use field::{field, Field, FieldBuilder};`
- Export in `cn` and `prelude` modules.

**Step 2: Add one compile-time usage smoke test**

```rust
let _ui = form().child(field("Email").required().child(cn::input(&email_data)));
```

**Step 3: Run package tests**

Run: `cargo test -p blinc_cn`
Expected: PASS

### Task 5: Documentation and compatibility checks

**Files:**
- Modify: `docs/book/src/cn/form.md`
- Modify: `crates/blinc_cn/README.md`

**Step 1: Update docs to reflect real API signatures**
- Replace stale `input()` usage examples with `input(&data)` where applicable.
- Add `form()` + `field()` examples.

**Step 2: Ensure docs don’t claim unimplemented features**
- Keep scope to layout + field-level messages.

**Step 3: Run docs-adjacent compile checks if available**
Run: `cargo test -p blinc_cn`
Expected: PASS (doctests are mostly `ignore`, but package stays healthy)

### Task 6: Quality gates and review loop

**Files:**
- Modify: all touched files above

**Step 1: Format gate (mandatory before completion claims)**
Run:
- `cargo fmt --all`
- `cargo fmt --all -- --check`

Expected: no diff and check pass.

**Step 2: Lint/test verification**
Run:
- `cargo test -p blinc_cn`

Expected: PASS with no new failures.

**Step 3: Self-review checklist**
- API consistency with existing `blinc_cn` builders
- No hardcoded design values where theme tokens exist
- Error vs description precedence documented and tested
- Export surface complete (`components`, `cn`, `prelude`)

**Step 4: Gemini code review loop**
- Request Gemini review on changed files.
- Apply fixes.
- Re-run fmt/tests.
- Repeat until Gemini reports no actionable review items.

---

## Required Skills During Execution
- `superpowers:test-driven-development`
- `superpowers:verification-before-completion`
- `superpowers:requesting-code-review`

## Non-Goals (This iteration)
- Full schema-driven validation framework
- Async form submission state machine
- Auto-focus/submit keyboard handling abstraction
