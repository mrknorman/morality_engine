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

## Stage 0: Safety + Checkpoint
- [ ] Create a clean checkpoint commit before functional changes.
- [ ] Record baseline behavior notes for main menu, pause menu, options/video, dropdowns, tabs, modals, debug showcase.
- [ ] Confirm current compile status and capture baseline command outputs.

## Stage 1: Audit and Classification
- [ ] Inventory all UI modules under `src/systems/ui/*` and `src/systems/ui/menu/*`.
- [ ] Classify each as primitive-compliant, partially compliant, or non-compliant.
- [ ] Produce migration table: owner file, current construction pattern, target primitive pattern.
- [ ] Identify all remaining bundle-first reusable APIs to migrate.

Deliverable:
- [ ] Compliance matrix and migration backlog.

## Stage 2: Architecture Boundaries and Contracts
- [ ] Reconfirm strict module boundaries:
  - `systems/ui/*` = reusable primitives only
  - `systems/ui/menu/*` = composition/policy only
  - scenes/startup = consumers
- [ ] Reconfirm dependency direction and enforce it in code organization.
- [ ] Reconfirm owner/root identity conventions for all layered UI elements.
- [ ] Reconfirm query-safety standards (`ParamSet`/`Without`) as mandatory contract.

Deliverable:
- [ ] Updated architecture contract doc aligned with actual module boundaries.

## Stage 3: Owner-Scoped Interaction Context + Layer Manager
- [ ] Ensure all interaction gates resolve by owner, never globally.
- [ ] Centralize active-layer resolution (`Base`, `Dropdown`, `Modal`) by owner.
- [ ] Remove ad-hoc layer scans in menu/tab/dropdown/modal systems.
- [ ] Route dimming/focus/interaction decisions through one layer source-of-truth.

Deliverable:
- [ ] Deterministic owner-scoped layer arbitration across all UI surfaces.

## Stage 4: Primitive Contract Normalization
- [ ] Standardize root primitives and contracts for:
  - menu surface
  - selector/cycler surface
  - tab bar
  - dropdown surface/state
  - scrollable root + scrollbar
  - hover box
  - discrete slider
- [ ] Ensure each primitive owns required child hierarchy via insert/lifecycle hooks.
- [ ] Remove hidden external wiring assumptions from primitive behavior systems.

Deliverable:
- [ ] Single-root primitive insertion stands up each primitive behavior unit.

## Stage 5: Menu Composition Migration
- [ ] Refactor `src/systems/ui/menu/*` to compose primitives only.
- [ ] Remove menu-specific primitive reimplementations.
- [ ] Keep reducer/effects split while moving mechanics to primitives.
- [ ] Reduce feature-specific branching in generic menu flow paths.

Deliverable:
- [ ] Menu modules are policy + command mapping only.

## Stage 6: UI Module Realignment
- [ ] Ensure menu modules live under `ui::menu` with clean public API boundaries.
- [ ] Remove or avoid transitional re-export shims.
- [ ] Keep owner/layer/interaction contracts intact after module cleanup.

Deliverable:
- [ ] Stable module topology aligned with architecture contract.

## Stage 7: Dropdown, Tabs, and Footer Primitive Unification
- [ ] Keep dropdown open/close/single-visible/outside-click logic fully in reusable dropdown primitive.
- [ ] Keep tab selection/activation/arbitration in reusable tab primitive path.
- [ ] Keep horizontal footer navigation reusable and composition-driven.
- [ ] Ensure independent owners can host tabs/dropdowns without cross-talk.

Deliverable:
- [ ] Shared dropdown/tab/footer primitives used consistently by menu composition.

## Stage 8: Command Reducer + Effects Split
- [ ] Keep pure reducer transitions separate from Bevy side effects.
- [ ] Ensure command side effects are centralized and deterministic.
- [ ] Keep behavior compatibility with existing flows during migration.

Deliverable:
- [ ] Reducer/effects architecture with clear contracts and tests.

## Stage 9: Deterministic Input Arbitration
- [ ] Enforce strict priority: layer > focus group > keyboard lock > hover.
- [ ] Remove first-match query-iteration dependence.
- [ ] Ensure one owner-level system decides selection priority.
- [ ] Stabilize behavior under rapid mixed keyboard/mouse interaction.

Deliverable:
- [ ] No selection jitter or nondeterministic ownership conflicts.

## Stage 10: Main Menu Composition Migration
- [ ] Move main menu option list fully to shared menu composition path.
- [ ] Remove scene-local duplicate menu behavior.
- [ ] Reuse shared navigation audio + selection behavior paths.

Deliverable:
- [ ] Main menu uses same composition system as other UI menus.

## Stage 11: JSON Menu/Settings Schema Interface
- [ ] Define JSON schema for menu structure (title, hint, options, shortcuts, layout bindings).
- [ ] Implement typed command registry bridge (`string id -> typed Rust handler`).
- [ ] Add explicit validation failures (no silent fallback).
- [ ] Migrate one menu as pilot and evaluate extension cost.

Deliverable:
- [ ] Validated schema-driven menu composition path.

## Stage 12: Discrete Slider Primitive and Integration
- [ ] Implement/normalize reusable `DiscreteSlider` primitive (keyboard + mouse).
- [ ] Integrate into appropriate rankable options (off/low/medium/high patterns).
- [ ] Ensure selector and slider interaction do not conflict.
- [ ] Ensure slider behavior is owner/layer safe and composable.

Deliverable:
- [ ] Stable slider primitive adopted in settings UI.

## Stage 13: Scrollable RTT Primitive

### Stage 13.1: Architecture Contract
- [ ] Define reusable primitives (`ScrollableRoot`, `ScrollableViewport`, `ScrollableContentCamera`, `ScrollableRenderTarget`, `ScrollableItem`, `ScrollState`).
- [ ] Keep owner-scoped integration with `UiLayer.owner` and `InteractionGate`.
- [ ] Keep backend enum extensible (`RenderToTexture` first-class).

### Stage 13.2: Render Target + Camera Lifecycle
- [ ] Add per-root render-target allocation/pooling.
- [ ] Spawn dedicated content camera per root targeting RTT.
- [ ] Assign dedicated render layers for scroll content.
- [ ] Handle viewport resize/rebuild safely.

### Stage 13.3: Viewport Surface Composition
- [ ] Render scroll output as clipped world-space surface.
- [ ] Guarantee no overdraw beyond viewport bounds.
- [ ] Keep stable visual ordering with text/borders/glow and CRT pipeline.

### Stage 13.4: Scroll Reducer and Motion Model
- [ ] Implement pure scroll state reducer (`offset`, `content`, `viewport`, `max`, optional velocity/snap).
- [ ] Support wheel, keyboard step/page/home/end, thumb drag, focus-follow intents.
- [ ] Keep deterministic clamping and ordering under mixed inputs.

### Stage 13.5: Input Mapping on RTT Content
- [ ] Map cursor viewport-space to content-space deterministically.
- [ ] Resolve hovered/pressed rows by stable index/key.
- [ ] Keep keyboard semantics aligned with menu systems.
- [ ] Keep row/name/value click parity with non-scroll contexts.

### Stage 13.6: Reusable Adapters
- [ ] Implement `ScrollableTableAdapter` for table menus.
- [ ] Implement `ScrollableListAdapter<T>` for generic lists.
- [ ] Add focus-follow hooks to keep selected option visible.

### Stage 13.7: Performance and Safety Hardening
- [ ] Add texture budget controls and fallback policies.
- [ ] Ensure camera/target cleanup on despawn.
- [ ] Enforce query disjointness contracts in all scroll systems.

### Stage 13.8: Validation and Rollout
- [ ] Add reducer/coordinate unit tests.
- [ ] Add integration tests for mixed scroll + selection + dropdown/modal layering.
- [ ] Roll out first to video options, then at least one secondary context.

Deliverable:
- [ ] Reusable scroll primitive with stable interaction in multi-layer UI.

## Stage 14: HoverBox Primitive + Video Pilot
- [ ] Define reusable `HoverBox` primitive API in `src/systems/ui/hover_box.rs`.
- [ ] Add owner-scoped/layer-scoped hover arbitration contract.
- [ ] Add delay behavior (`0.5s`) with deterministic show/hide transitions.
- [ ] Add anchored placement below target + bounds-safe clamping.
- [ ] Add style/config components (`HoverBoxStyle`, `HoverBoxContent`).
- [ ] Integrate option-name descriptions in video menu.
- [ ] Integrate dropdown value descriptions (e.g. tonemapper), excluding resolution values.
- [ ] Add regression tests for timing, gating, mapping, and exclusions.
- [ ] Validate behavior under mixed keyboard/mouse with overlays.

Deliverable:
- [ ] Reusable hover tooltip primitive used by menu composition.

## Stage 15: Debug UI Showcase Rebuild
- [ ] Move debug showcase to dedicated composition module.
- [ ] Build interactive windows from real primitives (selector, tabs, dropdown, scroll, hover box).
- [ ] Remove visual-only or one-off showcase interaction logic.
- [ ] Ensure debug demos mirror production primitive wiring.

Deliverable:
- [ ] Debug showcase acts as live primitive reference implementation.

## Stage 16: Known Bug Sprint (Pre-Refactor and Mid-Refactor)
- [ ] Fix value-cell interaction dead zones and hover/click mismatches.
- [ ] Fix keyboard transfer bugs between tabs/footer/options.
- [ ] Fix dropdown flicker/jitter and alignment regressions.
- [ ] Fix modal/input gating and layering regressions.
- [ ] Add regression tests for each bug class before closure.

Deliverable:
- [ ] Current UI bugs stabilized before advancing later stages.

## Stage 17: Query-Safety Hardening
- [ ] Audit all UI systems for overlapping mutable query risk.
- [ ] Apply `ParamSet` and `Without<T>` disjointness contracts where needed.
- [ ] Add concise query contract comments on multi-query systems.
- [ ] Verify no B0001 panic paths remain.

Deliverable:
- [ ] B0001-safe UI query architecture.

## Stage 18: Test Coverage Expansion
- [ ] Add/extend unit tests for primitive reducers and state transitions.
- [ ] Add integration tests for tabs, dropdowns, selectors/cyclers, modals, scrollbars, and layer gating.
- [ ] Add debug showcase smoke hooks for interaction and query safety.
- [ ] Add owner-scoped stress tests for layered coexistence.

Deliverable:
- [ ] Regression-safe primitive + composition test suite.

## Stage 19: Runtime Stress Validation
- [ ] Run GPU-capable stress passes across main/options/video/dropdown/modal/pause paths.
- [ ] Execute rapid mixed keyboard + mouse interaction scripts.
- [ ] Capture logs with backtraces and enforce no-panic/no-B0001 acceptance.
- [ ] Patch any discovered race and re-run until clean.

Deliverable:
- [ ] Stable runtime interaction under stress.

## Stage 20: Documentation and Adoption
- [ ] Document primitive APIs and composition recipes (`clickable`, `selectable_menu`, layer manager, dropdown, tabs, scroll, hover box).
- [ ] Add do/don't examples and extension guides.
- [ ] Mark migration targets and deprecated patterns clearly.
- [ ] Ensure docs reflect actual code and boundaries.

Deliverable:
- [ ] Up-to-date UI architecture and implementation docs.

## Stage 21: Tooling and Test Framework Rollout
- [ ] Add `mdBook` coverage for UI architecture and extension playbook.
- [ ] Expand rustdoc for UI primitives/contracts.
- [ ] Add/validate `cargo-nextest` setup.
- [ ] Add `rstest` and/or `proptest` where property tests add value.

Deliverable:
- [ ] Faster, clearer, repeatable UI validation workflow.

## Stage 22: Cleanup and Redundancy Pass
- [ ] Remove dead/redundant UI code paths.
- [ ] Consolidate duplicated helpers across menu/tab/dropdown/scroll paths.
- [ ] Re-run compile/test to confirm no behavior regressions.
- [ ] Final readability pass on module boundaries and naming.

Deliverable:
- [ ] Clean, minimal, composable UI codebase.

## Final Acceptance Checklist
- [ ] Primitive-root insertion is sufficient for each major UI feature.
- [ ] No new reusable bundle-first APIs were introduced.
- [ ] Layering and input arbitration are deterministic and owner-scoped.
- [ ] No known B0001 query conflicts in UI systems.
- [ ] Debug showcase windows are fully interactive and primitive-backed.
- [ ] Menu, dropdown, tab, selector, slider, scroll, hover, and modal flows are regression-tested.
