# UI ECS Reference

This document describes the reusable UI interaction and menu architecture in this project, with an emphasis on composability and avoiding duplicate logic.

## Scope

Covered systems and modules:

- `src/systems/interaction/mod.rs`
- `src/systems/ui/dropdown.rs`
- `src/systems/ui/layer.rs`
- `src/systems/ui/selector.rs`
- `src/systems/ui/tabs.rs`
- `src/entities/text/mod.rs`
- `src/startup/system_menu.rs`
- `src/startup/menus/mod.rs`
- `src/startup/menus/stack.rs`
- `src/entities/sprites/window.rs`

## Design Goals

- UI behavior should be data-driven by components/resources, not hardcoded per menu.
- Keyboard/mouse behavior should be consistent across menus and submenus.
- Menu layering (root menu, dropdown, modal) should be explicit and conflict-safe.
- New UI elements should reuse shared systems (`Clickable`, `SelectableMenu`, `OptionCycler`, etc.) instead of adding one-off handlers.

## Core Interaction Primitives

### `InteractionGate` and `InteractionCapture`

- `InteractionGate` controls whether an entity can be interacted with in the current context.
- `InteractionCapture` marks contexts where gameplay interaction should be suppressed by menu interaction.
- Use `interaction_context_active(...)` + `interaction_gate_allows(...)` to gate every interaction system.

Pattern:

- Gameplay entities: `InteractionGate::GameplayOnly`
- Menu entities opened over gameplay: `InteractionGate::PauseMenuOnly`

### `Clickable<T>`

Component for pointer-triggered actions.

Fields:

- `actions: Vec<T>`: action keys mapped in `ActionPallet`.
- `region: Option<Vec2>`: optional explicit hit area.
- `triggered: bool`: one-frame latch set by interaction systems.

Use when:

- Entity should respond to mouse click.
- Entity is also selectable and you want keyboard activation to funnel into the same click path.

### `Pressable<T>`

Component for keyboard-triggered actions through key mappings.

Fields:

- `mappings: Vec<KeyMapping<T>>`
- `triggered_mapping: Option<usize>`

Use when:

- The action does not need pointer hit-testing and should map directly to keys.

### `SelectableMenu` + `Selectable`

Provides menu-style selection and activation.

- `SelectableMenu` is a menu root/controller.
- `Selectable` marks selectable children with `(menu_entity, index)`.

`SelectableMenu` key fields:

- `selected_index`
- `up_keys`
- `down_keys`
- `activate_keys`
- `wrap`
- `click_activation: SelectableClickActivation`

Click activation modes:

- `SelectedOnAnyClick`: any click activates the currently selected item.
- `HoveredOnly`: only clicked hovered item activates.

Use `HoveredOnly` for dropdowns (avoids accidental forced activation), and use `SelectedOnAnyClick` for full-screen modal/pause-style menus where forced click is desired.

### `OptionCycler`

Adds left/right cycling behavior to a selected menu option.

Fields:

- `left_triggered`, `right_triggered`
- `at_min`, `at_max`

Flow:

1. `option_cycler_input_system` sets left/right triggers from keyboard for selected row.
2. Menu logic consumes those triggers and applies domain-specific changes.
3. Bounds (`at_min`/`at_max`) drive arrow visibility and clamping.

## Visual State Pipeline

### `InteractionVisualState`

Single-frame visual intent:

- `hovered`
- `pressed`
- `selected`
- `keyboard_locked`

Reset each frame by `reset_interaction_visual_state`, then rebuilt by interaction systems.

### `InteractionVisualPalette`

Color palette applied by `apply_interaction_visuals`.

Use this instead of manual per-frame color writes when possible.

## System Menu Building Blocks (`system_menu.rs`)

### Root and Chrome

- `spawn_root(...)`: spawns a root menu with `SelectableMenu` and switch audio pallet.
- `spawn_chrome_with_marker(...)`: panel, border, title, hint.
- `play_navigation_switch(...)`: shared keyboard-nav sound utility used by both main-scene and startup menu systems.

### Option Bundle

- `SystemMenuOptionBundle::new_at(...)` creates text option + `Selectable` + visual palette state.

### Indicators and Bars

- Unified style config: `SystemMenuOptionVisualStyle`.
  - Selection arrows: `selection_indicator` (`SystemMenuSelectionIndicatorStyle`).
  - Selection bar: `selection_bar` (`SystemMenuSelectionBarStyle`).
  - Left/right cycle arrows: `cycle_arrows` (`SystemMenuCycleArrowStyle`).
- Rendering/update systems:
  - `ensure_selection_indicators` + `update_selection_indicators`
  - `ensure_selection_bars` + `update_selection_bars`
  - `ensure_cycle_arrows` + `update_cycle_arrows`

Rule:

- Prefer `SystemMenuOptionVisualStyle` + these systems over custom indicator rendering so interaction look/feel stays consistent.

## Menu Stack Architecture (`startup/menus`)

### Core Types

- `MenuRoot { host, gate }`: root identity + interaction gate.
- `MenuStack`: page stack + remembered selected index per frame.
- `MenuPage`, `MenuCommand`, `MenuOptionCommand`: declarative navigation/action model.
- `MenuPageContent`: marker for spawned page children to support clean rebuild/despawn.

### Page Composition Module

Menu page content spawn/rebuild now lives in `src/startup/menus/page_content.rs`:

- `spawn_page_content(...)`
- `rebuild_menu_page(...)`

Rule:

- Keep page composition (tables, option rows, dropdown children) isolated from runtime
  command/input systems so new layout changes do not increase command-flow complexity.

### Navigation State Resource (Menu-specific)

`MenuNavigationState` (`stack.rs`) stores cross-system transient intent:

- `exit_prompt_target_menu`
- `exit_prompt_closes_menu_system`
- `pending_exit_menu`
- `pending_exit_closes_menu_system`

Purpose:

- Keep exit/unsaved-confirm transitions out of `VideoSettingsState`.
- Make stack/modal flow reusable for non-video menus.

### Command Dispatch Pattern

`handle_menu_option_commands` now follows a reducer + effects model:

- Pure reduction:
  - lives in `src/startup/menus/command_reducer.rs`
  - `reduce_push_menu_command(...)`
  - `reduce_pop_menu_command(...)`
  - `reduce_toggle_resolution_dropdown_command(...)`
  - `reduce_toggle_display_mode_command(...)`
  - `reduce_toggle_vsync_command(...)`
  - `reduce_reset_video_defaults_command(...)`
  - `reduce_menu_command(...)` dispatches `MenuCommand` to reducers.
- Effect application:
  - reducers return `MenuReducerResult` (flags + payloads only).
  - lives in `src/startup/menus/command_effects.rs`
  - `apply_menu_reducer_result(...)` applies world side effects:
    - dropdown open/close
    - modal spawn
    - state transitions
    - window/render apply
    - menu close/rebuild bookkeeping
  - specialized effects remain isolated:
    - `handle_apply_video_settings_command(...)`
    - `handle_exit_application_command(...)`

Guideline:

- Keep transition logic pure and unit-testable.
- Keep Bevy/world side effects centralized and explicit.

### Video Option Schema Registry

`src/startup/menus/defs.rs` now exposes a single schema primitive:

- `VideoTopOptionKey` (DisplayMode, Resolution, Vsync, etc.)
- `video_top_option_key(tab, row)` / `video_top_option_keys(tab)`

All top-row video option behavior now routes through this registry:

- labels (`label()`, `video_top_option_labels(...)`)
- choice count (`choice_count()`, `video_top_option_choice_count(...)`)
- dropdown/selector values (`values()`, `video_top_option_values(...)`)
- snapshot read/write (`selected_index()`, `apply_selected_index()`)
- cycling (`cycle()`, `cycle_video_top_option(...)`)
- table value rendering (`value_text()`, `video_top_value_strings(...)`)

Extension rule:

- Add a new top-row video option by adding one `VideoTopOptionKey` variant and updating its methods,
  then wire it into `video_top_option_keys(...)`.

### Dropdown State Resource (Shared UI)

`DropdownLayerState` (`src/systems/ui/dropdown.rs`):

- Owner-scoped open parent map (`owner -> open parent`).
- Owner-scoped one-frame suppression latch for reopen protection.

Generic helpers (component-parametric):

- `any_open<D>()`
- `open_for_parent<D>()`
- `close_all<D>()`
- `close_for_parent<D>()`
- `close_for_owner<D>()`
- `enforce_single_visible_layer<D, R>()`

These helpers are reusable for any dropdown component type `D` and any root-owner marker `R` (menus, window panels, etc.).

### UI Layer Model (Shared UI)

`src/systems/ui/layer.rs` provides reusable layered navigation state:

- `UiLayerKind`: `Base`, `Dropdown`, `Modal` (with explicit priority).
- `UiLayer { owner, kind }`: attached to each layer root entity.
- `active_layers_by_owner_scoped(...)`: resolves the active layer per owner using:
  - `Visibility`
  - owner-scoped interaction capture + `InteractionGate`
  - kind priority (`Modal > Dropdown > Base`)

Usage rule:

- Attach `UiLayer` to every menu-layer root you spawn, then use `active_layers_by_owner_scoped(...)` in shortcut/command systems instead of custom modal/dropdown-open checks.

### Selector/Shortcut Utilities (Shared UI)

`src/systems/ui/selector.rs`:

- `ShortcutKey(KeyCode)` component
- `collect_shortcut_commands(...)` for active parent entities
- `sync_option_cycler_bounds(...)`

Use `ShortcutKey` on selectable options that should be triggerable by direct keybind.

### Tab Utilities (Shared UI)

`src/systems/ui/tabs.rs`:

- `TabBar { owner }`: binds a reusable tab selector to a UI owner (menu/window root).
- `TabItem { index }`: marks selectable tab options within the tab bar.
- `TabActivationPolicy { activate_keys }`: per-tab-root keyboard activation bindings.
- `TabBarState { active_index }`: stable selected tab index for rendering/behavior.
- `TabChanged` message: emitted when active tab changes.
- `collect_clicked_tab_indices(...)`: shared helper for tab-click target extraction used by tabbed menu handlers.
  - tie-break is deterministic (stable entity rank) if multiple tab items report triggered in one frame.
- `keyboard_activation_target(...)`: reusable keyboard activation target resolver.
- `apply_tab_activation_with_audio(...)`: reusable tab-state update + optional click-audio + `TabChanged` emission.

Reusable systems:

- `sanitize_tab_selection_indices(...)`: keeps tab selected indices valid when tab items change.
- `sync_tab_bar_state(...)`: syncs `TabBarState` from current `SelectableMenu` state.

Usage rule:

- Keep tab interaction in `systems/ui/tabs.rs`, and keep page-specific meaning of each tab (content shown, focus transfer, etc.) in feature modules (for example `startup/menus/mod.rs`).

### Tabbed Menu Arbitration (Menus)

`src/startup/menus/tabbed_menu.rs` now centralizes owner-level tabbed-menu arbitration in
`sync_tabbed_menu_focus(...)`:

- keyboard focus transfers (options <-> tabs)
- footer left/right movement and footer up handoff
- pointer hover focus transfer
- tab click focus transfer

Rule:

- Keep tabbed-menu focus/selection priority logic in this single owner-level arbiter to avoid competing systems writing `SelectableMenu` state in the same frame.

### Table Cell Borders

`Column` in `src/entities/text/mod.rs` now supports optional per-cell border-side control:

- `Column::with_cell_boundary_sides(RectangleSides)`

Guideline:

- Use this instead of custom border entities when you need table-driven boxed cells (for example tab headers whose selected tab hides the bottom border).

## Scheduling and Layering

`MenusPlugin` uses system sets to keep behavior deterministic:

- `MenuSystems::Core`
- `MenuSystems::Commands`
- `MenuSystems::PostCommands`
- `MenuSystems::Visual`

High-level order:

1. Input + intent creation.
2. Command execution.
3. Invariant/sanity enforcement.
4. Visual synchronization.

Layer focus guard:

- `enforce_active_layer_focus` runs after `InteractionSystem::Selectable` and ensures only the currently active layer (base/dropdown/modal) can keep selection/trigger state.
- Inactive layers have their option click/visual state cleared and their previous selected index restored.

Conflict safety:

- Use `ParamSet` when two queries may touch the same component type (for example, multiple `&mut SelectableMenu` queries for roots and dropdown menus).
- If a system reads layer visibility (`Option<&Visibility>`) and also mutates dropdown visibility (`&mut Visibility`), place those queries in one `ParamSet` and access them sequentially.
- Prefer this over relying on implicit disjointness.
- Where systems aggregate by menu (`HashMap<Entity, ...>`), process owners in stable entity order to avoid query-iteration-order behavior drift.

## Window Integration

Window UI already shares interaction primitives with menus via `Clickable`, `Selectable`, and `InteractionGate`.

- Window z/layer behavior and click blocking are handled in `src/entities/sprites/window.rs` and respected by the interaction systems.
- Dropdown/selector utilities now live under `systems/ui`, so non-menu window UIs can reuse:
  - `DropdownLayerState` + dropdown helpers for layered popup lists.
  - `ShortcutKey` + shortcut collection for window-local key actions.

This keeps menu and window interactions on the same ECS foundation without duplicating behavior.

## Modal Patterns

Current reusable modal pattern in menus:

1. Spawn modal root with `SelectableMenu` and standard option bundle children.
2. Route keys to modal buttons through `MenuIntent`.
3. Handle clicked modal buttons in one command system.
4. Despawn modal via `close_video_modals(...)`.

This should be reused for yes/no confirmations and countdown confirmations.

## Dropdown Patterns

Correct behavior model:

- Open with right arrow while parent option is selected.
- Navigate dropdown items with up/down.
- Confirm item with enter or click on hovered item.
- Left/backspace/escape closes dropdown without leaving page.
- Click outside closes dropdown.
- Dropdown opening is suppressed for one frame per owner after a selection to avoid immediate reopen loops.
- Text centering reuse: dropdown row vertical recentering should use the shared `centered_text_y_correction(...)` helper from `src/entities/text/mod.rs` (same centering model as table cell text).

## Input Model Guidelines

### Keyboard should override hover until pointer movement

This is handled by `SelectableMenu` pointer lock behavior (`keyboard_locked`).

Guideline:

- Do not add local ad-hoc hover suppression. Reuse existing keyboard lock semantics.

### Forced click activation

Use `SelectableClickActivation` instead of custom click hacks:

- `SelectedOnAnyClick`: menu-only overlays where click-anywhere confirms selected option.
- `HoveredOnly`: dropdowns and precision lists.

## Reuse Recipes

### Add a new menu page

1. Add a `MenuPage` variant.
2. Add `MenuOptionDef` list + `MenuPageDef` entry.
3. Wire `MenuCommand` variants for navigation/action.
4. Rebuild content through existing `spawn_page_content`/`rebuild_menu_page` flow.

### Add a new dropdown to another option

1. Add dropdown marker component (similar to `VideoResolutionDropdown`).
2. Spawn dropdown as child of the menu root page content.
3. Give dropdown root a `SelectableMenu` with `HoveredOnly` activation.
4. Use generic helpers in `src/systems/ui/dropdown.rs` for open/close/single-visible invariants.
5. Reuse `DropdownLayerState` instead of new ad-hoc state fields.

### Add shortcut keys to options

1. Attach `ShortcutKey(KeyCode)` to option entity.
2. Let `collect_shortcut_commands(...)` emit `MenuIntent::TriggerCommand`.
3. Handle the same `MenuCommand` in the normal command system.

This keeps keyboard shortcuts and click paths unified.

## Anti-Patterns to Avoid

- Duplicating click/selection logic in custom systems when `SelectableMenu` already provides it.
- Storing unrelated navigation/dropdown state inside domain resources (for example, video settings resource).
- Writing new one-off arrow/indicator rendering when `system_menu` primitives exist.
- Directly mutating many UI entities from many systems without a clear set/order boundary.
- Relying on query disjointness assumptions when `ParamSet` can guarantee safety.

## Manual Validation Checklist

- Keyboard up/down moves selection and plays switch sound.
- Enter and click activate same command path.
- Dropdown open/close works with keyboard and mouse.
- Dropdown hover highlights; selected item remains selected after close.
- Modal options are selectable by keyboard and mouse.
- Escape/backspace semantics match intended stack behavior.
- Unsaved-changes modal appears whenever leaving dirty video settings.
- Video tabs:
  - Tab key cycles tabs.
  - Up on top video option moves keyboard focus to tabs.
  - Left/right cycles tabs while tab focus is active.
  - Mouse hover/click selects tabs.
  - Selected tab hides its bottom border and tab text styling updates.
- No Bevy B0001 query conflicts at startup/runtime.

## Suggested Extension Direction

For future composability across non-video UI:

- Keep generic dropdown logic in `src/systems/ui/dropdown.rs` component-parametric (`D` + root marker `R`).
- Keep generic tab logic in `src/systems/ui/tabs.rs`; only map tab index to domain behavior in scene/menu modules.
- Keep stack/exit prompt behavior in `stack.rs` and route through `MenuIntent`.
- Keep row visuals in `system_menu.rs` (indicators, bars, cycle arrows) and configure via components.
- Add new menu features by composition (components + systems), not by introducing page-specific interaction loops.
