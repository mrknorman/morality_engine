# UI Scrollable Usage Guide

## Purpose
`ScrollableRoot` and `ScrollBar` provide a reusable render-to-texture clipping primitive for UI content that must overflow a viewport while keeping owner-scoped interaction/layer arbitration intact.

Primary module:
- `src/systems/ui/scroll/mod.rs`
- `src/systems/ui/scroll/lifecycle.rs`
- `src/systems/ui/scroll/behavior.rs`
- `src/systems/ui/scroll/scrollbar.rs`

Menu adapter example:
- `src/systems/ui/menu/scroll_adapter.rs`

## Core Components
Attach these to a scroll root entity:
1. `ScrollableRoot { owner, axis, backend, input_layer }`
2. `ScrollableViewport { size }`
3. `ScrollState` (auto-required by `ScrollableRoot`)
4. Optional `ScrollableContentExtent` for explicit content size

Optional explicit child content root:
1. `ScrollableContent`
   - `ScrollPlugin` now auto-seeds a default `ScrollableContent` child if one is missing.
   - Add your own explicit `ScrollableContent` child when you want deterministic naming/placement during composition.

Attach this to optional scrollbar entity (can be a child of the root or auto-reparented):
1. `ScrollBar::new(scrollable_root_entity)`

Attach this to scroll items when you want automatic extent aggregation:
1. `ScrollableItem { key, index, extent }`

## Ownership and Layering Contract
1. `ScrollableRoot.owner` must match the owner used by `UiLayer`.
2. Scroll input is accepted only when the owner’s active layer matches `ScrollableRoot.input_layer`.
3. For base panel scrolling, use `ScrollableRoot::new(...).with_input_layer(UiLayerKind::Base)`.
4. For modal-scoped scrolling, set `with_input_layer(UiLayerKind::Modal)`.
5. Interaction gates are respected through `InteractionGate` checks.

## Render-Target Configuration
1. `ScrollPlugin` initializes `ScrollRenderSettings`.
2. `ScrollRenderSettings.target_format` controls RTT texture format (default `Rgba16Float`).
3. RTT images are recreated when viewport size or target format changes.

## Integration Recipe (Reusable)
1. Spawn a root entity with `ScrollableRoot` + `ScrollableViewport`.
2. Optionally spawn an explicit `ScrollableContent` child entity (or rely on auto-seeded default).
3. Parent overflow content under the `ScrollableContent` child.
4. Add `ScrollableContentExtent` or `ScrollableItem` entries to define content size.
5. Add `ScrollBar` if a visual scrollbar is needed.
6. Ensure your owning UI root participates in `UiLayer` arbitration.

## Module Responsibilities
1. `scroll/lifecycle.rs`:
   - Render target allocation/sync
   - Camera/surface runtime entity management
   - Scroll content render-layer synchronization
2. `scroll/behavior.rs`:
   - Extent aggregation (`ScrollableItem`/`ScrollableContentExtent`)
   - Wheel/keyboard/edge-zone input reduction
   - Content transform offset application
3. `scroll/scrollbar.rs`:
   - Scrollbar part composition
   - Thumb/track visuals and click regions
   - Drag/track-click input behavior
4. `scroll/geometry.rs` + `scroll/scrollbar_math.rs`:
   - Pure helper math shared by systems/tests

## Menu Adapter Pattern
Keep menu logic out of the primitive:
1. Put menu-specific focus-follow in `systems/ui/menu/*`.
2. Store adapter metadata (`menu_entity`, row extents/count).
3. In menu `PostCommands`, adjust `ScrollState.offset_px` to keep selected row visible.

Reference implementation:
- `src/systems/ui/menu/scroll_adapter.rs`
- `src/systems/ui/menu/page_content.rs`

## Query-Safety Rules
When adding systems around `Scrollable`:
1. Use `Without<T>` for disjoint mutable queries over shared components.
2. Use `ParamSet` when a system needs both read and write access paths to the same component.
3. Keep state reducers and visual sync in separate stages where possible.
4. Add a minimal plugin update test for panic detection.
5. Prefer owner-scoped layer resolution (`active_layers_by_owner_scoped`) for all input paths.

Current guard:
- `systems::ui::scroll::tests::scroll_plugin_update_is_query_safe`

## Testing Checklist for New Contexts
1. Base layer accepts wheel/keyboard scrolling.
2. Dropdown/modal layers block scrolling for same owner.
3. Scroll offset stays clamped under rapid layer toggles.
4. No B0001 panics under repeated `app.update()` in test harness.
5. Existing menu navigation, dropdown, selector, and slider tests still pass.
