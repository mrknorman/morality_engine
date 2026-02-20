# UI Master Refactor Plan

Status legend:
- `[ ]` not started
- `[-]` in progress
- `[x]` completed

Last updated: 2026-02-20
Canonical plan: this file (`ui_masterplan.md`).

## Objectives
- Rebuild UI architecture into strict, reusable ECS primitives.
- Keep interaction deterministic under mixed keyboard + mouse input.
- Keep layered UI owner-scoped so independent surfaces never cross-interfere.
- Make menu/dropdown/tab/scroll/hover primitives composable across the project.

## Non-negotiable Constraints
1. Max composability: UI pieces must be reusable primitives.
2. Interaction gating must be layered and owner-scoped.
3. Tabs must be reusable primitives with abstracted logic.
4. No new bundle-first APIs for reusable primitives.
5. New reusable primitives must use:
   - a root component
   - `#[require(...)]`
   - `#[component(on_insert = ...)]`
6. Primitive behavior truth comes from interaction primitives (`Hoverable`, `Clickable`, `Pressable`, `SelectableMenu`, `Selectable`, `OptionCycler`), not `InteractionVisualState`.

## Progress Audit Snapshot (2026-02-20)

Legend:
- `status: done` = implemented and in active use
- `status: partial` = implemented but with known gaps/regressions
- `status: pending` = not yet started or not yet reliable

### Stage status summary
- Stage 0 Safety + Checkpoint: `status: partial`
  - Compile baseline is known (`cargo check` passes), but clean pre-change checkpoint discipline has not been consistently maintained.
- Stage 1 Audit and Classification: `status: done`
  - Compliance matrix and migration backlog now captured in `docs/ui_compliance_matrix.md`.
- Stage 2 Architecture Boundaries and Contracts: `status: done`
  - Contract docs are aligned with current boundaries and now cross-referenced by the compliance matrix (`docs/ui_compliance_matrix.md`).
- Stage 3 Owner-Scoped Interaction Context + Layer Manager: `status: done`
  - Owner-scoped active-layer arbitration is centralized in `ui::layer` and routed through shared deterministic ordering helpers.
  - Menu/tab/dropdown/modal systems now consume `active_layers_by_owner_scoped` + ordered owner/layer helpers instead of ad-hoc local scans.
  - Owner-scoped arbitration has explicit regression coverage for cross-owner isolation and deterministic priority.
- Stage 4 Primitive Contract Normalization: `status: done`
  - Root primitives are normalized for menu surface, selector/cycler, tab bar, dropdown, scroll root/scrollbar, hover box, and discrete slider.
  - Primitive insertion contracts are covered by insertion smoke tests (`MenuSurface`, `SelectorSurface`, `DropdownSurface`, `TabBar`, `HoverBoxRoot`, `DiscreteSlider`).
  - Option composition is owned by `SystemMenuOptionRoot` via `#[require]` + `on_insert`.
- Stage 5 Menu Composition Migration: `status: done`
  - Menu composition paths are primitive-backed (`system_menu::spawn_option`, `MenuSurface`, `DropdownSurface`, `TabBar`, `DiscreteSlider`, `HoverBoxRoot`).
  - Reducer/effects split remains in place (`command_reducer` + `command_effects` + `command_flow`).
  - Main menu composition and command routing are on shared `ui::menu` modules; scene-local duplicate behavior path was removed.
- Stage 6 UI Module Realignment: `status: done`
  - Menu composition is under `src/systems/ui/menu/*` with clear submodules.
- Stage 7 Dropdown, Tabs, Footer Primitive Unification: `status: done`
  - Unified primitives are wired.
  - Footer single-selection visual override is now registered in visual sync; dropdown anchoring edge cases still need hardening.
- Stage 8 Command Reducer + Effects Split: `status: done`
  - `command_reducer.rs`, `command_effects.rs`, `command_flow.rs` are separated and active.
- Stage 9 Deterministic Input Arbitration: `status: done`
  - Arbitration order is enforced and regression-covered.
  - Recent progress: tabbed focus and scroll focus-follow no longer depend on option `InteractionVisualState` as behavior truth.
  - Recent progress: visual arbitration now explicitly suppresses top-option highlight state when tab focus is active, and suppresses option visuals for non-base active layers.
  - Recent progress: directional shortcut dispatch now resolves deterministically per menu by entity rank, and emits intents in stable owner order.
  - Recent progress: escape/modal/dropdown keyboard shortcut routing now uses deterministic owner/menu ordering rather than query first-match behavior.
  - Recent progress: dropdown keyboard-open flow now deterministically chooses the lowest owner index when multiple owners match in-frame.
  - Recent progress: dropdown value-apply routing now resolves active dropdown owners in shared layer order, removing local hash-key sort/dedup behavior.
  - Recent progress: tabbed-focus reducer now has explicit priority coverage for keyboard-vs-hover and click-vs-hover arbitration.
  - Recent progress: mixed-input jitter guards are now regression-tested (`keyboard_lock_prevents_hover_jitter_until_pointer_reengages`, `overlapping_hover_candidates_resolve_selection_deterministically`).
- Stage 10 Main Menu Composition Migration: `status: done`
  - Main menu options now route through shared `MenuCommand` reducer/effects, including `NextScene` and options-overlay spawn effects.
  - Main menu option-list assembly moved to shared composition (`ui::menu::spawn_main_menu_option_list`).
  - Main-menu overlay camera-follow and navigation-audio systems are now owned by `ui::menu::main_menu` instead of scene-local systems.
  - Legacy scene-local `MenuActions`/`ActionPallet` path and duplicate overlay-open handler were removed.
  - Scene module now only composes scene visuals/content and invokes shared menu composition.
- Stage 11 JSON Menu/Settings Schema Interface: `status: done`
  - Main menu and options menu both load through schema + typed command registry resolution.
  - Schema validation now fails on blank optional fields and non-finite option positions.
  - Options schema enforces strict layout container/group + shortcut parsing with explicit errors.
- Stage 12 Discrete Slider Primitive and Integration: `status: done`
  - `DiscreteSlider` primitive exists and is integrated in video options.
- Stage 13 Scrollable RTT Primitive: `status: done`
  - RTT scroll primitive + scrollbar are implemented and used.
  - Shared table/list adapter contracts now live in `ui::scroll` and menu adapters consume shared row/focus helpers.
  - Render-target budget controls and fallback policy are covered by targeted tests.
- Stage 14 HoverBox Primitive + Video Pilot: `status: done`
  - Option and dropdown hover descriptions are reintroduced and synced from video option metadata.
  - Timing/gating/mapping/exclusion regression tests are in place.
  - Mixed keyboard/mouse overlay toggling is now regression-covered.
- Stage 15 Debug UI Showcase Rebuild: `status: done`
  - Showcase windows are interactive and primitive-backed (selector, tabs, dropdown, scroll, hover box).
  - Dropdown demo now uses shared dropdown-layer state helpers instead of one-off open/close visibility toggles.
  - Tabs demo includes live `HoverBox` primitive wiring on tab labels (owner/layer scoped, delayed display).
  - Debug-showcase smoke coverage includes system initialization and root-construction checks.
- Stage 16 Known Bug Sprint: `status: done`
  - Historical value-cell, tab/footer transfer, dropdown alignment/flicker, and modal/layer gating regressions are covered by targeted tests and currently green.
  - Latest `./scripts/ui_regression.sh` + nextest pass confirms no active scripted regressions.
- Stage 17 Query-Safety Hardening: `status: done`
  - Many `ParamSet`/`Without` contracts are present.
  - Added explicit query-disjointness contract comments in dropdown view sync systems (`sync_resolution_dropdown_items`, `update_resolution_dropdown_value_arrows`, `recenter_resolution_dropdown_item_text`).
  - Added query-safety smoke tests that initialize high-risk systems without running gameplay state:
    - `command_flow::command_flow_systems_initialize_without_query_alias_panics`
    - `dropdown_view::dropdown_view_systems_initialize_without_query_alias_panics`
    - `menu_input::menu_input_systems_initialize_without_query_alias_panics`
    - `dropdown_flow::dropdown_flow_systems_initialize_without_query_alias_panics`
    - `main_menu::main_menu_systems_initialize_without_query_alias_panics`
    - `modal_flow::modal_flow_systems_initialize_without_query_alias_panics`
    - `scroll_adapter::scroll_adapter_systems_initialize_without_query_alias_panics`
    - `debug_showcase::debug_showcase_systems_initialize_without_query_alias_panics`
    - `tabbed_menu::tabbed_menu_systems_initialize_without_query_alias_panics`
    - `video_visuals::video_visual_systems_initialize_without_query_alias_panics`
  - `./scripts/ui_query_safety.sh` passes after the latest layer-ordering refactor.
  - Full query-safety preflight (`./scripts/ui_query_safety.sh`) and full UI regression (`./scripts/ui_regression.sh`) are currently green.
- Stage 18 Test Coverage Expansion: `status: done`
  - Added targeted regression tests for footer highlight resolution and hover description mapping.
  - Added scroll focus-follow regression for option-lock path without navigation key input.
  - Added top-table owner-resolution regression for scroll-parented video tables.
  - Added dropdown-flow regressions for scroll-aware dropdown opening and outside-click item protection.
  - Added menu-input regression tests for directional shortcut behavior (right activate, left back-only, tabs-focus block).
  - Added query-safety smoke tests for command/input/dropdown/modal/video visual systems.
  - Added tabbed-menu regression coverage for focused-owner suppression and stale-state cleanup restoration.
  - Added menu-input suppression regressions for non-base layer gating and tab-focus top-row suppression.
  - Added directional-shortcut regression coverage for deterministic highest-rank option dispatch when duplicate targets match.
  - Added modal-shortcut regression coverage for deterministic owner ordering.
  - Added dropdown keyboard-open regression coverage for deterministic owner ordering across multiple eligible menus.
  - Added stack-state regression coverage for stale menu target cleanup (`clear_stale_menu_targets`).
  - Added menu-input active-layer context regression (`active_shortcut_context_excludes_non_base_layers_and_marks_footer_nav`).
  - Added debug-showcase smoke coverage for command and visual system initialization.
  - Added multi-owner tab/dropdown isolation regression coverage in flow tests.
- Stage 19 Runtime Stress Validation: `status: done`
  - Repeatable pass now exists via `./scripts/ui_regression.sh` + full `cargo nextest run` (currently 139/139 passing, including mixed input/layer stress tests).
  - Added opt-in GPU input smoke lane (`./scripts/ui_gpu_smoke.sh`, or `UI_RUN_GPU_SMOKE=1 ./scripts/ui_regression.sh`) with keyboard+mouse-wheel scripted checks.
  - Manual in-game verification checklist is now documented in `docs/ui_manual_validation_checklist.md`.
- Stage 20 Documentation and Adoption: `status: done`
  - Primitive contracts, do/don't guidance, migration targets, and query-safety/test workflows are documented and aligned with current code.
- Stage 21 Tooling and Test Framework Rollout: `status: done`
  - `mdBook` content now includes the `./scripts/ui_regression.sh` flow and `nextest` profile usage.
  - Added `./scripts/ui_query_safety.sh` for fast query-alias/B0001 preflight checks.
  - Expanded rustdoc coverage on core UI interaction/layer/dropdown/tab primitives.
  - Decision: keep deterministic sampled property coverage in-tree (no additional property-test crate required currently).
- Stage 22 Cleanup and Redundancy Pass: `status: done`
  - Removed duplicated menu camera-center helper into shared `ui::menu::camera`.
  - Consolidated repeated dropdown/menu query signatures behind shared menu aliases.
  - Added primitive-insertion coverage for `ScrollableRoot` and `SystemMenuOptionRoot`.
  - Re-ran cleanup validation (`cargo check`, `./scripts/ui_query_safety.sh`, `./scripts/ui_regression.sh`) with green results.

## Active Bug Backlog (Priority)

1. No currently active scripted regressions in menu/dropdown/tab/scroll/modal flows.
   - Guardrail: continue running `./scripts/ui_regression.sh` after each stage-level change.
2. Runtime/manual QA still required for final closeout.
   - Use `docs/ui_manual_validation_checklist.md` for in-game stress verification.

## Contract Drift Notes (to reconcile during Stage 2/4/9)

1. `InteractionVisualState` is still read for behavior arbitration in some menu/tab paths.
   - `option_cycler_input_system` now keys from `Selectable` + `SelectableMenu.selected_index` (not visual-state selected).
   - `sync_video_top_scroll_focus_follow` now keys from owner-scoped tabbed option-lock state (not option visual-state flags).
   - Target: move behavior arbitration to primitive truth (`Hoverable`, `Clickable`, `SelectableMenu`) and keep visual state as output only.
2. Some composition callsites still rely on ad-hoc post-spawn wiring instead of deeper primitive root hooks.
   - Target: continue migrating behavior-critical contracts to required-component + `on_insert` ownership where practical.
3. Main-menu scene-local composition drift is resolved.
   - Main menu option composition + overlay follow/audio routing is now owned by shared `ui::menu` modules.

## Next Execution Sequence

1. [x] Reintroduce and harden HoverBox descriptions for video option names and relevant value options.
2. [x] Add focused regression tests for footer/tabs/dropdown interaction and hover description mapping.
3. [x] Resume bundle-first migration cleanup in menu composition paths.
4. [x] Complete tooling pass with repeatable test commands (`./scripts/ui_regression.sh`, `cargo nextest` when available).
5. [x] Run a dedicated mixed keyboard+mouse stress pass for owner-scoped arbitration.
6. [x] Add option-lock focus-follow regression test and route scroll focus-follow from tabbed focus lock state instead of visual-state reads.

## Stage 0: Safety + Checkpoint
- [ ] Create a clean checkpoint commit before functional changes.
- [ ] Record baseline behavior notes for main menu, pause menu, options/video, dropdowns, tabs, modals, debug showcase.
- [ ] Confirm current compile status and capture baseline command outputs.

## Stage 1: Audit and Classification
- [x] Inventory all UI modules under `src/systems/ui/*` and `src/systems/ui/menu/*`.
- [x] Classify each as primitive-compliant, partially compliant, or non-compliant.
- [x] Produce migration table: owner file, current construction pattern, target primitive pattern.
- [x] Identify all remaining bundle-first reusable APIs to migrate.

Deliverable:
- [x] Compliance matrix and migration backlog.

## Stage 2: Architecture Boundaries and Contracts
- [x] Reconfirm strict module boundaries:
  - `systems/ui/*` = reusable primitives only
  - `systems/ui/menu/*` = composition/policy only
  - scenes/startup = consumers
- [x] Reconfirm dependency direction and enforce it in code organization.
- [x] Reconfirm owner/root identity conventions for all layered UI elements.
- [x] Reconfirm query-safety standards (`ParamSet`/`Without`) as mandatory contract.

Deliverable:
- [x] Updated architecture contract doc aligned with actual module boundaries.

## Stage 3: Owner-Scoped Interaction Context + Layer Manager
- [x] Ensure all interaction gates resolve by owner, never globally.
- [x] Centralize active-layer resolution (`Base`, `Dropdown`, `Modal`) by owner.
- [x] Remove ad-hoc layer scans in menu/tab/dropdown/modal systems.
- [x] Route dimming/focus/interaction decisions through one layer source-of-truth.

Deliverable:
- [x] Deterministic owner-scoped layer arbitration across all UI surfaces.

## Stage 4: Primitive Contract Normalization
- [x] Standardize root primitives and contracts for:
  - menu surface
  - selector/cycler surface
  - tab bar
  - dropdown surface/state
  - scrollable root + scrollbar
  - hover box
  - discrete slider
- [x] Ensure each primitive owns required child hierarchy via insert/lifecycle hooks.
- [x] Remove hidden external wiring assumptions from primitive behavior systems.

Deliverable:
- [x] Single-root primitive insertion stands up each primitive behavior unit.

## Stage 5: Menu Composition Migration
- [x] Refactor `src/systems/ui/menu/*` to compose primitives only.
- [x] Remove menu-specific primitive reimplementations.
- [x] Keep reducer/effects split while moving mechanics to primitives.
- [x] Reduce feature-specific branching in generic menu flow paths.

Deliverable:
- [x] Menu modules are policy + command mapping only.

## Stage 6: UI Module Realignment
- [x] Ensure menu modules live under `ui::menu` with clean public API boundaries.
- [x] Remove or avoid transitional re-export shims.
- [x] Keep owner/layer/interaction contracts intact after module cleanup.

Deliverable:
- [x] Stable module topology aligned with architecture contract.

## Stage 7: Dropdown, Tabs, and Footer Primitive Unification
- [x] Keep dropdown open/close/single-visible/outside-click logic fully in reusable dropdown primitive.
- [x] Keep tab selection/activation/arbitration in reusable tab primitive path.
- [x] Keep horizontal footer navigation reusable and composition-driven.
- [x] Ensure independent owners can host tabs/dropdowns without cross-talk.

Deliverable:
- [x] Shared dropdown/tab/footer primitives used consistently by menu composition.

## Stage 8: Command Reducer + Effects Split
- [x] Keep pure reducer transitions separate from Bevy side effects.
- [x] Ensure command side effects are centralized and deterministic.
- [x] Keep behavior compatibility with existing flows during migration.

Deliverable:
- [x] Reducer/effects architecture with clear contracts and tests.

## Stage 9: Deterministic Input Arbitration
- [x] Enforce strict priority: layer > focus group > keyboard lock > hover.
- [x] Remove first-match query-iteration dependence.
- [x] Ensure one owner-level system decides selection priority.
- [x] Stabilize behavior under rapid mixed keyboard/mouse interaction.

Deliverable:
- [x] No selection jitter or nondeterministic ownership conflicts.

## Stage 10: Main Menu Composition Migration
- [x] Move main menu option list fully to shared menu composition path.
- [x] Remove scene-local duplicate menu behavior.
- [x] Reuse shared navigation audio + selection behavior paths.

Deliverable:
- [x] Main menu uses same composition system as other UI menus.

## Stage 11: JSON Menu/Settings Schema Interface
- [x] Define JSON schema for menu structure (title, hint, options, shortcuts, layout bindings).
- [x] Implement typed command registry bridge (`string id -> typed Rust handler`).
- [x] Add explicit validation failures (no silent fallback).
- [x] Migrate one menu as pilot and evaluate extension cost.

Deliverable:
- [x] Validated schema-driven menu composition path.

## Stage 12: Discrete Slider Primitive and Integration
- [x] Implement/normalize reusable `DiscreteSlider` primitive (keyboard + mouse).
- [x] Integrate into appropriate rankable options (off/low/medium/high patterns).
- [x] Ensure selector and slider interaction do not conflict.
- [x] Ensure slider behavior is owner/layer safe and composable.

Deliverable:
- [x] Stable slider primitive adopted in settings UI.

## Stage 13: Scrollable RTT Primitive

### Stage 13.1: Architecture Contract
- [x] Define reusable primitives (`ScrollableRoot`, `ScrollableViewport`, `ScrollableContentCamera`, `ScrollableRenderTarget`, `ScrollableItem`, `ScrollState`).
- [x] Keep owner-scoped integration with `UiLayer.owner` and `InteractionGate`.
- [x] Keep backend enum extensible (`RenderToTexture` first-class).

### Stage 13.2: Render Target + Camera Lifecycle
- [x] Add per-root render-target allocation/pooling.
- [x] Spawn dedicated content camera per root targeting RTT.
- [x] Assign dedicated render layers for scroll content.
- [x] Handle viewport resize/rebuild safely.

### Stage 13.3: Viewport Surface Composition
- [x] Render scroll output as clipped world-space surface.
- [x] Guarantee no overdraw beyond viewport bounds.
- [x] Keep stable visual ordering with text/borders/glow and CRT pipeline.

### Stage 13.4: Scroll Reducer and Motion Model
- [x] Implement pure scroll state reducer (`offset`, `content`, `viewport`, `max`, optional velocity/snap).
- [x] Support wheel, keyboard step/page/home/end, thumb drag, focus-follow intents.
- [x] Keep deterministic clamping and ordering under mixed inputs.

### Stage 13.5: Input Mapping on RTT Content
- [x] Map cursor viewport-space to content-space deterministically.
- [x] Resolve hovered/pressed rows by stable index/key.
- [x] Keep keyboard semantics aligned with menu systems.
- [x] Keep row/name/value click parity with non-scroll contexts.

### Stage 13.6: Reusable Adapters
- [x] Implement `ScrollableTableAdapter` for table menus.
- [x] Implement `ScrollableListAdapter<T>` for generic lists.
- [x] Add focus-follow hooks to keep selected option visible.

### Stage 13.7: Performance and Safety Hardening
- [x] Add texture budget controls and fallback policies.
- [x] Ensure camera/target cleanup on despawn.
- [x] Enforce query disjointness contracts in all scroll systems.

### Stage 13.8: Validation and Rollout
- [x] Add reducer/coordinate unit tests.
- [x] Add integration tests for mixed scroll + selection + dropdown/modal layering.
- [x] Roll out first to video options, then at least one secondary context.

Deliverable:
- [x] Reusable scroll primitive with stable interaction in multi-layer UI.

## Stage 14: HoverBox Primitive + Video Pilot
- [x] Define reusable `HoverBox` primitive API in `src/systems/ui/hover_box.rs`.
- [x] Add owner-scoped/layer-scoped hover arbitration contract.
- [x] Add delay behavior (`0.5s`) with deterministic show/hide transitions.
- [x] Add anchored placement below target + bounds-safe clamping.
- [x] Add style/config components (`HoverBoxStyle`, `HoverBoxContent`).
- [x] Integrate option-name descriptions in video menu (short, descriptive, layperson-readable).
- [x] Integrate dropdown value descriptions where relevant (exclude resolution values).
- [x] Add regression tests for timing, gating, mapping, and exclusions.
- [x] Validate behavior under mixed keyboard/mouse with overlays.

Deliverable:
- [x] Reusable hover tooltip primitive used by menu composition.

## Stage 15: Debug UI Showcase Rebuild
- [x] Move debug showcase to dedicated composition module.
- [x] Build interactive windows from real primitives (selector, tabs, dropdown, scroll, hover box).
- [x] Remove visual-only or one-off showcase interaction logic.
- [x] Ensure debug demos mirror production primitive wiring.

Deliverable:
- [x] Debug showcase acts as live primitive reference implementation.

## Stage 16: Known Bug Sprint (Pre-Refactor and Mid-Refactor)
- [x] Fix value-cell interaction dead zones and hover/click mismatches.
- [x] Fix keyboard transfer bugs between tabs/footer/options.
- [x] Fix dropdown flicker/jitter and alignment regressions.
- [x] Fix modal/input gating and layering regressions.
- [x] Add regression tests for each bug class before closure.

Deliverable:
- [x] Current UI bugs stabilized before advancing later stages.

## Stage 17: Query-Safety Hardening
- [x] Audit all UI systems for overlapping mutable query risk.
- [x] Apply `ParamSet` and `Without<T>` disjointness contracts where needed.
- [x] Add concise query contract comments on multi-query systems.
- [x] Verify no B0001 panic paths remain.

Deliverable:
- [x] B0001-safe UI query architecture.

## Stage 18: Test Coverage Expansion
- [x] Add/extend unit tests for primitive reducers and state transitions.
- [x] Add integration tests for tabs, dropdowns, selectors/cyclers, modals, scrollbars, and layer gating.
- [x] Add debug showcase smoke hooks for interaction and query safety.
- [x] Add owner-scoped stress tests for layered coexistence.

Deliverable:
- [x] Regression-safe primitive + composition test suite.

## Stage 19: Runtime Stress Validation
- [x] Run GPU-capable stress passes across main/options/video/dropdown/modal/pause paths.
- [x] Execute rapid mixed keyboard + mouse interaction scripts.
- [x] Capture logs with backtraces and enforce no-panic/no-B0001 acceptance.
- [x] Patch any discovered race and re-run until clean.

Deliverable:
- [x] Stable runtime interaction under stress.

## Stage 20: Documentation and Adoption
- [x] Document primitive APIs and composition recipes (`clickable`, `selectable_menu`, layer manager, dropdown, tabs, scroll, hover box).
- [x] Add do/don't examples and extension guides.
- [x] Mark migration targets and deprecated patterns clearly.
- [x] Ensure docs reflect actual code and boundaries.

Deliverable:
- [x] Up-to-date UI architecture and implementation docs.

## Stage 21: Tooling and Test Framework Rollout
- [x] Add `mdBook` coverage for UI architecture and extension playbook.
- [x] Expand rustdoc for UI primitives/contracts.
- [x] Add/validate `cargo-nextest` setup.
- [x] Evaluate `rstest`/`proptest` adoption and retain deterministic sampled property coverage for now.

Deliverable:
- [x] Faster, clearer, repeatable UI validation workflow.

## Stage 22: Cleanup and Redundancy Pass
- [x] Remove dead/redundant UI code paths.
- [x] Consolidate duplicated helpers across menu/tab/dropdown/scroll paths.
- [x] Re-run compile/test to confirm no behavior regressions.
- [x] Final readability pass on module boundaries and naming.

Deliverable:
- [x] Clean, minimal, composable UI codebase.

## Final Acceptance Checklist
- [x] Primitive-root insertion is sufficient for each major UI feature.
- [x] No new reusable bundle-first APIs were introduced.
- [x] Layering and input arbitration are deterministic and owner-scoped.
- [x] No known B0001 query conflicts in UI systems.
- [x] Debug showcase windows are fully interactive and primitive-backed.
- [x] Menu, dropdown, tab, selector, slider, scroll, hover, and modal flows are regression-tested.
