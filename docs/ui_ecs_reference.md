# UI ECS Reference

This document describes the reusable UI interaction and menu architecture in this project, with an emphasis on composability and avoiding duplicate logic.

## Scope

Covered systems and modules:

- `src/systems/interaction/mod.rs`
- `src/systems/ui/dropdown.rs`
- `src/systems/ui/layer.rs`
- `src/systems/ui/selector.rs`
- `src/systems/ui/tabs.rs`
- `src/systems/ui/menu/debug_showcase.rs`
- `src/systems/ui/menu/flow_tests.rs`
- `src/entities/text/mod.rs`
- `src/startup/system_menu.rs`
- `src/systems/ui/menu/mod.rs`
- `src/systems/ui/menu/stack.rs`
- `src/entities/sprites/window.rs`

## Design Goals

- UI behavior should be data-driven by components/resources, not hardcoded per menu.
- Keyboard/mouse behavior should be consistent across menus and submenus.
- Menu layering (root menu, dropdown, modal) should be explicit and conflict-safe.
- New UI elements should reuse shared systems (`Clickable`, `SelectableMenu`, `OptionCycler`, etc.) instead of adding one-off handlers.
- Interaction behavior must use interaction primitives as source of truth; visual state is derived output only.
- Primitive authoring should prefer required-components plus insert hooks over new Bundle-first APIs.

## Latest Style and Coding Preferences (2026-02-20)

- Build reusable UI as self-contained primitives first (`src/systems/ui/*`), then compose in feature modules.
- For reusable primitives, do not add new bundle-first APIs.
- Use root component + `#[require(...)]` + `#[component(on_insert = ...)]` as the default construction model.
- Adding a primitive root should produce a working behavior unit without manual hidden-child wiring.
- Keep behavior truth in interaction primitives (`Hoverable`, `Clickable`, `Pressable`, `SelectableMenu`, `Selectable`, `OptionCycler`), not `InteractionVisualState`.
- Keep owner/layer scoping explicit (`UiLayer { owner, kind }`) and maintain query disjointness (`ParamSet`/`Without<T>`).

## Primitive Construction Guidelines

### Preferred Pattern (Required Components + Insert Hooks)

When adding a reusable primitive:

1. Define a root component for the primitive.
2. Use `#[require(...)]` to declare mandatory component contracts.
3. Use `#[component(on_insert = ...)]` to spawn and wire child entities and internal state.
4. Keep caller API minimal: inserting the root primitive should create a working, interactive unit.

### Why this pattern

- Prevents partial wiring bugs where visuals exist but interaction contracts are missing.
- Keeps primitive internals encapsulated and reusable across menus, windows, and scenes.
- Reduces composition duplication and makes behavior portable.

### Hard rule for new UI work

- Do not introduce new Bundle-first construction APIs for reusable UI primitives.
- Existing Bundle helpers are compatibility shims and should be migrated over time, not expanded.

### Do / Don't Quick Reference

Do:

- Insert a primitive root and let its insert hook wire internals.
- Keep command/reducer systems focused on domain mapping (not primitive mechanics).
- Reuse shared dropdown/tab/selector/scroll helpers before adding new behavior paths.

Don't:

- Spawn primitive-internal child contracts manually from composition modules.
- Add per-menu one-off hover/click/select state machines when primitive state already exists.
- Use `InteractionVisualState` as behavior truth.

## Core Interaction Primitives

### `InteractionGate` and `InteractionCapture`

- `InteractionGate` controls whether an entity can be interacted with in the current context.
- `InteractionCapture` marks contexts where gameplay interaction should be suppressed by menu interaction.
- Use `interaction_context_active(...)` + `interaction_gate_allows(...)` to gate every interaction system.

Pattern:

- Gameplay entities: `InteractionGate::GameplayOnly`
- Menu entities opened over gameplay: `InteractionGate::PauseMenuOnly`

### `Hoverable`

Pointer hover truth for behavior systems.

Fields:

- `hovered: bool`: one-frame hover result from `hoverable_system`.

Use when:

- Any system needs to know whether an entity is currently hovered (click routing, drag start, hover-driven effects, menu/tab/dropdown pointer selection).

Rules:

- Treat `Hoverable.hovered` as the canonical hover signal.
- Do not use `InteractionVisualState.hovered` as hover truth for behavior.
- `Clickable<T>` requires `Hoverable`, so pointer click behavior and hover behavior stay aligned.

### `Clickable<T>`

Component for pointer-triggered actions.

Fields:

- `actions: Vec<T>`: action keys mapped in `ActionPallet`.
- `region: Option<Vec2>`: optional explicit hit area.
- `triggered: bool`: one-frame latch set by interaction systems.

Use when:

- Entity should respond to mouse click.
- Entity is also selectable and you want keyboard activation to funnel into the same click path.

Rules:

- Treat `Clickable<T>.triggered` as the canonical "activated by click/select" signal.
- Systems that consume activation should read this latch, then clear/allow frame reset via interaction systems.
- Do not infer activation from `InteractionVisualState.pressed`.

### `Pressable<T>`

Component for keyboard-triggered actions through key mappings.

Fields:

- `mappings: Vec<KeyMapping<T>>`
- `triggered_mapping: Option<usize>`

Use when:

- The action does not need pointer hit-testing and should map directly to keys.

Rules:

- Treat `Pressable<T>.triggered_mapping` as the canonical keyboard activation signal.
- Do not use `InteractionVisualState.pressed` as keyboard action truth.

### `SelectableMenu` + `Selectable`

Provides menu-style selection and activation.

- `SelectableMenu` is a menu root/controller.
- `Selectable` marks selectable children with `(menu_entity, index)`.

`SelectableMenu` key fields:

- `selected_index`
- `keyboard_locked`
- `up_keys`
- `down_keys`
- `activate_keys`
- `wrap`
- `click_activation: SelectableClickActivation`

Click activation modes:

- `SelectedOnAnyClick`: any click activates the currently selected item.
- `HoveredOnly`: only clicked hovered item activates.

Use `HoveredOnly` for dropdowns (avoids accidental forced activation), and use `SelectedOnAnyClick` for full-screen modal/pause-style menus where forced click is desired.

Rules:

- Treat `SelectableMenu.selected_index` as the canonical selected option.
- Treat `SelectableMenu.keyboard_locked` as the canonical pointer-vs-keyboard arbitration state.
- Do not use `InteractionVisualState.selected`/`keyboard_locked` as source of truth for menu logic.

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

Single-frame visual output (derived):

- `hovered`
- `pressed`
- `selected`
- `keyboard_locked`

Reset each frame by `reset_interaction_visual_state`, then rebuilt by interaction systems.

Hard rule:

- `InteractionVisualState` is visual-only and must not be used as behavioral state in reducers, command handlers, navigation, tab/dropdown/modal flow, or interaction arbitration.
- If behavior needs hover/press/select information, read `Hoverable`, `Clickable`, `Pressable`, `SelectableMenu`, `Selectable`, and `OptionCycler` instead.

### `InteractionVisualPalette`

Color palette applied by `apply_interaction_visuals`.

Use this instead of manual per-frame color writes when possible.

## Interaction Source-of-Truth Hierarchy

Use this order when writing UI behavior:

1. Gating and scope: `InteractionGate`, `InteractionCapture`, `UiLayer`/active-layer resolution.
2. Hover: `Hoverable.hovered`.
3. Activation:
   - Pointer/selection activation: `Clickable<T>.triggered`.
   - Keyboard mapping activation: `Pressable<T>.triggered_mapping`.
4. Selection/navigation: `SelectableMenu.selected_index`, `Selectable.menu_entity/index`, `SelectableMenu.keyboard_locked`.
5. Value cycling: `OptionCycler` trigger/bounds fields.
6. Visuals only: `InteractionVisualState` + `InteractionVisualPalette`.

Developer checklist before adding new UI behavior:

- If you read `InteractionVisualState` outside visual sync systems, stop and switch to primitive components above.
- Reuse existing primitive systems before adding menu/page-specific input code.
- Keep visual mutation (`TextColor`, bars/arrows, scale/glow) in visual systems and keep behavior systems side-effect focused.

## System Menu Building Blocks (`system_menu.rs`)

### Root and Chrome

- `spawn_root(...)`: spawns a root menu with `SelectableMenu` and switch audio pallet.
- `spawn_chrome_with_marker(...)`: panel, border, title, hint.
- `play_navigation_switch(...)`: shared keyboard-nav sound utility used by both main-scene and UI menu systems.

### Option Construction (Legacy + Preferred)

- Legacy compatibility helper: `SystemMenuOptionBundle::new_at(...)` creates text option + selection + visual palette state.
- Preferred pattern for new/reusable surfaces:
  - Use primitive roots (`MenuSurface`, `SelectorSurface`, `DropdownSurface`) and let insert hooks wire required contracts.
  - Keep bundle-style helpers as migration shims; do not add new reusable bundle-first APIs.

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

## Menu Stack Architecture (`systems/ui/menu`)

### Core Types

- `MenuRoot { host, gate }`: root identity + interaction gate.
- `MenuStack`: page stack + remembered selected index per frame.
- `MenuPage`, `MenuCommand`, `MenuOptionCommand`: declarative navigation/action model.
- `MenuPageContent`: marker for spawned page children to support clean rebuild/despawn.

### Page Composition Module

Menu page content spawn/rebuild now lives in `src/systems/ui/menu/page_content.rs`:

- `spawn_page_content(...)`
- `rebuild_menu_page(...)`

Rule:

- Keep page composition (tables, option rows, dropdown children) isolated from runtime
  command/input systems so new layout changes do not increase command-flow complexity.

### Navigation State Resource (Menu-specific)

`MenuNavigationState` (`stack.rs`) stores cross-system transient intent:

## Debug Showcase Reference Composition

The debug UI showcase is implemented as a primitive-backed reference in
`src/systems/ui/menu/debug_showcase.rs`.

- Root component: `DebugUiShowcaseRoot` (`#[require]` + `on_insert`) spawns all demo windows.
- Demos are interactive and composed from the same primitives used by real menus:
  `SelectableMenu`, `Selectable`, `Clickable`, `OptionCycler`, `TabBar`, and `ScrollableRoot`.
- The reducer/effects layer toggles the showcase root, instead of constructing one-off table visuals.

This module is the preferred example for composing multiple reusable primitives inside `Window` entities without re-implementing menu engines.

## Flow Tests

Cross-system behavior checks live in `src/systems/ui/menu/flow_tests.rs` and should be extended when changing:

- layer arbitration
- tab focus transitions
- dropdown open/close semantics
- menu stack/pop/push state transitions

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
  - lives in `src/systems/ui/menu/command_reducer.rs`
  - `reduce_push_menu_command(...)`
  - `reduce_pop_menu_command(...)`
  - `reduce_toggle_resolution_dropdown_command(...)`
  - `reduce_toggle_display_mode_command(...)`
  - `reduce_toggle_vsync_command(...)`
  - `reduce_reset_video_defaults_command(...)`
  - `reduce_menu_command(...)` dispatches `MenuCommand` to reducers.
- Effect application:
  - reducers return `MenuReducerResult` (flags + payloads only).
  - lives in `src/systems/ui/menu/command_effects.rs`
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

`src/systems/ui/menu/defs.rs` now exposes a single schema primitive:

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

- `open_for_parent<D>()`
- `close_all<D>()`
- `close_for_parent<D>()`
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

- Keep tab interaction in `systems/ui/tabs.rs`, and keep page-specific meaning of each tab (content shown, focus transfer, etc.) in feature modules (for example `src/systems/ui/menu/mod.rs`).

### Tabbed Menu Arbitration (Menus)

`src/systems/ui/menu/tabbed_menu.rs` now centralizes owner-level tabbed-menu arbitration in
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

1. Spawn modal root with `MenuSurface` + `SelectableMenu` and selector-surface option rows.
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
