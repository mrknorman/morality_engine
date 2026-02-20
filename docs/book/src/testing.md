# Testing Guide

## Baseline Commands

```bash
cargo check --manifest-path Cargo.toml
cargo test --manifest-path Cargo.toml -- --nocapture
```

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
