# Scrollable + ScrollBar RTT Plan

Last updated: 2026-02-19  
Status: planning only (no implementation in this document)

## Goal
Deliver a robust, reusable, composable scrolling system that supports multiple UI contexts using render-to-texture clipping, with a reusable `Scrollable` primitive and a reusable `ScrollBar` primitive.

## Scope Constraints
1. Must integrate with existing owner-scoped UI layering (`UiLayer`) and interaction gating (`InteractionGate`).
2. Must preserve deterministic input arbitration (keyboard + mouse) and avoid query aliasing panics (`B0001`).
3. Must remain composable outside video/options menus (future lists/panels/windows).
4. Must avoid regressions in current menu systems, dropdowns, tabs, selectors, and sliders.

## Current State Review (Baseline)
From the current codebase:
- UI primitives live in `src/systems/ui/` (`dropdown`, `layer`, `tabs`, `selector`, `discrete_slider`, `menu/*`).
- Menu orchestration is modularized in `src/systems/ui/menu/` with clear system sets (`Core`, `Commands`, `PostCommands`, `Visual`).
- Interaction pipeline is centralized in `src/systems/interaction/mod.rs` with generic `Clickable<T>`, `SelectableMenu`, and owner-aware gate checks.
- Rendering already uses an offscreen -> fullscreen post-process path in `src/startup/render.rs` (CRT pipeline with `RenderLayers` 0/1).

Implication:
- We should add a dedicated scroll RTT path that composes with current world/UI render flow, without coupling scroll behavior to menu-specific logic.

## Phase 1: UI/Menu Pre-Cleanup and Hardening
Purpose: ensure clean baseline before introducing scroll RTT complexity.

Execution status: completed on 2026-02-19.

Completed checklist:
- [x] Dedicated primitive boundary created at `src/systems/ui/scroll/mod.rs`.
- [x] Menu scheduling integration points documented in `src/systems/ui/menu/mod.rs`.
- [x] Owner/layer and query-safety contracts captured in a stage report:
  `docs/scrollable_stage1_report.md`.
- [x] No menu-specific scroll logic added to primitive boundary module.

1. Review and tighten boundaries
- Keep `Scrollable` and `ScrollBar` in a generic module (`src/systems/ui/scroll.rs` or `src/systems/ui/scroll/*`).
- Keep menu-specific adapters in `src/systems/ui/menu/` (no menu logic in primitive module).

2. Query-safety and ownership contracts
- Add/confirm explicit query contracts where scroll will read/write shared components (`Visibility`, `Transform`, `Clickable`, `Selectable`).
- Predefine `ParamSet`/`Without` use in the planned systems.

3. Deterministic interaction arbitration hooks
- Define where scroll input is resolved in relation to existing order:
  - after pointer state updates
  - before menu command execution
  - owner/layer filtered (`Modal > Dropdown > Base`).

4. Integration checklist before coding
- Confirm system insertion points in `MenusPlugin` and interaction pipeline.
- Confirm no reliance on global singleton scroll state.

Exit criteria:
- Primitive boundaries documented.
- System ordering and ownership contracts declared.
- No pending unresolved conflicts in UI/menu modules for planned insertion.

## Phase 2: Clean Commit Gate (Required Before Implementation)
Create a clean checkpoint commit immediately before implementation begins.

Execution status: completed on 2026-02-19.

Completed checklist:
- [x] Verified baseline changed files were intentional Stage 1 artifacts.
- [x] Ran baseline verification:
  - `cargo check --manifest-path Cargo.toml`
  - `cargo test --manifest-path Cargo.toml -- --nocapture`
- [x] Created checkpoint commit:
  - `Checkpoint: pre-scrollable-rtt implementation baseline`
- [x] Re-verified clean working tree after checkpoint.

Checklist:
1. Ensure working tree is clean (`git status`).
2. Run baseline verification:
- `cargo check --manifest-path Cargo.toml`
- `cargo test --manifest-path Cargo.toml -- --nocapture`
3. Commit with clear marker message, e.g.:
- `Checkpoint: pre-scrollable-rtt implementation baseline`

Exit criteria:
- Clean, reproducible baseline commit exists.

## Phase 3: Implement `Scrollable` (Render-to-Texture)
Implement as reusable primitive independent of menu semantics.

### 3.1 Components/Resources
- `ScrollableRoot { owner: Entity, axis: ScrollAxis, backend: ScrollBackend }`
- `ScrollableViewport { size: Vec2 }`
- `ScrollableContent`
- `ScrollableItem { key: u64, index: usize, extent: f32 }`
- `ScrollState { offset_px, content_extent, viewport_extent, max_offset, velocity, snap_mode }`
- `ScrollableRenderTarget { image: Handle<Image>, size_px: UVec2 }`
- `ScrollableContentCamera`
- `ScrollableSurface` (sprite/mesh displaying render target)
- `ScrollInputLock` (optional owner-local lock metadata)

### 3.2 Systems (Primitive)
- Lifecycle:
  - allocate/update render target per scroll root
  - spawn/despawn content camera and surface
  - resize target on viewport size change
- State:
  - compute extents
  - reduce intents into `ScrollState` (wheel, keyboard step/page/home/end, focus-follow)
  - clamp/snap/inertia update
- Rendering:
  - apply content offset transform
  - map content camera target and render layer
  - ensure z-order and visibility contracts

### 3.3 Render Layers + Pipeline Integration
- Reserve a dedicated layer range for scroll content/cameras (documented constants).
- Keep compatibility with existing CRT/post-process pipeline:
  - scroll surface is rendered as regular world UI element
  - avoid introducing additional global post-processing passes.

### 3.4 Input Mapping
- Convert cursor position viewport-local -> content-local.
- Resolve hovered item deterministically by stable index/key.
- Route enter/click activation through existing `Clickable<SystemMenuActions>` patterns.

Exit criteria:
- Reusable `Scrollable` primitive works for one vertical list with keyboard + wheel + click.
- Content is truly clipped by RTT viewport bounds.
- No menu-specific assumptions in primitive.

## Phase 4: Implement `ScrollBar` Primitive
Implement scrollbar as separate, composable primitive that can attach to any `ScrollableRoot`.

### 4.1 Components
- `ScrollBarRoot { scrollable_root: Entity }`
- `ScrollTrack`
- `ScrollThumb`
- `ScrollBarStyle { width, min_thumb_size, colors, margins }`
- `ScrollBarInputState { dragging, drag_anchor }`

### 4.2 Behavior
- Compute thumb size/position from `ScrollState` ratios.
- Support:
  - thumb drag
  - track click page jumps
  - optional hover/pressed visuals via existing interaction palette patterns.
- Keep scrollbar logic independent from menu option commands.

### 4.3 Arbitration
- Scrollbar input only active for the owner’s active layer.
- Drag lock must prevent conflicting menu navigation while dragging.

Exit criteria:
- `ScrollBar` can be attached/detached without modifying core `Scrollable`.
- Correct sync in both directions (`ScrollState` <-> thumb).

## Phase 5: Menu Adapter Integration
Use thin adapters to connect menu/table rows to `Scrollable`.

1. Create adapter in `src/systems/ui/menu/`:
- `ScrollableTableAdapter` for top options panel.
- stable row key/index mapping.

2. Preserve existing menu semantics:
- keyboard selection remains menu-driven.
- focus-follow scroll keeps selected row visible.
- dropdown/modal layer behavior unchanged.

3. Pilot context:
- Video options middle panel first.

Exit criteria:
- Video menu can handle more rows than viewport without overlap/regression.
- Existing tabs/footer/dropdown/modal behavior remains correct.

## Phase 6: Testing Plan (with ScrollBar Focus)
### 6.1 Unit Tests (`Scrollable`)
- reducer clamp/wrap behavior
- extent and `max_offset` math
- focus-follow math
- deterministic item resolution for pointer mapping

### 6.2 Unit Tests (`ScrollBar`) - required
- thumb size calculation (min-size and proportional cases)
- thumb position mapping to/from offset
- drag updates offset correctly
- track click page movement
- bounds clamping under resize/content change

### 6.3 Integration/Regression Tests
- mixed keyboard + mouse wheel input order determinism
- layer gating with dropdown/modal overlays
- no `B0001` query panics in stress-like simulation sequences
- no regressions in selector/slider/dropdown interaction

### 6.4 Runtime Validation Checklist
- `cargo check --manifest-path Cargo.toml`
- `cargo test --manifest-path Cargo.toml -- --nocapture`
- manual interactive pass in menu contexts with rapid mixed input

## Phase 7: Rollout and Stabilization
1. Pilot in Video options panel.
2. Patch issues from pilot.
3. Promote as reusable primitive for additional contexts (other menus/windows/lists).
4. Document extension recipe and contracts.

## Deliverables
1. Reusable `Scrollable` RTT primitive.
2. Reusable `ScrollBar` primitive.
3. Menu adapter for Video options panel.
4. ScrollBar-focused tests + regression suite updates.
5. Updated architecture docs for usage and extension.
