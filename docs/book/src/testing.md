# Testing Guide

## Baseline Commands

```bash
cargo check --manifest-path Cargo.toml
cargo test --manifest-path Cargo.toml -- --nocapture
```

## UI Regression Script

```bash
./scripts/ui_regression.sh
```

This runs compile checks, no-run test build, targeted UI regression tests, and `cargo nextest run` when `cargo-nextest` is available.

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
