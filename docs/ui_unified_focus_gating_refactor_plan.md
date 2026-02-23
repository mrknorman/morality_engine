# Unified UI Focus + Action Gating Refactor Plan

## Goal

Replace the current split interaction model (`InteractionGate`, capture markers, per-feature focus logic) with one unified input/focus/gating system that is:

- owner-scoped
- layer-aware
- deterministic for keyboard and mouse arbitration
- reusable across menus, windows, dropdowns, tabs, scrollables, and hover systems

This is a full replacement. No compatibility layer is used.

## Why This Refactor

The current system works but duplicates routing logic in multiple places:

- interaction primitives (`clickable/selectable/draggable`)
- menu layer routing and tab focus systems
- scroll and hover arbitration
- window close/drag behavior

This causes drift and brittle behavior when new UI is added.

## Non-Negotiable Design Constraints

1. No dual-path support. Old gate/capture APIs are removed, not wrapped.
2. Owner-first routing. Input resolution is explicit per owner scope.
3. Layer-first arbitration inside owner (`Modal > Dropdown > Base`).
4. One focus source of truth computed once per frame.
5. Visual state is downstream only; not behavioral truth.
6. Required-components and `on_insert` hooks stay preferred construction style.

## Target Architecture

### Core Types

Introduce new interaction model types in `src/systems/interaction/mod.rs`:

1. `UiInputMode`
   - `World`
   - `Captured`

2. `UiInputCapture`
   - resource containing active captures (owner + mode + rank metadata)
   - authoritative capture state; replaces `InteractionCapture` and `InteractionCaptureOwner`

3. `UiInputPolicy` (component)
   - allowed input modes
   - optional owner scope
   - optional layer scope (`UiLayerKind`)
   - optional focus requirement

4. `UiFocusScope` (component)
   - owner entity for focus routing

5. `UiInteractionState` (resource, recomputed each frame)
   - active mode
   - active layer per owner
   - focused owner
   - focused entity per owner/layer where needed

### Resolution Pipeline

Single resolver pass early in update:

1. Resolve input mode from app state + captures.
2. Resolve active layer per owner (visibility + policy filtered).
3. Resolve focused owner deterministically by z/entity rank.
4. Publish immutable `UiInteractionState`.

All feature systems consume this state and never re-derive capture logic.

## Migration Plan (Phased)

## Execution Protocol (Mandatory)

1. Phase-by-phase checkpoint commits are required.
   - At the end of every phase, create one commit that captures the full phase result.
   - Do not start the next phase until that commit exists.
2. Documentation is updated continuously, not deferred.
   - For each phase, update relevant UI documentation in `docs/` to reflect new behavior/API.
   - Keep docs aligned with implementation decisions at phase boundary.
3. No compatibility layer is permitted.
   - Old APIs/components are removed, not wrapped.
4. Compile health gate per phase.
   - `cargo check` must pass at phase boundary unless the phase is explicitly marked as
     an in-progress migration checkpoint.

### Phase 0: Baseline and Safety

1. Tag baseline in commit history (already checkpointed).
2. Add temporary migration checklist comments in affected modules.
3. Add explicit test matrix doc section (menu, window, scroll, modal, dropdown, tabs).
4. Add phase execution notes to UI docs indicating this is a full replacement with no bridge.

Deliverable:
- clean compile before intrusive edits.

### Phase 1: Replace Core Interaction Types

Files:
- `src/systems/interaction/mod.rs`

Actions:
1. Add new core types/resources/components listed above.
2. Remove `InteractionGate`, `InteractionCapture`, `InteractionCaptureOwner`.
3. Remove helper APIs:
   - `interaction_context_active*`
   - `interaction_gate_allows*`
4. Update required-components on primitives:
   - `Clickable`, `Pressable`, `Selectable`, `SelectableMenu`, `Draggable`
   to use `UiInputPolicy` / `UiFocusScope` as appropriate.
5. Update docs for new interaction primitives and removed legacy gates/capture markers.

Deliverable:
- interaction module compiles with new APIs only.

### Phase 2: Unified Resolver System

Files:
- `src/systems/interaction/mod.rs`
- `src/systems/ui/layer.rs`

Actions:
1. Move layer arbitration into unified resolver.
2. Ensure `UiInteractionState` stores:
   - active mode
   - active owners/layers
   - focused owner
3. Keep deterministic tie-breaking rules centralized.
4. Update docs describing the resolver pipeline and state ownership rules.

Deliverable:
- one frame-level source of truth for gating and focus.

### Phase 3: Migrate Interaction Primitive Systems

Files:
- `src/systems/interaction/mod.rs`

Actions:
1. Update `hoverable_system`, `clickable_system`, `pressable_system`, `selectable_system`, `Draggable::enact`.
2. Replace all gate/capture lookups with `UiInteractionState + UiInputPolicy`.
3. Preserve existing keyboard-lock semantics and click activation policies.
4. Update docs for primitive behavioral contracts and required components.

Deliverable:
- primitive interactions behaviorally equivalent under new model.

### Phase 4: Migrate Menu Stack + Layered UI

Files:
- `src/systems/ui/menu/mod.rs`
- `src/systems/ui/menu/menu_input.rs`
- `src/systems/ui/menu/tabbed_menu.rs`
- `src/systems/ui/menu/modal_flow.rs`
- `src/systems/ui/menu/dropdown_flow.rs`
- `src/systems/ui/menu/command_flow.rs`
- `src/systems/ui/menu/level_select.rs`

Actions:
1. Replace all `capture_query`/gate filtering with `UiInteractionState`.
2. Ensure menu active-owner ordering comes from unified state.
3. Keep tabbed focus transitions intact but remove duplicated owner gating.
4. Update menu and layered UI docs with new routing model.

Deliverable:
- menus, modals, dropdowns, tabs all routed by same focus/gating engine.

### Phase 5: Migrate Window and Close/Drag Routing

Files:
- `src/systems/ui/window/mod.rs`

Actions:
1. Replace gate propagation hacks with explicit `UiInputPolicy` inheritance.
2. Ensure close button, drag region, resize handles obey focused owner and active layer.
3. Verify window interactions never affect unfocused owners via keyboard.
4. Update window interaction docs with owner/focus/gating semantics.

Deliverable:
- stable window interaction model without per-feature gate patches.

### Phase 6: Migrate Scroll + Hover Subsystems

Files:
- `src/systems/ui/scroll/behavior.rs`
- `src/systems/ui/hover_box.rs`

Actions:
1. Remove independent focus-owner resolution.
2. Use unified focused owner and active layer only.
3. Keep nested scroll behavior deterministic and owner-scoped.
4. Update scroll/hover docs with focus and routing contract changes.

Deliverable:
- scroll/hover arbitration consistent with menus/windows.

### Phase 7: Migrate Remaining Gate Consumers

Files:
- `src/systems/colors/mod.rs`
- `src/systems/cascade/mod.rs`
- `src/startup/pause.rs`
- `src/startup/debug.rs`
- any remaining `InteractionGate` usage found by ripgrep

Actions:
1. Replace all legacy gating checks with policy/state checks.
2. Remove all legacy component insertions from spawners.
3. Update any remaining module docs that referenced legacy gate/capture APIs.

Deliverable:
- zero references to legacy gate/capture APIs.

### Phase 8: Cleanup + Docs + Tests

Files:
- `docs/ui_ecs_reference.md`
- `docs/ui_scrollable_usage.md`
- new/updated tests in interaction/menu/window/scroll modules

Actions:
1. Remove dead code and stale comments.
2. Update docs with unified model and authoring rules.
3. Add tests for:
   - owner focus precedence
   - layer precedence
   - keyboard vs mouse arbitration
   - modal/dropdown blocking
   - window close/drag correctness
   - no cross-owner keyboard leakage

Deliverable:
- clean docs and test coverage for core routing guarantees.

## Acceptance Criteria

1. No `InteractionGate`, `InteractionCapture`, or compatibility aliases remain.
2. `cargo check` passes; menu/window/interaction test suites pass.
3. Behavior matches expectations in:
   - main menu
   - pause menu
   - options video tabs
   - dropdowns
   - modals
   - level select overlay
   - debug windows
   - scrollbars and hover boxes
4. Keyboard controls only affect focused owner scope where expected.
5. Mouse hover/click arbitration remains deterministic and stable under overlap.

## Implementation Order Notes

1. Do Phases 1-3 in one contiguous sequence to avoid partial API states.
2. Then migrate by vertical slice (menu, window, scroll, hover).
3. Keep each phase as a dedicated commit with compile green at phase boundary.

## Risk Areas and Mitigations

1. Risk: focus regressions in tabbed video menu.
   - Mitigation: preserve existing tabbed reducer logic and swap only data source.
2. Risk: window close/drag conflicts.
   - Mitigation: explicit policy inheritance and focused-owner checks in one place.
3. Risk: scroll owner arbitration changes.
   - Mitigation: port existing deterministic ranking rules directly into unified resolver.
4. Risk: ECS query conflicts (`B0001`) during migration.
   - Mitigation: keep disjoint queries + `ParamSet` where mutable query roles overlap.

## Completion Checklist

- [x] Phase 0 complete
- [x] Phase 1 complete
- [x] Phase 2 complete
- [x] Phase 3 complete
- [ ] Phase 4 complete
- [ ] Phase 5 complete
- [ ] Phase 6 complete
- [ ] Phase 7 complete
- [ ] Phase 8 complete
