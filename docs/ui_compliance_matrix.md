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
| `hover_box.rs` | Partial | Primitive root is compliant; helper `spawn_hover_box_root(..., extra_components: impl Bundle)` still accepts bundle injection. | Replace helper bundle arg with explicit component attachment path. |
| `discrete_slider.rs` | Compliant | Primitive root + insert hook builds internal slot hierarchy. | None. |
| `scroll/mod.rs` + `scroll/*` | Partial | Reusable RTT scroll primitive and scrollbar with strong tests. | Complete remaining live-flow validation and nested-context polish. |

## Menu Composition Modules (`src/systems/ui/menu/*`)

| Module | Status | Current Pattern | Migration Target |
|---|---|---|---|
| `command_reducer.rs` | Compliant | Pure transition reducer, test-covered. | Keep pure state transitions here only. |
| `command_effects.rs` | Compliant | Side-effect boundary for reducer outputs. | Keep deterministic effect dispatch. |
| `command_flow.rs` | Compliant | Reducer + effects orchestration. | None. |
| `stack.rs` | Compliant | Menu stack/navigation state model. | None. |
| `schema.rs` | Partial | Typed schema parsing exists for main menu only. | Expand schema usage across settings/menu pages. |
| `footer_nav.rs` | Compliant | Footer navigation utility systems. | None. |
| `scroll_adapter.rs` | Partial | Video/options-specific adapter logic mixed with menu selection concerns. | Push reusable pieces into scroll primitive layer. |
| `root_spawn.rs` | Partial | Root spawn now explicit (no generic bundle arg), but composition callers still attach markers manually after spawn. | Continue reducing ad-hoc caller wiring via root hook contracts where practical. |
| `page_content.rs` | Partial | Composes primitives; still monolithic and feature-dense. | Split by reusable composition units (tabs/footer/top-options/modal triggers). |
| `modal_flow.rs` | Partial | Uses shared primitives, but local helper still takes `marker: impl Bundle`. | Replace helper with explicit component insertion flow. |
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

Current known bundle-style signatures in UI stack:

1. `src/systems/ui/hover_box.rs`
   - `spawn_hover_box_root(..., extra_components: impl Bundle)`
2. `src/systems/ui/menu/modal_flow.rs`
   - `spawn_video_modal_base(..., marker: impl Bundle)`

Status:
- Legacy `SystemMenuOptionBundle` has been removed.
- `spawn_menu_root(..., extra_components: Bundle)` has been removed.

## Prioritized Migration Backlog

1. Replace remaining `impl Bundle` helper parameters in reusable UI paths (`hover_box`, modal helper).
2. Continue moving behavior arbitration to primitive truth components (`Hoverable`, `Clickable`, `SelectableMenu`, `Selectable`, `OptionCycler`) and keep `InteractionVisualState` visual-only.
3. Reduce composition monolith size in `page_content.rs` and `menu_input.rs` via focused sub-composers/handlers.
4. Expand schema-driven composition beyond main menu so settings tabs/options are data-driven.
5. Keep extending mixed keyboard+mouse regression coverage as new menu features land.
