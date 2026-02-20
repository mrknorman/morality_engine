# UI Compliance Matrix

Last updated: 2026-02-20

Scope:
- `src/systems/ui/*`
- `src/systems/ui/menu/*`

Legend:
- `Compliant`: matches current ECS contract (`root component`, `#[require]`, `on_insert` where applicable, owner/layer scoping, no bundle-first reusable API).
- `Partial`: mostly aligned but still has composition drift, ad-hoc wiring, or migration debt.
- `Non-compliant`: violates target architecture and should be migrated first.

## Primitive Modules (`src/systems/ui/*`)

| Module | Status | Current Pattern | Migration Target |
|---|---|---|---|
| `layer.rs` | Compliant | Owner-scoped layer resolution and arbitration. | Keep as canonical layer truth. |
| `menu_surface.rs` | Compliant | Root component + required contracts. | None. |
| `selector.rs` | Compliant | `SelectorSurface` `on_insert` wiring for `Selectable`/`OptionCycler`. | None. |
| `tabs.rs` | Compliant | `TabBar` primitive with required state and sync systems. | Keep selection/activation semantics here. |
| `dropdown.rs` | Compliant | Reusable dropdown root/state with owner-scoped behavior. | Keep all generic open/close logic here. |
| `hover_box.rs` | Compliant | Primitive root + explicit post-spawn marker attachment at callsites. | None. |
| `discrete_slider.rs` | Compliant | Primitive root + insert hook builds internal slot hierarchy. | None. |
| `scroll/mod.rs` + `scroll/*` | Partial | Reusable RTT scroll primitive and scrollbar with strong tests. | Complete remaining live-flow validation and nested-context polish. |

## Menu Composition Modules (`src/systems/ui/menu/*`)

| Module | Status | Current Pattern | Migration Target |
|---|---|---|---|
| `command_reducer.rs` | Compliant | Pure transition reducer, test-covered. | Keep pure state transitions here only. |
| `command_effects.rs` | Compliant | Side-effect boundary for reducer outputs. | Keep deterministic effect dispatch. |
| `command_flow.rs` | Compliant | Reducer + effects orchestration. | None. |
| `stack.rs` | Compliant | Menu stack/navigation state model. | None. |
| `schema.rs` | Partial | Typed schema parsing exists with command registries; now used by both main menu and options menu, with explicit validation for blank optional fields and invalid shortcut/layout bindings. | Expand schema usage across remaining menu/settings pages. |
| `footer_nav.rs` | Compliant | Footer navigation utility systems. | None. |
| `main_menu.rs` | Compliant | Shared main-menu option-list composition + command-id mapping + overlay follow/navigation systems. | Keep main-menu behavior routed through shared menu command flow. |
| `scroll_adapter.rs` | Partial | Video/options-specific adapter logic mixed with menu selection concerns; focus-follow now reads tabbed option-lock state instead of visual-state outputs. | Push reusable pieces into scroll primitive layer. |
| `root_spawn.rs` | Partial | Root spawn now explicit (no generic bundle arg), but composition callers still attach markers manually after spawn. | Continue reducing ad-hoc caller wiring via root hook contracts where practical. |
| `page_content.rs` | Partial | Composes primitives; still monolithic and feature-dense. | Split by reusable composition units (tabs/footer/top-options/modal triggers). |
| `modal_flow.rs` | Partial | Uses shared primitives with explicit marker insertion (no bundle helper arg). | Continue splitting modal-specific layout constants from generic flow. |
| `dropdown_flow.rs` | Partial | Owner-scoped behavior mostly clean; still tied to video-option specifics in places. | Keep generic dropdown behavior in primitive module only. |
| `dropdown_view.rs` | Partial | Mostly visual sync; includes some option-page assumptions. | Keep view sync generic or isolate feature-specific mapping. |
| `tabbed_focus.rs` | Partial | Focus arbitration logic exists and is tested. | Keep arbitration rules centralized and generic across tabbed menus. |
| `tabbed_menu.rs` | Partial | Reusable model exists; still coupled to video menu index contracts. | Decouple from page-specific index constants. |
| `menu_input.rs` | Partial | Central input routing, but still broad and dense. | Continue splitting into focused handlers per intent class. |
| `video_visuals.rs` | Partial | Visual sync is robust and tested; still large and video-specific. | Keep as feature composition, but avoid behavior decisions from visual-state inputs. |
| `defs.rs` | Partial | Single source for many constants/option maps; large and mixed concerns. | Split domain option data from generic UI geometry/helpers. |
| `debug_showcase.rs` | Partial | Showcase uses primitives; still needs ongoing fit-and-finish polish. | Keep as canonical integration reference for developers. |
| `mod.rs` | Partial | Plugin wiring is stable but dense. | Continue extracting focused submodule wiring as systems grow. |
| `flow_tests.rs` | Compliant | Covers key owner/layer/dropdown/tab interactions; catches regressions. | Keep extending around real bug classes. |

## Remaining Bundle-Style API Targets

Status:
- Legacy `SystemMenuOptionBundle` has been removed.
- `spawn_menu_root(..., extra_components: Bundle)` has been removed.
- `spawn_hover_box_root(..., extra_components: impl Bundle)` has been removed.
- `spawn_video_modal_base(..., marker: impl Bundle)` has been removed.
- No remaining reusable UI helper in `systems/ui` or `systems/ui/menu` currently requires a bundle-style extension parameter.

## Prioritized Migration Backlog

1. Continue moving behavior arbitration to primitive truth components (`Hoverable`, `Clickable`, `SelectableMenu`, `Selectable`, `OptionCycler`) and keep `InteractionVisualState` visual-only.
   - Recent progress: tabbed focus arbitration and video scroll focus-follow no longer depend on option-level `InteractionVisualState` for behavior truth.
2. Reduce composition monolith size in `page_content.rs` and `menu_input.rs` via focused sub-composers/handlers.
   - Recent progress: decomposed `menu_input::handle_menu_shortcuts` into focused helper units.
   - Recent progress: extracted video scaffold composition into `spawn_video_page_scaffold` within `page_content`.
3. Expand schema-driven composition beyond main/options to remaining pages (pause/debug/video).
4. Keep extending mixed keyboard+mouse regression coverage as new menu features land.
   - Recent progress: added regression coverage for scroll-parented top-table owner resolution in video options.
   - Recent progress: added dropdown-flow regressions for scroll-aware opening and outside-click item safety.
   - Recent progress: added directional shortcut regressions in `menu_input` (right activate, left back-only, tabs-focus block).
   - Recent progress: added query-safety smoke tests that initialize command/input/dropdown/modal/video systems and fail fast on query aliasing.
   - Recent progress: added tabbed-menu query-safety smoke coverage and stale-state cleanup regression.
   - Recent progress: added menu-input suppression regressions for non-base layer gating and tabs-focus top-row visual suppression.
   - Recent progress: added debug-showcase query-safety smoke coverage for command and visual systems.
   - Recent progress: added query-safety smoke coverage for `main_menu` and `dropdown_view` systems.
   - Recent progress: added query-safety smoke coverage for `scroll_adapter` systems.
   - Recent progress: added regression coverage for stale menu-target cleanup in `stack`.
   - Recent progress: deterministic shortcut arbitration hardening landed in `menu_input`, `modal_flow`, and dropdown keyboard navigation.
   - Recent progress: shared layer owner-order helpers now drive escape/modal/dropdown owner traversal, replacing remaining ad-hoc owner sorting in those paths.
   - Recent progress: tabbed focus arbitration owner iteration now routes through shared layer-owner ordering helpers.
   - Recent progress: added regressions for deterministic directional shortcut tie-breaking and modal owner ordering.
   - Recent progress: added regression coverage ensuring dropdown keyboard-open picks a deterministic owner when multiple menus match.
   - Recent progress: added active-layer shortcut-context coverage to verify non-base menus are excluded from directional shortcut dispatch.
   - Recent progress: added owner-scoped cross-talk regressions for tab/dropdown flows and owner-scoped capture gate resolution.
5. Keep trimming dead menu/UI helpers as command/input paths are consolidated.
   - Recent progress: removed unused slider click-region constant and unused top-option cycle helper.
   - Recent progress: removed unused legacy helper `startup::system_menu::play_navigation_sound`.
