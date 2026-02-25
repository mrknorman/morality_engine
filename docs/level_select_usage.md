# Level Select Usage

Last updated: 2026-02-25

This document covers the current level-select behavior, hierarchy, and integration points.

## Catalog Structure

Top-level folders:

- `dilemmas`
- `chat_logs`

File conventions in the UI:

- dilemma files: `*.dilem`
- dialogue files: `*.log`

Notes:

- Dialogue source names are stored without suffix and rendered as `name.log` in the UI.
- Folder expansion state is collapsed by default and persisted while the overlay remains open.

## Interaction Model

- Open from main menu overlay command flow.
- Search input is focused by default.
- Typing while overlay is open re-focuses search and updates results incrementally.
- Search projection includes ancestor folders for matching files.
- Empty search restores normal tree projection using current expansion state.
- Folder activation toggles expansion.
- File activation launches scene actions.

Keyboard behavior:

- `ArrowUp` / `ArrowDown`: move selection.
- `Enter` / `ArrowRight`: activate selected row.
- Search supports standard text editing keys (`Backspace`, `Delete`, alphanumerics).

## Unlock Policy

- `debug_assertions`: all entries are unlocked.
- release/profile without `debug_assertions`:
  - unlocked set is session-scoped from campaign progress
  - only reached dilemma/dialogue scenes are launchable

Campaign progress tracking:

- `track_campaign_reached_dilemmas` records reached `Scene::Dilemma` and `Scene::Dialogue` entries while in `SceneFlowMode::Campaign`.

## Dilemma Launch Modes

Dilemma entry activation path:

- dev (`debug_assertions`): opens launch modal.
- release: launches play-once flow directly.

Dev launch modal options:

- `CONTINUE [c]`: reset stats, configure campaign from selected dilemma, route via `SceneNavigator`.
- `PLAY ONCE [p]`: reset stats, configure single-level queue, route via `SceneNavigator`.
- `CANCEL [esc]`: close modal only, no queue or state mutation.

## Architecture Map

Key files:

- catalog model: `src/systems/ui/menu/level_select_catalog.rs`
- overlay/runtime/rendering: `src/systems/ui/menu/level_select.rs`
- command dispatch: `src/systems/ui/menu/command_flow.rs`
- command reducer/effects: `src/systems/ui/menu/command_reducer.rs`, `src/systems/ui/menu/command_effects.rs`
- scene queue runtime: `src/scenes/mod.rs`

Primitive dependencies used (no menu-domain logic inside primitives):

- `TextInputBox`
- `SearchBox`
- `UiWindow` + `UiWindowContent`
- `ScrollableRoot`/window scroll handling
- `SelectableMenu`/`Clickable`

## Validation Commands

- `cargo check`
- `cargo test level_select -- --nocapture`
- `cargo test configure_campaign_from_dilemma_sets_campaign_mode_and_queue -- --nocapture`
