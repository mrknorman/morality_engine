# Testing Guide

## Baseline Commands

```bash
cargo check --manifest-path Cargo.toml
cargo test --manifest-path Cargo.toml -- --nocapture
```

## UI Query-Safety Preflight

```bash
./scripts/ui_query_safety.sh
```

Runs targeted system-initialization smoke tests that catch B0001-style query
alias conflicts before full regression runs.

## UI Regression Script

```bash
./scripts/ui_regression.sh
```

This runs compile checks, no-run test build, targeted UI regression tests, and `cargo nextest run` when `cargo-nextest` is available.

Optional GPU smoke lane:

```bash
UI_RUN_GPU_SMOKE=1 ./scripts/ui_regression.sh
```

or directly:

```bash
./scripts/ui_gpu_smoke.sh
```

The GPU smoke tests are opt-in and intentionally skip on environments without a
usable GPU/display backend.

## Nextest (if installed)

```bash
cargo nextest run
```

Project config lives at `.config/nextest.toml`.

## Refactor Validation Flow

1. Run unit tests after each stage.
2. Re-run full tests after interaction or layer changes.
3. For UI behavior changes, run manual mixed-input checks:
   - keyboard navigation
   - mouse hover/click
   - dropdown/modal layering
   - pause/menu owner-scoped gating
4. Before final sign-off, run the manual checklist:
   - `docs/ui_manual_validation_checklist.md`
