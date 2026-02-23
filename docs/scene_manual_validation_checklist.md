# Scene Manual Validation Checklist

Last updated: 2026-02-23

Use this checklist after scene-flow refactors and before release.

## Preconditions

- `cargo check` passes.
- Scene unit tests pass:
  - `cargo test scenes::runtime::tests`
  - `cargo test scenes::flow::tests`
  - `cargo test scenes::tests`

## Latest Validation Snapshot (2026-02-23)

- `cargo check`: pass
- `cargo test scenes::runtime::tests`: pass
- `cargo test scenes::flow::tests`: pass
- `cargo test scenes::tests`: pass
- Interactive in-window scenario sweep: not executed in this terminal-only validation pass

## Core Campaign Flow

- Launch from menu and start normal campaign flow.
- Confirm scene order progresses correctly:
  - `Menu -> Loading -> Dialogue -> Dilemma`
- Confirm dialogue completion advances to the expected next scene.
- Confirm final branch can reach ending without panic.

## Single-Level Flow

- From level-select/debug flow, start single-level mode.
- Confirm queue is `Dilemma(selected) -> Menu`.
- Confirm finishing the level returns to menu and does not continue campaign.

## Fault-Tolerance Paths

- Trigger next-scene action on an empty queue.
- Confirm transition falls back to menu (no panic).
- Force a scene-kind mismatch (for example route to dialogue state with non-dialogue scene).
- Confirm fallback route to menu is applied safely.
- Confirm invalid ending parse routes to menu fallback.

## Ownership / Query Safety

- Re-enter loading scene repeatedly.
- Confirm loading button text updates without singleton panic.
- During dialogue, rapidly advance/skip and confirm no entity-command panic.
- Run dilemma results with sparse/no stats and confirm no panic.

## Transition API Consistency

- Validate both trigger points use canonical scene transition mapping:
  - interaction `NextScene`
  - menu `NextScene` command
- Confirm both paths produce identical target states for the same queue head.

## Reset Behavior

- Trigger reset from in-game.
- Confirm scene queue resets through `SceneQueue::reset_campaign()`.
- Confirm stats reset and flow returns to menu/loading start route.

## Stability Sweep

- Rapidly mix keyboard + mouse input while transitioning between scenes.
- Confirm no panic in scene setup/teardown paths.
- If any panic occurs, capture `RUST_BACKTRACE=1` stack and active scene/state.
