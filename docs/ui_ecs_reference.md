# UI ECS Reference

This document describes the reusable UI interaction and menu architecture in this project, with an emphasis on composability and avoiding duplicate logic.

## Scope

Covered systems and modules:

- `src/systems/interaction/mod.rs`
- `src/startup/system_menu.rs`
- `src/startup/menus/mod.rs`
- `src/startup/menus/dropdown.rs`
- `src/startup/menus/selector.rs`
- `src/startup/menus/stack.rs`

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

### Option Bundle

- `SystemMenuOptionBundle::new_at(...)` creates text option + `Selectable` + visual palette state.

### Indicators and Bars

- Selection arrows: `ensure_selection_indicators` + `update_selection_indicators`.
- Selection bar: `SystemMenuSelectionBarStyle`, `ensure_selection_bars`, `update_selection_bars`.
- Left/right cycle arrows: `SystemMenuCycleArrowOffset`, `ensure_cycle_arrows`, `update_cycle_arrows`.

Rule:

- Prefer these systems over custom indicator rendering so interaction look/feel stays consistent.

## Menu Stack Architecture (`startup/menus`)

### Core Types

- `MenuRoot { host, gate }`: root identity + interaction gate.
- `MenuStack`: page stack + remembered selected index per frame.
- `MenuPage`, `MenuCommand`, `MenuOptionCommand`: declarative navigation/action model.
- `MenuPageContent`: marker for spawned page children to support clean rebuild/despawn.

### Navigation State Resource

`MenuNavigationState` (`stack.rs`) stores cross-system transient intent:

- `exit_prompt_target_menu`
- `exit_prompt_closes_menu_system`
- `pending_exit_menu`
- `pending_exit_closes_menu_system`

Purpose:

- Keep exit/unsaved-confirm transitions out of `VideoSettingsState`.
- Make stack/modal flow reusable for non-video menus.

### Dropdown State Resource

`MenuDropdownState` (`dropdown.rs`):

- `open_menu: Option<Entity>`
- `suppress_toggle_once: bool`

Generic helpers (component-parametric):

- `any_open<D>()`
- `open_for_menu<D>()`
- `close_all<D>()`
- `close_for_menu<D>()`
- `enforce_single_visible_layer<D>()`

These helpers are reusable for any dropdown component type `D`.

### Selector/Shortcut Utilities

`selector.rs`:

- `MenuOptionShortcut(KeyCode)` component
- `collect_shortcut_commands(...)`
- `sync_option_cycler_bounds(...)`

Use `MenuOptionShortcut` on options that should be triggerable by direct keybind.

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

Conflict safety:

- Use `ParamSet` when two queries may touch the same component type (for example, multiple `&mut SelectableMenu` queries for roots and dropdown menus).
- Prefer this over relying on implicit disjointness.

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
- Dropdown opening is suppressed for one frame after a selection (`suppress_toggle_once`) to avoid immediate reopen loops.

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
4. Use generic helpers in `dropdown.rs` for open/close/single-visible invariants.
5. Reuse `MenuDropdownState` instead of new ad-hoc state fields.

### Add shortcut keys to options

1. Attach `MenuOptionShortcut(KeyCode)` to option entity.
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
- No Bevy B0001 query conflicts at startup/runtime.

## Suggested Extension Direction

For future composability across non-video UI:

- Keep generic dropdown logic in `dropdown.rs` component-parametric (`D`).
- Keep stack/exit prompt behavior in `stack.rs` and route through `MenuIntent`.
- Keep row visuals in `system_menu.rs` (indicators, bars, cycle arrows) and configure via components.
- Add new menu features by composition (components + systems), not by introducing page-specific interaction loops.
