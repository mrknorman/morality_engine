# UI Secondary Refactor Plan

Purpose:
- Enforce strict primitive architecture across all UI modules.
- Eliminate partial/visual-only compositions for interactive features.
- Ensure debug UI showcase examples are fully interactive and built from the same reusable primitives.

## Guardrails (Non-negotiable)

1. No new Bundle-first APIs for reusable UI primitives.
2. New primitives must use:
   - Root component
   - `#[require(...)]` contracts
   - `#[component(on_insert = ...)]` hierarchy construction
3. Primitives must be self-contained and independently operable.
4. Interaction behavior must read primitive truth components (`Hoverable`, `Clickable`, `Pressable`, `SelectableMenu`, `Selectable`, `OptionCycler`) and not `InteractionVisualState`.

## Stage 1: Audit and Classification

- [x] Inventory all UI modules under `src/systems/ui/*` and `src/systems/ui/menu/*`.
- [x] Classify each feature as:
  - Primitive-compliant
  - Partially compliant
  - Non-compliant (bundle-first, externally wired internals, or visual-only facades).
- [x] Produce a migration table with owner file, current construction pattern, and target primitive pattern.

Deliverable:
- A compliance matrix appended to this plan.

### Initial Compliance Matrix (Baseline)

| Module | Classification | Notes |
| --- | --- | --- |
| `src/systems/ui/layer.rs` | Primitive-compliant | Owner-scoped layer resolution is reusable and domain-agnostic. |
| `src/systems/ui/dropdown.rs` | Primitive-compliant | Generic owner-scoped dropdown state + helpers, no menu domain coupling. |
| `src/systems/ui/tabs.rs` | Primitive-compliant | Generic tab state/activation helpers. |
| `src/systems/ui/selector.rs` | Primitive-compliant | Generic shortcut + selector bound helpers. |
| `src/systems/ui/scroll/*` | Primitive-compliant | Reusable render-to-texture scroll + scrollbar primitives with tests. |
| `src/systems/ui/hover_box.rs` | Primitive-compliant | Reusable hover tooltip primitive. |
| `src/systems/ui/discrete_slider.rs` | Partially compliant | Reusable widget, but showcase/adapters need stronger self-contained root integration. |
| `src/systems/ui/menu/*` | Composition-heavy / partially compliant | Strong reducer/effects split, but still contains some primitive-like behavior that should migrate down into `systems/ui/*`. |
| `src/systems/ui/menu/debug_showcase.rs` | Primitive-compliant | Dedicated self-contained showcase root with insert-hook composition and interactive primitive demos. |

## Stage 2: Primitive Contract Normalization

- [x] Define and standardize root primitive components for:
  - Menu surface
  - Selector/cycler surface
  - Tab bar
  - Dropdown
  - Scrollable/scrollbar
  - Hover box
- [x] Ensure each root primitive owns its required child hierarchy via primitive-owned insertion/lifecycle hooks.
- [x] Remove hidden external wiring assumptions from primitive behavior systems.

Deliverable:
- Primitive APIs that can be used by adding a single root component plus config components.

Progress notes:
- `TabBar` now enforces required primitive contracts on insertion (`SelectableMenu`, `TabBarState`, activation policy/state sync) in `src/systems/ui/tabs.rs`.
- `DiscreteSlider` now enforces baseline transform/visibility contracts in `src/systems/ui/discrete_slider.rs`.
- `DiscreteSlider` lifecycle is now primitive-owned via `DiscreteSliderPlugin` (`EnsureSlots`/`SyncSlots` system sets), and menu code now composes by ordering against those sets instead of scheduling slider internals directly (`src/systems/ui/discrete_slider.rs`, `src/systems/ui/menu/mod.rs`, `src/startup/mod.rs`).
- `DiscreteSlider` root now seeds slot child hierarchy through an insert hook, with runtime ensure/sync systems retained for dynamic slot-count drift correction (`src/systems/ui/discrete_slider.rs`).
- `HoverBoxRoot` now self-initializes primitive defaults via insert hook (owner-scoped `UiLayer`, hidden root visuals, and required child label/border structure) in `src/systems/ui/hover_box.rs`.
- `ScrollBar` now enforces required root contracts (`Transform`, `Visibility`, drag state) and seeds track/thumb child hierarchy via insert hook; runtime ensure flow now also repairs stale/missing part handles (`src/systems/ui/scroll/mod.rs`, `src/systems/ui/scroll/scrollbar.rs`).
- `SelectorSurface` now provides a root primitive contract for selector/cycler rows (`Selectable` + optional `OptionCycler` via insert hook) in `src/systems/ui/selector.rs`.
- Menu composition now adopts `SelectorSurface` for video tabs/dropdown rows/cyclers and debug showcase tabs/dropdowns in `src/systems/ui/menu/page_content.rs` and `src/systems/ui/menu/debug_showcase.rs`.
- `DropdownSurface` now provides a root dropdown primitive contract (`UiLayer::Dropdown`, default hidden visibility, selectable menu baseline) via insert hook in `src/systems/ui/dropdown.rs`, and video dropdown composition now consumes it in `src/systems/ui/menu/page_content.rs`.
- `DropdownSurface` now also owns click-activation policy wiring for dropdown menus, and debug showcase dropdown panels now compose through it.
- `SystemMenuOptionBundle` now routes selection construction through `SelectorSurface` (instead of directly embedding `Selectable`) in `src/startup/system_menu.rs`.
- `MenuSurface` now provides a root menu primitive contract (`UiLayer`, `SelectableMenu`, click-activation policy) via insert hook in `src/systems/ui/menu_surface.rs`, and root menu spawning now consumes it in `src/systems/ui/menu/root_spawn.rs`.
- Debug showcase menu roots now also consume `MenuSurface` (including dropdown-layer panel ownership) in `src/systems/ui/menu/debug_showcase.rs`.
- Video modal roots now compose through `MenuSurface` with `UiLayerKind::Modal`, and video tabs/dropdown roots removed redundant `SelectableMenu` boilerplate where primitive defaults already apply (`src/systems/ui/menu/modal_flow.rs`, `src/systems/ui/menu/page_content.rs`).
- `ScrollableRoot` runtime lifecycle now self-seeds a default `ScrollableContent` child when missing, alongside camera/surface children, removing menu-specific content-root wiring assumptions (`src/systems/ui/scroll/lifecycle.rs`).

## Stage 3: Menu Composition Migration

- [x] Refactor `src/systems/ui/menu/*` to compose primitives only, not rebuild primitive internals.
- [x] Reduce menu-specific branching where primitive state machines already exist.
- [x] Keep reducer/effects split intact while moving mechanics to primitives.

Deliverable:
- Menu modules focused on policy and command mapping.

Progress notes:
- Menu composition no longer directly constructs primitive internals in runtime paths (`Selectable`, `OptionCycler`, `UiLayer`) and instead uses primitive roots (`MenuSurface`, `SelectorSurface`, `DropdownSurface`).
- Dropdown behavior paths now consistently route through shared dropdown primitive helpers (`open_for_parent`, `close_for_parent`, `close_all`) with row-support checks centralized in `src/systems/ui/menu/dropdown_flow.rs`.
- Reducer/effects split remains intact (`command_reducer` pure transition logic, `command_effects` Bevy side effects), with menu modules acting as policy/input orchestration.

## Stage 4: Debug UI Showcase Rebuild

- [x] Move debug showcase into its own module (outside command effect body).
- [x] Replace table-only demos with primitive-backed demos:
  - Interactive selector menu window
  - Interactive tabs window
  - Interactive dropdown window
  - Interactive scroll window
- [x] Ensure each demo window is assembled from reusable primitives with no one-off interaction logic.

Deliverable:
- Debug showcase as a reference implementation of composable primitives.

## Stage 5: Interaction and Layer Conformance Pass

- [x] Verify owner-scoped layer behavior for all primitives (`Base`, `Dropdown`, `Modal`).
- [x] Verify input arbitration consistency (keyboard lock, hover handoff, active layer priority).
- [x] Remove any behavior paths that still depend on visual-state fields.

Deliverable:
- Deterministic interaction behavior under mixed mouse+keyboard input.

Progress notes:
- Layer-priority ordering (`Modal > Dropdown > Base`) is now covered by integration tests in `src/systems/ui/menu/flow_tests.rs`.
- Scroll primitives now validate owner/layer gating behavior (base, dropdown, modal) in `src/systems/ui/scroll/tests.rs`.
- Selector/cycler mixed input arbitration is now covered by mouse+keyboard integration in `src/systems/ui/menu/flow_tests.rs`.
- Added regression coverage for nested tab interaction surfaces to ensure `MenuSurface::without_layer()` preserves owner base-layer activity (`src/systems/ui/menu/flow_tests.rs`).
- Behavior flow paths (`command_flow`, `dropdown_flow`, `menu_input` shortcuts) now derive decisions from `Hoverable`/`Clickable`/`SelectableMenu`/layer state; remaining `InteractionVisualState` usage is isolated to visual sync systems.

## Stage 6: Query-Safety Hardening

- [x] Audit all UI systems for overlapping mutable query risk.
- [x] Apply `ParamSet` and `Without<T>` contracts where needed.
- [x] Add short query contract comments at each multi-query system.

Deliverable:
- No B0001 panics under stress interactions in menus/debug showcase.

Progress notes:
- Debug showcase dropdown close path now enforces disjoint trigger/panel queries via `Without<T>` filters and explicit query contract comments.
- Added query-safety contract comments to additional multi-query systems (`sync_discrete_slider_slots`, dropdown outside-click close path) to keep alias boundaries explicit as UI primitives evolve.
- Added full regression run checkpoint after primitive/plugin changes: `cargo test --manifest-path Cargo.toml systems::ui:: --quiet` (82 passed, no B0001).
- Full UI test subset now passes (`cargo test --manifest-path Cargo.toml systems::ui:: --quiet`) with no B0001 panics.
- Scroll multi-query systems now carry explicit query contracts (`sync_scroll_extents`, `handle_scrollable_pointer_and_keyboard_input`, `sync_scroll_content_offsets`, `ensure_scrollable_runtime_entities`, `sync_scrollable_render_entities`, `sync_scroll_content_layers`) to keep alias boundaries readable as primitives evolve.
- Edge auto-scroll geometry now consumes per-root scroll edge-zone settings (instead of module constants), so scroll behavior configuration remains primitive-scoped and deterministic.
- Updated regression checkpoint after scroll primitive lifecycle hardening: `cargo test --manifest-path Cargo.toml systems::ui:: --quiet` (85 passed, no B0001).

## Stage 7: Test Coverage

- [x] Add/extend unit tests for primitive reducers and state transitions.
- [x] Add integration-level tests for:
  - Tabs activation and content switching
  - Dropdown open/select/close paths
  - Selector/cycler keyboard+mouse interaction
  - Scrollbar drag and wheel behavior
- [x] Add debug showcase interaction smoke test hooks where feasible.

Deliverable:
- Regression safety for primitive behavior and composition.

Progress notes:
- `command_reducer` and menu flow tests are in place (`src/systems/ui/menu/command_reducer.rs`, `src/systems/ui/menu/flow_tests.rs`).
- Debug showcase now has targeted unit coverage for core index-cycling behavior (`src/systems/ui/menu/debug_showcase.rs`).
- Owner-scoped dropdown open/select/close integration behavior is covered in `src/systems/ui/menu/flow_tests.rs`.
- Tabs now have sync behavior tests for activation + explicit-mode semantics in `src/systems/ui/tabs.rs`.
- Tabbed focus activation/content-switch behavior is now covered in `src/systems/ui/menu/flow_tests.rs`.
- Selector/cycler mouse-selection + keyboard-cycle behavior is now covered in `src/systems/ui/menu/flow_tests.rs`.
- Scrollbar wheel + drag-path clamp behavior is now covered in `src/systems/ui/scroll/tests.rs`.
- Scrollbar root insertion behavior is now covered (insert-hook part seeding + root parenting) in `src/systems/ui/scroll/tests.rs`.
- Scrollbar stale-part repair behavior is now covered (missing part entity triggers deterministic rebuild) in `src/systems/ui/scroll/tests.rs`.
- Debug showcase smoke hooks continue to validate visual sync systems execute without query alias panics in `src/systems/ui/menu/debug_showcase.rs`.

## Stage 8: Documentation and Adoption

- [x] Update docs/examples to use primitive-root construction patterns.
- [x] Add "do/dont" examples for primitive composition in `docs/ui_ecs_reference.md`.
- [x] Mark deprecated bundle-first helper surfaces as migration targets.

Deliverable:
- Documentation aligned with implementation and enforced conventions.

## Acceptance Checklist

- [x] Debug showcase windows are all interactive (not visual-only).
- [x] Primitive-root insertion is sufficient to stand up each major UI feature.
- [x] No new reusable UI bundles introduced.
- [x] Layering and input arbitration are owner-scoped and deterministic.
- [x] No known B0001 query conflicts in UI systems.
